//! OpenTelemetry tracing integration for workflow engine
//!
//! Provides distributed tracing for workflow execution using knhk-otel.

use crate::error::WorkflowResult;
use knhk_otel::{SpanContext, SpanStatus, Tracer};
use std::sync::Arc;
use std::sync::Mutex;

/// Tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Enable tracing
    pub enabled: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "knhk-workflow-engine".to_string(),
            service_version: "1.0.0".to_string(),
            enabled: true,
        }
    }
}

/// Workflow tracer
pub struct WorkflowTracer {
    tracer: Arc<Mutex<Tracer>>,
    config: TracingConfig,
    /// Current active span context (for parent tracking)
    current_span: Arc<Mutex<Option<SpanContext>>>,
}

impl WorkflowTracer {
    /// Create new workflow tracer
    pub fn new(config: TracingConfig) -> WorkflowResult<Self> {
        let tracer = Tracer::new();
        Ok(Self {
            tracer: Arc::new(Mutex::new(tracer)),
            config,
            current_span: Arc::new(Mutex::new(None)),
        })
    }

    /// Start a span for workflow operation
    pub fn start_span(
        &self,
        operation: &str,
        workflow_id: &str,
        case_id: &str,
    ) -> WorkflowResult<()> {
        if !self.config.enabled {
            return Ok(()); // Legitimate: tracing disabled, no work to do
        }

        // Get parent span context if available
        let parent_span = self
            .current_span
            .lock()
            .map_err(|e| {
                crate::error::WorkflowError::Internal(format!(
                    "Failed to lock current_span mutex: {:?}",
                    e
                ))
            })?
            .clone();

        // Create span name with operation details
        let span_name = format!("{}.{}", self.config.service_name, operation);

        // Start span using knhk-otel API
        let mut tracer = self.tracer.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Failed to lock tracer mutex: {:?}", e))
        })?;
        let span_ctx = tracer.start_span(span_name, parent_span);

        // Add attributes for workflow context
        tracer.add_attribute(
            span_ctx.clone(),
            "knhk.workflow.id".to_string(),
            workflow_id.to_string(),
        );
        tracer.add_attribute(
            span_ctx.clone(),
            "knhk.case.id".to_string(),
            case_id.to_string(),
        );
        tracer.add_attribute(
            span_ctx.clone(),
            "knhk.service.name".to_string(),
            self.config.service_name.clone(),
        );
        tracer.add_attribute(
            span_ctx.clone(),
            "knhk.service.version".to_string(),
            self.config.service_version.clone(),
        );
        drop(tracer);

        // Store as current span for child spans
        *self.current_span.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!(
                "Failed to lock current_span mutex: {:?}",
                e
            ))
        })? = Some(span_ctx);

        Ok(())
    }

    /// End current span
    pub fn end_span(&self) -> WorkflowResult<()> {
        if !self.config.enabled {
            return Ok(()); // Legitimate: tracing disabled, no work to do
        }

        // Get current span and clear it
        let span_ctx = {
            let mut current = self.current_span.lock().map_err(|e| {
                crate::error::WorkflowError::Internal(format!(
                    "Failed to lock current_span mutex: {:?}",
                    e
                ))
            })?;
            current.take()
        };

        if let Some(span) = span_ctx {
            // End span with success status
            let mut tracer = self.tracer.lock().map_err(|e| {
                crate::error::WorkflowError::Internal(format!(
                    "Failed to lock tracer mutex: {:?}",
                    e
                ))
            })?;
            tracer.end_span(span, SpanStatus::Ok);
        }

        Ok(())
    }

    /// End current span with error status
    pub fn end_span_with_error(&self, error: &str) -> WorkflowResult<()> {
        if !self.config.enabled {
            return Ok(()); // Legitimate: tracing disabled, no work to do
        }

        // Get current span and clear it
        let span_ctx = {
            let mut current = self.current_span.lock().map_err(|e| {
                crate::error::WorkflowError::Internal(format!(
                    "Failed to lock current_span mutex: {:?}",
                    e
                ))
            })?;
            current.take()
        };

        if let Some(span) = span_ctx {
            let mut tracer = self.tracer.lock().map_err(|e| {
                crate::error::WorkflowError::Internal(format!(
                    "Failed to lock tracer mutex: {:?}",
                    e
                ))
            })?;
            // Add error attribute
            tracer.add_attribute(span.clone(), "error".to_string(), error.to_string());
            // End span with error status
            tracer.end_span(span, SpanStatus::Error);
        }

        Ok(())
    }

    /// Get tracer instance (for export)
    pub fn tracer(&self) -> &Arc<Mutex<Tracer>> {
        &self.tracer
    }
}
