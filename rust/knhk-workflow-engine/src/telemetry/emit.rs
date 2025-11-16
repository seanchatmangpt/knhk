// rust/knhk-workflow-engine/src/telemetry/emit.rs
// Telemetry emission functions for workflow execution
// Covenant 6: Observations Drive Everything

//! # Telemetry Emission
//!
//! Functions for emitting workflow execution telemetry that conforms to the
//! schema defined in `registry/knhk-workflow-engine.yaml`.
//!
//! Every observable workflow event emits structured telemetry:
//! - Workflow registration
//! - Case creation
//! - Task execution
//! - Pattern execution (all 43 Van der Aalst patterns)
//! - Decision points
//! - Resource allocation
//! - Errors and failures

use super::TelemetryContext;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Emit workflow registration telemetry
///
/// Emitted when a workflow specification is registered.
/// Maps to span: `knhk.workflow_engine.register_workflow`
pub fn emit_workflow_registered(
    ctx: &TelemetryContext,
    spec_id: &str,
    success: bool,
    latency_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = ctx.attributes.clone();
    attributes.insert("knhk.operation.name".to_string(), "register_workflow".to_string());
    attributes.insert("knhk.operation.type".to_string(), "workflow_registration".to_string());
    attributes.insert("knhk.workflow_engine.operation".to_string(), "register_workflow".to_string());
    attributes.insert("knhk.workflow_engine.spec_id".to_string(), spec_id.to_string());
    attributes.insert("knhk.workflow_engine.success".to_string(), success.to_string());
    attributes.insert("knhk.workflow_engine.latency_ms".to_string(), latency_ms.to_string());

    emit_span(
        ctx,
        "knhk.workflow_engine.register_workflow",
        attributes,
        latency_ms,
    )
}

/// Emit case creation telemetry
///
/// Emitted when a new workflow case is created.
/// Maps to span: `knhk.workflow_engine.create_case`
pub fn emit_case_created(
    ctx: &TelemetryContext,
    spec_id: &str,
    case_id: &str,
    case_state: &str,
    success: bool,
    latency_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = ctx.attributes.clone();
    attributes.insert("knhk.operation.name".to_string(), "create_case".to_string());
    attributes.insert("knhk.operation.type".to_string(), "case_creation".to_string());
    attributes.insert("knhk.workflow_engine.operation".to_string(), "create_case".to_string());
    attributes.insert("knhk.workflow_engine.spec_id".to_string(), spec_id.to_string());
    attributes.insert("knhk.workflow_engine.case_id".to_string(), case_id.to_string());
    attributes.insert("knhk.workflow_engine.case_state".to_string(), case_state.to_string());
    attributes.insert("knhk.workflow_engine.success".to_string(), success.to_string());
    attributes.insert("knhk.workflow_engine.latency_ms".to_string(), latency_ms.to_string());

    emit_span(
        ctx,
        "knhk.workflow_engine.create_case",
        attributes,
        latency_ms,
    )
}

/// Emit task execution telemetry
///
/// Emitted when a workflow task is executed.
/// Maps to span: `knhk.workflow_engine.execute_task`
pub fn emit_task_executed(
    ctx: &TelemetryContext,
    case_id: &str,
    task_id: &str,
    pattern_id: i32,
    success: bool,
    latency_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = ctx.attributes.clone();
    attributes.insert("knhk.operation.name".to_string(), "execute_task".to_string());
    attributes.insert("knhk.operation.type".to_string(), "task_execution".to_string());
    attributes.insert("knhk.workflow_engine.case_id".to_string(), case_id.to_string());
    attributes.insert("knhk.workflow_engine.task_id".to_string(), task_id.to_string());
    attributes.insert("knhk.workflow_engine.pattern_id".to_string(), pattern_id.to_string());
    attributes.insert("knhk.workflow_engine.success".to_string(), success.to_string());
    attributes.insert("knhk.workflow_engine.latency_ms".to_string(), latency_ms.to_string());

    emit_span(
        ctx,
        "knhk.workflow_engine.execute_task",
        attributes,
        latency_ms,
    )
}

