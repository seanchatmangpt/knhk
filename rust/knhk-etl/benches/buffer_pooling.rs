// rust/knhk-etl/benches/buffer_pooling.rs
// Benchmark buffer pooling optimization (Week 1)
// Target: 75% allocation reduction
//
// Measures:
// - Allocation count reduction
// - Memory footprint reduction
// - Hot path performance impact
// - Pool contention overhead

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knhk_etl::load::{LoadStage, SoAArrays};
use knhk_etl::transform::{TransformResult, TypedTriple};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Allocation tracking allocator
struct TrackingAllocator {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    bytes_allocated: AtomicUsize,
}

impl TrackingAllocator {
    const fn new() -> Self {
        Self {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
        }
    }

    fn reset(&self) {
        self.allocations.store(0, Ordering::SeqCst);
        self.deallocations.store(0, Ordering::SeqCst);
        self.bytes_allocated.store(0, Ordering::SeqCst);
    }

    fn stats(&self) -> AllocationStats {
        AllocationStats {
            allocations: self.allocations.load(Ordering::SeqCst),
            deallocations: self.deallocations.load(Ordering::SeqCst),
            bytes_allocated: self.bytes_allocated.load(Ordering::SeqCst),
        }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocations.fetch_add(1, Ordering::SeqCst);
        self.bytes_allocated
            .fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocations.fetch_add(1, Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

#[derive(Debug, Clone, Copy)]
struct AllocationStats {
    allocations: usize,
    deallocations: usize,
    bytes_allocated: usize,
}

impl AllocationStats {
    fn net_allocations(&self) -> usize {
        self.allocations.saturating_sub(self.deallocations)
    }
}

/// Generate test triples for benchmarking
fn generate_test_triples(count: usize) -> Vec<TypedTriple> {
    (0..count)
        .map(|i| TypedTriple {
            subject: (i + 1) as u64,
            predicate: 100,
            object: (i + 100) as u64,
            datatype: None,
        })
        .collect()
}

/// Benchmark hot path WITHOUT buffer pooling (baseline)
fn bench_without_pooling(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pooling/without_pool");

    for num_triples in [1, 4, 8].iter() {
        let triples = generate_test_triples(*num_triples);
        let transform_result = TransformResult {
            typed_triples: triples,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(num_triples),
            &transform_result,
            |b, input| {
                let load_stage = LoadStage::new();

                b.iter(|| {
                    ALLOCATOR.reset();
                    let result = load_stage.load(black_box(input.clone()));
                    let _stats = ALLOCATOR.stats();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark hot path WITH buffer pooling (optimized)
fn bench_with_pooling(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pooling/with_pool");

    for num_triples in [1, 4, 8].iter() {
        let triples = generate_test_triples(*num_triples);
        let transform_result = TransformResult {
            typed_triples: triples,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(num_triples),
            &transform_result,
            |b, input| {
                // FUTURE: Create LoadStage with pooling enabled
                // let load_stage = LoadStage::new_with_pooling();
                let load_stage = LoadStage::new();

                b.iter(|| {
                    ALLOCATOR.reset();
                    let result = load_stage.load(black_box(input.clone()));
                    let _stats = ALLOCATOR.stats();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark allocation count reduction
fn bench_allocation_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pooling/allocation_count");

    let triples = generate_test_triples(8);
    let transform_result = TransformResult {
        typed_triples: triples,
    };

    // Without pooling (baseline)
    group.bench_function("without_pool", |b| {
        let load_stage = LoadStage::new();

        b.iter_custom(|iters| {
            ALLOCATOR.reset();

            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = load_stage.load(black_box(transform_result.clone()));
            }
            let elapsed = start.elapsed();

            let stats = ALLOCATOR.stats();
            println!(
                "\n[WITHOUT POOL] Allocations: {}, Bytes: {}, Net: {}",
                stats.allocations,
                stats.bytes_allocated,
                stats.net_allocations()
            );

            elapsed
        });
    });

    // With pooling (optimized)
    group.bench_function("with_pool", |b| {
        // FUTURE: Create LoadStage with pooling enabled
        let load_stage = LoadStage::new();

        b.iter_custom(|iters| {
            ALLOCATOR.reset();

            let start = std::time::Instant::now();
            for _ in 0..iters {
                let _ = load_stage.load(black_box(transform_result.clone()));
            }
            let elapsed = start.elapsed();

            let stats = ALLOCATOR.stats();
            println!(
                "\n[WITH POOL] Allocations: {}, Bytes: {}, Net: {}",
                stats.allocations,
                stats.bytes_allocated,
                stats.net_allocations()
            );

            elapsed
        });
    });

    group.finish();
}

/// Benchmark pool contention (concurrent access)
fn bench_pool_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pooling/contention");

    let triples = generate_test_triples(8);
    let transform_result = TransformResult {
        typed_triples: triples,
    };

    // Single-threaded access
    group.bench_function("single_thread", |b| {
        let load_stage = LoadStage::new();

        b.iter(|| {
            let result = load_stage.load(black_box(transform_result.clone()));
            black_box(result)
        });
    });

    // Multi-threaded access (if rayon feature enabled)
    #[cfg(feature = "parallel")]
    group.bench_function("multi_thread_4", |b| {
        use rayon::prelude::*;

        let load_stage = LoadStage::new();

        b.iter(|| {
            let results: Vec<_> = (0..4)
                .into_par_iter()
                .map(|_| load_stage.load(black_box(transform_result.clone())))
                .collect();
            black_box(results)
        });
    });

    group.finish();
}

/// Benchmark memory footprint
fn bench_memory_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pooling/memory_footprint");

    for num_iterations in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_iterations),
            num_iterations,
            |b, &iters| {
                let load_stage = LoadStage::new();
                let triples = generate_test_triples(8);
                let transform_result = TransformResult {
                    typed_triples: triples,
                };

                b.iter_custom(|_| {
                    ALLOCATOR.reset();

                    let start = std::time::Instant::now();
                    for _ in 0..iters {
                        let _ = load_stage.load(black_box(transform_result.clone()));
                    }
                    let elapsed = start.elapsed();

                    let stats = ALLOCATOR.stats();
                    println!(
                        "\n[{} iterations] Total bytes: {}, Avg per iteration: {}",
                        iters,
                        stats.bytes_allocated,
                        stats.bytes_allocated / iters
                    );

                    elapsed
                });
            },
        );
    }

    group.finish();
}

/// Custom benchmark to validate 75% allocation reduction target
fn validate_allocation_reduction() {
    println!("\n{}", "=".repeat(80));
    println!("WEEK 1 TARGET VALIDATION: 75% Allocation Reduction");
    println!("{}", "=".repeat(80));

    let triples = generate_test_triples(8);
    let transform_result = TransformResult {
        typed_triples: triples,
    };
    let iterations = 1000;

    // Baseline (without pooling)
    ALLOCATOR.reset();
    let load_stage_baseline = LoadStage::new();
    for _ in 0..iterations {
        let _ = load_stage_baseline.load(transform_result.clone());
    }
    let baseline_stats = ALLOCATOR.stats();

    println!("\nBaseline (without pooling):");
    println!("  Allocations: {}", baseline_stats.allocations);
    println!("  Bytes allocated: {}", baseline_stats.bytes_allocated);
    println!(
        "  Avg bytes/iteration: {}",
        baseline_stats.bytes_allocated / iterations
    );

    // Optimized (with pooling)
    ALLOCATOR.reset();
    // FUTURE: Replace with pooled version when implemented
    let load_stage_pooled = LoadStage::new();
    for _ in 0..iterations {
        let _ = load_stage_pooled.load(transform_result.clone());
    }
    let pooled_stats = ALLOCATOR.stats();

    println!("\nOptimized (with pooling):");
    println!("  Allocations: {}", pooled_stats.allocations);
    println!("  Bytes allocated: {}", pooled_stats.bytes_allocated);
    println!(
        "  Avg bytes/iteration: {}",
        pooled_stats.bytes_allocated / iterations
    );

    // Calculate reduction
    let allocation_reduction =
        100.0 * (1.0 - (pooled_stats.allocations as f64 / baseline_stats.allocations as f64));
    let bytes_reduction = 100.0
        * (1.0 - (pooled_stats.bytes_allocated as f64 / baseline_stats.bytes_allocated as f64));

    println!("\nReduction Metrics:");
    println!("  Allocation count: {:.1}%", allocation_reduction);
    println!("  Bytes allocated: {:.1}%", bytes_reduction);

    if allocation_reduction >= 75.0 {
        println!(
            "\n✅ WEEK 1 TARGET MET: {:.1}% reduction ≥ 75%",
            allocation_reduction
        );
    } else {
        println!(
            "\n❌ WEEK 1 TARGET NOT MET: {:.1}% reduction < 75%",
            allocation_reduction
        );
    }

    println!("{}", "=".repeat(80));
}

criterion_group!(
    benches,
    bench_without_pooling,
    bench_with_pooling,
    bench_allocation_count,
    bench_pool_contention,
    bench_memory_footprint
);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_tracking() {
        ALLOCATOR.reset();

        let v: Vec<u64> = vec![1, 2, 3, 4, 5];
        drop(v);

        let stats = ALLOCATOR.stats();
        assert!(stats.allocations > 0);
        assert!(stats.bytes_allocated > 0);
    }

    #[test]
    fn run_allocation_validation() {
        validate_allocation_reduction();
    }
}
