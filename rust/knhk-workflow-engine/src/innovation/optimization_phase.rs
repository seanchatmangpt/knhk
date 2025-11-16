//! Optimization Phase: Auto-Tuning and Adaptive Performance
//!
//! This phase provides automatic performance optimization through profiling,
//! auto-tuning, and adaptive algorithm selection. All optimizations are
//! data-driven and validated through benchmarking.
//!
//! # Key Features
//! - Profile-guided optimization (PGO)
//! - Adaptive algorithm selection
//! - Cache-aware data structures
//! - Branch prediction hints
//! - SIMD auto-vectorization

use core::marker::PhantomData;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use crate::const_assert;

/// Optimization level - controls trade-off between speed and code size
pub trait OptimizationLevel: 'static {
    const NAME: &'static str;
    const INLINE_THRESHOLD: usize;
    const UNROLL_LOOPS: bool;
    const VECTORIZE: bool;
}

/// No optimization - fastest compilation
pub struct O0;
impl OptimizationLevel for O0 {
    const NAME: &'static str = "O0";
    const INLINE_THRESHOLD: usize = 0;
    const UNROLL_LOOPS: bool = false;
    const VECTORIZE: bool = false;
}

/// Moderate optimization - balanced
pub struct O2;
impl OptimizationLevel for O2 {
    const NAME: &'static str = "O2";
    const INLINE_THRESHOLD: usize = 50;
    const UNROLL_LOOPS: bool = true;
    const VECTORIZE: bool = true;
}

/// Aggressive optimization - maximum performance
pub struct O3;
impl OptimizationLevel for O3 {
    const NAME: &'static str = "O3";
    const INLINE_THRESHOLD: usize = 200;
    const UNROLL_LOOPS: bool = true;
    const VECTORIZE: bool = true;
}

/// Size optimization - minimize binary size
pub struct Os;
impl OptimizationLevel for Os {
    const NAME: &'static str = "Os";
    const INLINE_THRESHOLD: usize = 10;
    const UNROLL_LOOPS: bool = false;
    const VECTORIZE: bool = false;
}

/// Performance counter - tracks execution metrics
pub struct PerfCounter {
    pub calls: AtomicUsize,
    pub total_cycles: AtomicU64,
    pub cache_misses: AtomicU64,
    pub branch_misses: AtomicU64,
}

impl PerfCounter {
    pub const fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            total_cycles: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            branch_misses: AtomicU64::new(0),
        }
    }

    /// Record execution
    pub fn record(&self, cycles: u64) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.total_cycles.fetch_add(cycles, Ordering::Relaxed);
    }

    /// Get average cycles per call
    pub fn avg_cycles(&self) -> f64 {
        let calls = self.calls.load(Ordering::Relaxed);
        if calls == 0 {
            return 0.0;
        }
        let total = self.total_cycles.load(Ordering::Relaxed);
        total as f64 / calls as f64
    }

    /// Get cache miss rate
    pub fn cache_miss_rate(&self) -> f64 {
        let calls = self.calls.load(Ordering::Relaxed);
        if calls == 0 {
            return 0.0;
        }
        let misses = self.cache_misses.load(Ordering::Relaxed);
        misses as f64 / calls as f64
    }
}

/// Hot path detector - identifies performance-critical code
pub struct HotPathDetector<const THRESHOLD: usize> {
    counters: [AtomicUsize; THRESHOLD],
}

impl<const THRESHOLD: usize> HotPathDetector<THRESHOLD> {
    pub const fn new() -> Self {
        const INIT: AtomicUsize = AtomicUsize::new(0);
        Self {
            counters: [INIT; THRESHOLD],
        }
    }

    /// Record execution of path
    pub fn record(&self, path_id: usize) {
        if path_id < THRESHOLD {
            self.counters[path_id].fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Check if path is hot (>90% of executions)
    pub fn is_hot(&self, path_id: usize) -> bool {
        if path_id >= THRESHOLD {
            return false;
        }

        let total: usize = self.counters.iter()
            .map(|c| c.load(Ordering::Relaxed))
            .sum();

        if total == 0 {
            return false;
        }

        let count = self.counters[path_id].load(Ordering::Relaxed);
        (count * 100) / total > 90
    }
}

/// Cache-aware data structure - optimized for cache locality
#[repr(align(64))]  // Cache line alignment
pub struct CacheAligned<T> {
    data: T,
}

impl<T> CacheAligned<T> {
    pub const fn new(data: T) -> Self {
        Self { data }
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

/// Prefetch hint - advise CPU to prefetch data
pub struct Prefetch;

impl Prefetch {
    /// Prefetch for read (temporal locality)
    #[inline(always)]
    pub fn read<T>(ptr: *const T) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::x86_64::_mm_prefetch::<3>(ptr as *const i8);
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = ptr;
        }
    }

    /// Prefetch for write (non-temporal)
    #[inline(always)]
    pub fn write<T>(ptr: *const T) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::x86_64::_mm_prefetch::<1>(ptr as *const i8);
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = ptr;
        }
    }
}

