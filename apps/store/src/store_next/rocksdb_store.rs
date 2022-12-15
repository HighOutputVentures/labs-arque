use super::{
    Event, InsertEventError, InsertEventParams, ListAggregateEventsError,
    ListAggregateEventsParams, Store, StoreError,
};
use byteorder::{BigEndian, ByteOrder};
use rocksdb::{ColumnFamily, Options, WriteBatch, DB};
use std::path::Path;

pub struct RocksDBStore {
    db: DB,
}

impl RocksDBStore {
    pub fn open(path: &Path) -> Result<RocksDBStore, rocksdb::Error> {
        let mut opts = Options::default();
        opts.create_missing_column_families(true);
        opts.create_if_missing(true);

        let db = DB::open_cf(
            &opts,
            path,
            vec!["events", "aggregate_events", "aggregate_versions"],
        )?;

        Ok(RocksDBStore { db })
    }

    fn get_cf(&self, cf: &str) -> Result<&ColumnFamily, StoreError> {
        match self.db.cf_handle(cf) {
            Some(cf) => Ok(cf),
            None => Err(StoreError::Unexpected {
                message: format!("column family `{}` should exist", cf),
            }),
        }
    }
}

impl Store for RocksDBStore {
    fn insert_event(&self, params: &InsertEventParams) -> Result<(), StoreError> {
        let cf1 = self.get_cf("events")?;
        let cf2 = self.get_cf("aggregate_events")?;
        let cf3 = self.get_cf("aggregate_versions")?;

        let aggregate_version = match self.db.get_pinned_cf(cf3, params.aggregate_id) {
            Ok(Some(aggregate_version)) => BigEndian::read_u32(&aggregate_version),
            Ok(None) => 0,
            Err(e) => {
                return Err(StoreError::Unexpected {
                    message: format!("unable to retrieve `aggregate_version`:\n{}", e),
                })
            }
        };

        if params.aggregate_version != aggregate_version + 1 {
            return Err(StoreError::InvalidAggregateVersion {
                current: aggregate_version,
                next: params.aggregate_version,
            });
        }

        let mut batch = WriteBatch::default();

        let aggregate_version = params.aggregate_version.to_be_bytes();

        batch.put_cf(cf1, params.id, params.data);
        batch.put_cf(
            cf2,
            [params.aggregate_id, &aggregate_version].concat(),
            params.id,
        );
        batch.put_cf(
            cf3,
            params.aggregate_id,
            &aggregate_version,
        );

        match self.db.write(batch) {
            Err(e) => {
                return Err(StoreError::Unexpected {
                    message: format!("unable to write to database:\n{}", e),
                })
            }
            _ => (),
        };

        Ok(())
    }

    fn list_aggregate_events(
        &self,
        params: &ListAggregateEventsParams,
    ) -> Result<Vec<Event>, ListAggregateEventsError> {
        let cf1 = match self.db.cf_handle("events") {
            Some(cf) => cf,
            None => {
                return Err(ListAggregateEventsError::Unexpected {
                    message: "`events` column family should exist".to_string(),
                })
            }
        };

        let cf2 = match self.db.cf_handle("aggregate_events") {
            Some(cf) => cf,
            None => {
                return Err(ListAggregateEventsError::Unexpected {
                    message: "`aggregate_events` column family should exist".to_string(),
                })
            }
        };

        let aggregate_version = params.aggregate_version.unwrap_or(0);

        let anchor = [params.aggregate_id, &aggregate_version.to_be_bytes()].concat();

        let ids = self
            .db
            .iterator_cf(
                cf2,
                rocksdb::IteratorMode::From(anchor.as_slice(), rocksdb::Direction::Forward),
            )
            .take(params.limit.unwrap_or(100))
            .take_while(|item| match item {
                Ok((key, _)) => key.starts_with(params.aggregate_id),
                Err(_) => false,
            })
            .filter_map(|item| match item {
                Ok((_, value)) => Some(value),
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        let events = self
            .db
            .batched_multi_get_cf(cf1, ids, true)
            .into_iter()
            .filter_map(|item| match item {
                Ok(value) => match value {
                    Some(value) => Some(Event { buf: value }),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<Event>>();

        Ok(events)
    }
}

#[cfg(test)]
#[path = "../../tests/helpers_next.rs"]
mod helpers;
#[cfg(test)]
mod tests {
    use super::*;
    use arque_common::object_id::ObjectId;
    use flatbuffers::FlatBufferBuilder;
    use helpers::{generate_fake_event, GenerateFakeEventArgs};
    use rstest::*;
    use tempdir::TempDir;

    #[fixture]
    fn store() -> RocksDBStore {
        let temp_dir = TempDir::new("arque_test").unwrap();
        let db = RocksDBStore::open(temp_dir.path()).unwrap();

        db
    }

    #[rstest]
    #[tokio::test]
    async fn test_insert_event(store: impl Store) {
        let id = ObjectId::new();
        let aggregate_id = ObjectId::new();

        let mut fbb = FlatBufferBuilder::new();

        let args = GenerateFakeEventArgs {
            id: Some(id.to_bytes()),
            aggregate_id: Some(aggregate_id.to_bytes()),
            ..GenerateFakeEventArgs::default()
        };

        let event = generate_fake_event(&mut fbb, &args);

        fbb.finish(event, None);

        let params = InsertEventParams {
            id: id.to_bytes(),
            aggregate_id: aggregate_id.to_bytes(),
            aggregate_version: args.aggregate_version.unwrap(),
            data: fbb.finished_data(),
        };

        let result = store.insert_event(&params);

        assert!(result.is_ok());
    }

    #[rstest]
    #[tokio::test]
    async fn test_insert_event_invalid_aggregate_version_error(store: impl Store) {
        let aggregate_id = ObjectId::new();

        let insert_event = |aggregate_version: u32| {
            let id = ObjectId::new();

            let mut fbb = FlatBufferBuilder::new();

            let args = GenerateFakeEventArgs {
                id: Some(id.to_bytes()),
                aggregate_id: Some(aggregate_id.to_bytes()),
                aggregate_version: Some(aggregate_version),
                ..GenerateFakeEventArgs::default()
            };

            let event = generate_fake_event(&mut fbb, &args);

            fbb.finish(event, None);

            let params = InsertEventParams {
                id: args.id.unwrap(),
                aggregate_id: args.aggregate_id.unwrap(),
                aggregate_version,
                data: fbb.finished_data(),
            };

            store.insert_event(&params)
        };

        let result = insert_event(1);
        assert!(result.is_ok());

        let result = insert_event(1);
        assert!(result.is_err());
    }
}
