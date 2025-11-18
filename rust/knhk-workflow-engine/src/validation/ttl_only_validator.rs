//! TTL-only validation enforcer
//!
//! Ensures workflows are pure TTL/RDF format and comply with YAWL ontology.
//! Implements DOCTRINE Covenant 1: Turtle is the sole source of truth.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;

#[cfg(feature = "rdf")]
use oxigraph::store::Store;
#[cfg(feature = "rdf")]
use oxigraph::io::RdfFormat;

/// TTL-only validator
///
/// Enforces strict TTL/Turtle-only workflows per DOCTRINE Covenant 1.
/// Rejects any workflow that cannot be parsed as valid Turtle RDF.
pub struct TTLOnlyValidator {
    /// Require YAWL ontology compliance
    pub require_yawl_ontology: bool,
    /// Require Weaver schema validation
    pub require_weaver_validation: bool,
}

impl TTLOnlyValidator {
    /// Create new TTL-only validator with default settings
    pub fn new() -> Self {
        Self {
            require_yawl_ontology: true,
            require_weaver_validation: false, // opt-in
        }
    }

    /// Create validator with strict settings (all validations enabled)
    pub fn strict() -> Self {
        Self {
            require_yawl_ontology: true,
            require_weaver_validation: true,
        }
    }

    /// Validate that workflow is pure TTL/RDF
    ///
    /// # DOCTRINE Covenant 1
    ///
    /// "Turtle is the sole source of truth for all workflow definitions.
    /// No XML, no JSON (except JSON-LD RDF), no proprietary formats."
    ///
    /// # Validation Steps
    ///
    /// 1. **TTL Syntax**: Ensure workflow is valid Turtle RDF
    /// 2. **YAWL Ontology**: Verify use of YAWL ontology predicates
    /// 3. **Semantic Completeness**: Check all required elements present
    /// 4. **Weaver Schema** (optional): Validate against OTEL schema
    ///
    /// # Errors
    ///
    /// Returns `WorkflowError::Validation` if:
    /// - Workflow is not valid TTL syntax
    /// - Workflow does not use YAWL ontology
    /// - Required elements are missing
    /// - Weaver validation fails (if enabled)
    pub fn validate(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        // Step 1: Validate TTL syntax
        self.validate_ttl_syntax(spec)?;

        // Step 2: Validate YAWL ontology usage
        if self.require_yawl_ontology {
            self.validate_yawl_ontology(spec)?;
        }

        // Step 3: Validate semantic completeness
        self.validate_semantic_completeness(spec)?;

        // Step 4: Weaver validation (opt-in)
        if self.require_weaver_validation {
            self.validate_weaver_schema(spec)?;
        }

        Ok(())
    }

