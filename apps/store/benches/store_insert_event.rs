#[path = "../tests/helpers_next.rs"]
mod helpers;

use arque_common::object_id::ObjectId;
use arque_store::store_next::{InsertEventParams, RocksDBStore, Store};
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
                    let id = ObjectId::new();
                    let aggregate_id = random_bytes(12);
                    let body = random_bytes(size);

                    let mut fbb = FlatBufferBuilder::new();

                    let args = GenerateFakeEventArgs {
                        id: Some(id.to_bytes()),
                        aggregate_id: Some(&aggregate_id),
                        aggregate_version: Some(1),
                        body: Some(&body),
                        ..GenerateFakeEventArgs::default()
                    };

                    let event = generate_fake_event(&mut fbb, &args);

                    fbb.finish(event, None);

                    (id, aggregate_id, fbb)
                },
                |(id, aggregate_id, fbb)| {
                    let params = InsertEventParams {
                        id: id.to_bytes(),
                        aggregate_id: &aggregate_id,
                        aggregate_version: 1,
                        data: fbb.finished_data(),
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
