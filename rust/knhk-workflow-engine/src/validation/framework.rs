//! Van der Aalst End-to-End Validation Framework
//!
//! Orchestrates all validation phases based on Wil M.P. van der Aalst's
//! process mining methodology:
//! - Fitness: Can the process actually be executed?
//! - Precision: Does the process match the specification?
//! - Generalization: Does the process work beyond the examples?

use crate::error::WorkflowResult;
use crate::parser::WorkflowSpecId;
use crate::patterns::RegisterAllExt;
use crate::WorkflowEngine;

use super::fitness::FitnessValidator;
use super::formal::FormalVerifier;
use super::generalization::GeneralizationValidator;
use super::jtbd::{create_default_workflow_pattern_jtbd_scenarios, WorkflowPatternJtbdValidator};
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
    /// 6. JTBD validation (validates patterns accomplish intended purpose)
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

        // Phase 6: JTBD Validation (validates patterns accomplish intended purpose)
        // Create a new registry with all patterns registered
        let mut registry = crate::patterns::PatternRegistry::new();
        registry.register_all_patterns();
        let mut jtbd_validator = create_default_workflow_pattern_jtbd_scenarios(registry);
        let jtbd_results = jtbd_validator.validate_all();
        let jtbd_summary = jtbd_validator.get_summary(&jtbd_results);

        let mut jtbd_details = Vec::new();
        for result in &jtbd_results {
            jtbd_details.push(crate::validation::report::ValidationDetail {
                name: result.pattern_name.clone(),
                status: if result.jtbd_success {
                    crate::validation::report::ValidationStatus::Pass
                } else {
                    crate::validation::report::ValidationStatus::Fail
                },
                message: format!(
                    "Execution: {}, JTBD: {}, Expected: {}, Actual: {}",
                    result.execution_success,
                    result.jtbd_success,
                    result.expected_behavior,
                    result.actual_behavior
                ),
                duration_ms: result.latency_ms,
            });
        }

        let jtbd_result = ValidationResult {
            phase: "jtbd".to_string(),
            status: if jtbd_summary.all_passed() {
                crate::validation::report::ValidationStatus::Pass
            } else {
                crate::validation::report::ValidationStatus::Fail
            },
            passed: jtbd_summary.jtbd_passed,
            failed: jtbd_summary.jtbd_failed + jtbd_summary.execution_failed,
            warnings: 0,
            skipped: 0,
            details: jtbd_details,
            metrics: {
                let mut m = std::collections::HashMap::new();
                m.insert("pass_rate".to_string(), jtbd_summary.pass_rate());
                m.insert(
                    "avg_latency_ms".to_string(),
                    jtbd_summary.avg_latency_ms as f64,
                );
                m
            },
        };
        report.add_phase_result("jtbd", jtbd_result);

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
            "jtbd" => {
                // Create a new registry with all patterns registered
                let mut registry = crate::patterns::PatternRegistry::new();
                registry.register_all_patterns();
                let mut jtbd_validator = create_default_workflow_pattern_jtbd_scenarios(registry);
                let jtbd_results = jtbd_validator.validate_all();
                let jtbd_summary = jtbd_validator.get_summary(&jtbd_results);

                let mut jtbd_details = Vec::new();
                for result in &jtbd_results {
                    jtbd_details.push(crate::validation::report::ValidationDetail {
                        name: result.pattern_name.clone(),
                        status: if result.jtbd_success {
                            crate::validation::report::ValidationStatus::Pass
                        } else {
                            crate::validation::report::ValidationStatus::Fail
                        },
                        message: format!(
                            "Execution: {}, JTBD: {}, Expected: {}, Actual: {}",
                            result.execution_success,
                            result.jtbd_success,
                            result.expected_behavior,
                            result.actual_behavior
                        ),
                        duration_ms: result.latency_ms,
                    });
                }

                Ok(ValidationResult {
                    phase: "jtbd".to_string(),
                    status: if jtbd_summary.all_passed() {
                        crate::validation::report::ValidationStatus::Pass
                    } else {
                        crate::validation::report::ValidationStatus::Fail
                    },
                    passed: jtbd_summary.jtbd_passed,
                    failed: jtbd_summary.jtbd_failed + jtbd_summary.execution_failed,
                    warnings: 0,
                    skipped: 0,
                    details: jtbd_details,
                    metrics: {
                        let mut m = std::collections::HashMap::new();
                        m.insert("pass_rate".to_string(), jtbd_summary.pass_rate());
                        m.insert(
                            "avg_latency_ms".to_string(),
                            jtbd_summary.avg_latency_ms as f64,
                        );
                        m
                    },
                })
            }
            _ => Err(crate::error::WorkflowError::InvalidSpecification(format!(
                "Unknown validation phase: {}",
                phase
            ))),
        }
    }
}
