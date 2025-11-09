//! SPARQL-based workflow validation
//!
//! Implements YAWL validation rules using SPARQL queries against RDF/Turtle workflow definitions.
//! This provides structural validation beyond deadlock detection.

use oxigraph::io::RdfFormat;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;

/// Validation result for a specific SPARQL rule
#[derive(Debug, Clone)]
pub struct SparqlValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// The rule ID that was validated
    pub rule_id: String,
    /// List of violations found (empty if valid)
    pub violations: Vec<ValidationViolation>,
}

/// Individual validation violation
#[derive(Debug, Clone)]
pub struct ValidationViolation {
    /// ID of the RDF element that violates the rule
    pub element_id: String,
    /// Human-readable description of the violation
    pub message: String,
}

/// SPARQL validator for YAWL workflows
pub struct SparqlValidator;

impl SparqlValidator {
    /// Create a new SPARQL validator
    pub fn new() -> Self {
        Self
    }

    /// Validate workflow against all SPARQL rules
    pub async fn validate_workflow(
        &self,
        _turtle: &str,
    ) -> Result<Vec<SparqlValidationResult>, String> {
        Err("SPARQL validation not yet implemented".to_string())
    }

    /// Validate VR-N001: Input Condition Required
    ///
    /// Every workflow specification must have at least one input condition.
    /// SPARQL query checks for specifications without input conditions.
    pub async fn validate_rule_vr_n001(
        &self,
        turtle: &str,
    ) -> Result<SparqlValidationResult, String> {
        // Create RDF store and load turtle
        let store = Store::new().map_err(|e| format!("Failed to create RDF store: {:?}", e))?;

        store
            .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| format!("Failed to parse Turtle: {:?}", e))?;

        // First check if any specifications exist
        let check_spec_query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            SELECT ?spec WHERE {
                ?spec a yawl:Specification .
            }
        "#;

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        let spec_results = oxigraph::sparql::SparqlEvaluator::new()
            .parse_query(check_spec_query)
            .map_err(|e| format!("VR-N001: Failed to parse query: {:?}", e))?
            .on_store(&store)
            .execute()
            .map_err(|e| format!("VR-N001: Failed to check specifications: {:?}", e))?;

        let mut has_specs = false;
        if let QueryResults::Solutions(solutions) = spec_results {
            for _ in solutions {
                has_specs = true;
                break;
            }
        }

        if !has_specs {
            // No specifications found - this is valid (empty workflow)
            return Ok(SparqlValidationResult {
                is_valid: true,
                rule_id: "VR-N001".to_string(),
                violations: Vec::new(),
            });
        }

        // SPARQL query to find specifications without input conditions
        let query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

            SELECT ?spec WHERE {
                ?spec a yawl:Specification .
                FILTER NOT EXISTS {
                    ?spec yawl:hasInputCondition ?ic .
                }
            }
        "#;

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        let results = oxigraph::sparql::SparqlEvaluator::new()
            .parse_query(query)
            .map_err(|e| format!("VR-N001: Failed to parse query: {:?}", e))?
            .on_store(&store)
            .execute()
            .map_err(|e| format!("VR-N001: SPARQL query failed: {:?}", e))?;

        // Check if any violations found
        let mut violations = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| format!("VR-N001: Failed to process query result: {:?}", e))?;

                if let Some(spec) = solution.get("spec") {
                    violations.push(ValidationViolation {
                        element_id: spec.to_string(),
                        message: "Specification missing required input condition".to_string(),
                    });
                }
            }
        }

        if violations.is_empty() {
            Ok(SparqlValidationResult {
                is_valid: true,
                rule_id: "VR-N001".to_string(),
                violations: Vec::new(),
            })
        } else {
            Err(format!(
                "VR-N001: Workflow missing required input condition. Violations: {:?}",
                violations
            ))
        }
    }

    /// Validate VR-DF001: Data Flow Binding Required
    ///
    /// Every input parameter must be bound by an incoming flow.
    /// SPARQL query checks for input parameters without bindings.
    pub async fn validate_rule_vr_df001(
        &self,
        turtle: &str,
    ) -> Result<SparqlValidationResult, String> {
        // Create RDF store and load turtle
        let store = Store::new().map_err(|e| format!("Failed to create RDF store: {:?}", e))?;

        store
            .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| format!("Failed to parse Turtle: {:?}", e))?;

        // SPARQL query to find input parameters without bindings
        let query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

            SELECT ?param ?task WHERE {
                ?task yawl:hasInputParameter ?param .
                ?param a yawl:InputParameter .
                FILTER NOT EXISTS {
                    ?flow yawl:bindsParameter ?param .
                }
            }
        "#;

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        let results = oxigraph::sparql::SparqlEvaluator::new()
            .parse_query(query)
            .map_err(|e| format!("VR-N001: Failed to parse query: {:?}", e))?
            .on_store(&store)
            .execute()
            .map_err(|e| format!("VR-DF001: SPARQL query failed: {:?}", e))?;

        // Check if any violations found
        let mut violations = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| format!("VR-DF001: Failed to process query result: {:?}", e))?;

                if let Some(param) = solution.get("param") {
                    violations.push(ValidationViolation {
                        element_id: param.to_string(),
                        message: "Input parameter not bound by any incoming flow".to_string(),
                    });
                }
            }
        }

        if violations.is_empty() {
            Ok(SparqlValidationResult {
                is_valid: true,
                rule_id: "VR-DF001".to_string(),
                violations: Vec::new(),
            })
        } else {
            Err(format!(
                "VR-DF001: Input parameters not bound by flows. Violations: {:?}",
                violations
            ))
        }
    }
}

