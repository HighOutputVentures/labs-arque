use arque_common::{
    request_generated::{Event, EventArgs},
    response_generated::ResponseStatus,
};
use arque_driver::Driver;
use flatbuffers::FlatBufferBuilder;
use neon::prelude::*;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

#[allow(dead_code)]
fn vec_to_js_array<'b>(v: &[u8], cx: &mut FunctionContext<'b>) -> JsResult<'b, JsArray> {
    let arr = JsArray::new(cx, v.len() as u32);

    for (i, v) in v.iter().enumerate() {
        let val = cx.number(*v);
        arr.set(cx, i as u32, val).unwrap();
    }

    Ok(arr)
}

fn js_array_to_vec<'a, 'b>(js_array: Handle<'a, JsArray>, cx: &mut FunctionContext<'b>) -> Vec<u8> {
    let js_array_vec = js_array.to_vec(cx).unwrap();

    let mut vecx: Vec<u8> = Vec::new();

    for (_, item) in js_array_vec.iter().enumerate() {
        let id_down = item.downcast::<JsNumber, _>(cx).unwrap();
        let id_value = id_down.value(cx) as u8;
        vecx.push(id_value);
    }

    vecx
}

#[allow(dead_code)]
fn event_to_js_object<'a>(e: &Event, cx: &mut FunctionContext<'a>) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    let id = vec_to_js_array(e.id().unwrap(), cx).unwrap();
    obj.set(cx, "id", id).unwrap();

    let type_ = cx.number(e.type_());
    obj.set(cx, "type", type_).unwrap();

    let aggregate_id = vec_to_js_array(e.aggregate_id().unwrap(), cx).unwrap();
    obj.set(cx, "aggregate_id", aggregate_id).unwrap();

    let aggregate_version = cx.number(e.aggregate_version());
    obj.set(cx, "aggregate_version", aggregate_version).unwrap();

    let body = vec_to_js_array(e.body().unwrap(), cx).unwrap();
    obj.set(cx, "body", body).unwrap();

    let meta = vec_to_js_array(e.meta().unwrap(), cx).unwrap();
    obj.set(cx, "meta", meta).unwrap();

    Ok(obj)
}

fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
}

struct ArqueDriver {
    driver: Driver,
}

impl Finalize for ArqueDriver {
    fn finalize<'a, C: Context<'a>>(self, _: &mut C) {}
}

impl ArqueDriver {
    pub fn new(endpoint: Option<String>) -> Self {
        let driver_context = Driver::new(endpoint.unwrap_or("tcp://127.0.0.1:4000".to_string()));

        Self {
            driver: driver_context,
        }
    }

    // pub async fn insert_event<'a>(
    //     &mut self,
    //     event: Event<'a>,
    // ) -> Result<ResponseStatus, Box<dyn std::error::Error + Send>> {
    //     let driver = &mut  self.driver.lock().await;
    //     driver.insert_event(event).await
    // }
}

impl ArqueDriver {
    pub fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<ArqueDriver>> {
        let arque_driver = ArqueDriver::new(None);

        Ok(cx.boxed(arque_driver))
    }

    pub fn js_insert_event<'c>(mut cx: FunctionContext<'c>) -> JsResult<JsPromise> {
        // let arque_driver = cx.argument::<JsBox<ArqueDriver>>(0)?;

        // let arque_driver = (&**cx.argument::<JsBox<ArqueDriver>>(0)?).clone();
        // let mut driver = arque_driver.driver.unwrap();

        let event_object = cx.argument::<JsObject>(1)?;

        let js_id: Handle<JsArray> = event_object.get(&mut cx, "id")?;
        let js_type_: Handle<JsNumber> = event_object.get(&mut cx, "type_")?;
        let js_aggregate_id: Handle<JsArray> = event_object.get(&mut cx, "aggregateId")?;
        let js_aggregate_version: Handle<JsNumber> =
            event_object.get(&mut cx, "aggregateVersion")?;
        let js_body: Handle<JsArray> = event_object.get(&mut cx, "body")?;
        let js_meta: Handle<JsArray> = event_object.get(&mut cx, "meta")?;

        let rust_id = js_array_to_vec(js_id, &mut cx);
        let rust_type_ = js_type_.value(&mut cx) as u16;
        let rust_aggregate_id = js_array_to_vec(js_aggregate_id, &mut cx);
        let rust_aggregate_version = js_aggregate_version.value(&mut cx) as u32;
        let rust_body = js_array_to_vec(js_body, &mut cx);
        let rust_meta = js_array_to_vec(js_meta, &mut cx);

        let (deffered, promise) = cx.promise();

        let runtime = runtime(&mut cx)?;
        let channel = cx.channel();

        // temporary
        let arque_driver = ArqueDriver::new(None);
        let mut driver = arque_driver.driver;

        // Spawn a thread to complete the execution. This will _not_ block the
        // JavaScript event loop.
        runtime.spawn(async move {
            let mut fbb = FlatBufferBuilder::new();

            let event_args = EventArgs {
                id: Some(fbb.create_vector(&rust_id)),
                type_: rust_type_,
                aggregate_id: Some(fbb.create_vector(&rust_aggregate_id)),
                aggregate_version: rust_aggregate_version,
                body: Some(fbb.create_vector(&rust_body)),
                meta: Some(fbb.create_vector(&rust_meta)),
            };

            let event_body = Event::create(&mut fbb, &event_args);

            fbb.finish(event_body, None);

            let event_data = fbb.finished_data();

            let event = flatbuffers::root::<Event>(event_data).unwrap();

            let result = driver.insert_event(event).await;

            deffered.settle_with(&channel, move |mut cx| {
                //Convert a `reqwest::Error` to a JavaScript exception
                let response_status = result.or_else(|err| cx.throw_error(err.to_string()))?;

                match response_status {
                    // Resolve the promise with the release date
                    ResponseStatus::Ok => Ok(cx.number(0)),
                    _ => Ok(cx.number(-1)),
                }
                // Ok(cx.number(0))
            });
        });

        // deffered.resolve(&mut cx, idx_js_array.unwrap());

        Ok(promise)
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("driverNew", ArqueDriver::js_new)
        .unwrap();
    cx.export_function("insertEvent", ArqueDriver::js_insert_event)
        .unwrap();
    Ok(())
}
