//! Practical Byzantine Fault Tolerance (PBFT) Consensus
//!
//! PBFT provides Byzantine fault tolerance with a 4-phase commit protocol:
//! 1. Pre-Prepare: Primary broadcasts proposal
//! 2. Prepare: Replicas broadcast prepare messages
//! 3. Commit: Replicas broadcast commit messages
//! 4. Reply: Once 2f+1 commits received, consensus is reached
//!
//! Tolerates f = ⌊(n-1)/3⌋ Byzantine faults.

use crate::{
    errors::{ByzantineError, Result},
    network::ByzantineNetwork,
    protocols::{Consensus, ConsensusProof, PrivateKey, PublicKey, Signature},
    Block, Hash, NodeId, WorkflowDecision,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// PBFT consensus phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PBFTPhase {
    PrePrepare,
    Prepare,
    Commit,
    ViewChange,
}

/// PBFT message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PBFTMessage {
    PrePrepare {
        view: u64,
        sequence: u64,
        block: Block,
        signature: Signature,
    },
    Prepare {
        view: u64,
        sequence: u64,
        block_hash: Hash,
        node_id: NodeId,
        signature: Signature,
    },
    Commit {
        view: u64,
        sequence: u64,
        block_hash: Hash,
        node_id: NodeId,
        signature: Signature,
    },
    ViewChange {
        new_view: u64,
        node_id: NodeId,
        prepared_proof: Vec<PBFTMessage>,
        signature: Signature,
    },
    NewView {
        new_view: u64,
        view_change_messages: Vec<PBFTMessage>,
        signature: Signature,
    },
}

/// State for a single consensus instance
#[derive(Debug)]
struct ConsensusInstance {
    block: Block,
    phase: PBFTPhase,
    prepare_messages: HashMap<NodeId, Signature>,
    commit_messages: HashMap<NodeId, Signature>,
    #[allow(dead_code)]
    start_time: Instant,
}

/// PBFT consensus implementation
pub struct PBFTConsensus {
    node_id: NodeId,
    nodes: Vec<NodeId>,
    view: Arc<RwLock<u64>>,
    f: usize,
    timeout: Duration,
    private_key: PrivateKey,
    public_keys: HashMap<NodeId, PublicKey>,
    network: Arc<ByzantineNetwork>,
    instances: Arc<DashMap<u64, ConsensusInstance>>,
    sequence_number: Arc<RwLock<u64>>,
    committed_blocks: Arc<RwLock<Vec<Block>>>,
}

