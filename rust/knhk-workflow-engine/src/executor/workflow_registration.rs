//! Workflow registration methods
//!
//! Handles workflow specification registration with validation and persistence.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use crate::validation::DeadlockDetector;

use super::engine::WorkflowEngine;

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

        // Load source turtle into RDF store if available
        if let Some(ref turtle) = spec.source_turtle {
            self.load_spec_rdf(turtle)
                .await
                .map_err(|e| WorkflowError::Internal(format!("Failed to load spec RDF: {}", e)))?;
        }

        let spec_clone = spec.clone();
        self.specs.insert(spec.id, spec);

        // Persist to state store
        let store_arc = self.state_store.read().await;
        (*store_arc).save_spec(&spec_clone)?;

        Ok(())
    }
}
