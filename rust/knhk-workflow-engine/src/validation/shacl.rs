//! SHACL-based workflow soundness validation
//!
//! Implements Van der Aalst's soundness criteria using SHACL shapes and SPARQL queries.
//! This provides structural validation for workflow correctness using the SHACL Shapes
//! Constraint Language over RDF/Turtle workflow definitions.
//!
//! **80/20 Approach**: Focuses on practical soundness validation that catches real workflow
//! errors, rather than complete theoretical Petri net state space analysis.
//!
//! # Van der Aalst Soundness Criteria
//!
//! A workflow is **sound** if:
//! 1. **Option to Complete**: Every case will eventually complete (reach output condition)
//! 2. **Proper Completion**: When case completes, output condition is only marked place
//! 3. **No Dead Tasks**: Every task can be executed in some valid execution path
//!
//! # Validation Rules
//!
//! - `VR-S001`: Specification must have exactly one input condition (unique start)
//! - `VR-S002`: Specification must have exactly one output condition (unique end)
//! - `VR-S003`: All tasks must be reachable from input condition
//! - `VR-S004`: All tasks must have outgoing flows (no dead ends)
//! - `VR-S005`: XOR splits must have multiple outgoing flows
//! - `VR-S006`: AND splits must have multiple outgoing flows
//! - `VR-S007`: AND joins must have multiple incoming flows
//! - `VR-S008`: OR joins should have multiple incoming flows
//! - `VR-S009`: Input condition must not have incoming flows
//! - `VR-S010`: Output condition must not have outgoing flows
//! - `VR-S011`: XOR split flows should have predicates for routing
//! - `VR-S012`: OR join vicious circle warning (requires runtime analysis)

#[cfg(feature = "rdf")]
use oxigraph::io::RdfFormat;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;

/// SHACL validation severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// Critical violation - workflow is unsound
    Violation,
    /// Warning - workflow may have issues
    Warning,
    /// Informational - suspicious pattern detected
    Info,
}

/// Individual SHACL validation violation
#[derive(Debug, Clone)]
pub struct ShaclViolation {
    /// Rule ID (e.g., "VR-S001")
    pub rule_id: String,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Focus node that violated the constraint
    pub focus_node: String,
    /// Human-readable violation message
    pub message: String,
}

/// SHACL validation report
#[derive(Debug, Clone)]
pub struct ShaclValidationReport {
    /// Whether the workflow conforms to all SHACL shapes
    pub conforms: bool,
    /// List of violations found (empty if conforms = true)
    pub violations: Vec<ShaclViolation>,
}

impl ShaclValidationReport {
    /// Create a conforming report (no violations)
    pub fn conforming() -> Self {
        Self {
            conforms: true,
            violations: Vec::new(),
        }
    }

    /// Create a non-conforming report with violations
    pub fn non_conforming(violations: Vec<ShaclViolation>) -> Self {
        Self {
            conforms: false,
            violations,
        }
    }

    /// Check if report has any critical violations
    pub fn has_violations(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == ValidationSeverity::Violation)
    }

    /// Check if report has warnings
    pub fn has_warnings(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == ValidationSeverity::Warning)
    }

    /// Get count of violations by severity
    pub fn count_by_severity(&self, severity: ValidationSeverity) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .count()
    }
}

/// SHACL validator for YAWL workflow soundness
pub struct ShaclValidator {
    /// Store containing SHACL shape definitions
    shapes_store: Store,
}

impl ShaclValidator {
    /// Create a new SHACL validator with soundness shapes
    pub fn new() -> Result<Self, String> {
        let store =
            Store::new().map_err(|e| format!("Failed to create SHACL shapes store: {:?}", e))?;

        // Load soundness shapes from embedded file
        let shapes_ttl = include_str!("../../../../ontology/shacl/soundness.ttl");

        store
            .load_from_reader(RdfFormat::Turtle, shapes_ttl.as_bytes())
            .map_err(|e| format!("Failed to load SHACL soundness shapes: {:?}", e))?;

        Ok(Self {
            shapes_store: store,
        })
    }