impl PBFTConsensus {
    /// Create a new PBFT consensus instance
    pub fn new(
        node_id: NodeId,
        nodes: Vec<NodeId>,
        timeout: Duration,
        network: Arc<ByzantineNetwork>,
    ) -> Self {
        let n = nodes.len();
        let f = (n - 1) / 3;

        // Generate keypair (simplified - use ed25519-dalek in production)
        let private_key = PrivateKey(vec![0u8; 32]);
        let public_keys = nodes
            .iter()
            .map(|&id| (id, PublicKey(vec![0u8; 32])))
            .collect();

        info!(
            "PBFT initialized: node={}, n={}, f={}, timeout={:?}",
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
            instances: Arc::new(DashMap::new()),
            sequence_number: Arc::new(RwLock::new(0)),
            committed_blocks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current primary node
    fn primary(&self, view: u64) -> NodeId {
        let idx = (view as usize) % self.nodes.len();
        self.nodes[idx]
    }

    /// Check if this node is the current primary
    async fn is_primary(&self) -> bool {
        let view = *self.view.read().await;
        self.primary(view) == self.node_id
    }

    /// Propose a new block (primary only)
    pub async fn propose(&self, decisions: Vec<WorkflowDecision>) -> Result<Consensus> {
        if !self.is_primary().await {
            return Err(ByzantineError::Configuration(
                "Only primary can propose blocks".to_string(),
            ));
        }

        let view = *self.view.read().await;
        let mut seq = self.sequence_number.write().await;
        *seq += 1;
        let sequence = *seq;
        drop(seq);

        // Create block
        let parent_hash = self
            .committed_blocks
            .read()
            .await
            .last()
            .map(|b| b.hash)
            .unwrap_or_default();
        let block = Block::new(parent_hash, view, decisions);

        info!(
            "Proposing block: view={}, seq={}, hash={}",
            view, sequence, block.hash
        );

        // Sign and broadcast pre-prepare
        let signature = self.sign_block(&block);
        let pre_prepare = PBFTMessage::PrePrepare {
            view,
            sequence,
            block: block.clone(),
            signature: signature.clone(),
        };

        self.network
            .broadcast(bincode::serialize(&pre_prepare)?)
            .await?;

        // Initialize consensus instance
        self.instances.insert(
            sequence,
            ConsensusInstance {
                block: block.clone(),
                phase: PBFTPhase::PrePrepare,
                prepare_messages: HashMap::new(),
                commit_messages: HashMap::new(),
                start_time: Instant::now(),
            },
        );

        // Wait for consensus
        self.wait_for_consensus(sequence).await
    }

    /// Process incoming PBFT message
    pub async fn handle_message(&self, msg: PBFTMessage) -> Result<()> {
        match msg {
            PBFTMessage::PrePrepare {
                view,
                sequence,
                block,
                signature,
            } => self.handle_pre_prepare(view, sequence, block, signature).await,
            PBFTMessage::Prepare {
                view,
                sequence,
                block_hash,
                node_id,
                signature,
            } => {
                self.handle_prepare(view, sequence, block_hash, node_id, signature)
                    .await
            }
            PBFTMessage::Commit {
                view,
                sequence,
                block_hash,
                node_id,
                signature,
            } => {
                self.handle_commit(view, sequence, block_hash, node_id, signature)
                    .await
            }
            PBFTMessage::ViewChange { .. } => self.handle_view_change(msg).await,
            PBFTMessage::NewView { .. } => self.handle_new_view(msg).await,
        }
    }

    async fn handle_pre_prepare(
        &self,
        view: u64,
        sequence: u64,
        block: Block,
        signature: Signature,
    ) -> Result<()> {
        let current_view = *self.view.read().await;
        if view != current_view {
            return Err(ByzantineError::InvalidView {
                expected: current_view,
                got: view,
            });
        }

        // Verify signature
        if !self.verify_signature(&block, &signature, self.primary(view)) {
            return Err(ByzantineError::InvalidSignature {
                node_id: self.primary(view).0,
            });
        }

        debug!("Received pre-prepare: view={}, seq={}", view, sequence);

        // Store instance
        self.instances.insert(
            sequence,
            ConsensusInstance {
                block: block.clone(),
                phase: PBFTPhase::Prepare,
                prepare_messages: HashMap::new(),
                commit_messages: HashMap::new(),
                start_time: Instant::now(),
            },
        );

        // Send prepare message
        let prepare = PBFTMessage::Prepare {
            view,
            sequence,
            block_hash: block.hash,
            node_id: self.node_id,
            signature: self.sign_block(&block),
        };

        self.network
            .broadcast(bincode::serialize(&prepare)?)
            .await?;

        Ok(())
    }

    async fn handle_prepare(
        &self,
        view: u64,
        sequence: u64,
        block_hash: Hash,
        node_id: NodeId,
        signature: Signature,
    ) -> Result<()> {
        if let Some(mut instance) = self.instances.get_mut(&sequence) {
            if instance.block.hash != block_hash {
                warn!("Block hash mismatch in prepare");
                return Err(ByzantineError::InvalidBlock(
                    "Hash mismatch".to_string(),
                ));
            }

            // Store prepare message
            instance.prepare_messages.insert(node_id, signature);

            // Check if we have 2f prepare messages (quorum)
            if instance.prepare_messages.len() > 2 * self.f - 1 {
                debug!("Prepare quorum reached: seq={}", sequence);
                instance.phase = PBFTPhase::Commit;

                // Send commit message
                let commit = PBFTMessage::Commit {
                    view,
                    sequence,
                    block_hash,
                    node_id: self.node_id,
                    signature: self.sign_block(&instance.block),
                };

                drop(instance);
                self.network
                    .broadcast(bincode::serialize(&commit)?)
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_commit(
        &self,
        _view: u64,
        sequence: u64,
        block_hash: Hash,
        node_id: NodeId,
        signature: Signature,
    ) -> Result<()> {
        if let Some(mut instance) = self.instances.get_mut(&sequence) {
            if instance.block.hash != block_hash {
                warn!("Block hash mismatch in commit");
                return Err(ByzantineError::InvalidBlock(
                    "Hash mismatch".to_string(),
                ));
            }

            // Store commit message
            instance.commit_messages.insert(node_id, signature);

            // Check if we have 2f+1 commit messages (quorum)
            if instance.commit_messages.len() > 2 * self.f {
                info!("Commit quorum reached: seq={}", sequence);
                let block = instance.block.clone();
                drop(instance);

                // Commit block
                self.committed_blocks.write().await.push(block);
            }
        }

        Ok(())
    }

    async fn wait_for_consensus(&self, sequence: u64) -> Result<Consensus> {
        let deadline = Instant::now() + self.timeout;

        loop {
            if Instant::now() > deadline {
                return Err(ByzantineError::ConsensusTimeout {
                    timeout_ms: self.timeout.as_millis() as u64,
                });
            }

            if let Some(instance) = self.instances.get(&sequence) {
                if instance.commit_messages.len() > 2 * self.f {
                    let commit_sigs: Vec<_> = instance
                        .commit_messages
                        .iter()
                        .map(|(&id, sig)| (id, sig.clone()))
                        .collect();

                    return Ok(Consensus {
                        block: instance.block.clone(),
                        proof: ConsensusProof::PBFT {
                            view: *self.view.read().await,
                            commit_signatures: commit_sigs,
                        },
                    });
                }
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Initiate view change
    pub async fn view_change(&self) -> Result<()> {
        let mut view = self.view.write().await;
        *view += 1;
        let new_view = *view;
        drop(view);

        info!("Initiating view change to view {}", new_view);

        // Collect prepared proofs
        let prepared_proof: Vec<_> = self
            .instances
            .iter()
            .filter(|entry| entry.value().prepare_messages.len() >= 2 * self.f)
            .map(|entry| {
                PBFTMessage::Prepare {
                    view: new_view - 1,
                    sequence: *entry.key(),
                    block_hash: entry.value().block.hash,
                    node_id: self.node_id,
                    signature: self.sign_block(&entry.value().block),
                }
            })
            .collect();

        let view_change = PBFTMessage::ViewChange {
            new_view,
            node_id: self.node_id,
            prepared_proof,
            signature: Signature(vec![0u8; 64]),
        };

        self.network
            .broadcast(bincode::serialize(&view_change)?)
            .await?;

        Ok(())
    }

    async fn handle_view_change(&self, _msg: PBFTMessage) -> Result<()> {
        // Simplified view change handling
        debug!("Received view change message");
        Ok(())
    }

    async fn handle_new_view(&self, _msg: PBFTMessage) -> Result<()> {
        // Simplified new view handling
        debug!("Received new view message");
        Ok(())
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

    /// Get committed blocks
    pub async fn committed_blocks(&self) -> Vec<Block> {
        self.committed_blocks.read().await.clone()
    }

    /// Get current view
    pub async fn current_view(&self) -> u64 {
        *self.view.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pbft_primary_rotation() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
        let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
        let pbft = PBFTConsensus::new(NodeId(0), nodes.clone(), Duration::from_secs(5), network);

        assert_eq!(pbft.primary(0), NodeId(0));
        assert_eq!(pbft.primary(1), NodeId(1));
        assert_eq!(pbft.primary(2), NodeId(2));
        assert_eq!(pbft.primary(3), NodeId(3));
        assert_eq!(pbft.primary(4), NodeId(0));
    }

    #[tokio::test]
    async fn test_pbft_f_calculation() {
        let nodes = vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)];
        let network = Arc::new(ByzantineNetwork::new(nodes.clone()));
        let pbft = PBFTConsensus::new(NodeId(0), nodes, Duration::from_secs(5), network);

        assert_eq!(pbft.f, 1); // (4-1)/3 = 1
    }
}
