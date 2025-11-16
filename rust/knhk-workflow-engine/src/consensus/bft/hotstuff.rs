//! HotStuff BFT implementation
//!
//! HotStuff is a modern BFT protocol with:
//! - Linear communication complexity (O(n) instead of O(n²))
//! - Simpler view change protocol
//! - Better performance than PBFT
//!
//! # Protocol Phases
//!
//! HotStuff uses a 3-chain commit rule:
//! 1. **PREPARE**: Leader proposes, replicas vote
//! 2. **PRE-COMMIT**: Leader aggregates votes, broadcasts
//! 3. **COMMIT**: Leader aggregates votes, broadcasts
//! 4. **DECIDE**: Leader aggregates votes, commit decision
//!
//! A block is committed when it has 3 consecutive certified blocks in its chain.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// HotStuff configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotStuffConfig {
    /// Node ID
    pub node_id: NodeId,

    /// Total number of nodes (3f+1)
    pub total_nodes: usize,

    /// Byzantine threshold (f)
    pub byzantine_threshold: usize,

    /// Timeout for each phase
    pub phase_timeout: Duration,
}

impl HotStuffConfig {
    /// Create config for 3f+1 nodes
    pub fn new(node_id: NodeId, total_nodes: usize) -> ConsensusResult<Self> {
        if total_nodes < 4 {
            return Err(ConsensusError::Internal(
                "HotStuff requires at least 4 nodes (3f+1 where f=1)".to_string(),
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
            phase_timeout: Duration::from_millis(50),
        })
    }

    /// Get quorum size (2f+1)
    pub fn quorum_size(&self) -> usize {
        2 * self.byzantine_threshold + 1
    }
}

/// HotStuff block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block view
    pub view: ViewNumber,

    /// Parent block hash
    pub parent_hash: [u8; 32],

    /// Block data
    pub data: Vec<u8>,

    /// Quorum certificate from previous round
    pub qc: Option<QuorumCertificate>,
}

impl Block {
    /// Compute block hash
    pub fn hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.view.inner().to_le_bytes());
        hasher.update(&self.parent_hash);
        hasher.update(&self.data);
        hasher.finalize().into()
    }
}

/// Quorum certificate (proof of 2f+1 votes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumCertificate {
    /// View number
    pub view: ViewNumber,

    /// Block hash being certified
    pub block_hash: [u8; 32],

    /// Aggregated signatures
    pub signatures: HashMap<NodeId, Signature>,
}

impl QuorumCertificate {
    /// Check if QC has quorum
    pub fn has_quorum(&self, quorum_size: usize) -> bool {
        self.signatures.len() >= quorum_size
    }
}

/// HotStuff vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// View number
    pub view: ViewNumber,

    /// Block hash being voted for
    pub block_hash: [u8; 32],

    /// Voter node ID
    pub voter_id: NodeId,

    /// Signature
    pub signature: Signature,
}

/// HotStuff phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotStuffPhase {
    /// PREPARE phase
    Prepare,

    /// PRE-COMMIT phase
    PreCommit,

    /// COMMIT phase
    Commit,

    /// DECIDE phase
    Decide,
}

/// HotStuff node implementation
pub struct HotStuffNode {
    /// Configuration
    config: HotStuffConfig,

    /// Current view
    view: ViewNumber,

    /// Current phase
    phase: HotStuffPhase,

    /// Proposed blocks by view
    blocks: HashMap<ViewNumber, Block>,

    /// Votes received by view
    votes: HashMap<ViewNumber, HashMap<NodeId, Vote>>,

    /// Highest QC seen
    high_qc: Option<QuorumCertificate>,

    /// Crypto provider
    crypto: Arc<CryptoProvider>,
}

impl HotStuffNode {
    /// Create a new HotStuff node
    pub fn new(config: HotStuffConfig, crypto: CryptoProvider) -> Self {
        Self {
            config,
            view: ViewNumber::new(0),
            phase: HotStuffPhase::Prepare,
            blocks: HashMap::new(),
            votes: HashMap::new(),
            high_qc: None,
            crypto: Arc::new(crypto),
        }
    }

