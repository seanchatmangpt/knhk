use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knhk_kms_advanced::simd_ops::{SimdHasher, SimdSigner};

fn bench_simd_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_hash");

    let hasher = SimdHasher::new();

    for size in [8, 16, 32, 64, 128, 256] {
        let inputs: Vec<[u8; 32]> = (0..size).map(|i| [i as u8; 32]).collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                black_box(hasher.batch_hash(black_box(&inputs)));
            });
        });
    }

    group.finish();
}

fn bench_simd_sign(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_sign");

    let key = [42u8; 32];
    let signer = SimdSigner::<64>::new(key);

    for size in [8, 16, 32, 64, 128, 256] {
        let messages: Vec<[u8; 32]> = (0..size).map(|i| [i as u8; 32]).collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                black_box(signer.batch_sign(black_box(&messages)).unwrap());
            });
        });
    }

    group.finish();
}

fn bench_simd_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_verify");

    let key = [42u8; 32];
    let signer = SimdSigner::<64>::new(key);

    for size in [8, 16, 32, 64, 128, 256] {
        let messages: Vec<[u8; 32]> = (0..size).map(|i| [i as u8; 32]).collect();
        let signatures = signer.batch_sign(&messages).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                black_box(
                    signer
                        .batch_verify(black_box(&messages), black_box(&signatures))
                        .unwrap(),
                );
            });
        });
    }

    group.finish();
}

fn bench_simd_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_compare");

    let hasher = SimdHasher::new();

    for size in [8, 16, 32, 64, 128, 256] {
        let left: Vec<[u8; 32]> = (0..size).map(|i| [i as u8; 32]).collect();
        let right: Vec<[u8; 32]> = (0..size).map(|i| [i as u8; 32]).collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                black_box(hasher.batch_compare(black_box(&left), black_box(&right)));
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_simd_hash,
    bench_simd_sign,
    bench_simd_verify,
    bench_simd_compare
);
criterion_main!(benches);
