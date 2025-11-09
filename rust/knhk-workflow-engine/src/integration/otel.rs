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

            // Add XES-compatible attributes
            let timestamp = chrono::Utc::now().to_rfc3339();
            tracer.add_attribute(span_ctx.clone(), "time:timestamp".to_string(), timestamp);
            tracer.add_attribute(
                span_ctx.clone(),
                "lifecycle:transition".to_string(),
                "start".to_string(),
            );

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
            // Create parent context with trace_id from case_id for trace correlation
            let trace_id = Self::trace_id_from_case_id(case_id);
            let parent_ctx = Some(SpanContext {
                trace_id,
                span_id: knhk_otel::SpanId(0), // Root span
                parent_span_id: None,
                flags: 1,
            });

            let span_ctx =
                tracer.start_span("knhk.workflow_engine.create_case".to_string(), parent_ctx);

            // Add XES-compatible attributes
            let timestamp = chrono::Utc::now().to_rfc3339();
            tracer.add_attribute(span_ctx.clone(), "time:timestamp".to_string(), timestamp);
            tracer.add_attribute(
                span_ctx.clone(),
                "lifecycle:transition".to_string(),
                "start".to_string(),
            );

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
            // Create parent context with trace_id from case_id for trace correlation
            let trace_id = Self::trace_id_from_case_id(case_id);
            let parent_ctx = Some(SpanContext {
                trace_id,
                span_id: knhk_otel::SpanId(0), // Root span
                parent_span_id: None,
                flags: 1,
            });

            let span_ctx =
                tracer.start_span("knhk.workflow_engine.execute_case".to_string(), parent_ctx);

            // Add XES-compatible attributes
            let timestamp = chrono::Utc::now().to_rfc3339();
            tracer.add_attribute(span_ctx.clone(), "time:timestamp".to_string(), timestamp);
            tracer.add_attribute(
                span_ctx.clone(),
                "lifecycle:transition".to_string(),
                "start".to_string(),
            );

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
        parent_span: Option<&SpanContext>,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            // Use parent span for trace correlation, or create from case_id
            let parent_ctx = if let Some(parent) = parent_span {
                Some(parent.clone())
            } else {
                // Create parent context with trace_id from case_id
                let trace_id = Self::trace_id_from_case_id(case_id);
                Some(SpanContext {
                    trace_id,
                    span_id: knhk_otel::SpanId(0),
                    parent_span_id: None,
                    flags: 1,
                })
            };

            let span_ctx =
                tracer.start_span("knhk.workflow_engine.execute_task".to_string(), parent_ctx);

            // Add XES-compatible attributes
            let timestamp = chrono::Utc::now().to_rfc3339();
            tracer.add_attribute(span_ctx.clone(), "time:timestamp".to_string(), timestamp);
            tracer.add_attribute(
                span_ctx.clone(),
                "lifecycle:transition".to_string(),
                "start".to_string(),
            );

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
        parent_span: Option<&SpanContext>,
    ) -> WorkflowResult<Option<SpanContext>> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            // Use parent span for trace correlation, or create from case_id
            let parent_ctx = if let Some(parent) = parent_span {
                Some(parent.clone())
            } else {
                // Create parent context with trace_id from case_id
                let trace_id = Self::trace_id_from_case_id(case_id);
                Some(SpanContext {
                    trace_id,
                    span_id: knhk_otel::SpanId(0),
                    parent_span_id: None,
                    flags: 1,
                })
            };

            let span_ctx = tracer.start_span(
                "knhk.workflow_engine.execute_pattern".to_string(),
                parent_ctx,
            );

            // Add XES-compatible attributes
            let timestamp = chrono::Utc::now().to_rfc3339();
            tracer.add_attribute(span_ctx.clone(), "time:timestamp".to_string(), timestamp);
            tracer.add_attribute(
                span_ctx.clone(),
                "lifecycle:transition".to_string(),
                "start".to_string(),
            );

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
        self.start_execute_pattern_span(pattern_id, &dummy_case_id, None)
            .await
    }

    /// End a span with status and lifecycle transition
    pub async fn end_span(&self, span_ctx: SpanContext, status: SpanStatus) -> WorkflowResult<()> {
        let mut guard = self.tracer.write().await;
        if let Some(ref mut tracer) = *guard {
            // Add lifecycle transition based on status
            let transition = match status {
                SpanStatus::Ok => "complete",
                SpanStatus::Error => "cancel",
                SpanStatus::Unset => "complete",
            };
            tracer.add_attribute(
                span_ctx.clone(),
                "lifecycle:transition".to_string(),
                transition.to_string(),
            );
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

    /// Add lifecycle transition attribute (XES standard)
    /// Values: "start", "complete", "cancel"
    pub async fn add_lifecycle_transition(
        &self,
        span_ctx: SpanContext,
        transition: &str,
    ) -> WorkflowResult<()> {
        self.add_attribute(
            span_ctx,
            "lifecycle:transition".to_string(),
            transition.to_string(),
        )
        .await
    }

    /// Add timestamp attribute (XES standard, ISO 8601)
    pub async fn add_timestamp(
        &self,
        span_ctx: SpanContext,
        timestamp: &str,
    ) -> WorkflowResult<()> {
        self.add_attribute(
            span_ctx,
            "time:timestamp".to_string(),
            timestamp.to_string(),
        )
        .await
    }

    /// Add resource attributes (XES standard)
    pub async fn add_resource(
        &self,
        span_ctx: SpanContext,
        resource: Option<&str>,
        role: Option<&str>,
    ) -> WorkflowResult<()> {
        if let Some(res) = resource {
            self.add_attribute(
                span_ctx.clone(),
                "org:resource".to_string(),
                res.to_string(),
            )
            .await?;
        }
        if let Some(r) = role {
            self.add_attribute(span_ctx, "org:role".to_string(), r.to_string())
                .await?;
        }
        Ok(())
    }

    /// Add all XES-compatible attributes at once
    /// Includes lifecycle transition, timestamp, and optionally resource
    pub async fn add_xes_attributes(
        &self,
        span_ctx: SpanContext,
        transition: &str,
        timestamp: &str,
        resource: Option<&str>,
        role: Option<&str>,
    ) -> WorkflowResult<()> {
        self.add_lifecycle_transition(span_ctx.clone(), transition)
            .await?;
        self.add_timestamp(span_ctx.clone(), timestamp).await?;
        self.add_resource(span_ctx, resource, role).await?;
        Ok(())
    }

    /// Generate trace_id from case_id for trace correlation
    /// All spans for a case share the same trace_id
    pub fn trace_id_from_case_id(case_id: &CaseId) -> knhk_otel::TraceId {
        // Use case_id UUID bytes to generate trace_id
        // Convert case_id string to bytes and hash to 128-bit trace_id
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        case_id.to_string().hash(&mut hasher);
        let hash = hasher.finish();
        // Create 128-bit trace_id from hash (use hash twice for 128 bits)
        knhk_otel::TraceId((hash as u128) << 64 | hash as u128)
    }
}

impl Default for OtelIntegration {
    fn default() -> Self {
        Self::new(None)
    }
}
