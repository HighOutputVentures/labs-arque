use super::store::{ListAggregateEventsError, ListAggregateEventsParamsNext};
use super::Store;
use super::{InsertEventError, InsertEventParams, ListAggregateEventsParams};
use arque_common::event_generated::Event;
use rocksdb::Error;

pub struct MockRocksDBStore;

impl Store for MockRocksDBStore {
    fn insert_event(&self, _params: InsertEventParams) -> Result<(), InsertEventError> {
        println!("insert_event");
        Ok(())
    }

    fn list_aggregate_events(
        &self,
        _params: ListAggregateEventsParams,
    ) -> Result<Vec<Vec<u8>>, Error> {
        println!("list_aggregate_events");
        Ok(vec![])
    }
}
