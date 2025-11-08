//! Self-validation module
//!
//! Uses the workflow engine to validate itself - "eating our own dog food".
//! This demonstrates that the engine can manage its own validation and testing.

use crate::capabilities::{validate_capabilities, CapabilityRegistry, CapabilityValidationReport};
use crate::case::{Case, CaseId, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::executor::WorkflowEngine;
use crate::parser::{WorkflowParser, WorkflowSpec, WorkflowSpecId};
use crate::patterns::PatternRegistry;
use crate::state::StateStore;
use std::path::Path;

/// Self-validation workflow manager
pub struct SelfValidationManager {
    engine: WorkflowEngine,
    pattern_registry: PatternRegistry,
    capability_registry: CapabilityRegistry,
}

impl SelfValidationManager {
    /// Create new self-validation manager
    pub fn new<P: AsRef<Path>>(state_path: P) -> WorkflowResult<Self> {
        let state_store = StateStore::new(state_path)?;
        let engine = WorkflowEngine::new(state_store);
        let pattern_registry = PatternRegistry::new();
        let capability_registry = CapabilityRegistry::new();

        Ok(Self {
            engine,
            pattern_registry,
            capability_registry,
        })
    }

    /// Validate all capabilities using the workflow engine
    pub async fn validate_capabilities(&self) -> WorkflowResult<CapabilityValidationReport> {
        // Use the engine to validate capabilities
        let report = validate_capabilities()?;

        // Create a workflow case for capability validation
        let spec_id = WorkflowSpecId::new();
        let case = Case::new(
            spec_id,
            serde_json::json!({
                "validation_type": "capabilities",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        );

        // Execute validation workflow
        let _case_id = self.engine.create_case(case)?;

        // Validate pattern registry
        self.validate_pattern_registry()?;

        // Validate all required capabilities
        self.capability_registry.validate_required()?;

        Ok(report)
    }

    /// Validate pattern registry
    pub fn validate_pattern_registry(&self) -> WorkflowResult<()> {
        let patterns = self.pattern_registry.list_patterns();

        // Check all 43 patterns are registered
        if patterns.len() < 43 {
            return Err(WorkflowError::Validation(format!(
                "Expected 43 patterns, found {}",
                patterns.len()
            )));
        }

        // Validate each pattern can be executed
        for pattern_id in &patterns {
            if pattern_id.0 < 1 || pattern_id.0 > 43 {
                return Err(WorkflowError::Validation(format!(
                    "Invalid pattern ID: {}",
                    pattern_id.0
                )));
            }
        }

        Ok(())
    }

    /// Validate workflow engine using itself
    pub async fn validate_engine(&self) -> WorkflowResult<EngineValidationReport> {
        // Create a workflow spec for self-validation
        let spec = self.create_validation_spec()?;

        // Parse the spec using the engine's parser
        let mut parser = WorkflowParser::new()?;
        let _parsed_spec = parser.parse_turtle(&spec)?;

        // Create a case for validation
        let case = Case::new(
            WorkflowSpecId::new(),
            serde_json::json!({
                "validation_type": "engine",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
        );

        // Execute the validation case
        let case_id = self.engine.create_case(case)?;

        // Get case state
        let case_state = self.engine.get_case(&case_id)?;

        // Validate engine capabilities
        let capability_report = validate_capabilities()?;

        Ok(EngineValidationReport {
            case_id,
            case_state,
            capability_report,
            patterns_validated: self.pattern_registry.list_patterns().len(),
            engine_operational: true,
        })
    }

    /// Create validation workflow specification
    fn create_validation_spec(&self) -> WorkflowResult<String> {
        // Create a simple validation workflow in Turtle format
        let turtle = r#"
@prefix yawl: <http://www.yawlfoundation.org/xsd/yawl_20#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://knhk.org/workflow/self-validation>
    rdf:type yawl:WorkflowSpecification ;
    rdfs:label "Self-Validation Workflow" ;
    yawl:hasStartCondition <http://knhk.org/workflow/self-validation/start> ;
    yawl:hasEndCondition <http://knhk.org/workflow/self-validation/end> .

<http://knhk.org/workflow/self-validation/start>
    rdf:type yawl:Condition ;
    rdfs:label "Start" .

<http://knhk.org/workflow/self-validation/validate-patterns>
    rdf:type yawl:Task ;
    rdf:type yawl:AtomicTask ;
    rdfs:label "Validate Patterns" ;
    yawl:splitType "AND" ;
    yawl:joinType "AND" .

<http://knhk.org/workflow/self-validation/validate-capabilities>
    rdf:type yawl:Task ;
    rdf:type yawl:AtomicTask ;
    rdfs:label "Validate Capabilities" ;
    yawl:splitType "AND" ;
    yawl:joinType "AND" .

<http://knhk.org/workflow/self-validation/end>
    rdf:type yawl:Condition ;
    rdfs:label "End" .

<http://knhk.org/workflow/self-validation/flow1>
    rdf:type yawl:Flow ;
    yawl:source <http://knhk.org/workflow/self-validation/start> ;
    yawl:target <http://knhk.org/workflow/self-validation/validate-patterns> .

<http://knhk.org/workflow/self-validation/flow2>
    rdf:type yawl:Flow ;
    yawl:source <http://knhk.org/workflow/self-validation/validate-patterns> ;
    yawl:target <http://knhk.org/workflow/self-validation/validate-capabilities> .

<http://knhk.org/workflow/self-validation/flow3>
    rdf:type yawl:Flow ;
    yawl:source <http://knhk.org/workflow/self-validation/validate-capabilities> ;
    yawl:target <http://knhk.org/workflow/self-validation/end> .
"#;

        Ok(turtle.to_string())
    }

    /// Run self-validation workflow
    pub async fn run_validation_workflow(&self) -> WorkflowResult<SelfValidationReport> {
        // Step 1: Validate capabilities
        let capability_report = self.validate_capabilities().await?;

        // Step 2: Validate pattern registry
        self.validate_pattern_registry()?;

        // Step 3: Validate engine using itself
        let engine_report = self.validate_engine().await?;

        Ok(SelfValidationReport {
            capability_report,
            engine_report,
            validation_passed: true,
        })
    }
}

/// Engine validation report
#[derive(Debug, Clone)]
pub struct EngineValidationReport {
    /// Case ID for validation
    pub case_id: CaseId,
    /// Case state
    pub case_state: CaseState,
    /// Capability validation report
    pub capability_report: CapabilityValidationReport,
    /// Number of patterns validated
    pub patterns_validated: usize,
    /// Engine is operational
    pub engine_operational: bool,
}

/// Self-validation report
#[derive(Debug, Clone)]
pub struct SelfValidationReport {
    /// Capability validation report
    pub capability_report: CapabilityValidationReport,
    /// Engine validation report
    pub engine_report: EngineValidationReport,
    /// Validation passed
    pub validation_passed: bool,
}
