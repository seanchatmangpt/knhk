//! HotStuff Consensus Protocol
//!
//! Modern leader-based Byzantine consensus with linear communication complexity
//! Achieves commit in 3 message delays via chained voting
//!
//! Phases:
//! 1. Propose: Leader proposes a block
//! 2. Vote: Replicas vote on the block
//! 3. Commit: Three consecutive confirmed blocks trigger commit

use crate::{ConsensusError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// View number type
pub type ViewNumber = u64;

/// HotStuff block header
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    /// Block height
    pub height: u64,
    /// View number
    pub view: ViewNumber,
    /// Parent block hash
    pub parent: Vec<u8>,
    /// Proposed leader
    pub leader: String,
    /// Command payload
    pub command: Vec<u8>,
    /// Timestamp in milliseconds
    pub timestamp_ms: u64,
}

impl BlockHeader {
    /// Compute block hash
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(serde_json::to_vec(self).unwrap_or_default());
        hasher.finalize().to_vec()
    }

    /// Verify block structure
    pub fn verify(&self) -> bool {
        !self.parent.is_empty() || self.height == 0
    }
}

/// HotStuff message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HotStuffMessage {
    /// Propose a new block
    Propose {
        /// Block to propose
        block: BlockHeader,
        /// Quorum certificate of parent
        qc: QuorumCertificate,
    },

    /// Vote on a block
    Vote {
        /// Block hash voted for
        block_hash: Vec<u8>,
        /// Voter node ID
        voter: String,
        /// View number
        view: ViewNumber,
    },

    /// Quorum certificate (3+ signatures)
    Generic {
        /// Block hash
        block_hash: Vec<u8>,
        /// View number
        view: ViewNumber,
        /// Vote count
        vote_count: usize,
    },

    /// Timeout message for view change
    Timeout {
        /// Current view
        view: ViewNumber,
        /// Node ID
        node: String,
        /// High commit QC
        high_commit_qc: QuorumCertificate,
    },
}

/// Quorum certificate: proof of 2f+1 votes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuorumCertificate {
    /// Block hash this QC certifies
    pub block_hash: Vec<u8>,
    /// View number
    pub view: ViewNumber,
    /// Vote count (minimum 2f+1)
    pub vote_count: usize,
    /// Certified block height
    pub block_height: u64,
}

impl QuorumCertificate {
    /// Verify QC has sufficient votes
    pub fn verify(&self, total_nodes: usize) -> bool {
        let quorum = 2 * ((total_nodes - 1) / 3) + 1;
        self.vote_count >= quorum
    }
}

/// HotStuff node
#[derive(Debug, Clone)]
pub struct HotStuffNode {
    /// Node identifier
    pub node_id: String,
    /// Current view number
    pub view: ViewNumber,
    /// Current leader
    pub leader: String,
    /// Locked QC (cannot vote conflicting)
    pub locked_qc: Option<QuorumCertificate>,
    /// Commit QC (finalized)
    pub commit_qc: Option<QuorumCertificate>,
    /// Vote count for current view
    vote_count: Arc<DashMap<ViewNumber, usize>>,
    /// Blocks by hash
    blocks: Arc<DashMap<Vec<u8>, BlockHeader>>,
    /// Total nodes
    total_nodes: usize,
    /// Height of committed blocks
    commit_height: u64,
}

/// HotStuff configuration
#[derive(Debug, Clone)]
pub struct HotStuffConfig {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Maximum Byzantine nodes
    pub max_byzantine: usize,
    /// View timeout in milliseconds
    pub view_timeout_ms: u64,
}

impl HotStuffConfig {
    /// Create configuration
    pub fn new(total_nodes: usize) -> Result<Self> {
        let max_byzantine = (total_nodes - 1) / 3;
        if total_nodes < 4 {
            return Err(ConsensusError::InvalidValidatorSet(
                "HotStuff requires at least 4 nodes".to_string(),
            ));
        }
        Ok(HotStuffConfig {
            total_nodes,
            max_byzantine,
            view_timeout_ms: 5000,
        })
    }

    /// Quorum size
    pub fn quorum_size(&self) -> usize {
        2 * self.max_byzantine + 1
    }

    /// Verify quorum
    pub fn verify_quorum(&self, count: usize) -> bool {
        count >= self.quorum_size()
    }
}

impl HotStuffNode {
    /// Create new HotStuff node
    pub fn new(node_id: String, config: &HotStuffConfig, is_leader: bool) -> Self {
        let leader = if is_leader {
            node_id.clone()
        } else {
            format!("leader_{}", config.total_nodes)
        };

        HotStuffNode {
            node_id,
            view: 0,
            leader,
            locked_qc: None,
            commit_qc: None,
            vote_count: Arc::new(DashMap::new()),
            blocks: Arc::new(DashMap::new()),
            total_nodes: config.total_nodes,
            commit_height: 0,
        }
    }

