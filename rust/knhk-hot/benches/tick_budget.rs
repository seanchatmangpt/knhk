// rust/knhk-hot/benches/tick_budget.rs
// Tick budget validation benchmark (Week 1 & Week 2)
// Target: ≤7 ticks (Week 1), ≤5 ticks (Week 2)
//
// Measures:
// - Actual CPU cycles per operation
// - Hot path compliance
// - Progressive optimization impact

mod cycle_bench;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use cycle_bench::{BenchmarkHarness, BenchmarkResult};
use knhk_hot::*;

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn read_tsc() -> u64 {
    unsafe {
        let mut aux = 0u32;
        std::arch::x86_64::__rdtscp(&mut aux)
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
fn read_tsc() -> u64 {
    0 // Fallback for non-x86_64 platforms
}

/// Create test pattern context
fn create_test_context() -> *mut PatternContext {
    unsafe {
        let layout = std::alloc::Layout::from_size_align(
            std::mem::size_of::<PatternContext>(),
            64,
        )
        .unwrap();
        std::alloc::alloc_zeroed(layout) as *mut PatternContext
    }
}

fn destroy_test_context(ctx: *mut PatternContext) {
    unsafe {
        let layout = std::alloc::Layout::from_size_align(
            std::mem::size_of::<PatternContext>(),
            64,
        )
        .unwrap();
        std::alloc::dealloc(ctx as *mut u8, layout);
    }
}

/// Benchmark Pattern 20 (Timeout Discrimination) - Hot path critical
fn bench_pattern_timeout_ticks(c: &mut Criterion) {
    extern "C" fn test_branch(_ctx: *mut PatternContext) -> bool {
        black_box(42u64);
        true
    }

    extern "C" fn test_fallback(_ctx: *mut PatternContext) -> bool {
        black_box(17u64);
        false
    }

    let mut group = c.benchmark_group("tick_budget/pattern_timeout");

    let ctx = create_test_context();
    let timeout_ms = 1000u64;

    group.bench_function("baseline", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            let mut total_ticks = 0u64;

            for _ in 0..iters {
                let tick_start = read_tsc();
                unsafe {
                    cpu_dispatch::knhk_pattern_timeout(
                        black_box(ctx),
                        test_branch,
                        timeout_ms,
                        test_fallback,
                    );
                }
                let tick_end = read_tsc();
                total_ticks += tick_end.wrapping_sub(tick_start);
            }

            let avg_ticks = total_ticks / iters;
            println!("\n  Average ticks: {}", avg_ticks);

            start.elapsed()
        });
    });

    destroy_test_context(ctx);
    group.finish();
}

/// Benchmark ring buffer operations - Hot path critical
fn bench_ring_operations_ticks(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_budget/ring_buffer");

    // Enqueue benchmark
    group.bench_function("enqueue", |b| {
        let mut ring = DeltaRing::new(64).expect("Failed to create ring");
        let mut tick = 0u64;

        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            let mut total_ticks = 0u64;

            for _ in 0..iters {
                tick = (tick + 1) % 8;
                let delta = 42u64;

                let tick_start = read_tsc();
                ring.enqueue(black_box(tick), black_box(delta)).unwrap_or(());
                let tick_end = read_tsc();

                total_ticks += tick_end.wrapping_sub(tick_start);
            }

            let avg_ticks = total_ticks / iters;
            println!("\n  Enqueue average ticks: {}", avg_ticks);

            start.elapsed()
        });
    });

    // Dequeue benchmark
    group.bench_function("dequeue", |b| {
        let mut ring = DeltaRing::new(64).expect("Failed to create ring");

        // Pre-populate
        for tick in 0..8 {
            for _ in 0..5 {
                ring.enqueue(tick, 42u64 + tick).unwrap();
            }
        }

        let mut tick = 0u64;

        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            let mut total_ticks = 0u64;

            for _ in 0..iters {
                tick = (tick + 1) % 8;

                let tick_start = read_tsc();
                let _ = ring.dequeue(black_box(tick));
                let tick_end = read_tsc();

                total_ticks += tick_end.wrapping_sub(tick_start);
            }

            let avg_ticks = total_ticks / iters;
            println!("\n  Dequeue average ticks: {}", avg_ticks);

            start.elapsed()
        });
    });

    group.finish();
}

/// Benchmark content hashing - Supporting operation
fn bench_content_hash_ticks(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_budget/content_hash");

    for size in [64, 256, 1024].iter() {
        let data = vec![42u8; *size];

        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, input| {
            b.iter_custom(|iters| {
                let start = std::time::Instant::now();
                let mut total_ticks = 0u64;

                for _ in 0..iters {
                    let tick_start = read_tsc();
                    let _ = content_hash(black_box(input));
                    let tick_end = read_tsc();

                    total_ticks += tick_end.wrapping_sub(tick_start);
                }

                let avg_ticks = total_ticks / iters;
                println!("\n  Content hash ({} bytes) average ticks: {}", size, avg_ticks);

                start.elapsed()
            });
        });
    }

    group.finish();
}

