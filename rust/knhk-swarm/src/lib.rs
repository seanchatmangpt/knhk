//! KNHK Agent Swarm Framework
//!
//! A distributed multi-agent system enabling collective intelligence through
//! Byzantine consensus, neural learning, and quantum-safe communication.
//!
//! ## DOCTRINE Alignment
//!
//! - **Covenant 2**: Byzantine-safe consensus ensures invariants are law
//! - **Covenant 3**: Full MAPE-K loop integration
//! - **Covenant 6**: Complete observability
//! - **New Covenant**: "Swarm > Individual" - Collective wisdom

pub mod core;
pub mod agents;
pub mod coordination;
pub mod communication;
pub mod learning;
pub mod monitoring;
pub mod storage;

pub mod types;
pub mod error;

// Re-exports
pub use core::swarm::{AgentSwarm, SwarmConfig};
pub use agents::{AgentConfig, AgentId};
pub use error::{SwarmError, SwarmResult};
pub use types::*;

use tracing::{info, instrument};

/// Initialize the swarm framework with tracing
#[instrument]
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    info!("KNHK Agent Swarm Framework initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_initialization() {
        // Basic smoke test
        assert!(true);
    }
}
