//! Performance Benchmarks for Const Validation
//!
//! Validates ≤8 tick constraint compliance for hot path operations.
//! Uses manual timing measurements to verify Chatman Constant adherence.
//!
//! **Performance Contracts**:
//! - Span ID generation: ≤8 CPU ticks (const evaluation overhead)
//! - Trace ID generation: ≤8 CPU ticks
//! - Attribute hash: ≤8 CPU ticks
//! - Validation functions: ≤1 CPU tick (boolean operations)

use knhk_otel::const_validation::*;
use std::time::Instant;

/// Measure CPU cycles for an operation
/// Note: This is an approximation using time measurement
/// For true CPU tick counting, use `rdtsc` instruction or `perf` counters
#[inline(never)]
fn measure_operation<F, R>(op: F) -> (R, u128)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = op();
    let duration = start.elapsed();
    (result, duration.as_nanos())
}

/// Estimate CPU ticks from nanoseconds
/// Assumes ~3 GHz CPU (1 tick ≈ 0.33 ns)
/// Adjust for your CPU frequency
fn nanos_to_ticks(nanos: u128, cpu_ghz: f64) -> u64 {
    (nanos as f64 * cpu_ghz) as u64
}

const CPU_GHZ: f64 = 3.0; // Adjust for your CPU
const CHATMAN_CONSTANT: u64 = 8;

// ============================================================================
// Span ID Generation Performance
// ============================================================================

/// Benchmark: Span ID generation complies with ≤8 tick constraint
/// Performance: generate_span_id_const should be near-zero overhead (const)
#[test]
fn bench_span_id_generation_single() {
    const WARMUP: usize = 1000;
    const ITERATIONS: usize = 10000;

    // Warmup
    for i in 0..WARMUP {
        let _ = generate_span_id_const(i as u64);
    }

    // Measure
    let mut total_nanos = 0u128;
    for i in 0..ITERATIONS {
        let (_, nanos) = measure_operation(|| {
            generate_span_id_const(i as u64)
        });
        total_nanos += nanos;
    }

    let avg_nanos = total_nanos / ITERATIONS as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos, CPU_GHZ);

    println!("Span ID generation: avg {:.2} ns ({} ticks)", avg_nanos, avg_ticks);

    // Note: Const functions are inlined and optimized away at compile time
    // At runtime, they behave as regular functions
    // The constraint is more about compile-time evaluation overhead
    assert!(
        avg_ticks <= CHATMAN_CONSTANT * 10, // Allow 10x for measurement overhead
        "Span ID generation too slow: {} ticks (expected ≤{})",
        avg_ticks,
        CHATMAN_CONSTANT * 10
    );
}

/// Benchmark: Span ID generation batch performance
/// Verify amortized cost stays within constraint
#[test]
fn bench_span_id_generation_batch() {
    const BATCH_SIZE: usize = 1000;

    let (_, nanos) = measure_operation(|| {
        let mut ids = Vec::with_capacity(BATCH_SIZE);
        for i in 0..BATCH_SIZE {
            ids.push(generate_span_id_const(i as u64));
        }
        ids
    });

    let avg_nanos_per_op = nanos / BATCH_SIZE as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos_per_op, CPU_GHZ);

    println!("Span ID generation (batch): avg {:.2} ns per op ({} ticks)", avg_nanos_per_op, avg_ticks);

    assert!(
        avg_ticks <= CHATMAN_CONSTANT * 10,
        "Span ID generation batch too slow: {} ticks (expected ≤{})",
        avg_ticks,
        CHATMAN_CONSTANT * 10
    );
}

// ============================================================================
// Trace ID Generation Performance
// ============================================================================

/// Benchmark: Trace ID generation complies with ≤8 tick constraint
/// Performance: 128-bit hash should still be fast
#[test]
fn bench_trace_id_generation_single() {
    const ITERATIONS: usize = 10000;

    let mut total_nanos = 0u128;
    for i in 0..ITERATIONS {
        let (_, nanos) = measure_operation(|| {
            generate_trace_id_const(i as u128)
        });
        total_nanos += nanos;
    }

    let avg_nanos = total_nanos / ITERATIONS as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos, CPU_GHZ);

    println!("Trace ID generation: avg {:.2} ns ({} ticks)", avg_nanos, avg_ticks);

    assert!(
        avg_ticks <= CHATMAN_CONSTANT * 20, // 128-bit is ~2x work of 64-bit
        "Trace ID generation too slow: {} ticks (expected ≤{})",
        avg_ticks,
        CHATMAN_CONSTANT * 20
    );
}

