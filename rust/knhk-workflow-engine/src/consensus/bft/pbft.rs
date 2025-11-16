//! PBFT (Practical Byzantine Fault Tolerance) implementation
//!
//! PBFT is a classic 3-phase BFT protocol that tolerates up to f Byzantine failures in 3f+1 nodes.
//!
//! # Protocol Phases
//!
//! 1. **PRE-PREPARE**: Primary broadcasts proposal to all replicas
//! 2. **PREPARE**: Replicas broadcast PREPARE messages after validating proposal
//! 3. **COMMIT**: Replicas broadcast COMMIT after receiving 2f PREPARE messages
//! 4. **REPLY**: Replicas execute request after receiving 2f+1 COMMIT messages
//!
//! # View Changes
//!
//! If primary fails or times out, replicas initiate view change to elect new primary.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// PBFT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbftConfig {
    /// Node ID
    pub node_id: NodeId,

    /// Total number of nodes (3f+1)
    pub total_nodes: usize,

    /// Byzantine threshold (f)
    pub byzantine_threshold: usize,

    /// View change timeout
    pub view_change_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,
}

impl PbftConfig {
    /// Create config for 3f+1 nodes
    pub fn new(node_id: NodeId, total_nodes: usize) -> ConsensusResult<Self> {
        if total_nodes < 4 {
            return Err(ConsensusError::Internal(
                "PBFT requires at least 4 nodes (3f+1 where f=1)".to_string(),
            ));
        }

        if (total_nodes - 1) % 3 != 0 {
            return Err(ConsensusError::Internal(
                "Total nodes must be 3f+1".to_string(),
            ));
        }

        let byzantine_threshold = (total_nodes - 1) / 3;

        Ok(Self {
            node_id,
            total_nodes,
            byzantine_threshold,
            view_change_timeout: Duration::from_millis(100),
            request_timeout: Duration::from_secs(1),
        })
    }

    /// Get quorum size (2f+1)
    pub fn quorum_size(&self) -> usize {
        2 * self.byzantine_threshold + 1
    }

    /// Get prepare quorum size (2f)
    pub fn prepare_quorum(&self) -> usize {
        2 * self.byzantine_threshold
    }
}

/// PBFT message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PbftMessage {
    /// PRE-PREPARE from primary
    PrePrepare {
        view: ViewNumber,
        sequence: SequenceNumber,
        digest: [u8; 32],
        request: Vec<u8>,
    },

    /// PREPARE from replica
    Prepare {
        view: ViewNumber,
        sequence: SequenceNumber,
        digest: [u8; 32],
        replica_id: NodeId,
    },

    /// COMMIT from replica
    Commit {
        view: ViewNumber,
        sequence: SequenceNumber,
        digest: [u8; 32],
        replica_id: NodeId,
    },

    /// VIEW-CHANGE to elect new primary
    ViewChange {
        new_view: ViewNumber,
        replica_id: NodeId,
    },
}

/// PBFT replica state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PbftPhase {
    /// Waiting for PRE-PREPARE
    Idle,

    /// Received PRE-PREPARE, sent PREPARE
    Prepared,

    /// Received 2f PREPARE messages, sent COMMIT
    PreCommitted,

    /// Received 2f+1 COMMIT messages, ready to execute
    Committed,
}

/// PBFT node implementation
pub struct PbftNode {
    /// Configuration
    config: PbftConfig,

    /// Current view
    view: ViewNumber,

    /// Current phase
    phase: PbftPhase,

    /// Sequence number
    sequence: SequenceNumber,

    /// PREPARE messages received
    prepare_messages: HashMap<SequenceNumber, HashSet<NodeId>>,

    /// COMMIT messages received
    commit_messages: HashMap<SequenceNumber, HashSet<NodeId>>,

    /// Crypto provider
    crypto: Arc<CryptoProvider>,
}

impl PbftNode {
    /// Create a new PBFT node
    pub fn new(config: PbftConfig, crypto: CryptoProvider) -> Self {
        Self {
            config,
            view: ViewNumber::new(0),
            phase: PbftPhase::Idle,
            sequence: SequenceNumber::new(0),
            prepare_messages: HashMap::new(),
            commit_messages: HashMap::new(),
            crypto: Arc::new(crypto),
        }
    }

