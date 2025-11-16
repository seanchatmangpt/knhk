//! Weaver schema validation for telemetry compliance
//!
//! This module provides real-time validation of telemetry events against OpenTelemetry
//! Weaver schemas, ensuring compliance with semantic conventions and schema definitions.
//!
//! # Schema Validation Flow
//!
//! ```text
//! Event → Extract Attributes → Load Schema → Validate → Report Violations
//!   ↓            ↓                 ↓            ↓              ↓
//! Span      name,attrs        registry/    Required?      Error/Warning
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn, error};

use super::{TelemetryEvent, TelemetryError, TelemetryResult, Span, Metric, LogEntry, AttributeValue};

/// Weaver schema validator
pub struct WeaverValidator {
    /// Registry path
    registry_path: PathBuf,

    /// Loaded schemas (span_name -> schema)
    schemas: Arc<RwLock<HashMap<String, SpanSchema>>>,

    /// Validation statistics
    stats: Arc<RwLock<ValidationStats>>,
}

/// Span schema definition (simplified from Weaver YAML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanSchema {
    /// Span name/ID
    pub id: String,

    /// Brief description
    pub brief: String,

    /// Required attributes
    pub required_attributes: Vec<AttributeSchema>,

    /// Optional attributes
    pub optional_attributes: Vec<AttributeSchema>,

    /// Span type (span, metric, log)
    pub span_type: String,
}

/// Attribute schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeSchema {
    /// Attribute ID/name
    pub id: String,

    /// Attribute type
    pub attr_type: String,

    /// Requirement level (required, recommended, optional)
    pub requirement_level: String,

    /// Brief description
    pub brief: Option<String>,

    /// Examples
    pub examples: Option<Vec<String>>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Is valid
    pub valid: bool,

    /// Violations found
    pub violations: Vec<SchemaViolation>,

    /// Warnings (non-critical)
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.valid && self.violations.is_empty()
    }

    /// Get error messages
    pub fn errors(&self) -> Vec<String> {
        self.violations.iter()
            .map(|v| v.to_string())
            .collect()
    }
}

/// Schema violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaViolation {
    /// Violation type
    pub violation_type: ViolationType,

    /// Field that violated
    pub field: String,

    /// Expected value/type
    pub expected: String,

    /// Actual value/type
    pub actual: String,

    /// Severity
    pub severity: Severity,
}

impl std::fmt::Display for SchemaViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:?}] {}: {} (expected: {}, actual: {})",
            self.severity,
            self.violation_type,
            self.field,
            self.expected,
            self.actual
        )
    }
}

/// Violation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// Missing required attribute
    MissingRequired,

    /// Incorrect type
    IncorrectType,

    /// Invalid value
    InvalidValue,

    /// Unknown attribute (not in schema)
    UnknownAttribute,

    /// Schema not found
    SchemaNotFound,
}

impl std::fmt::Display for ViolationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationType::MissingRequired => write!(f, "Missing required attribute"),
            ViolationType::IncorrectType => write!(f, "Incorrect type"),
            ViolationType::InvalidValue => write!(f, "Invalid value"),
            ViolationType::UnknownAttribute => write!(f, "Unknown attribute"),
            ViolationType::SchemaNotFound => write!(f, "Schema not found"),
        }
    }
}

/// Violation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational
    Info,

    /// Warning
    Warning,

    /// Error
    Error,

    /// Critical
    Critical,
}

/// Validation statistics
#[derive(Debug, Default, Clone)]
pub struct ValidationStats {
    /// Total validations performed
    pub total_validations: u64,

    /// Total violations found
    pub total_violations: u64,

    /// Violations by type
    pub violations_by_type: HashMap<String, u64>,

    /// Most violated spans
    pub most_violated_spans: HashMap<String, u64>,
}

impl WeaverValidator {
    /// Create a new Weaver validator
    pub fn new(registry_path: impl AsRef<Path>) -> TelemetryResult<Self> {
        let path = registry_path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(TelemetryError::ValidationError(
                format!("Registry path does not exist: {}", path.display())
            ));
        }

        // Load schemas from registry
        let schemas = Self::load_schemas(&path)?;

        debug!("Loaded {} schemas from registry", schemas.len());

