//! Cache-Optimized Memory Layout for SIMD Guards
//!
//! This module implements SoA (Struct of Arrays) layout transformation
//! for optimal SIMD performance and cache utilization.
//!
//! # Memory Layout
//!
//! Traditional AoS (Array of Structs):
//! ```text
//! [Guard0: value, min, max] [Guard1: value, min, max] [Guard2: value, min, max] ...
//! └─────── 24 bytes ────────┘└─────── 24 bytes ────────┘
//! ```
//! - Poor SIMD access (need to gather/scatter)
//! - Poor cache utilization (loads unused fields)
//!
//! Optimized SoA (Struct of Arrays):
//! ```text
//! Values: [v0, v1, v2, v3, v4, v5, v6, v7] ← 64 bytes, cache-aligned
//! Mins:   [m0, m1, m2, m3, m4, m5, m6, m7] ← 64 bytes, cache-aligned
//! Maxs:   [M0, M1, M2, M3, M4, M5, M6, M7] ← 64 bytes, cache-aligned
//! ```
//! - Perfect SIMD access (contiguous loads)
//! - Optimal cache utilization (only load needed fields)
//! - 256-bit aligned for AVX2

use super::{SimdGuardBatch, SIMD_ALIGNMENT, SIMD_BATCH_SIZE};
use crate::guards::GuardId;
use crate::isa::GuardContext;

/// Maximum number of guard batches in a pool
pub const MAX_GUARD_BATCHES: usize = 128;

/// Cache line size (64 bytes on x86_64)
pub const CACHE_LINE_SIZE: usize = 64;

/// Guard Batch Pool - cache-optimized storage for multiple batches
///
/// This structure stores multiple guard batches in a cache-friendly layout.
/// Each field (values, mins, maxs) is stored contiguously for all batches,
/// enabling efficient prefetching and cache utilization.
#[repr(C, align(256))]
pub struct GuardBatchPool {
    /// Number of active batches
    count: usize,

    /// Batch values (SoA layout)
    /// [batch0_values..., batch1_values..., batch2_values...]
    values: [[u64; SIMD_BATCH_SIZE]; MAX_GUARD_BATCHES],

    /// Batch minimums (SoA layout)
    mins: [[u64; SIMD_BATCH_SIZE]; MAX_GUARD_BATCHES],

    /// Batch maximums (SoA layout)
    maxs: [[u64; SIMD_BATCH_SIZE]; MAX_GUARD_BATCHES],
}

impl GuardBatchPool {
    /// Create a new guard batch pool
    pub const fn new() -> Self {
        Self {
            count: 0,
            values: [[0; SIMD_BATCH_SIZE]; MAX_GUARD_BATCHES],
            mins: [[0; SIMD_BATCH_SIZE]; MAX_GUARD_BATCHES],
            maxs: [[u64::MAX; SIMD_BATCH_SIZE]; MAX_GUARD_BATCHES],
        }
    }

    /// Add a batch to the pool
    ///
    /// # Returns
    ///
    /// Index of the added batch, or None if pool is full
    pub fn add_batch(&mut self, batch: &SimdGuardBatch) -> Option<usize> {
        if self.count >= MAX_GUARD_BATCHES {
            return None;
        }

        let idx = self.count;
        self.values[idx] = batch.values;
        self.mins[idx] = batch.mins;
        self.maxs[idx] = batch.maxs;
        self.count += 1;

        Some(idx)
    }

    /// Get a batch from the pool
    pub fn get_batch(&self, index: usize) -> Option<SimdGuardBatch> {
        if index >= self.count {
            return None;
        }

        Some(SimdGuardBatch {
            values: self.values[index],
            mins: self.mins[index],
            maxs: self.maxs[index],
        })
    }

    /// Clear all batches
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Get number of batches
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Check if pool is empty
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if pool is full
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.count >= MAX_GUARD_BATCHES
    }

    /// Get total number of guards in pool
    #[inline(always)]
    pub const fn total_guards(&self) -> usize {
        self.count * SIMD_BATCH_SIZE
    }

    /// Prefetch all batches into cache
    ///
    /// This hints to the CPU to load all guard data into L1 cache
    /// before evaluation, hiding memory latency.
    pub fn prefetch_all(&self) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};

            for i in 0..self.count {
                // Prefetch values, mins, maxs for each batch
                _mm_prefetch(self.values[i].as_ptr() as *const i8, _MM_HINT_T0);
                _mm_prefetch(self.mins[i].as_ptr() as *const i8, _MM_HINT_T0);
                _mm_prefetch(self.maxs[i].as_ptr() as *const i8, _MM_HINT_T0);
            }
        }
    }
}

