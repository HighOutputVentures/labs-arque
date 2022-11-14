use rocksdb::Error;

use super::Store;
use super::{InsertEventError, InsertEventParams, ListAggregateEventsParams};

pub struct MockRocksDBStore;

impl Store for MockRocksDBStore {
    fn insert_event(&self, params: InsertEventParams) -> Result<(), InsertEventError> {
        println!("insert_event");
        Ok(())
    }

    fn list_aggregate_events(
        &self,
        params: ListAggregateEventsParams,
    ) -> Result<Vec<Vec<u8>>, Error> {
        println!("list_aggregate_events");
        Ok(vec![])
    }
}

