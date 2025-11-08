//! SIMD optimizations
//!
//! Provides SIMD-optimized operations for workflow patterns.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
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
fn fallback_pattern_match(pattern: &[u8], data: &[u8]) -> Option<usize> {
    if pattern.is_empty() || data.len() < pattern.len() {
        return None;
    }

    for i in 0..=data.len() - pattern.len() {
        if &data[i..i + pattern.len()] == pattern {
            return Some(i);
        }
    }
    None
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
