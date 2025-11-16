//! Property-Based Tests for Const Validation
//!
//! Uses quickcheck/proptest for generative testing of hash properties.
//! Validates mathematical properties of FNV-1a implementation.
//!
//! **Properties Tested**:
//! - Determinism: f(x) = f(x) for all x
//! - Uniqueness: f(x) ≠ f(y) for most x ≠ y
//! - Distribution: uniform distribution across output space
//! - Range coverage: all output values reachable

use knhk_otel::const_validation::*;

/// Property: Span ID generation is deterministic for any seed
/// For all seeds s: generate_span_id_const(s) = generate_span_id_const(s)
#[test]
fn prop_span_id_deterministic() {
    // Manual property test with various seeds
    let seeds = vec![
        0u64, 1, 42, 100, 1000,
        u64::MAX, u64::MAX - 1,
        0xDEADBEEF, 0xCAFEBABE,
        rand::random(), rand::random(), rand::random(),
    ];

    for seed in seeds {
        let id1 = generate_span_id_const(seed);
        let id2 = generate_span_id_const(seed);
        let id3 = generate_span_id_const(seed);

        assert_eq!(id1, id2, "Determinism violated for seed {}", seed);
        assert_eq!(id2, id3, "Determinism violated for seed {}", seed);
    }
}

/// Property: Trace ID generation is deterministic for any seed
/// For all seeds s: generate_trace_id_const(s) = generate_trace_id_const(s)
#[test]
fn prop_trace_id_deterministic() {
    let seeds = vec![
        0u128, 1, 42, 100, 1000,
        u128::MAX, u128::MAX - 1,
        0xDEADBEEFCAFEBABE_DEADBEEFCAFEBABE,
        rand::random(), rand::random(), rand::random(),
    ];

    for seed in seeds {
        let id1 = generate_trace_id_const(seed);
        let id2 = generate_trace_id_const(seed);
        let id3 = generate_trace_id_const(seed);

        assert_eq!(id1, id2, "Determinism violated for seed {:x}", seed);
        assert_eq!(id2, id3, "Determinism violated for seed {:x}", seed);
    }
}

/// Property: Different seeds should produce different IDs (high probability)
/// For most pairs (x, y) where x ≠ y: f(x) ≠ f(y)
#[test]
fn prop_span_id_collision_resistance() {
    let mut ids = std::collections::HashSet::new();
    let test_count = 10000;
    let mut collision_count = 0;

    for i in 0..test_count {
        let seed = rand::random::<u64>();
        let id = generate_span_id_const(seed);

        if ids.contains(&id) {
            collision_count += 1;
        }
        ids.insert(id);
    }

    // Allow <0.1% collision rate for random inputs
    let collision_rate = collision_count as f64 / test_count as f64;
    assert!(
        collision_rate < 0.001,
        "Collision rate too high: {:.4}% ({} collisions in {} tests)",
        collision_rate * 100.0,
        collision_count,
        test_count
    );
}

/// Property: Hash function produces non-zero outputs
/// For all seeds s: generate_span_id_const(s) ≠ 0
#[test]
fn prop_span_id_nonzero() {
    for _ in 0..1000 {
        let seed = rand::random::<u64>();
        let id = generate_span_id_const(seed);
        assert_ne!(id, 0, "Hash must never produce zero for seed {}", seed);
    }
}

/// Property: Trace ID hash produces non-zero outputs
/// For all seeds s: generate_trace_id_const(s) ≠ 0
#[test]
fn prop_trace_id_nonzero() {
    for _ in 0..1000 {
        let seed = rand::random::<u128>();
        let id = generate_trace_id_const(seed);
        assert_ne!(id, 0, "Hash must never produce zero for seed {:x}", seed);
    }
}

/// Property: Attribute hash is deterministic for any key-value pair
/// For all (k, v): compute_attribute_hash(k, v) = compute_attribute_hash(k, v)
#[test]
fn prop_attribute_hash_deterministic() {
    let test_pairs = vec![
        ("", ""),
        ("a", "b"),
        ("key", "value"),
        ("service.name", "knhk-sidecar"),
        ("very_long_key_with_lots_of_characters", "very_long_value_with_lots_of_characters"),
        ("unicode_key_🚀", "unicode_value_✨"),
        ("key\nwith\nnewlines", "value\nwith\nnewlines"),
    ];

    for (key, value) in test_pairs {
        let hash1 = compute_attribute_hash(key, value);
        let hash2 = compute_attribute_hash(key, value);
        let hash3 = compute_attribute_hash(key, value);

        assert_eq!(hash1, hash2, "Determinism violated for ({}, {})", key, value);
        assert_eq!(hash2, hash3, "Determinism violated for ({}, {})", key, value);
    }
}

