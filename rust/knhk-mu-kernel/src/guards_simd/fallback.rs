//! Scalar Fallback Implementations for Non-SIMD Platforms
//!
//! This module provides scalar implementations of all SIMD guard operations
//! for platforms that don't support AVX2 or when SIMD is disabled.
//!
//! # Performance
//!
//! Scalar performance is ~3-4x slower than SIMD, but provides:
//! - Identical functionality and correctness
//! - Portability to all platforms
//! - Fallback for runtime CPU feature detection

use super::{GuardBitmap, SimdGuardBatch, SIMD_BATCH_SIZE};
use crate::guards::GuardResult;

/// Scalar guard evaluation (fallback for non-SIMD platforms)
///
/// This implements the same logic as SIMD evaluation but using
/// sequential scalar operations.
///
/// # Performance
///
/// - ~8 cycles for 8 guards (1 cycle per guard)
/// - ~3-4x slower than SIMD
///
/// # Algorithm
///
/// ```text
/// for i in 0..8:
///     if mins[i] <= values[i] <= maxs[i]:
///         bitmap |= 1 << i
/// ```
#[inline]
pub fn evaluate_scalar(batch: &SimdGuardBatch) -> GuardBitmap {
    let mut bitmap: u8 = 0;

    for i in 0..SIMD_BATCH_SIZE {
        let value = batch.values[i];
        let min = batch.mins[i];
        let max = batch.maxs[i];

        // Range check (branchless via bit manipulation)
        let ge_min = (value >= min) as u8;
        let le_max = (value <= max) as u8;
        let passed = ge_min & le_max;

        // Set bit in bitmap
        bitmap |= passed << i;
    }

    bitmap
}

/// Scalar range check for multiple guards
///
/// # Performance
///
/// - ~8 cycles for 8 guards
#[inline]
pub fn range_check_scalar(
    values: &[u64; SIMD_BATCH_SIZE],
    mins: &[u64; SIMD_BATCH_SIZE],
    maxs: &[u64; SIMD_BATCH_SIZE],
) -> GuardBitmap {
    let mut bitmap: u8 = 0;

    for i in 0..SIMD_BATCH_SIZE {
        if values[i] >= mins[i] && values[i] <= maxs[i] {
            bitmap |= 1 << i;
        }
    }

    bitmap
}

/// Scalar threshold comparison (>=)
///
/// # Performance
///
/// - ~8 cycles for 8 guards
#[inline]
pub fn threshold_ge_scalar(values: &[u64; SIMD_BATCH_SIZE], threshold: u64) -> GuardBitmap {
    let mut bitmap: u8 = 0;

    for i in 0..SIMD_BATCH_SIZE {
        if values[i] >= threshold {
            bitmap |= 1 << i;
        }
    }

    bitmap
}

/// Scalar threshold comparison (<=)
///
/// # Performance
///
/// - ~8 cycles for 8 guards
#[inline]
pub fn threshold_le_scalar(values: &[u64; SIMD_BATCH_SIZE], threshold: u64) -> GuardBitmap {
    let mut bitmap: u8 = 0;

    for i in 0..SIMD_BATCH_SIZE {
        if values[i] <= threshold {
            bitmap |= 1 << i;
        }
    }

    bitmap
}

/// Scalar equality check
///
/// # Performance
///
/// - ~8 cycles for 8 guards
#[inline]
pub fn equals_scalar(
    values: &[u64; SIMD_BATCH_SIZE],
    expected: &[u64; SIMD_BATCH_SIZE],
) -> GuardBitmap {
    let mut bitmap: u8 = 0;

    for i in 0..SIMD_BATCH_SIZE {
        if values[i] == expected[i] {
            bitmap |= 1 << i;
        }
    }

    bitmap
}

/// Scalar bitmask check
///
/// # Performance
///
/// - ~16 cycles for 8 guards (AND + compare per guard)
#[inline]
pub fn bitmask_check_scalar(
    values: &[u64; SIMD_BATCH_SIZE],
    masks: &[u64; SIMD_BATCH_SIZE],
    expected: &[u64; SIMD_BATCH_SIZE],
) -> GuardBitmap {
    let mut bitmap: u8 = 0;

    for i in 0..SIMD_BATCH_SIZE {
        if (values[i] & masks[i]) == expected[i] {
            bitmap |= 1 << i;
        }
    }

    bitmap
}