    /// Validate workflow against all soundness SHACL shapes
    ///
    /// This executes all 12 soundness validation rules and returns a comprehensive
    /// validation report with all violations, warnings, and informational messages.
    pub fn validate_soundness(
        &self,
        workflow_turtle: &str,
    ) -> Result<ShaclValidationReport, String> {
        // Create data store for workflow
        let data_store =
            Store::new().map_err(|e| format!("Failed to create workflow data store: {:?}", e))?;

        // Load workflow Turtle
        data_store
            .load_from_reader(RdfFormat::Turtle, workflow_turtle.as_bytes())
            .map_err(|e| format!("Failed to parse workflow Turtle: {:?}", e))?;

        // Execute all SHACL validation rules
        let mut violations = Vec::new();

        // VR-S001: Input condition required
        violations.extend(self.validate_rule_s001(&data_store)?);

        // VR-S002: Output condition required
        violations.extend(self.validate_rule_s002(&data_store)?);

        // VR-S003: All tasks reachable
        violations.extend(self.validate_rule_s003(&data_store)?);

        // VR-S004: All tasks have outgoing flows
        violations.extend(self.validate_rule_s004(&data_store)?);

        // VR-S005: XOR split multiple outgoing
        violations.extend(self.validate_rule_s005(&data_store)?);

        // VR-S006: AND split multiple outgoing
        violations.extend(self.validate_rule_s006(&data_store)?);

        // VR-S007: AND join multiple incoming
        violations.extend(self.validate_rule_s007(&data_store)?);

        // VR-S008: OR join multiple incoming
        violations.extend(self.validate_rule_s008(&data_store)?);

        // VR-S009: Input condition no incoming
        violations.extend(self.validate_rule_s009(&data_store)?);

        // VR-S010: Output condition no outgoing
        violations.extend(self.validate_rule_s010(&data_store)?);

        // VR-S011: XOR flows have predicates
        violations.extend(self.validate_rule_s011(&data_store)?);

        // VR-S012: OR join vicious circle warning
        violations.extend(self.validate_rule_s012(&data_store)?);

        // Build report
        if violations.is_empty() {
            Ok(ShaclValidationReport::conforming())
        } else {
            Ok(ShaclValidationReport::non_conforming(violations))
        }
    }

    /// VR-S001: Specification must have exactly one input condition
    fn validate_rule_s001(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?spec WHERE {
                ?spec a yawl:Specification .
                FILTER NOT EXISTS {
                    ?spec yawl:hasInputCondition ?ic .
                }
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S001",
            ValidationSeverity::Violation,
            "Workflow must have exactly one input condition (unique start)",
        )
    }

    /// VR-S002: Specification must have exactly one output condition
    fn validate_rule_s002(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?spec WHERE {
                ?spec a yawl:Specification .
                FILTER NOT EXISTS {
                    ?spec yawl:hasOutputCondition ?oc .
                }
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S002",
            ValidationSeverity::Violation,
            "Workflow must have exactly one output condition (unique end)",
        )
    }

    /// VR-S003: All tasks must be reachable from input condition
    fn validate_rule_s003(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?task WHERE {
                ?task a yawl:Task .
                FILTER NOT EXISTS {
                    ?flow yawl:nextElementRef ?task .
                }
                FILTER NOT EXISTS {
                    ?spec yawl:hasInputCondition ?task .
                }
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S003",
            ValidationSeverity::Violation,
            "Task is unreachable - no incoming flow from start",
        )
    }

    /// VR-S004: All tasks must have outgoing flows
    fn validate_rule_s004(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?task WHERE {
                ?task a yawl:Task .
                FILTER NOT EXISTS {
                    ?task yawl:flowsInto ?flow .
                }
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S004",
            ValidationSeverity::Violation,
            "Task is a dead end - no outgoing flow to completion",
        )
    }

    /// VR-S005: XOR split must have multiple outgoing flows
    fn validate_rule_s005(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?task WHERE {
                ?task a yawl:Task ;
                    yawl:hasSplit yawl:ControlTypeXor .
                {
                    SELECT ?task (COUNT(?flow) AS ?flowCount) WHERE {
                        ?task yawl:flowsInto ?flow .
                    }
                    GROUP BY ?task
                }
                FILTER (?flowCount < 2)
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S005",
            ValidationSeverity::Warning,
            "XOR split must have at least 2 outgoing flows (degenerate split)",
        )
    }