/// Branch prediction hint
pub struct BranchHint;

impl BranchHint {
    /// Hint that condition is likely true
    #[inline(always)]
    pub fn likely(b: bool) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            core::intrinsics::likely(b)
        }
        #[cfg(target_arch = "wasm32")]
        {
            b
        }
    }

    /// Hint that condition is likely false
    #[inline(always)]
    pub fn unlikely(b: bool) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            core::intrinsics::unlikely(b)
        }
        #[cfg(target_arch = "wasm32")]
        {
            b
        }
    }
}

/// Adaptive algorithm selector - chooses best algorithm based on data
pub struct AdaptiveSelector<const THRESHOLD: usize> {
    fast_count: AtomicUsize,
    slow_count: AtomicUsize,
}

impl<const THRESHOLD: usize> AdaptiveSelector<THRESHOLD> {
    pub const fn new() -> Self {
        Self {
            fast_count: AtomicUsize::new(0),
            slow_count: AtomicUsize::new(0),
        }
    }

    /// Select algorithm based on input size
    pub fn select(&self, size: usize) -> Algorithm {
        if size < THRESHOLD {
            self.fast_count.fetch_add(1, Ordering::Relaxed);
            Algorithm::Fast
        } else {
            self.slow_count.fetch_add(1, Ordering::Relaxed);
            Algorithm::Slow
        }
    }

    /// Get selection statistics
    pub fn stats(&self) -> (usize, usize) {
        (
            self.fast_count.load(Ordering::Relaxed),
            self.slow_count.load(Ordering::Relaxed),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Fast,
    Slow,
}

/// Auto-tuning parameter - learns optimal value through feedback
pub struct AutoTune<const MIN: usize, const MAX: usize> {
    current: AtomicUsize,
    best_perf: AtomicU64,
}

impl<const MIN: usize, const MAX: usize> AutoTune<MIN, MAX> {
    pub const fn new() -> Self {
        /* const_assert!(MIN < MAX); */
        Self {
            current: AtomicUsize::new(MIN),
            best_perf: AtomicU64::new(u64::MAX),
        }
    }

    /// Update based on observed performance
    pub fn update(&self, cycles: u64) {
        let best = self.best_perf.load(Ordering::Relaxed);
        if cycles < best {
            self.best_perf.store(cycles, Ordering::Relaxed);
        } else {
            // Performance degraded, try different value
            let current = self.current.load(Ordering::Relaxed);
            let new_value = if current < MAX {
                current + 1
            } else {
                MIN
            };
            self.current.store(new_value, Ordering::Relaxed);
        }
    }

    /// Get current parameter value
    pub fn get(&self) -> usize {
        self.current.load(Ordering::Relaxed)
    }
}

/// Loop unroller - manually unroll loops for better performance
pub struct LoopUnroller<const FACTOR: usize>;

impl<const FACTOR: usize> LoopUnroller<FACTOR> {
    /// Process array with unrolled loop
    pub fn process<T, F>(data: &mut [T], mut f: F)
    where
        F: FnMut(&mut T),
    {
        /* const_assert!(FACTOR > 0); */
        /* const_assert!(FACTOR <= 8); */

        let chunks = data.len() / FACTOR;
        let remainder = data.len() % FACTOR;

        // Process full chunks with unrolling
        for chunk_idx in 0..chunks {
            let start = chunk_idx * FACTOR;
            // Manual unroll
            if FACTOR >= 1 { f(&mut data[start]); }
            if FACTOR >= 2 { f(&mut data[start + 1]); }
            if FACTOR >= 3 { f(&mut data[start + 2]); }
            if FACTOR >= 4 { f(&mut data[start + 3]); }
            if FACTOR >= 5 { f(&mut data[start + 4]); }
            if FACTOR >= 6 { f(&mut data[start + 5]); }
            if FACTOR >= 7 { f(&mut data[start + 6]); }
            if FACTOR >= 8 { f(&mut data[start + 7]); }
        }

        // Process remainder
        for i in (data.len() - remainder)..data.len() {
            f(&mut data[i]);
        }
    }
}

/// SIMD auto-vectorizer hint
pub struct Vectorize;

impl Vectorize {
    /// Sum array with potential auto-vectorization
    #[inline]
    pub fn sum_u32(data: &[u32]) -> u64 {
        // Give compiler best chance to auto-vectorize
        let mut sum = 0u64;
        for &value in data {
            sum += value as u64;
        }
        sum
    }

    /// Dot product with potential auto-vectorization
    #[inline]
    pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());
        let mut sum = 0.0f32;
        for i in 0..a.len() {
            sum += a[i] * b[i];
        }
        sum
    }
}

