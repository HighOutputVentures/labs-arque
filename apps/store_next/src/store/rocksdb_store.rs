use super::{Event, InsertEventParams, ListAggregateEventsParams, Store, StoreError};
use arque_common::event_generated::{root_as_event, EventArgs};
use byteorder::{BigEndian, ByteOrder};
use flatbuffers::FlatBufferBuilder;
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

        let aggregate_version = params.aggregate_version.to_be_bytes();

        let mut batch = WriteBatch::default();

        batch.put_cf(cf3, params.aggregate_id, &aggregate_version);

        batch.put_cf(
            cf2,
            &[params.aggregate_id, &aggregate_version].concat(),
            params.id.to_bytes(),
        );

        let fbb = {
            let mut fbb = FlatBufferBuilder::with_capacity(1024);

            let args = EventArgs {
                id: Some(fbb.create_vector(params.id.to_bytes())),
                type_: params.type_,
                aggregate_id: Some(fbb.create_vector(params.aggregate_id)),
                aggregate_version: params.aggregate_version,
                body: Some(fbb.create_vector(params.body)),
                meta: Some(fbb.create_vector(params.meta)),
                timestamp: params.timestamp,
            };

            let event = arque_common::event_generated::Event::create(&mut fbb, &args);

            fbb.finish(event, None);

            fbb
        };

        batch.put_cf(cf1, params.id.to_bytes(), fbb.finished_data());

        match self.db.write(batch) {
            Err(e) => {
                return Err(StoreError::Unexpected {
                    message: format!("unable to write data:\n{}", e),
                })
            }
            _ => (),
        };

        Ok(())
    }

    fn list_aggregate_events(
        &self,
        params: &ListAggregateEventsParams,
    ) -> Result<Vec<Event>, StoreError> {
        let cf1 = match self.db.cf_handle("events") {
            Some(cf) => cf,
            None => {
                return Err(StoreError::Unexpected {
                    message: "`events` column family should exist".to_string(),
                })
            }
        };

        let cf2 = match self.db.cf_handle("aggregate_events") {
            Some(cf) => cf,
            None => {
                return Err(StoreError::Unexpected {
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
                rocksdb::IteratorMode::From(&anchor, rocksdb::Direction::Forward),
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
                    Some(value) => {
                        let event = match root_as_event(&value) {
                            Ok(event) => event,
                            Err(_) => return None,
                        };

                        let id = match event.id() {
                            Some(id) => id,
                            None => return None,
                        };

                        let aggregate_id = match event.aggregate_id() {
                            Some(aggregate_id) => aggregate_id,
                            None => return None,
                        };

                        Some(Event {
                            id: id.to_vec(),
                            type_: event.type_(),
                            aggregate_id: aggregate_id.to_vec(),
                            aggregate_version: event.aggregate_version(),
                            data: value,
                        })
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<Event>>();

        Ok(events)
    }
}

#[cfg(test)]
#[path = "../../tests/helpers.rs"]
mod helpers;
#[cfg(test)]
mod tests {
    use super::*;
    use arque_common::EventId;
    use flatbuffers::FlatBufferBuilder;
    use helpers::{generate_fake_event, random_bytes, GenerateFakeEventArgs};
    use rstest::*;
    use tempdir::TempDir;
    // use flatbuffers::FlatBufferBuilder;
    // use helpers::{generate_fake_event, random_bytes, GenerateFakeEventArgs};

    // fn insert_event(
    //     aggregate_id: &EventId,
    //     aggregate_version: u32,
    //     store: &impl Store,
    // ) -> Result<(), StoreError> {
    //     let id = EventId::new();

    //     let mut fbb = FlatBufferBuilder::new();

    //     let args = GenerateFakeEventArgs {
    //         id: Some(id.to_bytes()),
    //         aggregate_id: Some(aggregate_id.to_bytes()),
    //         aggregate_version: Some(aggregate_version),
    //         ..GenerateFakeEventArgs::default()
    //     };

    //     let event = generate_fake_event(&mut fbb, &args);

    //     fbb.finish(event, None);

    //     let params = InsertEventParams {
    //         id: args.id.unwrap(),
    //         aggregate_id: args.aggregate_id.unwrap(),
    //         aggregate_version,
    //         data: fbb.finished_data(),
    //     };

    //     store.insert_event(&params)
    // }

    #[fixture]
    fn store() -> RocksDBStore {
        let temp_dir = TempDir::new("arque_test").unwrap();
        let db = RocksDBStore::open(temp_dir.path()).unwrap();

        db
    }

    #[rstest]
    fn test_insert_event(store: impl Store) {
        let id = EventId::new();
        let aggregate_id: [u8;12] = random_bytes(12).try_into().unwrap();
        let aggregate_version = 1;

        let mut fbb = FlatBufferBuilder::new();

        let args = GenerateFakeEventArgs {
            id,
            aggregate_id,
            aggregate_version,
            ..GenerateFakeEventArgs::default()
        };

        let event = generate_fake_event(&mut fbb, &args);
        
        let params = InsertEventParams {
            id: &id,
            type_: event.type_(),
            aggregate_id: &aggregate_id,
            aggregate_version: event.aggregate_version(),
            body: event.body().unwrap(),
            meta: event.meta().unwrap(),
            timestamp: event.timestamp(),
        };

        let result = store.insert_event(&params);

        assert!(result.is_ok());
    }

    // #[rstest]
    // #[tokio::test]
    // async fn test_insert_event_invalid_aggregate_version_error(store: impl Store) {
    //     let aggregate_id = EventId::new();

    //     for i in 1..=10 {
    //         insert_event(&aggregate_id, i, &store).unwrap();
    //     }

    //     let result = insert_event(&aggregate_id, 10, &store);
    //     assert!(
    //         result.unwrap_err()
    //             == StoreError::InvalidAggregateVersion {
    //                 current: 10,
    //                 next: 10
    //             }
    //     );
    // }

    // #[rstest]
    // #[tokio::test]
    // async fn test_list_aggregate_events(store: impl Store) {
    //     let aggregate_id = EventId::new();

    //     let insert_events = |aggregate_id: &EventId, count: usize| {
    //         for i in 1..=count {
    //             insert_event(aggregate_id, i as u32, &store).unwrap();
    //         }
    //     };

    //     insert_events(&aggregate_id, 10);
    //     for _ in 0..5 {
    //         insert_events(&EventId::new(), 10);
    //     }

    //     let params = ListAggregateEventsParams {
    //         aggregate_id: aggregate_id.to_bytes(),
    //         aggregate_version: None,
    //         limit: None,
    //     };

    //     let result = store.list_aggregate_events(&params);
    //     assert!(result.is_ok());

    //     for (event, i) in result.unwrap().into_iter().zip(1..=10) {
    //         assert_eq!(event.aggregate_version, i);
    //     }
    // }
}