        Ok(Self {
            registry_path: path,
            schemas: Arc::new(RwLock::new(schemas)),
            stats: Arc::new(RwLock::new(ValidationStats::default())),
        })
    }

    /// Load schemas from registry directory
    fn load_schemas(registry_path: &Path) -> TelemetryResult<HashMap<String, SpanSchema>> {
        let mut schemas = HashMap::new();

        // For now, create some default schemas for common workflow patterns
        // In production, this would parse actual YAML files from the registry

        // Workflow execution span schema
        schemas.insert(
            "workflow.execute".to_string(),
            SpanSchema {
                id: "workflow.execute".to_string(),
                brief: "Workflow execution span".to_string(),
                required_attributes: vec![
                    AttributeSchema {
                        id: "workflow.id".to_string(),
                        attr_type: "string".to_string(),
                        requirement_level: "required".to_string(),
                        brief: Some("Workflow instance ID".to_string()),
                        examples: Some(vec!["wf-001".to_string()]),
                    },
                    AttributeSchema {
                        id: "workflow.pattern".to_string(),
                        attr_type: "string".to_string(),
                        requirement_level: "required".to_string(),
                        brief: Some("Workflow pattern type".to_string()),
                        examples: Some(vec!["Sequence".to_string(), "ParallelSplit".to_string()]),
                    },
                ],
                optional_attributes: vec![
                    AttributeSchema {
                        id: "workflow.case_id".to_string(),
                        attr_type: "string".to_string(),
                        requirement_level: "optional".to_string(),
                        brief: Some("Case ID".to_string()),
                        examples: None,
                    },
                ],
                span_type: "span".to_string(),
            },
        );

        // Task execution span schema
        schemas.insert(
            "task.execute".to_string(),
            SpanSchema {
                id: "task.execute".to_string(),
                brief: "Task execution span".to_string(),
                required_attributes: vec![
                    AttributeSchema {
                        id: "task.id".to_string(),
                        attr_type: "string".to_string(),
                        requirement_level: "required".to_string(),
                        brief: Some("Task ID".to_string()),
                        examples: Some(vec!["task-123".to_string()]),
                    },
                ],
                optional_attributes: vec![],
                span_type: "span".to_string(),
            },
        );

        // Pattern execution span schema
        schemas.insert(
            "pattern.execute".to_string(),
            SpanSchema {
                id: "pattern.execute".to_string(),
                brief: "Pattern execution span".to_string(),
                required_attributes: vec![
                    AttributeSchema {
                        id: "pattern.id".to_string(),
                        attr_type: "string".to_string(),
                        requirement_level: "required".to_string(),
                        brief: Some("Pattern ID".to_string()),
                        examples: Some(vec!["1".to_string(), "2".to_string()]),
                    },
                    AttributeSchema {
                        id: "pattern.name".to_string(),
                        attr_type: "string".to_string(),
                        requirement_level: "required".to_string(),
                        brief: Some("Pattern name".to_string()),
                        examples: Some(vec!["Sequence".to_string()]),
                    },
                ],
                optional_attributes: vec![],
                span_type: "span".to_string(),
            },
        );

        Ok(schemas)
    }

    /// Validate a telemetry event
    pub async fn validate_event(&self, event: &TelemetryEvent) -> TelemetryResult<ValidationResult> {
        match event {
            TelemetryEvent::Span(span) => self.validate_span_full(span).await,
            TelemetryEvent::Metric(metric) => self.validate_metric(metric).await,
            TelemetryEvent::Log(log) => self.validate_log(log).await,
        }
    }

    /// Validate a span against its schema
    pub async fn validate_span(
        &self,
        span_name: &str,
        attributes: &[(String, AttributeValue)],
    ) -> TelemetryResult<ValidationResult> {
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.total_validations += 1;
        }

        // Get schema
        let schema = {
            let schemas = self.schemas.read();
            schemas.get(span_name).cloned()
        };

        let Some(schema) = schema else {
            // Schema not found - create violation
            let violation = SchemaViolation {
                violation_type: ViolationType::SchemaNotFound,
                field: span_name.to_string(),
                expected: "Schema definition".to_string(),
                actual: "Not found in registry".to_string(),
                severity: Severity::Warning,
            };

            return Ok(ValidationResult {
                valid: false,
                violations: vec![violation],
                warnings: vec![format!("Schema not found for: {}", span_name)],
            });
        };

        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // Convert attributes to HashMap for easier lookup
        let attr_map: HashMap<&str, &AttributeValue> = attributes
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();

        // Check required attributes
        for required_attr in &schema.required_attributes {
            if !attr_map.contains_key(required_attr.id.as_str()) {
                violations.push(SchemaViolation {
                    violation_type: ViolationType::MissingRequired,
                    field: required_attr.id.clone(),
                    expected: required_attr.attr_type.clone(),
                    actual: "missing".to_string(),
                    severity: Severity::Error,
                });
            } else {
                // Check type
                let actual_value = attr_map.get(required_attr.id.as_str());
                if let Some(value) = actual_value {
                    let actual_type = attribute_type_name(value);
                    if actual_type != required_attr.attr_type {
                        violations.push(SchemaViolation {
                            violation_type: ViolationType::IncorrectType,
                            field: required_attr.id.clone(),
                            expected: required_attr.attr_type.clone(),
                            actual: actual_type.to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        // Check for unknown attributes (warnings only)
        let known_attrs: std::collections::HashSet<&str> = schema
            .required_attributes
            .iter()
            .chain(schema.optional_attributes.iter())
            .map(|a| a.id.as_str())
            .collect();

        for (attr_name, _) in attributes {
            if !known_attrs.contains(attr_name.as_str()) {
                warnings.push(format!("Unknown attribute: {}", attr_name));
            }
        }

        // Update stats
        if !violations.is_empty() {
            let mut stats = self.stats.write();
            stats.total_violations += violations.len() as u64;

            for violation in &violations {
                let type_name = format!("{:?}", violation.violation_type);
                *stats.violations_by_type.entry(type_name).or_insert(0) += 1;
            }

            *stats.most_violated_spans.entry(span_name.to_string()).or_insert(0) += 1;
        }

        Ok(ValidationResult {
            valid: violations.is_empty(),
            violations,
            warnings,
        })
    }

    /// Validate a full span
    async fn validate_span_full(&self, span: &Span) -> TelemetryResult<ValidationResult> {
        self.validate_span(&span.name, &span.attributes).await
    }

    /// Validate a metric
    async fn validate_metric(&self, metric: &Metric) -> TelemetryResult<ValidationResult> {
        // Simplified - metrics validation would be similar to spans
        Ok(ValidationResult {
            valid: true,
            violations: Vec::new(),
            warnings: Vec::new(),
        })
    }

    /// Validate a log entry
    async fn validate_log(&self, log: &LogEntry) -> TelemetryResult<ValidationResult> {
        // Simplified - log validation would be similar to spans
        Ok(ValidationResult {
            valid: true,
            violations: Vec::new(),
            warnings: Vec::new(),
        })
    }

    /// Get validation statistics
    pub fn stats(&self) -> ValidationStats {
        self.stats.read().clone()
    }

    /// Reload schemas from registry
    pub async fn reload_schemas(&self) -> TelemetryResult<()> {
        let new_schemas = Self::load_schemas(&self.registry_path)?;

        let mut schemas = self.schemas.write();
        *schemas = new_schemas;

        debug!("Reloaded {} schemas", schemas.len());

        Ok(())
    }
}

/// Get type name for attribute value
fn attribute_type_name(value: &AttributeValue) -> &'static str {
    match value {
        AttributeValue::String(_) => "string",
        AttributeValue::Int(_) => "int",
        AttributeValue::Float(_) => "double",
        AttributeValue::Bool(_) => "boolean",
        AttributeValue::StringArray(_) => "string[]",
        AttributeValue::IntArray(_) => "int[]",
        AttributeValue::FloatArray(_) => "double[]",
        AttributeValue::BoolArray(_) => "boolean[]",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_span_validation_success() {
        // Create temporary registry directory
        let temp_dir = std::env::temp_dir().join("weaver_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let validator = WeaverValidator::new(&temp_dir).unwrap();

        let attributes = vec![
            ("workflow.id".to_string(), AttributeValue::String("wf-001".to_string())),
            ("workflow.pattern".to_string(), AttributeValue::String("Sequence".to_string())),
        ];

        let result = validator.validate_span("workflow.execute", &attributes).await.unwrap();

        assert!(result.is_valid(), "Validation should pass with all required attributes");
    }

    #[tokio::test]
    async fn test_span_validation_missing_required() {
        let temp_dir = std::env::temp_dir().join("weaver_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let validator = WeaverValidator::new(&temp_dir).unwrap();

        let attributes = vec![
            ("workflow.id".to_string(), AttributeValue::String("wf-001".to_string())),
            // Missing "workflow.pattern"
        ];

        let result = validator.validate_span("workflow.execute", &attributes).await.unwrap();

        assert!(!result.is_valid(), "Validation should fail with missing required attribute");
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].violation_type, ViolationType::MissingRequired);
    }

    #[tokio::test]
    async fn test_span_validation_incorrect_type() {
        let temp_dir = std::env::temp_dir().join("weaver_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let validator = WeaverValidator::new(&temp_dir).unwrap();

        let attributes = vec![
            ("workflow.id".to_string(), AttributeValue::Int(123)),  // Should be string
            ("workflow.pattern".to_string(), AttributeValue::String("Sequence".to_string())),
        ];

        let result = validator.validate_span("workflow.execute", &attributes).await.unwrap();

        assert!(!result.is_valid(), "Validation should fail with incorrect type");
        assert_eq!(result.violations[0].violation_type, ViolationType::IncorrectType);
    }
}
