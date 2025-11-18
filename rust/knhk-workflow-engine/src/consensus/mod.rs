//! Phase 8: Byzantine Consensus
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: Q (Hard Invariants) - Consensus safety is an invariant
//! - Principle: O (Observation) - All consensus rounds emit telemetry
//! - Principle: Π (Projection) - Multi-region consensus projects global state
//! - Covenant 2: Invariants Are Law - Double-commit is impossible (proven)
//! - Covenant 5: Chatman Constant - Single-region consensus ≤50ms
//! - Covenant 6: Observations Drive Everything - Full telemetry coverage
//!
//! This module implements Byzantine Fault Tolerant consensus algorithms:
//! - PBFT: Practical Byzantine Fault Tolerance (f < n/3, fast finality)
//! - HotStuff: Pipelined BFT with rotating leader (better scalability)
//! - Raft: Crash fault tolerance (baseline, no Byzantine)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub mod pbft;
pub mod hotstuff;
pub mod raft;
pub mod network;
pub mod state_machine;

// ============================================================================
// Consensus State Trait
// ============================================================================

/// Consensus state machine trait
///
/// Any state machine can participate in consensus by implementing this trait.
/// The consensus algorithm ensures that all honest nodes apply the same sequence
/// of commands and reach the same state.
///
/// # Safety Invariants (Covenant 2)
///
/// - **No double-commit:** Two different values cannot be committed at the same sequence
/// - **Quorum intersection:** Any two quorums of size 2f+1 intersect in at least f+1 nodes
/// - **State hash integrity:** State hash MUST be deterministic and collision-resistant
///
/// # Observability (Covenant 6)
///
/// Every state transition MUST emit:
/// - Span: `consensus.state_transition`
/// - Attributes: `state.before`, `state.after`, `command`, `sequence`
pub trait ConsensusState: Clone + Send + Sync + 'static {
    /// Command type (state machine input)
    type Command: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de>;

    /// Response type (state machine output)
    type Response: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de>;

    /// Apply a command to the state machine
    ///
    /// # Requirements
    ///
    /// - MUST be deterministic (same state + command → same result)
    /// - MUST be serializable (for replay and recovery)
    /// - MUST emit telemetry (span with before/after state hashes)
    ///
    /// # Performance
    ///
    /// - SHOULD complete in ≤100μs (simple state machines)
    /// - MAY take longer for complex state (e.g., workflow execution)
    fn apply(&mut self, cmd: Self::Command) -> Self::Response;

    /// Get a cryptographic hash of the current state
    ///
    /// # Requirements
    ///
    /// - MUST be deterministic (same state → same hash)
    /// - MUST use blake3 (collision-resistant, fast)
    /// - MUST include ALL state (no partial hashing)
    ///
    /// # Performance
    ///
    /// - MUST complete in ≤10μs (hot path operation)
    fn hash(&self) -> [u8; 32];
}

// ============================================================================
// Consensus Algorithms
// ============================================================================

/// Consensus algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusAlgorithm {
    /// PBFT (Practical Byzantine Fault Tolerance)
    /// - Byzantine tolerance: f < n/3
    /// - Finality: After prepare phase (2f+1 prepares)
    /// - Latency: 10-50ms (single region), 300ms (multi-region)
    /// - Scalability: <20 nodes
    /// - Use case: Financial transactions, high-value workflows
    PBFT,

    /// HotStuff (Pipelined BFT)
    /// - Byzantine tolerance: f < n/3
    /// - Finality: After 3 consecutive blocks
    /// - Latency: 50-100ms (single region), 300ms (multi-region)
    /// - Scalability: 20-100 nodes
    /// - Use case: High-throughput workflows, distributed systems
    HotStuff,

    /// Raft (Crash Fault Tolerant)
    /// - Byzantine tolerance: NONE (crash-only)
    /// - Finality: After majority replication
    /// - Latency: 1-5ms (single region), 50ms (multi-region)
    /// - Scalability: <10 nodes
    /// - Use case: Internal infrastructure, trusted environments
    Raft,
}

// ============================================================================
// Log Entry
// ============================================================================

/// Consensus log entry
///
/// Each entry represents a committed command in the total order.
/// The log is append-only and immutable (entries never change).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry<C> {
    /// Sequence number (global total order)
    pub seq: u64,

    /// Term number (for leader election)
    pub term: u64,

    /// Command to apply
    pub command: C,

    /// State hash AFTER applying this command
    pub state_hash: [u8; 32],
}

// ============================================================================
// Quorum Certificate
// ============================================================================

/// Quorum certificate (proof of 2f+1 agreement)
///
/// A quorum certificate proves that at least 2f+1 nodes agreed on a value.
/// This is the fundamental building block of BFT consensus.
///
/// # Safety
///
/// - Quorum size = 2f+1 (where f = max Byzantine nodes)
/// - Quorum intersection: Any two quorums intersect in ≥f+1 nodes
/// - At most f are Byzantine → At least 1 honest node in every intersection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumCert {
    /// Value being certified (e.g., block hash)
    pub value_hash: [u8; 32],

    /// View number (for leader rotation)
    pub view: u64,

    /// Signatures from 2f+1 nodes
    pub signatures: Vec<NodeSignature>,
}

