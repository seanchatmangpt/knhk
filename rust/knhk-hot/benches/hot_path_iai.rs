// knhk-hot/benches/hot_path_iai.rs
// IAI-Callgrind benchmarks for hot path validation (≤8 ticks)
// Uses Valgrind's Callgrind for cache-aware, instruction-level precision

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use knhk_hot::*;
use std::hint::black_box;

/// Setup a minimal pattern context for benchmarking
fn create_test_context() -> *mut PatternContext {
    unsafe {
        let layout =
            std::alloc::Layout::from_size_align(std::mem::size_of::<PatternContext>(), 64).unwrap();
        std::alloc::alloc_zeroed(layout) as *mut PatternContext
    }
}

fn destroy_test_context(ctx: *mut PatternContext) {
    unsafe {
        let layout =
            std::alloc::Layout::from_size_align(std::mem::size_of::<PatternContext>(), 64).unwrap();
        std::alloc::dealloc(ctx as *mut u8, layout);
    }
}

/// Benchmark Pattern 20: Timeout Discrimination (hot path)
#[library_benchmark]
fn bench_pattern_timeout() {
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

    unsafe {
        cpu_dispatch::knhk_pattern_timeout(ctx, test_branch, timeout_ms, test_fallback);
    }

    destroy_test_context(ctx);
}

/// Benchmark ring buffer enqueue operation (hot path)
#[library_benchmark]
fn bench_ring_enqueue() {
    let mut ring = DeltaRing::new(64).expect("Failed to create ring");
    let tick = 0u64;
    let delta = 42u64;

    black_box(ring.enqueue(tick, delta));
}

/// Benchmark ring buffer dequeue operation (hot path)
#[library_benchmark]
fn bench_ring_dequeue() {
    let mut ring = DeltaRing::new(64).expect("Failed to create ring");

    // Pre-populate with deltas
    for tick in 0..8 {
        ring.enqueue(tick, 42u64 + tick).unwrap();
    }

    let tick = 0u64;
    black_box(ring.dequeue(tick));
}

/// Benchmark branchless dispatch table lookup (hot path)
#[library_benchmark]
fn bench_branchless_dispatch() {
    let ctx = create_test_context();
    let pattern_type = 1u32;

    unsafe {
        cpu_dispatch::knhk_dispatch_pattern(pattern_type, ctx, std::ptr::null_mut(), 0);
    }

    destroy_test_context(ctx);
}

/// Benchmark pattern discriminator (Pattern 9) - hot path
#[library_benchmark]
fn bench_pattern_discriminator() {
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
    let branches: Vec<BranchFn> = vec![branch1, branch2, branch3, branch4];

    let dispatcher = CpuDispatcher::global();
    let discriminator_fn = dispatcher.get_discriminator();
    unsafe {
        discriminator_fn(ctx, branches.as_ptr(), branches.len() as u32);
    }

    destroy_test_context(ctx);
}

/// Benchmark content hash (BLAKE3) - hot path
#[library_benchmark]
fn bench_content_hash() {
    use knhk_hot::content_hash;
    let data = vec![42u8; 64]; // 64 bytes (cache line size)
    black_box(content_hash(&data));
}

library_benchmark_group!(
    name = hot_path_benches;
    benchmarks =
        bench_pattern_timeout,
        bench_ring_enqueue,
        bench_ring_dequeue,
        bench_branchless_dispatch,
        bench_pattern_discriminator,
        bench_content_hash
);

main!(library_benchmark_group = hot_path_benches);
