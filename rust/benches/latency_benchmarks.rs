//! Latency Benchmarks using Criterion
//!
//! Measures all hot path operations in CPU ticks (not wall clock time)
//! to verify compliance with the ≤8 ticks constraint.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::arch::x86_64::{__rdtscp, _mm_lfence};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::time::Duration;

/// RDTSC-based benchmark timing
pub struct TickMeasurement {
    samples: Vec<u64>,
}

impl TickMeasurement {
    pub fn new() -> Self {
        Self {
            samples: Vec::with_capacity(10000),
        }
    }

    #[inline(always)]
    pub fn start() -> u64 {
        unsafe {
            _mm_lfence();
            let mut aux: u32 = 0;
            let cycles = __rdtscp(&mut aux);
            _mm_lfence();
            cycles
        }
    }

    #[inline(always)]
    pub fn end(start: u64) -> u64 {
        unsafe {
            _mm_lfence();
            let mut aux: u32 = 0;
            let end = __rdtscp(&mut aux);
            _mm_lfence();
            end.saturating_sub(start)
        }
    }

    pub fn measure<F: Fn()>(&mut self, f: F) -> u64 {
        let start = Self::start();
        f();
        let ticks = Self::end(start);
        self.samples.push(ticks);
        ticks
    }

    pub fn percentile(&self, p: f64) -> u64 {
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let idx = ((p / 100.0) * sorted.len() as f64) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }
}

/// Executor benchmark target
pub struct Executor {
    pattern_table: HashMap<u64, fn() -> u64>,
    guard_cache: Vec<bool>,
    receipt_counter: AtomicU64,
    descriptor_version: AtomicU64,
}

impl Executor {
    pub fn new() -> Self {
        let mut pattern_table = HashMap::new();
        for i in 0..256 {
            pattern_table.insert(i, || i * 2);
        }

        Self {
            pattern_table,
            guard_cache: vec![true; 10],
            receipt_counter: AtomicU64::new(0),
            descriptor_version: AtomicU64::new(1),
        }
    }

    #[inline(always)]
    pub fn dispatch_pattern(&self, pattern_id: u64) -> u64 {
        self.pattern_table.get(&(pattern_id % 256)).unwrap()()
    }

    #[inline(always)]
    pub fn evaluate_guards(&self, values: &[u64]) -> bool {
        values.iter().zip(&self.guard_cache)
            .all(|(val, &cached)| (*val > 0) == cached)
    }

