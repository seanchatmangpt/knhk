// rust/knhk-workflow-engine/src/telemetry/mod.rs
// OpenTelemetry integration for workflow engine
// Covenant 6: Observations Drive Everything

//! # Telemetry Module
//!
//! This module provides OpenTelemetry integration for the KNHK workflow engine,
//! implementing Covenant 6: Observations Drive Everything.
//!
//! ## Architecture
//!
//! - `emit`: Functions for emitting workflow execution telemetry
//! - `schema`: Runtime schema validation against Weaver schemas
//! - `mape_k`: MAPE-K autonomic feedback loop integration
//!
//! ## Key Principles
//!
//! 1. **All behavior is observable**: Every workflow action emits telemetry
//! 2. **Schema-first**: Runtime telemetry must conform to declared schemas
//! 3. **MAPE-K integration**: Telemetry feeds autonomic feedback loops
//! 4. **Weaver validation**: Only trust schema-validated observations

pub mod emit;
pub mod mape_k;
pub mod schema;

// Re-export commonly used types
pub use emit::{emit_case_created, emit_pattern_executed, emit_task_executed, emit_workflow_registered};
pub use mape_k::{MapekCycle, MapekComponent};
pub use schema::{SchemaValidator, ValidationResult};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Telemetry context for a workflow execution
#[derive(Debug, Clone)]
pub struct TelemetryContext {
    /// Trace ID for distributed tracing
    pub trace_id: String,
    /// Span ID for current operation
    pub span_id: String,
    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,
    /// Custom attributes for this context
    pub attributes: HashMap<String, String>,
}

impl TelemetryContext {
    /// Create a new telemetry context
    pub fn new(trace_id: String, span_id: String) -> Self {
        Self {
            trace_id,
            span_id,
            parent_span_id: None,
            attributes: HashMap::new(),
        }
    }

    /// Create a child context
    pub fn with_parent(&self, new_span_id: String) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: new_span_id,
            parent_span_id: Some(self.span_id.clone()),
            attributes: self.attributes.clone(),
        }
    }

    /// Add an attribute to this context
    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }
}

/// Global telemetry configuration
pub struct TelemetryConfig {
    /// OTLP endpoint for exporting telemetry
    pub otlp_endpoint: String,
    /// Whether to enable schema validation
    pub enable_validation: bool,
    /// Path to Weaver registry
    pub registry_path: String,
    /// Whether to enable MAPE-K feedback
    pub enable_mapek: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: "http://localhost:4317".to_string(),
            enable_validation: true,
            registry_path: "./registry".to_string(),
            enable_mapek: true,
        }
    }
}

lazy_static::lazy_static! {
    /// Global telemetry configuration
    pub static ref TELEMETRY_CONFIG: Arc<Mutex<TelemetryConfig>> =
        Arc::new(Mutex::new(TelemetryConfig::default()));
}

/// Initialize telemetry subsystem
pub fn init_telemetry(config: TelemetryConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Update global config
    let mut global_config = TELEMETRY_CONFIG
        .lock()
        .map_err(|e| format!("Failed to lock telemetry config: {}", e))?;
    *global_config = config;

    // Initialize schema validator if enabled
    if global_config.enable_validation {
        schema::init_validator(&global_config.registry_path)?;
    }

    // Initialize MAPE-K if enabled
    if global_config.enable_mapek {
        mape_k::init_mapek()?;
    }

    Ok(())
}

/// Shutdown telemetry subsystem
pub fn shutdown_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    // Flush any pending telemetry
    emit::flush_telemetry()?;

    // Shutdown MAPE-K
    mape_k::shutdown_mapek()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_context_creation() {
        let ctx = TelemetryContext::new("trace-123".to_string(), "span-456".to_string());
        assert_eq!(ctx.trace_id, "trace-123");
        assert_eq!(ctx.span_id, "span-456");
        assert!(ctx.parent_span_id.is_none());
    }

    #[test]
    fn test_telemetry_context_child() {
        let parent = TelemetryContext::new("trace-123".to_string(), "span-456".to_string());
        let child = parent.with_parent("span-789".to_string());

        assert_eq!(child.trace_id, "trace-123");
        assert_eq!(child.span_id, "span-789");
        assert_eq!(child.parent_span_id, Some("span-456".to_string()));
    }

    #[test]
    fn test_default_config() {
        let config = TelemetryConfig::default();
        assert_eq!(config.otlp_endpoint, "http://localhost:4317");
        assert!(config.enable_validation);
        assert_eq!(config.registry_path, "./registry");
        assert!(config.enable_mapek);
    }
}
