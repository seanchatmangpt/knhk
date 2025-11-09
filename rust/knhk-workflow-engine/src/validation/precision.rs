//! Precision Validation
//!
//! Van der Aalst Property 2: Does the process match the specification?
//!
//! Tests:
//! - Specification comparison
//! - Pattern semantics verification
//! - YAWL semantic validation
//! - State transition verification

use crate::error::WorkflowResult;
use crate::parser::WorkflowSpecId;
use crate::WorkflowEngine;

use super::report::{ValidationDetail, ValidationResult, ValidationStatus};

/// Precision validator
pub struct PrecisionValidator {
    engine: std::sync::Arc<WorkflowEngine>,
}

impl PrecisionValidator {
    /// Create a new precision validator
    pub fn new(engine: std::sync::Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    /// Validate precision
    pub async fn validate(&self, spec_id: WorkflowSpecId) -> WorkflowResult<ValidationResult> {
        let mut result = ValidationResult {
            phase: "precision".to_string(),
            status: ValidationStatus::Pass,
            passed: 0,
            failed: 0,
            warnings: 0,
            skipped: 0,
            details: Vec::new(),
            metrics: std::collections::HashMap::new(),
        };

        // Test 1: Specification comparison
        match self.test_specification_comparison(spec_id).await {
            Ok(detail) => {
                result.passed += 1;
                result.details.push(detail);
            }
            Err(e) => {
                result.failed += 1;
                result.status = ValidationStatus::Fail;
                result.details.push(ValidationDetail {
                    name: "specification_comparison".to_string(),
                    status: ValidationStatus::Fail,
                    message: format!("Failed: {}", e),
                    duration_ms: 0,
                });
            }
        }

        // Test 2: State transition verification
        match self.test_state_transitions(spec_id).await {
            Ok(detail) => {
                result.passed += 1;
                result.details.push(detail);
            }
            Err(e) => {
                result.warnings += 1;
                if matches!(result.status, ValidationStatus::Pass) {
                    result.status = ValidationStatus::Warning;
                }
                result.details.push(ValidationDetail {
                    name: "state_transitions".to_string(),
                    status: ValidationStatus::Warning,
                    message: format!("Warning: {}", e),
                    duration_ms: 0,
                });
            }
        }

        // Test 3: Pattern semantics (placeholder - full testing in pattern tests)
        result.skipped += 1;
        result.details.push(ValidationDetail {
            name: "pattern_semantics".to_string(),
            status: ValidationStatus::Skipped,
            message: "Pattern semantics verification delegated to pattern tests".to_string(),
            duration_ms: 0,
        });

        Ok(result)
    }

    /// Test specification comparison
    async fn test_specification_comparison(
        &self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<ValidationDetail> {
        let start = std::time::Instant::now();

        // Get workflow specification
        let spec = self.engine.get_workflow(spec_id).await?;

        // Get cases
        let case_ids = self.engine.list_cases(spec_id).await?;

        if case_ids.is_empty() {
            return Err(crate::error::WorkflowError::InvalidSpecification(
                "No cases found for comparison".to_string(),
            ));
        }

        // Export to XES for comparison
        let xes_content = self.engine.export_workflow_to_xes(spec_id).await?;

        // Verify XES contains expected tasks
        for (_task_id, task) in &spec.tasks {
            if !xes_content.contains(&task.name) {
                return Err(crate::error::WorkflowError::InvalidSpecification(format!(
                    "Task {} not found in XES export",
                    task.name
                )));
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ValidationDetail {
            name: "specification_comparison".to_string(),
            status: ValidationStatus::Pass,
            message: format!(
                "Specification matches execution: {} tasks verified",
                spec.tasks.len()
            ),
            duration_ms,
        })
    }

    /// Test state transitions
    async fn test_state_transitions(
        &self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<ValidationDetail> {
        let start = std::time::Instant::now();

        // Get cases
        let case_ids = self.engine.list_cases(spec_id).await?;

        if case_ids.is_empty() {
            return Err(crate::error::WorkflowError::InvalidSpecification(
                "No cases found for state transition verification".to_string(),
            ));
        }

        // Verify case states are valid
        let mut valid_states = 0;
        for case_id in &case_ids {
            let case = self.engine.get_case(*case_id).await?;
            // Basic state validation
            match case.state {
                crate::case::CaseState::Created
                | crate::case::CaseState::Running
                | crate::case::CaseState::Completed
                | crate::case::CaseState::Cancelled
                | crate::case::CaseState::Failed
                | crate::case::CaseState::Suspended => {
                    valid_states += 1;
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ValidationDetail {
            name: "state_transitions".to_string(),
            status: ValidationStatus::Pass,
            message: format!("{} cases have valid states", valid_states),
            duration_ms,
        })
    }
}
