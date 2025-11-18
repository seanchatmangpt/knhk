//! Gossip Protocol Configuration

use super::{AgentId, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Gossip protocol configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GossipConfig {
    /// Agent ID
    pub agent_id: AgentId,
    /// Total number of agents in swarm
    pub swarm_size: usize,
    /// Peer sample size (k parameter)
    pub peer_sample_size: usize,
    /// Gossip round interval
    pub round_interval: Duration,
    /// Network timeout for peer communication
    pub network_timeout: Duration,
    /// Enable Byzantine detection
    pub byzantine_detection: bool,
    /// Use hierarchical topology for large swarms
    pub hierarchical: bool,
    /// Hierarchical topology: agents per sub-swarm
    pub subswarm_size: usize,
    /// Enable topology optimization (latency-aware peer selection)
    pub topology_optimization: bool,
    /// Maximum Byzantine tolerance (f < n/3)
    pub max_byzantine_faults: usize,
}

impl GossipConfig {
    /// Create new gossip configuration
    pub fn new(agent_id: AgentId, swarm_size: usize) -> Self {
        let peer_sample_size = Self::default_peer_sample_size(swarm_size);
        let max_byzantine_faults = (swarm_size - 1) / 3;

        Self {
            agent_id,
            swarm_size,
            peer_sample_size,
            round_interval: Duration::from_millis(50),
            network_timeout: Duration::from_millis(100),
            byzantine_detection: true,
            hierarchical: swarm_size > 1000,
            subswarm_size: 100,
            topology_optimization: true,
            max_byzantine_faults,
        }
    }

    /// Default peer sample size based on swarm size
    fn default_peer_sample_size(swarm_size: usize) -> usize {
        match swarm_size {
            0..=10 => 3,
            11..=100 => 5,
            101..=1000 => 8,
            1001..=10000 => 10,
            10001..=100000 => 12,
            _ => 15,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.swarm_size < 3 {
            return Err(crate::ConsensusError::InvalidValidatorSet(
                "Swarm must have at least 3 agents".to_string(),
            ));
        }

        if self.peer_sample_size == 0 {
            return Err(crate::ConsensusError::InvalidValidatorSet(
                "Peer sample size must be > 0".to_string(),
            ));
        }

        if self.peer_sample_size >= self.swarm_size {
            return Err(crate::ConsensusError::InvalidValidatorSet(
                "Peer sample size must be < swarm size".to_string(),
            ));
        }

        if self.max_byzantine_faults >= self.swarm_size / 3 {
            return Err(crate::ConsensusError::InvalidValidatorSet(format!(
                "Byzantine fault tolerance requires f < n/3, but {} >= {}/3",
                self.max_byzantine_faults, self.swarm_size
            )));
        }

        Ok(())
    }

    /// Expected convergence rounds
    pub fn expected_convergence_rounds(&self) -> usize {
        super::expected_convergence_rounds(self.swarm_size, self.peer_sample_size)
    }

    /// Maximum Byzantine tolerance
    pub fn max_byzantine_tolerance(&self) -> usize {
        super::max_byzantine_tolerance(self.swarm_size)
    }
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self::new(1, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_peer_sample_size() {
        assert_eq!(GossipConfig::default_peer_sample_size(10), 3);
        assert_eq!(GossipConfig::default_peer_sample_size(100), 5);
        assert_eq!(GossipConfig::default_peer_sample_size(1000), 8);
        assert_eq!(GossipConfig::default_peer_sample_size(10000), 10);
        assert_eq!(GossipConfig::default_peer_sample_size(100000), 12);
        assert_eq!(GossipConfig::default_peer_sample_size(1000000), 15);
    }

    #[test]
    fn test_config_validation() {
        let config = GossipConfig::new(1, 10);
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.swarm_size = 2;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.peer_sample_size = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_expected_convergence_rounds() {
        let config = GossipConfig::new(1, 1000);
        assert_eq!(config.expected_convergence_rounds(), 3); // log_8(1000) ≈ 3
    }
}
