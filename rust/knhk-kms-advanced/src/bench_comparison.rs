//! Performance comparison utilities
//!
//! This module provides tools for comparing SIMD vs scalar performance,
//! measuring dispatch overhead, and tracking performance metrics.

use std::time::{Duration, Instant};

/// Performance measurement result
#[derive(Debug, Clone)]
pub struct BenchResult {
    /// Operation name
    pub name: String,

    /// Total duration
    pub duration: Duration,

    /// Number of operations
    pub operations: usize,

    /// Operations per second
    pub ops_per_sec: f64,

    /// Average time per operation (nanoseconds)
    pub avg_nanos: f64,
}

impl BenchResult {
    /// Create a new benchmark result
    pub fn new(name: impl Into<String>, duration: Duration, operations: usize) -> Self {
        let dur_secs = duration.as_secs_f64();
        let ops_per_sec = if dur_secs > 0.0 {
            operations as f64 / dur_secs
        } else {
            0.0
        };

        let avg_nanos = if operations > 0 {
            duration.as_nanos() as f64 / operations as f64
        } else {
            0.0
        };

        Self {
            name: name.into(),
            duration,
            operations,
            ops_per_sec,
            avg_nanos,
        }
    }

    /// Calculate speedup relative to another result
    pub fn speedup(&self, baseline: &BenchResult) -> f64 {
        if baseline.avg_nanos > 0.0 {
            baseline.avg_nanos / self.avg_nanos
        } else {
            0.0
        }
    }

    /// Check if performance meets Chatman Constant (≤8 ticks)
    ///
    /// Assumes 3 GHz CPU (1 tick = 0.33 ns)
    pub fn meets_chatman_constant(&self) -> bool {
        const TICKS_LIMIT: f64 = 8.0;
        const NS_PER_TICK: f64 = 0.33; // 3 GHz CPU
        const NS_LIMIT: f64 = TICKS_LIMIT * NS_PER_TICK;

        self.avg_nanos <= NS_LIMIT
    }

    /// Get performance in ticks (assumes 3 GHz CPU)
    pub fn ticks(&self) -> f64 {
        const NS_PER_TICK: f64 = 0.33; // 3 GHz CPU
        self.avg_nanos / NS_PER_TICK
    }
}

impl std::fmt::Display for BenchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {:.2} ops/sec, {:.2} ns/op, {:.2} ticks/op",
            self.name,
            self.ops_per_sec,
            self.avg_nanos,
            self.ticks()
        )
    }
}

/// Performance comparison between SIMD and scalar implementations
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// SIMD result
    pub simd: BenchResult,

    /// Scalar result
    pub scalar: BenchResult,

    /// Speedup factor (scalar_time / simd_time)
    pub speedup: f64,
}

impl PerformanceComparison {
    /// Create a new performance comparison
    pub fn new(simd: BenchResult, scalar: BenchResult) -> Self {
        let speedup = simd.speedup(&scalar);
        Self {
            simd,
            scalar,
            speedup,
        }
    }

    /// Check if SIMD implementation meets performance targets
    ///
    /// Targets:
    /// - 3-4x speedup over scalar
    /// - ≤8 ticks per operation (Chatman Constant)
    pub fn meets_targets(&self) -> bool {
        const MIN_SPEEDUP: f64 = 3.0;
        self.speedup >= MIN_SPEEDUP && self.simd.meets_chatman_constant()
    }
}

impl std::fmt::Display for PerformanceComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Performance Comparison:")?;
        writeln!(f, "  SIMD:   {}", self.simd)?;
        writeln!(f, "  Scalar: {}", self.scalar)?;
        writeln!(f, "  Speedup: {:.2}x", self.speedup)?;
        writeln!(
            f,
            "  Meets targets: {}",
            if self.meets_targets() { "YES" } else { "NO" }
        )?;
        Ok(())
    }
}

/// Benchmark a function
pub fn bench<F>(name: impl Into<String>, operations: usize, mut f: F) -> BenchResult
where
    F: FnMut(),
{
    let start = Instant::now();
    f();
    let duration = start.elapsed();

    BenchResult::new(name, duration, operations)
}

/// Compare SIMD vs scalar performance
pub fn compare_simd_scalar<FS, FV>(
    name: impl Into<String>,
    operations: usize,
    simd: FS,
    scalar: FV,
) -> PerformanceComparison
where
    FS: FnMut(),
    FV: FnMut(),
{
    let simd_name = format!("{} (SIMD)", name.into());
    let scalar_name = format!("{} (Scalar)", simd_name.trim_end_matches(" (SIMD)"));

    let simd_result = bench(simd_name, operations, simd);
    let scalar_result = bench(scalar_name, operations, scalar);

    PerformanceComparison::new(simd_result, scalar_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_bench_result() {
        let result = BenchResult::new("test", Duration::from_nanos(1000), 100);

        assert_eq!(result.operations, 100);
        assert_eq!(result.avg_nanos, 10.0);
    }

    #[test]
    fn test_speedup_calculation() {
        let fast = BenchResult::new("fast", Duration::from_nanos(1000), 100);
        let slow = BenchResult::new("slow", Duration::from_nanos(4000), 100);

        assert_eq!(fast.speedup(&slow), 4.0);
    }

    #[test]
    fn test_chatman_constant() {
        // 8 ticks at 3 GHz = 2.64 ns
        let fast = BenchResult::new("fast", Duration::from_nanos(2), 1);
        assert!(fast.meets_chatman_constant());

        let slow = BenchResult::new("slow", Duration::from_nanos(10), 1);
        assert!(!slow.meets_chatman_constant());
    }

    #[test]
    fn test_performance_comparison() {
        let simd = BenchResult::new("simd", Duration::from_nanos(1000), 1000);
        let scalar = BenchResult::new("scalar", Duration::from_nanos(4000), 1000);

        let comparison = PerformanceComparison::new(simd, scalar);
        assert_eq!(comparison.speedup, 4.0);
        assert!(comparison.meets_targets());
    }

    #[test]
    fn test_bench_function() {
        let result = bench("sleep", 10, || {
            for _ in 0..10 {
                thread::sleep(Duration::from_micros(100));
            }
        });

        assert_eq!(result.operations, 10);
        assert!(result.duration.as_micros() >= 1000);
    }
}
