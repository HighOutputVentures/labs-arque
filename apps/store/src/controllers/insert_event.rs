use crate::store::InsertEventParams;
use arque_common::request_generated::Event;

use super::ControllerContext;

pub fn insert_event(
    ctx: &ControllerContext,
    body: &Event,
) -> Result<(), Box<dyn std::error::Error>> {
    /**
     * - call Store#insert_event
     * - forward events to Kafka
     */
    ctx.store
        .insert_event(InsertEventParams {
            id: body.id().unwrap(),
            aggregate_id: body.aggregate_id().unwrap(),
            aggregate_version: body.aggregate_version(),
            payload: &body.body().unwrap().to_vec(),
        })
        .unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arque_common::request_generated::EventArgs;
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