impl Default for SparqlValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vr_n001_detects_missing_input_condition() {
        let turtle = r#"
            @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasTask <http://example.org/task1> .

            <http://example.org/task1> a yawl:AtomicTask ;
                yawl:taskName "ProcessOrder" .
        "#;

        let validator = SparqlValidator::new();
        let result = validator.validate_rule_vr_n001(turtle).await;

        assert!(result.is_err(), "Should detect missing input condition");
        let error = result.unwrap_err();
        assert!(
            error.contains("VR-N001"),
            "Error should reference rule VR-N001"
        );
    }

    #[tokio::test]
    async fn test_vr_n001_passes_with_input_condition() {
        let turtle = r#"
            @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasTask <http://example.org/task1> .

            <http://example.org/input1> a yawl:InputCondition ;
                yawl:conditionName "Start" .

            <http://example.org/task1> a yawl:AtomicTask ;
                yawl:taskName "ProcessOrder" .
        "#;

        let validator = SparqlValidator::new();
        let result = validator.validate_rule_vr_n001(turtle).await;

        assert!(result.is_ok(), "Should pass with input condition");
        let validation_result = result.unwrap();
        assert!(validation_result.is_valid);
        assert_eq!(validation_result.rule_id, "VR-N001");
        assert!(validation_result.violations.is_empty());
    }

    #[tokio::test]
    async fn test_vr_df001_detects_unbound_parameter() {
        let turtle = r#"
            @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasTask <http://example.org/task1> .

            <http://example.org/task1> a yawl:AtomicTask ;
                yawl:taskName "ProcessOrder" ;
                yawl:hasInputParameter <http://example.org/param1> .

            <http://example.org/param1> a yawl:InputParameter ;
                yawl:paramName "orderAmount" .
        "#;

        let validator = SparqlValidator::new();
        let result = validator.validate_rule_vr_df001(turtle).await;

        assert!(result.is_err(), "Should detect unbound parameter");
        let error = result.unwrap_err();
        assert!(
            error.contains("VR-DF001"),
            "Error should reference rule VR-DF001"
        );
    }

    #[tokio::test]
    async fn test_vr_df001_passes_with_bound_parameter() {
        let turtle = r#"
            @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasTask <http://example.org/task1> ;
                yawl:hasFlow <http://example.org/flow1> .

            <http://example.org/task1> a yawl:AtomicTask ;
                yawl:taskName "ProcessOrder" ;
                yawl:hasInputParameter <http://example.org/param1> .

            <http://example.org/param1> a yawl:InputParameter ;
                yawl:paramName "orderAmount" .

            <http://example.org/flow1> a yawl:Flow ;
                yawl:flowsInto <http://example.org/task1> ;
                yawl:bindsParameter <http://example.org/param1> ;
                yawl:sourceExpression "netVariable.amount" .
        "#;

        let validator = SparqlValidator::new();
        let result = validator.validate_rule_vr_df001(turtle).await;

        assert!(result.is_ok(), "Should pass with bound parameter");
        let validation_result = result.unwrap();
        assert!(validation_result.is_valid);
        assert_eq!(validation_result.rule_id, "VR-DF001");
        assert!(validation_result.violations.is_empty());
    }
}
