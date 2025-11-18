//! Gossip-Based Consensus Protocol for Massive AI Agent Swarms
//!
//! Implements epidemic dissemination with Byzantine-robust voting for scaling
//! to 10k-1M agents where traditional Byzantine consensus (PBFT/HotStuff) doesn't scale.
//!
//! # Architecture
//!
//! - **Epidemic Dissemination**: Information spreads in O(log n) rounds
//! - **Byzantine-Robust Voting**: Merkle-tree based proofs with >2f+1 majority
//! - **Hierarchical Topology**: Tree structure for 10k+ agents
//! - **State Machine Replication**: Versioned state with conflict resolution
//!
//! # DOCTRINE Alignment
//!
//! - **O (Observability)**: All gossip messages observable via OTEL
//! - **Σ (State Machine)**: Deterministic state agreement protocol
//! - **Q (Chatman Constant)**: ≤8 ticks hot path (hash comparison), async off-path (state merge)
//!
//! # Performance Targets
//!
//! | Swarm Size | Peers (k) | Rounds | Latency | Throughput |
//! |-----------|-----------|--------|---------|-----------|
//! | 10        | 3         | 3      | <10ms   | >10k msg/s |
//! | 100       | 5         | 7      | <50ms   | >50k msg/s |
//! | 1k        | 8         | 10     | <100ms  | >100k msg/s |
//! | 10k       | 10        | 14     | <250ms  | >500k msg/s |
//! | 100k      | 12        | 17     | <500ms  | >1M msg/s |
//! | 1M        | 15        | 20     | <1s     | >1M msg/s |

pub mod config;
pub mod convergence;
pub mod hierarchical;
pub mod merkle;
pub mod protocol;
pub mod state;
pub mod topology;

pub use config::GossipConfig;
pub use convergence::ConvergenceTracker;
pub use hierarchical::HierarchicalGossip;
pub use merkle::{MerkleProof, StateProof};
pub use protocol::GossipProtocol;
pub use state::{StateValue, VersionedState};
pub use topology::{PeerSampler, TopologyOptimizer};

use crate::ConsensusError;
use blake3::Hash as Blake3Hash;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, warn};

/// Agent identifier (UUID or numeric ID)
pub type AgentId = u64;

/// Timestamp in milliseconds since epoch
pub type Timestamp = u64;

/// Gossip round number (monotonically increasing)
pub type RoundNumber = u64;

/// Gossip protocol result type
pub type Result<T> = std::result::Result<T, ConsensusError>;

/// Maximum number of Byzantine agents that can be tolerated
pub fn max_byzantine_tolerance(total_agents: usize) -> usize {
    (total_agents - 1) / 3
}

/// Calculate expected convergence rounds for swarm size
pub fn expected_convergence_rounds(swarm_size: usize, peer_sample_size: usize) -> usize {
    // O(log n) convergence with base = peer_sample_size
    if swarm_size <= peer_sample_size {
        return 1;
    }
    let log_n = (swarm_size as f64).log(peer_sample_size as f64);
    log_n.ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byzantine_tolerance() {
        assert_eq!(max_byzantine_tolerance(4), 1); // 4 nodes, f=1
        assert_eq!(max_byzantine_tolerance(7), 2); // 7 nodes, f=2
        assert_eq!(max_byzantine_tolerance(10), 3); // 10 nodes, f=3
        assert_eq!(max_byzantine_tolerance(1000), 333); // 1000 nodes, f=333
    }

    #[test]
    fn test_expected_convergence_rounds() {
        // k=3 peers
        assert_eq!(expected_convergence_rounds(10, 3), 3);
        assert_eq!(expected_convergence_rounds(100, 3), 5);
        assert_eq!(expected_convergence_rounds(1000, 3), 7);

        // k=10 peers
        assert_eq!(expected_convergence_rounds(100, 10), 2);
        assert_eq!(expected_convergence_rounds(1000, 10), 3);
        assert_eq!(expected_convergence_rounds(10000, 10), 4);
    }
}
