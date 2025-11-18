//! Workflow state machine with immutable transitions
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: Q1 (No retrocausation - immutable state DAG)
//! - Covenant: Covenant 2 (Invariants are law)
//! - Validation: State transitions validated by Weaver telemetry

use crate::engine::messages::{TaskId, WorkflowId};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

/// Workflow execution state (immutable transitions only)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowState {
    /// Workflow created but not started
    Created,
    /// Workflow actively executing
    Executing,
    /// Workflow suspended (can be resumed)
    Suspended,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow cancelled
    Cancelled,
}

impl WorkflowState {
    /// Check if transition is valid (enforces Q1 - no retrocausation)
    pub fn can_transition_to(&self, next: WorkflowState) -> bool {
        use WorkflowState::*;

        match (self, next) {
            // From Created
            (Created, Executing) => true,
            (Created, Cancelled) => true,

            // From Executing
            (Executing, Suspended) => true,
            (Executing, Completed) => true,
            (Executing, Failed) => true,
            (Executing, Cancelled) => true,

            // From Suspended
            (Suspended, Executing) => true,
            (Suspended, Cancelled) => true,

            // Terminal states cannot transition
            (Completed, _) | (Failed, _) | (Cancelled, _) => false,

            // All other transitions invalid
            _ => false,
        }
    }

    /// Check if state is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            WorkflowState::Completed | WorkflowState::Failed | WorkflowState::Cancelled
        )
    }
}

/// Task execution state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskState {
    /// Task is ready but not started
    Ready,
    /// Task is currently executing
    Executing,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

impl TaskState {
    pub fn can_transition_to(&self, next: TaskState) -> bool {
        use TaskState::*;

        match (self, next) {
            (Ready, Executing) => true,
            (Executing, Completed) => true,
            (Executing, Failed) => true,
            (Executing, Cancelled) => true,
            (Failed, Executing) => true, // Allow retry
            (Completed, _) | (Cancelled, _) => false,
            _ => false,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskState::Completed | TaskState::Failed | TaskState::Cancelled
        )
    }
}

/// Workflow state record with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStateRecord {
    pub workflow_id: WorkflowId,
    pub state: WorkflowState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Task state record with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStateRecord {
    pub task_id: TaskId,
    pub workflow_id: WorkflowId,
    pub state: TaskState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retry_count: u32,
    pub metadata: serde_json::Value,
}

/// State store errors
#[derive(Debug, Error)]
pub enum StateError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: WorkflowState, to: WorkflowState },

    #[error("Workflow {0} not found")]
    WorkflowNotFound(WorkflowId),

    #[error("Task {0} not found")]
    TaskNotFound(TaskId),

    #[error("Task state transition invalid from {from:?} to {to:?}")]
    InvalidTaskTransition { from: TaskState, to: TaskState },
}

