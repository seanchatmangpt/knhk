// knhk-kernel: Receipt generation performance benchmarks
// Measures cryptographic hashing and verification

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_kernel::{
    receipt::{Receipt, ReceiptBuilder, ReceiptStatus, ReceiptStore},
    timer::read_tsc,
};

fn bench_receipt_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_creation");

    group.bench_function("minimal_receipt", |b| {
        b.iter(|| {
            let receipt = Receipt::new(black_box(1), black_box(100));
            black_box(receipt)
        });
    });

    group.bench_function("full_receipt", |b| {
        b.iter(|| {
            let receipt = ReceiptBuilder::new(1, 100)
                .with_budget(8)
                .with_state(0, 1)
                .with_inputs(&[1, 2, 3, 4, 5])
                .with_outputs(&[10, 20, 30])
                .with_result(ReceiptStatus::Success, 5)
                .add_guard(1, true, 1)
                .add_guard(2, false, 2)
                .add_guard(3, true, 1)
                .build();

            black_box(receipt)
        });
    });

    group.bench_function("receipt_with_many_guards", |b| {
        b.iter(|| {
            let mut builder = ReceiptBuilder::new(1, 100)
                .with_budget(8)
                .with_result(ReceiptStatus::Success, 5);

            for i in 0..8 {
                builder = builder.add_guard(i, i % 2 == 0, 1);
            }

            let receipt = builder.build();
            black_box(receipt)
        });
    });

    group.finish();
}

fn bench_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_hashing");

    // Small receipt
    let small_receipt = ReceiptBuilder::new(1, 100)
        .with_inputs(&[1, 2])
        .with_result(ReceiptStatus::Success, 3)
        .build();

    // Large receipt
    let large_receipt = ReceiptBuilder::new(2, 200)
        .with_inputs(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
        .with_outputs(&[20, 21, 22, 23, 24, 25, 26, 27])
        .add_guard(1, true, 1)
        .add_guard(2, true, 1)
        .add_guard(3, false, 2)
        .add_guard(4, true, 1)
        .add_guard(5, true, 1)
        .add_guard(6, false, 2)
        .add_guard(7, true, 1)
        .add_guard(8, true, 1)
        .with_result(ReceiptStatus::Success, 6)
        .build();

    group.bench_function("hash_small", |b| {
        let mut receipt = small_receipt.clone();
        b.iter(|| {
            receipt.compute_hash();
            black_box(&receipt.receipt_hash)
        });
    });

    group.bench_function("hash_large", |b| {
        let mut receipt = large_receipt.clone();
        b.iter(|| {
            receipt.compute_hash();
            black_box(&receipt.receipt_hash)
        });
    });

    group.bench_function("verify_small", |b| {
        b.iter(|| {
            let valid = small_receipt.verify();
            black_box(valid)
        });
    });

    group.bench_function("verify_large", |b| {
        b.iter(|| {
            let valid = large_receipt.verify();
            black_box(valid)
        });
    });

    group.finish();
}

fn bench_digest_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_digest");

    let observations: Vec<Vec<u64>> = vec![
        vec![1, 2],
        vec![1, 2, 3, 4],
        vec![1, 2, 3, 4, 5, 6, 7, 8],
        (0..16).collect(),
    ];

    for obs in observations {
        let size = obs.len();
        group.bench_with_input(
            BenchmarkId::new("input_digest", size),
            &obs,
            |b, observations| {
                let mut receipt = Receipt::new(1, 100);
                b.iter(|| {
                    receipt.set_input_digest(black_box(observations));
                    black_box(receipt.input_digest)
                });
            },
        );
    }

    group.finish();
}

fn bench_receipt_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_store");
    group.throughput(Throughput::Elements(1));

    let mut store = ReceiptStore::new(10000);

    // Pre-create receipts
    let receipts: Vec<Receipt> = (0..100)
        .map(|i| {
            ReceiptBuilder::new(i, i as u64 * 100)
                .with_inputs(&[i as u64, (i + 1) as u64])
                .with_result(ReceiptStatus::Success, 4)
                .build()
        })
        .collect();

    group.bench_function("store_receipt", |b| {
        let mut idx = 0;
        b.iter(|| {
            store.store(receipts[idx].clone());
            idx = (idx + 1) % receipts.len();
        });
    });

    group.bench_function("get_by_id", |b| {
        // Pre-store some receipts
        for receipt in &receipts[..50] {
            store.store(receipt.clone());
        }

        b.iter(|| {
            let receipt = store.get_by_id(black_box(25));
            black_box(receipt)
        });
    });

    group.bench_function("get_by_task", |b| {
        b.iter(|| {
            let task_receipts = store.get_by_task(black_box(2500));
            black_box(task_receipts)
        });
    });

    group.bench_function("get_recent", |b| {
        b.iter(|| {
            let recent = store.get_recent(black_box(10));
            black_box(recent)
        });
    });

    group.finish();
}

