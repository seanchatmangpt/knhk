//! Case management methods

use crate::case::{Case, CaseId, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::fortune5::RuntimeClass;
use crate::parser::WorkflowSpecId;
use std::time::Instant;

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
        let case = Case::new(spec_id, data.clone());
        let case_id = case.id;

        // Create case RDF store
        self.create_case_rdf_store(case_id, &data)
            .await
            .map_err(|e| {
                WorkflowError::Internal(format!("Failed to create case RDF store: {}", e))
            })?;

        // Store case (lock-free DashMap access)
        let case_clone = case.clone();
        self.cases.insert(case_id, case);

        // Persist to state store
        let store_arc = self.state_store.read().await;
        (*store_arc).save_case(case_id, &case_clone)?;

        // Save to state manager for event sourcing
        self.state_manager.save_case(&case_clone).await?;

        // Log CaseCreated event
        let event = crate::state::manager::StateEvent::CaseCreated {
            case_id,
            spec_id,
            timestamp: chrono::Utc::now(),
        };
        {
            // Add to in-memory event log
            // Note: StateManager doesn't have a public method to add events directly,
            // so we'll persist it to store and it will be loaded by get_case_history
            let store_arc = self.state_store.read().await;
            (*store_arc).save_case_history_event(&case_id, &event)?;
        }

        Ok(case_id)
    }

    /// Start a case
    pub async fn start_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        let mut case_ref = self
            .cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        case_ref.value_mut().start()?;
        let case_clone = case_ref.value().clone();
        drop(case_ref);

        // Persist state
        let store_arc = self.state_store.read().await;
        (*store_arc).save_case(case_id, &case_clone)?;

        // Save to state manager for event sourcing (will emit CaseStateChanged event)
        self.state_manager.save_case(&case_clone).await?;

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

        // Get case (DashMap is thread-safe, no lock needed)
        let mut case_ref = self
            .cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        // Start if not already started
        if case_ref.value().state == CaseState::Created {
            case_ref.value_mut().start()?;
        }

        // Get workflow specification (DashMap is thread-safe)
        let spec_id = case_ref.value().spec_id;
        let spec = self.specs.get(&spec_id).ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Workflow {} not found", spec_id))
        })?;
        let spec_clone = spec.value().clone();
        drop(case_ref);

        // Execute workflow from start to end condition
        // Note: execute_workflow doesn't actually recurse to execute_case, so this is safe
        super::workflow_execution::execute_workflow(self, case_id, &spec_clone).await?;

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
        let mut case_guard = self
            .cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        case_guard.value_mut().cancel()?;

        // Persist state (clone before dropping guard)
        let case_clone = case_guard.value().clone();
        drop(case_guard);
        let store_arc = self.state_store.read().await;
        (*store_arc).save_case(case_id, &case_clone)?;

        // Save to state manager for event sourcing (will emit CaseStateChanged event)
        self.state_manager.save_case(&case_clone).await?;

        Ok(())
    }

    /// Get case status
    pub async fn get_case(&self, case_id: CaseId) -> WorkflowResult<Case> {
        self.cases
            .get(&case_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))
    }

    /// List all cases for a workflow specification
    pub async fn list_cases(&self, spec_id: WorkflowSpecId) -> WorkflowResult<Vec<CaseId>> {
        let store = self.state_store.read().await;
        (*store).list_cases(&spec_id)
    }
}
