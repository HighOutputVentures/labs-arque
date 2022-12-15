use super::ControllerContext;
use crate::store::InsertEventParams;
use arque_common::request_generated::{Event, EventArgs, InsertEventRequestBody};
use custom_error::custom_error;
use flatbuffers::FlatBufferBuilder;

custom_error! {pub InsertEventError
    InvalidAggregateVersion = "invalid aggregate version",
    Unknown{message:String} = "unknown: {message}"
}

pub fn insert_event(
    ctx: &ControllerContext,
    body: &InsertEventRequestBody,
) -> Result<(), InsertEventError> {
    let event = body.event().expect("event should not be None");

    let mut bldr = FlatBufferBuilder::new();

    let args = EventArgs {
        id: Some(bldr.create_vector(event.id().expect("event.id should not be None"))),
        type_: event.type_(),
        aggregate_id: Some(
            bldr.create_vector(
                event
                    .aggregate_id()
                    .expect("event.aggregate_id should not be None"),
            ),
        ),
        aggregate_version: event.aggregate_version(),
        body: Some(bldr.create_vector(event.body().expect("event.body should not be None"))),
        meta: event.meta().map(|meta| bldr.create_vector(meta)),
    };

    let event_ = Event::create(&mut bldr, &args);

    bldr.finish(event_, None);

    let params = InsertEventParams {
        id: event.id().expect("event.id should not be None"),
        aggregate_id: event
            .aggregate_id()
            .expect("event.aggregate_id should not be None"),
        aggregate_version: event.aggregate_version(),
        payload: &bldr.finished_data().to_vec(),
    };

    match ctx.store.insert_event(params) {
        Err(_) => return Err(InsertEventError::InvalidAggregateVersion),
        _ => (),
    };

    // ctx.stream
    //     .send(hex::encode(event.aggregate_id().unwrap()), event_vec);

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

        assert_eq!(
            insert_event(&controller_context, &insert_event_request_body.unwrap()).unwrap(),
            (),
            "insert_event should execute successfully"
        );
    }
}
