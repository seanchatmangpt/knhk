//! OTEL integration for workflow engine
//!
//! Provides comprehensive OpenTelemetry tracing for workflow execution,
//! pattern execution, and case management.

use crate::case::CaseId;
use crate::error::WorkflowResult;
use crate::parser::WorkflowSpecId;
use crate::patterns::PatternId;
use knhk_otel::{SpanContext, SpanStatus, Tracer};
use std::sync::Arc;
use tokio::sync::RwLock;

/// OTEL integration for workflow engine
pub struct OtelIntegration {
    tracer: Arc<RwLock<Option<Tracer>>>,
    otlp_endpoint: Option<String>,
}

impl OtelIntegration {
    /// Create new OTEL integration
    pub fn new(otlp_endpoint: Option<String>) -> Self {
        Self {
            tracer: Arc::new(RwLock::new(None)),
            otlp_endpoint,
        }
    }

    /// Initialize tracer with OTLP endpoint
    pub async fn initialize(&self) -> WorkflowResult<()> {
        if let Some(ref endpoint) = self.otlp_endpoint {
            let tracer = Tracer::with_otlp_exporter(endpoint.clone());
            let mut guard = self.tracer.write().await;
            *guard = Some(tracer);
        }
        Ok(())
    }

    /// Start a span for workflow execution
    pub async fn start_workflow_span(
        &self,
        workflow_id: &WorkflowSpecId,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx =
                tracer.start_span(format!("knhk.workflow.execute.{}", workflow_id), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow.id".to_string(),
                workflow_id.to_string(),
            );
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// Start a span for case execution
    pub async fn start_case_span(&self, case_id: &CaseId) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx = tracer.start_span(format!("knhk.case.execute.{}", case_id), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.case.id".to_string(),
                case_id.to_string(),
            );
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// Start a span for pattern execution
    pub async fn start_pattern_span(
        &self,
        pattern_id: &PatternId,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx = tracer.start_span(format!("knhk.pattern.execute.{}", pattern_id), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.pattern.id".to_string(),
                pattern_id.clone(),
            );
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// End a span with status
    pub async fn end_span(&self, span_ctx: SpanContext, status: SpanStatus) -> WorkflowResult<()> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            tracer.end_span(span_ctx, status);
        }
        Ok(())
    }

    /// Add attribute to span
    pub async fn add_attribute(
        &self,
        span_ctx: SpanContext,
        key: String,
        value: String,
    ) -> WorkflowResult<()> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            tracer.add_attribute(span_ctx, key, value);
        }
        Ok(())
    }

    /// Export spans
    pub async fn export(&self) -> WorkflowResult<()> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            tracer.export().map_err(|e| {
                crate::error::WorkflowError::Internal(format!("OTEL export failed: {:?}", e))
            })?;
        }
        Ok(())
    }
}

impl Default for OtelIntegration {
    fn default() -> Self {
        Self::new(None)
    }
}
