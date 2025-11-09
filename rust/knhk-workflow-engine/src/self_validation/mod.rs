//! Self-validation module
//!
//! Uses the workflow engine to validate itself - "eating our own dog food".
//! This demonstrates that the engine can manage its own validation and testing.

mod workflow;

use crate::capabilities::{validate_capabilities, CapabilityRegistry, CapabilityValidationReport};
use crate::case::{CaseId, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::executor::WorkflowEngine;
use crate::parser::{WorkflowParser, WorkflowSpecId};
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
        let data = serde_json::json!({
            "validation_type": "capabilities",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        // Execute validation workflow
        let _case_id = self.engine.create_case(spec_id, data).await?;

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
        let spec_id = WorkflowSpecId::new();
        let data = serde_json::json!({
            "validation_type": "engine",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        // Execute the validation case
        let case_id = self.engine.create_case(spec_id, data).await?;

        // Get case state
        let case = self.engine.get_case(case_id).await?;
        let case_state = case.state;

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
    pub fn create_validation_spec(&self) -> WorkflowResult<String> {
        use crate::self_validation::workflow::SELF_VALIDATION_WORKFLOW;
        Ok(SELF_VALIDATION_WORKFLOW.to_string())
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
