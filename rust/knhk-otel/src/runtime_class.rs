// rust/knhk-otel/src/runtime_class.rs
// OTEL integration for runtime classes and SLO monitoring
// Exports metrics for runtime class counts, latencies, SLO violations, and failure actions

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::format;
use crate::{Tracer, Metric, MetricValue, SpanStatus, Span};

/// Record runtime class operation count
/// 
/// # Arguments
/// * `tracer` - OTEL tracer
/// * `class` - Runtime class (R1/W1/C1)
pub fn record_runtime_class_operation(tracer: &mut Tracer, class: &str) {
    let metric = Metric {
        name: "knhk.runtime_class.operations.count".to_string(),
        value: MetricValue::Counter(1),
        timestamp_ms: get_timestamp_ms(),
        attributes: {
            let mut attrs = BTreeMap::new();
            attrs.insert("runtime_class".to_string(), class.to_string());
            attrs
        },
    };
    tracer.record_metric(metric);
}

/// Record runtime class latency histogram
/// 
/// # Arguments
/// * `tracer` - OTEL tracer
/// * `class` - Runtime class (R1/W1/C1)
/// * `latency_ns` - Latency in nanoseconds
pub fn record_runtime_class_latency(tracer: &mut Tracer, class: &str, latency_ns: u64) {
    let metric = Metric {
        name: "knhk.runtime_class.operations.latency".to_string(),
        value: MetricValue::Histogram({
            let mut v = Vec::new();
            v.push(latency_ns);
            v
        }),
        timestamp_ms: get_timestamp_ms(),
        attributes: {
            let mut attrs = BTreeMap::new();
            attrs.insert("runtime_class".to_string(), class.to_string());
            attrs
        },
    };
    tracer.record_metric(metric);
}

/// Record SLO violation
/// 
/// # Arguments
/// * `tracer` - OTEL tracer
/// * `class` - Runtime class that violated SLO
/// * `p99_latency_ns` - Actual p99 latency in nanoseconds
/// * `slo_threshold_ns` - SLO threshold in nanoseconds
/// * `violation_percent` - Violation percentage
pub fn record_slo_violation(
    tracer: &mut Tracer,
    class: &str,
    p99_latency_ns: u64,
    slo_threshold_ns: u64,
    violation_percent: f64,
) {
    // Create error span for SLO violation
    let span_context = tracer.start_span(
        format!("slo_violation_{}", class),
        None,
    );
    
    tracer.add_attribute(
        span_context.clone(),
        "runtime_class".to_string(),
        class.to_string(),
    );
    tracer.add_attribute(
        span_context.clone(),
        "p99_latency_ns".to_string(),
        format!("{}", p99_latency_ns),
    );
    tracer.add_attribute(
        span_context.clone(),
        "slo_threshold_ns".to_string(),
        format!("{}", slo_threshold_ns),
    );
    tracer.add_attribute(
        span_context.clone(),
        "violation_percent".to_string(),
        format!("{:.2}", violation_percent),
    );
    
    tracer.end_span(span_context, SpanStatus::Error);

    // Record violation metric
    let metric = Metric {
        name: "knhk.slo.violations.count".to_string(),
        value: MetricValue::Counter(1),
        timestamp_ms: get_timestamp_ms(),
        attributes: {
            let mut attrs = BTreeMap::new();
            attrs.insert("runtime_class".to_string(), class.to_string());
            attrs.insert("p99_latency_ns".to_string(), format!("{}", p99_latency_ns));
            attrs.insert("slo_threshold_ns".to_string(), format!("{}", slo_threshold_ns));
            attrs.insert("violation_percent".to_string(), format!("{:.2}", violation_percent));
            attrs
        },
    };
    tracer.record_metric(metric);
}

