//! Workflow execution engine

use crate::case::{Case, CaseId, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternId, PatternRegistry,
};
use crate::resource::{AllocationRequest, ResourceAllocator};
use crate::state::StateStore;
use crate::validation::DeadlockDetector;
use crate::worklets::{WorkletExecutor, WorkletRepository};
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
    /// Resource allocator
    resource_allocator: Arc<ResourceAllocator>,
    /// Worklet repository
    worklet_repository: Arc<WorkletRepository>,
    /// Worklet executor
    worklet_executor: Arc<WorkletExecutor>,
}

impl WorkflowEngine {
    /// Create a new workflow engine with all 43 patterns registered
    pub fn new(state_store: StateStore) -> Self {
        let mut registry = PatternRegistry::new();
        crate::patterns::register_all_patterns(&mut registry);

        let resource_allocator = Arc::new(ResourceAllocator::new());
        let worklet_repository = Arc::new(WorkletRepository::new());
        let worklet_executor = Arc::new(WorkletExecutor::new(worklet_repository.clone()));

        Self {
            pattern_registry: Arc::new(registry),
            state_store: Arc::new(RwLock::new(state_store)),
            specs: Arc::new(RwLock::new(HashMap::new())),
            cases: Arc::new(RwLock::new(HashMap::new())),
            resource_allocator,
            worklet_repository,
            worklet_executor,
        }
    }

    /// Register a workflow specification with deadlock validation
    pub async fn register_workflow(&self, spec: WorkflowSpec) -> WorkflowResult<()> {
        // Validate for deadlocks before registration
        let detector = DeadlockDetector;
        detector.validate(&spec)?;

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

    /// Execute a case (run workflow) with resource allocation and worklet support
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
        let spec = specs.get(&case.spec_id).ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Workflow {} not found", case.spec_id))
        })?;
        let spec_clone = spec.clone();
        drop(specs);
        drop(cases);

        // Execute workflow with resource allocation and worklet support
        self.execute_workflow_tasks(case_id, &spec_clone).await?;

        Ok(())
    }

    /// Execute workflow tasks with resource allocation
    async fn execute_workflow_tasks(
        &self,
        case_id: CaseId,
        spec: &WorkflowSpec,
    ) -> WorkflowResult<()> {
        // Start from start condition
        if let Some(ref start_condition_id) = spec.start_condition {
            if let Some(start_condition) = spec.conditions.get(start_condition_id) {
                // Execute tasks from start condition
                for task_id in &start_condition.outgoing_flows {
                    if let Some(task) = spec.tasks.get(task_id) {
                        self.execute_task_with_allocation(case_id, spec.id, task)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a task with resource allocation
    async fn execute_task_with_allocation(
        &self,
        case_id: CaseId,
        spec_id: WorkflowSpecId,
        task: &crate::parser::Task,
    ) -> WorkflowResult<()> {
        // Allocate resources if allocation policy is specified
        if let Some(ref policy) = task.allocation_policy {
            let request = AllocationRequest {
                task_id: task.id.clone(),
                spec_id,
                required_roles: task.required_roles.clone(),
                required_capabilities: task.required_capabilities.clone(),
                policy: *policy,
                priority: task.priority.unwrap_or(100) as u8,
            };

            match self.resource_allocator.allocate(request).await {
                Ok(allocation) => {
                    // Resources allocated - proceed with task execution
                    // In production, would track allocation and release after execution
                    for resource_id in &allocation.resource_ids {
                        self.resource_allocator
                            .update_workload(*resource_id, 1)
                            .await?;
                    }
                }
                Err(e) => {
                    // Resource allocation failed - try worklet exception handling
                    if let Some(worklet_id) = task.exception_worklet {
                        let context = PatternExecutionContext {
                            case_id,
                            workflow_id: spec_id,
                            variables: HashMap::new(),
                        };
                        if let Some(result) = self
                            .worklet_executor
                            .handle_exception("resource_unavailable", context)
                            .await?
                        {
                            if !result.success {
                                return Err(e);
                            }
                        } else {
                            return Err(e);
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Execute task (simplified - full implementation would execute actual task logic)
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

    /// Get resource allocator
    pub fn resource_allocator(&self) -> &ResourceAllocator {
        &self.resource_allocator
    }

    /// Get worklet repository
    pub fn worklet_repository(&self) -> &WorkletRepository {
        &self.worklet_repository
    }

    /// Get worklet executor
    pub fn worklet_executor(&self) -> &WorkletExecutor {
        &self.worklet_executor
    }
}
