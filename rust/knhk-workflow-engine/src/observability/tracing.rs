//! Distributed tracing utilities
//!
//! Provides structured tracing for workflow operations with OTEL integration.

use crate::error::WorkflowError;
use knhk_otel::Tracer;
use opentelemetry::trace::{Span, TraceContextExt, Tracer as _};
use opentelemetry::Context;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

/// Tracing manager for workflow operations
pub struct TracingManager {
    tracer: Option<Arc<Tracer>>,
    service_name: String,
    service_version: String,
}

impl TracingManager {
    /// Create new tracing manager
    pub fn new(service_name: String, service_version: String) -> Result<Self, WorkflowError> {
        let tracer = Tracer::new(&service_name, &service_version)
            .map_err(|e| WorkflowError::Internal(format!("Failed to create tracer: {}", e)))?;

        Ok(Self {
            tracer: Some(Arc::new(tracer)),
            service_name,
            service_version,
        })
    }

    /// Start span for workflow operation
    #[instrument(skip(self), fields(workflow_id = %workflow_id, case_id = %case_id))]
    pub fn start_workflow_span(
        &self,
        operation: &str,
        workflow_id: &str,
        case_id: &str,
    ) -> Option<Span> {
        self.tracer.as_ref().map(|tracer| {
            let span = tracer
                .span_builder(format!("workflow.{}", operation))
                .with_attributes(vec![
                    opentelemetry::KeyValue::new("workflow.id", workflow_id.to_string()),
                    opentelemetry::KeyValue::new("case.id", case_id.to_string()),
                    opentelemetry::KeyValue::new("service.name", self.service_name.clone()),
                    opentelemetry::KeyValue::new("service.version", self.service_version.clone()),
                ])
                .start(tracer);
            span
        })
    }

    /// Record workflow event
    pub fn record_event(&self, level: &str, message: &str, context: &[(&str, &str)]) {
        match level {
            "error" => error!(?context, "{}", message),
            "warn" => warn!(?context, "{}", message),
            "info" => info!(?context, "{}", message),
            "debug" => debug!(?context, "{}", message),
            _ => info!(?context, "{}", message),
        }
    }
}
