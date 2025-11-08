//! Case management methods

use crate::case::{Case, CaseId, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::fortune5::RuntimeClass;
use crate::parser::WorkflowSpecId;
use std::time::Instant;

use super::task;
use super::WorkflowEngine;

impl WorkflowEngine {
    /// Create a new case
    pub async fn create_case(
        &self,
        spec_id: WorkflowSpecId,
        data: serde_json::Value,
    ) -> WorkflowResult<CaseId> {
        // Admission gate: validate case data before execution
        self.admission_gate.admit(&data)?;

        // Verify workflow exists
        let _spec = self.get_workflow(spec_id).await?;

        // Create case
        let case = Case::new(spec_id, data);
        let case_id = case.id;

        // Store case
        let mut cases = self.cases.write().await;
        let case_clone = case.clone();
        cases.insert(case_id, case);

        // Persist to state store
        let store_arc = self.state_store.read().await;
        (*store_arc).save_case(case_id, &case_clone)?;

        Ok(case_id)
    }

    /// Start a case
    pub async fn start_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        let mut cases = self.cases.write().await;
        let case = cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        case.start()?;

        // Persist state
        let store_arc = self.state_store.read().await;
        (*store_arc).save_case(case_id, case)?;

        Ok(())
    }

    /// Execute a case (run workflow) with resource allocation, worklet support, and Fortune 5 SLO tracking
    pub async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        // Check promotion gate if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let gate_allowed = fortune5.check_promotion_gate().await?;
            if !gate_allowed {
                return Err(WorkflowError::Validation(
                    "Promotion gate blocked case execution".to_string(),
                ));
            }
        }

        let start_time = Instant::now();

        // Get case
        let mut cases = self.cases.write().await;
        let case = cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        // Start if not already started
        if case.state == CaseState::Created {
            case.start()?;
        }

        // Get workflow specification
        let specs = self.specs.read().await;
        let spec = specs.get(&case.spec_id).ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Workflow {} not found", case.spec_id))
        })?;
        let spec_clone = spec.clone();
        drop(specs);
        drop(cases);

        // Execute workflow with resource allocation and worklet support
        task::execute_workflow_tasks(self, case_id, &spec_clone).await?;

        // Record SLO metrics if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let elapsed_ns = start_time.elapsed().as_nanos() as u64;
            // Determine runtime class based on execution time
            let runtime_class = if elapsed_ns <= 2_000 {
                RuntimeClass::R1 // Hot path (≤2ns)
            } else if elapsed_ns <= 1_000_000 {
                RuntimeClass::W1 // Warm path (≤1ms)
            } else {
                RuntimeClass::C1 // Cold path (≤500ms)
            };
            fortune5.record_slo_metric(runtime_class, elapsed_ns).await;
        }

        Ok(())
    }

    /// Cancel a case
    pub async fn cancel_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        let mut cases = self.cases.write().await;
        let case = cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        case.cancel()?;

        // Persist state
        let store_arc = self.state_store.read().await;
        (*store_arc).save_case(case_id, case)?;

        Ok(())
    }

    /// Get case status
    pub async fn get_case(&self, case_id: CaseId) -> WorkflowResult<Case> {
        let cases = self.cases.read().await;
        cases
            .get(&case_id)
            .cloned()
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))
    }

    /// List all cases for a workflow specification
    pub async fn list_cases(&self, spec_id: WorkflowSpecId) -> WorkflowResult<Vec<CaseId>> {
        let store = self.state_store.read().await;
        (*store).list_cases(&spec_id)
    }
}
