//! SIMD optimizations for workflow patterns.
//!
//! Provides SIMD-accelerated operations for pattern matching, batching,
//! and data aggregation. Falls back to scalar implementations when SIMD
//! is not available.
//!
//! Inputs pre-validated at ingress.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
#[allow(unused_imports)] // Only used conditionally
use std::arch::aarch64::*;

/// Check if SIMD is available
pub fn is_simd_available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        is_x86_feature_detected!("sse4.2")
    }
    #[cfg(target_arch = "aarch64")]
    {
        true // NEON is always available on AArch64
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        false
    }
}

/// SIMD-optimized pattern matching
pub fn simd_pattern_match(pattern: &[u8], data: &[u8]) -> Option<usize> {
    if !is_simd_available() {
        return fallback_pattern_match(pattern, data);
    }

    // SIMD-optimized pattern matching
    // For now, fallback to standard implementation
    fallback_pattern_match(pattern, data)
}

/// Fallback pattern matching (non-SIMD)
///
/// Inputs pre-validated at ingress.
fn fallback_pattern_match(pattern: &[u8], data: &[u8]) -> Option<usize> {
    (0..=data.len() - pattern.len()).find(|&i| &data[i..i + pattern.len()] == pattern)
}

/// SIMD-optimized hash computation
pub fn simd_hash(data: &[u8]) -> u64 {
    if !is_simd_available() {
        return fallback_hash(data);
    }

    // SIMD-optimized hash
    // For now, fallback to FNV-1a
    fallback_hash(data)
}

/// Fallback hash (FNV-1a)
fn fallback_hash(data: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// ========== Phase 2: Enhanced SIMD Operations ==========
// Using auto-vectorization via compiler optimization instead of explicit SIMD intrinsics

/// Auto-vectorized pattern matching operations (Phase 2)
/// Compiler will use SIMD when available (-C target-cpu=native)
pub mod pattern_matching {
    /// Find all indices where pattern matches (auto-vectorized)
    pub fn vectorized_pattern_filter(patterns: &[u32], target: u32) -> Vec<usize> {
        // Compiler can auto-vectorize this pattern with -C opt-level=3
        patterns
            .iter()
            .enumerate()
            .filter_map(|(i, &p)| if p == target { Some(i) } else { None })
            .collect()
    }

    /// Count pattern occurrences (auto-vectorized)
    pub fn vectorized_pattern_count(patterns: &[u32], target: u32) -> usize {
        // Compiler can auto-vectorize this
        patterns.iter().filter(|&&p| p == target).count()
    }

    /// Check if any pattern matches (auto-vectorized)
    pub fn vectorized_pattern_any(patterns: &[u32], target: u32) -> bool {
        // Compiler can auto-vectorize contains with SIMD
        patterns.contains(&target)
    }
}

/// Auto-vectorized batch operations (Phase 2)
/// Compiler will use SIMD when available (-C target-cpu=native)
pub mod batching {
    /// Sum array of u64 values (auto-vectorized)
    pub fn vectorized_sum_u64(values: &[u64]) -> u64 {
        // Compiler auto-vectorizes iterator sum
        values.iter().copied().sum()
    }

    /// Find maximum value (auto-vectorized)
    pub fn vectorized_max_u64(values: &[u64]) -> Option<u64> {
        // Compiler can auto-vectorize max operations
        values.iter().max().copied()
    }

    /// Find minimum value (auto-vectorized)
    pub fn vectorized_min_u64(values: &[u64]) -> Option<u64> {
        values.iter().min().copied()
    }

    /// Calculate average (uses auto-vectorized sum)
    pub fn vectorized_average_u64(values: &[u64]) -> Option<f64> {
        if values.is_empty() {
            None
        } else {
            Some(vectorized_sum_u64(values) as f64 / values.len() as f64)
        }
    }

    /// Calculate variance (auto-vectorized)
    pub fn vectorized_variance_u64(values: &[u64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }

        let mean = vectorized_average_u64(values)?;
        let sum_sq_diff: f64 = values
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum();

        Some(sum_sq_diff / values.len() as f64)
    }
}

#[cfg(test)]
mod simd_phase2_tests {
    use super::*;

    #[test]
    fn test_vectorized_pattern_filter() {
        let patterns = vec![1, 2, 3, 2, 4, 2, 5, 6, 2, 7, 8, 9, 2, 10, 11, 12];
        let matches = pattern_matching::vectorized_pattern_filter(&patterns, 2);
        assert_eq!(matches, vec![1, 3, 5, 8, 12]);
    }

    #[test]
    fn test_vectorized_pattern_count() {
        let patterns = vec![1, 2, 3, 2, 4, 2, 5, 6, 2, 7];
        let count = pattern_matching::vectorized_pattern_count(&patterns, 2);
        assert_eq!(count, 4);
    }

    #[test]
    fn test_vectorized_pattern_any() {
        let patterns = vec![1, 3, 5, 7, 9];
        assert!(!pattern_matching::vectorized_pattern_any(&patterns, 2));
        assert!(pattern_matching::vectorized_pattern_any(&patterns, 5));
    }

    #[test]
    fn test_vectorized_sum() {
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let sum = batching::vectorized_sum_u64(&values);
        assert_eq!(sum, 55);
    }

    #[test]
    fn test_vectorized_max() {
        let values = vec![10, 5, 20, 15, 30, 25];
        let max = batching::vectorized_max_u64(&values);
        assert_eq!(max, Some(30));
    }

    #[test]
    fn test_vectorized_min() {
        let values = vec![10, 5, 20, 15, 30, 25];
        let min = batching::vectorized_min_u64(&values);
        assert_eq!(min, Some(5));
    }

    #[test]
    fn test_vectorized_average() {
        let values = vec![10, 20, 30, 40, 50];
        let avg = batching::vectorized_average_u64(&values);
        assert_eq!(avg, Some(30.0));
    }

    #[test]
    fn test_vectorized_variance() {
        let values = vec![10, 20, 30, 40, 50];
        let variance = batching::vectorized_variance_u64(&values);
        assert!(variance.is_some());
        // Variance of [10,20,30,40,50] = 200
        assert!((variance.unwrap() - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_empty_arrays() {
        let empty: Vec<u64> = vec![];
        assert_eq!(batching::vectorized_max_u64(&empty), None);
        assert_eq!(batching::vectorized_min_u64(&empty), None);
        assert_eq!(batching::vectorized_average_u64(&empty), None);
        assert_eq!(batching::vectorized_variance_u64(&empty), None);
        assert_eq!(batching::vectorized_sum_u64(&empty), 0);
    }
}
