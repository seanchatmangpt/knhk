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

    /// Start a span for workflow registration (matches Weaver schema: knhk.workflow_engine.register_workflow)
    pub async fn start_register_workflow_span(
        &self,
        spec_id: &WorkflowSpecId,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx =
                tracer.start_span("knhk.workflow_engine.register_workflow".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.spec_id".to_string(),
                spec_id.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.operation".to_string(),
                "register_workflow".to_string(),
            );
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// Start a span for case creation (matches Weaver schema: knhk.workflow_engine.create_case)
    pub async fn start_create_case_span(
        &self,
        spec_id: &WorkflowSpecId,
        case_id: &CaseId,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx = tracer.start_span("knhk.workflow_engine.create_case".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.spec_id".to_string(),
                spec_id.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.case_id".to_string(),
                case_id.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.operation".to_string(),
                "create_case".to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.case_state".to_string(),
                "Created".to_string(),
            );
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// Start a span for case execution (matches Weaver schema: knhk.workflow_engine.execute_case)
    pub async fn start_execute_case_span(
        &self,
        case_id: &CaseId,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx = tracer.start_span("knhk.workflow_engine.execute_case".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.case_id".to_string(),
                case_id.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.operation".to_string(),
                "execute_case".to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.case_state".to_string(),
                "Running".to_string(),
            );
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// Start a span for task execution (matches Weaver schema: knhk.workflow_engine.execute_task)
    pub async fn start_execute_task_span(
        &self,
        case_id: &CaseId,
        task_id: &str,
        pattern_id: Option<&PatternId>,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx = tracer.start_span("knhk.workflow_engine.execute_task".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.case_id".to_string(),
                case_id.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.task_id".to_string(),
                task_id.to_string(),
            );
            if let Some(pid) = pattern_id {
                tracer.add_attribute(
                    span_ctx.clone(),
                    "knhk.workflow_engine.pattern_id".to_string(),
                    pid.0.to_string(),
                );
            }
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// Start a span for pattern execution (matches Weaver schema: knhk.workflow_engine.execute_pattern)
    pub async fn start_execute_pattern_span(
        &self,
        pattern_id: &PatternId,
        case_id: &CaseId,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx =
                tracer.start_span("knhk.workflow_engine.execute_pattern".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.pattern_id".to_string(),
                pattern_id.0.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.case_id".to_string(),
                case_id.to_string(),
            );
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// Start a span for case history query (matches Weaver schema: knhk.workflow_engine.get_case_history)
    pub async fn start_get_case_history_span(
        &self,
        case_id: &CaseId,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx =
                tracer.start_span("knhk.workflow_engine.get_case_history".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.case_id".to_string(),
                case_id.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.operation".to_string(),
                "get_case_history".to_string(),
            );
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// Start a span for multiple instance task execution (matches Weaver schema: knhk.workflow_engine.execute_mi_task)
    pub async fn start_execute_mi_task_span(
        &self,
        case_id: &CaseId,
        task_id: &str,
        pattern_id: &PatternId,
        instance_count: u32,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            let span_ctx =
                tracer.start_span("knhk.workflow_engine.execute_mi_task".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.case_id".to_string(),
                case_id.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.task_id".to_string(),
                task_id.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.pattern_id".to_string(),
                pattern_id.0.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.workflow_engine.instance_count".to_string(),
                instance_count.to_string(),
            );
            Ok(Some(span_ctx))
        } else {
            Ok(None)
        }
    }

    /// Legacy method for backward compatibility
    #[deprecated(note = "Use start_execute_case_span instead")]
    pub async fn start_workflow_span(
        &self,
        workflow_id: &WorkflowSpecId,
    ) -> WorkflowResult<Option<SpanContext>> {
        self.start_register_workflow_span(workflow_id).await
    }

    /// Legacy method for backward compatibility
    #[deprecated(note = "Use start_execute_case_span instead")]
    pub async fn start_case_span(&self, case_id: &CaseId) -> WorkflowResult<Option<SpanContext>> {
        self.start_execute_case_span(case_id).await
    }

    /// Legacy method for backward compatibility
    #[deprecated(note = "Use start_execute_pattern_span instead")]
    pub async fn start_pattern_span(
        &self,
        pattern_id: &PatternId,
    ) -> WorkflowResult<Option<SpanContext>> {
        // Legacy method doesn't have case_id, so we create a dummy one
        // This is for backward compatibility only
        let dummy_case_id = CaseId::new();
        self.start_execute_pattern_span(pattern_id, &dummy_case_id)
            .await
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
