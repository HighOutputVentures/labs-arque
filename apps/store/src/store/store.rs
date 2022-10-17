use arque_common::event_generated::Event;
use custom_error::custom_error;
use rocksdb::Error;

custom_error! {pub InsertEventError
  InvalidAggregateVersion = "invalid aggregate version"
}

pub struct ListAggregateEventsParams<'a> {
    pub aggregate_id: &'a [u8],
    pub aggregate_version: Option<u32>,
    pub limit: usize,
}

pub trait Store {
    fn insert_event(&self, event: &Event) -> Result<(), InsertEventError>;
    fn list_aggregate_events(&self, params: ListAggregateEventsParams)
        -> Result<Vec<Event>, Error>;
}