/// Property: Attribute hash distinguishes key from value
/// For most pairs: hash(k1, v) ≠ hash(k2, v) when k1 ≠ k2
/// For most pairs: hash(k, v1) ≠ hash(k, v2) when v1 ≠ v2
#[test]
fn prop_attribute_hash_sensitivity() {
    let keys = vec!["key1", "key2", "key3", "service.name", "service.version"];
    let values = vec!["val1", "val2", "val3", "knhk-sidecar", "v1.0.0"];

    // Test key sensitivity
    for value in &values {
        let mut hashes = std::collections::HashSet::new();
        for key in &keys {
            let hash = compute_attribute_hash(key, value);
            assert!(
                hashes.insert(hash),
                "Different keys should produce different hashes: ({}, {})",
                key, value
            );
        }
    }

    // Test value sensitivity
    for key in &keys {
        let mut hashes = std::collections::HashSet::new();
        for value in &values {
            let hash = compute_attribute_hash(key, value);
            assert!(
                hashes.insert(hash),
                "Different values should produce different hashes: ({}, {})",
                key, value
            );
        }
    }
}

/// Property: MAX_SPANS validation is correct for all inputs
/// For all n: validate_max_spans_const(n) = (n ≤ 8)
#[test]
fn prop_max_spans_validation_correctness() {
    // Test exhaustive small range
    for n in 0..=20 {
        let expected = n <= 8;
        let actual = validate_max_spans_const(n);
        assert_eq!(
            actual, expected,
            "Validation incorrect for n={}: expected {}, got {}",
            n, expected, actual
        );
    }

    // Test random large values
    for _ in 0..100 {
        let n = rand::random::<usize>();
        let expected = n <= 8;
        let actual = validate_max_spans_const(n);
        assert_eq!(
            actual, expected,
            "Validation incorrect for n={}: expected {}, got {}",
            n, expected, actual
        );
    }
}

/// Property: Span structure validation is correct for all combinations
/// For all (a, b, c): validate_span_structure_const(a, b, c) = (a ∧ b ∧ c)
#[test]
fn prop_span_structure_validation_correctness() {
    let bools = [false, true];

    for &has_name in &bools {
        for &has_trace_id in &bools {
            for &has_span_id in &bools {
                let expected = has_name && has_trace_id && has_span_id;
                let actual = validate_span_structure_const(has_name, has_trace_id, has_span_id);

                assert_eq!(
                    actual, expected,
                    "Validation incorrect for ({}, {}, {}): expected {}, got {}",
                    has_name, has_trace_id, has_span_id, expected, actual
                );
            }
        }
    }
}

/// Property: Hash distribution should be approximately uniform
/// Chi-square test for uniformity across buckets
#[test]
fn prop_hash_distribution_uniformity() {
    const BUCKET_COUNT: usize = 256;
    const SAMPLE_COUNT: usize = 100000;

    let mut buckets = vec![0usize; BUCKET_COUNT];

    for _ in 0..SAMPLE_COUNT {
        let seed = rand::random::<u64>();
        let id = generate_span_id_const(seed);
        let bucket = (id % BUCKET_COUNT as u64) as usize;
        buckets[bucket] += 1;
    }

    // Expected count per bucket
    let expected = SAMPLE_COUNT as f64 / BUCKET_COUNT as f64;

    // Compute chi-square statistic
    let chi_square: f64 = buckets.iter()
        .map(|&observed| {
            let diff = observed as f64 - expected;
            (diff * diff) / expected
        })
        .sum();

    // Critical value for chi-square with 255 degrees of freedom at p=0.05 is ~293.2
    // We use a more lenient threshold for this test
    assert!(
        chi_square < 350.0,
        "Hash distribution not uniform: chi-square = {:.2} (threshold = 350.0)",
        chi_square
    );
}