    #[inline(always)]
    pub fn generate_receipt(&self) -> u64 {
        self.receipt_counter.fetch_add(1, Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn swap_descriptor(&self) -> u64 {
        self.descriptor_version.fetch_add(1, Ordering::AcqRel)
    }
}

/// Pattern dispatch latency benchmark
fn bench_pattern_dispatch(c: &mut Criterion) {
    let executor = Executor::new();
    let mut measurement = TickMeasurement::new();

    // Warmup
    for _ in 0..10000 {
        black_box(executor.dispatch_pattern(42));
    }

    c.bench_function("pattern_dispatch_ticks", |b| {
        b.iter_custom(|iters| {
            let mut total_ticks = 0u64;
            for _ in 0..iters {
                let ticks = measurement.measure(|| {
                    black_box(executor.dispatch_pattern(black_box(42)));
                });
                total_ticks += ticks;
            }
            Duration::from_nanos(total_ticks)
        });
    });

    // Report percentiles
    println!("Pattern Dispatch Latency:");
    println!("  P50: {} ticks", measurement.percentile(50.0));
    println!("  P95: {} ticks", measurement.percentile(95.0));
    println!("  P99: {} ticks", measurement.percentile(99.0));

    assert!(measurement.percentile(99.0) <= 8,
        "Pattern dispatch P99 latency exceeds 8 ticks");
}

/// Guard evaluation latency benchmark
fn bench_guard_evaluation(c: &mut Criterion) {
    let executor = Executor::new();
    let mut measurement = TickMeasurement::new();
    let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Warmup
    for _ in 0..10000 {
        black_box(executor.evaluate_guards(&values));
    }

    c.bench_function("guard_evaluation_ticks", |b| {
        b.iter_custom(|iters| {
            let mut total_ticks = 0u64;
            for _ in 0..iters {
                let ticks = measurement.measure(|| {
                    black_box(executor.evaluate_guards(black_box(&values)));
                });
                total_ticks += ticks;
            }
            Duration::from_nanos(total_ticks)
        });
    });

    println!("Guard Evaluation Latency:");
    println!("  P50: {} ticks", measurement.percentile(50.0));
    println!("  P95: {} ticks", measurement.percentile(95.0));
    println!("  P99: {} ticks", measurement.percentile(99.0));

    assert!(measurement.percentile(99.0) <= 8,
        "Guard evaluation P99 latency exceeds 8 ticks");
}

/// Receipt generation latency benchmark
fn bench_receipt_generation(c: &mut Criterion) {
    let executor = Executor::new();
    let mut measurement = TickMeasurement::new();

    // Warmup
    for _ in 0..10000 {
        black_box(executor.generate_receipt());
    }

    c.bench_function("receipt_generation_ticks", |b| {
        b.iter_custom(|iters| {
            let mut total_ticks = 0u64;
            for _ in 0..iters {
                let ticks = measurement.measure(|| {
                    black_box(executor.generate_receipt());
                });
                total_ticks += ticks;
            }
            Duration::from_nanos(total_ticks)
        });
    });

    println!("Receipt Generation Latency:");
    println!("  P50: {} ticks", measurement.percentile(50.0));
    println!("  P95: {} ticks", measurement.percentile(95.0));
    println!("  P99: {} ticks", measurement.percentile(99.0));

    assert!(measurement.percentile(99.0) <= 8,
        "Receipt generation P99 latency exceeds 8 ticks");
}

/// Descriptor swap latency benchmark
fn bench_descriptor_swap(c: &mut Criterion) {
    let executor = Executor::new();
    let mut measurement = TickMeasurement::new();

    // Warmup
    for _ in 0..10000 {
        black_box(executor.swap_descriptor());
    }

    c.bench_function("descriptor_swap_ticks", |b| {
        b.iter_custom(|iters| {
            let mut total_ticks = 0u64;
            for _ in 0..iters {
                let ticks = measurement.measure(|| {
                    black_box(executor.swap_descriptor());
                });
                total_ticks += ticks;
            }
            Duration::from_nanos(total_ticks)
        });
    });

    println!("Descriptor Swap Latency:");
    println!("  P50: {} ticks", measurement.percentile(50.0));
    println!("  P95: {} ticks", measurement.percentile(95.0));
    println!("  P99: {} ticks", measurement.percentile(99.0));

    assert!(measurement.percentile(99.0) <= 8,
        "Descriptor swap P99 latency exceeds 8 ticks");
}

/// Complete workflow latency benchmark
fn bench_complete_workflow(c: &mut Criterion) {
    let executor = Executor::new();
    let mut measurement = TickMeasurement::new();
    let values = vec![1, 2, 3, 4, 5];

    c.bench_function("complete_workflow_ticks", |b| {
        b.iter_custom(|iters| {
            let mut total_ticks = 0u64;
            for i in 0..iters {
                let ticks = measurement.measure(|| {
                    // Complete workflow execution
                    let pattern = executor.dispatch_pattern(i);
                    let guards_ok = executor.evaluate_guards(&values);
                    if guards_ok {
                        executor.generate_receipt();
                    }
                    if i % 10 == 0 {
                        executor.swap_descriptor();
                    }
                    black_box((pattern, guards_ok));
                });
                total_ticks += ticks;
            }
            Duration::from_nanos(total_ticks)
        });
    });

    println!("Complete Workflow Latency:");
    println!("  P50: {} ticks", measurement.percentile(50.0));
    println!("  P95: {} ticks", measurement.percentile(95.0));
    println!("  P99: {} ticks", measurement.percentile(99.0));
}

/// Parameterized benchmarks for different input sizes
fn bench_scalability(c: &mut Criterion) {
    let executor = Executor::new();
    let mut group = c.benchmark_group("scalability");

    for size in [1, 5, 10, 20, 50].iter() {
        let values: Vec<u64> = (0..*size).collect();

        group.bench_with_input(
            BenchmarkId::new("guard_evaluation", size),
            size,
            |b, _| {
                b.iter(|| executor.evaluate_guards(black_box(&values)));
            },
        );
    }

    group.finish();
}

/// Memory barrier impact benchmark
fn bench_memory_barriers(c: &mut Criterion) {
    let counter = AtomicU64::new(0);

    c.bench_function("atomic_relaxed", |b| {
        b.iter(|| {
            counter.fetch_add(1, Ordering::Relaxed)
        });
    });

    c.bench_function("atomic_acquire", |b| {
        b.iter(|| {
            counter.load(Ordering::Acquire)
        });
    });

    c.bench_function("atomic_release", |b| {
        b.iter(|| {
            counter.store(black_box(42), Ordering::Release)
        });
    });

    c.bench_function("atomic_seqcst", |b| {
        b.iter(|| {
            counter.fetch_add(1, Ordering::SeqCst)
        });
    });
}

/// Cache effects benchmark
fn bench_cache_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_effects");

    // Sequential access (cache friendly)
    let sequential_data: Vec<u64> = (0..1000).collect();
    group.bench_function("sequential_access", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for &val in sequential_data.iter() {
                sum = sum.wrapping_add(val);
            }
            black_box(sum)
        });
    });

    // Random access (cache unfriendly)
    let mut random_indices: Vec<usize> = (0..1000).collect();
    use rand::seq::SliceRandom;
    random_indices.shuffle(&mut rand::thread_rng());

    group.bench_function("random_access", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for &idx in random_indices.iter() {
                sum = sum.wrapping_add(sequential_data[idx]);
            }
            black_box(sum)
        });
    });

    group.finish();
}

/// Branch prediction effects
fn bench_branch_prediction(c: &mut Criterion) {
    let mut group = c.benchmark_group("branch_prediction");

    // Predictable branches
    let predictable = vec![true; 1000];
    group.bench_function("predictable", |b| {
        b.iter(|| {
            let mut count = 0;
            for &val in predictable.iter() {
                if val {
                    count += 1;
                }
            }
            black_box(count)
        });
    });

    // Unpredictable branches
    let unpredictable: Vec<bool> = (0..1000)
        .map(|i| i % 2 == rand::random::<u8>() % 2)
        .collect();

    group.bench_function("unpredictable", |b| {
        b.iter(|| {
            let mut count = 0;
            for &val in unpredictable.iter() {
                if val {
                    count += 1;
                }
            }
            black_box(count)
        });
    });

    group.finish();
}

criterion_group!(
    latency_benches,
    bench_pattern_dispatch,
    bench_guard_evaluation,
    bench_receipt_generation,
    bench_descriptor_swap,
    bench_complete_workflow,
    bench_scalability,
    bench_memory_barriers,
    bench_cache_effects,
    bench_branch_prediction
);

criterion_main!(latency_benches);