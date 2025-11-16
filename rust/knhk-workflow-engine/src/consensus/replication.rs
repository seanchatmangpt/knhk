//! State machine replication for consensus
//!
//! This module implements replicated state machines that:
//! - Apply log entries to workflow state
//! - Create snapshots for log compaction
//! - Synchronize state across nodes
//!
//! # State Machine Interface
//!
//! State machines must implement deterministic operations:
//! - Same input → Same output (no randomness, wall clock, etc.)
//! - No side effects (idempotent)
//! - Serializable state

use super::*;
use crate::case::{Case, CaseId, CaseState};
use crate::state::StateManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State machine operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateMachineOp {
    /// Create a new workflow case
    CreateCase {
        case_id: CaseId,
        spec_id: String,
        data: serde_json::Value,
    },

    /// Update case state
    UpdateCaseState {
        case_id: CaseId,
        new_state: CaseState,
    },

    /// Apply policy change
    ApplyPolicy {
        policy_id: String,
        policy_data: Vec<u8>,
    },

    /// Deploy overlay application
    DeployOverlay {
        overlay_id: String,
        overlay_data: Vec<u8>,
    },

    /// Custom operation
    Custom {
        operation: String,
        data: Vec<u8>,
    },
}

/// Snapshot of replicated state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Last included log index
    pub last_included_index: LogIndex,

    /// Last included term
    pub last_included_term: Term,

    /// Snapshot data
    pub data: Vec<u8>,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(last_included_index: LogIndex, last_included_term: Term, data: Vec<u8>) -> Self {
        Self {
            last_included_index,
            last_included_term,
            data,
        }
    }
}

/// Replicated state machine
pub struct ReplicatedStateMachine {
    /// Current state
    state: Arc<RwLock<WorkflowState>>,

    /// Last applied log index
    last_applied: Arc<RwLock<LogIndex>>,

    /// State manager for persistence
    state_manager: Option<Arc<StateManager>>,
}

/// Workflow state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Active cases
    pub cases: HashMap<CaseId, Case>,

    /// Policies
    pub policies: HashMap<String, Vec<u8>>,

    /// Overlay applications
    pub overlays: HashMap<String, Vec<u8>>,

    /// Custom state
    pub custom: HashMap<String, Vec<u8>>,
}

impl ReplicatedStateMachine {
    /// Create a new replicated state machine
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(WorkflowState::default())),
            last_applied: Arc::new(RwLock::new(LogIndex::new(0))),
            state_manager: None,
        }
    }

    /// Create with state manager for persistence
    pub fn with_state_manager(state_manager: StateManager) -> Self {
        Self {
            state: Arc::new(RwLock::new(WorkflowState::default())),
            last_applied: Arc::new(RwLock::new(LogIndex::new(0))),
            state_manager: Some(Arc::new(state_manager)),
        }
    }

    /// Apply a log entry to the state machine
    pub async fn apply(&self, index: LogIndex, term: Term, data: &[u8]) -> ConsensusResult<()> {
        // Check if already applied
        let last_applied = *self.last_applied.read().await;
        if index <= last_applied {
            debug!(index = ?index, "Entry already applied");
            return Ok(());
        }

        // Deserialize operation
        let op: StateMachineOp = bincode::deserialize(data)
            .map_err(|e| ConsensusError::Serialization(e.to_string()))?;

        // Apply operation
        let mut state = self.state.write().await;

        match op {
            StateMachineOp::CreateCase { case_id, spec_id, data } => {
                let case = Case {
                    id: case_id.clone(),
                    spec_id,
                    state: CaseState::Created,
                    data,
                };

                state.cases.insert(case_id.clone(), case);

                info!(case_id = ?case_id, "Created case via consensus");
            }

            StateMachineOp::UpdateCaseState { case_id, new_state } => {
                if let Some(case) = state.cases.get_mut(&case_id) {
                    case.state = new_state.clone();

                    info!(case_id = ?case_id, new_state = ?new_state, "Updated case state via consensus");
                } else {
                    warn!(case_id = ?case_id, "Case not found for state update");
                }
            }

            StateMachineOp::ApplyPolicy { policy_id, policy_data } => {
                state.policies.insert(policy_id.clone(), policy_data);

                info!(policy_id, "Applied policy via consensus");
            }

            StateMachineOp::DeployOverlay { overlay_id, overlay_data } => {
                state.overlays.insert(overlay_id.clone(), overlay_data);

                info!(overlay_id, "Deployed overlay via consensus");
            }

            StateMachineOp::Custom { operation, data } => {
                state.custom.insert(operation.clone(), data);

                info!(operation, "Applied custom operation via consensus");
            }
        }

        // Update last applied
        *self.last_applied.write().await = index;

        Ok(())
    }

    /// Create a snapshot of current state
    pub async fn create_snapshot(&self, last_index: LogIndex, last_term: Term) -> ConsensusResult<Snapshot> {
        let state = self.state.read().await;

        // Serialize state
        let data = bincode::serialize(&*state)
            .map_err(|e| ConsensusError::Serialization(e.to_string()))?;

        info!(
            last_index = ?last_index,
            last_term = ?last_term,
            size_bytes = data.len(),
            "Created snapshot"
        );

        Ok(Snapshot::new(last_index, last_term, data))
    }

    /// Restore from snapshot
    pub async fn restore_snapshot(&self, snapshot: &Snapshot) -> ConsensusResult<()> {
        // Deserialize state
        let restored_state: WorkflowState = bincode::deserialize(&snapshot.data)
            .map_err(|e| ConsensusError::Serialization(e.to_string()))?;

        // Replace state
        *self.state.write().await = restored_state;
        *self.last_applied.write().await = snapshot.last_included_index;

        info!(
            last_index = ?snapshot.last_included_index,
            last_term = ?snapshot.last_included_term,
            "Restored from snapshot"
        );

        Ok(())
    }

    /// Get current state
    pub async fn get_state(&self) -> WorkflowState {
        self.state.read().await.clone()
    }

    /// Get last applied index
    pub async fn last_applied(&self) -> LogIndex {
        *self.last_applied.read().await
    }

    /// Get a case by ID
    pub async fn get_case(&self, case_id: &CaseId) -> Option<Case> {
        self.state.read().await.cases.get(case_id).cloned()
    }
}

