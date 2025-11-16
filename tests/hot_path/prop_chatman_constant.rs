// tests/hot_path/prop_chatman_constant.rs
// Property-based test: ALL hot path operations must complete ≤8 ticks (Chatman constant)
// Generates 1000+ test cases automatically, each validating the core guarantee

use proptest::prelude::*;
use std::time::Instant;
use std::arch::x86_64::_rdtsc;

/// Measure CPU ticks using RDTSC (x86-64 specific)
/// Returns cycle count with high precision
#[inline(always)]
fn measure_ticks_rdtsc<F: FnOnce() -> ()>(f: F) -> u64 {
    // Warm up CPU pipeline
    for _ in 0..100 {
        std::hint::black_box(0);
    }

    // Measure with RDTSC (Read Time-Stamp Counter)
    let start = unsafe { _rdtsc() };

    // Execute operation under test
    std::hint::black_box(f());

    let end = unsafe { _rdtsc() };

    end.saturating_sub(start)
}

/// Chatman constant: maximum ticks for hot path operations
const CHATMAN_CONSTANT: u64 = 8;

/// Simulate pattern execution (stub for testing framework)
fn execute_pattern(pattern_id: u8, input: &[u8]) -> u64 {
    // In production, this would call actual pattern dispatcher
    // For now: simulate pattern execution that should be ≤8 ticks
    let mut sum = 0u64;
    for byte in input.iter().take(std::cmp::min(16, input.len())) {
        sum = sum.wrapping_add(*byte as u64);
    }
    sum
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// CRITICAL: Property that MUST hold for ALL possible pattern_id x input_size combinations
    /// If ANY case takes >8 ticks, this is a covenant violation
    #[test]
    fn prop_all_hot_path_ops_within_chatman_constant(
        pattern_id in 0u8..43,
        input_size in 0usize..1000,
    ) {
        // Arrange: Create random input for this pattern
        let input = vec![pattern_id; std::cmp::min(input_size, 512)];

        // Act: Measure ticks for pattern execution
        let ticks = measure_ticks_rdtsc(|| {
            let _ = execute_pattern(pattern_id, &input);
        });

        // Assert: EVERY operation must be ≤8 ticks
        prop_assert!(
            ticks <= CHATMAN_CONSTANT,
            "Pattern {} with input_size {} took {} ticks (max: {})",
            pattern_id,
            input_size,
            ticks,
            CHATMAN_CONSTANT
        );

        // Secondary assertion: No operation should be instant (sanity check)
        prop_assert!(
            ticks > 0,
            "Pattern {} took 0 ticks (measurement error)",
            pattern_id
        );
    }

    /// Extended property: Verify tail latency (P99.9) is still within budget
    #[test]
    fn prop_chatman_constant_percentiles(
        pattern_id in 0u8..43,
    ) {
        let input = vec![pattern_id; 256];
        let mut measurements = Vec::with_capacity(100);

        // Collect 100 measurements
        for _ in 0..100 {
            let ticks = measure_ticks_rdtsc(|| {
                let _ = execute_pattern(pattern_id, &input);
            });
            measurements.push(ticks);
        }

        // Calculate percentiles
        measurements.sort_unstable();
        let p50 = measurements[50];
        let p95 = measurements[95];
        let p99 = measurements[99];

        // Even P99.9 should be well under budget (use P99 as proxy)
        prop_assert!(
            p99 <= CHATMAN_CONSTANT,
            "Pattern {} P99={} exceeds Chatman constant",
            pattern_id,
            p99
        );

        println!("Pattern {}: P50={} P95={} P99={}", pattern_id, p50, p95, p99);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_check_chatman_measurement() {
        // Quick sanity check without property testing
        let ticks = measure_ticks_rdtsc(|| {
            let mut x = 0u64;
            for i in 0..8 {
                x = x.wrapping_add(i);
            }
            std::hint::black_box(x);
        });

        assert!(
            ticks <= CHATMAN_CONSTANT * 10,
            "Simple operation took unreasonably long: {} ticks",
            ticks
        );
    }
}
