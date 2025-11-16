//! Distributed consensus protocols for multi-datacenter KNHK deployments
//!
//! This module provides:
//! - **Raft Consensus**: Crash fault tolerance with leader election and log replication
//! - **Byzantine Fault Tolerance**: PBFT and HotStuff for malicious actors
//! - **Hybrid Protocol**: Automatic selection based on threat model
//! - **State Machine Replication**: Replicated workflow state across regions
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │         Consensus Protocol Layer                │
//! ├─────────────────────────────────────────────────┤
//! │  ┌──────────────┐  ┌──────────────────────────┐│
//! │  │  Raft (Fast) │  │  BFT (Secure)           ││
//! │  │  - Leader    │  │  - PBFT                 ││
//! │  │  - Log       │  │  - HotStuff             ││
//! │  │  - Snapshot  │  │  - Crypto               ││
//! │  └──────────────┘  └──────────────────────────┘│
//! │           │                    │                │
//! │           └────────┬───────────┘                │
//! │                    ▼                            │
//! │         ┌─────────────────────┐                 │
//! │         │  Hybrid Dispatcher  │                 │
//! │         │  - Threat Detection │                 │
//! │         │  - Auto Fallback    │                 │
//! │         └─────────────────────┘                 │
//! │                    │                            │
//! │                    ▼                            │
//! │         ┌─────────────────────┐                 │
//! │         │ State Machine       │                 │
//! │         │ Replication         │                 │
//! │         └─────────────────────┘                 │
//! └─────────────────────────────────────────────────┘
//!                      │
//!                      ▼
//!         ┌─────────────────────────┐
//!         │  KNHK Workflow State    │
//!         │  - Cases                │
//!         │  - Policies             │
//!         │  - Overlays             │
//!         └─────────────────────────┘
//! ```
//!
//! # Performance Targets
//!
//! | Metric | Target | Typical |
//! |--------|--------|---------|
//! | Raft consensus latency | <10ms | 5-8ms |
//! | Raft throughput | >10K ops/sec | 15K ops/sec |
//! | BFT consensus latency | <50ms | 30-40ms |
//! | BFT throughput | >1K ops/sec | 2K ops/sec |
//! | Recovery time | <5s | 2-3s |
//!
//! # Usage
//!
//! ```rust,no_run
//! use knhk_workflow_engine::consensus::*;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), ConsensusError> {
//! // Initialize Raft cluster for normal operations
//! let raft = RaftCluster::builder()
//!     .node_id(1)
//!     .peers(vec![
//!         "node2:9001".parse()?,
//!         "node3:9001".parse()?,
//!     ])
//!     .election_timeout(Duration::from_millis(150))
//!     .heartbeat_interval(Duration::from_millis(50))
//!     .build()?;
//!
//! // Propose workflow state change
//! let proposal = WorkflowStateChange {
//!     case_id: "case-123".to_string(),
//!     new_state: "Running".to_string(),
//! };
//!
//! let commit_index = raft.propose(proposal).await?;
//! raft.wait_for_commit(commit_index).await?;
//!
//! // Use hybrid protocol for critical operations
//! let hybrid = HybridConsensus::new(raft, bft_cluster)?;
//! hybrid.propose_with_threat_detection(critical_proposal).await?;
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

pub mod bft;
pub mod hybrid;
pub mod raft;
pub mod replication;

pub use bft::{BftCluster, BftConfig, BftProtocol};
pub use hybrid::{HybridConsensus, ThreatLevel, ThreatModel};
pub use raft::{RaftCluster, RaftConfig, RaftNode, RaftRole};
pub use replication::{ReplicatedStateMachine, Snapshot, StateMachineOp};

/// Consensus protocol errors
#[derive(Error, Debug, Clone)]
pub enum ConsensusError {
    /// Not the leader
    #[error("Not the leader. Current leader: {leader:?}")]
    NotLeader { leader: Option<NodeId> },

    /// Timeout waiting for consensus
    #[error("Consensus timeout after {duration:?}")]
    Timeout { duration: Duration },

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid proposal
    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),

    /// Byzantine failure detected
    #[error("Byzantine failure detected: {0}")]
    ByzantineFailure(String),

    /// Insufficient quorum
    #[error("Insufficient quorum: have {current}, need {required}")]
    InsufficientQuorum { current: usize, required: usize },

    /// Log inconsistency
    #[error("Log inconsistency: {0}")]
    LogInconsistency(String),

    /// State machine error
    #[error("State machine error: {0}")]
    StateMachine(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for consensus operations
pub type ConsensusResult<T> = Result<T, ConsensusError>;

/// Node identifier in the consensus cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Create a new node ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the inner value
    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

impl std::str::FromStr for SocketAddr {
    type Err = std::net::AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
    }
}

/// Term number in Raft consensus (monotonically increasing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Term(pub u64);

impl Term {
    /// Create a new term
    pub fn new(term: u64) -> Self {
        Self(term)
    }

    /// Increment the term
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    /// Get the inner value
    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Term({})", self.0)
    }
}

/// Log index in the replicated log
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogIndex(pub u64);

impl LogIndex {
    /// Create a new log index
    pub fn new(index: u64) -> Self {
        Self(index)
    }

    /// Increment the index
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    /// Get the inner value
    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for LogIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LogIndex({})", self.0)
    }
}

