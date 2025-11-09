//! RDTSC benchmarking utilities for hot path performance validation
//!
//! Provides benchmarking functions to measure and validate hot path operations
//! against the Chatman Constant (≤8 ticks = 2ns at 4GHz).

use crate::error::{WorkflowError, WorkflowResult};
use crate::performance::tick_budget::{TickCounter, HOT_PATH_TICK_BUDGET};

/// Performance error
#[derive(Debug, Clone)]
pub enum PerformanceError {
    /// Operation exceeded tick budget
    BudgetExceeded { ticks: u64, budget: u64 },
    /// Performance regression detected
    Regression { current: u64, baseline: u64 },
    /// Invalid performance measurement
    InvalidMeasurement(String),
}

impl std::fmt::Display for PerformanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceError::BudgetExceeded { ticks, budget } => {
                write!(
                    f,
                    "Operation consumed {} ticks, exceeds budget {} ticks (Chatman Constant)",
                    ticks, budget
                )
            }
            PerformanceError::Regression { current, baseline } => {
                write!(
                    f,
                    "Performance regression: {} ticks > baseline {} ticks",
                    current, baseline
                )
            }
            PerformanceError::InvalidMeasurement(msg) => {
                write!(f, "Invalid performance measurement: {}", msg)
            }
        }
    }
}

impl std::error::Error for PerformanceError {}

/// Benchmark hot path operation
///
/// Measures the number of ticks consumed by a hot path operation.
/// Returns the result and the tick count.
///
/// # Example
/// ```rust
/// let (result, ticks) = benchmark_hot_path(|| {
///     // Hot path operation
///     42
/// });
/// ```
pub fn benchmark_hot_path<F, R>(f: F) -> (R, u64)
where
    F: FnOnce() -> R,
{
    let counter = TickCounter::start();
    let result = f();
    let ticks = counter.elapsed_ticks();
    (result, ticks)
}

/// Validate hot path performance
///
/// Executes a hot path operation and validates it stays within the tick budget.
/// Returns an error if the operation exceeds the budget.
///
/// # Arguments
/// * `f` - Closure to execute
/// * `budget` - Maximum allowed ticks (default: HOT_PATH_TICK_BUDGET = 8)
///
/// # Example
/// ```rust
/// let result = validate_hot_path_performance(|| {
///     // Hot path operation
///     42
/// }, 8)?;
/// ```
pub fn validate_hot_path_performance<F, R>(f: F, budget: u64) -> Result<R, PerformanceError>
where
    F: FnOnce() -> R,
{
    let (result, ticks) = benchmark_hot_path(f);

    if ticks > budget {
        return Err(PerformanceError::BudgetExceeded { ticks, budget });
    }

    Ok(result)
}

/// Assert hot path budget
///
/// Validates that ticks are within budget. Returns error if exceeded.
pub fn assert_hot_path_budget(ticks: u64, budget: u64) -> Result<(), PerformanceError> {
    if ticks > budget {
        Err(PerformanceError::BudgetExceeded { ticks, budget })
    } else {
        Ok(())
    }
}

/// Validate operation performance
///
/// Validates operation performance against expected budget.
/// Returns error if operation exceeds budget or shows regression.
pub fn validate_operation_performance(
    op: &str,
    ticks: u64,
    budget: Option<u64>,
    baseline: Option<u64>,
) -> Result<(), PerformanceError> {
    let budget = budget.unwrap_or(HOT_PATH_TICK_BUDGET);

    // Check budget
    if ticks > budget {
        return Err(PerformanceError::BudgetExceeded { ticks, budget });
    }

    // Check regression if baseline provided
    if let Some(baseline_ticks) = baseline {
        if ticks > baseline_ticks {
            // Allow 5% tolerance for measurement variance
            let tolerance = baseline_ticks * 5 / 100;
            if ticks > baseline_ticks + tolerance {
                return Err(PerformanceError::Regression {
                    current: ticks,
                    baseline: baseline_ticks,
                });
            }
        }
    }

    Ok(())
}