impl Default for GuardBatchPool {
    fn default() -> Self {
        Self::new()
    }
}

/// AoS to SoA converter - transforms guard contexts to SIMD batches
///
/// This takes an array of guard contexts (AoS layout) and converts them
/// to optimized SIMD batches (SoA layout) for efficient evaluation.
pub struct AosToSoaConverter {
    /// Current batch being built
    current_batch: SimdGuardBatch,
    /// Number of guards in current batch
    batch_size: usize,
    /// Completed batches
    batches: alloc::vec::Vec<SimdGuardBatch>,
}

impl AosToSoaConverter {
    /// Create a new converter
    pub fn new() -> Self {
        Self {
            current_batch: SimdGuardBatch::new(),
            batch_size: 0,
            batches: alloc::vec::Vec::new(),
        }
    }

    /// Add a guard context
    ///
    /// Extracts guard parameters from context and adds to current batch.
    /// When batch is full, it's moved to completed batches.
    pub fn add_context(
        &mut self,
        ctx: &GuardContext,
        value_idx: usize,
        min_idx: usize,
        max_idx: usize,
    ) {
        if self.batch_size >= SIMD_BATCH_SIZE {
            // Batch full - move to completed batches
            self.batches.push(self.current_batch);
            self.current_batch = SimdGuardBatch::new();
            self.batch_size = 0;
        }

        // Extract guard parameters from context
        let value = ctx.params.get(value_idx).copied().unwrap_or(0);
        let min = ctx.params.get(min_idx).copied().unwrap_or(0);
        let max = ctx.params.get(max_idx).copied().unwrap_or(u64::MAX);

        // Add to current batch
        self.current_batch.values[self.batch_size] = value;
        self.current_batch.mins[self.batch_size] = min;
        self.current_batch.maxs[self.batch_size] = max;
        self.batch_size += 1;
    }

    /// Finish conversion and get all batches
    ///
    /// Flushes any partial batch and returns all completed batches.
    pub fn finish(mut self) -> alloc::vec::Vec<SimdGuardBatch> {
        if self.batch_size > 0 {
            // Flush partial batch
            self.batches.push(self.current_batch);
        }

        self.batches
    }

    /// Get number of completed batches
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }
}

impl Default for AosToSoaConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache-line aligned guard storage
///
/// Ensures each guard batch starts on a cache line boundary,
/// preventing false sharing and optimizing cache utilization.
#[repr(C, align(64))]
pub struct CacheAlignedBatch {
    /// The guard batch
    pub batch: SimdGuardBatch,
    /// Padding to ensure cache line alignment
    _padding: [u8; CACHE_LINE_SIZE - core::mem::size_of::<SimdGuardBatch>() % CACHE_LINE_SIZE],
}

impl CacheAlignedBatch {
    /// Create a new cache-aligned batch
    pub const fn new() -> Self {
        Self {
            batch: SimdGuardBatch::new(),
            _padding: [0; CACHE_LINE_SIZE
                - core::mem::size_of::<SimdGuardBatch>() % CACHE_LINE_SIZE],
        }
    }

    /// Create from existing batch
    pub const fn from_batch(batch: SimdGuardBatch) -> Self {
        Self {
            batch,
            _padding: [0; CACHE_LINE_SIZE
                - core::mem::size_of::<SimdGuardBatch>() % CACHE_LINE_SIZE],
        }
    }
}

impl Default for CacheAlignedBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory statistics for guard batches
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Bytes used for guard data
    pub guard_bytes: usize,
    /// Bytes used for alignment padding
    pub padding_bytes: usize,
    /// Number of cache lines used
    pub cache_lines: usize,
    /// Cache utilization percentage (0-100)
    pub utilization_percent: u8,
}

