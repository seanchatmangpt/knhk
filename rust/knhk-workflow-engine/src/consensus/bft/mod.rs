//! Byzantine Fault Tolerance implementations
//!
//! This module provides BFT consensus protocols for scenarios with malicious actors:
//! - **PBFT (Practical Byzantine Fault Tolerance)**: Classic 3-phase BFT protocol
//! - **HotStuff**: Modern streamlined BFT with linear communication complexity
//!
//! # Byzantine Fault Tolerance
//!
//! BFT protocols tolerate up to f Byzantine (arbitrary) failures in a system of 3f+1 nodes.
//! Byzantine failures include:
//! - Crash failures
//! - Message tampering
//! - Equivocation (sending different messages to different nodes)
//! - Arbitrary malicious behavior
//!
//! # Safety and Liveness
//!
//! - **Safety**: Never commit conflicting values (even with f Byzantine nodes)
//! - **Liveness**: Eventually commit if ≤ f nodes are Byzantine and network is synchronous

pub mod crypto;
pub mod hotstuff;
pub mod pbft;

pub use crypto::{CryptoProvider, KeyPair, Signature, SignatureVerifier};
pub use hotstuff::{HotStuffConfig, HotStuffNode};
pub use pbft::{PbftConfig, PbftNode};

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BFT protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BftProtocol {
    /// PBFT (Practical Byzantine Fault Tolerance)
    Pbft,
    /// HotStuff (Linear BFT)
    HotStuff,
}

/// BFT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BftConfig {
    /// Node ID
    pub node_id: NodeId,

    /// Peer addresses
    pub peers: Vec<SocketAddr>,

    /// Protocol type
    pub protocol: BftProtocol,

    /// Byzantine threshold (f in 3f+1)
    pub byzantine_threshold: usize,

    /// View change timeout
    pub view_change_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,
}

impl Default for BftConfig {
    fn default() -> Self {
        Self {
            node_id: NodeId::new(1),
            peers: Vec::new(),
            protocol: BftProtocol::HotStuff, // Default to HotStuff (better performance)
            byzantine_threshold: 1, // 3f+1 = 4 nodes minimum
            view_change_timeout: Duration::from_millis(100),
            request_timeout: Duration::from_secs(1),
        }
    }
}

/// BFT cluster
pub struct BftCluster {
    /// Configuration
    config: BftConfig,

    /// Current view
    view: Arc<RwLock<ViewNumber>>,

    /// Crypto provider
    crypto: Arc<CryptoProvider>,

    /// Metrics
    metrics: Arc<RwLock<ConsensusMetrics>>,
}

impl BftCluster {
    /// Create a new BFT cluster
    pub fn new(config: BftConfig, crypto: CryptoProvider) -> Self {
        Self {
            config,
            view: Arc::new(RwLock::new(ViewNumber::new(0))),
            crypto: Arc::new(crypto),
            metrics: Arc::new(RwLock::new(ConsensusMetrics::new())),
        }
    }

    /// Create a new PBFT cluster
    pub fn new_pbft(
        node_id: NodeId,
        peers: Vec<SocketAddr>,
        crypto: CryptoProvider,
    ) -> ConsensusResult<Self> {
        let config = BftConfig {
            node_id,
            peers,
            protocol: BftProtocol::Pbft,
            ..Default::default()
        };

        Ok(Self::new(config, crypto))
    }

    /// Create a new HotStuff cluster
    pub fn new_hotstuff(
        node_id: NodeId,
        peers: Vec<SocketAddr>,
        crypto: CryptoProvider,
    ) -> ConsensusResult<Self> {
        let config = BftConfig {
            node_id,
            peers,
            protocol: BftProtocol::HotStuff,
            ..Default::default()
        };

        Ok(Self::new(config, crypto))
    }

    /// Propose a value for consensus
    pub async fn propose<T: Serialize>(&self, value: T) -> ConsensusResult<Decision<T>> {
        let start = Instant::now();

        // Serialize value
        let data = bincode::serialize(&value)
            .map_err(|e| ConsensusError::Serialization(e.to_string()))?;

        // Create proposal
        let proposal = Proposal {
            view: *self.view.read().await,
            sequence: SequenceNumber::new(1), // Simplified
            data,
        };

        // Sign proposal
        let signature = self.crypto.sign(&proposal.data);

        info!(
            node_id = ?self.config.node_id,
            view = ?proposal.view,
            protocol = ?self.config.protocol,
            "Proposing value for BFT consensus"
        );

        // Run consensus protocol
        let decision = match self.config.protocol {
            BftProtocol::Pbft => self.run_pbft(proposal, signature).await?,
            BftProtocol::HotStuff => self.run_hotstuff(proposal, signature).await?,
        };

        // Record metrics
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        let mut metrics = self.metrics.write().await;
        metrics.record_proposal(latency_ms, decision.committed);

        Ok(decision)
    }

    /// Run PBFT consensus
    async fn run_pbft(
        &self,
        proposal: Proposal,
        signature: Signature,
    ) -> ConsensusResult<Decision<Vec<u8>>> {
        // PBFT 3-phase protocol:
        // 1. PRE-PREPARE: Primary broadcasts proposal
        // 2. PREPARE: Replicas broadcast PREPARE messages
        // 3. COMMIT: Replicas broadcast COMMIT messages

        // Simplified implementation - real version would:
        // - Broadcast PRE-PREPARE to all replicas
        // - Collect 2f PREPARE messages
        // - Broadcast COMMIT
        // - Collect 2f+1 COMMIT messages
        // - Deliver decision

        Ok(Decision {
            view: proposal.view,
            sequence: proposal.sequence,
            value: proposal.data,
            committed: true,
        })
    }

    /// Run HotStuff consensus
    async fn run_hotstuff(
        &self,
        proposal: Proposal,
        signature: Signature,
    ) -> ConsensusResult<Decision<Vec<u8>>> {
        // HotStuff 3-chain protocol:
        // 1. PREPARE: Leader proposes, replicas vote
        // 2. PRE-COMMIT: Collect 2f+1 votes, broadcast
        // 3. COMMIT: Collect 2f+1 votes, broadcast
        // 4. DECIDE: Collect 2f+1 votes, commit

        // Simplified implementation
        Ok(Decision {
            view: proposal.view,
            sequence: proposal.sequence,
            value: proposal.data,
            committed: true,
        })
    }

    /// Get consensus metrics
    pub async fn metrics(&self) -> ConsensusMetrics {
        self.metrics.read().await.clone()
    }
}

/// BFT proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Proposal {
    /// View number
    view: ViewNumber,

    /// Sequence number
    sequence: SequenceNumber,

    /// Proposal data
    data: Vec<u8>,
}

/// BFT decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision<T> {
    /// View number
    pub view: ViewNumber,

    /// Sequence number
    pub sequence: SequenceNumber,

    /// Decided value
    pub value: T,

    /// Whether value was committed
    pub committed: bool,
}

impl<T> Decision<T> {
    /// Check if decision was committed
    pub fn is_committed(&self) -> bool {
        self.committed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bft_cluster_creation() {
        let crypto = CryptoProvider::new();
        let cluster = BftCluster::new_hotstuff(
            NodeId::new(1),
            vec![],
            crypto,
        ).unwrap();

        let view = *cluster.view.read().await;
        assert_eq!(view.inner(), 0);
    }
}
