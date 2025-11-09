//! Workflow registration methods
//!
//! Handles workflow specification registration with validation and persistence.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use crate::validation::DeadlockDetector;
use knhk_otel::SpanStatus;
use std::time::Instant;

use super::engine::WorkflowEngine;

impl WorkflowEngine {
    /// Register a workflow specification with deadlock validation and Fortune 5 checks
    pub async fn register_workflow(&self, spec: WorkflowSpec) -> WorkflowResult<()> {
        let start_time = Instant::now();

        // Start OTEL span for workflow registration
        let span_ctx: Option<knhk_otel::SpanContext> = if let Some(ref otel) = self.otel_integration
        {
            otel_span!(
                otel,
                "knhk.workflow_engine.register_workflow",
                spec_id: Some(&spec.id),
            )
            .await?
        } else {
            None
        };

        // Check promotion gate if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let gate_allowed = fortune5.check_promotion_gate().await?;
            if !gate_allowed {
                otel_span_end!(
                    otel,
                    span_ctx,
                    success: false,
                    start_time: start_time
                )
                .await?;
                return Err(WorkflowError::Validation(
                    "Promotion gate blocked workflow registration".to_string(),
                ));
            }
        }

        // Validate for deadlocks before registration
        let detector = DeadlockDetector;
        let validation_result = detector.validate(&spec);

        if let Err(e) = validation_result {
            if let (Some(ref otel), Some(ref span)) =
                (self.otel_integration.as_ref(), span_ctx.as_ref())
            {
                otel_span_end!(
                    otel,
                    span_ctx,
                    success: false,
                    start_time: start_time
                )
                .await?;
            }
            return Err(e);
        }

        // Load source turtle into RDF store if available
        if let Some(ref turtle) = spec.source_turtle {
            self.load_spec_rdf(turtle).await.map_err(|e| {
                if let (Some(ref otel), Some(ref span)) =
                    (self.otel_integration.as_ref(), span_ctx.as_ref())
                {
                    let _ = otel.add_attribute(
                        (*span).clone(),
                        "knhk.workflow_engine.success".to_string(),
                        "false".to_string(),
                    );
                    let _ = otel.end_span((*span).clone(), SpanStatus::Error);
                }
                WorkflowError::Internal(format!("Failed to load spec RDF: {}", e))
            })?;
        }

        let spec_clone = spec.clone();
        self.specs.insert(spec.id, spec);

        // Persist to state store
        let store_arc = self.state_store.read().await;
        let persist_result = (*store_arc).save_spec(&spec_clone);

        let latency_ms = start_time.elapsed().as_millis();
        let success = persist_result.is_ok();

        // End OTEL span
        if let (Some(ref otel), Some(ref span)) =
            (self.otel_integration.as_ref(), span_ctx.as_ref())
        {
            otel_span_end!(
                otel,
                span_ctx,
                success: success,
                latency_ms: latency_ms
            )?;
        }

        persist_result
    }
}
