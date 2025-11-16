//! Hot-path micro-benchmarks
//!
//! These benchmarks verify that hot-path operations are extremely fast.
//! They measure CPU cycles rather than wall-clock time.

use criterion::{black_box, criterion_group, criterion_main, Criterion, measurement::WallTime};
use knhk_promotion::*;
use knhk_projections::CompiledProjections;
use std::sync::Arc;
use std::time::SystemTime;

fn setup_hot_path() {
    init_hot_path();

    // Store an initial descriptor
    let snapshot_id = [1u8; 32];
    let artifacts = Arc::new(CompiledProjections {
        snapshot_id,
        snapshot_hash: [0; 32],
        rust_models: Default::default(),
        openapi_spec: Default::default(),
        hooks_config: Default::default(),
        markdown_docs: Default::default(),
        otel_schema: Default::default(),
        compiled_at: SystemTime::now(),
    });

    let descriptor = SnapshotDescriptor::new(snapshot_id, artifacts);
    hot_path::store_descriptor(descriptor).expect("Failed to store descriptor");
}

fn bench_get_current_snapshot(c: &mut Criterion) {
    setup_hot_path();

    let mut group = c.benchmark_group("hot_path_operations");
    group.measurement_time(std::time::Duration::from_secs(2));
    group.sample_size(1000);

    group.bench_function("get_current_snapshot", |b| {
        b.iter(|| {
            // This is the critical hot-path operation
            // Must be ≤3 CPU ticks (typically 1-2 nanoseconds)
            black_box(get_current_snapshot())
        })
    });

    group.finish();
}

fn bench_load_descriptor(c: &mut Criterion) {
    setup_hot_path();

    c.bench_function("load_descriptor", |b| {
        b.iter(|| {
            // Load full descriptor (slightly more expensive than just ID)
            let descriptor = black_box(hot_path::load_current_descriptor());
            descriptor.snapshot_id()
        })
    });
}

fn bench_descriptor_access(c: &mut Criterion) {
    setup_hot_path();

    c.bench_function("descriptor_field_access", |b| {
        b.iter(|| {
            let descriptor = hot_path::load_current_descriptor();
            (
                black_box(descriptor.snapshot_id()),
                black_box(descriptor.epoch()),
            )
        })
    });
}

fn bench_hot_path_binder(c: &mut Criterion) {
    setup_hot_path();

    c.bench_function("hot_path_binder", |b| {
        b.iter(|| {
            let binder = HotPathBinder::new();
            black_box(binder.current_snapshot())
        })
    });
}

criterion_group!(
    benches,
    bench_get_current_snapshot,
    bench_load_descriptor,
    bench_descriptor_access,
    bench_hot_path_binder,
);
criterion_main!(benches);
