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
        println!("params: {:?}", params.aggregate_id);
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
mod tests {
    use super::*;
    use arque_common::event_args_to_fb;
    use arque_common::event_generated::Event;
    use arque_common::fb_to_event;
    use arque_common::EventArgsType;
    use chrono::Local;
    use rstest::*;
    use uuid::Uuid;

    #[fixture]
    fn generate_fake_event_args(#[default(1)] aggregate_version: u32) -> EventArgsType {
        let event_args = EventArgsType {
            id: Uuid::new_v4().as_bytes().to_vec(),
            type_: 1,
            timestamp: Local::now().timestamp() as u32,
            aggregate_id: Uuid::new_v4().as_bytes().to_vec(),
            aggregate_version,
            body: Uuid::new_v4().as_bytes().to_vec(),
            metadata: Uuid::new_v4().as_bytes().to_vec(),
            version: 1,
        };

        event_args
    }

    #[fixture]
    fn open_db(#[default("./src/db")] path: &str) -> RocksDBStore {
        let db = RocksDBStore::open(Path::new(path)).unwrap();

        db
    }

    #[rstest]
    #[tokio::test]
    async fn create_column_families_test(_open_db: RocksDBStore) {
        let data = DB::list_cf(&Options::default(), "./src/db").unwrap();
        assert_eq!(
            data,
            ["default", "events", "aggregate_events", "aggregate_version"]
        );
    }

    #[rstest]
    #[tokio::test]
    async fn insert_event_test(
        #[with("./src/db1")] open_db: RocksDBStore,
        generate_fake_event_args: EventArgsType,
    ) {
        let fb_event_data_bytes = event_args_to_fb(generate_fake_event_args);
        let aggregate_id = Uuid::new_v4();
        let id = Uuid::new_v4();

        let params = InsertEventParams {
            aggregate_id: aggregate_id.as_bytes(),
            id: id.as_bytes(),
            payload: fb_event_data_bytes.as_ref(),
            aggregate_version: 1,
        };

        open_db.insert_event(params).expect("failed to save event");

        open_db.db.flush().expect("failed to flush");
    }

    #[rstest]
    #[tokio::test]
    #[should_panic(expected = "failed to save event: InvalidAggregateVersion")]
    async fn insert_event_error_test(
        #[with("./src/db2")] open_db: RocksDBStore,
        generate_fake_event_args: EventArgsType,
    ) {
        let fb_event_data_bytes = event_args_to_fb(generate_fake_event_args);
        let aggregate_id = Uuid::new_v4();
        let id = Uuid::new_v4();

        let params1 = InsertEventParams {
            aggregate_id: aggregate_id.as_bytes(),
            id: id.as_bytes(),
            payload: fb_event_data_bytes.as_ref(),
            aggregate_version: 1,
        };

        let params2 = InsertEventParams {
            aggregate_id: aggregate_id.as_bytes(),
            id: id.as_bytes(),
            payload: fb_event_data_bytes.as_ref(),
            aggregate_version: 1,
        };

        open_db.insert_event(params1).expect("failed to save event");

        open_db.insert_event(params2).expect("failed to save event");
    }

    #[rstest]
    #[tokio::test]
    async fn list_aggregate_events_test_1(#[with("./src/db3")] open_db: RocksDBStore) {
        let aggregate_id = Uuid::new_v4();

        for i in 0..20 {
            let id = Uuid::new_v4();

            let args = EventArgsType {
                aggregate_id: aggregate_id.as_bytes().to_vec(),
                ..generate_fake_event_args(i)
            };

            let fb_event_data_bytes = event_args_to_fb(args);

            let params = InsertEventParams {
                aggregate_id: aggregate_id.as_bytes(),
                id: id.as_bytes(),
                payload: fb_event_data_bytes.as_ref(),
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
            .map(|data| fb_to_event(data))
            .collect::<Vec<Event>>();

        assert_eq!(events.len(), 10);

        open_db.db.flush().expect("failed to flush");
    }

    #[rstest]
    #[tokio::test]
    async fn list_aggregate_events_test_2(#[with("./src/db4")] open_db: RocksDBStore) {
        let aggregate_id = Uuid::new_v4();

        for i in 0..20 {
            let id = Uuid::new_v4();
            let agg_id = Uuid::new_v4();

            let mut args = generate_fake_event_args(i);

            let mut params = InsertEventParams {
                aggregate_id: agg_id.as_bytes(),
                id: id.as_bytes(),
                payload: &id.as_ref().to_vec(),
                aggregate_version: i,
            };

            if i >= 10 && i < 15 {
                args = EventArgsType {
                    aggregate_id: aggregate_id.as_bytes().to_vec(),
                    ..args
                };

                params = InsertEventParams {
                    aggregate_id: aggregate_id.as_bytes(),
                    ..params
                }
            }

            let fb_event_data_bytes = event_args_to_fb(args);

            params = InsertEventParams {
                payload: fb_event_data_bytes.as_ref(),
                ..params
            };

            open_db.insert_event(params).expect("failed to save event");
        }

        let list_aggregate_events_params = ListAggregateEventsParams {
            aggregate_id: aggregate_id.as_bytes(),
            aggregate_version: Option::Some(10),
            limit: 20,
        };

        let event_data = open_db
            .list_aggregate_events(list_aggregate_events_params)
            .expect("failed to query");

        let events = event_data
            .iter()
            .map(|data| fb_to_event(data))
            .collect::<Vec<Event>>();

        assert_eq!(events.len(), 5);

        open_db.db.flush().expect("failed to flush");
    }

    // #[rstest]
    // #[tokio::test]
    // async fn close_db_test() {
    //     let path1 = "./src/db";
    //     let path2 = "./src/db1";
    //     let path3 = "./src/db2";
    //     let path4 = "./src/db3";
    //     let path5 = "./src/db4";

    //     close(path1);
    //     close(path2);
    //     close(path3);
    //     close(path4);
    //     close(path5);
    // }
}
