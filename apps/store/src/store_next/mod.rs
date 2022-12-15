mod rocksdb_store;

use custom_error::custom_error;
use rocksdb::DBPinnableSlice;

pub use rocksdb_store::RocksDBStore;

custom_error! {pub StoreError
    InvalidAggregateVersion{current:u32, next:u32} = "invalid aggregate version: current={current},next={next}",
    Unexpected{message:String} = "unexpected: {message}"
}

custom_error! {pub InsertEventError
    InvalidAggregateVersion = "invalid aggregate version",
    Unexpected{message:String} = "unexpected: {message}"
}

custom_error! {pub ListAggregateEventsError
    Unexpected{message:String} = "unexpected: {message}"
}

pub struct InsertEventParams<'a> {
    pub id: &'a [u8],
    pub aggregate_id: &'a [u8],
    pub aggregate_version: u32,
    pub data: &'a [u8],
}

pub struct ListAggregateEventsParams<'a> {
    pub aggregate_id: &'a [u8],
    pub aggregate_version: Option<u32>,
    pub limit: Option<usize>,
}

pub struct Event<'a> {
    buf: DBPinnableSlice<'a>,
}

pub trait Store {
    fn insert_event(&self, params: &InsertEventParams) -> Result<(), StoreError>;
    fn list_aggregate_events(
        &self,
        params: &ListAggregateEventsParams,
    ) -> Result<Vec<Event>, ListAggregateEventsError>;
}
