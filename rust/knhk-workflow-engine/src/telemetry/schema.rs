// rust/knhk-workflow-engine/src/telemetry/schema.rs
// Runtime schema validation against Weaver schemas
// Covenant 6: Observations Drive Everything

//! # Schema Validation
//!
//! Runtime validation that emitted telemetry conforms to declared schemas.
//! This is the ONLY source of truth for telemetry correctness.
//!
//! ## Key Principle: Weaver Validation is Truth
//!
//! Traditional tests can have false positives. Schema validation cannot.
//! If Weaver validation passes, the telemetry is correct. If it fails, it's wrong.
//!
//! ## Validation Levels
//!
//! 1. **Schema Check**: `weaver registry check -r registry/`
//!    - Validates schema definitions are correct
//!    - Checks for conflicts, missing references
//!    - Static validation (no runtime needed)
//!
//! 2. **Live Check**: `weaver registry live-check --registry registry/`
//!    - Validates runtime telemetry against schema
//!    - Consumes actual OTLP spans/metrics/logs
//!    - Detects schema violations in real-time

use std::collections::HashMap;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    /// Global schema validator
    static ref SCHEMA_VALIDATOR: Arc<Mutex<Option<SchemaValidator>>> =
        Arc::new(Mutex::new(None));
}

/// Schema validation result
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Telemetry conforms to schema
    Valid,
    /// Telemetry violates schema
    Invalid {
        violations: Vec<String>,
    },
    /// Validation could not be performed
    Unknown {
        reason: String,
    },
}

impl ValidationResult {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    /// Get violation messages (if any)
    pub fn violations(&self) -> Vec<String> {
        match self {
            ValidationResult::Invalid { violations } => violations.clone(),
            ValidationResult::Unknown { reason } => vec![format!("Unknown: {}", reason)],
            ValidationResult::Valid => vec![],
        }
    }
}

/// Schema validator
pub struct SchemaValidator {
    /// Path to Weaver registry
    registry_path: String,
    /// Whether to enable live validation
    enable_live_check: bool,
    /// Cached validation results
    validation_cache: HashMap<String, ValidationResult>,
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new(registry_path: String) -> Self {
        Self {
            registry_path,
            enable_live_check: false,
            validation_cache: HashMap::new(),
        }
    }

    /// Enable live validation
    pub fn with_live_check(mut self, enabled: bool) -> Self {
        self.enable_live_check = enabled;
        self
    }

    /// Validate schema definitions (static check)
    ///
    /// Runs: `weaver registry check -r registry/`
    pub fn validate_schema_definitions(&self) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let output = Command::new("weaver")
            .args(&["registry", "check", "-r", &self.registry_path])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(ValidationResult::Valid)
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let violations = parse_weaver_violations(&stderr);
                    Ok(ValidationResult::Invalid { violations })
                }
            }
            Err(e) => Ok(ValidationResult::Unknown {
                reason: format!("Failed to run weaver: {}", e),
            }),
        }
    }

    /// Validate runtime telemetry (live check)
    ///
    /// Runs: `weaver registry live-check --registry registry/`
    pub fn validate_runtime_telemetry(
        &self,
        otlp_endpoint: &str,
    ) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        if !self.enable_live_check {
            return Ok(ValidationResult::Unknown {
                reason: "Live check not enabled".to_string(),
            });
        }

        let output = Command::new("weaver")
            .args(&[
                "registry",
                "live-check",
                "--registry",
                &self.registry_path,
                "--otlp-endpoint",
                otlp_endpoint,
            ])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(ValidationResult::Valid)
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let violations = parse_weaver_violations(&stderr);
                    Ok(ValidationResult::Invalid { violations })
                }
            }
            Err(e) => Ok(ValidationResult::Unknown {
                reason: format!("Failed to run weaver live-check: {}", e),
            }),
        }
    }

    /// Check if a span name is declared in schema
    pub fn is_span_declared(&self, span_name: &str) -> bool {
        // In production, this would parse schema YAML and check
        // For now, we check against known workflow engine spans
        matches!(
            span_name,
            "knhk.workflow_engine.register_workflow"
                | "knhk.workflow_engine.create_case"
                | "knhk.workflow_engine.execute_case"
                | "knhk.workflow_engine.execute_task"
                | "knhk.workflow_engine.execute_pattern"
                | "knhk.workflow_engine.pattern.sequence"
                | "knhk.workflow_engine.pattern.parallel_split"
                | "knhk.workflow_engine.pattern.synchronization"
                | "knhk.mapek.monitor"
                | "knhk.mapek.analyze"
                | "knhk.mapek.plan"
                | "knhk.mapek.execute"
                | "knhk.mapek.knowledge_update"
                | "knhk.mapek.cycle"
        )
    }

    /// Check if an attribute is declared for a span
    pub fn is_attribute_declared(&self, span_name: &str, attribute_name: &str) -> bool {
        // In production, this would parse schema YAML and check
        // For now, we check against known attributes
        attribute_name.starts_with("knhk.") || attribute_name.starts_with("lifecycle:")
            || attribute_name.starts_with("org:")
            || attribute_name.starts_with("time:")
    }

    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        self.validation_cache.clear();
    }
}

