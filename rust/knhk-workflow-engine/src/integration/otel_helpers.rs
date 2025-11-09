//! OTEL Helper Functions and Macros
//!
//! Provides convenient helper functions and macros for OpenTelemetry instrumentation
//! in the workflow engine, following Van der Aalst production OTEL requirements.

use crate::case::CaseId;
use crate::error::WorkflowResult;
use crate::integration::OtelIntegration;
use crate::parser::WorkflowSpecId;
use crate::patterns::PatternId;
use knhk_otel::{SpanContext, SpanStatus};
use std::time::Instant;

/// Helper to add common workflow attributes to a span
pub async fn add_workflow_attributes(
    otel: &OtelIntegration,
    span_ctx: SpanContext,
    case_id: Option<&CaseId>,
    spec_id: Option<&WorkflowSpecId>,
    task_id: Option<&str>,
    pattern_id: Option<&PatternId>,
) -> WorkflowResult<()> {
    if let Some(cid) = case_id {
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.workflow_engine.case_id".to_string(),
            cid.to_string(),
        )
        .await?;
    }
    if let Some(sid) = spec_id {
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.workflow_engine.spec_id".to_string(),
            sid.to_string(),
        )
        .await?;
    }
    if let Some(tid) = task_id {
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.workflow_engine.task_id".to_string(),
            tid.to_string(),
        )
        .await?;
    }
    if let Some(pid) = pattern_id {
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.workflow_engine.pattern_id".to_string(),
            pid.0.to_string(),
        )
        .await?;
    }
    Ok(())
}

/// Helper to end a span with success/latency and lifecycle transition
pub async fn end_span_with_result(
    otel: &OtelIntegration,
    span_ctx: SpanContext,
    success: bool,
    start_time: Instant,
) -> WorkflowResult<()> {
    let latency_ms = start_time.elapsed().as_millis();

    otel.add_attribute(
        span_ctx.clone(),
        "knhk.workflow_engine.success".to_string(),
        success.to_string(),
    )
    .await?;

    otel.add_attribute(
        span_ctx.clone(),
        "knhk.workflow_engine.latency_ms".to_string(),
        latency_ms.to_string(),
    )
    .await?;

    // Add lifecycle transition
    let transition = if success { "complete" } else { "cancel" };
    otel.add_lifecycle_transition(span_ctx.clone(), transition)
        .await?;

    // End span
    otel.end_span(
        span_ctx,
        if success {
            SpanStatus::Ok
        } else {
            SpanStatus::Error
        },
    )
    .await?;

    Ok(())
}

/// Helper to check for bottlenecks and add attribute
pub fn check_bottleneck(latency_ms: u128, threshold_ms: u128) -> bool {
    latency_ms > threshold_ms
}

/// Helper to add bottleneck detection attribute
pub async fn add_bottleneck_if_detected(
    otel: &OtelIntegration,
    span_ctx: SpanContext,
    latency_ms: u128,
    threshold_ms: u128,
) -> WorkflowResult<()> {
    if check_bottleneck(latency_ms, threshold_ms) {
        otel.add_attribute(
            span_ctx,
            "knhk.workflow_engine.bottleneck_detected".to_string(),
            "true".to_string(),
        )
        .await?;
    }
    Ok(())
}

/// Helper to add conformance checking attributes
pub async fn add_conformance_attributes(
    otel: &OtelIntegration,
    span_ctx: SpanContext,
    expected_pattern: Option<u32>,
    actual_pattern: u32,
) -> WorkflowResult<()> {
    if let Some(expected) = expected_pattern {
        otel.add_attribute(
            span_ctx.clone(),
            "knhk.workflow_engine.expected_pattern".to_string(),
            expected.to_string(),
        )
        .await?;

        let violation = expected != actual_pattern;
        otel.add_attribute(
            span_ctx,
            "knhk.workflow_engine.conformance_violation".to_string(),
            violation.to_string(),
        )
        .await?;
    }
    Ok(())
}

/// Helper to create a span context with trace correlation
pub fn create_trace_context(case_id: &CaseId) -> Option<SpanContext> {
    let trace_id = OtelIntegration::trace_id_from_case_id(case_id);
    Some(SpanContext {
        trace_id,
        span_id: knhk_otel::SpanId(0),
        parent_span_id: None,
        flags: 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_bottleneck() {
        assert!(check_bottleneck(1500, 1000));
        assert!(!check_bottleneck(500, 1000));
    }
}
