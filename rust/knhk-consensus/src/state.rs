//! State Machine Replication
//!
//! Deterministic command log with sequential consistency
//! Provides snapshotting, catchup, and rollback for replica synchronization

use crate::{ConsensusError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// State snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Snapshot version/height
    pub version: u64,
    /// State data (compressed)
    pub data: Vec<u8>,
    /// State hash for verification
    pub hash: Vec<u8>,
    /// Timestamp in milliseconds
    pub timestamp_ms: u64,
}

impl StateSnapshot {
    /// Verify snapshot integrity
    pub fn verify(&self) -> bool {
        let computed_hash = Self::compute_hash(&self.data);
        computed_hash == self.hash
    }

    /// Compute hash of data
    pub fn compute_hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}

/// Command log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    /// Sequence number
    pub sequence: u64,
    /// Command bytes
    pub command: Vec<u8>,
    /// View when committed
    pub view: u64,
    /// Execution result
    pub result: Option<Vec<u8>>,
    /// Is committed
    pub committed: bool,
}

/// Command log for state machine replication
#[derive(Debug, Clone)]
pub struct CommandLog {
    /// All entries
    entries: Arc<DashMap<u64, CommandEntry>>,
    /// Log in order
    order: Arc<parking_lot::Mutex<VecDeque<u64>>>,
    /// Current applied index
    applied_index: Arc<parking_lot::Mutex<u64>>,
}

impl CommandLog {
    /// Create new command log
    pub fn new() -> Self {
        CommandLog {
            entries: Arc::new(DashMap::new()),
            order: Arc::new(parking_lot::Mutex::new(VecDeque::new())),
            applied_index: Arc::new(parking_lot::Mutex::new(0)),
        }
    }

    /// Append command to log
    pub fn append(&self, command: Vec<u8>, view: u64) -> Result<u64> {
        let mut order = self.order.lock();
        let sequence = order.len() as u64;

        let entry = CommandEntry {
            sequence,
            command,
            view,
            result: None,
            committed: false,
        };

        self.entries.insert(sequence, entry);
        order.push_back(sequence);

        debug!(sequence = sequence, view = view, "Command appended");

        Ok(sequence)
    }

    /// Mark command as committed
    pub fn commit(&self, sequence: u64) -> Result<()> {
        if let Some(mut entry) = self.entries.get_mut(&sequence) {
            entry.committed = true;
            debug!(sequence = sequence, "Command committed");
            Ok(())
        } else {
            Err(ConsensusError::CommandLogError(format!(
                "Sequence {} not found",
                sequence
            )))
        }
    }

    /// Set execution result
    pub fn set_result(&self, sequence: u64, result: Vec<u8>) -> Result<()> {
        if let Some(mut entry) = self.entries.get_mut(&sequence) {
            entry.result = Some(result);
            Ok(())
        } else {
            Err(ConsensusError::CommandLogError(format!(
                "Sequence {} not found",
                sequence
            )))
        }
    }

    /// Get command at sequence
    pub fn get(&self, sequence: u64) -> Option<CommandEntry> {
        self.entries.get(&sequence).map(|e| e.clone())
    }

    /// Get all committed commands since last apply
    pub fn get_committed_since(&self, from: u64) -> Vec<CommandEntry> {
        let mut result = Vec::new();
        for seq in from..self.entries.len() as u64 {
            if let Some(entry) = self.entries.get(&seq) {
                if entry.committed {
                    result.push(entry.clone());
                }
            }
        }
        result
    }

    /// Apply commands up to index
    pub fn apply_up_to(&self, index: u64) -> Result<()> {
        let mut applied = self.applied_index.lock();

        if index < *applied {
            return Err(ConsensusError::CommandLogError(
                "Cannot apply backwards".to_string(),
            ));
        }

        *applied = index;
        debug!(applied_index = index, "Commands applied");

        Ok(())
    }

    /// Get last applied index
    pub fn last_applied(&self) -> u64 {
        *self.applied_index.lock()
    }

