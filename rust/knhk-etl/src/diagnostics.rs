// rust/knhk-etl/src/diagnostics.rs
// ETL Pipeline Diagnostics Integration
// Integrates with knhk-validation diagnostics and adds OTEL span context

#![cfg(feature = "std")]

use alloc::string::String;
use alloc::collections::BTreeMap;

#[cfg(feature = "knhk-otel")]
use knhk_otel::generate_span_id;

#[cfg(feature = "knhk-validation")]
use knhk_validation::diagnostics::{DiagnosticMessage, Diagnostics, Severity};

/// ETL-specific diagnostic helpers
pub struct EtlDiagnostics;

impl EtlDiagnostics {
    /// Create a pipeline stage error diagnostic
    pub fn pipeline_stage_error(
        stage: &str,
        error: &str,
        span_id: Option<String>,
    ) -> DiagnosticMessage {
        let mut diag = DiagnosticMessage::new(
            "PIPELINE_STAGE_ERROR",
            format!("Pipeline stage '{}' failed: {}", stage, error),
        )
        .with_severity(Severity::Error)
        .with_context("stage", stage.to_string())
        .with_context("error", error.to_string());
        
        if let Some(span_id) = span_id {
            diag = diag.with_span_id(span_id);
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag = diag.with_span_id(generate_span_id().to_string());
            }
        }
        
        diag
    }
    
    /// Create a connector error diagnostic
    pub fn connector_error(
        connector_id: &str,
        error: &str,
        span_id: Option<String>,
    ) -> DiagnosticMessage {
        let mut diag = DiagnosticMessage::new(
            "CONNECTOR_ERROR",
            format!("Connector '{}' error: {}", connector_id, error),
        )
        .with_severity(Severity::Error)
        .with_context("connector_id", connector_id.to_string())
        .with_context("error", error.to_string());
        
        if let Some(span_id) = span_id {
            diag = diag.with_span_id(span_id);
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag = diag.with_span_id(generate_span_id().to_string());
            }
        }
        
        diag
    }
    
    /// Create an ingest error diagnostic
    pub fn ingest_error(
        source: &str,
        error: &str,
        span_id: Option<String>,
    ) -> DiagnosticMessage {
        let mut diag = DiagnosticMessage::new(
            "INGEST_ERROR",
            format!("Ingest from '{}' failed: {}", source, error),
        )
        .with_severity(Severity::Error)
        .with_context("source", source.to_string())
        .with_context("error", error.to_string());
        
        if let Some(span_id) = span_id {
            diag = diag.with_span_id(span_id);
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag = diag.with_span_id(generate_span_id().to_string());
            }
        }
        
        diag
    }
    
    /// Create a transform error diagnostic
    pub fn transform_error(
        operation: &str,
        error: &str,
        span_id: Option<String>,
    ) -> DiagnosticMessage {
        let mut diag = DiagnosticMessage::new(
            "TRANSFORM_ERROR",
            format!("Transform operation '{}' failed: {}", operation, error),
        )
        .with_severity(Severity::Error)
        .with_context("operation", operation.to_string())
        .with_context("error", error.to_string());
        
        if let Some(span_id) = span_id {
            diag = diag.with_span_id(span_id);
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag = diag.with_span_id(generate_span_id().to_string());
            }
        }
        
        diag
    }
    
    /// Create a load error diagnostic
    pub fn load_error(
        error: &str,
        span_id: Option<String>,
    ) -> DiagnosticMessage {
        let mut diag = DiagnosticMessage::new(
            "LOAD_ERROR",
            format!("Load operation failed: {}", error),
        )
        .with_severity(Severity::Error)
        .with_context("error", error.to_string());
        
        if let Some(span_id) = span_id {
            diag = diag.with_span_id(span_id);
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag = diag.with_span_id(generate_span_id().to_string());
            }
        }
        
        diag
    }
    
    /// Create a reflex error diagnostic
    pub fn reflex_error(
        operation: &str,
        error: &str,
        span_id: Option<String>,
    ) -> DiagnosticMessage {
        let mut diag = DiagnosticMessage::new(
            "REFLEX_ERROR",
            format!("Reflex operation '{}' failed: {}", operation, error),
        )
        .with_severity(Severity::Error)
        .with_context("operation", operation.to_string())
        .with_context("error", error.to_string());
        
        if let Some(span_id) = span_id {
            diag = diag.with_span_id(span_id);
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag = diag.with_span_id(generate_span_id().to_string());
            }
        }
        
        diag
    }
    
    /// Create an emit error diagnostic
    pub fn emit_error(
        endpoint: &str,
        error: &str,
        span_id: Option<String>,
    ) -> DiagnosticMessage {
        let mut diag = DiagnosticMessage::new(
            "EMIT_ERROR",
            format!("Emit to '{}' failed: {}", endpoint, error),
        )
        .with_severity(Severity::Error)
        .with_context("endpoint", endpoint.to_string())
        .with_context("error", error.to_string());
        
        if let Some(span_id) = span_id {
            diag = diag.with_span_id(span_id);
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag = diag.with_span_id(generate_span_id().to_string());
            }
        }
        
        diag
    }
}