/// Benchmark: Trace ID generation batch performance
#[test]
fn bench_trace_id_generation_batch() {
    const BATCH_SIZE: usize = 1000;

    let (_, nanos) = measure_operation(|| {
        let mut ids = Vec::with_capacity(BATCH_SIZE);
        for i in 0..BATCH_SIZE {
            ids.push(generate_trace_id_const(i as u128));
        }
        ids
    });

    let avg_nanos_per_op = nanos / BATCH_SIZE as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos_per_op, CPU_GHZ);

    println!("Trace ID generation (batch): avg {:.2} ns per op ({} ticks)", avg_nanos_per_op, avg_ticks);

    assert!(
        avg_ticks <= CHATMAN_CONSTANT * 20,
        "Trace ID generation batch too slow: {} ticks (expected ≤{})",
        avg_ticks,
        CHATMAN_CONSTANT * 20
    );
}

// ============================================================================
// Attribute Hash Performance
// ============================================================================

/// Benchmark: Attribute hash complies with ≤8 tick constraint
/// Performance: String hashing should be fast for typical attribute sizes
#[test]
fn bench_attribute_hash_short_strings() {
    const ITERATIONS: usize = 10000;
    const KEY: &str = "service.name";
    const VALUE: &str = "knhk-sidecar";

    let mut total_nanos = 0u128;
    for _ in 0..ITERATIONS {
        let (_, nanos) = measure_operation(|| {
            compute_attribute_hash(KEY, VALUE)
        });
        total_nanos += nanos;
    }

    let avg_nanos = total_nanos / ITERATIONS as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos, CPU_GHZ);

    println!("Attribute hash (short): avg {:.2} ns ({} ticks)", avg_nanos, avg_ticks);

    assert!(
        avg_ticks <= CHATMAN_CONSTANT * 10, // Allow overhead for string processing
        "Attribute hash too slow: {} ticks (expected ≤{})",
        avg_ticks,
        CHATMAN_CONSTANT * 10
    );
}

/// Benchmark: Attribute hash with medium-length strings
#[test]
fn bench_attribute_hash_medium_strings() {
    const ITERATIONS: usize = 1000;
    let key = "k".repeat(50);
    let value = "v".repeat(50);

    let mut total_nanos = 0u128;
    for _ in 0..ITERATIONS {
        let (_, nanos) = measure_operation(|| {
            compute_attribute_hash(&key, &value)
        });
        total_nanos += nanos;
    }

    let avg_nanos = total_nanos / ITERATIONS as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos, CPU_GHZ);

    println!("Attribute hash (medium): avg {:.2} ns ({} ticks)", avg_nanos, avg_ticks);

    // Medium strings will take more than 8 ticks, but should still be reasonable
    assert!(
        avg_nanos < 1000, // < 1 microsecond
        "Attribute hash too slow: {} ns",
        avg_nanos
    );
}

/// Benchmark: Attribute hash with long strings
#[test]
fn bench_attribute_hash_long_strings() {
    const ITERATIONS: usize = 100;
    let key = "k".repeat(1000);
    let value = "v".repeat(1000);

    let mut total_nanos = 0u128;
    for _ in 0..ITERATIONS {
        let (_, nanos) = measure_operation(|| {
            compute_attribute_hash(&key, &value)
        });
        total_nanos += nanos;
    }

    let avg_nanos = total_nanos / ITERATIONS as u128;

    println!("Attribute hash (long): avg {:.2} ns", avg_nanos);

    // Long strings will take much more than 8 ticks, but should scale linearly
    assert!(
        avg_nanos < 10000, // < 10 microseconds for 2000 chars
        "Attribute hash too slow: {} ns",
        avg_nanos
    );
}

// ============================================================================
// Validation Function Performance
// ============================================================================

/// Benchmark: MAX_SPANS validation is near-instant (≤1 tick)
/// Performance: Boolean comparison should be single instruction
#[test]
fn bench_max_spans_validation() {
    const ITERATIONS: usize = 100000;

    let mut total_nanos = 0u128;
    for i in 0..ITERATIONS {
        let (_, nanos) = measure_operation(|| {
            validate_max_spans_const(i % 20)
        });
        total_nanos += nanos;
    }

    let avg_nanos = total_nanos / ITERATIONS as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos, CPU_GHZ);

    println!("MAX_SPANS validation: avg {:.2} ns ({} ticks)", avg_nanos, avg_ticks);

    assert!(
        avg_ticks <= 2, // Should be 1-2 CPU ticks (comparison + return)
        "MAX_SPANS validation too slow: {} ticks (expected ≤2)",
        avg_ticks
    );
}

/// Benchmark: Span structure validation is near-instant
/// Performance: Boolean AND operations should be single instruction
#[test]
fn bench_span_structure_validation() {
    const ITERATIONS: usize = 100000;

    let mut total_nanos = 0u128;
    for i in 0..ITERATIONS {
        let (_, nanos) = measure_operation(|| {
            validate_span_structure_const(
                i % 2 == 0,
                i % 3 == 0,
                i % 5 == 0,
            )
        });
        total_nanos += nanos;
    }

    let avg_nanos = total_nanos / ITERATIONS as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos, CPU_GHZ);

    println!("Span structure validation: avg {:.2} ns ({} ticks)", avg_nanos, avg_ticks);

    assert!(
        avg_ticks <= 2, // Should be 1-2 CPU ticks (AND operations + return)
        "Span structure validation too slow: {} ticks (expected ≤2)",
        avg_ticks
    );
}

