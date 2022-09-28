use rocksdb::{Error, DB};
use arque_common::event_generated::Event;
use super::InsertEventError;

pub struct RocksDBStore {
  db: DB,
}

impl RocksDBStore {
  fn open(path: &str) -> Result<Self, Error> {
    let db = DB::open_default(path)?;

    Ok(RocksDBStore { db })
  }

  fn insert_event(&self, event: &Event) -> Result<(), InsertEventError> {
    event.aggregate_id();

  // events
  //   key: event id
  //   value: event
  
  // aggregate_events
  //   key: aggregate_id + aggregate_version
  //   value: event id

    Ok(())
  }

  fn list_aggregate_events(&self) -> Result<(), Error> {
    Ok(())
  }
}