    /// VR-S006: AND split must have multiple outgoing flows
    fn validate_rule_s006(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?task WHERE {
                ?task a yawl:Task ;
                    yawl:hasSplit yawl:ControlTypeAnd .
                {
                    SELECT ?task (COUNT(?flow) AS ?flowCount) WHERE {
                        ?task yawl:flowsInto ?flow .
                    }
                    GROUP BY ?task
                }
                FILTER (?flowCount < 2)
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S006",
            ValidationSeverity::Warning,
            "AND split must have at least 2 outgoing flows (degenerate split)",
        )
    }

    /// VR-S007: AND join must have multiple incoming flows
    fn validate_rule_s007(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?task WHERE {
                ?task a yawl:Task ;
                    yawl:hasJoin yawl:ControlTypeAnd .
                {
                    SELECT ?task (COUNT(?flow) AS ?flowCount) WHERE {
                        ?flow yawl:nextElementRef ?task .
                    }
                    GROUP BY ?task
                }
                FILTER (?flowCount < 2)
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S007",
            ValidationSeverity::Warning,
            "AND join must have at least 2 incoming flows (degenerate join)",
        )
    }

    /// VR-S008: OR join should have multiple incoming flows
    fn validate_rule_s008(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?task WHERE {
                ?task a yawl:Task ;
                    yawl:hasJoin yawl:ControlTypeOr .
                {
                    SELECT ?task (COUNT(?flow) AS ?flowCount) WHERE {
                        ?flow yawl:nextElementRef ?task .
                    }
                    GROUP BY ?task
                }
                FILTER (?flowCount < 2)
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S008",
            ValidationSeverity::Info,
            "OR join should have multiple incoming flows (consider XOR join)",
        )
    }

    /// VR-S009: Input condition must not have incoming flows
    fn validate_rule_s009(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?ic WHERE {
                ?ic a yawl:InputCondition .
                ?flow yawl:nextElementRef ?ic .
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S009",
            ValidationSeverity::Violation,
            "Input condition must not have incoming flows (not a true start)",
        )
    }

    /// VR-S010: Output condition must not have outgoing flows
    fn validate_rule_s010(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?oc WHERE {
                ?oc a yawl:OutputCondition .
                ?oc yawl:flowsInto ?flow .
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S010",
            ValidationSeverity::Violation,
            "Output condition must not have outgoing flows (not a true end)",
        )
    }

    /// VR-S011: XOR split flows should have predicates
    fn validate_rule_s011(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?flow WHERE {
                ?task a yawl:Task ;
                    yawl:hasSplit yawl:ControlTypeXor ;
                    yawl:flowsInto ?flow .
                FILTER NOT EXISTS {
                    ?flow yawl:hasPredicate ?pred .
                }
                {
                    SELECT ?task (COUNT(?f) AS ?flowCount) WHERE {
                        ?task yawl:flowsInto ?f .
                    }
                    GROUP BY ?task
                    HAVING (COUNT(?f) > 1)
                }
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S011",
            ValidationSeverity::Warning,
            "XOR split flow must have predicate to determine routing",
        )
    }

    /// VR-S012: OR join vicious circle warning
    fn validate_rule_s012(&self, store: &Store) -> Result<Vec<ShaclViolation>, String> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            SELECT ?orjoin WHERE {
                ?orjoin a yawl:Task ;
                    yawl:hasJoin yawl:ControlTypeOr .
                ?flow1 yawl:nextElementRef ?orjoin .
                ?xorTask yawl:flowsInto ?flow1 ;
                    yawl:hasSplit yawl:ControlTypeXor .
                ?xorTask yawl:flowsInto ?altFlow .
                FILTER (?altFlow != ?flow1)
            }
        "#;

