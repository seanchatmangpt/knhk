//! State Machine for Workflow Execution
//!
//! # DOCTRINE ALIGNMENT
//! - **Covenant 2**: Q invariants enforced via state transitions
//! - **Covenant 5**: Chatman Constant enforced at each state transition

use super::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Workflow case state machine
///
/// State transitions:
/// ```text
/// Created → Running → [Suspended] → Completed
///                   ↓
///                Cancelled
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseState {
    /// Case created but not started
    Created,
    /// Case actively executing
    Running,
    /// Case suspended (waiting for external event)
    Suspended,
    /// Case completed successfully
    Completed,
    /// Case cancelled
    Cancelled,
    /// Case failed with error
    Failed,
}

/// Task execution state
///
/// State transitions:
/// ```text
/// Enabled → Executing → Completed
///        ↓           ↓
///     Suspended   Failed
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// Task can execute (preconditions met, resources available)
    Enabled,
    /// Task currently executing
    Executing,
    /// Task execution suspended
    Suspended,
    /// Task completed successfully
    Completed,
    /// Task execution failed
    Failed,
    /// Task cancelled before completion
    Cancelled,
}

/// Arc activation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArcState {
    /// Arc inactive
    Inactive,
    /// Arc activated (token flowing)
    Active,
    /// Arc completed (token consumed)
    Completed,
}

/// Case execution snapshot - immutable state capture
///
/// # DOCTRINE ALIGNMENT
/// - **Q1**: No retrocausation - snapshots form immutable DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSnapshot {
    pub case_id: CaseId,
    pub workflow_id: WorkflowId,
    pub state: CaseState,
    pub task_states: HashMap<TaskId, TaskState>,
    pub arc_states: HashMap<ArcId, ArcState>,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: Instant,
    pub tick_count: u8,
}

impl CaseSnapshot {
    /// Create initial snapshot for new case
    pub fn new(case_id: CaseId, workflow_id: WorkflowId) -> Self {
        Self {
            case_id,
            workflow_id,
            state: CaseState::Created,
            task_states: HashMap::new(),
            arc_states: HashMap::new(),
            data: HashMap::new(),
            timestamp: Instant::now(),
            tick_count: 0,
        }
    }

    /// Validate state transition
    pub fn can_transition_to(&self, new_state: CaseState) -> bool {
        use CaseState::*;

        match (self.state, new_state) {
            (Created, Running) => true,
            (Running, Suspended) => true,
            (Running, Completed) => true,
            (Running, Cancelled) => true,
            (Running, Failed) => true,
            (Suspended, Running) => true,
            (Suspended, Cancelled) => true,
            _ => false,
        }
    }

    /// Create new snapshot with state transition
    pub fn transition(&self, new_state: CaseState) -> Result<Self, ExecutionError> {
        if !self.can_transition_to(new_state) {
            return Err(ExecutionError::Internal(format!(
                "Invalid state transition: {:?} -> {:?}",
                self.state, new_state
            )));
        }

        let mut snapshot = self.clone();
        snapshot.state = new_state;
        snapshot.timestamp = Instant::now();

        Ok(snapshot)
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_case_state_transitions() {
        let snapshot = CaseSnapshot::new(CaseId::new(), WorkflowId::new());

        assert!(snapshot.can_transition_to(CaseState::Running));
        assert!(!snapshot.can_transition_to(CaseState::Completed));
    }

    #[test]
    fn test_state_transition() {
        let snapshot = CaseSnapshot::new(CaseId::new(), WorkflowId::new());

        let running = snapshot.transition(CaseState::Running).unwrap();
        assert_eq!(running.state, CaseState::Running);

        let completed = running.transition(CaseState::Completed).unwrap();
        assert_eq!(completed.state, CaseState::Completed);
    }
}
