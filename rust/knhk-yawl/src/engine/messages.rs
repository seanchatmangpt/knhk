//! Actor message types for workflow execution
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: MAPE-K (message-driven autonomic execution)
//! - Covenant: Covenant 3 (Feedback loops at machine speed)
//! - Validation: Message routing < 8 ticks (Chatman constant)

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for workflows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

impl WorkflowId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for tokens (data flow)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenId(pub Uuid);

impl TokenId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TokenId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TokenId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Messages exchanged between workflow actors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowMessage {
    /// Start workflow execution
    StartWorkflow {
        workflow_id: WorkflowId,
        initial_data: serde_json::Value,
    },

    /// Execute a specific task
    ExecuteTask {
        task_id: TaskId,
        workflow_id: WorkflowId,
        input_tokens: Vec<TokenId>,
    },

    /// Task completed successfully
    TaskCompleted {
        task_id: TaskId,
        workflow_id: WorkflowId,
        output_data: serde_json::Value,
    },

    /// Task failed with error
    TaskFailed {
        task_id: TaskId,
        workflow_id: WorkflowId,
        error: String,
        retry_count: u32,
    },

    /// Propagate token to next task
    PropagateToken {
        token_id: TokenId,
        from_task: TaskId,
        to_task: TaskId,
        data: serde_json::Value,
    },

    /// Suspend workflow execution
    Suspend {
        workflow_id: WorkflowId,
        reason: String,
    },

    /// Resume workflow execution
    Resume {
        workflow_id: WorkflowId,
    },

    /// Cancel workflow execution
    Cancel {
        workflow_id: WorkflowId,
        reason: String,
    },

    /// Workflow completed
    WorkflowCompleted {
        workflow_id: WorkflowId,
        final_data: serde_json::Value,
    },

    /// Actor health check
    HealthCheck,

    /// Actor shutdown
    Shutdown,
}

impl WorkflowMessage {
    /// Get workflow ID from message if present
    pub fn workflow_id(&self) -> Option<WorkflowId> {
        match self {
            Self::StartWorkflow { workflow_id, .. }
            | Self::ExecuteTask { workflow_id, .. }
            | Self::TaskCompleted { workflow_id, .. }
            | Self::TaskFailed { workflow_id, .. }
            | Self::Suspend { workflow_id, .. }
            | Self::Resume { workflow_id, .. }
            | Self::Cancel { workflow_id, .. }
            | Self::WorkflowCompleted { workflow_id, .. } => Some(*workflow_id),
            _ => None,
        }
    }

    /// Get task ID from message if present
    pub fn task_id(&self) -> Option<TaskId> {
        match self {
            Self::ExecuteTask { task_id, .. }
            | Self::TaskCompleted { task_id, .. }
            | Self::TaskFailed { task_id, .. } => Some(*task_id),
            Self::PropagateToken { from_task, .. } => Some(*from_task),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_creation() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_workflow_message_extraction() {
        let wf_id = WorkflowId::new();
        let task_id = TaskId::new();

        let msg = WorkflowMessage::ExecuteTask {
            task_id,
            workflow_id: wf_id,
            input_tokens: vec![],
        };

        assert_eq!(msg.workflow_id(), Some(wf_id));
        assert_eq!(msg.task_id(), Some(task_id));
    }
}
