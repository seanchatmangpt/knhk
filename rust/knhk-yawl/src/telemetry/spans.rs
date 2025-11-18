//! OpenTelemetry span helpers
//!
//! Provides convenience functions for creating telemetry spans.

use tracing::{span, Level};

/// Create a workflow execution span
#[must_use]
pub fn workflow_execution_span(workflow_id: &str, instance_id: &str) -> tracing::Span {
    span!(
        Level::INFO,
        "workflow_execution",
        workflow_id = %workflow_id,
        instance_id = %instance_id
    )
}

/// Create a task execution span
#[must_use]
pub fn task_execution_span(task_id: &str, task_type: &str) -> tracing::Span {
    span!(
        Level::INFO,
        "task_execution",
        task_id = %task_id,
        task_type = %task_type
    )
}

/// Create a pattern execution span
#[must_use]
pub fn pattern_execution_span(pattern_type: &str) -> tracing::Span {
    span!(
        Level::DEBUG,
        "pattern_execution",
        pattern_type = %pattern_type
    )
}

/// Create a token operation span
#[must_use]
pub fn token_operation_span(operation: &str, token_id: &str) -> tracing::Span {
    span!(
        Level::DEBUG,
        "token_operation",
        operation = %operation,
        token_id = %token_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_span_creation() {
        let _span = workflow_execution_span("wf1", "inst1");
    }

    #[test]
    fn test_task_span_creation() {
        let _span = task_execution_span("task1", "Atomic");
    }

    #[test]
    fn test_pattern_span_creation() {
        let _span = pattern_execution_span("Sequence");
    }

    #[test]
    fn test_token_span_creation() {
        let _span = token_operation_span("create", "token123");
    }
}