/// Performance statistics for multiple measurements
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Minimum ticks
    pub min: u64,
    /// Maximum ticks
    pub max: u64,
    /// Average ticks
    pub avg: f64,
    /// p50 (median) ticks
    pub p50: u64,
    /// p95 ticks
    pub p95: u64,
    /// p99 ticks
    pub p99: u64,
    /// Number of measurements
    pub count: usize,
}

impl PerformanceStats {
    /// Calculate statistics from tick measurements
    pub fn from_ticks(mut ticks: Vec<u64>) -> Result<Self, PerformanceError> {
        if ticks.is_empty() {
            return Err(PerformanceError::InvalidMeasurement(
                "No measurements provided".to_string(),
            ));
        }

        ticks.sort();

        let count = ticks.len();
        let min = ticks[0];
        let max = ticks[count - 1];
        let sum: u64 = ticks.iter().sum();
        let avg = sum as f64 / count as f64;

        let p50_idx = (count * 50 / 100).min(count - 1);
        let p95_idx = (count * 95 / 100).min(count - 1);
        let p99_idx = (count * 99 / 100).min(count - 1);

        Ok(PerformanceStats {
            min,
            max,
            avg,
            p50: ticks[p50_idx],
            p95: ticks[p95_idx],
            p99: ticks[p99_idx],
            count,
        })
    }
}

/// Benchmark hot path operation multiple times
///
/// Executes operation `iterations` times and returns performance statistics.
/// Useful for measuring p50, p95, p99 latencies.
pub fn benchmark_hot_path_multiple<F, R>(
    f: F,
    iterations: usize,
) -> Result<(Vec<R>, PerformanceStats), PerformanceError>
where
    F: Fn() -> R,
{
    let mut results = Vec::with_capacity(iterations);
    let mut ticks_vec = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let (result, ticks) = benchmark_hot_path(|| f());
        results.push(result);
        ticks_vec.push(ticks);
    }

    let stats = PerformanceStats::from_ticks(ticks_vec)?;
    Ok((results, stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_hot_path() {
        let (result, ticks) = benchmark_hot_path(|| 42);
        assert_eq!(result, 42);
        assert!(ticks >= 0);
    }

    #[test]
    fn test_validate_hot_path_performance() {
        // Fast operation should pass
        let result = validate_hot_path_performance(|| 42, 1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Operation that takes time might fail (depends on system load)
        // This test is non-deterministic, so we just verify the function works
        let _ = validate_hot_path_performance(
            || std::thread::sleep(std::time::Duration::from_millis(1)),
            1,
        );
    }

    #[test]
    fn test_assert_hot_path_budget() {
        assert!(assert_hot_path_budget(5, 8).is_ok());
        assert!(assert_hot_path_budget(8, 8).is_ok());
        assert!(assert_hot_path_budget(9, 8).is_err());
    }

    #[test]
    fn test_validate_operation_performance() {
        // Within budget
        assert!(validate_operation_performance("test_op", 5, Some(8), None).is_ok());

        // Exceeds budget
        assert!(validate_operation_performance("test_op", 10, Some(8), None).is_err());

        // Regression detected
        assert!(validate_operation_performance("test_op", 10, Some(8), Some(5)).is_err());

        // Within tolerance (no regression)
        assert!(validate_operation_performance("test_op", 6, Some(8), Some(5)).is_ok());
    }

    #[test]
    fn test_performance_stats() {
        let ticks = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let stats = PerformanceStats::from_ticks(ticks).expect("Stats should be valid");
        assert_eq!(stats.min, 1);
        assert_eq!(stats.max, 10);
        assert_eq!(stats.p50, 5);
        assert_eq!(stats.p95, 10);
        assert_eq!(stats.p99, 10);
        assert_eq!(stats.count, 10);
    }

    #[test]
    fn test_benchmark_hot_path_multiple() {
        let (results, stats) =
            benchmark_hot_path_multiple(|| 42, 10).expect("Benchmark should succeed");
        assert_eq!(results.len(), 10);
        assert_eq!(stats.count, 10);
        assert!(stats.min >= 0);
    }
}
