//! Workflow execution engine
//!
//! The WorkflowExecutor manages the execution of entire workflows.

use crate::core::{ExecutionContext, Workflow};
use crate::core::context::ExecutionStatus;
use crate::engine::TokenManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Workflow executor
///
/// Manages execution of YAWL workflows.
/// Uses Arc<Mutex<>> for minimal shared state locking (Covenant 5).
#[derive(Debug)]
pub struct WorkflowExecutor {
    /// Registry of workflows (workflow_id -> Workflow)
    registry: Arc<Mutex<HashMap<String, Workflow>>>,

    /// State store (instance_id -> ExecutionContext)
    state_store: Arc<Mutex<HashMap<String, ExecutionContext>>>,

    /// Token manager
    token_manager: Arc<TokenManager>,
}

impl WorkflowExecutor {
    /// Create a new workflow executor
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(HashMap::new())),
            state_store: Arc::new(Mutex::new(HashMap::new())),
            token_manager: Arc::new(TokenManager::new()),
        }
    }

    /// Register a workflow
    ///
    /// # Errors
    /// Returns error if workflow registration fails
    pub fn register_workflow(&self, workflow: Workflow) -> crate::Result<()> {
        // Validate workflow structure
        workflow.validate()?;

        let mut registry = self.registry.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock registry: {}", e))
        })?;

        registry.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    /// Start workflow execution
    ///
    /// # Errors
    /// Returns error if workflow execution fails to start
    #[tracing::instrument(skip(self))]
    pub fn start_workflow(
        &self,
        workflow_id: &str,
        instance_id: &str,
    ) -> crate::Result<ExecutionContext> {
        tracing::info!("Starting workflow: {}", workflow_id);

        // Get workflow from registry
        let registry = self.registry.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock registry: {}", e))
        })?;

        let _workflow = registry.get(workflow_id).ok_or_else(|| {
            crate::Error::InvalidWorkflow(format!("Workflow not found: {}", workflow_id))
        })?;

        // Create execution context
        let mut context = ExecutionContext::builder()
            .workflow_id(workflow_id)
            .instance_id(instance_id)
            .build();

        context.mark_started();

        // Store context
        let mut state_store = self.state_store.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock state store: {}", e))
        })?;

        state_store.insert(instance_id.to_string(), context.clone());

        tracing::info!("Workflow started: {} (instance: {})", workflow_id, instance_id);

        Ok(context)
    }

    /// Get execution context for a workflow instance
    ///
    /// # Errors
    /// Returns error if instance not found
    pub fn get_context(&self, instance_id: &str) -> crate::Result<ExecutionContext> {
        let state_store = self.state_store.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock state store: {}", e))
        })?;

        state_store.get(instance_id).cloned().ok_or_else(|| {
            crate::Error::Other(anyhow::anyhow!(
                "Workflow instance not found: {}",
                instance_id
            ))
        })
    }

    /// Update execution context
    ///
    /// # Errors
    /// Returns error if update fails
    pub fn update_context(&self, context: ExecutionContext) -> crate::Result<()> {
        let mut state_store = self.state_store.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock state store: {}", e))
        })?;

        state_store.insert(context.instance_id.clone(), context);
        Ok(())
    }

    /// Complete workflow execution
    ///
    /// # Errors
    /// Returns error if completion fails
    #[tracing::instrument(skip(self))]
    pub fn complete_workflow(&self, instance_id: &str) -> crate::Result<()> {
        tracing::info!("Completing workflow instance: {}", instance_id);

        let mut state_store = self.state_store.lock().map_err(|e| {
            crate::Error::Other(anyhow::anyhow!("Failed to lock state store: {}", e))
        })?;

        if let Some(context) = state_store.get_mut(instance_id) {
            context.mark_completed();
            tracing::info!("Workflow completed: {}", instance_id);
            Ok(())
        } else {
            Err(crate::Error::Other(anyhow::anyhow!(
                "Workflow instance not found: {}",
                instance_id
            )))
        }
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Task, TaskType};

    #[test]
    fn test_workflow_registration() {
        let executor = WorkflowExecutor::new();

        let task = Task::builder()
            .id("task1")
            .name("Test Task")
            .task_type(TaskType::Atomic)
            .build();

        let workflow = Workflow::builder()
            .id("wf1")
            .name("Test Workflow")
            .version("1.0.0")
            .add_task(task)
            .build();

        assert!(executor.register_workflow(workflow).is_ok());
    }

    #[test]
    fn test_workflow_execution() {
        let executor = WorkflowExecutor::new();

        let task = Task::builder()
            .id("task1")
            .name("Test Task")
            .task_type(TaskType::Atomic)
            .build();

        let workflow = Workflow::builder()
            .id("wf1")
            .name("Test Workflow")
            .add_task(task)
            .build();

        executor.register_workflow(workflow).unwrap();

        let context = executor.start_workflow("wf1", "inst1").unwrap();
        assert_eq!(context.workflow_id, "wf1");
        assert_eq!(context.instance_id, "inst1");
        assert_eq!(context.status, ExecutionStatus::Running);
    }

    #[test]
    fn test_workflow_completion() {
        let executor = WorkflowExecutor::new();

        let task = Task::builder()
            .id("task1")
            .name("Test Task")
            .task_type(TaskType::Atomic)
            .build();

        let workflow = Workflow::builder()
            .id("wf1")
            .name("Test Workflow")
            .add_task(task)
            .build();

        executor.register_workflow(workflow).unwrap();
        executor.start_workflow("wf1", "inst1").unwrap();
        executor.complete_workflow("inst1").unwrap();

        let context = executor.get_context("inst1").unwrap();
        assert_eq!(context.status, ExecutionStatus::Completed);
    }
}
