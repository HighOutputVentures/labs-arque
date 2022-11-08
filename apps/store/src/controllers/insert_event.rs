use crate::store::InsertEventParams;
use arque_common::request_generated::{InsertEventRequestBody};

use super::ControllerContext;

pub fn insert_event(
    ctx: &ControllerContext,
    body: &InsertEventRequestBody,
) -> Result<(), Box<dyn std::error::Error>> {
    /**
     * - call Store#insert_event
     * - forward events to Kafka
     */

    let event = body.event().unwrap();

    ctx.store
        .insert_event(InsertEventParams {
            id: event.id().unwrap(),
            aggregate_id: event.aggregate_id().unwrap(),
            aggregate_version: event.aggregate_version(),
            payload: &event.body().unwrap().to_vec(), // this should be the entire event object
        })
        .unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arque_common::request_generated::{Event,EventArgs};
    use chrono::Local;
    use flatbuffers::FlatBufferBuilder;
    use rstest::*;
    use uuid::Uuid;

    #[rstest]
    #[tokio::test]
    async fn insert_event_request_test() {
        let mut bldr = FlatBufferBuilder::new();

        bldr.reset();

        let id = Uuid::new_v4();
        let aggregate_id = Uuid::new_v4();

        let insert_event_args = EventArgs {
            id: Some(bldr.create_vector(&id.as_bytes().as_slice())),
            type_: 1u16,
            aggregate_id: Some(bldr.create_vector(&aggregate_id.as_bytes().as_slice())),
            aggregate_version: 1u32,
            body: Some(bldr.create_vector(&Uuid::new_v4().as_bytes().as_slice())),
            metadata: Some(bldr.create_vector(&Uuid::new_v4().as_bytes().as_slice())),
            timestamp: Local::now().timestamp() as u32,
        };

        let insert_event_data = Event::create(&mut bldr, &insert_event_args);

        bldr.finish(insert_event_data, None);
    }
}
