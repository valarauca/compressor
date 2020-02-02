use std::hash::{Hash, Hasher};

#[macro_use]
extern crate criterion;
use criterion::{black_box, BenchmarkId, Criterion, Throughput};

extern crate getrandom;
use getrandom::getrandom;
extern crate xxhash;
use xxhash::bits32::{xxhash32_reference, XXHash32};
extern crate twox_hash;
use twox_hash::XxHash32;

fn bench_local_streaming(c: &mut Criterion) {
    let mut data = [0u8; 32786];
    getrandom(data.as_mut()).unwrap();

    let mut group = c.benchmark_group("xxhash32_streaming_input_size");
    for size in vec![1, 4, 8, 16, 32usize] {
        let sized = size * 1024usize;
        let slice: &[u8] = &data[0..sized];
        group.throughput(Throughput::Bytes(sized as u64));
        group.bench_with_input(BenchmarkId::from_parameter(sized), &slice, |b, data| {
            b.iter(|| {
                let mut x = XXHash32::default();
                x.write(data);
                let _ = black_box(x.finish());
            });
        });
    }
}

/*
fn bench_external_reference(c: &mut Criterion) {
    let mut data = [0u8;32786];
    getrandom(data.as_mut()).unwrap();
    let mut group = c.benchmark_group("twox_hash32_streaming_input_size");
    for size in vec![1,4,8,16,32usize] {
        let sized = size * 1024usize;
        let slice: &[u8] = &data[0..sized];
        group.throughput(Throughput::Bytes(sized as u64));
        group.bench_with_input(BenchmarkId::from_parameter(sized),&slice, |b,data| {
            b.iter(|| {
               let mut x = XxHash32::default();
               x.write(data);
               let _ = black_box(x.finish());
            });
        });
    }
}

fn bench_local_reference(c: &mut Criterion) {
    let mut data = [0u8;32786];
    getrandom(data.as_mut()).unwrap();
    let mut group = c.benchmark_group("xxhash_internal_reference_size");
    for size in vec![1,4,8,16,32usize] {
        let sized = size * 1024usize;
        let slice: &[u8] = &data[0..sized];
        group.throughput(Throughput::Bytes(sized as u64));
        group.bench_with_input(BenchmarkId::from_parameter(sized),&slice, |b,data| {
            b.iter(|| {
               let _ = black_box(xxhash32_reference(0,data));
            });
        });
    }
}
*/
criterion_group!(benches, bench_local_streaming); //bench_external_reference, bench_local_reference);
criterion_main!(benches);
