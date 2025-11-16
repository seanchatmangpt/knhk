//! Const fn helpers for compile-time computation and validation
//!
//! Provides compile-time functions for span ID generation, validation,
//! and hash computation to enable zero-overhead hot path telemetry.

use crate::{SpanId, TraceId};

/// Compile-time span ID generation
///
/// Generates a deterministic span ID at compile time using FNV-1a hash.
/// This is useful for testing and compile-time validation.
///
/// # Arguments
/// * `seed` - Seed value for hash generation
///
/// # Returns
/// * Deterministic span ID computed at compile time
///
/// # Example
/// ```rust
/// const SPAN_ID: u64 = generate_span_id_const(12345);
/// ```
pub const fn generate_span_id_const(seed: u64) -> u64 {
    // FNV-1a hash for compile-time computation
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET;
    let mut value = seed;

    // Process 8 bytes (64-bit value) - manually unroll loop for const fn
    hash ^= value & 0xFF;
    hash = hash.wrapping_mul(FNV_PRIME);
    value >>= 8;
    hash ^= value & 0xFF;
    hash = hash.wrapping_mul(FNV_PRIME);
    value >>= 8;
    hash ^= value & 0xFF;
    hash = hash.wrapping_mul(FNV_PRIME);
    value >>= 8;
    hash ^= value & 0xFF;
    hash = hash.wrapping_mul(FNV_PRIME);
    value >>= 8;
    hash ^= value & 0xFF;
    hash = hash.wrapping_mul(FNV_PRIME);
    value >>= 8;
    hash ^= value & 0xFF;
    hash = hash.wrapping_mul(FNV_PRIME);
    value >>= 8;
    hash ^= value & 0xFF;
    hash = hash.wrapping_mul(FNV_PRIME);
    value >>= 8;
    hash ^= value & 0xFF;
    hash = hash.wrapping_mul(FNV_PRIME);

    hash
}

/// Compile-time trace ID generation
///
/// Generates a deterministic trace ID at compile time using FNV-1a hash.
///
/// # Arguments
/// * `seed` - Seed value for hash generation
///
/// # Returns
/// * Deterministic trace ID (128-bit) computed at compile time
pub const fn generate_trace_id_const(seed: u128) -> u128 {
    // FNV-1a hash for compile-time computation (128-bit)
    const FNV_OFFSET_128: u128 = 14695981039346656037;
    const FNV_PRIME_128: u128 = 1099511628211;

    let mut hash = FNV_OFFSET_128;
    let mut value = seed;

    // Process 16 bytes (128-bit value) - manually unroll loop for const fn
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);
    value >>= 8;
    hash ^= (value & 0xFF) as u128;
    hash = hash.wrapping_mul(FNV_PRIME_128);

    hash
}

/// Compile-time validation that MAX_SPANS ≤ 8
///
/// This const assertion ensures that span buffers cannot exceed the Chatman Constant.
/// Returns `true` if MAX_SPANS ≤ 8, `false` otherwise.
///
/// # Example
/// ```rust
/// const MAX_SPANS: usize = 8;
/// const IS_VALID: bool = validate_max_spans_const(MAX_SPANS);
/// assert!(IS_VALID); // Compile-time assertion
/// ```
pub const fn validate_max_spans_const(max_spans: usize) -> bool {
    max_spans <= 8
}

/// Compile-time hash computation for span attributes
///
/// Computes a hash of attribute key-value pairs at compile time.
/// This enables compile-time attribute validation and optimization.
///
/// # Arguments
/// * `key` - Attribute key (must be a string literal)
/// * `value` - Attribute value (must be a string literal)
///
/// # Returns
/// * Hash value computed at compile time
pub const fn compute_attribute_hash(key: &str, value: &str) -> u64 {
    // FNV-1a hash for compile-time computation
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET;

    // Hash key
    let key_bytes = key.as_bytes();
    let mut i = 0;
    while i < key_bytes.len() {
        hash ^= key_bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }

    // Hash separator
    hash ^= 0xFF;
    hash = hash.wrapping_mul(FNV_PRIME);

    // Hash value
    let value_bytes = value.as_bytes();
    i = 0;
    while i < value_bytes.len() {
        hash ^= value_bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }

    hash
}

/// Compile-time validation helper for span structure
///
/// Validates that span has required fields at compile time.
/// This is useful for compile-time testing and validation.
///
/// # Arguments
/// * `has_name` - Whether span has a name
/// * `has_trace_id` - Whether span has a trace ID
/// * `has_span_id` - Whether span has a span ID
///
/// # Returns
/// * `true` if span structure is valid, `false` otherwise
pub const fn validate_span_structure_const(
    has_name: bool,
    has_trace_id: bool,
    has_span_id: bool,
) -> bool {
    has_name && has_trace_id && has_span_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_span_id_const() {
        const SPAN_ID: u64 = generate_span_id_const(12345);
        assert_ne!(SPAN_ID, 0);

        // Same seed should produce same ID
        const SPAN_ID_2: u64 = generate_span_id_const(12345);
        assert_eq!(SPAN_ID, SPAN_ID_2);

        // Different seed should produce different ID
        const SPAN_ID_3: u64 = generate_span_id_const(67890);
        assert_ne!(SPAN_ID, SPAN_ID_3);
    }

    #[test]
    fn test_generate_trace_id_const() {
        const TRACE_ID: u128 = generate_trace_id_const(12345);
        assert_ne!(TRACE_ID, 0);

        // Same seed should produce same ID
        const TRACE_ID_2: u128 = generate_trace_id_const(12345);
        assert_eq!(TRACE_ID, TRACE_ID_2);
    }

    #[test]
    fn test_validate_max_spans_const() {
        const MAX_SPANS_8: usize = 8;
        const IS_VALID_8: bool = validate_max_spans_const(MAX_SPANS_8);
        assert!(IS_VALID_8);

        const MAX_SPANS_1: usize = 1;
        const IS_VALID_1: bool = validate_max_spans_const(MAX_SPANS_1);
        assert!(IS_VALID_1);

        const MAX_SPANS_9: usize = 9;
        const IS_VALID_9: bool = validate_max_spans_const(MAX_SPANS_9);
        assert!(!IS_VALID_9);
    }

    #[test]
    fn test_compute_attribute_hash() {
        const HASH1: u64 = compute_attribute_hash("key", "value");
        const HASH2: u64 = compute_attribute_hash("key", "value");
        assert_eq!(HASH1, HASH2);

        const HASH3: u64 = compute_attribute_hash("key", "different");
        assert_ne!(HASH1, HASH3);
    }

    #[test]
    fn test_validate_span_structure_const() {
        const IS_VALID: bool = validate_span_structure_const(true, true, true);
        assert!(IS_VALID);

        const IS_INVALID: bool = validate_span_structure_const(false, true, true);
        assert!(!IS_INVALID);
    }
}
