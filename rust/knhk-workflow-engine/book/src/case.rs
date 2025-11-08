//! Workflow case management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::WorkflowResult;

/// Unique identifier for a workflow case
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CaseId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl CaseId {
    /// Generate a new case ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse from string
    pub fn parse_str(s: &str) -> WorkflowResult<Self> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|e| crate::error::WorkflowError::Parse(format!("Invalid case ID: {}", e)))
    }
}

impl Default for CaseId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Case execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaseState {
    /// Case is created but not yet started
    Created,
    /// Case is currently executing
    Running,
    /// Case completed successfully
    Completed,
    /// Case was cancelled
    Cancelled,
    /// Case failed with error
    Failed,
    /// Case is suspended (waiting for external event)
    Suspended,
}

/// Workflow case (instance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    /// Unique case identifier
    pub id: CaseId,
    /// Workflow specification ID this case belongs to
    pub spec_id: crate::parser::WorkflowSpecId,
    /// Current state
    pub state: CaseState,
    /// Case creation timestamp
    pub created_at: DateTime<Utc>,
    /// Case start timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Case completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Case data (input/output variables)
    pub data: serde_json::Value,
    /// Current task execution state
    pub task_states: std::collections::HashMap<String, TaskState>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Task execution state within a case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {
    /// Task is ready to execute
    Ready,
    /// Task is currently executing
    Executing,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed(String),
    /// Task was cancelled
    Cancelled,
    /// Task is waiting for condition
    Waiting,
}

impl Case {
    /// Create a new case
    pub fn new(spec_id: crate::parser::WorkflowSpecId, data: serde_json::Value) -> Self {
        Self {
            id: CaseId::new(),
            spec_id,
            state: CaseState::Created,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            data,
            task_states: std::collections::HashMap::new(),
            error: None,
        }
    }

    /// Start the case
    pub fn start(&mut self) -> WorkflowResult<()> {
        match self.state {
            CaseState::Created => {
                self.state = CaseState::Running;
                self.started_at = Some(Utc::now());
                Ok(())
            }
            _ => Err(crate::error::WorkflowError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "Running".to_string(),
            }),
        }
    }

    /// Complete the case
    pub fn complete(&mut self) -> WorkflowResult<()> {
        match self.state {
            CaseState::Running | CaseState::Suspended => {
                self.state = CaseState::Completed;
                self.completed_at = Some(Utc::now());
                Ok(())
            }
            _ => Err(crate::error::WorkflowError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "Completed".to_string(),
            }),
        }
    }

    /// Cancel the case
    pub fn cancel(&mut self) -> WorkflowResult<()> {
        match self.state {
            CaseState::Created | CaseState::Running | CaseState::Suspended => {
                self.state = CaseState::Cancelled;
                self.completed_at = Some(Utc::now());
                Ok(())
            }
            _ => Err(crate::error::WorkflowError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "Cancelled".to_string(),
            }),
        }
    }

    /// Fail the case
    pub fn fail(&mut self, error: String) -> WorkflowResult<()> {
        match self.state {
            CaseState::Running | CaseState::Suspended => {
                self.state = CaseState::Failed;
                self.error = Some(error);
                self.completed_at = Some(Utc::now());
                Ok(())
            }
            _ => Err(crate::error::WorkflowError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "Failed".to_string(),
            }),
        }
    }

    /// Suspend the case
    pub fn suspend(&mut self) -> WorkflowResult<()> {
        match self.state {
            CaseState::Running => {
                self.state = CaseState::Suspended;
                Ok(())
            }
            _ => Err(crate::error::WorkflowError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "Suspended".to_string(),
            }),
        }
    }

    /// Resume the case
    pub fn resume(&mut self) -> WorkflowResult<()> {
        match self.state {
            CaseState::Suspended => {
                self.state = CaseState::Running;
                Ok(())
            }
            _ => Err(crate::error::WorkflowError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "Running".to_string(),
            }),
        }
    }
}