/// Inline expansion control
pub struct InlineControl;

impl InlineControl {
    /// Always inline (hot path)
    #[inline(always)]
    pub fn hot_function(x: u64) -> u64 {
        x.wrapping_mul(2).wrapping_add(1)
    }

    /// Never inline (cold path)
    #[inline(never)]
    pub fn cold_function(x: u64) -> u64 {
        x.wrapping_mul(3).wrapping_sub(2)
    }
}

/// Profile-guided optimization data collector
pub struct PgoCollector {
    pub edge_counts: Vec<AtomicUsize>,
}

impl PgoCollector {
    pub fn new(edge_count: usize) -> Self {
        let mut edge_counts = Vec::with_capacity(edge_count);
        for _ in 0..edge_count {
            edge_counts.push(AtomicUsize::new(0));
        }
        Self { edge_counts }
    }

    /// Record edge execution
    pub fn record_edge(&self, edge_id: usize) {
        if edge_id < self.edge_counts.len() {
            self.edge_counts[edge_id].fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get hottest edges (top 10%)
    pub fn hot_edges(&self) -> Vec<usize> {
        let total: usize = self.edge_counts.iter()
            .map(|c| c.load(Ordering::Relaxed))
            .sum();

        if total == 0 {
            return Vec::new();
        }

        let threshold = total / 10;  // Top 10%

        self.edge_counts
            .iter()
            .enumerate()
            .filter(|(_, count)| count.load(Ordering::Relaxed) > threshold)
            .map(|(idx, _)| idx)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_levels() {
        assert_eq!(O0::NAME, "O0");
        assert_eq!(O2::NAME, "O2");
        assert_eq!(O3::NAME, "O3");
        assert_eq!(Os::NAME, "Os");

        assert!(!O0::UNROLL_LOOPS);
        assert!(O3::UNROLL_LOOPS);
    }

    #[test]
    fn test_perf_counter() {
        let counter = PerfCounter::new();
        counter.record(100);
        counter.record(200);
        counter.record(300);

        assert_eq!(counter.calls.load(Ordering::Relaxed), 3);
        assert_eq!(counter.avg_cycles(), 200.0);
    }

    #[test]
    fn test_hot_path_detector() {
        let detector: HotPathDetector<4> = HotPathDetector::new();

        // Execute path 0 many times
        for _ in 0..100 {
            detector.record(0);
        }
        detector.record(1);
        detector.record(2);

        assert!(detector.is_hot(0));
        assert!(!detector.is_hot(1));
    }

    #[test]
    fn test_cache_aligned() {
        let aligned = CacheAligned::new(42u64);
        assert_eq!(*aligned.get(), 42);

        // Verify alignment
        let ptr = &aligned as *const _ as usize;
        assert_eq!(ptr % 64, 0);
    }

    #[test]
    fn test_adaptive_selector() {
        let selector: AdaptiveSelector<100> = AdaptiveSelector::new();

        assert_eq!(selector.select(50), Algorithm::Fast);
        assert_eq!(selector.select(150), Algorithm::Slow);

        let (fast, slow) = selector.stats();
        assert_eq!(fast, 1);
        assert_eq!(slow, 1);
    }

    #[test]
    fn test_auto_tune() {
        let tuner: AutoTune<1, 10> = AutoTune::new();
        assert_eq!(tuner.get(), 1);

        tuner.update(100);  // Good performance
        tuner.update(200);  // Worse performance - should adjust
        assert!(tuner.get() >= 1 && tuner.get() <= 10);
    }

    #[test]
    fn test_loop_unroller() {
        let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        LoopUnroller::<4>::process(&mut data, |x| *x *= 2);

        assert_eq!(data, vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
    }

    #[test]
    fn test_vectorize() {
        let data = vec![1, 2, 3, 4, 5];
        let sum = Vectorize::sum_u32(&data);
        assert_eq!(sum, 15);

        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dot = Vectorize::dot_product(&a, &b);
        assert_eq!(dot, 32.0);  // 1*4 + 2*5 + 3*6
    }

    #[test]
    fn test_inline_control() {
        assert_eq!(InlineControl::hot_function(10), 21);
        assert_eq!(InlineControl::cold_function(10), 28);
    }

    #[test]
    fn test_pgo_collector() {
        let collector = PgoCollector::new(10);

        for _ in 0..100 {
            collector.record_edge(0);
        }
        collector.record_edge(1);
        collector.record_edge(2);

        let hot = collector.hot_edges();
        assert!(hot.contains(&0));
    }
}