    /// Check if we are the leader for current view
    pub fn is_leader(&self) -> bool {
        self.view.primary(self.config.total_nodes) == self.config.node_id
    }

    /// Create a new block proposal (leader only)
    pub fn create_block(&mut self, data: Vec<u8>) -> ConsensusResult<Block> {
        if !self.is_leader() {
            return Err(ConsensusError::Internal("Only leader can create blocks".to_string()));
        }

        let parent_hash = self.high_qc
            .as_ref()
            .map(|qc| qc.block_hash)
            .unwrap_or([0u8; 32]);

        let block = Block {
            view: self.view,
            parent_hash,
            data,
            qc: self.high_qc.clone(),
        };

        self.blocks.insert(self.view, block.clone());

        Ok(block)
    }

    /// Vote on a block
    pub fn vote_on_block(&self, block: &Block) -> ConsensusResult<Vote> {
        let block_hash = block.hash();

        // Sign the block hash
        let signature = self.crypto.sign(&block_hash);

        Ok(Vote {
            view: block.view,
            block_hash,
            voter_id: self.config.node_id,
            signature,
        })
    }

    /// Process a vote
    pub fn process_vote(&mut self, vote: Vote) -> ConsensusResult<Option<QuorumCertificate>> {
        // Verify signature
        if !self.crypto.verify(vote.voter_id, &vote.block_hash, &vote.signature) {
            warn!(voter = ?vote.voter_id, "Invalid vote signature");
            return Ok(None);
        }

        // Add vote
        self.votes
            .entry(vote.view)
            .or_insert_with(HashMap::new)
            .insert(vote.voter_id, vote.clone());

        // Check if we have quorum
        let vote_count = self.votes.get(&vote.view).map(|v| v.len()).unwrap_or(0);

        if vote_count >= self.config.quorum_size() {
            // Create QC
            let signatures = self.votes
                .get(&vote.view)
                .unwrap()
                .iter()
                .map(|(id, v)| (*id, v.signature.clone()))
                .collect();

            let qc = QuorumCertificate {
                view: vote.view,
                block_hash: vote.block_hash,
                signatures,
            };

            // Update high QC
            self.high_qc = Some(qc.clone());

            return Ok(Some(qc));
        }

        Ok(None)
    }

    /// Advance to next view
    pub fn advance_view(&mut self) {
        self.view.increment();
        self.phase = HotStuffPhase::Prepare;
    }

    /// Check if 3-chain rule is satisfied (for commit)
    pub fn is_committable(&self, block: &Block) -> bool {
        // Simplified: check if we have 3 consecutive QCs
        // Real implementation would traverse the chain
        self.high_qc.is_some() && block.qc.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotstuff_config() {
        let config = HotStuffConfig::new(NodeId::new(1), 4).unwrap();
        assert_eq!(config.byzantine_threshold, 1);
        assert_eq!(config.quorum_size(), 3); // 2f+1
    }

    #[test]
    fn test_block_hash() {
        let block = Block {
            view: ViewNumber::new(1),
            parent_hash: [0u8; 32],
            data: b"test data".to_vec(),
            qc: None,
        };

        let hash1 = block.hash();
        let hash2 = block.hash();

        // Hash should be deterministic
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_quorum_certificate() {
        let mut signatures = HashMap::new();
        signatures.insert(NodeId::new(1), Signature { bytes: [0u8; 64] });
        signatures.insert(NodeId::new(2), Signature { bytes: [0u8; 64] });
        signatures.insert(NodeId::new(3), Signature { bytes: [0u8; 64] });

        let qc = QuorumCertificate {
            view: ViewNumber::new(1),
            block_hash: [0u8; 32],
            signatures,
        };

        assert!(qc.has_quorum(3));
        assert!(!qc.has_quorum(4));
    }

    #[test]
    fn test_hotstuff_leader() {
        let config = HotStuffConfig::new(NodeId::new(0), 4).unwrap();
        let crypto = CryptoProvider::new();
        let node = HotStuffNode::new(config, crypto);

        // View 0, leader is node 0
        assert!(node.is_leader());
    }
}