fn bench_receipt_ticks(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_ticks");

    // Measure critical operations in CPU ticks
    group.bench_function("new_receipt_ticks", |b| {
        b.iter_custom(|iters| {
            let mut total_ticks = 0u64;

            for _ in 0..iters {
                let start = read_tsc();
                let _receipt = Receipt::new(1, 100);
                let end = read_tsc();
                total_ticks += end - start;
            }

            std::time::Duration::from_nanos(total_ticks / iters)
        });
    });

    group.bench_function("build_receipt_ticks", |b| {
        b.iter_custom(|iters| {
            let mut total_ticks = 0u64;

            for _ in 0..iters {
                let start = read_tsc();
                let _receipt = ReceiptBuilder::new(1, 100)
                    .with_budget(8)
                    .with_inputs(&[1, 2, 3])
                    .with_result(ReceiptStatus::Success, 5)
                    .build();
                let end = read_tsc();
                total_ticks += end - start;
            }

            std::time::Duration::from_nanos(total_ticks / iters)
        });
    });

    group.bench_function("hash_compute_ticks", |b| {
        let mut receipt = ReceiptBuilder::new(1, 100)
            .with_inputs(&[1, 2, 3, 4, 5])
            .build();

        b.iter_custom(|iters| {
            let mut total_ticks = 0u64;

            for _ in 0..iters {
                let start = read_tsc();
                receipt.compute_hash();
                let end = read_tsc();
                total_ticks += end - start;
            }

            std::time::Duration::from_nanos(total_ticks / iters)
        });
    });

    group.finish();
}

fn bench_receipt_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_patterns");

    // Success path receipt
    group.bench_function("success_receipt", |b| {
        b.iter(|| {
            let receipt = ReceiptBuilder::new(1, 100)
                .with_budget(8)
                .with_state(1, 5) // Ready -> Completed
                .with_inputs(&[42])
                .with_outputs(&[84])
                .with_result(ReceiptStatus::Success, 5)
                .build();

            black_box(receipt)
        });
    });

    // Failed guard receipt
    group.bench_function("guard_failed_receipt", |b| {
        b.iter(|| {
            let receipt = ReceiptBuilder::new(2, 200)
                .with_budget(8)
                .with_state(1, 4) // Ready -> Suspended
                .with_inputs(&[42])
                .add_guard(1, true, 1)
                .add_guard(2, false, 2) // Failed guard
                .with_result(ReceiptStatus::GuardFailed, 3)
                .build();

            black_box(receipt)
        });
    });

    // Budget exceeded receipt
    group.bench_function("budget_exceeded_receipt", |b| {
        b.iter(|| {
            let receipt = ReceiptBuilder::new(3, 300)
                .with_budget(8)
                .with_state(2, 6) // Running -> Failed
                .with_inputs(&[1, 2, 3, 4, 5])
                .with_result(ReceiptStatus::BudgetExceeded, 10) // > 8 ticks
                .build();

            black_box(receipt)
        });
    });

    group.finish();
}

fn bench_receipt_summary(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_summary");

    let simple = ReceiptBuilder::new(1, 100)
        .with_result(ReceiptStatus::Success, 5)
        .build();

    let complex = ReceiptBuilder::new(2, 200)
        .with_budget(8)
        .with_inputs(&[1, 2, 3, 4, 5])
        .with_outputs(&[10, 20, 30])
        .add_guard(1, true, 1)
        .add_guard(2, true, 1)
        .add_guard(3, true, 1)
        .with_result(ReceiptStatus::Success, 7)
        .build();

    group.bench_function("simple_summary", |b| {
        b.iter(|| {
            let summary = simple.summary();
            black_box(summary)
        });
    });

    group.bench_function("complex_summary", |b| {
        b.iter(|| {
            let summary = complex.summary();
            black_box(summary)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_receipt_creation,
    bench_hashing,
    bench_digest_computation,
    bench_receipt_store,
    bench_receipt_ticks,
    bench_receipt_patterns,
    bench_receipt_summary
);

criterion_main!(benches);