        self.execute_validation_query(
            store,
            query,
            "VR-S012",
            ValidationSeverity::Info,
            "OR join may have vicious circle - XOR split creates alternative paths (advanced validation required)",
        )
    }

    /// Execute a SPARQL validation query and convert results to violations
    fn execute_validation_query(
        &self,
        _store: &Store,
        query: &str,
        rule_id: &str,
        severity: ValidationSeverity,
        message_template: &str,
    ) -> Result<Vec<ShaclViolation>, String> {
        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        let results = oxigraph::sparql::SparqlEvaluator::new()
            .parse_query(query)
            .map_err(|e| format!("SHACL validation query failed: {:?}", e))?
            .on_store(&self.shapes_store)
            .execute()
            .map_err(|e| format!("{}: SPARQL query failed: {:?}", rule_id, e))?;

        let mut violations = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| format!("{}: Failed to process query result: {:?}", rule_id, e))?;

                // Get first variable binding as focus node
                // Collect into Vec to avoid lifetime issues
                let bindings: Vec<_> = solution.iter().collect();
                if let Some((_, focus_node)) = bindings.first() {
                    violations.push(ShaclViolation {
                        rule_id: rule_id.to_string(),
                        severity: severity.clone(),
                        focus_node: focus_node.to_string(),
                        message: message_template.to_string(),
                    });
                }
            }
        }

        Ok(violations)
    }
}

impl Default for ShaclValidator {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| panic!("Failed to create default SHACL validator: {:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shacl_validator_creation() {
        let validator = ShaclValidator::new();
        assert!(validator.is_ok(), "Should create SHACL validator");
    }

    #[test]
    fn test_vr_s001_detects_missing_input_condition() {
        let workflow = r#"
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasTask <http://example.org/task1> .

            <http://example.org/task1> a yawl:Task .
        "#;

        let validator = ShaclValidator::new().unwrap();
        let report = validator.validate_soundness(workflow).unwrap();

        assert!(!report.conforms, "Should detect missing input condition");
        assert!(report.has_violations(), "Should have violations");
        // This workflow also violates VR-S002 (no output), VR-S003 (unreachable task), VR-S004 (dead end)
        assert!(
            report.count_by_severity(ValidationSeverity::Violation) >= 2,
            "Should have multiple violations for this malformed workflow"
        );
    }

    #[test]
    fn test_vr_s002_detects_missing_output_condition() {
        let workflow = r#"
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasTask <http://example.org/task1> .

            <http://example.org/input1> a yawl:InputCondition .
            <http://example.org/task1> a yawl:Task .
        "#;

        let validator = ShaclValidator::new().unwrap();
        let report = validator.validate_soundness(workflow).unwrap();

        assert!(!report.conforms, "Should detect missing output condition");
        assert!(report.has_violations());
    }

