#[path = "../tests/helpers.rs"]
mod helpers;

use arque_common::{EventId, event_generated::{root_as_event_unchecked}};
use arque_store::store::{InsertEventParams, RocksDBStore, Store};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use flatbuffers::FlatBufferBuilder;
use helpers::{generate_fake_event, random_bytes, GenerateFakeEventArgs};
use tempdir::TempDir;

pub fn criterion_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new("arque").unwrap();
    let store: Box<dyn Store> = Box::new(RocksDBStore::open(temp_dir.path()).unwrap());

    let mut group = c.benchmark_group("insert_event");

    for size in [1024 as usize, 4096, 16384].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let id = EventId::new();
                    let aggregate_id: [u8;12] = random_bytes(12).try_into().unwrap();

                    let mut fbb = FlatBufferBuilder::new();

                    let args = GenerateFakeEventArgs {
                        id,
                        aggregate_id,
                        body: random_bytes(size),
                        ..GenerateFakeEventArgs::default()
                    };

                    generate_fake_event(&mut fbb, &args);

                    (id, fbb)
                },
                |(id, fbb)| {
                    let event = unsafe {
                        root_as_event_unchecked(fbb.finished_data())
                    };

                    let params = InsertEventParams {
                        id: &id,
                        type_: event.type_(),
                        aggregate_id: event.aggregate_id().unwrap(),
                        aggregate_version: 1,
                        body: event.body().unwrap(),
                        meta: event.meta().unwrap(),
                        timestamp: event.timestamp(),
                    };

                    store.insert_event(&params).unwrap();
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