/// Scalar conditional select
///
/// # Performance
///
/// - ~8 cycles for 8 guards (branch or cmov per guard)
#[inline]
pub fn select_scalar(
    mask: GuardBitmap,
    true_vals: &[u64; SIMD_BATCH_SIZE],
    false_vals: &[u64; SIMD_BATCH_SIZE],
) -> [u64; SIMD_BATCH_SIZE] {
    let mut result = [0u64; SIMD_BATCH_SIZE];

    for i in 0..SIMD_BATCH_SIZE {
        let bit_set = (mask & (1 << i)) != 0;
        result[i] = if bit_set { true_vals[i] } else { false_vals[i] };
    }

    result
}

/// Branchless scalar select (using arithmetic)
///
/// Avoids branching by using arithmetic to select values.
/// Can be faster than branching select on some CPUs.
///
/// # Performance
///
/// - ~8-12 cycles for 8 guards (arithmetic per guard)
#[inline]
pub fn select_branchless_scalar(
    mask: GuardBitmap,
    true_vals: &[u64; SIMD_BATCH_SIZE],
    false_vals: &[u64; SIMD_BATCH_SIZE],
) -> [u64; SIMD_BATCH_SIZE] {
    let mut result = [0u64; SIMD_BATCH_SIZE];

    for i in 0..SIMD_BATCH_SIZE {
        // Branchless: result = (mask ? true_val : false_val)
        // Using: result = false_val + mask * (true_val - false_val)
        let bit_set = ((mask >> i) & 1) as u64;
        let false_val = false_vals[i];
        let true_val = true_vals[i];

        // Arithmetic select (no branches)
        result[i] = false_val.wrapping_add(bit_set.wrapping_mul(true_val.wrapping_sub(false_val)));
    }

    result
}

/// CPU feature detection
///
/// Detects at runtime whether SIMD instructions are available.
/// This enables automatic fallback to scalar implementations.
pub struct CpuFeatures {
    /// AVX2 support
    pub avx2: bool,
    /// AVX512 support
    pub avx512: bool,
    /// NEON support (ARM)
    pub neon: bool,
}

impl CpuFeatures {
    /// Detect CPU features at compile-time
    ///
    /// # Note
    ///
    /// In no_std environment, feature detection is done at compile-time only.
    /// This ensures zero runtime cost but requires building with appropriate
    /// target features enabled (e.g., RUSTFLAGS="-C target-feature=+avx2").
    #[inline]
    pub const fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self {
                avx2: cfg!(target_feature = "avx2"),
                avx512: cfg!(target_feature = "avx512f"),
                neon: false,
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            Self {
                avx2: false,
                avx512: false,
                neon: cfg!(target_feature = "neon"),
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self {
                avx2: false,
                avx512: false,
                neon: false,
            }
        }
    }

    /// Check if SIMD is available
    #[inline(always)]
    pub const fn has_simd(&self) -> bool {
        self.avx2 || self.avx512 || self.neon
    }

    /// Get recommended batch size based on CPU features
    #[inline(always)]
    pub const fn recommended_batch_size(&self) -> usize {
        if self.avx512 {
            16 // AVX512 can handle 16 u64s
        } else if self.avx2 {
            8 // AVX2 handles 8 u64s
        } else if self.neon {
            4 // NEON handles 4 u64s (128-bit)
        } else {
            1 // Scalar fallback
        }
    }
}

/// Dynamic dispatch based on CPU features
///
/// Automatically selects SIMD or scalar implementation based on
/// runtime CPU feature detection.
pub struct DynamicGuardEvaluator {
    features: CpuFeatures,
}

impl DynamicGuardEvaluator {
    /// Create a new dynamic evaluator
    pub fn new() -> Self {
        Self {
            features: CpuFeatures::detect(),
        }
    }

    /// Evaluate guard batch (automatically selects SIMD or scalar)
    #[inline]
    pub fn evaluate(&self, batch: &SimdGuardBatch) -> GuardBitmap {
        #[cfg(all(feature = "simd", target_feature = "avx2"))]
        {
            if self.features.avx2 {
                return batch.evaluate(); // Use SIMD
            }
        }

        // Fallback to scalar
        evaluate_scalar(batch)
    }

