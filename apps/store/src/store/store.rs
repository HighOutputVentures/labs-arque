use arque_common::event_generated::Event;
use rocksdb::Error;
use custom_error::custom_error;

custom_error!{pub InsertEventError
  InvalidAggregateVersion = "invalid aggregate version"
}

pub trait Store {
  fn insert_event(event: &Event) -> Result<(), InsertEventError>;
  fn list_aggregate_events(&self) -> Result<(), Error>;
}
