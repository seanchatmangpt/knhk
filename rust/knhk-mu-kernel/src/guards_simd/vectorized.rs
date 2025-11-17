//! Vectorized SIMD Comparison Operations
//!
//! This module implements optimized SIMD comparison operations for guard evaluation.
//! All operations are branchless and designed for maximum throughput.

#![cfg_attr(all(feature = "simd", target_feature = "avx2"), feature(portable_simd))]

// GuardResult not used in this module

#[cfg(all(feature = "simd", target_feature = "avx2"))]
use core::simd::{mask64x8, u32x16, u64x8, Simd, SimdInt, SimdPartialOrd};

use super::{GuardBitmap, SimdGuardBatch, SIMD_BATCH_SIZE};

/// SIMD range check: min <= value <= max (AVX2 accelerated)
///
/// This performs parallel range checking on 8 values simultaneously.
///
/// # Performance
///
/// - AVX2: 3 cycles (load + 2 comparisons + combine)
/// - Scalar: 8 cycles (8 sequential comparisons)
///
/// # Algorithm
///
/// ```text
/// for i in 0..8 (parallel):
///     ge[i] = values[i] >= mins[i]
///     le[i] = values[i] <= maxs[i]
///     result[i] = ge[i] && le[i]
/// ```
#[inline]
#[cfg(all(feature = "simd", target_feature = "avx2"))]
pub fn simd_range_check(values: &[u64; 8], mins: &[u64; 8], maxs: &[u64; 8]) -> GuardBitmap {
    // Load SIMD vectors (branchless, cache-aligned)
    let v = Simd::<u64, 8>::from_array(*values);
    let min = Simd::<u64, 8>::from_array(*mins);
    let max = Simd::<u64, 8>::from_array(*maxs);

    // Parallel comparison (2 cycles)
    let ge_min = v.simd_ge(min);
    let le_max = v.simd_le(max);

    // Combine masks (1 cycle)
    let mask = ge_min & le_max;

    // Convert to bitfield (1 cycle)
    mask.to_bitmask()
}

/// SIMD threshold comparison: value >= threshold (AVX2 accelerated)
///
/// # Performance
///
/// - AVX2: 2 cycles (load + compare)
/// - Scalar: 8 cycles
#[inline]
#[cfg(all(feature = "simd", target_feature = "avx2"))]
pub fn simd_threshold_ge(values: &[u64; 8], threshold: u64) -> GuardBitmap {
    let v = Simd::<u64, 8>::from_array(*values);
    let thresh = Simd::<u64, 8>::splat(threshold);

    let mask = v.simd_ge(thresh);
    mask.to_bitmask()
}

/// SIMD threshold comparison: value <= threshold (AVX2 accelerated)
///
/// # Performance
///
/// - AVX2: 2 cycles (load + compare)
/// - Scalar: 8 cycles
#[inline]
#[cfg(all(feature = "simd", target_feature = "avx2"))]
pub fn simd_threshold_le(values: &[u64; 8], threshold: u64) -> GuardBitmap {
    let v = Simd::<u64, 8>::from_array(*values);
    let thresh = Simd::<u64, 8>::splat(threshold);

    let mask = v.simd_le(thresh);
    mask.to_bitmask()
}

/// SIMD equality check: value == expected (AVX2 accelerated)
///
/// # Performance
///
/// - AVX2: 2 cycles (load + compare)
/// - Scalar: 8 cycles
#[inline]
#[cfg(all(feature = "simd", target_feature = "avx2"))]
pub fn simd_equals(values: &[u64; 8], expected: &[u64; 8]) -> GuardBitmap {
    let v = Simd::<u64, 8>::from_array(*values);
    let exp = Simd::<u64, 8>::from_array(*expected);

    let mask = v.simd_eq(exp);
    mask.to_bitmask()
}

/// SIMD bitwise AND check: (value & mask) == expected (AVX2 accelerated)
///
/// Used for authorization bitmap checks.
///
/// # Performance
///
/// - AVX2: 3 cycles (load + AND + compare)
/// - Scalar: 16 cycles (8 AND + 8 compare)
#[inline]
#[cfg(all(feature = "simd", target_feature = "avx2"))]
pub fn simd_bitmask_check(values: &[u64; 8], masks: &[u64; 8], expected: &[u64; 8]) -> GuardBitmap {
    let v = Simd::<u64, 8>::from_array(*values);
    let m = Simd::<u64, 8>::from_array(*masks);
    let exp = Simd::<u64, 8>::from_array(*expected);

    // Bitwise AND (1 cycle)
    let masked = v & m;

    // Compare (1 cycle)
    let result = masked.simd_eq(exp);

    result.to_bitmask()
}