/// Emit pattern execution telemetry
///
/// Emitted when a Van der Aalst workflow pattern is executed.
/// Maps to span: `knhk.workflow_engine.execute_pattern` and pattern-specific spans
pub fn emit_pattern_executed(
    ctx: &TelemetryContext,
    case_id: &str,
    pattern_id: i32,
    pattern_name: &str,
    pattern_category: &str,
    success: bool,
    latency_ms: u64,
    extra_attrs: Option<HashMap<String, String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = ctx.attributes.clone();
    attributes.insert("knhk.operation.name".to_string(), "execute_pattern".to_string());
    attributes.insert("knhk.operation.type".to_string(), "pattern_execution".to_string());
    attributes.insert("knhk.workflow_engine.case_id".to_string(), case_id.to_string());
    attributes.insert("knhk.workflow_engine.pattern_id".to_string(), pattern_id.to_string());
    attributes.insert("knhk.workflow_engine.pattern_name".to_string(), pattern_name.to_string());
    attributes.insert("knhk.workflow_engine.pattern_category".to_string(), pattern_category.to_string());
    attributes.insert("knhk.workflow_engine.success".to_string(), success.to_string());
    attributes.insert("knhk.workflow_engine.latency_ms".to_string(), latency_ms.to_string());

    // Add any extra pattern-specific attributes
    if let Some(extra) = extra_attrs {
        attributes.extend(extra);
    }

    // Emit generic pattern execution span
    emit_span(
        ctx,
        "knhk.workflow_engine.execute_pattern",
        attributes.clone(),
        latency_ms,
    )?;

    // Also emit pattern-specific span if we have mapping
    let pattern_span_name = get_pattern_span_name(pattern_id);
    if let Some(span_name) = pattern_span_name {
        emit_span(ctx, span_name, attributes, latency_ms)?;
    }

    // Record pattern execution metric
    emit_counter("knhk.workflow_engine.pattern.execution_count", 1, attributes.clone())?;
    emit_histogram("knhk.workflow_engine.pattern.execution_latency", latency_ms as f64, attributes)?;

    Ok(())
}

/// Emit case execution telemetry
///
/// Emitted when a workflow case is executed.
/// Maps to span: `knhk.workflow_engine.execute_case`
pub fn emit_case_executed(
    ctx: &TelemetryContext,
    case_id: &str,
    case_state: &str,
    success: bool,
    latency_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = ctx.attributes.clone();
    attributes.insert("knhk.operation.name".to_string(), "execute_case".to_string());
    attributes.insert("knhk.operation.type".to_string(), "case_execution".to_string());
    attributes.insert("knhk.workflow_engine.operation".to_string(), "execute_case".to_string());
    attributes.insert("knhk.workflow_engine.case_id".to_string(), case_id.to_string());
    attributes.insert("knhk.workflow_engine.case_state".to_string(), case_state.to_string());
    attributes.insert("knhk.workflow_engine.success".to_string(), success.to_string());
    attributes.insert("knhk.workflow_engine.latency_ms".to_string(), latency_ms.to_string());

    emit_span(
        ctx,
        "knhk.workflow_engine.execute_case",
        attributes,
        latency_ms,
    )
}

/// Emit XES-compatible lifecycle event
///
/// For process mining compatibility, emit XES standard events
pub fn emit_lifecycle_event(
    ctx: &TelemetryContext,
    case_id: &str,
    activity: &str,
    transition: &str, // "start", "complete", "cancel"
    resource: Option<&str>,
    role: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = ctx.attributes.clone();
    attributes.insert("knhk.workflow_engine.case_id".to_string(), case_id.to_string());
    attributes.insert("lifecycle:transition".to_string(), transition.to_string());
    attributes.insert("concept:name".to_string(), activity.to_string());

    if let Some(res) = resource {
        attributes.insert("org:resource".to_string(), res.to_string());
    }
    if let Some(r) = role {
        attributes.insert("org:role".to_string(), r.to_string());
    }

    // Add timestamp in XES format (ISO 8601)
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?;
    let iso_timestamp = format_iso8601(timestamp.as_millis() as u64);
    attributes.insert("time:timestamp".to_string(), iso_timestamp);

    emit_event(ctx, "lifecycle_event", attributes)
}

// ============================================================================
// Internal helper functions
// ============================================================================

/// Emit a span (internal)
fn emit_span(
    ctx: &TelemetryContext,
    span_name: &str,
    attributes: HashMap<String, String>,
    duration_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // In production, this would use opentelemetry-rust to emit actual spans
    // For now, we emit JSON to stdout for Weaver live-check to consume

    let span_json = serde_json::json!({
        "trace_id": ctx.trace_id,
        "span_id": ctx.span_id,
        "parent_span_id": ctx.parent_span_id,
        "name": span_name,
        "kind": "INTERNAL",
        "start_time_unix_nano": get_current_time_nanos() - (duration_ms * 1_000_000),
        "end_time_unix_nano": get_current_time_nanos(),
        "attributes": attributes,
        "status": { "code": "OK" }
    });

    // Emit to stdout (Weaver live-check can consume this)
    println!("OTEL_SPAN: {}", span_json);

    Ok(())
}