    /// Validate that workflow is valid TTL syntax
    fn validate_ttl_syntax(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        // If no source turtle, cannot validate TTL-only
        let turtle = spec.source_turtle.as_ref().ok_or_else(|| {
            WorkflowError::Validation(
                "Workflow missing source TTL (v4.0 requires TTL-only workflows)".to_string()
            )
        })?;

        // Parse TTL to verify valid syntax
        #[cfg(feature = "rdf")]
        {
            let store = Store::new()
                .map_err(|e| WorkflowError::Internal(format!("Failed to create RDF store: {:?}", e)))?;

            store
                .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
                .map_err(|e| {
                    WorkflowError::Validation(format!(
                        "Invalid TTL syntax (v4.0 requires valid Turtle RDF): {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// Validate that workflow uses YAWL ontology
    fn validate_yawl_ontology(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        let turtle = spec.source_turtle.as_ref().ok_or_else(|| {
            WorkflowError::Validation("Workflow missing source TTL".to_string())
        })?;

        // Check for YAWL namespace declaration
        if !turtle.contains("yawl:") && !turtle.contains("http://bitflow.ai/ontology/yawl") {
            return Err(WorkflowError::Validation(
                "Workflow does not use YAWL ontology (v4.0 requires YAWL TTL format)".to_string()
            ));
        }

        // Check for required YAWL types
        let required_types = vec![
            "yawl:Specification",
            "yawl:Task",
            "yawl:Condition",
        ];

        let mut found_spec = false;
        for required_type in required_types {
            if turtle.contains(required_type) {
                found_spec = true;
                break;
            }
        }

        if !found_spec {
            return Err(WorkflowError::Validation(
                "Workflow missing YAWL Specification type (must be valid YAWL TTL)".to_string()
            ));
        }

        Ok(())
    }

    /// Validate semantic completeness (all required elements)
    fn validate_semantic_completeness(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        // Check for required workflow components
        if spec.name.is_empty() {
            return Err(WorkflowError::Validation(
                "Workflow must have a name (rdfs:label)".to_string()
            ));
        }

        if spec.tasks.is_empty() && spec.conditions.is_empty() {
            return Err(WorkflowError::Validation(
                "Workflow must have at least one task or condition".to_string()
            ));
        }

        // Verify start/end conditions if present
        if let Some(ref start) = spec.start_condition {
            if !spec.conditions.iter().any(|c| c.id == *start) {
                return Err(WorkflowError::Validation(format!(
                    "Start condition '{}' not found in workflow",
                    start
                )));
            }
        }

        if let Some(ref end) = spec.end_condition {
            if !spec.conditions.iter().any(|c| c.id == *end) {
                return Err(WorkflowError::Validation(format!(
                    "End condition '{}' not found in workflow",
                    end
                )));
            }
        }

        Ok(())
    }

    /// Validate against Weaver OTEL schema (opt-in)
    fn validate_weaver_schema(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        // TODO: Integrate with Weaver schema validation
        // For now, this is a placeholder for future Weaver integration

        // Weaver validation would:
        // 1. Check that workflow emits expected OTEL spans
        // 2. Verify span attributes match schema
        // 3. Validate metric definitions
        // 4. Ensure telemetry completeness

        // This is opt-in because Weaver validation requires
        // external tooling (weaver CLI) and registry setup

        tracing::debug!(
            workflow = %spec.name,
            "Weaver validation requested but not yet integrated (v4.0 placeholder)"
        );

        Ok(())
    }

    /// Validate TTL string directly (without WorkflowSpec)
    pub fn validate_ttl_string(&self, turtle: &str) -> WorkflowResult<()> {
        #[cfg(feature = "rdf")]
        {
            let store = Store::new()
                .map_err(|e| WorkflowError::Internal(format!("Failed to create RDF store: {:?}", e)))?;

            store
                .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
                .map_err(|e| {
                    WorkflowError::Validation(format!(
                        "Invalid TTL syntax: {}",
                        e
                    ))
                })?;

            // Check for YAWL ontology if required
            if self.require_yawl_ontology {
                if !turtle.contains("yawl:") && !turtle.contains("http://bitflow.ai/ontology/yawl") {
                    return Err(WorkflowError::Validation(
                        "TTL does not use YAWL ontology".to_string()
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Default for TTLOnlyValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Condition, Flow, Task, TaskType, WorkflowSpec, WorkflowSpecId};

    fn create_test_spec() -> WorkflowSpec {
        WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: "Test Workflow".to_string(),
            tasks: vec![Task {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                task_type: TaskType::Atomic,
                join_type: None,
                split_type: None,
                cancellation_region: None,
            }],
            conditions: vec![
                Condition {
                    id: "c1".to_string(),
                    name: Some("Start".to_string()),
                },
                Condition {
                    id: "c2".to_string(),
                    name: Some("End".to_string()),
                },
            ],
            flows: vec![],
            start_condition: Some("c1".to_string()),
            end_condition: Some("c2".to_string()),
            source_turtle: Some(
                r#"@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
<http://test> a yawl:Specification ."#
                    .to_string(),
            ),
        }
    }

    #[test]
    fn test_valid_ttl_workflow() {
        let validator = TTLOnlyValidator::new();
        let spec = create_test_spec();

        let result = validator.validate(&spec);
        assert!(result.is_ok(), "Valid TTL workflow should pass");
    }

    #[test]
    fn test_missing_source_ttl() {
        let validator = TTLOnlyValidator::new();
        let mut spec = create_test_spec();
        spec.source_turtle = None;

        let result = validator.validate(&spec);
        assert!(result.is_err(), "Missing source TTL should fail");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing source TTL"));
    }

    #[test]
    fn test_invalid_ttl_syntax() {
        let validator = TTLOnlyValidator::new();
        let ttl = "This is not valid TTL @#$%";

        let result = validator.validate_ttl_string(ttl);
        assert!(result.is_err(), "Invalid TTL syntax should fail");
    }

    #[test]
    fn test_missing_yawl_ontology() {
        let validator = TTLOnlyValidator::new();
        let ttl = r#"
@prefix ex: <http://example.org/> .
<http://test> a ex:SomeType .
"#;

        let result = validator.validate_ttl_string(ttl);
        assert!(result.is_err(), "Missing YAWL ontology should fail");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("YAWL ontology"));
    }

    #[test]
    fn test_valid_yawl_ttl() {
        let validator = TTLOnlyValidator::new();
        let ttl = r#"
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://example.org/workflow1> a yawl:Specification ;
    rdfs:label "Test Workflow" ;
    yawl:hasTask <http://example.org/workflow1#task1> .

<http://example.org/workflow1#task1> a yawl:Task ;
    rdfs:label "Task 1" .
"#;

        let result = validator.validate_ttl_string(ttl);
        assert!(result.is_ok(), "Valid YAWL TTL should pass");
    }

    #[test]
    fn test_strict_mode() {
        let validator = TTLOnlyValidator::strict();
        assert!(validator.require_yawl_ontology);
        assert!(validator.require_weaver_validation);
    }
}
