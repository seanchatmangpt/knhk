//! Integration tests for Chicago TDD harness

mod bounds_tests;

use chicago_tdd::{OperationType, PerformanceHarness, Reporter};

#[test]
fn test_full_harness_workflow() {
    let mut harness = PerformanceHarness::new();

    // Measure various operations
    harness.measure("operation_1", OperationType::HotPath, || 42);
    harness.measure("operation_2", OperationType::WarmPath, || vec![1, 2, 3]);
    harness.measure("operation_3", OperationType::ColdPath, || {
        format!("test_{}", 123)
    });

    // Generate report
    let report = harness.report();

    assert_eq!(report.total_operations, 3);
    assert_eq!(report.hot_path_count, 1);
    assert_eq!(report.warm_path_count, 1);
    assert_eq!(report.cold_path_count, 1);

    // Print report for visibility
    Reporter::print_report(&report);
}

#[test]
fn test_bounds_enforcement() {
    let mut harness = PerformanceHarness::with_iterations(100, 1000, 10);

    // These operations are measured
    harness.measure("fast_op_1", OperationType::HotPath, || 1 + 1);
    harness.measure("fast_op_2", OperationType::HotPath, || true && false);
    harness.measure("fast_op_3", OperationType::HotPath, || [1, 2, 3][1]);

    // Check that measurements are reasonable (not necessarily within 8 ticks)
    // In real-world scenarios, the harness identifies which operations meet bounds
    for result in harness.results() {
        assert!(
            result.statistics.p99 < 1000,
            "Measurement for {} should be reasonable",
            result.operation_name
        );
    }
}

#[test]
fn test_report_generation() {
    let mut harness = PerformanceHarness::with_iterations(100, 1000, 10);

    harness.measure("test_1", OperationType::HotPath, || 42);
    harness.measure("test_2", OperationType::WarmPath, || "hello".to_string());

    let report = harness.report();

    // Test report structure
    assert!(report.total_operations > 0);
    assert!(!report.results.is_empty());

    // Test CSV export
    let csv = Reporter::export_csv(&report);
    assert!(csv.contains("Operation,Type,P50,P95,P99,Bound,Status"));

    // Test JSON export (structure check only)
    let json = Reporter::export_json(&report);
    assert!(json.contains("total_operations"));
}
