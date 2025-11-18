//! Byzantine Fault-Tolerant Consensus for Distributed MAPE-K
//!
//! This crate implements Byzantine consensus protocols (PBFT, HotStuff) to enable
//! distributed autonomic decision-making across workflow networks, tolerating up to
//! 1/3 faulty or malicious nodes.
//!
//! # DOCTRINE Alignment
//!
//! - **Principle Q (Hard Invariants)**: Consensus safety and liveness guarantees
//! - **Covenant 2**: Distributed decision invariants are law
//! - **Covenant 3**: MAPE-K at machine speed across unreliable networks
//!
//! # Architecture
//!
//! - `protocols::pbft`: Practical Byzantine Fault Tolerance (4-phase commit)
//! - `protocols::hotstuff`: HotStuff (optimistic 3-RTT consensus)
//! - `network`: Byzantine network simulation with failure injection
//! - `mapek_byzantine`: Byzantine-resistant MAPE-K integration
//! - `qc_manager`: Quorum certificate management and verification

pub mod errors;
pub mod mapek_byzantine;
pub mod network;
pub mod protocols;
pub mod qc_manager;

pub use errors::{ByzantineError, Result};
pub use mapek_byzantine::ByzantineMAPEK;
pub use network::ByzantineNetwork;
pub use protocols::{hotstuff::HotStuffConsensus, pbft::PBFTConsensus};
pub use qc_manager::{QuorumCertificate, QuorumCertificateManager};

use serde::{Deserialize, Serialize};
use std::fmt;

/// Node identifier in the Byzantine network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

/// Workflow decision proposed for consensus
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowDecision {
    pub workflow_id: String,
    pub action: DecisionAction,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionAction {
    Execute,
    Reject,
    Delay { until: u64 },
    Reconfigure { new_config: String },
}

/// Block containing workflow decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub hash: Hash,
    pub parent_hash: Hash,
    pub view: u64,
    pub decisions: Vec<WorkflowDecision>,
    pub timestamp: u64,
}

impl Block {
    pub fn new(parent_hash: Hash, view: u64, decisions: Vec<WorkflowDecision>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut block = Self {
            hash: Hash::default(),
            parent_hash,
            view,
            decisions,
            timestamp,
        };
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> Hash {
        use sha2::{Digest, Sha256};
        let data = bincode::serialize(&(
            &self.parent_hash,
            self.view,
            &self.decisions,
            self.timestamp,
        ))
        .unwrap();
        let mut hasher = Sha256::new();
        hasher.update(data);
        Hash(hasher.finalize().into())
    }
}

/// Cryptographic hash (SHA-256)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Hash(pub [u8; 32]);

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0[..8]))
    }
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