/// Validate tick budget targets
fn validate_tick_budgets() {
    println!("\n{}", "=".repeat(80));
    println!("TICK BUDGET VALIDATION");
    println!("Week 1 Target: ≤7 ticks | Week 2 Target: ≤5 ticks");
    println!("{}", "=".repeat(80));

    #[cfg(not(target_arch = "x86_64"))]
    {
        println!("\n⚠️  TSC (Time Stamp Counter) only available on x86_64 architecture");
        println!("   Tick measurement not available on this platform\n");
        return;
    }

    #[cfg(target_arch = "x86_64")]
    {
        init_cpu_dispatch();

        let iterations = 10_000;

        // Pattern 20: Timeout Discrimination
        {
            extern "C" fn test_branch(_ctx: *mut PatternContext) -> bool {
                black_box(42u64);
                true
            }

            extern "C" fn test_fallback(_ctx: *mut PatternContext) -> bool {
                black_box(17u64);
                false
            }

            let ctx = create_test_context();
            let timeout_ms = 1000u64;

            let mut total_ticks = 0u64;
            for _ in 0..iterations {
                let start = read_tsc();
                unsafe {
                    cpu_dispatch::knhk_pattern_timeout(ctx, test_branch, timeout_ms, test_fallback);
                }
                let end = read_tsc();
                total_ticks += end.wrapping_sub(start);
            }

            let avg_ticks = (total_ticks as f64) / (iterations as f64);
            println!("\nPattern 20 (Timeout Discrimination):");
            println!("  Average: {:.2} ticks", avg_ticks);

            if avg_ticks <= 5.0 {
                println!("  ✅ Week 2 compliant: {:.2} ≤ 5 ticks", avg_ticks);
            } else if avg_ticks <= 7.0 {
                println!("  ✅ Week 1 compliant: {:.2} ≤ 7 ticks", avg_ticks);
                println!("  ⚠️  Week 2 target: {:.2} > 5 ticks", avg_ticks);
            } else {
                println!("  ❌ Exceeds budget: {:.2} > 7 ticks", avg_ticks);
            }

            destroy_test_context(ctx);
        }

        // Ring Buffer Enqueue
        {
            let mut ring = DeltaRing::new(64).expect("Failed to create ring");
            let mut total_ticks = 0u64;

            for i in 0..iterations {
                let tick = (i % 8) as u64;
                let delta = 42u64;

                let start = read_tsc();
                ring.enqueue(tick, delta).unwrap_or(());
                let end = read_tsc();

                total_ticks += end.wrapping_sub(start);
            }

            let avg_ticks = (total_ticks as f64) / (iterations as f64);
            println!("\nRing Buffer Enqueue:");
            println!("  Average: {:.2} ticks", avg_ticks);

            if avg_ticks <= 5.0 {
                println!("  ✅ Week 2 compliant: {:.2} ≤ 5 ticks", avg_ticks);
            } else if avg_ticks <= 7.0 {
                println!("  ✅ Week 1 compliant: {:.2} ≤ 7 ticks", avg_ticks);
                println!("  ⚠️  Week 2 target: {:.2} > 5 ticks", avg_ticks);
            } else {
                println!("  ❌ Exceeds budget: {:.2} > 7 ticks", avg_ticks);
            }
        }

        // Ring Buffer Dequeue
        {
            let mut ring = DeltaRing::new(64).expect("Failed to create ring");

            // Pre-populate
            for tick in 0..8 {
                for _ in 0..100 {
                    ring.enqueue(tick, 42u64 + tick).unwrap();
                }
            }

            let mut total_ticks = 0u64;

            for i in 0..iterations {
                let tick = (i % 8) as u64;

                let start = read_tsc();
                let _ = ring.dequeue(tick);
                let end = read_tsc();

                total_ticks += end.wrapping_sub(start);
            }

            let avg_ticks = (total_ticks as f64) / (iterations as f64);
            println!("\nRing Buffer Dequeue:");
            println!("  Average: {:.2} ticks", avg_ticks);

            if avg_ticks <= 5.0 {
                println!("  ✅ Week 2 compliant: {:.2} ≤ 5 ticks", avg_ticks);
            } else if avg_ticks <= 7.0 {
                println!("  ✅ Week 1 compliant: {:.2} ≤ 7 ticks", avg_ticks);
                println!("  ⚠️  Week 2 target: {:.2} > 5 ticks", avg_ticks);
            } else {
                println!("  ❌ Exceeds budget: {:.2} > 7 ticks", avg_ticks);
            }
        }

        println!("\n{}", "=".repeat(80));
    }
}

criterion_group!(
    benches,
    bench_pattern_timeout_ticks,
    bench_ring_operations_ticks,
    bench_content_hash_ticks
);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_tick_budget_validation() {
        validate_tick_budgets();
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_tsc_works() {
        let tsc1 = read_tsc();
        let tsc2 = read_tsc();
        assert!(tsc2 > tsc1, "TSC should be monotonically increasing");
    }
}
