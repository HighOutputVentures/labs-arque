use neon::prelude::*;

use arque_common::event_generated::Event;
use arque_common::{event_args_to_fb, fb_to_event, EventArgsType};

fn vec_to_js_array<'b>(v: &[u8], cx: &mut FunctionContext<'b>) -> JsResult<'b, JsArray> {
    let arr = JsArray::new(cx, v.len() as u32);

    for (i, v) in v.iter().enumerate() {
        let val = cx.number(*v);
        arr.set(cx, i as u32, val).unwrap();
    }

    Ok(arr)
}

fn event_to_js_object<'a>(e: &Event, cx: &mut FunctionContext<'a>) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    let id = vec_to_js_array(e.id().unwrap(), cx).unwrap();
    obj.set(cx, "id", id).unwrap();

    let type_ = cx.number(e.type_());
    obj.set(cx, "type", type_).unwrap();

    let timestamp = cx.number(e.timestamp());
    obj.set(cx, "timestamp", timestamp).unwrap();

    let aggregate_id = vec_to_js_array(e.aggregate_id().unwrap(), cx).unwrap();
    obj.set(cx, "aggregate_id", aggregate_id).unwrap();

    let aggregate_version = cx.number(e.aggregate_version());
    obj.set(cx, "aggregate_version", aggregate_version).unwrap();

    let body = vec_to_js_array(e.body().unwrap(), cx).unwrap();
    obj.set(cx, "body", body).unwrap();

    let metadata = vec_to_js_array(e.metadata().unwrap(), cx).unwrap();
    obj.set(cx, "metadata", metadata).unwrap();

    let version = cx.number(e.version());
    obj.set(cx, "version", version).unwrap();

    Ok(obj)
}

fn insert_event<'c>(mut cx: FunctionContext<'c>) -> JsResult<JsPromise> {
    let event_args = EventArgsType {
        id: vec![1, 2, 3, 4, 5],
        type_: 1,
        timestamp: 12345 as u32,
        aggregate_id: vec![1, 2, 3, 4, 5],
        aggregate_version: 1,
        body: vec![1, 2, 3, 4, 5],
        metadata: vec![1, 2, 3, 4, 5],
        version: 1,
    };

    let event_data = event_args_to_fb(event_args);

    let event = fb_to_event(&event_data);

    println!("{:?}", event);

    let event_object = event_to_js_object(&event, &mut cx).unwrap();

    let (deffered, promise) = cx.promise();

    deffered.resolve(&mut cx, event_object);

    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("insertEvent", insert_event).unwrap();
    Ok(())
}
