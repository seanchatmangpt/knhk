//! Chicago TDD tests for MetricsHelper

#![cfg(feature = "std")]

use knhk_otel::{MetricValue, MetricsHelper, Tracer};

#[test]
fn test_metrics_helper_record_hook_latency() {
    let mut tracer = Tracer::new();
    MetricsHelper::record_hook_latency(&mut tracer, 5, "test.operation");
    assert_eq!(tracer.metrics().len(), 1);
    let metric = tracer.metrics().first().expect("Expected metric");
    assert_eq!(metric.name, "knhk.hook.latency.ticks");
    match &metric.value {
        MetricValue::Histogram(buckets) => {
            assert_eq!(buckets.len(), 1);
            assert_eq!(buckets[0], 5);
        }
        _ => panic!("Expected Histogram variant"),
    }
    assert_eq!(
        metric.attributes.get("operation"),
        Some(&"test.operation".to_string())
    );
}

#[test]
fn test_metrics_helper_record_receipt() {
    let mut tracer = Tracer::new();
    MetricsHelper::record_receipt(&mut tracer, "receipt-123");
    assert_eq!(tracer.metrics().len(), 1);
    let metric = tracer.metrics().first().expect("Expected metric");
    assert_eq!(metric.name, "knhk.receipt.generated");
    match &metric.value {
        MetricValue::Counter(count) => assert_eq!(*count, 1),
        _ => panic!("Expected Counter variant"),
    }
    assert_eq!(
        metric.attributes.get("receipt_id"),
        Some(&"receipt-123".to_string())
    );
}

#[test]
fn test_metrics_helper_record_guard_violation() {
    let mut tracer = Tracer::new();
    MetricsHelper::record_guard_violation(&mut tracer, "max_run_len");
    assert_eq!(tracer.metrics().len(), 1);
    let metric = tracer.metrics().first().expect("Expected metric");
    assert_eq!(metric.name, "knhk.guard.violation");
    match &metric.value {
        MetricValue::Counter(count) => assert_eq!(*count, 1),
        _ => panic!("Expected Counter variant"),
    }
    assert_eq!(
        metric.attributes.get("guard_type"),
        Some(&"max_run_len".to_string())
    );
}

#[test]
fn test_metrics_helper_record_warm_path_latency() {
    let mut tracer = Tracer::new();
    MetricsHelper::record_warm_path_latency(&mut tracer, 100, "test.operation");
    // record_warm_path_latency records both latency and count metrics
    assert_eq!(tracer.metrics().len(), 2);
    let latency_metric = tracer
        .metrics()
        .iter()
        .find(|m| m.name == "knhk.warm_path.operations.latency")
        .expect("Expected latency metric");
    match &latency_metric.value {
        MetricValue::Histogram(buckets) => {
            assert_eq!(buckets.len(), 1, "Histogram should have one bucket");
            assert_eq!(
                buckets[0], 100,
                "Histogram should contain the latency value"
            );
        }
        _ => panic!("Expected Histogram variant"),
    }
    assert_eq!(
        latency_metric.attributes.get("operation"),
        Some(&"test.operation".to_string())
    );
    // Verify count metric exists
    let count_metric = tracer
        .metrics()
        .iter()
        .find(|m| m.name == "knhk.warm_path.operations.count")
        .expect("Expected count metric");
    match &count_metric.value {
        MetricValue::Counter(count) => assert_eq!(*count, 1),
        _ => panic!("Expected Counter variant"),
    }
}

#[test]
fn test_metrics_helper_record_multiple_metrics() {
    let mut tracer = Tracer::new();
    MetricsHelper::record_hook_latency(&mut tracer, 5, "op1");
    MetricsHelper::record_receipt(&mut tracer, "receipt-1");
    MetricsHelper::record_guard_violation(&mut tracer, "max_run_len");
    assert_eq!(tracer.metrics().len(), 3);
}
