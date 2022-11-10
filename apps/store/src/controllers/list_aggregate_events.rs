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
    use crate::store::RocksDBStore;
    use super::*;
    use arque_common::request_generated::ListAggregateEventsRequestBodyArgs;
    use flatbuffers::FlatBufferBuilder;
    use rstest::*;
    use uuid::Uuid;
    use std::path::Path;

    #[fixture]
    fn open_db(#[default("./src/db")] path: &str) -> RocksDBStore {
        let db = RocksDBStore::open(Path::new(path)).unwrap();

        db
    }

    #[rstest]
    #[tokio::test]
    async fn list_aggregate_events_request_test(#[with("./src/db1")] open_db: RocksDBStore) {
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
            store: Box::new(open_db),
        };

        list_aggregate_events(
            &controller_context,
            &list_aggregate_events_request_body.unwrap(),
        )
        .unwrap();
    }
}
