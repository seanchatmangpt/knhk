// rust/knhk-etl/src/diagnostics.rs
// ETL Pipeline Diagnostics Integration
// Note: knhk-validation integration disabled to avoid circular dependency

#![cfg(feature = "std")]

use alloc::string::String;
use alloc::collections::BTreeMap;

#[cfg(feature = "knhk-otel")]
use knhk_otel::generate_span_id;

// Note: knhk-validation diagnostics disabled to avoid circular dependency
// #[cfg(feature = "knhk-validation")]
// use knhk_validation::diagnostics::{DiagnosticMessage, Diagnostics, Severity};

/// ETL-specific diagnostic helpers
/// Note: Simplified version without knhk-validation dependency
pub struct EtlDiagnostics;

impl EtlDiagnostics {
    /// Create a pipeline stage error diagnostic (simplified)
    pub fn pipeline_stage_error(
        stage: &str,
        error: &str,
        span_id: Option<String>,
    ) -> String {
        let mut diag = format!("Pipeline stage '{}' failed: {}", stage, error);
        if let Some(span_id) = span_id {
            diag.push_str(&format!(" [span_id: {}]", span_id));
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag.push_str(&format!(" [span_id: {}]", generate_span_id()));
            }
        }
        diag
    }
    
    /// Create a connector error diagnostic (simplified)
    pub fn connector_error(
        connector_id: &str,
        error: &str,
        span_id: Option<String>,
    ) -> String {
        let mut diag = format!("Connector '{}' error: {}", connector_id, error);
        if let Some(span_id) = span_id {
            diag.push_str(&format!(" [span_id: {}]", span_id));
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag.push_str(&format!(" [span_id: {}]", generate_span_id()));
            }
        }
        diag
    }
    
    /// Create an ingest error diagnostic (simplified)
    pub fn ingest_error(
        source: &str,
        error: &str,
        span_id: Option<String>,
    ) -> String {
        let mut diag = format!("Ingest from '{}' failed: {}", source, error);
        if let Some(span_id) = span_id {
            diag.push_str(&format!(" [span_id: {}]", span_id));
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag.push_str(&format!(" [span_id: {}]", generate_span_id()));
            }
        }
        diag
    }
    
    /// Create a transform error diagnostic (simplified)
    pub fn transform_error(
        operation: &str,
        error: &str,
        span_id: Option<String>,
    ) -> String {
        let mut diag = format!("Transform operation '{}' failed: {}", operation, error);
        if let Some(span_id) = span_id {
            diag.push_str(&format!(" [span_id: {}]", span_id));
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag.push_str(&format!(" [span_id: {}]", generate_span_id()));
            }
        }
        diag
    }
    
    /// Create a load error diagnostic (simplified)
    pub fn load_error(
        error: &str,
        span_id: Option<String>,
    ) -> String {
        let mut diag = format!("Load operation failed: {}", error);
        if let Some(span_id) = span_id {
            diag.push_str(&format!(" [span_id: {}]", span_id));
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag.push_str(&format!(" [span_id: {}]", generate_span_id()));
            }
        }
        diag
    }
    
    /// Create a reflex error diagnostic (simplified)
    pub fn reflex_error(
        operation: &str,
        error: &str,
        span_id: Option<String>,
    ) -> String {
        let mut diag = format!("Reflex operation '{}' failed: {}", operation, error);
        if let Some(span_id) = span_id {
            diag.push_str(&format!(" [span_id: {}]", span_id));
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag.push_str(&format!(" [span_id: {}]", generate_span_id()));
            }
        }
        diag
    }
    
    /// Create an emit error diagnostic (simplified)
    pub fn emit_error(
        endpoint: &str,
        error: &str,
        span_id: Option<String>,
    ) -> String {
        let mut diag = format!("Emit to '{}' failed: {}", endpoint, error);
        if let Some(span_id) = span_id {
            diag.push_str(&format!(" [span_id: {}]", span_id));
        } else {
            #[cfg(feature = "knhk-otel")]
            {
                diag.push_str(&format!(" [span_id: {}]", generate_span_id()));
            }
        }
        diag
    }
}

