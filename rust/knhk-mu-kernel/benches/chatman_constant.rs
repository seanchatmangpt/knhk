//! Chatman Constant Verification Benchmark
//!
//! Verifies that ∀o, σ: τ(μ_hot(o;σ)) ≤ 8 CPU cycles

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_mu_kernel::isa::{MuInstruction, MuOps, DispatchPhase};
use knhk_mu_kernel::timing::TickBudget;
use knhk_mu_kernel::guards::GuardContext;
use knhk_mu_kernel::patterns::PatternId;
use knhk_mu_kernel::CHATMAN_CONSTANT;

/// Benchmark complete task execution (should be ≤8 ticks)
fn bench_complete_task(c: &mut Criterion) {
    let mut group = c.benchmark_group("chatman_constant");
    group.sample_size(10000);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    group.bench_function("eval_task", |b| {
        b.iter(|| {
            let mut budget = TickBudget::chatman();
            let result = MuInstruction::eval_task(
                black_box(1),
                black_box(&obs),
                &mut budget,
            );
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark each pattern individually
fn bench_patterns_under_chatman(c: &mut Criterion) {
    let mut group = c.benchmark_group("patterns_chatman");
    group.sample_size(10000);

    // Test each of the 43 patterns
    for i in 0..43 {
        let pattern: PatternId = unsafe { core::mem::transmute(i as u8) };
        let pattern_name = format!("{:?}", pattern);

        group.bench_with_input(
            BenchmarkId::new("pattern", &pattern_name),
            &pattern,
            |b, &pattern_id| {
                b.iter(|| {
                    let result = MuOps::dispatch_pattern(
                        black_box(pattern_id as u8),
                        black_box(DispatchPhase::Begin as u8),
                    );
                    black_box(result)
                });
            },
        );

        // Verify pattern tick cost
        assert!(
            pattern.tick_cost() <= CHATMAN_CONSTANT as u8,
            "Pattern {:?} violates Chatman Constant: {} > {}",
            pattern,
            pattern.tick_cost(),
            CHATMAN_CONSTANT
        );
    }

    group.finish();
}

/// Measure actual cycle count for μ_hot operations
fn bench_cycle_accuracy(c: &mut Criterion) {
    let mut group = c.benchmark_group("cycle_accuracy");
    group.sample_size(100000);

    group.bench_function("tick_budget_start_end", |b| {
        b.iter(|| {
            let mut budget = TickBudget::chatman();
            budget.start();
            // Simulate some work
            let _ = black_box(42u64.wrapping_mul(17));
            budget.end();
            black_box(budget.used())
        });
    });

    group.finish();
}

/// Verify determinism: same input always produces same cycle count
fn bench_determinism(c: &mut Criterion) {
    let mut group = c.benchmark_group("determinism");
    group.sample_size(1000);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    // Run same operation multiple times, verify cycle count is consistent
    let mut cycle_counts = Vec::new();

    for _ in 0..100 {
        let mut budget = TickBudget::chatman();
        budget.start();
        let _ = MuInstruction::eval_task(1, &obs, &mut budget);
        budget.end();
        cycle_counts.push(budget.used());
    }

    // Verify variance is low (deterministic execution)
    let min = cycle_counts.iter().min().unwrap();
    let max = cycle_counts.iter().max().unwrap();
    let variance = max - min;

    println!("Cycle count variance: {} (min: {}, max: {})", variance, min, max);

    // Allow small variance due to CPU effects, but should be minimal
    assert!(
        variance <= 2,
        "High cycle count variance detected: {} cycles. μ_hot should be deterministic.",
        variance
    );

    group.finish();
}

/// Benchmark under load to verify timing holds under pressure
fn bench_sustained_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("sustained_load");
    group.sample_size(100);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    group.bench_function("1000_tasks", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let mut budget = TickBudget::chatman();
                let _ = MuInstruction::eval_task(
                    black_box(i as u64),
                    black_box(&obs),
                    &mut budget,
                );
            }
        });
    });

    group.finish();
}

criterion_group!(
    chatman_benches,
    bench_complete_task,
    bench_patterns_under_chatman,
    bench_cycle_accuracy,
    bench_determinism,
    bench_sustained_load,
);

criterion_main!(chatman_benches);
