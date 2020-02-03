#[macro_use]
extern crate criterion;
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
extern crate getrandom;
use getrandom::getrandom;
extern crate xxhash;
use xxhash::bits64::XXHash64;

use std::hash::Hasher;

fn bench_local_64_streaming(c: &mut Criterion) {
    let mut data = [0u8; 32786];
    getrandom(data.as_mut()).unwrap();

    let mut group = c.benchmark_group("xxhash64_streaming_input_size");
    for size in vec![1, 4, 8, 16, 32usize] {
        let sized = size * 1024usize;
        let slice: &[u8] = &data[0..sized];
        group.throughput(Throughput::Bytes(sized as u64));
        group.bench_with_input(BenchmarkId::from_parameter(sized), &slice, |b, data| {
            b.iter(|| {
                let mut x = XXHash64::default();
                x.write(data);
                let _ = black_box(x.finish());
            });
        });
    }
}
criterion_group!(benches, bench_local_64_streaming);
criterion_main!(benches);