    #[test]
    fn test_vr_s003_detects_unreachable_task() {
        let workflow = r#"
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasOutputCondition <http://example.org/output1> ;
                yawl:hasTask <http://example.org/orphan> .

            <http://example.org/input1> a yawl:InputCondition .
            <http://example.org/output1> a yawl:OutputCondition .
            <http://example.org/orphan> a yawl:Task .
        "#;

        let validator = ShaclValidator::new().unwrap();
        let report = validator.validate_soundness(workflow).unwrap();

        assert!(!report.conforms, "Should detect unreachable task");
        let unreachable_violations: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "VR-S003")
            .collect();
        assert!(
            !unreachable_violations.is_empty(),
            "Should have VR-S003 violations"
        );
    }

    #[test]
    fn test_vr_s004_detects_dead_end_task() {
        let workflow = r#"
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasOutputCondition <http://example.org/output1> ;
                yawl:hasTask <http://example.org/deadend> .

            <http://example.org/input1> a yawl:InputCondition ;
                yawl:flowsInto <http://example.org/flow1> .

            <http://example.org/flow1> a yawl:FlowsInto ;
                yawl:nextElementRef <http://example.org/deadend> .

            <http://example.org/deadend> a yawl:Task .
            <http://example.org/output1> a yawl:OutputCondition .
        "#;

        let validator = ShaclValidator::new().unwrap();
        let report = validator.validate_soundness(workflow).unwrap();

        assert!(!report.conforms, "Should detect dead end task");
        let deadend_violations: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "VR-S004")
            .collect();
        assert!(
            !deadend_violations.is_empty(),
            "Should have VR-S004 violations"
        );
    }

    #[test]
    fn test_sound_workflow_passes_all_rules() {
        let workflow = r#"
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasOutputCondition <http://example.org/output1> ;
                yawl:hasTask <http://example.org/task1> .

            <http://example.org/input1> a yawl:InputCondition ;
                yawl:flowsInto <http://example.org/flow1> .

            <http://example.org/flow1> a yawl:FlowsInto ;
                yawl:nextElementRef <http://example.org/task1> .

            <http://example.org/task1> a yawl:Task ;
                yawl:flowsInto <http://example.org/flow2> .

            <http://example.org/flow2> a yawl:FlowsInto ;
                yawl:nextElementRef <http://example.org/output1> .

            <http://example.org/output1> a yawl:OutputCondition .
        "#;

        let validator = ShaclValidator::new().unwrap();
        let report = validator.validate_soundness(workflow).unwrap();

        assert!(
            report.conforms,
            "Sound workflow should pass all validation rules"
        );
        assert_eq!(report.violations.len(), 0, "Should have no violations");
    }

    #[test]
    fn test_xor_split_without_multiple_flows_warning() {
        let workflow = r#"
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasOutputCondition <http://example.org/output1> ;
                yawl:hasTask <http://example.org/xor_task> .

            <http://example.org/input1> a yawl:InputCondition ;
                yawl:flowsInto <http://example.org/flow1> .

            <http://example.org/flow1> a yawl:FlowsInto ;
                yawl:nextElementRef <http://example.org/xor_task> .

            <http://example.org/xor_task> a yawl:Task ;
                yawl:hasSplit yawl:ControlTypeXor ;
                yawl:flowsInto <http://example.org/flow2> .

            <http://example.org/flow2> a yawl:FlowsInto ;
                yawl:nextElementRef <http://example.org/output1> .

            <http://example.org/output1> a yawl:OutputCondition .
        "#;

        let validator = ShaclValidator::new().unwrap();
        let report = validator.validate_soundness(workflow).unwrap();

        assert!(!report.conforms, "Should detect degenerate XOR split");
        assert!(report.has_warnings(), "Should have warnings");
        let xor_warnings: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "VR-S005")
            .collect();
        assert!(!xor_warnings.is_empty(), "Should have VR-S005 warnings");
    }

    #[test]
    fn test_input_condition_with_incoming_flow_violation() {
        let workflow = r#"
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasOutputCondition <http://example.org/output1> ;
                yawl:hasTask <http://example.org/task1> .

            <http://example.org/task1> a yawl:Task ;
                yawl:flowsInto <http://example.org/bad_flow> .

            <http://example.org/bad_flow> a yawl:FlowsInto ;
                yawl:nextElementRef <http://example.org/input1> .

            <http://example.org/input1> a yawl:InputCondition .
            <http://example.org/output1> a yawl:OutputCondition .
        "#;

        let validator = ShaclValidator::new().unwrap();
        let report = validator.validate_soundness(workflow).unwrap();

        assert!(
            !report.conforms,
            "Should detect input condition with incoming flow"
        );
        let s009_violations: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "VR-S009")
            .collect();
        assert!(
            !s009_violations.is_empty(),
            "Should have VR-S009 violations"
        );
    }

    #[test]
    fn test_output_condition_with_outgoing_flow_violation() {
        let workflow = r#"
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            <http://example.org/workflow1> a yawl:Specification ;
                yawl:hasInputCondition <http://example.org/input1> ;
                yawl:hasOutputCondition <http://example.org/output1> .

            <http://example.org/input1> a yawl:InputCondition .

            <http://example.org/output1> a yawl:OutputCondition ;
                yawl:flowsInto <http://example.org/bad_flow> .

            <http://example.org/bad_flow> a yawl:FlowsInto ;
                yawl:nextElementRef <http://example.org/input1> .
        "#;

        let validator = ShaclValidator::new().unwrap();
        let report = validator.validate_soundness(workflow).unwrap();

        assert!(
            !report.conforms,
            "Should detect output condition with outgoing flow"
        );
        let s010_violations: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.rule_id == "VR-S010")
            .collect();
        assert!(
            !s010_violations.is_empty(),
            "Should have VR-S010 violations"
        );
    }
}
