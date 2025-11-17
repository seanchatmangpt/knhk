//! Regression detection tests
//!
//! Ensures performance doesn't regress over time.

use chicago_tdd::{MeasurementResult, OperationType, PerformanceHarness, Statistics};

/// Create a baseline measurement for testing
fn create_baseline(name: &str, p50: u64) -> MeasurementResult {
    let measurements = vec![p50; 1000];
    let statistics = Statistics::from_measurements(&measurements);

    MeasurementResult {
        operation_name: name.to_string(),
        operation_type: OperationType::HotPath,
        measurements,
        statistics,
        bounds_violated: false,
        overhead_ticks: 0,
    }
}

#[test]
fn test_regression_detection_pass() {
    let mut harness = PerformanceHarness::with_iterations(100, 1000, 10);

    let result = harness.measure("stable_operation", OperationType::HotPath, || 42 + 58);

    // Create baseline with similar performance
    let baseline = create_baseline("stable_operation", result.statistics.p50);

    // Should not detect regression (within 20% threshold)
    assert!(result
        .check_regression(baseline.statistics.p50, 20.0)
        .is_ok());
}

#[test]
fn test_regression_detection_fail() {
    let result = create_baseline("slow_operation", 100);
    let fast_baseline = 50; // Baseline was 50, now it's 100 (100% slower)

    // Should detect regression (exceeds 50% threshold)
    assert!(result.check_regression(fast_baseline, 50.0).is_err());
}

#[test]
fn test_regression_within_threshold() {
    let result = create_baseline("slightly_slower", 55);
    let baseline = 50; // 10% slower

    // Should pass with 20% threshold
    assert!(result.check_regression(baseline, 20.0).is_ok());

    // Should fail with 5% threshold
    assert!(result.check_regression(baseline, 5.0).is_err());
}

#[test]
fn test_batch_regression_check() {
    let mut harness = PerformanceHarness::with_iterations(100, 1000, 10);

    // Measure several operations
    let r1 = harness.measure("op1", OperationType::HotPath, || 1 + 1);
    let r2 = harness.measure("op2", OperationType::HotPath, || 2 * 2);
    let r3 = harness.measure("op3", OperationType::HotPath, || 10 / 2);

    // Create baselines using actual measurements
    let baselines = vec![
        create_baseline("op1", r1.statistics.p50),
        create_baseline("op2", r2.statistics.p50),
        create_baseline("op3", r3.statistics.p50),
    ];

    // Should pass regression check with generous threshold
    assert!(
        harness.check_regressions(&baselines, 200.0).is_ok(),
        "Regression check should pass with high threshold"
    );
}
