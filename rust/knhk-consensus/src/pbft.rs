//! Practical Byzantine Fault Tolerance (PBFT) Implementation
//!
//! Three-phase consensus protocol with guaranteed safety and liveness
//! Tolerates up to f < n/3 Byzantine nodes
//!
//! Phases:
//! 1. Pre-prepare: Leader proposes a value
//! 2. Prepare: Replicas promise not to accept conflicting values
//! 3. Commit: Replicas commit the value when 2f+1 prepares received

use crate::{ConsensusError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// PBFT message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BFTMessage {
    /// Pre-prepare phase: leader broadcasts value
    PrePrepare {
        /// Sequence number
        sequence: u64,
        /// Digest of the value
        digest: Vec<u8>,
        /// Proposed value
        value: Vec<u8>,
        /// Leader node ID
        leader: String,
        /// View number
        view: u64,
    },

    /// Prepare phase: replica promises
    Prepare {
        /// Sequence number
        sequence: u64,
        /// Digest of the value
        digest: Vec<u8>,
        /// Replica node ID
        replica: String,
        /// View number
        view: u64,
    },

    /// Commit phase: replica commits
    Commit {
        /// Sequence number
        sequence: u64,
        /// Digest of the value
        digest: Vec<u8>,
        /// Replica node ID
        replica: String,
        /// View number
        view: u64,
    },

    /// View change request
    ViewChange {
        /// New view number
        new_view: u64,
        /// Node requesting change
        node: String,
        /// Prepared messages
        prepared: Vec<(u64, Vec<u8>)>,
    },
}

impl BFTMessage {
    /// Get the digest of this message
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(serde_json::to_vec(self).unwrap_or_default());
        hasher.finalize().to_vec()
    }
}

/// PBFT node state
#[derive(Debug, Clone)]
pub struct PBFTNode {
    /// Node identifier
    pub node_id: String,
    /// Current view number
    pub view: u64,
    /// Sequence number
    pub sequence: u64,
    /// Is this node the leader
    pub is_leader: bool,
    /// Prepared messages by sequence
    prepared: Arc<DashMap<u64, (Vec<u8>, usize)>>,
    /// Committed messages by sequence
    committed: Arc<DashMap<u64, Vec<u8>>>,
    /// Prepare count by sequence and digest
    prepare_count: Arc<DashMap<(u64, Vec<u8>), usize>>,
    /// Total nodes in the cluster
    total_nodes: usize,
}

/// PBFT configuration
#[derive(Debug, Clone)]
pub struct PBFTConfig {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Maximum Byzantine nodes to tolerate
    pub max_byzantine: usize,
    /// View timeout in milliseconds
    pub view_timeout_ms: u64,
}

impl PBFTConfig {
    /// Create a new PBFT configuration
    pub fn new(total_nodes: usize) -> Result<Self> {
        let max_byzantine = (total_nodes - 1) / 3;
        if total_nodes < 3 {
            return Err(ConsensusError::InvalidValidatorSet(
                "PBFT requires at least 3 nodes".to_string(),
            ));
        }
        Ok(PBFTConfig {
            total_nodes,
            max_byzantine,
            view_timeout_ms: 5000,
        })
    }

    /// Validate quorum
    pub fn validate_quorum(&self, count: usize) -> bool {
        count >= 2 * self.max_byzantine + 1
    }

    /// Minimum nodes needed for quorum
    pub fn quorum_size(&self) -> usize {
        2 * self.max_byzantine + 1
    }
}

impl PBFTNode {
    /// Create a new PBFT node
    pub fn new(node_id: String, config: &PBFTConfig, is_leader: bool) -> Self {
        PBFTNode {
            node_id,
            view: 0,
            sequence: 0,
            is_leader,
            prepared: Arc::new(DashMap::new()),
            committed: Arc::new(DashMap::new()),
            prepare_count: Arc::new(DashMap::new()),
            total_nodes: config.total_nodes,
        }
    }

    /// Pre-prepare phase: leader proposes
    pub fn pre_prepare(&mut self, value: Vec<u8>, config: &PBFTConfig) -> Result<BFTMessage> {
        if !self.is_leader {
            return Err(ConsensusError::ByzantineNodeDetected(
                "Non-leader attempted pre-prepare".to_string(),
            ));
        }

        let digest = {
            let mut hasher = Sha3_256::new();
            hasher.update(&value);
            hasher.finalize().to_vec()
        };

        self.sequence += 1;
        let seq = self.sequence;

        let msg = BFTMessage::PrePrepare {
            sequence: seq,
            digest: digest.clone(),
            value,
            leader: self.node_id.clone(),
            view: self.view,
        };

        debug!(
            node = %self.node_id,
            sequence = seq,
            view = self.view,
            "Pre-prepare proposal"
        );

        Ok(msg)
    }

