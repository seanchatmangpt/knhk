//! SHACL Validation Integration
//!
//! Integrates SHACL (Shapes Constraint Language) for RDF graph validation.
//! Validates workflow graphs against SHACL shapes.

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SHACL validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclValidationResult {
    /// Whether validation passed
    pub conforms: bool,
    /// Validation violations
    pub violations: Vec<ShaclViolation>,
    /// Validation timestamp
    pub validated_at_ms: u64,
}

/// SHACL violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclViolation {
    /// Focus node
    pub focus_node: String,
    /// Result path
    pub result_path: Option<String>,
    /// Severity
    pub severity: ShaclSeverity,
    /// Message
    pub message: String,
    /// Source constraint component
    pub source_constraint: Option<String>,
}

/// SHACL severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaclSeverity {
    /// Info
    Info,
    /// Warning
    Warning,
    /// Violation
    Violation,
}

/// SHACL shape definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclShape {
    /// Shape ID
    pub id: String,
    /// Target class
    pub target_class: Option<String>,
    /// Property constraints
    pub properties: HashMap<String, PropertyConstraint>,
}

/// Property constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyConstraint {
    /// Property path
    pub path: String,
    /// Min count
    pub min_count: Option<u32>,
    /// Max count
    pub max_count: Option<u32>,
    /// Datatype
    pub datatype: Option<String>,
    /// Node kind
    pub node_kind: Option<NodeKind>,
}

/// Node kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    /// IRI node
    Iri,
    /// Blank node
    BlankNode,
    /// Literal
    Literal,
    /// IRI or literal
    IriOrLiteral,
    /// Blank node or IRI
    BlankNodeOrIri,
    /// Blank node or literal
    BlankNodeOrLiteral,
}

/// SHACL validator
pub struct ShaclValidator {
    /// Registered shapes
    shapes: HashMap<String, ShaclShape>,
}

impl ShaclValidator {
    /// Create new SHACL validator
    pub fn new() -> Self {
        Self {
            shapes: HashMap::new(),
        }
    }

    /// Register a SHACL shape
    pub fn register_shape(&mut self, shape: ShaclShape) -> WorkflowResult<()> {
        if shape.id.is_empty() {
            return Err(WorkflowError::Validation(
                "Shape ID cannot be empty".to_string(),
            ));
        }

        self.shapes.insert(shape.id.clone(), shape);
        Ok(())
    }

    /// Validate RDF graph against shapes
    pub fn validate(
        &self,
        graph_data: &serde_json::Value,
    ) -> WorkflowResult<ShaclValidationResult> {
        let mut violations = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Simplified validation - in production would use oxigraph SHACL
        for (_shape_id, shape) in &self.shapes {
            if let Some(target_class) = &shape.target_class {
                if let Some(nodes) = graph_data.get("@graph").and_then(|g| g.as_array()) {
                    for node in nodes {
                        if let Some(node_type) = node.get("@type").and_then(|t| t.as_str()) {
                            if node_type == target_class {
                                // Validate properties
                                for (prop_name, constraint) in &shape.properties {
                                    if let Some(min_count) = constraint.min_count {
                                        if min_count > 0 && node.get(prop_name).is_none() {
                                            violations.push(ShaclViolation {
                                                focus_node: node
                                                    .get("@id")
                                                    .and_then(|id| id.as_str())
                                                    .unwrap_or("unknown")
                                                    .to_string(),
                                                result_path: Some(prop_name.clone()),
                                                severity: ShaclSeverity::Violation,
                                                message: format!(
                                                    "Property '{}' is required (min count: {})",
                                                    prop_name, min_count
                                                ),
                                                source_constraint: Some("sh:minCount".to_string()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(ShaclValidationResult {
            conforms: violations.is_empty(),
            violations,
            validated_at_ms: timestamp,
        })
    }

    /// List registered shapes
    pub fn list_shapes(&self) -> Vec<&ShaclShape> {
        self.shapes.values().collect()
    }

    /// Get shape by ID
    pub fn get_shape(&self, id: &str) -> Option<&ShaclShape> {
        self.shapes.get(id)
    }
}

impl Default for ShaclValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shacl_validator() {
        let mut validator = ShaclValidator::new();

        let shape = ShaclShape {
            id: "WorkflowShape".to_string(),
            target_class: Some("Workflow".to_string()),
            properties: {
                let mut props = HashMap::new();
                props.insert(
                    "name".to_string(),
                    PropertyConstraint {
                        path: "name".to_string(),
                        min_count: Some(1),
                        max_count: None,
                        datatype: Some("xsd:string".to_string()),
                        node_kind: None,
                    },
                );
                props
            },
        };

        validator
            .register_shape(shape)
            .expect("Failed to register shape");

        // Valid graph
        let valid_graph = serde_json::json!({
            "@graph": [{
                "@id": "workflow1",
                "@type": "Workflow",
                "name": "My Workflow"
            }]
        });

        let result = validator.validate(&valid_graph).expect("Validation failed");
        assert!(result.conforms);

        // Invalid graph (missing name)
        let invalid_graph = serde_json::json!({
            "@graph": [{
                "@id": "workflow2",
                "@type": "Workflow"
            }]
        });

        let result = validator
            .validate(&invalid_graph)
            .expect("Validation failed");
        assert!(!result.conforms);
        assert_eq!(result.violations.len(), 1);
    }

    #[test]
    fn test_shape_registration() {
        let mut validator = ShaclValidator::new();

        let shape = ShaclShape {
            id: "TestShape".to_string(),
            target_class: None,
            properties: HashMap::new(),
        };

        validator
            .register_shape(shape)
            .expect("Registration failed");

        let shapes = validator.list_shapes();
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].id, "TestShape");
    }
}
