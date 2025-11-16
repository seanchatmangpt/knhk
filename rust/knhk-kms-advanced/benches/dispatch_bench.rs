use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knhk_kms_advanced::provider_dispatch::{
    AwsProvider, AzureProvider, KmsManager, LocalProvider, VaultProvider,
};

fn bench_provider_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("provider_dispatch");

    let message = b"test message for signing";

    // AWS Provider
    let aws_manager = KmsManager::<AwsProvider>::new("test-key");
    group.bench_function("aws_sign", |b| {
        b.iter(|| {
            black_box(aws_manager.sign(black_box(message)).unwrap());
        });
    });

    // Azure Provider
    let azure_manager = KmsManager::<AzureProvider>::new("test-key");
    group.bench_function("azure_sign", |b| {
        b.iter(|| {
            black_box(azure_manager.sign(black_box(message)).unwrap());
        });
    });

    // Vault Provider
    let vault_manager = KmsManager::<VaultProvider>::new("test-key");
    group.bench_function("vault_sign", |b| {
        b.iter(|| {
            black_box(vault_manager.sign(black_box(message)).unwrap());
        });
    });

    // Local Provider
    let local_manager = KmsManager::<LocalProvider>::new("test-key");
    group.bench_function("local_sign", |b| {
        b.iter(|| {
            black_box(local_manager.sign(black_box(message)).unwrap());
        });
    });

    group.finish();
}

fn bench_batch_sign_by_provider(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_sign_by_provider");

    let messages: Vec<&[u8]> = (0..100)
        .map(|i| {
            let msg: &'static [u8] = Box::leak(vec![i as u8; 32].into_boxed_slice());
            msg
        })
        .collect();

    // AWS Provider
    let aws_manager = KmsManager::<AwsProvider>::new("test-key");
    group.bench_function("aws_batch", |b| {
        b.iter(|| {
            black_box(aws_manager.batch_sign(black_box(&messages)).unwrap());
        });
    });

    // Azure Provider
    let azure_manager = KmsManager::<AzureProvider>::new("test-key");
    group.bench_function("azure_batch", |b| {
        b.iter(|| {
            black_box(azure_manager.batch_sign(black_box(&messages)).unwrap());
        });
    });

    group.finish();
}

fn bench_encryption_by_provider(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption_by_provider");

    let plaintext = b"secret data to encrypt";

    // AWS Provider
    let aws_manager = KmsManager::<AwsProvider>::new("test-key");
    group.bench_function("aws_encrypt", |b| {
        b.iter(|| {
            black_box(aws_manager.encrypt(black_box(plaintext)).unwrap());
        });
    });

    // Azure Provider
    let azure_manager = KmsManager::<AzureProvider>::new("test-key");
    group.bench_function("azure_encrypt", |b| {
        b.iter(|| {
            black_box(azure_manager.encrypt(black_box(plaintext)).unwrap());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_provider_dispatch,
    bench_batch_sign_by_provider,
    bench_encryption_by_provider
);
criterion_main!(benches);
