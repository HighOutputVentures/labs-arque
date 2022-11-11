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
        metadata: Some(bldr.create_vector(&event.metadata().unwrap())),
        timestamp: event.timestamp(),
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

    ctx.stream.send(hex::encode(event.aggregate_id().unwrap()), event_vec);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        store::{RocksDBStore, Store},
        stream::KafkaStream,
    };
    use arque_common::request_generated::{
        Event, EventArgs, InsertEventRequestBody, InsertEventRequestBodyArgs,
    };
    use chrono::Local;
    use flatbuffers::FlatBufferBuilder;
    use rstest::*;
    use uuid::Uuid;

    #[fixture]
    fn open_db(#[default("./src/db")] path: &str) -> RocksDBStore {
        let db = RocksDBStore::open(path).unwrap();

        db
    }

    #[rstest]
    #[tokio::test]
    async fn insert_event_request_test(#[with("./src/db1")] open_db: RocksDBStore) {
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
            metadata: Some(bldr.create_vector(&Uuid::new_v4().as_bytes().as_slice())),
            timestamp: Local::now().timestamp() as u32,
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

        let stream = KafkaStream {
            broker: "localhost:9092".to_string(),
        };

        let controller_context = ControllerContext {
            store: Box::new(open_db),
            stream: Box::new(stream),
        };

        insert_event(&controller_context, &insert_event_request_body.unwrap()).unwrap();
    }
}
