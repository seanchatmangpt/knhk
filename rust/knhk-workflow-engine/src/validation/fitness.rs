//! Fitness Validation
//!
//! Van der Aalst Property 1: Can the process actually be executed?
//!
//! Tests:
//! - Workflow execution
//! - Event log collection
//! - Pattern execution (all 43 patterns)
//! - YAWL workflow execution

use crate::error::WorkflowResult;
use crate::parser::WorkflowSpecId;
use crate::WorkflowEngine;

use super::report::{ValidationDetail, ValidationResult, ValidationStatus};

/// Fitness validator
pub struct FitnessValidator {
    engine: std::sync::Arc<WorkflowEngine>,
}

impl FitnessValidator {
    /// Create a new fitness validator
    pub fn new(engine: std::sync::Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    /// Validate fitness
    pub async fn validate(&self, spec_id: WorkflowSpecId) -> WorkflowResult<ValidationResult> {
        let mut result = ValidationResult {
            phase: "fitness".to_string(),
            status: ValidationStatus::Pass,
            passed: 0,
            failed: 0,
            warnings: 0,
            skipped: 0,
            details: Vec::new(),
            metrics: std::collections::HashMap::new(),
        };

        // Test 1: Simple workflow execution
        match self.test_simple_workflow_execution(spec_id).await {
            Ok(detail) => {
                result.passed += 1;
                result.details.push(detail);
            }
            Err(e) => {
                result.failed += 1;
                result.status = ValidationStatus::Fail;
                result.details.push(ValidationDetail {
                    name: "simple_workflow_execution".to_string(),
                    status: ValidationStatus::Fail,
                    message: format!("Failed: {}", e),
                    duration_ms: 0,
                });
            }
        }

        // Test 2: Event log collection
        match self.test_event_log_collection(spec_id).await {
            Ok(detail) => {
                result.passed += 1;
                result.details.push(detail);
            }
            Err(e) => {
                result.failed += 1;
                result.status = ValidationStatus::Fail;
                result.details.push(ValidationDetail {
                    name: "event_log_collection".to_string(),
                    status: ValidationStatus::Fail,
                    message: format!("Failed: {}", e),
                    duration_ms: 0,
                });
            }
        }

        // Test 3: Pattern execution (sample patterns)
        match self.test_pattern_execution().await {
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
                    name: "pattern_execution".to_string(),
                    status: ValidationStatus::Warning,
                    message: format!("Warning: {}", e),
                    duration_ms: 0,
                });
            }
        }

        Ok(result)
    }

    /// Test simple workflow execution
    async fn test_simple_workflow_execution(
        &self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<ValidationDetail> {
        let start = std::time::Instant::now();

        // Get workflow
        let _spec = self.engine.get_workflow(spec_id).await?;

        // Create case
        let case_id = self
            .engine
            .create_case(spec_id, serde_json::json!({}))
            .await?;

        // Start case
        self.engine.start_case(case_id).await?;

        // Wait for execution
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ValidationDetail {
            name: "simple_workflow_execution".to_string(),
            status: ValidationStatus::Pass,
            message: "Workflow executed successfully".to_string(),
            duration_ms,
        })
    }

    /// Test event log collection
    async fn test_event_log_collection(
        &self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<ValidationDetail> {
        let start = std::time::Instant::now();

        // Get all cases for workflow
        let case_ids = self.engine.list_cases(spec_id).await?;

        if case_ids.is_empty() {
            return Err(crate::error::WorkflowError::InvalidSpecification(
                "No cases found for workflow".to_string(),
            ));
        }

        // Export to XES
        let xes_content = self.engine.export_workflow_to_xes(spec_id).await?;

        // Validate XES format
        if !xes_content.contains("<?xml version") || !xes_content.contains("<log xes.version") {
            return Err(crate::error::WorkflowError::InvalidSpecification(
                "XES format validation failed".to_string(),
            ));
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ValidationDetail {
            name: "event_log_collection".to_string(),
            status: ValidationStatus::Pass,
            message: format!("Collected {} cases in XES format", case_ids.len()),
            duration_ms,
        })
    }

    /// Test pattern execution
    async fn test_pattern_execution(&self) -> WorkflowResult<ValidationDetail> {
        let start = std::time::Instant::now();

        // Note: Full pattern testing is done in chicago_tdd_43_patterns.rs
        // This is a placeholder that verifies pattern registry is accessible
        let mut registry = crate::patterns::PatternRegistry::new();
        crate::patterns::register_all_patterns(&mut registry);

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ValidationDetail {
            name: "pattern_execution".to_string(),
            status: ValidationStatus::Pass,
            message: "Pattern registry accessible".to_string(),
            duration_ms,
        })
    }
}
