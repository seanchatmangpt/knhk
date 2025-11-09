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
        let span_ctx = if let Some(ref otel) = self.otel_integration {
            otel.start_register_workflow_span(&spec.id).await?
        } else {
            None
        };

        // Check promotion gate if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let gate_allowed = fortune5.check_promotion_gate().await?;
            if !gate_allowed {
                if let (Some(ref otel), Some(ref span)) =
                    (self.otel_integration.as_ref(), span_ctx.as_ref())
                {
                    otel.add_attribute(
                        (*span).clone(),
                        "knhk.workflow_engine.success".to_string(),
                        "false".to_string(),
                    )
                    .await?;
                    otel.add_attribute(
                        (*span).clone(),
                        "knhk.workflow_engine.latency_ms".to_string(),
                        start_time.elapsed().as_millis().to_string(),
                    )
                    .await?;
                    otel.end_span((*span).clone(), SpanStatus::Error).await?;
                }
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
                otel.add_attribute(
                    (*span).clone(),
                    "knhk.workflow_engine.success".to_string(),
                    "false".to_string(),
                )
                .await?;
                otel.add_attribute(
                    (*span).clone(),
                    "knhk.workflow_engine.latency_ms".to_string(),
                    start_time.elapsed().as_millis().to_string(),
                )
                .await?;
                otel.end_span((*span).clone(), SpanStatus::Error).await?;
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
            otel.add_attribute(
                (*span).clone(),
                "knhk.workflow_engine.success".to_string(),
                success.to_string(),
            )
            .await?;
            otel.add_attribute(
                (*span).clone(),
                "knhk.workflow_engine.latency_ms".to_string(),
                latency_ms.to_string(),
            )
            .await?;

            // Add lifecycle transition based on success
            let transition = if success { "complete" } else { "cancel" };
            otel.add_lifecycle_transition((*span).clone(), transition)
                .await?;

            otel.end_span(
                (*span).clone(),
                if success {
                    SpanStatus::Ok
                } else {
                    SpanStatus::Error
                },
            )
            .await?;
        }

        persist_result
    }
}