/// View number in BFT consensus
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ViewNumber(pub u64);

impl ViewNumber {
    /// Create a new view number
    pub fn new(view: u64) -> Self {
        Self(view)
    }

    /// Increment the view
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    /// Get the inner value
    pub fn inner(&self) -> u64 {
        self.0
    }

    /// Get the primary node for this view
    pub fn primary(&self, num_nodes: usize) -> NodeId {
        NodeId::new((self.0 % num_nodes as u64))
    }
}

/// Sequence number in BFT consensus
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SequenceNumber(pub u64);

impl SequenceNumber {
    /// Create a new sequence number
    pub fn new(seq: u64) -> Self {
        Self(seq)
    }

    /// Increment the sequence number
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    /// Get the inner value
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// Consensus configuration shared across protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Node ID in the cluster
    pub node_id: NodeId,

    /// List of peer addresses
    pub peers: Vec<SocketAddr>,

    /// Heartbeat interval
    pub heartbeat_interval: Duration,

    /// Election timeout range (min, max)
    pub election_timeout_range: (Duration, Duration),

    /// Maximum log entries per AppendEntries RPC
    pub max_entries_per_rpc: usize,

    /// Snapshot threshold (entries before snapshot)
    pub snapshot_threshold: u64,

    /// Enable Byzantine fault tolerance
    pub enable_bft: bool,

    /// BFT threshold (f in 3f+1)
    pub bft_threshold: usize,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            node_id: NodeId::new(1),
            peers: Vec::new(),
            heartbeat_interval: Duration::from_millis(50),
            election_timeout_range: (Duration::from_millis(150), Duration::from_millis(300)),
            max_entries_per_rpc: 100,
            snapshot_threshold: 10000,
            enable_bft: false,
            bft_threshold: 1, // 3f+1 = 4 nodes minimum
        }
    }
}

/// Metrics for consensus operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    /// Total proposals submitted
    pub proposals_submitted: u64,

    /// Proposals committed
    pub proposals_committed: u64,

    /// Proposals rejected
    pub proposals_rejected: u64,

    /// Average consensus latency (milliseconds)
    pub avg_latency_ms: f64,

    /// P99 consensus latency (milliseconds)
    pub p99_latency_ms: f64,

    /// Leader elections performed
    pub leader_elections: u64,

    /// View changes (BFT)
    pub view_changes: u64,

    /// Byzantine failures detected
    pub byzantine_failures: u64,

    /// Snapshots created
    pub snapshots_created: u64,

    /// Total throughput (ops/sec)
    pub throughput: f64,
}

impl ConsensusMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a proposal
    pub fn record_proposal(&mut self, latency_ms: f64, committed: bool) {
        self.proposals_submitted += 1;
        if committed {
            self.proposals_committed += 1;
        } else {
            self.proposals_rejected += 1;
        }

        // Update running average (simple moving average)
        let n = self.proposals_submitted as f64;
        self.avg_latency_ms = ((n - 1.0) * self.avg_latency_ms + latency_ms) / n;

        // P99 approximation (exponential moving average with higher weight on peaks)
        self.p99_latency_ms = self.p99_latency_ms.max(latency_ms);
    }

    /// Record a leader election
    pub fn record_leader_election(&mut self) {
        self.leader_elections += 1;
    }

    /// Record a view change
    pub fn record_view_change(&mut self) {
        self.view_changes += 1;
    }

    /// Record a Byzantine failure
    pub fn record_byzantine_failure(&mut self) {
        self.byzantine_failures += 1;
    }

    /// Record a snapshot
    pub fn record_snapshot(&mut self) {
        self.snapshots_created += 1;
    }

    /// Calculate throughput
    pub fn calculate_throughput(&mut self, elapsed: Duration) {
        if elapsed.as_secs() > 0 {
            self.throughput = self.proposals_committed as f64 / elapsed.as_secs_f64();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new(42);
        assert_eq!(id.inner(), 42);
        assert_eq!(format!("{}", id), "Node(42)");
    }

    #[test]
    fn test_term_ordering() {
        let t1 = Term::new(1);
        let t2 = Term::new(2);
        assert!(t1 < t2);

        let mut t3 = Term::new(5);
        t3.increment();
        assert_eq!(t3.inner(), 6);
    }

    #[test]
    fn test_log_index() {
        let mut idx = LogIndex::new(10);
        idx.increment();
        assert_eq!(idx.inner(), 11);
    }

    #[test]
    fn test_view_number_primary() {
        let view = ViewNumber::new(5);
        let primary = view.primary(4);
        assert_eq!(primary.inner(), 1); // 5 % 4 = 1
    }

    #[test]
    fn test_consensus_metrics() {
        let mut metrics = ConsensusMetrics::new();

        metrics.record_proposal(10.0, true);
        assert_eq!(metrics.proposals_submitted, 1);
        assert_eq!(metrics.proposals_committed, 1);
        assert_eq!(metrics.avg_latency_ms, 10.0);

        metrics.record_proposal(20.0, true);
        assert_eq!(metrics.proposals_submitted, 2);
        assert_eq!(metrics.avg_latency_ms, 15.0);

        metrics.record_leader_election();
        assert_eq!(metrics.leader_elections, 1);
    }
}
