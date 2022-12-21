mod rocksdb_store;

use custom_error::custom_error;
use rocksdb::DBPinnableSlice;

pub use rocksdb_store::RocksDBStore;

custom_error! {
    #[derive(PartialEq)]
    pub StoreError
    InvalidAggregateVersion{current:u32, next:u32} = "invalid aggregate version: current={current},next={next}",
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
    pub id: Vec<u8>,
    pub type_: u16,
    pub aggregate_id: Vec<u8>,
    pub aggregate_version: u32,
    pub data: DBPinnableSlice<'a>,
}

pub trait Store {
    fn insert_event(&self, params: &InsertEventParams) -> Result<(), StoreError>;
    fn list_aggregate_events(
        &self,
        params: &ListAggregateEventsParams,
    ) -> Result<Vec<Event>, StoreError>;
}
