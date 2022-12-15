use super::store::{ListAggregateEventsError, ListAggregateEventsParamsNext};
use super::{InsertEventError, InsertEventParams, ListAggregateEventsParams, Store};
use byteorder::{BigEndian, ByteOrder};
use rocksdb::Options;
use rocksdb::{DBPinnableSlice, Error, WriteBatch, DB};
use std::path::Path;

pub struct RocksDBStore {
    db: DB,
}

impl RocksDBStore {
    pub fn open(path: &Path) -> Result<RocksDBStore, Error> {
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

    fn list_aggregate_events_next(
        &self,
        params: &ListAggregateEventsParamsNext,
    ) -> Result<Vec<DBPinnableSlice>, ListAggregateEventsError> {
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
                Ok(value) => value,
                _ => None,
            })
            .collect::<Vec<DBPinnableSlice>>();

        Ok(events)
    }
}

impl Store for RocksDBStore {
    fn insert_event(&self, params: InsertEventParams) -> Result<(), InsertEventError> {
        let cf1 = match self.db.cf_handle("events") {
            Some(cf) => cf,
            None => {
                return Err(InsertEventError::Unexpected {
                    message: "`events` column family should exist".to_string(),
                })
            }
        };

        let cf2 = match self.db.cf_handle("aggregate_events") {
            Some(cf) => cf,
            None => {
                return Err(InsertEventError::Unexpected {
                    message: "`aggregate_events` column family should exist".to_string(),
                })
            }
        };

        let cf3 = match self.db.cf_handle("aggregate_versions") {
            Some(cf) => cf,
            None => {
                return Err(InsertEventError::Unexpected {
                    message: "`aggregate_version` column family should exist".to_string(),
                })
            }
        };

        let aggregate_version = match self.db.get_pinned_cf(cf3, params.aggregate_id) {
            Ok(Some(aggregate_version)) => BigEndian::read_u32(&aggregate_version),
            Ok(None) => 0,
            Err(e) => {
                return Err(InsertEventError::Unexpected {
                    message: format!("unable to retrieve `aggregate_version`:\n{}", e),
                })
            }
        };

        if params.aggregate_version != aggregate_version + 1 {
            return Err(InsertEventError::InvalidAggregateVersion);
        }

        let mut batch = WriteBatch::default();

        batch.put_cf(cf1, params.id, params.payload);
        batch.put_cf(
            cf2,
            [params.aggregate_id, &params.aggregate_version.to_be_bytes()].concat(),
            params.id,
        );
        batch.put_cf(
            cf3,
            params.aggregate_id,
            params.aggregate_version.to_be_bytes(),
        );

        match self.db.write(batch) {
            Err(e) => {
                return Err(InsertEventError::Unexpected {
                    message: format!("unable to write to database:\n{}", e),
                })
            }
            _ => (),
        };

        Ok(())
    }

    fn list_aggregate_events(
        &self,
        params: ListAggregateEventsParams,
    ) -> Result<Vec<Vec<u8>>, Error> {
        let cf1 = self.db.cf_handle("events").unwrap();
        let cf2 = self.db.cf_handle("aggregate_events").unwrap();

        let event_ids = self
            .db
            .iterator_cf(
                cf2,
                rocksdb::IteratorMode::From(
                    &[
                        params.aggregate_id,
                        &params.aggregate_version.unwrap().to_be_bytes(),
                    ]
                    .concat(),
                    rocksdb::Direction::Forward,
                ),
            )
            .take(params.limit.try_into().unwrap())
            .take_while(|aggregate_events| {
                let (key, _) = aggregate_events.as_ref().unwrap();

                let (aggregate_id_key, _) = key.as_ref().split_at(key.len() - 4);

                aggregate_id_key.eq(params.aggregate_id)
            })
            .map(|aggregate_events| {
                let (_, value) = aggregate_events.unwrap();
                value
            })
            .collect::<Vec<_>>();

        let event_data = self.db.batched_multi_get_cf(cf1, event_ids, true);

        let events = event_data
            .into_iter()
            .map(|data| data.unwrap().unwrap().to_owned())
            .collect::<Vec<Vec<u8>>>();

        Ok(events)
    }
}

#[cfg(test)]
#[path = "../../tests/helpers.rs"]
mod helpers;
#[cfg(test)]
mod tests {

    use super::*;
    use arque_common::{object_id::ObjectId, request_generated::Event};
    use flatbuffers::FlatBufferBuilder;
    use helpers::{generate_fake_event, GenerateFakeEventArgs};
    use rstest::*;
    use tempdir::TempDir;
    use uuid::Uuid;

    #[fixture]
    fn open_db() -> RocksDBStore {
        let temp_dir = TempDir::new("arque_test").unwrap();
        let db = RocksDBStore::open(temp_dir.path()).unwrap();

        db
    }

    #[rstest]
    #[tokio::test]
    async fn test_rocksdb_store_insert_event(open_db: RocksDBStore) {
        let mut fbb = FlatBufferBuilder::new();
        let args = GenerateFakeEventArgs::default();

        let event = generate_fake_event(&mut fbb, &args);
        fbb.finish(event, None);

        let id = ObjectId::new();
        let aggregate_id = ObjectId::new();

        let params = InsertEventParams {
            id: id.to_bytes(),
            aggregate_id: aggregate_id.to_bytes(),
            aggregate_version: 1,
            payload: &fbb.finished_data().to_vec(),
        };

        open_db.insert_event(params).expect("failed to save event");

        open_db.db.flush().expect("failed to flush");
    }

    #[rstest]
    #[tokio::test]
    async fn test_rockdb_store_list_aggregate_events(open_db: RocksDBStore) {
        let aggregate_id = Uuid::new_v4();

        for i in 0..20 {
            let mut fbb = FlatBufferBuilder::new();

            let mut args = GenerateFakeEventArgs::default();

            args = GenerateFakeEventArgs {
                aggregate_version: Some(i),
                ..args
            };

            let insert_event = generate_fake_event(&mut fbb, &args);
            fbb.finish(insert_event, None);

            let id = Uuid::new_v4();

            let params = InsertEventParams {
                aggregate_id: aggregate_id.as_bytes(),
                id: id.as_bytes(),
                payload: &fbb.finished_data().to_vec(),
                aggregate_version: i,
            };

            open_db.insert_event(params).expect("failed to save event");
        }

        let list_aggregate_events_params = ListAggregateEventsParams {
            aggregate_id: aggregate_id.as_bytes(),
            aggregate_version: Option::Some(5),
            limit: 10,
        };

        let event_data = open_db
            .list_aggregate_events(list_aggregate_events_params)
            .expect("failed to query");

        let events = event_data
            .iter()
            .map(|data| flatbuffers::root::<Event>(data).unwrap())
            .collect::<Vec<Event>>();

        assert_eq!(events.len(), 10);

        open_db.db.flush().expect("failed to flush");
    }
}