impl Default for ReplicatedStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_machine_apply() {
        let sm = ReplicatedStateMachine::new();

        let op = StateMachineOp::CreateCase {
            case_id: CaseId::from("case-1".to_string()),
            spec_id: "spec-1".to_string(),
            data: serde_json::json!({"key": "value"}),
        };

        let data = bincode::serialize(&op).unwrap();

        sm.apply(LogIndex::new(1), Term::new(1), &data).await.unwrap();

        let state = sm.get_state().await;
        assert_eq!(state.cases.len(), 1);
        assert!(state.cases.contains_key(&CaseId::from("case-1".to_string())));
    }

    #[tokio::test]
    async fn test_state_machine_snapshot() {
        let sm = ReplicatedStateMachine::new();

        // Apply some operations
        let op1 = StateMachineOp::CreateCase {
            case_id: CaseId::from("case-1".to_string()),
            spec_id: "spec-1".to_string(),
            data: serde_json::json!({}),
        };

        let op2 = StateMachineOp::ApplyPolicy {
            policy_id: "policy-1".to_string(),
            policy_data: b"policy data".to_vec(),
        };

        sm.apply(LogIndex::new(1), Term::new(1), &bincode::serialize(&op1).unwrap()).await.unwrap();
        sm.apply(LogIndex::new(2), Term::new(1), &bincode::serialize(&op2).unwrap()).await.unwrap();

        // Create snapshot
        let snapshot = sm.create_snapshot(LogIndex::new(2), Term::new(1)).await.unwrap();
        assert_eq!(snapshot.last_included_index.inner(), 2);

        // Create new state machine and restore
        let sm2 = ReplicatedStateMachine::new();
        sm2.restore_snapshot(&snapshot).await.unwrap();

        let state = sm2.get_state().await;
        assert_eq!(state.cases.len(), 1);
        assert_eq!(state.policies.len(), 1);
    }

    #[tokio::test]
    async fn test_state_machine_idempotent() {
        let sm = ReplicatedStateMachine::new();

        let op = StateMachineOp::CreateCase {
            case_id: CaseId::from("case-1".to_string()),
            spec_id: "spec-1".to_string(),
            data: serde_json::json!({}),
        };

        let data = bincode::serialize(&op).unwrap();

        // Apply twice
        sm.apply(LogIndex::new(1), Term::new(1), &data).await.unwrap();
        sm.apply(LogIndex::new(1), Term::new(1), &data).await.unwrap(); // Should be skipped

        let state = sm.get_state().await;
        assert_eq!(state.cases.len(), 1);
    }
}