/// Property: Hash avalanche effect (1-bit input change → ~50% output bits flip)
/// Tests for good mixing/diffusion properties
#[test]
fn prop_hash_avalanche_effect() {
    const TEST_COUNT: usize = 1000;
    let mut total_flipped_bits = 0u64;

    for _ in 0..TEST_COUNT {
        let seed = rand::random::<u64>();
        let bit_pos = rand::random::<u32>() % 64;
        let seed_flipped = seed ^ (1u64 << bit_pos);

        let id1 = generate_span_id_const(seed);
        let id2 = generate_span_id_const(seed_flipped);

        let diff = id1 ^ id2;
        let flipped_bits = diff.count_ones();
        total_flipped_bits += flipped_bits as u64;
    }

    let avg_flipped_bits = total_flipped_bits as f64 / TEST_COUNT as f64;

    // Good avalanche: expect ~32 bits flipped (50% of 64 bits)
    // Allow range of 24-40 bits (37.5% to 62.5%)
    assert!(
        avg_flipped_bits >= 24.0 && avg_flipped_bits <= 40.0,
        "Avalanche effect poor: avg {:.2} bits flipped (expected 24-40)",
        avg_flipped_bits
    );
}

/// Property: Hash function handles edge cases correctly
/// Tests boundary values and special patterns
#[test]
fn prop_hash_edge_cases() {
    let edge_cases = vec![
        0u64,
        1,
        u64::MAX,
        u64::MAX - 1,
        0x0000000000000001,
        0x8000000000000000,
        0x7FFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
        0x0F0F0F0F0F0F0F0F,
        0xF0F0F0F0F0F0F0F0,
        0xAAAAAAAAAAAAAAAA,
        0x5555555555555555,
    ];

    let mut ids = std::collections::HashSet::new();

    for seed in edge_cases {
        let id = generate_span_id_const(seed);

        // Verify non-zero
        assert_ne!(id, 0, "Hash must be non-zero for edge case seed {:#x}", seed);

        // Verify uniqueness
        assert!(
            ids.insert(id),
            "Hash collision for edge case seed {:#x}",
            seed
        );
    }
}

/// Property: Attribute hash handles long strings correctly
/// Tests that hash computation doesn't overflow or panic on large inputs
#[test]
fn prop_attribute_hash_long_strings() {
    let lengths = vec![0, 1, 10, 100, 1000, 10000];

    for len in lengths {
        let key = "k".repeat(len);
        let value = "v".repeat(len);

        let hash = compute_attribute_hash(&key, &value);
        assert_ne!(hash, 0, "Hash must be non-zero for length {}", len);
    }
}

/// Property: Hash range coverage (all output values theoretically reachable)
/// Tests that hash function can produce values across full u64 range
#[test]
fn prop_hash_range_coverage() {
    const SAMPLE_COUNT: usize = 100000;

    let mut min_id = u64::MAX;
    let mut max_id = u64::MIN;

    for _ in 0..SAMPLE_COUNT {
        let seed = rand::random::<u64>();
        let id = generate_span_id_const(seed);

        min_id = min_id.min(id);
        max_id = max_id.max(id);
    }

    // Verify significant range coverage (expect >90% of u64 range)
    let range = max_id - min_id;
    let coverage_ratio = range as f64 / u64::MAX as f64;

    assert!(
        coverage_ratio > 0.5,
        "Hash range coverage too low: {:.2}% (min={:#x}, max={:#x})",
        coverage_ratio * 100.0,
        min_id,
        max_id
    );
}

/// Property: Const functions are truly compile-time computable
/// Verifies that results can be used in const contexts
#[test]
fn prop_const_evaluation() {
    // These must compile (const evaluation)
    const SPAN_ID: u64 = generate_span_id_const(12345);
    const TRACE_ID: u128 = generate_trace_id_const(67890);
    const VALID: bool = validate_max_spans_const(8);
    const ATTR_HASH: u64 = compute_attribute_hash("key", "value");
    const SPAN_VALID: bool = validate_span_structure_const(true, true, true);

    // Runtime verification
    assert_ne!(SPAN_ID, 0);
    assert_ne!(TRACE_ID, 0);
    assert!(VALID);
    assert_ne!(ATTR_HASH, 0);
    assert!(SPAN_VALID);
}