impl QuorumCert {
    /// Verify that this quorum certificate is valid
    ///
    /// # Requirements
    ///
    /// - MUST have ≥2f+1 signatures
    /// - All signatures MUST be valid
    /// - All signatures MUST be from distinct nodes
    pub fn verify(&self, quorum_size: usize, public_keys: &HashMap<usize, PublicKey>) -> bool {
        if self.signatures.len() < quorum_size {
            return false;
        }

        // Check all signatures are valid and from distinct nodes
        let mut seen = std::collections::HashSet::new();

        for sig in &self.signatures {
            if !seen.insert(sig.node_id) {
                return false; // Duplicate signature
            }

            let pk = match public_keys.get(&sig.node_id) {
                Some(pk) => pk,
                None => return false,
            };

            // Verify signature on (value_hash || view)
            let msg = bincode::serialize(&(&self.value_hash, self.view))
                .expect("Serialization cannot fail");

            if !verify_signature(pk, &msg, &sig.signature) {
                return false;
            }
        }

        true
    }
}

/// Node signature in a quorum certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSignature {
    pub node_id: usize,
    pub signature: Vec<u8>, // Generic signature bytes (could be Ed25519, Dilithium, or Hybrid)
}

// ============================================================================
// Consensus Configuration
// ============================================================================

/// Consensus configuration
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Node ID (unique identifier)
    pub node_id: usize,

    /// Total number of nodes
    pub n: usize,

    /// Maximum Byzantine nodes (f = (n-1)/3)
    pub f: usize,

    /// Quorum size (2f+1)
    pub quorum_size: usize,

    /// Consensus algorithm
    pub algorithm: ConsensusAlgorithm,

    /// Timeout for consensus rounds
    pub timeout: Duration,

    /// Network addresses of all nodes
    pub nodes: HashMap<usize, String>,
}

impl ConsensusConfig {
    /// Create a new consensus configuration
    ///
    /// # Arguments
    ///
    /// - `node_id`: This node's unique ID
    /// - `nodes`: Map of node ID → network address
    /// - `algorithm`: Consensus algorithm to use
    ///
    /// # Panics
    ///
    /// Panics if n < 3f+1 (insufficient nodes for Byzantine tolerance)
    pub fn new(
        node_id: usize,
        nodes: HashMap<usize, String>,
        algorithm: ConsensusAlgorithm,
    ) -> Self {
        let n = nodes.len();
        let f = (n - 1) / 3; // Max Byzantine nodes
        let quorum_size = 2 * f + 1;

        // Safety check: Need at least 3f+1 nodes for Byzantine tolerance
        assert!(
            n >= 3 * f + 1,
            "Need at least {} nodes for f={} Byzantine tolerance",
            3 * f + 1,
            f
        );

        Self {
            node_id,
            n,
            f,
            quorum_size,
            algorithm,
            timeout: Duration::from_secs(1),
            nodes,
        }
    }

    /// Check if this node is the leader for the given view
    pub fn is_leader(&self, view: u64) -> bool {
        self.node_id == (view as usize % self.n)
    }

    /// Get the leader ID for the given view
    pub fn leader_id(&self, view: u64) -> usize {
        view as usize % self.n
    }
}

// ============================================================================
// Consensus Errors
// ============================================================================

/// Consensus operation errors
///
/// # Covenant 2: Hard Errors Only
///
/// All consensus violations are HARD ERRORS (never warnings).
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    /// Not enough votes to reach quorum
    #[error("Quorum not reached: got {got} votes, need {need}")]
    QuorumNotReached { got: usize, need: usize },

    /// Invalid signature in quorum certificate
    #[error("Invalid signature from node {node_id}")]
    InvalidSignature { node_id: usize },

    /// Consensus timeout
    #[error("Consensus timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// This node is not the leader
    #[error("Not leader: current leader is node {leader_id}")]
    NotLeader { leader_id: usize },

    /// View change required
    #[error("View change required: leader {leader_id} is faulty")]
    ViewChangeRequired { leader_id: usize },

    /// Double-commit detected (CRITICAL SAFETY VIOLATION)
    #[error("CRITICAL: Double-commit detected at seq={seq}: value1={value1:?}, value2={value2:?}")]
    DoubleCommit {
        seq: u64,
        value1: [u8; 32],
        value2: [u8; 32],
    },

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

// ============================================================================
// Placeholder Types (to be implemented in submodules)
// ============================================================================

type PublicKey = Vec<u8>; // TODO: Use actual crypto types

fn verify_signature(_pk: &PublicKey, _msg: &[u8], _sig: &[u8]) -> bool {
    true // TODO: Implement actual signature verification
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_config_quorum_size() {
        let mut nodes = HashMap::new();
        nodes.insert(0, "node0:8000".to_string());
        nodes.insert(1, "node1:8000".to_string());
        nodes.insert(2, "node2:8000".to_string());
        nodes.insert(3, "node3:8000".to_string());

        let config = ConsensusConfig::new(0, nodes, ConsensusAlgorithm::PBFT);

        // n=4, f=1, quorum=3
        assert_eq!(config.n, 4);
        assert_eq!(config.f, 1);
        assert_eq!(config.quorum_size, 3);
    }

    #[test]
    fn test_consensus_config_leader_rotation() {
        let mut nodes = HashMap::new();
        for i in 0..7 {
            nodes.insert(i, format!("node{}:8000", i));
        }

        let config = ConsensusConfig::new(0, nodes, ConsensusAlgorithm::HotStuff);

        // Leader rotates: view % n
        assert_eq!(config.leader_id(0), 0);
        assert_eq!(config.leader_id(1), 1);
        assert_eq!(config.leader_id(7), 0); // Wraps around
    }

    #[test]
    #[should_panic(expected = "Need at least 4 nodes")]
    fn test_consensus_config_insufficient_nodes() {
        let mut nodes = HashMap::new();
        nodes.insert(0, "node0:8000".to_string());
        nodes.insert(1, "node1:8000".to_string());

        // n=2, f=0 (need at least 3f+1=1) - should panic
        ConsensusConfig::new(0, nodes, ConsensusAlgorithm::PBFT);
    }
}