/// Concurrent state store with immutable transitions
pub struct StateStore {
    workflow_states: Arc<DashMap<WorkflowId, WorkflowStateRecord>>,
    task_states: Arc<DashMap<TaskId, TaskStateRecord>>,
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            workflow_states: Arc::new(DashMap::new()),
            task_states: Arc::new(DashMap::new()),
        }
    }

    /// Create new workflow state
    #[tracing::instrument(skip(self))]
    pub fn create_workflow(&self, workflow_id: WorkflowId, metadata: serde_json::Value) {
        let now = Utc::now();
        let record = WorkflowStateRecord {
            workflow_id,
            state: WorkflowState::Created,
            created_at: now,
            updated_at: now,
            metadata,
        };

        self.workflow_states.insert(workflow_id, record);
        info!(workflow_id = %workflow_id, "Workflow created");
    }

    /// Transition workflow to new state (enforces Q1)
    #[tracing::instrument(skip(self))]
    pub fn transition_workflow(
        &self,
        workflow_id: WorkflowId,
        new_state: WorkflowState,
    ) -> Result<(), StateError> {
        let mut entry = self
            .workflow_states
            .get_mut(&workflow_id)
            .ok_or(StateError::WorkflowNotFound(workflow_id))?;

        let old_state = entry.state;

        if !old_state.can_transition_to(new_state) {
            warn!(
                workflow_id = %workflow_id,
                from = ?old_state,
                to = ?new_state,
                "Invalid state transition blocked"
            );
            return Err(StateError::InvalidTransition {
                from: old_state,
                to: new_state,
            });
        }

        entry.state = new_state;
        entry.updated_at = Utc::now();

        info!(
            workflow_id = %workflow_id,
            from = ?old_state,
            to = ?new_state,
            "Workflow state transition"
        );

        Ok(())
    }

    /// Get current workflow state
    pub fn get_workflow_state(&self, workflow_id: WorkflowId) -> Option<WorkflowState> {
        self.workflow_states.get(&workflow_id).map(|r| r.state)
    }

    /// Create new task state
    #[tracing::instrument(skip(self))]
    pub fn create_task(
        &self,
        task_id: TaskId,
        workflow_id: WorkflowId,
        metadata: serde_json::Value,
    ) {
        let now = Utc::now();
        let record = TaskStateRecord {
            task_id,
            workflow_id,
            state: TaskState::Ready,
            created_at: now,
            updated_at: now,
            retry_count: 0,
            metadata,
        };

        self.task_states.insert(task_id, record);
        info!(task_id = %task_id, workflow_id = %workflow_id, "Task created");
    }

    /// Transition task to new state
    #[tracing::instrument(skip(self))]
    pub fn transition_task(
        &self,
        task_id: TaskId,
        new_state: TaskState,
    ) -> Result<(), StateError> {
        let mut entry = self
            .task_states
            .get_mut(&task_id)
            .ok_or(StateError::TaskNotFound(task_id))?;

        let old_state = entry.state;

        if !old_state.can_transition_to(new_state) {
            return Err(StateError::InvalidTaskTransition {
                from: old_state,
                to: new_state,
            });
        }

        entry.state = new_state;
        entry.updated_at = Utc::now();

        if new_state == TaskState::Executing && old_state == TaskState::Failed {
            entry.retry_count += 1;
        }

        info!(
            task_id = %task_id,
            from = ?old_state,
            to = ?new_state,
            retry = entry.retry_count,
            "Task state transition"
        );

        Ok(())
    }

    /// Get current task state
    pub fn get_task_state(&self, task_id: TaskId) -> Option<TaskState> {
        self.task_states.get(&task_id).map(|r| r.state)
    }

    /// Get all tasks for a workflow
    pub fn get_workflow_tasks(&self, workflow_id: WorkflowId) -> Vec<TaskStateRecord> {
        self.task_states
            .iter()
            .filter(|entry| entry.workflow_id == workflow_id)
            .map(|entry| entry.value().clone())
            .collect()
    }
}

impl Default for StateStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_workflow_transitions() {
        assert!(WorkflowState::Created.can_transition_to(WorkflowState::Executing));
        assert!(WorkflowState::Executing.can_transition_to(WorkflowState::Completed));
        assert!(WorkflowState::Executing.can_transition_to(WorkflowState::Suspended));
        assert!(WorkflowState::Suspended.can_transition_to(WorkflowState::Executing));
    }

    #[test]
    fn test_invalid_workflow_transitions() {
        // Cannot go backwards
        assert!(!WorkflowState::Executing.can_transition_to(WorkflowState::Created));

        // Cannot leave terminal states
        assert!(!WorkflowState::Completed.can_transition_to(WorkflowState::Executing));
        assert!(!WorkflowState::Failed.can_transition_to(WorkflowState::Executing));
    }

    #[test]
    fn test_state_store_transitions() {
        let store = StateStore::new();
        let wf_id = WorkflowId::new();

        store.create_workflow(wf_id, serde_json::json!({}));
        assert_eq!(store.get_workflow_state(wf_id), Some(WorkflowState::Created));

        store.transition_workflow(wf_id, WorkflowState::Executing).unwrap();
        assert_eq!(store.get_workflow_state(wf_id), Some(WorkflowState::Executing));

        // Should fail - invalid transition
        let result = store.transition_workflow(wf_id, WorkflowState::Created);
        assert!(result.is_err());
    }

    #[test]
    fn test_task_retry_count() {
        let store = StateStore::new();
        let task_id = TaskId::new();
        let wf_id = WorkflowId::new();

        store.create_task(task_id, wf_id, serde_json::json!({}));
        store.transition_task(task_id, TaskState::Executing).unwrap();
        store.transition_task(task_id, TaskState::Failed).unwrap();

        // Retry
        store.transition_task(task_id, TaskState::Executing).unwrap();

        let task = store.task_states.get(&task_id).unwrap();
        assert_eq!(task.retry_count, 1);
    }
}