impl MemoryStats {
    /// Calculate memory statistics for a batch pool
    pub fn from_pool(pool: &GuardBatchPool) -> Self {
        let total_bytes = core::mem::size_of::<GuardBatchPool>();
        let guard_bytes = pool.count * 3 * SIMD_BATCH_SIZE * core::mem::size_of::<u64>();
        let padding_bytes = total_bytes - guard_bytes;
        let cache_lines = total_bytes / CACHE_LINE_SIZE;
        let utilization_percent = ((guard_bytes * 100) / total_bytes) as u8;

        Self {
            total_bytes,
            guard_bytes,
            padding_bytes,
            cache_lines,
            utilization_percent,
        }
    }

    /// Calculate memory statistics for a single batch
    pub fn from_batch(batch: &SimdGuardBatch) -> Self {
        let total_bytes = core::mem::size_of::<SimdGuardBatch>();
        let guard_bytes = 3 * SIMD_BATCH_SIZE * core::mem::size_of::<u64>();
        let padding_bytes = total_bytes - guard_bytes;
        let cache_lines = (total_bytes + CACHE_LINE_SIZE - 1) / CACHE_LINE_SIZE;
        let utilization_percent = ((guard_bytes * 100) / total_bytes) as u8;

        Self {
            total_bytes,
            guard_bytes,
            padding_bytes,
            cache_lines,
            utilization_percent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_batch_pool_creation() {
        let pool = GuardBatchPool::new();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
        assert!(!pool.is_full());
    }

    #[test]
    fn test_guard_batch_pool_add_get() {
        let mut pool = GuardBatchPool::new();
        let batch = SimdGuardBatch {
            values: [1, 2, 3, 4, 5, 6, 7, 8],
            mins: [0, 0, 0, 0, 0, 0, 0, 0],
            maxs: [10, 10, 10, 10, 10, 10, 10, 10],
        };

        let idx = pool.add_batch(&batch).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(pool.len(), 1);

        let retrieved = pool.get_batch(idx).unwrap();
        assert_eq!(retrieved.values, batch.values);
        assert_eq!(retrieved.mins, batch.mins);
        assert_eq!(retrieved.maxs, batch.maxs);
    }

    #[test]
    fn test_guard_batch_pool_clear() {
        let mut pool = GuardBatchPool::new();
        let batch = SimdGuardBatch::new();

        pool.add_batch(&batch).unwrap();
        pool.add_batch(&batch).unwrap();
        assert_eq!(pool.len(), 2);

        pool.clear();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_aos_to_soa_converter() {
        let mut converter = AosToSoaConverter::new();

        // Add 10 guard contexts (should create 2 batches: 8 + 2)
        for i in 0..10 {
            let ctx = GuardContext {
                task_id: i,
                obs_data: 0,
                params: [i * 10, 0, 100, 0],
            };
            converter.add_context(&ctx, 0, 1, 2);
        }

        let batches = converter.finish();
        assert_eq!(batches.len(), 2, "Should create 2 batches");

        // Check first batch (8 guards)
        assert_eq!(batches[0].values[0], 0);
        assert_eq!(batches[0].values[7], 70);

        // Check second batch (2 guards + 6 defaults)
        assert_eq!(batches[1].values[0], 80);
        assert_eq!(batches[1].values[1], 90);
    }

    #[test]
    fn test_cache_aligned_batch() {
        let batch = CacheAlignedBatch::new();
        let ptr = &batch as *const _ as usize;

        // Should be cache-line aligned
        assert_eq!(ptr % CACHE_LINE_SIZE, 0);
    }

    #[test]
    fn test_memory_stats() {
        let pool = GuardBatchPool::new();
        let stats = MemoryStats::from_pool(&pool);

        assert!(stats.total_bytes > 0);
        assert!(stats.cache_lines > 0);
        assert!(stats.utilization_percent <= 100);
    }

    #[test]
    fn test_pool_alignment() {
        let pool = GuardBatchPool::new();
        let ptr = &pool as *const _ as usize;

        // Pool should be 256-bit aligned for AVX2
        assert_eq!(ptr % SIMD_ALIGNMENT, 0);
    }

    #[test]
    fn test_batch_alignment() {
        let batch = SimdGuardBatch::new();
        let ptr = &batch as *const _ as usize;

        // Batch should be 256-bit aligned
        assert_eq!(ptr % SIMD_ALIGNMENT, 0);
    }
}