/// Record R1 failure action
/// 
/// # Arguments
/// * `tracer` - OTEL tracer
/// * `action_type` - Action type: "drop", "park", or "escalate"
pub fn record_r1_failure_action(tracer: &mut Tracer, action_type: &str) {
    let metric_name = match action_type {
        "drop" => "knhk.failure.r1.drop_count",
        "park" => "knhk.failure.r1.park_count",
        "escalate" => "knhk.failure.r1.escalate_count",
        _ => "knhk.failure.r1.unknown_count",
    };

    let metric = Metric {
        name: metric_name.to_string(),
        value: MetricValue::Counter(1),
        timestamp_ms: get_timestamp_ms(),
        attributes: {
            let mut attrs = BTreeMap::new();
            attrs.insert("action_type".to_string(), action_type.to_string());
            attrs
        },
    };
    tracer.record_metric(metric);
}

/// Record W1 failure action
/// 
/// # Arguments
/// * `tracer` - OTEL tracer
/// * `action_type` - Action type: "retry" or "cache_degrade"
/// * `retry_count` - Current retry count (for retry actions)
pub fn record_w1_failure_action(tracer: &mut Tracer, action_type: &str, retry_count: Option<u32>) {
    let metric_name = match action_type {
        "retry" => "knhk.failure.w1.retry_count",
        "cache_degrade" => "knhk.failure.w1.cache_degrade_count",
        _ => "knhk.failure.w1.unknown_count",
    };

    let mut attrs = BTreeMap::new();
    attrs.insert("action_type".to_string(), action_type.to_string());
    if let Some(count) = retry_count {
        attrs.insert("retry_count".to_string(), format!("{}", count));
    }

    let metric = Metric {
        name: metric_name.to_string(),
        value: MetricValue::Counter(1),
        timestamp_ms: get_timestamp_ms(),
        attributes: attrs,
    };
    tracer.record_metric(metric);
}

/// Record C1 failure action
/// 
/// # Arguments
/// * `tracer` - OTEL tracer
/// * `operation_id` - Operation identifier
pub fn record_c1_failure_action(tracer: &mut Tracer, operation_id: &str) {
    let metric = Metric {
        name: "knhk.failure.c1.async_finalize_count".to_string(),
        value: MetricValue::Counter(1),
        timestamp_ms: get_timestamp_ms(),
        attributes: {
            let mut attrs = BTreeMap::new();
            attrs.insert("operation_id".to_string(), operation_id.to_string());
            attrs
        },
    };
    tracer.record_metric(metric);
}

/// Get current timestamp in milliseconds
fn get_timestamp_ms() -> u64 {
    #[cfg(feature = "std")]
    {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
    #[cfg(not(feature = "std"))]
    {
        0
    }
}

#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_runtime_class_operation() {
        let mut tracer = Tracer::new();
        record_runtime_class_operation(&mut tracer, "R1");
        
        let metrics = tracer.metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "knhk.runtime_class.operations.count");
    }

    #[test]
    fn test_record_runtime_class_latency() {
        let mut tracer = Tracer::new();
        record_runtime_class_latency(&mut tracer, "R1", 2);
        
        let metrics = tracer.metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "knhk.runtime_class.operations.latency");
    }

    #[test]
    fn test_record_slo_violation() {
        let mut tracer = Tracer::new();
        record_slo_violation(&mut tracer, "R1", 5, 2, 150.0);
        
        let metrics = tracer.metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "knhk.slo.violations.count");
        
        let spans = tracer.spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].status, SpanStatus::Error);
    }

    #[test]
    fn test_record_r1_failure_action() {
        let mut tracer = Tracer::new();
        record_r1_failure_action(&mut tracer, "escalate");
        
        let metrics = tracer.metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "knhk.failure.r1.escalate_count");
    }

    #[test]
    fn test_record_w1_failure_action() {
        let mut tracer = Tracer::new();
        record_w1_failure_action(&mut tracer, "retry", Some(2));
        
        let metrics = tracer.metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "knhk.failure.w1.retry_count");
    }

    #[test]
    fn test_record_c1_failure_action() {
        let mut tracer = Tracer::new();
        record_c1_failure_action(&mut tracer, "op123");
        
        let metrics = tracer.metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "knhk.failure.c1.async_finalize_count");
    }
}

