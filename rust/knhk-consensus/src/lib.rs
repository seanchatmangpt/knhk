//! Byzantine Fault-Tolerant Consensus Engine
//!
//! Phase 8: Multi-region consensus with PBFT and HotStuff implementations
//! Provides deterministic state machine replication for KNHK distributed deployments
//!
//! # Architecture
//!
//! - **PBFT (Practical Byzantine Fault Tolerance)**: Classical consensus with 3f+1 tolerance
//! - **HotStuff**: Modern leader-based consensus with linear communication complexity
//! - **State Machine Replication**: Deterministic command log with snapshotting
//! - **Network Layer**: P2P messaging with Byzantine sender detection
//! - **Validator Management**: Dynamic validator sets with reputation tracking

#![warn(missing_docs)]
#![warn(unused_crate_dependencies)]

pub mod pbft;
pub mod hotstuff;
pub mod state;
pub mod network;
pub mod validator;

pub use pbft::{BFTMessage, PBFTNode, PBFTConfig};
pub use hotstuff::{HotStuffNode, HotStuffConfig, ViewNumber};
pub use state::{StateMachineReplicator, CommandLog};
pub use network::{NetworkNode, PeerMessage, PeerDiscovery};
pub use validator::{ValidatorSet, ValidatorMetrics};

use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, warn};

/// Consensus module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum number of Byzantine nodes that can be tolerated
pub fn max_byzantine_tolerance(total_nodes: usize) -> usize {
    (total_nodes - 1) / 3
}

/// Consensus errors
#[derive(Debug, Error)]
pub enum ConsensusError {
    /// Quorum not reached
    #[error("Quorum not reached: {0}/{1}")]
    QuorumNotReached(usize, usize),

    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,

    /// State mismatch between replicas
    #[error("State mismatch: expected {expected}, got {actual}")]
    StateMismatch { expected: String, actual: String },

    /// Byzantine node detected
    #[error("Byzantine node detected: {0}")]
    ByzantineNodeDetected(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),

    /// Invalid validator set
    #[error("Invalid validator set: {0}")]
    InvalidValidatorSet(String),

    /// View synchronization timeout
    #[error("View synchronization timeout")]
    ViewSyncTimeout,

    /// Command log error
    #[error("Command log error: {0}")]
    CommandLogError(String),
}

/// Result type for consensus operations
pub type Result<T> = std::result::Result<T, ConsensusError>;

/// Consensus quality attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualityAttributes {
    /// Byzantine fault tolerance (f < n/3)
    pub byzantine_tolerance: bool,
    /// Deterministic state commitment
    pub deterministic_commit: bool,
    /// Message complexity is O(n²) for PBFT, O(n) for HotStuff
    pub linear_communication: bool,
    /// Latency in milliseconds
    pub commit_latency_ms: u64,
}

/// Consensus algorithm type
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsensusAlgorithm {
    /// Raft consensus (for crash-fault tolerance)
    Raft,
    /// PBFT (Practical Byzantine Fault Tolerance)
    PBFT,
    /// HotStuff (optimistic responsiveness)
    HotStuff,
}

/// Phase 8 Consensus Configuration
#[derive(Clone, Debug)]
pub struct ConsensusConfig {
    /// Selected consensus algorithm
    pub algorithm: ConsensusAlgorithm,
    /// Total number of nodes in cluster
    pub total_nodes: usize,
    /// Maximum Byzantine faulty nodes (f < n/3 for PBFT/HotStuff)
    pub max_faults: usize,
    /// Enable Byzantine fault detection
    pub detect_byzantine_faults: bool,
    /// Enable multi-region replication
    pub multi_region_enabled: bool,
    /// Batch timeout for consensus (ms)
    pub batch_timeout_ms: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            algorithm: ConsensusAlgorithm::Raft,
            total_nodes: 5,
            max_faults: 1,
            detect_byzantine_faults: true,
            multi_region_enabled: true,
            batch_timeout_ms: 1000,
        }
    }
}

impl ConsensusConfig {
    /// Validate configuration constraints
    pub fn validate(&self) -> Result<(), String> {
        if self.total_nodes < 3 {
            return Err("Cluster must have at least 3 nodes".to_string());
        }

        // For PBFT/HotStuff: f < n/3
        if self.max_faults >= self.total_nodes / 3 {
            return Err(format!(
                "Byzantine fault tolerance requires f < n/3, but {} >= {}/3",
                self.max_faults, self.total_nodes
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_config_defaults() {
        let config = ConsensusConfig::default();
        assert_eq!(config.algorithm, ConsensusAlgorithm::Raft);
        assert_eq!(config.total_nodes, 5);
        assert_eq!(config.max_faults, 1);
        assert!(config.detect_byzantine_faults);
    }

    #[test]
    fn test_consensus_config_validation() {
        let mut config = ConsensusConfig::default();
        assert!(config.validate().is_ok());

        config.total_nodes = 2;
        assert!(config.validate().is_err());

        config.total_nodes = 5;
        config.max_faults = 2;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_phase_8_prelude_imports() {
        // Verify all public types are accessible
        let _cfg = ConsensusConfig::default();
        let _algo = ConsensusAlgorithm::PBFT;
    }
}