/// Lane-wise select: branchless conditional selection (AVX2 accelerated)
///
/// For each lane: result[i] = mask[i] ? true_vals[i] : false_vals[i]
///
/// # Performance
///
/// - AVX2: 2 cycles (load + blend)
/// - Scalar: 8 cycles (8 branches or cmov)
#[inline]
#[cfg(all(feature = "simd", target_feature = "avx2"))]
pub fn simd_select(mask: GuardBitmap, true_vals: &[u64; 8], false_vals: &[u64; 8]) -> [u64; 8] {
    let tv = Simd::<u64, 8>::from_array(*true_vals);
    let fv = Simd::<u64, 8>::from_array(*false_vals);

    // Convert bitmap to SIMD mask
    let m = mask64x8::from_bitmask(mask);

    // Branchless select (1 cycle via vblendvpd on AVX2)
    let result = m.select(tv, fv);

    result.to_array()
}

/// Horizontal AND reduction: check if all lanes are true
///
/// # Performance
///
/// - AVX2: 1 cycle (bitmap check)
/// - Scalar: 8 cycles (8 AND operations)
#[inline(always)]
pub fn simd_all(bitmap: GuardBitmap) -> bool {
    bitmap == 0xFF
}

/// Horizontal OR reduction: check if any lane is true
///
/// # Performance
///
/// - AVX2: 1 cycle (bitmap check)
/// - Scalar: 8 cycles (8 OR operations)
#[inline(always)]
pub fn simd_any(bitmap: GuardBitmap) -> bool {
    bitmap != 0
}

/// Count number of true lanes
///
/// # Performance
///
/// - AVX2: 1 cycle (popcnt instruction)
/// - Scalar: 8 cycles (8 checks + sum)
#[inline(always)]
pub fn simd_count(bitmap: GuardBitmap) -> u32 {
    bitmap.count_ones()
}

/// Find first true lane (index)
///
/// Returns index of first set bit, or 8 if none.
///
/// # Performance
///
/// - AVX2: 1 cycle (tzcnt instruction)
/// - Scalar: 8 cycles (sequential scan)
#[inline(always)]
pub fn simd_find_first(bitmap: GuardBitmap) -> usize {
    if bitmap == 0 {
        8
    } else {
        bitmap.trailing_zeros() as usize
    }
}

/// Prefetch guard data (cache hint)
///
/// Hints to CPU to prefetch guard data into L1 cache before evaluation.
/// This can hide memory latency for non-cached guard batches.
#[inline(always)]
pub fn prefetch_batch(batch: &SimdGuardBatch) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::_mm_prefetch;
        use core::arch::x86_64::_MM_HINT_T0;

        // Prefetch values, mins, maxs (3 cache lines)
        _mm_prefetch(batch.values.as_ptr() as *const i8, _MM_HINT_T0);
        _mm_prefetch(batch.mins.as_ptr() as *const i8, _MM_HINT_T0);
        _mm_prefetch(batch.maxs.as_ptr() as *const i8, _MM_HINT_T0);
    }
}

// Scalar fallbacks for when SIMD is not available

/// Scalar fallback for range check
#[inline]
#[cfg(not(all(feature = "simd", target_feature = "avx2")))]
pub fn simd_range_check(values: &[u64; 8], mins: &[u64; 8], maxs: &[u64; 8]) -> GuardBitmap {
    let mut bitmap = 0u8;
    for i in 0..8 {
        if values[i] >= mins[i] && values[i] <= maxs[i] {
            bitmap |= 1 << i;
        }
    }
    bitmap
}

/// Scalar fallback for threshold >= check
#[inline]
#[cfg(not(all(feature = "simd", target_feature = "avx2")))]
pub fn simd_threshold_ge(values: &[u64; 8], threshold: u64) -> GuardBitmap {
    let mut bitmap = 0u8;
    for i in 0..8 {
        if values[i] >= threshold {
            bitmap |= 1 << i;
        }
    }
    bitmap
}

/// Scalar fallback for threshold <= check
#[inline]
#[cfg(not(all(feature = "simd", target_feature = "avx2")))]
pub fn simd_threshold_le(values: &[u64; 8], threshold: u64) -> GuardBitmap {
    let mut bitmap = 0u8;
    for i in 0..8 {
        if values[i] <= threshold {
            bitmap |= 1 << i;
        }
    }
    bitmap
}

/// Scalar fallback for equality check
#[inline]
#[cfg(not(all(feature = "simd", target_feature = "avx2")))]
pub fn simd_equals(values: &[u64; 8], expected: &[u64; 8]) -> GuardBitmap {
    let mut bitmap = 0u8;
    for i in 0..8 {
        if values[i] == expected[i] {
            bitmap |= 1 << i;
        }
    }
    bitmap
}

