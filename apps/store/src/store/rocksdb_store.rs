mod store;

use arque_common::event_generated::Event;
use arque_common::event_to_event_args;
use arque_common::event_to_fb;
use arque_common::fb_to_event;
use arque_common::EventArgsType;

use rocksdb::Options;
use rocksdb::{Error, WriteBatch, DB};

use chrono::Local;

use uuid::Uuid;

use byteorder::{BigEndian, ByteOrder};

use crate::store::{InsertEventError, ListAggregateEventsParams};

pub struct RocksDBStore {
    db: DB,
}

impl RocksDBStore {
    fn open(path: &str) -> Result<Self, Error> {
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

    fn insert_event(&self, event: &Event) -> Result<(), InsertEventError> {
        let mut batch = WriteBatch::default();

        let fb_event_data_bytes = event_to_fb(event_to_event_args(*event));

        let cf1 = self.db.cf_handle("events").unwrap();
        let cf2 = self.db.cf_handle("aggregate_events").unwrap();
        let cf3 = self.db.cf_handle("aggregate_version").unwrap();

        match self.db.get_pinned_cf(cf3, event.aggregate_id().unwrap()) {
            Ok(Some(aggregate_version)) => {
                if event
                    .aggregate_version()
                    .ne(&(BigEndian::read_u32(&aggregate_version.to_vec()) + 1))
                {
                    return Err(InsertEventError::InvalidAggregateVersion);
                }
            }
            Ok(None) => println!("value not found"),
            Err(e) => panic!("failed to query: {}", e),
        }

        batch.put_cf(cf1, event.id().unwrap(), fb_event_data_bytes);
        batch.put_cf(
            cf2,
            [
                event.aggregate_id().unwrap(),
                &event.aggregate_version().to_be_bytes(),
            ]
            .concat(),
            event.id().unwrap(),
        );
        batch.put_cf(
            cf3,
            event.aggregate_id().unwrap(),
            event.aggregate_version().to_be_bytes(),
        );

        self.db.write(batch).expect("failed to write");

        println!("event saved!");
        Ok(())
    }

    fn list_aggregate_events(&self, params: ListAggregateEventsParams) -> Result<(), Error> {
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

        for data in event_data {
            println!("value {:?}", data.unwrap().unwrap().to_vec())
        }

        println!("query successfull");
        Ok(())
    }

    fn close(path: &str) {
        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);

        DB::destroy(&db_opts, path).expect("failed to close");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    // #[fixture]
    // fn generate_fake_event_args(#[default(1)] aggregate_version: u32) -> EventArgsType {
    //     let event_args = EventArgsType {
    //         id: Uuid::new_v4().as_bytes().to_vec(),
    //         type_: 1,
    //         timestamp: Local::now().timestamp() as u32,
    //         aggregate_id: Uuid::new_v4().as_bytes().to_vec(),
    //         aggregate_version,
    //         body: Uuid::new_v4().as_bytes().to_vec(),
    //         metadata: Uuid::new_v4().as_bytes().to_vec(),
    //         version: 1,
    //     };

    //     event_args
    // }

    // #[fixture]
    // fn open_db(#[default("./src/db")] path: &str) -> RocksDBStore {
    //     let db = RocksDBStore::open(path).unwrap();

    //     db
    // }

    // #[rstest]
    // #[tokio::test]
    // async fn create_column_families_test(_open_db: RocksDBStore) {
    //     let data = DB::list_cf(&Options::default(), "./src/db").unwrap();
    //     assert_eq!(
    //         data,
    //         ["default", "events", "aggregate_events", "aggregate_version"]
    //     );
    // }

    // #[rstest]
    // #[tokio::test]
    // async fn insert_event_test(
    //     #[with("./src/db1")] open_db: RocksDBStore,
    //     generate_fake_event_args: EventArgsType,
    // ) {
    //     let fb_event_data_bytes = event_to_fb(generate_fake_event_args);

    //     open_db
    //         .insert_event(&fb_to_event(&fb_event_data_bytes))
    //         .expect("failed to save event");

    //     open_db.db.flush().expect("failed to flush");
    // }

    // #[rstest]
    // #[tokio::test]
    // #[should_panic(expected = "failed to save event: InvalidAggregateVersion")]
    // async fn insert_event_error_test(
    //     #[with("./src/db2")] open_db: RocksDBStore,
    //     #[with(2000000)] generate_fake_event_args: EventArgsType,
    // ) {
    //     let fb_event_data_bytes = event_to_fb(generate_fake_event_args);

    //     open_db
    //         .insert_event(&fb_to_event(&fb_event_data_bytes))
    //         .expect("failed to save event");

    //     open_db
    //         .insert_event(&fb_to_event(&fb_event_data_bytes))
    //         .expect("failed to save event");
    // }

    // #[rstest]
    // #[tokio::test]
    // async fn list_aggregate_events_test(#[with("./src/db3")] open_db: RocksDBStore) {
    //     let aggregate_id = Uuid::new_v4();

    //     for i in 0..20 {
    //         let args = EventArgsType {
    //             aggregate_id: aggregate_id.as_bytes().to_vec(),
    //             ..generate_fake_event_args(i)
    //         };

    //         let fb_event_data_bytes = event_to_fb(args);

    //         open_db
    //             .insert_event(&fb_to_event(&fb_event_data_bytes))
    //             .expect("failed to save event");
    //     }

    //     let list_aggregate_events_params = ListAggregateEventsParams {
    //         aggregate_id: aggregate_id.as_bytes(),
    //         aggregate_version: Option::Some(5),
    //         limit: 10,
    //     };

    //     open_db
    //         .list_aggregate_events(list_aggregate_events_params)
    //         .expect("failed to query");

    //     open_db.db.flush().expect("failed to flush");
    // }

    #[rstest]
    #[tokio::test]
    async fn close_db_test() {
        let path1 = "./src/db";
        let path2 = "./src/db1";
        let path3 = "./src/db2";
        let path4 = "./src/db3";

        RocksDBStore::close(path1);
        RocksDBStore::close(path2);
        RocksDBStore::close(path3);
        RocksDBStore::close(path4);
    }
}
