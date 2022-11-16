use super::{InsertEventError, InsertEventParams, ListAggregateEventsParams, Store};
use byteorder::{BigEndian, ByteOrder};
use rocksdb::Options;
use rocksdb::{Error, WriteBatch, DB};
use std::path::Path;

pub struct RocksDBStore {
    db: DB,
}

impl RocksDBStore {
    pub fn open(path: &Path) -> Result<RocksDBStore, Error> {
        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);

        let db = DB::open_cf(
            &db_opts,
            path,
            vec!["events", "aggregate_events", "aggregate_version"],
        )?;

        Ok(RocksDBStore { db })
    }
}

impl Store for RocksDBStore {
    fn insert_event(&self, params: InsertEventParams) -> Result<(), InsertEventError> {
        let mut batch = WriteBatch::default();

        let cf1 = self.db.cf_handle("events").unwrap();
        let cf2 = self.db.cf_handle("aggregate_events").unwrap();
        let cf3 = self.db.cf_handle("aggregate_version").unwrap();

        match self.db.get_pinned_cf(cf3, params.aggregate_id) {
            Ok(Some(aggregate_version)) => {
                if params
                    .aggregate_version
                    .ne(&(BigEndian::read_u32(&aggregate_version) + 1))
                {
                    return Err(InsertEventError::InvalidAggregateVersion);
                }
            }
            Ok(None) => println!("value not found"),
            Err(e) => panic!("failed to query: {}", e),
        }

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

        self.db.write(batch).expect("failed to write");

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
    use arque_common::request_generated::Event;
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

        let aggregate_id = Uuid::new_v4();
        let id = Uuid::new_v4();

        let params = InsertEventParams {
            aggregate_id: aggregate_id.as_bytes(),
            id: id.as_bytes(),
            payload: &fbb.finished_data().to_vec(),
            aggregate_version: 1,
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
