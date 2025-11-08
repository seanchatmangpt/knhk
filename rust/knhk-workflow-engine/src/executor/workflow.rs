//! Workflow management methods

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use crate::validation::DeadlockDetector;

use super::WorkflowEngine;

impl WorkflowEngine {
    /// Register a workflow specification with deadlock validation and Fortune 5 checks
    pub async fn register_workflow(&self, spec: WorkflowSpec) -> WorkflowResult<()> {
        // Check promotion gate if Fortune 5 is enabled
        if let Some(ref fortune5) = self.fortune5_integration {
            let gate_allowed = fortune5.check_promotion_gate().await?;
            if !gate_allowed {
                return Err(WorkflowError::Validation(
                    "Promotion gate blocked workflow registration".to_string(),
                ));
            }
        }

        // Validate for deadlocks before registration
        let detector = DeadlockDetector;
        detector.validate(&spec)?;

        let mut specs = self.specs.write().await;
        let spec_clone = spec.clone();
        specs.insert(spec.id, spec);

        // Persist to state store
        let store_arc = self.state_store.read().await;
        store_arc.save_spec(&spec_clone)?;

        Ok(())
    }

    /// Get workflow specification
    pub async fn get_workflow(&self, spec_id: WorkflowSpecId) -> WorkflowResult<WorkflowSpec> {
        // Try in-memory first
        let specs = self.specs.read().await;
        if let Some(spec) = specs.get(&spec_id) {
            return Ok(spec.clone());
        }
        drop(specs);

        // Fallback to state store
        let store_arc = self.state_store.read().await;
        store_arc.load_spec(&spec_id)?.ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Workflow {} not found", spec_id))
        })
    }

    /// List all registered workflow specifications
    pub async fn list_workflows(&self) -> WorkflowResult<Vec<WorkflowSpecId>> {
        let specs = self.specs.read().await;
        Ok(specs.keys().cloned().collect())
    }
}
