use super::ControllerContext;
use crate::store::ListAggregateEventsParams;
use arque_common::request_generated::ListAggregateEventsRequestBody;

pub fn list_aggregate_events(
    ctx: &ControllerContext,
    body: &ListAggregateEventsRequestBody,
) -> Result<(), Box<dyn std::error::Error>> {
    ctx.store
        .list_aggregate_events(ListAggregateEventsParams {
            aggregate_id: body.aggregate_id().unwrap(),
            aggregate_version: Some(body.aggregate_version()),
            limit: body.limit() as usize,
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
    async fn handle_list_aggregate_events_request_test() {
        let mut bldr = FlatBufferBuilder::new();

        bldr.reset();

        let aggregate_id = Uuid::new_v4();

        let list_aggregate_events_args = ListAggregateEventsRequestBodyArgs {
            aggregate_id: Some(bldr.create_vector(&aggregate_id.as_bytes().as_slice())),
            aggregate_version: 1u32,
            limit: 1u32,
        };

        let list_aggregate_events_data =
            ListAggregateEventsRequestBody::create(&mut bldr, &list_aggregate_events_args);

        bldr.finish(list_aggregate_events_data, None);
    }
}
