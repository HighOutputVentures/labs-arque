use std::{
    cell::RefCell,
    sync::{Arc, MutexGuard},
};

use arque_common::event_generated::{Event, EventArgs};
use arque_driver::{Driver, ListAggregateEventsError, ListAggregateEventsParams};
use casual_logger::Log;
use flatbuffers::FlatBufferBuilder;
use neon::{
    prelude::*,
    types::{buffer::TypedArray, Deferred},
};
use once_cell::sync::OnceCell;
use tokio::{runtime::Runtime, sync::Mutex};

#[allow(dead_code)]
fn vec_to_js_array<'b>(v: &[u8], cx: &mut FunctionContext<'b>) -> JsResult<'b, JsArray> {
    let arr = JsArray::new(cx, v.len() as u32);

    for (i, v) in v.iter().enumerate() {
        let val = cx.number(*v);
        arr.set(cx, i as u32, val).unwrap();
    }

    Ok(arr)
}

#[allow(dead_code)]
fn js_array_to_vec<'a, C>(js_array: Handle<'a, JsArray>, cx: &mut C) -> Vec<u8>
where
    C: Context<'a>,
{
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
fn js_buffer_to_vec<'a, C>(js_buffer: Handle<'a, JsBuffer>, cx: &mut C) -> Vec<u8>
where
    C: Context<'a>,
{
    let mut buf_vec: Vec<u8> = Vec::new();
    for (_, item) in js_buffer.as_slice(cx).iter().enumerate() {
        buf_vec.push(*item);
    }

    buf_vec
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

pub struct ArqueDriver {
    driver: Arc<Mutex<Driver>>,
}

impl Finalize for ArqueDriver {
    fn finalize<'a, C: Context<'a>>(self, _: &mut C) {}
}

impl ArqueDriver {
    pub fn new(endpoint: Option<String>) -> Self {
        let driver_context = Driver::new(endpoint.unwrap_or("tcp://127.0.0.1:4000".to_string()));

        Self {
            driver: Arc::new(Mutex::new(driver_context)),
        }
    }

    pub fn insert_event<'a, C: Context<'a>>(
        &'a self,
        mut cx: C,
        deffered: Deferred,
        event_object: Handle<'a, JsObject>,
    ) -> JsResult<JsUndefined> {
        let runtime = runtime(&mut cx).unwrap();
        let channel = cx.channel();
        let driver_context = Arc::clone(&self.driver);

        let js_id: Handle<JsBuffer> = event_object.get(&mut cx, "id").unwrap();
        let js_type_: Handle<JsNumber> = event_object.get(&mut cx, "type_").unwrap();
        let js_aggregate_id: Handle<JsBuffer> = event_object.get(&mut cx, "aggregateId").unwrap();
        let js_aggregate_version: Handle<JsNumber> =
            event_object.get(&mut cx, "aggregateVersion").unwrap();
        let js_body: Handle<JsBuffer> = event_object.get(&mut cx, "body").unwrap();
        let js_meta: Handle<JsBuffer> = event_object.get(&mut cx, "meta").unwrap();

        let rust_id = js_buffer_to_vec(js_id, &mut cx);
        let rust_type_ = js_type_.value(&mut cx) as u16;
        let rust_aggregate_id = js_buffer_to_vec(js_aggregate_id, &mut cx);
        let rust_aggregate_version = js_aggregate_version.value(&mut cx) as u32;
        let rust_body = js_buffer_to_vec(js_body, &mut cx);
        let rust_meta = js_buffer_to_vec(js_meta, &mut cx);

        let idxx = rust_id.clone();

        Log::trace(&format!("main_thread: {:?}", idxx));

        runtime.spawn(async move {
            let idxx = rust_id.clone();
            Log::trace(&format!("spawn_thread: {:?}", idxx));

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

            let mut driver = driver_context.lock().await;

            let result = driver.insert_event(event).await;

            deffered.settle_with(&channel, move |mut cx| {
                //Convert a `reqwest::Error` to a JavaScript exception
                let response_status = result.or_else(|err| cx.throw_error(err.to_string()))?;

                Ok(cx.number(response_status.0))
            });
        });

        Ok(cx.undefined())
    }

    pub fn list_aggregate_events<'a, C: Context<'a>>(
        &'a self,
        mut cx: C,
        deffered: Deferred,
        list_aggregate_events_params: Handle<'a, JsObject>,
    ) -> JsResult<JsUndefined> {
        let runtime = runtime(&mut cx).unwrap();
        let channel = cx.channel();
        let driver_context = Arc::clone(&self.driver);

        let js_aggregate_id: Handle<JsBuffer> = list_aggregate_events_params
            .get(&mut cx, "aggregateId")
            .unwrap();
        let js_aggregate_version: Handle<JsNumber> = list_aggregate_events_params
            .get(&mut cx, "aggregateVersion")
            .unwrap();
        let js_limit: Handle<JsNumber> =
            list_aggregate_events_params.get(&mut cx, "limit").unwrap();

        let rust_aggregate_id = js_buffer_to_vec(js_aggregate_id, &mut cx);
        let rust_aggregate_version = js_aggregate_version.value(&mut cx) as u32;
        let rust_limit = js_limit.value(&mut cx) as u32;

        runtime.spawn(async move {
            let mut driver = driver_context.lock().await;

            let list_aggregate_events_params = ListAggregateEventsParams {
                aggregate_id: rust_aggregate_id,
                aggregate_version: Some(rust_aggregate_version),
                limit: rust_limit,
            };

            static mut BUFFER: Vec<u8> = Vec::new();

            let result = unsafe {
                driver
                    .list_aggregate_events(list_aggregate_events_params, &mut BUFFER)
                    .await
            };

            deffered.settle_with(&channel, move |mut cx| {
                //Convert a `reqwest::Error` to a JavaScript exception
                let response_data = result.or_else(|err| cx.throw_error(err.to_string()))?;

                let array: Handle<JsArray> = JsArray::new(&mut cx, response_data.len() as u32);
                for(i, data) in response_data.iter().enumerate() {
                    let js_object = JsObject::new(&mut cx);

                    let js_id_buffer =  JsBuffer::external(&mut cx, data.id().unwrap().to_vec());
                    js_object.set(&mut cx, "id",  js_id_buffer).unwrap();

                    array.set(&mut cx, i as u32, js_object).unwrap();
                   
                }


                Ok(cx.number(0u32))
            });
        });

        Ok(cx.undefined())
    }
}

pub fn js_driver_new(mut cx: FunctionContext) -> JsResult<JsBox<ArqueDriver>> {
    let endpoint = cx.argument::<JsString>(0)?;

    let arque_driver = ArqueDriver::new(Some(endpoint.value(&mut cx)));

    Ok(cx.boxed(arque_driver))
}

pub fn js_driver_insert_event<'c>(mut cx: FunctionContext<'c>) -> JsResult<JsPromise> {
    let arque_driver = cx.argument::<JsBox<ArqueDriver>>(0)?;

    let event_object = cx.argument::<JsObject>(1)?;

    let (deffered, promise) = cx.promise();

    arque_driver
        .insert_event(cx, deffered, event_object)
        .unwrap();

    Ok(promise)
}

// pub fn js_driver_list_aggregate_events<'c>(mut cx: FunctionContext<'c>) -> JsResult<JsPromise> {
//     let arque_driver = cx.argument::<JsBox<ArqueDriver>>(0)?;

//     let event_object = cx.argument::<JsObject>(1)?;

//     let (deffered, promise) = cx.promise();

//     arque_driver
//         .list_aggregate_events(cx, deffered, event_object)
//         .unwrap();

//     Ok(promise)
// }

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("driverNew", js_driver_new)?;
    cx.export_function("insertEvent", js_driver_insert_event)?;
    Ok(())
}