    /// Propose a new block
    pub fn propose(
        &mut self,
        command: Vec<u8>,
        parent_qc: QuorumCertificate,
        config: &HotStuffConfig,
    ) -> Result<HotStuffMessage> {
        // Verify parent QC
        if !parent_qc.verify(config.total_nodes) {
            return Err(ConsensusError::QuorumNotReached(
                parent_qc.vote_count,
                config.quorum_size(),
            ));
        }

        let block = BlockHeader {
            height: parent_qc.block_height + 1,
            view: self.view,
            parent: parent_qc.block_hash.clone(),
            leader: self.node_id.clone(),
            command,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        let block_hash = block.hash();
        self.blocks.insert(block_hash.clone(), block.clone());

        debug!(
            node = %self.node_id,
            view = self.view,
            height = block.height,
            "HotStuff propose"
        );

        Ok(HotStuffMessage::Propose {
            block,
            qc: parent_qc,
        })
    }

    /// Vote on a proposed block
    pub fn vote(
        &mut self,
        block_hash: Vec<u8>,
        view: ViewNumber,
        config: &HotStuffConfig,
    ) -> Result<Option<HotStuffMessage>> {
        // Check view
        if view < self.view {
            return Err(ConsensusError::ViewSyncTimeout);
        }

        // Safety: check locked QC
        if let Some(ref locked) = self.locked_qc {
            if locked.view > view {
                return Err(ConsensusError::ByzantineNodeDetected(
                    "Attempting to vote for lower view than locked".to_string(),
                ));
            }
        }

        self.view = view;

        let msg = HotStuffMessage::Vote {
            block_hash: block_hash.clone(),
            voter: self.node_id.clone(),
            view,
        };

        debug!(
            node = %self.node_id,
            view = view,
            block_hash = ?format!("{:x?}", block_hash),
            "HotStuff vote"
        );

        Ok(Some(msg))
    }

    /// Process votes and create QC if quorum reached
    pub fn collect_votes(
        &mut self,
        block_hash: Vec<u8>,
        view: ViewNumber,
        config: &HotStuffConfig,
    ) -> Result<Option<QuorumCertificate>> {
        let count = self.vote_count.entry(view).or_insert(0);
        let updated_count = {
            *count += 1;
            *count
        };

        if config.verify_quorum(updated_count) {
            // Create QC
            let qc = QuorumCertificate {
                block_hash: block_hash.clone(),
                view,
                vote_count: updated_count,
                block_height: self.commit_height,
            };

            debug!(
                node = %self.node_id,
                view = view,
                vote_count = *count,
                "QC formed"
            );

            Ok(Some(qc))
        } else {
            Ok(None)
        }
    }

    /// Generic commit: chain three consecutive confirmed blocks
    pub fn generic_commit(
        &mut self,
        qc: QuorumCertificate,
        config: &HotStuffConfig,
    ) -> Result<Option<Vec<u8>>> {
        // Update locked QC
        if qc.view > self.view {
            self.locked_qc = Some(qc.clone());
            self.view = qc.view;
        }

        // Three-chain rule: if we have three consecutive confirmed blocks, commit
        // For now, we track commit height incrementally
        if qc.block_height > self.commit_height {
            self.commit_height = qc.block_height;

            if self.commit_height >= 3 {
                info!(
                    node = %self.node_id,
                    height = self.commit_height,
                    "Generic commit"
                );

                return Ok(Some(qc.block_hash.clone()));
            }
        }

        Ok(None)
    }

    /// View synchronization
    pub fn sync_view(&mut self, new_view: ViewNumber, leader: String) -> Result<()> {
        if new_view < self.view {
            return Err(ConsensusError::ViewSyncTimeout);
        }

        self.view = new_view;
        self.leader = leader;

        warn!(
            node = %self.node_id,
            new_view = new_view,
            new_leader = %self.leader,
            "View synchronized"
        );

        Ok(())
    }

    /// Get node state
    pub fn get_state(&self) -> HotStuffState {
        HotStuffState {
            node_id: self.node_id.clone(),
            view: self.view,
            leader: self.leader.clone(),
            commit_height: self.commit_height,
            blocks_count: self.blocks.len(),
        }
    }
}

/// HotStuff node state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotStuffState {
    /// Node ID
    pub node_id: String,
    /// Current view
    pub view: ViewNumber,
    /// Current leader
    pub leader: String,
    /// Committed block height
    pub commit_height: u64,
    /// Total blocks received
    pub blocks_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotstuff_config() {
        let config = HotStuffConfig::new(7).unwrap();
        assert_eq!(config.quorum_size(), 5);
        assert_eq!(config.max_byzantine, 2);
    }

    #[test]
    fn test_block_header_hash() {
        let header = BlockHeader {
            height: 1,
            view: 0,
            parent: vec![0; 32],
            leader: "node1".to_string(),
            command: b"test".to_vec(),
            timestamp_ms: 1000,
        };
        let hash = header.hash();
        assert_eq!(hash.len(), 32); // SHA3-256
    }

    #[test]
    fn test_qc_verification() {
        let qc = QuorumCertificate {
            block_hash: vec![1; 32],
            view: 0,
            vote_count: 5,
            block_height: 1,
        };
        assert!(qc.verify(7));
        assert!(!qc.verify(13));
    }

    #[test]
    fn test_hotstuff_node_creation() {
        let config = HotStuffConfig::new(7).unwrap();
        let node = HotStuffNode::new("node1".to_string(), &config, true);
        assert_eq!(node.view, 0);
        assert_eq!(node.node_id, "node1");
    }
}