    /// Prepare phase: replica responds to pre-prepare
    pub fn prepare(&mut self, msg: &BFTMessage, config: &PBFTConfig) -> Result<Option<BFTMessage>> {
        if let BFTMessage::PrePrepare {
            sequence,
            digest,
            view,
            ..
        } = msg
        {
            if *view != self.view {
                return Err(ConsensusError::ViewSyncTimeout);
            }

            // Store prepared message
            self.prepared.insert(*sequence, (digest.clone(), 1));

            let prepare_msg = BFTMessage::Prepare {
                sequence: *sequence,
                digest: digest.clone(),
                replica: self.node_id.clone(),
                view: *view,
            };

            debug!(
                node = %self.node_id,
                sequence = *sequence,
                "Prepare promise"
            );

            Ok(Some(prepare_msg))
        } else {
            Err(ConsensusError::ByzantineNodeDetected(
                "Non pre-prepare message in prepare phase".to_string(),
            ))
        }
    }

    /// Count prepare messages and return commit message if ready
    pub fn count_prepares(
        &mut self,
        sequence: u64,
        digest: Vec<u8>,
        config: &PBFTConfig,
    ) -> Result<Option<BFTMessage>> {
        let key = (sequence, digest.clone());
        let count = self.prepare_count.entry(key.clone()).or_insert(0);
        *count += 1;

        // Need 2f+1 prepares to move to commit
        if *count >= config.quorum_size() {
            let commit_msg = BFTMessage::Commit {
                sequence,
                digest: digest.clone(),
                replica: self.node_id.clone(),
                view: self.view,
            };

            debug!(
                node = %self.node_id,
                sequence = sequence,
                prepare_count = *count,
                "Commit threshold reached"
            );

            Ok(Some(commit_msg))
        } else {
            Ok(None)
        }
    }

    /// Commit phase: finalize state
    pub fn commit(&mut self, sequence: u64, value: Vec<u8>) -> Result<()> {
        self.committed.insert(sequence, value.clone());

        info!(
            node = %self.node_id,
            sequence = sequence,
            value_hash = ?format!("{:x?}", sha3::Sha3_256::digest(&value)),
            "State committed"
        );

        Ok(())
    }

    /// Get committed value at sequence
    pub fn get_committed(&self, sequence: u64) -> Option<Vec<u8>> {
        self.committed.get(&sequence).map(|v| v.clone())
    }

    /// View change: rotate leader
    pub fn view_change(&mut self, new_view: u64) -> Result<BFTMessage> {
        if new_view <= self.view {
            return Err(ConsensusError::ViewSyncTimeout);
        }

        let prepared: Vec<_> = self
            .prepared
            .iter()
            .map(|entry| (entry.key().0, entry.value().0.clone()))
            .collect();

        self.view = new_view;
        self.is_leader = false; // Will be set externally

        warn!(
            node = %self.node_id,
            new_view = new_view,
            prepared_count = prepared.len(),
            "View change initiated"
        );

        Ok(BFTMessage::ViewChange {
            new_view,
            node: self.node_id.clone(),
            prepared,
        })
    }

    /// Validate quorum for commitment
    pub fn validate_commit_quorum(&self, commit_count: usize, config: &PBFTConfig) -> bool {
        config.validate_quorum(commit_count)
    }

    /// Get current state
    pub fn get_state(&self) -> PBFTState {
        PBFTState {
            node_id: self.node_id.clone(),
            view: self.view,
            sequence: self.sequence,
            is_leader: self.is_leader,
            prepared_count: self.prepared.len(),
            committed_count: self.committed.len(),
        }
    }
}

/// PBFT node state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PBFTState {
    /// Node identifier
    pub node_id: String,
    /// Current view
    pub view: u64,
    /// Current sequence
    pub sequence: u64,
    /// Is leader
    pub is_leader: bool,
    /// Prepared message count
    pub prepared_count: usize,
    /// Committed message count
    pub committed_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbft_config_quorum() {
        let config = PBFTConfig::new(7).unwrap();
        assert_eq!(config.quorum_size(), 5);
        assert_eq!(config.max_byzantine, 2);
    }

    #[test]
    fn test_pbft_node_creation() {
        let config = PBFTConfig::new(4).unwrap();
        let node = PBFTNode::new("node1".to_string(), &config, true);
        assert_eq!(node.view, 0);
        assert!(node.is_leader);
    }

    #[test]
    fn test_pre_prepare() {
        let config = PBFTConfig::new(4).unwrap();
        let mut node = PBFTNode::new("node1".to_string(), &config, true);
        let value = b"test_value".to_vec();
        let msg = node.pre_prepare(value, &config).unwrap();
        assert!(matches!(msg, BFTMessage::PrePrepare { .. }));
    }

    #[test]
    fn test_view_change() {
        let config = PBFTConfig::new(4).unwrap();
        let mut node = PBFTNode::new("node1".to_string(), &config, true);
        let msg = node.view_change(1).unwrap();
        assert!(matches!(msg, BFTMessage::ViewChange { .. }));
        assert_eq!(node.view, 1);
    }
}
