use arque_common::event_generated::Event;
use arque_common::event_to_event_args;
use arque_common::event_to_fb;
use arque_common::fb_to_event;
use arque_common::EventArgsType;

use rocksdb::Options;
use rocksdb::{Error, WriteBatch, DB};

use chrono::Local;

use uuid::Uuid;

mod store;
use store::InsertEventError;

//TODO
// use cargo test/ other test frameworks
// add InsertEventError to insert_event (check for valid aggragate_version)
// code clean up (no copy in flatbuffer)

pub struct RocksDBStore {
    db: DB,
}

impl RocksDBStore {
    fn open(path: &str, db_opts: &Options) -> Result<Self, Error> {
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
                println!("value from event: {}", event.aggregate_version());
                println!(
                    "value from db: {:?}",
                    String::from_utf8(aggregate_version.to_vec())
                );
                if event
                    .aggregate_version()
                    .ne(&(String::from_utf8(aggregate_version.to_vec())
                        .unwrap()
                        .to_string()
                        .parse::<u32>()
                        .unwrap()
                        + 1))
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
                event.aggregate_version().to_string().as_bytes(),
            ]
            .concat(),
            event.id().unwrap(),
        );
        batch.put_cf(
            cf3,
            event.aggregate_id().unwrap(),
            event.aggregate_version().to_string(),
        );

        self.db.write(batch).expect("failed to write");

        println!("event saved!");
        Ok(())
    }

    fn list_aggregate_events(&self) -> Result<(), Error> {
        Ok(())
    }

    fn close(path: &str, db_opts: &Options) {
        DB::destroy(&db_opts, path).expect("failed to close");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_event_test() {
        println!("Starting Test...");

        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);

        let path = "./src/db";

        let db = RocksDBStore::open(path, &db_opts).unwrap();

        let data = DB::list_cf(&Options::default(), path).unwrap();
        assert_eq!(
            data,
            ["default", "events", "aggregate_events", "aggregate_version"]
        );

        let args = EventArgsType {
            id: Uuid::new_v4().as_bytes().to_vec(),
            type_: 1,
            timestamp: Local::now().timestamp() as u32,
            aggregate_id: Uuid::new_v4().as_bytes().to_vec(),
            aggregate_version: 1,
            body: Uuid::new_v4().as_bytes().to_vec(),
            metadata: Uuid::new_v4().as_bytes().to_vec(),
            version: 1,
        };

        let fb_event_data_bytes = event_to_fb(args);

        db.insert_event(&fb_to_event(&fb_event_data_bytes))
            .expect("failed to save event");

        // db.db.flush().expect("failed to flush");

        // RocksDBStore::close(path, &db_opts);
    }
}
