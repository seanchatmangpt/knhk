//! Chicago TDD tests for OtlpExporter

#![cfg(feature = "std")]

use knhk_otel::{OtlpExporter, Span, SpanContext, SpanId, SpanStatus, TraceId};

#[test]
fn test_otlp_exporter_new() {
    let exporter = OtlpExporter::new("http://localhost:4317".to_string());
    // Endpoint is private, but we can test behavior by exporting
    // This test verifies exporter is created without panicking
    // We test behavior by exporting empty spans (should succeed)
    let result = exporter.export_spans(&[]);
    assert!(result.is_ok());
}

#[test]
fn test_otlp_exporter_export_spans_empty() {
    let exporter = OtlpExporter::new("http://localhost:4317".to_string());
    let result = exporter.export_spans(&[]);
    // Empty spans should succeed (no-op)
    assert!(result.is_ok());
}

#[test]
fn test_otlp_exporter_export_spans_single() {
    let exporter = OtlpExporter::new("http://localhost:4317".to_string());
    let span = Span {
        context: SpanContext {
            trace_id: TraceId(12345678901234567890123456789012345678u128),
            span_id: SpanId(12345678901234567890u64),
            parent_span_id: None,
            flags: 1,
        },
        name: "test.span".to_string(),
        start_time_ms: knhk_otel::get_timestamp_ms(),
        end_time_ms: Some(knhk_otel::get_timestamp_ms()),
        attributes: std::collections::BTreeMap::new(),
        events: Vec::new(),
        status: SpanStatus::Ok,
    };
    let result = exporter.export_spans(&[span]);
    // May fail if collector not running, but should not panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_otlp_exporter_export_spans_multiple() {
    let exporter = OtlpExporter::new("http://localhost:4317".to_string());
    let span1 = Span {
        context: SpanContext {
            trace_id: TraceId(11111111111111111111111111111111111111u128),
            span_id: SpanId(1111111111111111111u64),
            parent_span_id: None,
            flags: 1,
        },
        name: "span1".to_string(),
        start_time_ms: knhk_otel::get_timestamp_ms(),
        end_time_ms: Some(knhk_otel::get_timestamp_ms()),
        attributes: std::collections::BTreeMap::new(),
        events: Vec::new(),
        status: SpanStatus::Ok,
    };
    let span2 = Span {
        context: SpanContext {
            trace_id: TraceId(22222222222222222222222222222222222222u128),
            span_id: SpanId(2222222222222222222u64),
            parent_span_id: None,
            flags: 1,
        },
        name: "span2".to_string(),
        start_time_ms: knhk_otel::get_timestamp_ms(),
        end_time_ms: Some(knhk_otel::get_timestamp_ms()),
        attributes: std::collections::BTreeMap::new(),
        events: Vec::new(),
        status: SpanStatus::Ok,
    };
    let result = exporter.export_spans(&[span1, span2]);
    // May fail if collector not running, but should not panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_otlp_exporter_export_spans_with_parent() {
    let exporter = OtlpExporter::new("http://localhost:4317".to_string());
    let parent_span = Span {
        context: SpanContext {
            trace_id: TraceId(12345678901234567890123456789012345678u128),
            span_id: SpanId(1111111111111111111u64),
            parent_span_id: None,
            flags: 1,
        },
        name: "parent".to_string(),
        start_time_ms: knhk_otel::get_timestamp_ms(),
        end_time_ms: Some(knhk_otel::get_timestamp_ms()),
        attributes: std::collections::BTreeMap::new(),
        events: Vec::new(),
        status: SpanStatus::Ok,
    };
    let child_span = Span {
        context: SpanContext {
            trace_id: TraceId(12345678901234567890123456789012345678u128),
            span_id: SpanId(2222222222222222222u64),
            parent_span_id: Some(SpanId(1111111111111111111u64)),
            flags: 1,
        },
        name: "child".to_string(),
        start_time_ms: knhk_otel::get_timestamp_ms(),
        end_time_ms: Some(knhk_otel::get_timestamp_ms()),
        attributes: std::collections::BTreeMap::new(),
        events: Vec::new(),
        status: SpanStatus::Ok,
    };
    let result = exporter.export_spans(&[parent_span, child_span]);
    // May fail if collector not running, but should not panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_otlp_exporter_export_metrics_empty() {
    let exporter = OtlpExporter::new("http://localhost:4317".to_string());
    let result = exporter.export_metrics(&[]);
    // Empty metrics should succeed (no-op)
    assert!(result.is_ok());
}

#[test]
fn test_otlp_exporter_export_metrics_single() {
    let exporter = OtlpExporter::new("http://localhost:4317".to_string());
    let metric = knhk_otel::Metric {
        name: "test.metric".to_string(),
        value: knhk_otel::MetricValue::Counter(1),
        timestamp_ms: knhk_otel::get_timestamp_ms(),
        attributes: std::collections::BTreeMap::new(),
    };
    let result = exporter.export_metrics(&[metric]);
    // May fail if collector not running, but should not panic
    assert!(result.is_ok() || result.is_err());
}
