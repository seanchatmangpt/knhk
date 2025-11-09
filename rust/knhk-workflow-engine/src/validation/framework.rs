//! Van der Aalst End-to-End Validation Framework
//!
//! Orchestrates all validation phases based on Wil M.P. van der Aalst's
//! process mining methodology:
//! - Fitness: Can the process actually be executed?
//! - Precision: Does the process match the specification?
//! - Generalization: Does the process work beyond the examples?

use crate::case::CaseId;
use crate::error::WorkflowResult;
use crate::parser::WorkflowSpecId;
use crate::WorkflowEngine;

use super::fitness::FitnessValidator;
use super::formal::FormalVerifier;
use super::generalization::GeneralizationValidator;
use super::precision::PrecisionValidator;
use super::process_mining::ProcessMiningAnalyzer;
use super::report::{ValidationReport, ValidationResult};

/// Validation framework orchestrator
pub struct ValidationFramework {
    engine: std::sync::Arc<WorkflowEngine>,
}

impl ValidationFramework {
    /// Create a new validation framework
    pub fn new(engine: std::sync::Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    /// Run complete validation framework
    ///
    /// Executes all validation phases:
    /// 1. Fitness validation
    /// 2. Precision validation
    /// 3. Generalization validation
    /// 4. Process mining analysis
    /// 5. Formal verification
    pub async fn run_complete_validation(
        &self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<ValidationReport> {
        let mut report = ValidationReport::new(spec_id);

        // Phase 1: Fitness Validation
        let fitness_validator = FitnessValidator::new(self.engine.clone());
        let fitness_result = fitness_validator.validate(spec_id).await?;
        report.add_phase_result("fitness", fitness_result);

        // Phase 2: Precision Validation
        let precision_validator = PrecisionValidator::new(self.engine.clone());
        let precision_result = precision_validator.validate(spec_id).await?;
        report.add_phase_result("precision", precision_result);

        // Phase 3: Generalization Validation
        let generalization_validator = GeneralizationValidator::new(self.engine.clone());
        let generalization_result = generalization_validator.validate(spec_id).await?;
        report.add_phase_result("generalization", generalization_result);

        // Phase 4: Process Mining Analysis
        let process_mining_analyzer = ProcessMiningAnalyzer::new(self.engine.clone());
        let process_mining_result = process_mining_analyzer.analyze(spec_id).await?;
        report.add_phase_result("process_mining", process_mining_result);

        // Phase 5: Formal Verification
        let formal_verifier = FormalVerifier::new(self.engine.clone());
        let formal_result = formal_verifier.verify(spec_id).await?;
        report.add_phase_result("formal", formal_result);

        Ok(report)
    }

    /// Run specific validation phase
    pub async fn run_phase(
        &self,
        phase: &str,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<ValidationResult> {
        match phase {
            "fitness" => {
                let validator = FitnessValidator::new(self.engine.clone());
                validator.validate(spec_id).await
            }
            "precision" => {
                let validator = PrecisionValidator::new(self.engine.clone());
                validator.validate(spec_id).await
            }
            "generalization" => {
                let validator = GeneralizationValidator::new(self.engine.clone());
                validator.validate(spec_id).await
            }
            "process_mining" => {
                let analyzer = ProcessMiningAnalyzer::new(self.engine.clone());
                analyzer.analyze(spec_id).await
            }
            "formal" => {
                let verifier = FormalVerifier::new(self.engine.clone());
                verifier.verify(spec_id).await
            }
            _ => Err(crate::error::WorkflowError::InvalidSpecification(format!(
                "Unknown validation phase: {}",
                phase
            ))),
        }
    }
}
