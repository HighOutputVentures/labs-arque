use arque_common::event_generated::Event;
use arque_common::event_to_event_args;
use arque_common::event_to_fb;
use arque_common::fb_to_event;
use arque_common::EventArgsType;

use rocksdb::ColumnFamily;
use rocksdb::Options;
use rocksdb::{Error, WriteBatch, DB};

use chrono::Local;

use uuid::Uuid;




pub struct RocksDBStore {
    db: DB,
}

impl RocksDBStore {
    fn open(path: &str, db_opts: &Options) -> Result<Self, Error> {
        let db = DB::open_cf(&db_opts, path, vec!["events", "aggregate_events"])?;

        Ok(RocksDBStore { db })
    }

    fn insert_event(&self, event: &Event, cfs: Vec<&ColumnFamily>) -> Result<(), Error> {
        let mut fb_event_data_bytes = Vec::<u8>::new();
        event_to_fb(event_to_event_args(*event), &mut fb_event_data_bytes);

        let mut batch = WriteBatch::default();
        batch.put_cf(cfs[0], event.id().unwrap(), fb_event_data_bytes);
        batch.put_cf(
            cfs[1],
            [
                event.aggregate_id().unwrap(),
                event.aggregate_version().to_string().as_bytes(),
            ]
            .concat(),
            event.id().unwrap(),
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

fn main() {
    println!("Starting Test...");

    let mut db_opts = Options::default();
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);

    let path = "./src/db";

    let db = RocksDBStore::open(path, &db_opts).unwrap();

    let cf1 = DB::cf_handle(&db.db, "events").unwrap();
    let cf2 = DB::cf_handle(&db.db, "aggregate_events").unwrap();

    let mut bytes = Vec::<u8>::new();

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

    event_to_fb(args, &mut bytes);

    db.insert_event(&fb_to_event(&bytes), vec![cf1, cf2])
        .expect("failed to save event");

    db.db.flush().expect("failed to flush");

    // RocksDBStore::close(path, &db_opts);
}
