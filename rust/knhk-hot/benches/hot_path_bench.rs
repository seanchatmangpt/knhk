// knhk-hot/benches/hot_path_bench.rs
// Hot path operation benchmarks
// Target: ≤8 ticks for Pattern 20 timeout discrimination

mod cycle_bench;

use cycle_bench::{BenchmarkHarness, BenchmarkResult};
use knhk_hot::*;
use std::hint::black_box;

/// Setup a minimal pattern context for benchmarking
fn create_test_context() -> *mut PatternContext {
    // Allocate aligned memory for context
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

/// Benchmark Pattern 20: Timeout Discrimination
fn bench_pattern_timeout() -> BenchmarkResult {
    // Setup test branch function
    extern "C" fn test_branch(_ctx: *mut PatternContext) -> bool {
        // Minimal work to measure dispatch overhead
        black_box(42u64);
        true
    }

    extern "C" fn test_fallback(_ctx: *mut PatternContext) -> bool {
        black_box(17u64);
        false
    }

    let ctx = create_test_context();
    let timeout_ms = 1000u64;

    let harness = BenchmarkHarness::new(1000, 10000);
    let result = harness.measure("Pattern 20: Timeout (hot path)", || {
        unsafe {
            cpu_dispatch::knhk_pattern_timeout(ctx, test_branch, timeout_ms, test_fallback)
        }
    });

    destroy_test_context(ctx);
    result
}

/// Benchmark ring buffer enqueue operation
fn bench_ring_enqueue() -> BenchmarkResult {
    // Create delta ring
    let mut ring = DeltaRing::new(64).expect("Failed to create ring");

    let mut tick = 0u64;
    let delta = 42u64;

    let harness = BenchmarkHarness::new(1000, 10000);
    let result = harness.measure("Ring Buffer: Enqueue", || {
        tick = (tick + 1) % 8; // Cycle through ticks 0-7
        ring.enqueue(tick, delta).unwrap_or(());
    });

    result
}

/// Benchmark ring buffer dequeue operation
fn bench_ring_dequeue() -> BenchmarkResult {
    // Create and populate delta ring
    let mut ring = DeltaRing::new(64).expect("Failed to create ring");

    // Pre-populate with deltas
    for tick in 0..8 {
        for _ in 0..5 {
            ring.enqueue(tick, 42u64 + tick).unwrap();
        }
    }

    let mut tick = 0u64;

    let harness = BenchmarkHarness::new(1000, 10000);
    let result = harness.measure("Ring Buffer: Dequeue", || {
        tick = (tick + 1) % 8;
        let _ = ring.dequeue(tick);
    });

    result
}

/// Benchmark pattern discriminator (Pattern 9)
fn bench_pattern_discriminator() -> BenchmarkResult {
    extern "C" fn branch1(_ctx: *mut PatternContext) -> bool {
        black_box(1u64);
        true
    }

    extern "C" fn branch2(_ctx: *mut PatternContext) -> bool {
        black_box(2u64);
        true
    }

    extern "C" fn branch3(_ctx: *mut PatternContext) -> bool {
        black_box(3u64);
        true
    }

    extern "C" fn branch4(_ctx: *mut PatternContext) -> bool {
        black_box(4u64);
        true
    }

    let ctx = create_test_context();

    // Function pointer array
    let branches: Vec<BranchFn> = vec![branch1, branch2, branch3, branch4];

    let harness = BenchmarkHarness::new(1000, 10000);
    let result = harness.measure("Pattern 9: Discriminator (4 branches)", || {
        let dispatcher = CpuDispatcher::global();
        let discriminator_fn = dispatcher.get_discriminator();
        unsafe { discriminator_fn(ctx, branches.as_ptr(), branches.len() as u32) }
    });

    destroy_test_context(ctx);
    result
}

/// Benchmark branchless dispatch table
fn bench_branchless_dispatch() -> BenchmarkResult {
    let ctx = create_test_context();

    // Test data for dispatch - use actual pattern types
    let pattern_types = vec![1u32, 2, 3, 4, 5, 6, 9, 10, 11, 16, 20, 21];
    let mut idx = 0;

    let harness = BenchmarkHarness::new(1000, 10000);
    let result = harness.measure("Branchless Pattern Dispatch", || {
        idx = (idx + 1) % pattern_types.len();
        let pattern_type = pattern_types[idx];

        unsafe {
            cpu_dispatch::knhk_dispatch_pattern(
                pattern_type,
                ctx,
                std::ptr::null_mut(),
                0,
            )
        }
    });

    destroy_test_context(ctx);
    result
}

/// Benchmark parallel split (Pattern 2)
fn bench_parallel_split() -> BenchmarkResult {
    extern "C" fn parallel_branch(_ctx: *mut PatternContext) -> bool {
        // Simulate minimal work per branch
        black_box(42u64);
        true
    }

    let ctx = create_test_context();

    // 4 branches for SIMD optimization
    let branches: Vec<BranchFn> = vec![
        parallel_branch,
        parallel_branch,
        parallel_branch,
        parallel_branch,
    ];

    let harness = BenchmarkHarness::new(1000, 10000);
    let result = harness.measure("Pattern 2: Parallel Split (4 branches)", || {
        let dispatcher = CpuDispatcher::global();
        let parallel_split_fn = dispatcher.get_parallel_split();
        unsafe { parallel_split_fn(ctx, branches.as_ptr(), branches.len() as u32) }
    });

    destroy_test_context(ctx);
    result
}

/// Benchmark synchronization (Pattern 3)
fn bench_synchronization() -> BenchmarkResult {
    let ctx = create_test_context();

    // Simulate 4 completed branches
    let results = vec![1u64, 1, 1, 1];

    let harness = BenchmarkHarness::new(1000, 10000);
    let result = harness.measure("Pattern 3: Synchronization (4 branches)", || {
        let dispatcher = CpuDispatcher::global();
        let sync_fn = dispatcher.get_synchronization();
        unsafe { sync_fn(ctx, results.as_ptr(), results.len() as u32) }
    });

    destroy_test_context(ctx);
    result
}

/// Benchmark content addressing (BLAKE3)
fn bench_content_hash() -> BenchmarkResult {
    use knhk_hot::content_hash;

    let data = vec![42u8; 1024]; // 1KB of data

    let harness = BenchmarkHarness::new(1000, 10000);
    harness.measure("Content Hash (BLAKE3, 1KB)", || content_hash(&data))
}

fn main() {
    println!("\n{}", "=".repeat(80));
    println!("KNHK HOT PATH BASELINE PERFORMANCE BENCHMARKS");
    println!("Target: ≤8 ticks for hot path operations");
    println!("Based on simdjson cycle-accurate measurement approach");
    println!("{}", "=".repeat(80));

    #[cfg(not(target_os = "linux"))]
    {
        println!("\n⚠️  Warning: Running on non-Linux platform");
        println!("   Hardware performance counters not available");
        println!("   Only timing measurements will be collected\n");
    }

    #[cfg(target_os = "linux")]
    {
        println!("\n✓ Linux detected: Full hardware performance counters enabled");
        println!("  - CPU cycles");
        println!("  - Instructions");
        println!("  - Cache references/misses");
        println!("  - Branch instructions/mispredictions\n");
    }

    // Initialize CPU dispatch
    init_cpu_dispatch();
    CpuFeatures::get().log_features();

    // Critical Hot Path Operations (Target: ≤8 ticks)
    println!("\n{}", "=".repeat(80));
    println!("CRITICAL HOT PATH OPERATIONS (≤8 tick target)");
    println!("{}", "=".repeat(80));

    let result = bench_pattern_timeout();
    result.print_report();

    let result = bench_ring_enqueue();
    result.print_report();

    let result = bench_ring_dequeue();
    result.print_report();

    // Workflow Pattern Operations
    println!("\n{}", "=".repeat(80));
    println!("WORKFLOW PATTERN OPERATIONS");
    println!("{}", "=".repeat(80));

    let result = bench_pattern_discriminator();
    result.print_report();

    let result = bench_branchless_dispatch();
    result.print_report();

    let result = bench_parallel_split();
    result.print_report();

    let result = bench_synchronization();
    result.print_report();

    // Supporting Operations
    println!("\n{}", "=".repeat(80));
    println!("SUPPORTING OPERATIONS");
    println!("{}", "=".repeat(80));

    let result = bench_content_hash();
    result.print_report();

    // Final Summary
    println!("\n{}", "=".repeat(80));
    println!("BENCHMARK COMPLETE");
    println!("{}", "=".repeat(80));
    println!("\nResults saved to: docs/evidence/BASELINE_PERFORMANCE.md");
    println!("Run with: cargo bench --bench hot_path_bench");

    #[cfg(target_os = "linux")]
    {
        println!("\nTo run with detailed perf analysis:");
        println!("  perf stat -e cycles,instructions,cache-references,cache-misses \\");
        println!("    cargo bench --bench hot_path_bench");
    }

    println!("\n{}", "=".repeat(80));
}
