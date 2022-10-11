use arque_common::event_generated::Event;
use rocksdb::Error;
use custom_error::custom_error;

custom_error!{pub InsertEventError
  InvalidAggregateVersion = "invalid aggregate version"
}

pub struct ListAggregateEventsParams<'a> {
  aggregate_id: &'a [u8],
  aggregate_version: Option<u32>
}

pub trait Store {
  fn insert_event(event: &Event) -> Result<(), InsertEventError>;
  fn list_aggregate_events(&self, params: ListAggregateEventsParams) -> Result<(), Error>; 
}
