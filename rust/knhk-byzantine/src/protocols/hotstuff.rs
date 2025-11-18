//! HotStuff Consensus Protocol
//!
//! HotStuff is a modern BFT consensus protocol with:
//! - 3-RTT optimistic path (vs PBFT's 4 phases)
//! - Pipelined design for higher throughput
//! - View synchronization for safety
//!
//! Tolerates f = ⌊(n-1)/3⌋ Byzantine faults.

use crate::{
    errors::{ByzantineError, Result},
    network::ByzantineNetwork,
    protocols::{Consensus, ConsensusProof, PrivateKey, PublicKey, Signature},
    qc_manager::{QuorumCertificate, QuorumCertificateManager},
    Block, Hash, NodeId, WorkflowDecision,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// HotStuff message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HotStuffMessage {
    Propose {
        block: Block,
        qc: QuorumCertificate,
        signature: Signature,
    },
    Vote {
        block_hash: Hash,
        view: u64,
        node_id: NodeId,
        signature: Signature,
    },
    NewView {
        view: u64,
        qc: QuorumCertificate,
        node_id: NodeId,
    },
}

/// HotStuff consensus state
pub struct HotStuffConsensus {
    node_id: NodeId,
    nodes: Vec<NodeId>,
    view: Arc<RwLock<u64>>,
    f: usize,
    timeout: Duration,
    private_key: PrivateKey,
    public_keys: HashMap<NodeId, PublicKey>,
    network: Arc<ByzantineNetwork>,
    qc_manager: Arc<QuorumCertificateManager>,

    // Consensus state
    locked_qc: Arc<RwLock<Option<QuorumCertificate>>>,
    committed_qc: Arc<RwLock<Option<QuorumCertificate>>>,
    proposal_tree: Arc<RwLock<ProposalTree>>,
    pending_votes: Arc<RwLock<HashMap<Hash, HashMap<NodeId, Signature>>>>,
    committed_blocks: Arc<RwLock<Vec<Block>>>,
}

/// Tree of proposed blocks
#[derive(Debug, Default)]
pub struct ProposalTree {
    blocks: HashMap<Hash, Block>,
    children: HashMap<Hash, Vec<Hash>>,
}

impl ProposalTree {
    pub fn insert(&mut self, block: Block) {
        let parent = block.parent_hash;
        let hash = block.hash;
        self.blocks.insert(hash, block);
        self.children.entry(parent).or_default().push(hash);
    }

    pub fn get(&self, hash: &Hash) -> Option<&Block> {
        self.blocks.get(hash)
    }

    pub fn get_chain(&self, from: Hash, to: Hash) -> Vec<Block> {
        let mut chain = Vec::new();
        let mut current = to;

        while current != from {
            if let Some(block) = self.blocks.get(&current) {
                chain.push(block.clone());
                current = block.parent_hash;
            } else {
                break;
            }
        }

        chain.reverse();
        chain
    }
}