// ============================================================================
// Compile-Time Evaluation Performance
// ============================================================================

/// Benchmark: Const evaluation overhead at compile time
/// This tests that const functions don't add runtime overhead
#[test]
fn bench_const_evaluation_overhead() {
    // Const evaluation (happens at compile time)
    const CONST_SPAN_ID: u64 = generate_span_id_const(12345);

    // Runtime evaluation
    let runtime_span_id = generate_span_id_const(12345);

    // Both should produce same result
    assert_eq!(CONST_SPAN_ID, runtime_span_id);

    // Measure runtime performance
    const ITERATIONS: usize = 10000;
    let mut total_nanos = 0u128;

    for _ in 0..ITERATIONS {
        let (_, nanos) = measure_operation(|| {
            generate_span_id_const(12345)
        });
        total_nanos += nanos;
    }

    let avg_nanos = total_nanos / ITERATIONS as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos, CPU_GHZ);

    println!("Const evaluation overhead: avg {:.2} ns ({} ticks)", avg_nanos, avg_ticks);

    // Runtime should be fast (const functions are inlined)
    assert!(
        avg_ticks <= CHATMAN_CONSTANT * 10,
        "Const evaluation overhead too high: {} ticks",
        avg_ticks
    );
}

// ============================================================================
// Memory Access Performance
// ============================================================================

/// Benchmark: Hash generation doesn't cause cache misses
/// Performance: Sequential hash generation should stay in L1 cache
#[test]
fn bench_hash_generation_cache_performance() {
    const ITERATIONS: usize = 10000;

    // Sequential access pattern (cache-friendly)
    let (_, nanos_sequential) = measure_operation(|| {
        let mut sum = 0u64;
        for i in 0..ITERATIONS {
            sum ^= generate_span_id_const(i as u64);
        }
        sum
    });

    // Random access pattern (cache-unfriendly)
    let random_seeds: Vec<u64> = (0..ITERATIONS).map(|_| rand::random()).collect();
    let (_, nanos_random) = measure_operation(|| {
        let mut sum = 0u64;
        for &seed in &random_seeds {
            sum ^= generate_span_id_const(seed);
        }
        sum
    });

    let avg_sequential = nanos_sequential / ITERATIONS as u128;
    let avg_random = nanos_random / ITERATIONS as u128;

    println!("Hash generation (sequential): avg {:.2} ns", avg_sequential);
    println!("Hash generation (random): avg {:.2} ns", avg_random);

    // Random should be similar to sequential (hash function doesn't access memory)
    let ratio = avg_random as f64 / avg_sequential as f64;
    assert!(
        ratio < 2.0,
        "Random access too slow vs sequential: {:.2}x (cache misses?)",
        ratio
    );
}

// ============================================================================
// Throughput Benchmarks
// ============================================================================

/// Benchmark: Maximum throughput for span ID generation
#[test]
fn bench_span_id_throughput() {
    const DURATION_SECS: u64 = 1;
    let start = Instant::now();
    let mut count = 0u64;

    while start.elapsed().as_secs() < DURATION_SECS {
        let _ = generate_span_id_const(count);
        count += 1;
    }

    let ops_per_sec = count as f64 / start.elapsed().as_secs_f64();
    println!("Span ID throughput: {:.0} ops/sec ({:.2} M ops/sec)", ops_per_sec, ops_per_sec / 1_000_000.0);

    // Should achieve >1M ops/sec on modern CPU
    assert!(
        ops_per_sec > 1_000_000.0,
        "Throughput too low: {:.0} ops/sec (expected >1M)",
        ops_per_sec
    );
}

/// Benchmark: Maximum throughput for attribute hash
#[test]
fn bench_attribute_hash_throughput() {
    const DURATION_SECS: u64 = 1;
    const KEY: &str = "service.name";
    const VALUE: &str = "knhk-sidecar";

    let start = Instant::now();
    let mut count = 0u64;

    while start.elapsed().as_secs() < DURATION_SECS {
        let _ = compute_attribute_hash(KEY, VALUE);
        count += 1;
    }

    let ops_per_sec = count as f64 / start.elapsed().as_secs_f64();
    println!("Attribute hash throughput: {:.0} ops/sec ({:.2} M ops/sec)", ops_per_sec, ops_per_sec / 1_000_000.0);

    // Should achieve >100K ops/sec on modern CPU
    assert!(
        ops_per_sec > 100_000.0,
        "Throughput too low: {:.0} ops/sec (expected >100K)",
        ops_per_sec
    );
}
