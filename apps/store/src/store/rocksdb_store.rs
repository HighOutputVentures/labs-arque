use rocksdb::{Error, DB};

use super::InsertEventError;

pub struct RocksDBStore {
  db: DB,
}

impl RocksDBStore {
  fn open(path: &str) -> Result<Self, Error> {
    let db = DB::open_default(path)?;

    Ok(RocksDBStore { db })
  }

  fn insert_event(&self) -> Result<(), InsertEventError> {
    Ok(())
  }

  fn list_aggregate_events(&self) -> Result<(), Error> {
    Ok(())
  }
}
