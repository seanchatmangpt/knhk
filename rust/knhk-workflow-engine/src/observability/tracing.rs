#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Distributed tracing for workflow engine

use crate::case::CaseId;
use crate::parser::WorkflowSpecId;
use tracing::{instrument, span, Level, Span};
use uuid::Uuid;

/// Tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name
    pub service_name: String,
    /// Enable distributed tracing
    pub distributed_tracing: bool,
    /// Sampling rate (0.0-1.0)
    pub sampling_rate: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "workflow-engine".to_string(),
            distributed_tracing: true,
            sampling_rate: 1.0,
        }
    }
}

/// Workflow tracer for distributed tracing
pub struct WorkflowTracer {
    config: TracingConfig,
}

impl WorkflowTracer {
    /// Create a new workflow tracer
    pub fn new(config: TracingConfig) -> Self {
        Self { config }
    }

    /// Start a span for workflow registration
    #[instrument(skip(self))]
    pub fn start_workflow_registration_span(&self, spec_id: WorkflowSpecId) -> Span {
        span!(
            Level::INFO,
            "workflow.register",
            workflow.spec_id = %spec_id,
            workflow.service = %self.config.service_name
        )
    }

    /// Start a span for case creation
    #[instrument(skip(self))]
    pub fn start_case_creation_span(&self, spec_id: WorkflowSpecId) -> Span {
        span!(
            Level::INFO,
            "case.create",
            workflow.spec_id = %spec_id,
            workflow.service = %self.config.service_name
        )
    }

    /// Start a span for case execution
    #[instrument(skip(self))]
    pub fn start_case_execution_span(&self, case_id: CaseId, spec_id: WorkflowSpecId) -> Span {
        span!(
            Level::INFO,
            "case.execute",
            case.id = %case_id,
            workflow.spec_id = %spec_id,
            workflow.service = %self.config.service_name
        )
    }

    /// Start a span for pattern execution
    #[instrument(skip(self))]
    pub fn start_pattern_execution_span(&self, pattern_id: u32, case_id: Option<CaseId>) -> Span {
        span!(
            Level::INFO,
            "pattern.execute",
            pattern.id = pattern_id,
            case.id = case_id.map(|id| id.to_string()),
            workflow.service = %self.config.service_name
        )
    }

    /// Start a span for state transition
    #[instrument(skip(self))]
    pub fn start_state_transition_span(
        &self,
        case_id: CaseId,
        from_state: &str,
        to_state: &str,
    ) -> Span {
        span!(
            Level::INFO,
            "case.state_transition",
            case.id = %case_id,
            state.from = from_state,
            state.to = to_state,
            workflow.service = %self.config.service_name
        )
    }
}

impl Default for WorkflowTracer {
    fn default() -> Self {
        Self::new(TracingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_creation() {
        let tracer = WorkflowTracer::default();
        let spec_id = WorkflowSpecId::new();
        let _span = tracer.start_workflow_registration_span(spec_id);
    }
}
