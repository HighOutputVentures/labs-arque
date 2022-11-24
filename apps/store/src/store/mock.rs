use arque_common::event_generated::Event;
use rocksdb::{Error};
use super::Store;
use super::store::{ListAggregateEventsParamsNext, ListAggregateEventsError};
use super::{InsertEventError, InsertEventParams, ListAggregateEventsParams};

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

    fn list_aggregate_events_next<'a>(
        &self,
        _params: &ListAggregateEventsParamsNext,
    ) -> Result<Vec<Event<'a>>, ListAggregateEventsError> {
        println!("list_aggregate_events_next");
        Ok(vec![])
    }
}