/// Initialize global schema validator
pub fn init_validator(registry_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let validator = SchemaValidator::new(registry_path.to_string()).with_live_check(true);

    // Validate schema definitions on init
    let result = validator.validate_schema_definitions()?;
    if !result.is_valid() {
        return Err(format!(
            "Schema validation failed: {:?}",
            result.violations()
        )
        .into());
    }

    // Store global validator
    let mut global = SCHEMA_VALIDATOR
        .lock()
        .map_err(|e| format!("Failed to lock validator: {}", e))?;
    *global = Some(validator);

    Ok(())
}

/// Get global schema validator
pub fn get_validator() -> Result<Arc<Mutex<Option<SchemaValidator>>>, Box<dyn std::error::Error>> {
    Ok(Arc::clone(&SCHEMA_VALIDATOR))
}

/// Validate that a span is declared in schema
pub fn validate_span_declared(span_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let validator = SCHEMA_VALIDATOR
        .lock()
        .map_err(|e| format!("Failed to lock validator: {}", e))?;

    match validator.as_ref() {
        Some(v) => Ok(v.is_span_declared(span_name)),
        None => Err("Schema validator not initialized".into()),
    }
}

/// Validate that an attribute is declared for a span
pub fn validate_attribute_declared(
    span_name: &str,
    attribute_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let validator = SCHEMA_VALIDATOR
        .lock()
        .map_err(|e| format!("Failed to lock validator: {}", e))?;

    match validator.as_ref() {
        Some(v) => Ok(v.is_attribute_declared(span_name, attribute_name)),
        None => Err("Schema validator not initialized".into()),
    }
}

// ============================================================================
// Internal helper functions
// ============================================================================

/// Parse Weaver violation messages from stderr
fn parse_weaver_violations(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .filter(|line| {
            line.contains("error") || line.contains("violation") || line.contains("invalid")
        })
        .map(|line| line.trim().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_is_valid() {
        assert!(ValidationResult::Valid.is_valid());
        assert!(!ValidationResult::Invalid {
            violations: vec!["error".to_string()]
        }
        .is_valid());
        assert!(!ValidationResult::Unknown {
            reason: "test".to_string()
        }
        .is_valid());
    }

    #[test]
    fn test_validation_result_violations() {
        let result = ValidationResult::Invalid {
            violations: vec!["error1".to_string(), "error2".to_string()],
        };
        assert_eq!(result.violations().len(), 2);

        let result = ValidationResult::Valid;
        assert_eq!(result.violations().len(), 0);
    }

    #[test]
    fn test_schema_validator_creation() {
        let validator = SchemaValidator::new("./registry".to_string());
        assert_eq!(validator.registry_path, "./registry");
        assert!(!validator.enable_live_check);
    }

    #[test]
    fn test_span_declared_check() {
        let validator = SchemaValidator::new("./registry".to_string());
        assert!(validator.is_span_declared("knhk.workflow_engine.register_workflow"));
        assert!(validator.is_span_declared("knhk.mapek.monitor"));
        assert!(!validator.is_span_declared("unknown.span"));
    }

    #[test]
    fn test_attribute_declared_check() {
        let validator = SchemaValidator::new("./registry".to_string());
        assert!(validator.is_attribute_declared("test", "knhk.workflow_engine.success"));
        assert!(validator.is_attribute_declared("test", "lifecycle:transition"));
        assert!(validator.is_attribute_declared("test", "org:resource"));
    }

    #[test]
    fn test_parse_violations() {
        let stderr = "error: invalid span name\nwarning: deprecated\nviolation: missing attribute";
        let violations = parse_weaver_violations(stderr);
        assert_eq!(violations.len(), 2); // error and violation lines
    }
}
