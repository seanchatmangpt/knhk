//! Workflow execution engine

use crate::case::{Case, CaseId, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternId, PatternRegistry,
};
use crate::state::StateStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Workflow execution engine
pub struct WorkflowEngine {
    /// Pattern registry
    pattern_registry: Arc<PatternRegistry>,
    /// State store
    state_store: Arc<RwLock<StateStore>>,
    /// Registered workflow specifications
    specs: Arc<RwLock<HashMap<WorkflowSpecId, WorkflowSpec>>>,
    /// Active cases
    cases: Arc<RwLock<HashMap<CaseId, Case>>>,
}

impl WorkflowEngine {
    /// Create a new workflow engine with all 43 patterns registered
    pub fn new(state_store: StateStore) -> Self {
        let mut registry = PatternRegistry::new();
        crate::patterns::register_all_patterns(&mut registry);

        Self {
            pattern_registry: Arc::new(registry),
            state_store: Arc::new(RwLock::new(state_store)),
            specs: Arc::new(RwLock::new(HashMap::new())),
            cases: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a workflow specification
    pub async fn register_workflow(&self, spec: WorkflowSpec) -> WorkflowResult<()> {
        let mut specs = self.specs.write().await;
        specs.insert(spec.id, spec);
        Ok(())
    }

    /// Get workflow specification
    pub async fn get_workflow(&self, spec_id: WorkflowSpecId) -> WorkflowResult<WorkflowSpec> {
        let specs = self.specs.read().await;
        specs.get(&spec_id).cloned().ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Workflow {} not found", spec_id))
        })
    }

    /// Create a new case
    pub async fn create_case(
        &self,
        spec_id: WorkflowSpecId,
        _data: serde_json::Value,
    ) -> WorkflowResult<CaseId> {
        // Verify workflow exists
        let _spec = self.get_workflow(spec_id).await?;

        // Create case
        let case = Case::new(spec_id, _data);
        let case_id = case.id;

        // Store case
        let mut cases = self.cases.write().await;
        let case_clone = case.clone();
        cases.insert(case_id, case);

        // Persist to state store
        let store = self.state_store.write().await;
        store.save_case(case_id, &case_clone)?;

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
        let store = self.state_store.write().await;
        store.save_case(case_id, case)?;

        Ok(())
    }

    /// Execute a case (run workflow)
    pub async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<()> {
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
        let _spec = specs.get(&case.spec_id).ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Workflow {} not found", case.spec_id))
        })?;

        drop(specs);
        drop(cases);

        // Execute workflow (simplified - full implementation would traverse workflow graph)
        // FUTURE: Implement full workflow execution with pattern matching

        Ok(())
    }

    /// Cancel a case
    pub async fn cancel_case(&self, case_id: CaseId) -> WorkflowResult<()> {
        let mut cases = self.cases.write().await;
        let case = cases
            .get_mut(&case_id)
            .ok_or_else(|| WorkflowError::CaseNotFound(case_id.to_string()))?;

        // Persist state
        let store = self.state_store.write().await;
        store.save_case(case_id, case)?;

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

    /// Execute a pattern
    pub async fn execute_pattern(
        &self,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionResult> {
        self.pattern_registry
            .execute(&pattern_id, &context)
            .ok_or_else(|| {
                WorkflowError::InvalidSpecification(format!("Pattern {} not found", pattern_id))
            })
    }

    /// Get pattern registry
    pub fn pattern_registry(&self) -> &PatternRegistry {
        &self.pattern_registry
    }
}