    /// Get CPU features
    #[inline(always)]
    pub const fn features(&self) -> &CpuFeatures {
        &self.features
    }

    /// Check if using SIMD
    #[inline(always)]
    pub const fn is_simd_enabled(&self) -> bool {
        self.features.has_simd()
    }
}

impl Default for DynamicGuardEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_scalar() {
        let batch = SimdGuardBatch {
            values: [5, 10, 15, 20, 25, 30, 35, 40],
            mins: [0, 5, 10, 15, 20, 25, 30, 35],
            maxs: [10, 15, 20, 25, 30, 35, 40, 45],
        };

        let bitmap = evaluate_scalar(&batch);
        assert_eq!(bitmap, 0xFF, "All guards should pass");
    }

    #[test]
    fn test_range_check_scalar() {
        let values = [5, 10, 15, 20, 25, 30, 35, 40];
        let mins = [0, 15, 10, 25, 20, 35, 30, 45];
        let maxs = [10, 20, 20, 30, 30, 40, 40, 50];

        let bitmap = range_check_scalar(&values, &mins, &maxs);

        // Guards 0, 2, 4, 6 should pass
        let expected = 0b01010101;
        assert_eq!(bitmap, expected);
    }

    #[test]
    fn test_threshold_ge_scalar() {
        let values = [5, 10, 15, 20, 25, 30, 35, 40];
        let threshold = 20;

        let bitmap = threshold_ge_scalar(&values, threshold);

        // Indices 3, 4, 5, 6, 7 should pass
        let expected = 0b11111000;
        assert_eq!(bitmap, expected);
    }

    #[test]
    fn test_threshold_le_scalar() {
        let values = [5, 10, 15, 20, 25, 30, 35, 40];
        let threshold = 20;

        let bitmap = threshold_le_scalar(&values, threshold);

        // Indices 0, 1, 2, 3 should pass
        let expected = 0b00001111;
        assert_eq!(bitmap, expected);
    }

    #[test]
    fn test_equals_scalar() {
        let values = [1, 2, 3, 4, 5, 6, 7, 8];
        let expected = [1, 2, 3, 4, 5, 6, 7, 9];

        let bitmap = equals_scalar(&values, &expected);

        // All except index 7 should match
        let expected_bitmap = 0b01111111;
        assert_eq!(bitmap, expected_bitmap);
    }

    #[test]
    fn test_bitmask_check_scalar() {
        let values = [
            0b1111, 0b1010, 0b0101, 0b1100, 0b0011, 0b1001, 0b0110, 0b1111,
        ];
        let masks = [
            0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
        ];
        let expected = [
            0b1111, 0b1010, 0b0101, 0b1100, 0b0011, 0b1001, 0b0110, 0b1111,
        ];

        let bitmap = bitmask_check_scalar(&values, &masks, &expected);
        assert_eq!(bitmap, 0xFF);
    }

    #[test]
    fn test_select_scalar() {
        let mask = 0b10101010;
        let true_vals = [1, 1, 1, 1, 1, 1, 1, 1];
        let false_vals = [0, 0, 0, 0, 0, 0, 0, 0];

        let result = select_scalar(mask, &true_vals, &false_vals);
        let expected = [0, 1, 0, 1, 0, 1, 0, 1];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_select_branchless_scalar() {
        let mask = 0b10101010;
        let true_vals = [1, 1, 1, 1, 1, 1, 1, 1];
        let false_vals = [0, 0, 0, 0, 0, 0, 0, 0];

        let result = select_branchless_scalar(mask, &true_vals, &false_vals);
        let expected = [0, 1, 0, 1, 0, 1, 0, 1];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_cpu_features_detect() {
        let features = CpuFeatures::detect();

        // Just check that detection doesn't crash
        let _ = features.has_simd();
        let _ = features.recommended_batch_size();
    }

    #[test]
    fn test_dynamic_guard_evaluator() {
        let evaluator = DynamicGuardEvaluator::new();
        let batch = SimdGuardBatch {
            values: [5, 10, 15, 20, 25, 30, 35, 40],
            mins: [0, 5, 10, 15, 20, 25, 30, 35],
            maxs: [10, 15, 20, 25, 30, 35, 40, 45],
        };

        let bitmap = evaluator.evaluate(&batch);
        assert_eq!(bitmap, 0xFF, "All guards should pass");
    }
}
