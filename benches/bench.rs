use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let s64 = black_box("1234.5678901234567");
    let b64 = s64.as_bytes();
    c.bench_function("fast_float-f64", |b| {
        b.iter(|| black_box(fast_float::parse_float::<f64>(b64).unwrap().0))
    });
    c.bench_function("lexical_core-f64", |b| {
        b.iter(|| black_box(lexical_core::parse_partial::<f64>(b64).unwrap().0))
    });
    c.bench_function("from_str-f64", |b| {
        b.iter(|| black_box(f64::from_str(s64).unwrap()))
    });
    let s32 = black_box("12.34567890");
    let b32 = s32.as_bytes();
    c.bench_function("fast_float-f32", |b| {
        b.iter(|| black_box(fast_float::parse_float::<f32>(b32).unwrap().0))
    });
    c.bench_function("lexical_core-f32", |b| {
        b.iter(|| black_box(lexical_core::parse_partial::<f32>(b32).unwrap().0))
    });
    c.bench_function("from_str-f32", |b| {
        b.iter(|| black_box(f32::from_str(s32).unwrap()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
