use super::ControllerContext;
use crate::store::InsertEventParams;
use arque_common::request_generated::{Event, EventArgs, InsertEventRequestBody};
use flatbuffers::FlatBufferBuilder;

pub fn insert_event(
    ctx: &ControllerContext,
    body: &InsertEventRequestBody,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = body.event().unwrap();

    let mut bldr = FlatBufferBuilder::new();

    bldr.reset();

    let event_args = EventArgs {
        id: Some(bldr.create_vector(&event.id().unwrap())),
        type_: event.type_(),
        aggregate_id: Some(bldr.create_vector(&event.aggregate_id().unwrap())),
        aggregate_version: event.aggregate_version(),
        body: Some(bldr.create_vector(&event.body().unwrap())),
        meta: Some(bldr.create_vector(&event.meta().unwrap())),
    };

    let event_data = Event::create(&mut bldr, &event_args);

    bldr.finish(event_data, None);

    let event_vec = bldr.finished_data().to_vec();

    ctx.store
        .insert_event(InsertEventParams {
            id: event.id().unwrap(),
            aggregate_id: event.aggregate_id().unwrap(),
            aggregate_version: event.aggregate_version(),
            payload: &event_vec, // this should be the entire event object
        })
        .unwrap();

    ctx.stream
        .send(hex::encode(event.aggregate_id().unwrap()), event_vec);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{store::MockRocksDBStore, stream::MockKafkaStream};
    use arque_common::request_generated::{
        Event, EventArgs, InsertEventRequestBody, InsertEventRequestBodyArgs,
    };

    use flatbuffers::FlatBufferBuilder;

    use rstest::*;
    use uuid::Uuid;

    #[rstest]
    #[tokio::test]
    async fn test_insert_event_request() {
        let mut bldr = FlatBufferBuilder::new();

        bldr.reset();

        let id = Uuid::new_v4();
        let aggregate_id = Uuid::new_v4();

        let event_args = EventArgs {
            id: Some(bldr.create_vector(&id.as_bytes().as_slice())),
            type_: 1u16,
            aggregate_id: Some(bldr.create_vector(&aggregate_id.as_bytes().as_slice())),
            aggregate_version: 1u32,
            body: Some(bldr.create_vector(&Uuid::new_v4().as_bytes().as_slice())),
            meta: Some(bldr.create_vector(&Uuid::new_v4().as_bytes().as_slice())),
        };

        let event_data = Event::create(&mut bldr, &event_args);

        let insert_event_request_body_args = InsertEventRequestBodyArgs {
            event: Some(event_data),
        };

        let insert_event_request_body_data =
            InsertEventRequestBody::create(&mut bldr, &insert_event_request_body_args);

        bldr.finish(insert_event_request_body_data, None);

        let data = bldr.finished_data();

        let insert_event_request_body = flatbuffers::root::<InsertEventRequestBody>(data);

        let controller_context = ControllerContext {
            store: Box::new(MockRocksDBStore {}),
            stream: Box::new(MockKafkaStream {}),
        };

        insert_event(&controller_context, &insert_event_request_body.unwrap()).unwrap();
    }
}