    /// Get log size
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for CommandLog {
    fn default() -> Self {
        Self::new()
    }
}

/// State machine replicator
#[derive(Debug, Clone)]
pub struct StateMachineReplicator {
    /// Current state
    state: Arc<parking_lot::Mutex<Vec<u8>>>,
    /// Command log
    log: CommandLog,
    /// Snapshots
    snapshots: Arc<DashMap<u64, StateSnapshot>>,
    /// Snapshot interval
    snapshot_interval: u64,
    /// Last snapshot version
    last_snapshot_version: Arc<parking_lot::Mutex<u64>>,
}

impl StateMachineReplicator {
    /// Create new state machine
    pub fn new(snapshot_interval: u64) -> Self {
        StateMachineReplicator {
            state: Arc::new(parking_lot::Mutex::new(Vec::new())),
            log: CommandLog::new(),
            snapshots: Arc::new(DashMap::new()),
            snapshot_interval,
            last_snapshot_version: Arc::new(parking_lot::Mutex::new(0)),
        }
    }

    /// Execute command
    pub fn execute(&self, command: Vec<u8>, sequence: u64) -> Result<()> {
        // Apply command to state machine
        let mut state = self.state.lock();

        // Simple concatenation (in practice, would be actual state transition)
        let mut new_state = state.clone();
        new_state.extend_from_slice(&command);

        *state = new_state;

        self.log.set_result(sequence, command)?;

        debug!(sequence = sequence, "Command executed");

        Ok(())
    }

    /// Take snapshot at version
    pub fn snapshot(&self, version: u64) -> Result<StateSnapshot> {
        let state = self.state.lock();
        let data = state.clone();
        let hash = StateSnapshot::compute_hash(&data);

        let snapshot = StateSnapshot {
            version,
            data,
            hash,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        // Verify snapshot
        if !snapshot.verify() {
            return Err(ConsensusError::StateMismatch {
                expected: "verified snapshot".to_string(),
                actual: "unverified snapshot".to_string(),
            });
        }

        self.snapshots.insert(version, snapshot.clone());
        *self.last_snapshot_version.lock() = version;

        info!(version = version, "Snapshot created");

        Ok(snapshot)
    }

    /// Restore from snapshot
    pub fn restore_from_snapshot(&self, snapshot: StateSnapshot) -> Result<()> {
        if !snapshot.verify() {
            return Err(ConsensusError::StateMismatch {
                expected: hex::encode(&snapshot.hash),
                actual: hex::encode(StateSnapshot::compute_hash(&snapshot.data)),
            });
        }

        let mut state = self.state.lock();
        *state = snapshot.data.clone();

        info!(version = snapshot.version, "State restored from snapshot");

        Ok(())
    }

    /// Get current state
    pub fn get_state(&self) -> Vec<u8> {
        self.state.lock().clone()
    }

    /// Get state hash
    pub fn get_state_hash(&self) -> Vec<u8> {
        let state = self.state.lock();
        let mut hasher = Sha256::new();
        hasher.update(&*state);
        hasher.finalize().to_vec()
    }

    /// Check state matches expected
    pub fn verify_state(&self, expected_hash: Vec<u8>) -> bool {
        self.get_state_hash() == expected_hash
    }

    /// Rollback to snapshot
    pub fn rollback(&self, version: u64) -> Result<()> {
        if let Some(snapshot) = self.snapshots.get(&version) {
            self.restore_from_snapshot(snapshot.clone())?;
            warn!(version = version, "Rolled back to snapshot");
            Ok(())
        } else {
            Err(ConsensusError::CommandLogError(format!(
                "Snapshot version {} not found",
                version
            )))
        }
    }

    /// Get command log
    pub fn log(&self) -> &CommandLog {
        &self.log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_snapshot() {
        let data = b"test_state".to_vec();
        let hash = StateSnapshot::compute_hash(&data);
        let snapshot = StateSnapshot {
            version: 1,
            data: data.clone(),
            hash,
            timestamp_ms: 1000,
        };
        assert!(snapshot.verify());
    }

    #[test]
    fn test_command_log() {
        let log = CommandLog::new();
        let seq = log.append(b"command1".to_vec(), 0).unwrap();
        assert_eq!(seq, 0);
        log.commit(seq).unwrap();
        assert!(log.get(seq).unwrap().committed);
    }

    #[test]
    fn test_state_machine() {
        let sm = StateMachineReplicator::new(10);
        sm.execute(b"cmd1".to_vec(), 0).unwrap();
        let snapshot = sm.snapshot(1).unwrap();
        assert_eq!(snapshot.version, 1);
    }
}