    /// Check if we are the primary for current view
    pub fn is_primary(&self) -> bool {
        self.view.primary(self.config.total_nodes) == self.config.node_id
    }

    /// Create PRE-PREPARE message (primary only)
    pub fn create_pre_prepare(&mut self, request: Vec<u8>) -> ConsensusResult<PbftMessage> {
        if !self.is_primary() {
            return Err(ConsensusError::Internal("Only primary can create PRE-PREPARE".to_string()));
        }

        self.sequence.increment();
        let digest = self.compute_digest(&request);

        Ok(PbftMessage::PrePrepare {
            view: self.view,
            sequence: self.sequence,
            digest,
            request,
        })
    }

    /// Handle PRE-PREPARE message
    pub fn handle_pre_prepare(
        &mut self,
        view: ViewNumber,
        sequence: SequenceNumber,
        digest: [u8; 32],
        request: &[u8],
    ) -> ConsensusResult<Option<PbftMessage>> {
        // Validate view
        if view != self.view {
            return Ok(None);
        }

        // Verify digest
        let computed_digest = self.compute_digest(request);
        if computed_digest != digest {
            warn!("Invalid digest in PRE-PREPARE");
            return Ok(None);
        }

        // Move to PREPARED phase and send PREPARE
        self.phase = PbftPhase::Prepared;

        Ok(Some(PbftMessage::Prepare {
            view: self.view,
            sequence,
            digest,
            replica_id: self.config.node_id,
        }))
    }

    /// Handle PREPARE message
    pub fn handle_prepare(
        &mut self,
        view: ViewNumber,
        sequence: SequenceNumber,
        digest: [u8; 32],
        replica_id: NodeId,
    ) -> ConsensusResult<Option<PbftMessage>> {
        // Validate view
        if view != self.view {
            return Ok(None);
        }

        // Add to prepare messages
        self.prepare_messages
            .entry(sequence)
            .or_insert_with(HashSet::new)
            .insert(replica_id);

        // Check if we have 2f PREPARE messages
        let prepare_count = self.prepare_messages.get(&sequence).map(|s| s.len()).unwrap_or(0);

        if prepare_count >= self.config.prepare_quorum() && self.phase == PbftPhase::Prepared {
            // Move to PRE-COMMITTED phase and send COMMIT
            self.phase = PbftPhase::PreCommitted;

            return Ok(Some(PbftMessage::Commit {
                view: self.view,
                sequence,
                digest,
                replica_id: self.config.node_id,
            }));
        }

        Ok(None)
    }

    /// Handle COMMIT message
    pub fn handle_commit(
        &mut self,
        view: ViewNumber,
        sequence: SequenceNumber,
        digest: [u8; 32],
        replica_id: NodeId,
    ) -> ConsensusResult<bool> {
        // Validate view
        if view != self.view {
            return Ok(false);
        }

        // Add to commit messages
        self.commit_messages
            .entry(sequence)
            .or_insert_with(HashSet::new)
            .insert(replica_id);

        // Check if we have 2f+1 COMMIT messages
        let commit_count = self.commit_messages.get(&sequence).map(|s| s.len()).unwrap_or(0);

        if commit_count >= self.config.quorum_size() && self.phase == PbftPhase::PreCommitted {
            // Move to COMMITTED phase - ready to execute
            self.phase = PbftPhase::Committed;
            return Ok(true);
        }

        Ok(false)
    }

    /// Compute digest of request
    fn compute_digest(&self, request: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(request);
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbft_config() {
        let config = PbftConfig::new(NodeId::new(1), 4).unwrap();
        assert_eq!(config.byzantine_threshold, 1);
        assert_eq!(config.quorum_size(), 3); // 2f+1 = 3
        assert_eq!(config.prepare_quorum(), 2); // 2f = 2
    }

    #[test]
    fn test_pbft_config_invalid() {
        // Less than 4 nodes
        assert!(PbftConfig::new(NodeId::new(1), 3).is_err());

        // Not 3f+1
        assert!(PbftConfig::new(NodeId::new(1), 5).is_err());
    }

    #[test]
    fn test_pbft_primary() {
        let config = PbftConfig::new(NodeId::new(0), 4).unwrap();
        let crypto = CryptoProvider::new();
        let node = PbftNode::new(config, crypto);

        // View 0, primary is node 0
        assert!(node.is_primary());
    }
}