/// Scalar fallback for bitmask check
#[inline]
#[cfg(not(all(feature = "simd", target_feature = "avx2")))]
pub fn simd_bitmask_check(values: &[u64; 8], masks: &[u64; 8], expected: &[u64; 8]) -> GuardBitmap {
    let mut bitmap = 0u8;
    for i in 0..8 {
        if (values[i] & masks[i]) == expected[i] {
            bitmap |= 1 << i;
        }
    }
    bitmap
}

/// Scalar fallback for select
#[inline]
#[cfg(not(all(feature = "simd", target_feature = "avx2")))]
pub fn simd_select(mask: GuardBitmap, true_vals: &[u64; 8], false_vals: &[u64; 8]) -> [u64; 8] {
    let mut result = [0u64; 8];
    for i in 0..8 {
        result[i] = if (mask & (1 << i)) != 0 {
            true_vals[i]
        } else {
            false_vals[i]
        };
    }
    result
}

/// Scalar fallback for prefetch (no-op)
#[inline(always)]
#[cfg(not(target_arch = "x86_64"))]
pub fn prefetch_batch(_batch: &SimdGuardBatch) {
    // No-op on non-x86_64 platforms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_range_check() {
        let values = [5, 10, 15, 20, 25, 30, 35, 40];
        let mins = [0, 5, 10, 15, 20, 25, 30, 35];
        let maxs = [10, 15, 20, 25, 30, 35, 40, 45];

        let bitmap = simd_range_check(&values, &mins, &maxs);
        assert_eq!(bitmap, 0xFF, "All values should be in range");
    }

    #[test]
    fn test_simd_threshold_ge() {
        let values = [5, 10, 15, 20, 25, 30, 35, 40];
        let threshold = 20;

        let bitmap = simd_threshold_ge(&values, threshold);
        // Indices 3, 4, 5, 6, 7 should pass (>= 20)
        let expected = 0b11111000;
        assert_eq!(bitmap, expected);
    }

    #[test]
    fn test_simd_threshold_le() {
        let values = [5, 10, 15, 20, 25, 30, 35, 40];
        let threshold = 20;

        let bitmap = simd_threshold_le(&values, threshold);
        // Indices 0, 1, 2, 3 should pass (<= 20)
        let expected = 0b00001111;
        assert_eq!(bitmap, expected);
    }

    #[test]
    fn test_simd_equals() {
        let values = [1, 2, 3, 4, 5, 6, 7, 8];
        let expected = [1, 2, 3, 4, 5, 6, 7, 8];

        let bitmap = simd_equals(&values, &expected);
        assert_eq!(bitmap, 0xFF, "All values should match");

        let different = [1, 2, 3, 4, 5, 6, 7, 9];
        let bitmap = simd_equals(&values, &different);
        assert_eq!(bitmap, 0b01111111, "Last value should not match");
    }

    #[test]
    fn test_simd_bitmask_check() {
        let values = [
            0b1111, 0b1010, 0b0101, 0b1100, 0b0011, 0b1001, 0b0110, 0b1111,
        ];
        let masks = [
            0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
        ];
        let expected = [
            0b1111, 0b1010, 0b0101, 0b1100, 0b0011, 0b1001, 0b0110, 0b1111,
        ];

        let bitmap = simd_bitmask_check(&values, &masks, &expected);
        assert_eq!(bitmap, 0xFF, "All masked values should match");
    }

    #[test]
    fn test_simd_select() {
        let mask = 0b10101010; // Alternating pattern
        let true_vals = [1, 1, 1, 1, 1, 1, 1, 1];
        let false_vals = [0, 0, 0, 0, 0, 0, 0, 0];

        let result = simd_select(mask, &true_vals, &false_vals);
        let expected = [0, 1, 0, 1, 0, 1, 0, 1];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_simd_all() {
        assert!(simd_all(0xFF));
        assert!(!simd_all(0xFE));
        assert!(!simd_all(0x00));
    }

    #[test]
    fn test_simd_any() {
        assert!(simd_any(0xFF));
        assert!(simd_any(0x01));
        assert!(!simd_any(0x00));
    }

    #[test]
    fn test_simd_count() {
        assert_eq!(simd_count(0xFF), 8);
        assert_eq!(simd_count(0b10101010), 4);
        assert_eq!(simd_count(0b00000001), 1);
        assert_eq!(simd_count(0x00), 0);
    }

    #[test]
    fn test_simd_find_first() {
        assert_eq!(simd_find_first(0b00000001), 0);
        assert_eq!(simd_find_first(0b00000010), 1);
        assert_eq!(simd_find_first(0b10000000), 7);
        assert_eq!(simd_find_first(0x00), 8);
    }
}
