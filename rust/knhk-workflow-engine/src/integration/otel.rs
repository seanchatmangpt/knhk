//! OTEL integration

use tracing::{instrument, Span};

/// OTEL integration for workflow engine
pub struct OtelIntegration;

impl OtelIntegration {
    /// Create new OTEL integration
    pub fn new() -> Self {
        Self
    }

    /// Start a span for workflow execution
    #[instrument(skip(self))]
    pub fn start_workflow_span(&self, workflow_id: &str) -> Span {
        tracing::info_span!("workflow.execute", workflow_id = workflow_id)
    }

    /// Start a span for case execution
    #[instrument(skip(self))]
    pub fn start_case_span(&self, case_id: &str) -> Span {
        tracing::info_span!("case.execute", case_id = case_id)
    }

    /// Start a span for pattern execution
    #[instrument(skip(self))]
    pub fn start_pattern_span(&self, pattern_id: u32) -> Span {
        tracing::info_span!("pattern.execute", pattern_id = pattern_id)
    }
}

impl Default for OtelIntegration {
    fn default() -> Self {
        Self::new()
    }
}
