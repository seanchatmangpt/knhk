//! Chicago TDD tests for Tracer struct and methods

#![cfg(feature = "std")]

use knhk_otel::{SpanEvent, SpanId, SpanStatus, Tracer};

#[test]
fn test_tracer_new_creates_empty_tracer() {
    let tracer = Tracer::new();
    assert_eq!(tracer.spans().len(), 0);
    assert_eq!(tracer.metrics().len(), 0);
}

#[test]
fn test_tracer_with_otlp_exporter() {
    let tracer = Tracer::with_otlp_exporter("http://localhost:4318".to_string());
    assert_eq!(tracer.spans().len(), 0);
    assert_eq!(tracer.metrics().len(), 0);
}

#[test]
fn test_tracer_start_span_creates_span() {
    let mut tracer = Tracer::new();
    let context = tracer.start_span("knhk.operation.execute".to_string(), None);
    assert_eq!(tracer.spans().len(), 1);
    assert_ne!(context.span_id.0, 0);
    assert_ne!(context.trace_id.0, 0);
    let span = tracer.spans().first().expect("Expected span");
    assert_eq!(span.name, "knhk.operation.execute");
    assert_eq!(span.context.span_id, context.span_id);
}

#[test]
fn test_tracer_start_span_uses_parent_trace_id() {
    let mut tracer = Tracer::new();
    let parent_context = tracer.start_span("parent".to_string(), None);
    let child_context = tracer.start_span("child".to_string(), Some(parent_context.clone()));
    assert_eq!(child_context.trace_id, parent_context.trace_id);
    assert_eq!(child_context.parent_span_id, Some(parent_context.span_id));
}

#[test]
fn test_tracer_end_span_sets_end_time_and_status() {
    let mut tracer = Tracer::new();
    let context = tracer.start_span("test.span".to_string(), None);
    let start_time = tracer.spans().first().expect("Expected span").start_time_ms;
    std::thread::sleep(std::time::Duration::from_millis(10));
    tracer.end_span(context, SpanStatus::Ok);
    let span = tracer.spans().first().expect("Expected span");
    assert!(span.end_time_ms.is_some());
    assert!(span.end_time_ms.unwrap() >= start_time);
    assert_eq!(span.status, SpanStatus::Ok);
}

#[test]
fn test_tracer_add_event() {
    let mut tracer = Tracer::new();
    let context = tracer.start_span("test.span".to_string(), None);
    let event = SpanEvent {
        name: "operation.started".to_string(),
        timestamp_ms: knhk_otel::get_timestamp_ms(),
        attributes: std::collections::BTreeMap::new(),
    };
    tracer.add_event(context.clone(), event.clone());
    let span = tracer.spans().first().expect("Expected span");
    assert_eq!(span.events.len(), 1);
    assert_eq!(span.events[0].name, event.name);
}

#[test]
fn test_tracer_add_attribute() {
    let mut tracer = Tracer::new();
    let context = tracer.start_span("test.span".to_string(), None);
    tracer.add_attribute(
        context.clone(),
        "knhk.operation.name".to_string(),
        "test.operation".to_string(),
    );
    let span = tracer.spans().first().expect("Expected span");
    assert_eq!(span.attributes.len(), 1);
    assert_eq!(
        span.attributes.get("knhk.operation.name"),
        Some(&"test.operation".to_string())
    );
}

#[test]
fn test_tracer_record_metric() {
    let mut tracer = Tracer::new();
    let metric = knhk_otel::Metric {
        name: "knhk.operation.executed".to_string(),
        value: knhk_otel::MetricValue::Counter(1),
        timestamp_ms: knhk_otel::get_timestamp_ms(),
        attributes: std::collections::BTreeMap::new(),
    };
    tracer.record_metric(metric.clone());
    assert_eq!(tracer.metrics().len(), 1);
    let recorded_metric = tracer.metrics().first().expect("Expected metric");
    assert_eq!(recorded_metric.name, metric.name);
}

#[test]
fn test_tracer_get_span_by_id() {
    let mut tracer = Tracer::new();
    let context1 = tracer.start_span("span1".to_string(), None);
    let context2 = tracer.start_span("span2".to_string(), None);
    let span1 = tracer.get_span(context1.span_id);
    let span2 = tracer.get_span(context2.span_id);
    assert!(span1.is_some());
    assert_eq!(span1.unwrap().name, "span1");
    assert!(span2.is_some());
    assert_eq!(span2.unwrap().name, "span2");
}

#[test]
fn test_tracer_get_span_returns_none_for_missing_span() {
    let tracer = Tracer::new();
    let non_existent_id = SpanId(9999999999999999999u64);
    assert!(tracer.get_span(non_existent_id).is_none());
}

#[test]
fn test_tracer_export_without_exporter_returns_error() {
    let mut tracer = Tracer::new();
    let context = tracer.start_span("test.span".to_string(), None);
    tracer.end_span(context, SpanStatus::Ok);
    let export_result = tracer.export();
    assert!(export_result.is_err());
    assert!(export_result.unwrap_err().contains("exporter"));
}

#[test]
fn test_tracer_trace_hierarchy() {
    let mut tracer = Tracer::new();
    let parent_context = tracer.start_span("parent".to_string(), None);
    let child_context = tracer.start_span("child".to_string(), Some(parent_context.clone()));
    tracer.end_span(child_context.clone(), SpanStatus::Ok);
    tracer.end_span(parent_context.clone(), SpanStatus::Ok);
    assert_eq!(tracer.spans().len(), 2);
    let parent_span = tracer
        .spans()
        .iter()
        .find(|s| s.name == "parent")
        .expect("Expected parent span");
    let child_span = tracer
        .spans()
        .iter()
        .find(|s| s.name == "child")
        .expect("Expected child span");
    assert_eq!(
        child_span.context.parent_span_id,
        Some(parent_span.context.span_id)
    );
    assert_eq!(child_span.context.trace_id, parent_span.context.trace_id);
}