impl HotStuffConsensus {
    /// Create new HotStuff consensus instance
    pub fn new(
        node_id: NodeId,
        nodes: Vec<NodeId>,
        timeout: Duration,
        network: Arc<ByzantineNetwork>,
    ) -> Self {
        let n = nodes.len();
        let f = (n - 1) / 3;

        // Generate keypair (simplified)
        let private_key = PrivateKey(vec![0u8; 32]);
        let public_keys = nodes
            .iter()
            .map(|&id| (id, PublicKey(vec![0u8; 32])))
            .collect();

        let qc_manager = Arc::new(QuorumCertificateManager::new(2 * f + 1));

        info!(
            "HotStuff initialized: node={}, n={}, f={}, timeout={:?}",
            node_id, n, f, timeout
        );

        Self {
            node_id,
            nodes,
            view: Arc::new(RwLock::new(0)),
            f,
            timeout,
            private_key,
            public_keys,
            network,
            qc_manager,
            locked_qc: Arc::new(RwLock::new(None)),
            committed_qc: Arc::new(RwLock::new(None)),
            proposal_tree: Arc::new(RwLock::new(ProposalTree::default())),
            pending_votes: Arc::new(RwLock::new(HashMap::new())),
            committed_blocks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current leader
    fn leader(&self, view: u64) -> NodeId {
        let idx = (view as usize) % self.nodes.len();
        self.nodes[idx]
    }

    /// Check if this node is current leader
    async fn is_leader(&self) -> bool {
        let view = *self.view.read().await;
        self.leader(view) == self.node_id
    }

    /// Propose a new block (leader only)
    pub async fn propose(&self, decisions: Vec<WorkflowDecision>) -> Result<Consensus> {
        if !self.is_leader().await {
            return Err(ByzantineError::Configuration(
                "Only leader can propose blocks".to_string(),
            ));
        }

        let view = *self.view.read().await;

        // Get highest QC
        let qc = self
            .locked_qc
            .read()
            .await
            .clone()
            .unwrap_or_else(|| self.genesis_qc());

        // Create block
        let parent_hash = qc.block_hash;
        let block = Block::new(parent_hash, view, decisions);

        info!("Proposing block: view={}, hash={}", view, block.hash);

        // Add to tree
        self.proposal_tree.write().await.insert(block.clone());

        // Sign and broadcast proposal
        let signature = self.sign_block(&block);
        let proposal = HotStuffMessage::Propose {
            block: block.clone(),
            qc: qc.clone(),
            signature,
        };

        self.network
            .broadcast(bincode::serialize(&proposal)?)
            .await?;

        // Wait for quorum
        self.wait_for_quorum(block.hash).await
    }

    /// Process incoming HotStuff message
    pub async fn handle_message(&self, msg: HotStuffMessage) -> Result<()> {
        match msg {
            HotStuffMessage::Propose { block, qc, signature } => {
                self.handle_propose(block, qc, signature).await
            }
            HotStuffMessage::Vote {
                block_hash,
                view,
                node_id,
                signature,
            } => self.handle_vote(block_hash, view, node_id, signature).await,
            HotStuffMessage::NewView { view, qc, node_id } => {
                self.handle_new_view(view, qc, node_id).await
            }
        }
    }

    async fn handle_propose(
        &self,
        block: Block,
        qc: QuorumCertificate,
        signature: Signature,
    ) -> Result<()> {
        let current_view = *self.view.read().await;
        if block.view != current_view {
            return Err(ByzantineError::InvalidView {
                expected: current_view,
                got: block.view,
            });
        }

        // Verify QC
        self.qc_manager.verify_qc(&qc).await?;

        // Verify block signature
        if !self.verify_signature(&block, &signature, self.leader(block.view)) {
            return Err(ByzantineError::InvalidSignature {
                node_id: self.leader(block.view).0,
            });
        }

        debug!("Received proposal: view={}, hash={}", block.view, block.hash);

        // Add to tree
        self.proposal_tree.write().await.insert(block.clone());

        // Update locked QC if qc.view > locked_qc.view
        {
            let mut locked = self.locked_qc.write().await;
            if locked.as_ref().map(|l| l.view).unwrap_or(0) < qc.view {
                *locked = Some(qc.clone());
            }
        }

        // Vote for block
        let vote = HotStuffMessage::Vote {
            block_hash: block.hash,
            view: block.view,
            node_id: self.node_id,
            signature: self.sign_block(&block),
        };

        self.network.broadcast(bincode::serialize(&vote)?).await?;

        // Update commit rule
        self.update_commit(&block).await?;

        Ok(())
    }

    async fn handle_vote(
        &self,
        block_hash: Hash,
        view: u64,
        node_id: NodeId,
        signature: Signature,
    ) -> Result<()> {
        // Store vote
        let mut votes = self.pending_votes.write().await;
        votes
            .entry(block_hash)
            .or_default()
            .insert(node_id, signature.clone());

        let vote_count = votes.get(&block_hash).map(|v| v.len()).unwrap_or(0);

        // Check if we have quorum (2f+1 votes)
        if vote_count > 2 * self.f {
            debug!("Quorum reached for block {}", block_hash);

            // Create QC
            let signatures: Vec<_> = votes
                .get(&block_hash)
                .unwrap()
                .iter()
                .map(|(&id, sig)| (id, sig.clone()))
                .collect();

            let qc = self.qc_manager.create_qc(block_hash, view, signatures);

            // Update locked QC
            {
                let mut locked = self.locked_qc.write().await;
                if locked.as_ref().map(|l| l.view).unwrap_or(0) < view {
                    *locked = Some(qc.clone());
                }
            }

            // If leader, propose next block
            if self.is_leader().await {
                // Leader will propose in next round
            }
        }

        Ok(())
    }

    async fn handle_new_view(
        &self,
        view: u64,
        qc: QuorumCertificate,
        _node_id: NodeId,
    ) -> Result<()> {
        let mut current_view = self.view.write().await;
        if view > *current_view {
            *current_view = view;
            debug!("Moved to new view {}", view);

            // Update locked QC
            let mut locked = self.locked_qc.write().await;
            if locked.as_ref().map(|l| l.view).unwrap_or(0) < qc.view {
                *locked = Some(qc);
            }
        }

        Ok(())
    }

    async fn update_commit(&self, block: &Block) -> Result<()> {
        // HotStuff commit rule: commit if 3-chain formed
        let tree = self.proposal_tree.read().await;

        if let Some(parent) = tree.get(&block.parent_hash) {
            if let Some(grandparent) = tree.get(&parent.parent_hash) {
                // 3-chain formed: grandparent -> parent -> block
                if block.view == parent.view + 1 && parent.view == grandparent.view + 1 {
                    info!("3-chain formed, committing block {}", grandparent.hash);

                    // Commit grandparent
                    self.committed_blocks.write().await.push(grandparent.clone());

                    // Update committed QC
                    let qc = self.qc_manager.create_qc(
                        grandparent.hash,
                        grandparent.view,
                        vec![], // Simplified
                    );
                    *self.committed_qc.write().await = Some(qc);
                }
            }
        }

        Ok(())
    }

    async fn wait_for_quorum(&self, block_hash: Hash) -> Result<Consensus> {
        let deadline = Instant::now() + self.timeout;

        loop {
            if Instant::now() > deadline {
                return Err(ByzantineError::ConsensusTimeout {
                    timeout_ms: self.timeout.as_millis() as u64,
                });
            }

            let votes = self.pending_votes.read().await;
            if let Some(vote_map) = votes.get(&block_hash) {
                if vote_map.len() > 2 * self.f {
                    let signatures: Vec<_> =
                        vote_map.iter().map(|(&id, sig)| (id, sig.clone())).collect();

                    let view = *self.view.read().await;
                    let qc = self.qc_manager.create_qc(block_hash, view, signatures);

                    let block = self
                        .proposal_tree
                        .read()
                        .await
                        .get(&block_hash)
                        .cloned()
                        .ok_or_else(|| {
                            ByzantineError::InvalidBlock("Block not found".to_string())
                        })?;

                    return Ok(Consensus {
                        block,
                        proof: ConsensusProof::HotStuff {
                            quorum_certificate: qc,
                        },
                    });
                }
            }
            drop(votes);

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Get committed blocks
    pub async fn decide(&self) -> Result<Vec<Block>> {
        Ok(self.committed_blocks.read().await.clone())
    }

    fn sign_block(&self, block: &Block) -> Signature {
        let data = bincode::serialize(block).unwrap();
        self.private_key.sign(&data)
    }

    fn verify_signature(&self, block: &Block, signature: &Signature, node_id: NodeId) -> bool {
        if let Some(public_key) = self.public_keys.get(&node_id) {
            let data = bincode::serialize(block).unwrap();
            signature.verify(&data, public_key)
        } else {
            false
        }
    }

    fn genesis_qc(&self) -> QuorumCertificate {
        QuorumCertificate {
            block_hash: Hash::default(),
            view: 0,
            signatures: vec![],
        }
    }

    /// Get current view
    pub async fn current_view(&self) -> u64 {
        *self.view.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_tree() {
        let mut tree = ProposalTree::default();

        let genesis = Block::new(Hash::default(), 0, vec![]);
        let block1 = Block::new(genesis.hash, 1, vec![]);
        let block2 = Block::new(block1.hash, 2, vec![]);

        tree.insert(genesis.clone());
        tree.insert(block1.clone());
        tree.insert(block2.clone());

        assert_eq!(tree.get(&genesis.hash).unwrap().view, 0);
        assert_eq!(tree.get(&block1.hash).unwrap().view, 1);
        assert_eq!(tree.get(&block2.hash).unwrap().view, 2);

        let chain = tree.get_chain(genesis.hash, block2.hash);
        assert_eq!(chain.len(), 2);
    }

    #[tokio::test]
    async fn test_hotstuff_leader_rotation() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
        let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
        let hotstuff = HotStuffConsensus::new(NodeId(0), nodes, Duration::from_secs(5), network);

        assert_eq!(hotstuff.leader(0), NodeId(0));
        assert_eq!(hotstuff.leader(1), NodeId(1));
        assert_eq!(hotstuff.leader(2), NodeId(2));
    }
}