/// Emit an event (internal)
fn emit_event(
    ctx: &TelemetryContext,
    event_name: &str,
    attributes: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_json = serde_json::json!({
        "trace_id": ctx.trace_id,
        "span_id": ctx.span_id,
        "name": event_name,
        "time_unix_nano": get_current_time_nanos(),
        "attributes": attributes,
    });

    println!("OTEL_EVENT: {}", event_json);

    Ok(())
}

/// Emit a counter metric (internal)
fn emit_counter(
    metric_name: &str,
    value: i64,
    attributes: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let metric_json = serde_json::json!({
        "name": metric_name,
        "type": "counter",
        "value": value,
        "time_unix_nano": get_current_time_nanos(),
        "attributes": attributes,
    });

    println!("OTEL_METRIC: {}", metric_json);

    Ok(())
}

/// Emit a histogram metric (internal)
fn emit_histogram(
    metric_name: &str,
    value: f64,
    attributes: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let metric_json = serde_json::json!({
        "name": metric_name,
        "type": "histogram",
        "value": value,
        "time_unix_nano": get_current_time_nanos(),
        "attributes": attributes,
    });

    println!("OTEL_METRIC: {}", metric_json);

    Ok(())
}

/// Get current time in nanoseconds
fn get_current_time_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Format timestamp as ISO 8601
fn format_iso8601(millis: u64) -> String {
    // Simple ISO 8601 formatting (in production, use chrono)
    let seconds = millis / 1000;
    let ms_part = millis % 1000;
    format!("{}T{:02}:{:02}:{:02}.{:03}Z",
        "2025-01-15", // Placeholder date
        (seconds / 3600) % 24,
        (seconds / 60) % 60,
        seconds % 60,
        ms_part
    )
}

/// Map pattern ID to pattern-specific span name
fn get_pattern_span_name(pattern_id: i32) -> Option<&'static str> {
    match pattern_id {
        1 => Some("knhk.workflow_engine.pattern.sequence"),
        2 => Some("knhk.workflow_engine.pattern.parallel_split"),
        3 => Some("knhk.workflow_engine.pattern.synchronization"),
        4 => Some("knhk.workflow_engine.pattern.exclusive_choice"),
        5 => Some("knhk.workflow_engine.pattern.simple_merge"),
        6 => Some("knhk.workflow_engine.pattern.multi_choice"),
        12 => Some("knhk.workflow_engine.pattern.mi_without_sync"),
        13 => Some("knhk.workflow_engine.pattern.mi_design_time"),
        14 => Some("knhk.workflow_engine.pattern.mi_runtime"),
        15 => Some("knhk.workflow_engine.pattern.mi_no_runtime"),
        16 => Some("knhk.workflow_engine.pattern.deferred_choice"),
        19 => Some("knhk.workflow_engine.pattern.cancel_activity"),
        _ => None, // Pattern-specific spans not defined for all 43 patterns yet
    }
}

/// Flush any pending telemetry (called on shutdown)
pub fn flush_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    // In production, this would flush OTLP exporters
    println!("OTEL_FLUSH: Telemetry flushed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_workflow_registered() {
        let ctx = TelemetryContext::new("trace-123".to_string(), "span-456".to_string());
        let result = emit_workflow_registered(&ctx, "spec-789", true, 50);
        assert!(result.is_ok());
    }

    #[test]
    fn test_emit_case_created() {
        let ctx = TelemetryContext::new("trace-123".to_string(), "span-456".to_string());
        let result = emit_case_created(&ctx, "spec-789", "case-abc", "Created", true, 30);
        assert!(result.is_ok());
    }

    #[test]
    fn test_emit_pattern_executed() {
        let ctx = TelemetryContext::new("trace-123".to_string(), "span-456".to_string());
        let result = emit_pattern_executed(
            &ctx,
            "case-abc",
            1,
            "Sequence",
            "Basic Control Flow",
            true,
            10,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_span_name_mapping() {
        assert_eq!(
            get_pattern_span_name(1),
            Some("knhk.workflow_engine.pattern.sequence")
        );
        assert_eq!(
            get_pattern_span_name(2),
            Some("knhk.workflow_engine.pattern.parallel_split")
        );
        assert!(get_pattern_span_name(999).is_none());
    }
}
