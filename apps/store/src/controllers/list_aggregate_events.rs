use super::ControllerContext;
use crate::store::ListAggregateEventsParams;
use arque_common::request_generated::{
    ListAggregateEventsRequestBody, ListAggregateEventsRequestBodyArgs,
};
use flatbuffers::FlatBufferBuilder;

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

    let mut bldr = FlatBufferBuilder::new();

    bldr.reset();

    let list_aggregate_events_request_body_args = ListAggregateEventsRequestBodyArgs {
        aggregate_id: Some(bldr.create_vector(&body.aggregate_id().unwrap())),
        aggregate_version: body.aggregate_version(),
        limit: body.limit(),
    };

    let list_aggregate_events_request_body_data =
        ListAggregateEventsRequestBody::create(&mut bldr, &list_aggregate_events_request_body_args);

    bldr.finish(list_aggregate_events_request_body_data, None);

    let data = bldr.finished_data().to_vec();

    ctx.stream
        .send(hex::encode(body.aggregate_id().unwrap()), data);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{store::MockRocksDBStore, stream::MockKafkaStream};
    use arque_common::request_generated::ListAggregateEventsRequestBodyArgs;
    use flatbuffers::FlatBufferBuilder;
    use rstest::*;
    use uuid::Uuid;

    #[rstest]
    #[tokio::test]
    async fn list_aggregate_events_request_test() {
        let mut bldr = FlatBufferBuilder::new();

        bldr.reset();

        let aggregate_id = Uuid::new_v4();

        let list_aggregate_events_request_body_args = ListAggregateEventsRequestBodyArgs {
            aggregate_id: Some(bldr.create_vector(&aggregate_id.as_bytes().as_slice())),
            aggregate_version: 1u32,
            limit: 1u32,
        };

        let list_aggregate_events_request_body_data = ListAggregateEventsRequestBody::create(
            &mut bldr,
            &list_aggregate_events_request_body_args,
        );

        bldr.finish(list_aggregate_events_request_body_data, None);

        let data = bldr.finished_data();

        let list_aggregate_events_request_body =
            flatbuffers::root::<ListAggregateEventsRequestBody>(data);

        let controller_context = ControllerContext {
            store: Box::new(MockRocksDBStore {}),
            stream: Box::new(MockKafkaStream {}),
        };

        list_aggregate_events(
            &controller_context,
            &list_aggregate_events_request_body.unwrap(),
        )
        .unwrap();
    }
}
