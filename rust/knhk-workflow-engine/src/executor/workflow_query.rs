//! Workflow query methods
//!
//! Handles workflow specification retrieval and listing operations.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::{WorkflowSpec, WorkflowSpecId};

use super::engine::WorkflowEngine;

impl WorkflowEngine {
    /// Get workflow specification
    ///
    /// Tries in-memory cache first, then falls back to state store.
    pub async fn get_workflow(&self, spec_id: WorkflowSpecId) -> WorkflowResult<WorkflowSpec> {
        // Try in-memory first (lock-free DashMap access)
        if let Some(spec) = self.specs.get(&spec_id) {
            return Ok(spec.value().clone());
        }

        // Fallback to state store
        let store_arc = self.state_store.read().await;
        (*store_arc).load_spec(&spec_id)?.ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Workflow {} not found", spec_id))
        })
    }

    /// List all registered workflow specifications
    ///
    /// Returns all workflow specification IDs from in-memory cache.
    pub async fn list_workflows(&self) -> WorkflowResult<Vec<WorkflowSpecId>> {
        Ok(self.specs.iter().map(|entry| *entry.key()).collect())
    }
}
