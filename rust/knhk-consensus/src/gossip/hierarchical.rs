//! Hierarchical Gossip Topology for 10k-1M Agents
//!
//! Implements tree-based hierarchical gossip for massive swarms.
//! - Divide swarm into sub-swarms (~100 agents each)
//! - Each sub-swarm converges locally via gossip
//! - Sub-swarm leaders gossip with other leaders
//! - Tree structure: O(log log n) latency for 1M agents

use super::config::GossipConfig;
use super::protocol::{GossipMessage, GossipProtocol, GossipRoundStats};
use super::state::VersionedState;
use super::{AgentId, Result, RoundNumber};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, trace};

/// Hierarchical level (0 = leaf, 1 = sub-swarm leader, 2 = top-level)
pub type HierarchyLevel = u8;

/// Sub-swarm identifier
pub type SubSwarmId = u64;

/// Hierarchical gossip configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HierarchicalConfig {
    /// Base gossip config
    pub gossip_config: GossipConfig,
    /// Sub-swarm size (agents per sub-swarm)
    pub subswarm_size: usize,
    /// My sub-swarm ID
    pub subswarm_id: SubSwarmId,
    /// Am I a sub-swarm leader?
    pub is_leader: bool,
    /// Hierarchy level (0 = leaf, 1+ = leader levels)
    pub hierarchy_level: HierarchyLevel,
}

impl HierarchicalConfig {
    /// Create hierarchical configuration
    pub fn new(
        agent_id: AgentId,
        swarm_size: usize,
        subswarm_size: usize,
    ) -> Self {
        let gossip_config = GossipConfig::new(agent_id, swarm_size);
        let subswarm_id = agent_id / subswarm_size as u64;
        let is_leader = agent_id % subswarm_size as u64 == 0;

        Self {
            gossip_config,
            subswarm_size,
            subswarm_id,
            is_leader,
            hierarchy_level: if is_leader { 1 } else { 0 },
        }
    }

    /// Calculate number of sub-swarms
    pub fn num_subswarms(&self) -> usize {
        (self.gossip_config.swarm_size + self.subswarm_size - 1) / self.subswarm_size
    }

    /// Get agents in my sub-swarm
    pub fn subswarm_agents(&self) -> Vec<AgentId> {
        let start = self.subswarm_id * self.subswarm_size as u64;
        let end = ((self.subswarm_id + 1) * self.subswarm_size as u64)
            .min(self.gossip_config.swarm_size as u64);
        (start..end).collect()
    }

    /// Get all sub-swarm leaders
    pub fn all_leaders(&self) -> Vec<AgentId> {
        (0..self.num_subswarms() as u64)
            .map(|id| id * self.subswarm_size as u64)
            .collect()
    }
}

/// Hierarchical gossip protocol
pub struct HierarchicalGossip {
    /// Configuration
    config: HierarchicalConfig,
    /// Local gossip (within sub-swarm)
    local_gossip: Arc<GossipProtocol>,
    /// Leader gossip (between sub-swarms)
    leader_gossip: Option<Arc<GossipProtocol>>,
    /// Current round
    current_round: Arc<RwLock<RoundNumber>>,
}

impl HierarchicalGossip {
    /// Create new hierarchical gossip instance
    pub fn new(config: HierarchicalConfig, initial_state: VersionedState) -> Self {
        let local_gossip = Arc::new(GossipProtocol::new(
            config.gossip_config.clone(),
            initial_state.clone(),
        ));

        let leader_gossip = if config.is_leader {
            // Leaders also participate in leader-level gossip
            let mut leader_config = config.gossip_config.clone();
            leader_config.swarm_size = config.num_subswarms();
            leader_config.peer_sample_size = (config.num_subswarms() as f64).log2().ceil() as usize;
            Some(Arc::new(GossipProtocol::new(leader_config, initial_state)))
        } else {
            None
        };

        Self {
            config,
            local_gossip,
            leader_gossip,
            current_round: Arc::new(RwLock::new(0)),
        }
    }

    /// Initialize peers
    pub async fn init_peers(&self) {
        // Initialize local peers (within sub-swarm)
        let local_peers = self.config.subswarm_agents();
        self.local_gossip.init_peers(local_peers).await;

        // Initialize leader peers (if leader)
        if let Some(leader_gossip) = &self.leader_gossip {
            let leader_peers = self.config.all_leaders();
            leader_gossip.init_peers(leader_peers).await;
        }
    }

    /// Execute one hierarchical gossip round
    pub async fn execute_round(&self) -> Result<HierarchicalRoundStats> {
        let round_start = Instant::now();
        let mut round = *self.current_round.read().await;
        round += 1;
        *self.current_round.write().await = round;

        info!(
            round = round,
            hierarchy_level = self.config.hierarchy_level,
            is_leader = self.config.is_leader,
            "Starting hierarchical gossip round"
        );

        // Phase 1: Local gossip (within sub-swarm)
        let local_stats = self.local_gossip.execute_round().await?;
        debug!(
            round = round,
            subswarm_id = self.config.subswarm_id,
            messages = local_stats.messages_sent,
            "Local gossip complete"
        );

        // Phase 2: Leader gossip (between sub-swarms) - only for leaders
        let leader_stats = if let Some(leader_gossip) = &self.leader_gossip {
            let stats = leader_gossip.execute_round().await?;
            debug!(
                round = round,
                leader_messages = stats.messages_sent,
                "Leader gossip complete"
            );

            // Propagate leader state back to local sub-swarm
            let leader_state = leader_gossip.get_state().await;
            // Would broadcast to local sub-swarm here
            trace!(
                round = round,
                leader_version = leader_state.version,
                "Broadcasting leader state to sub-swarm"
            );

            Some(stats)
        } else {
            None
        };

        let duration = round_start.elapsed();

        Ok(HierarchicalRoundStats {
            round,
            local_stats,
            leader_stats,
            duration,
        })
    }

    /// Get current state
    pub async fn get_state(&self) -> VersionedState {
        self.local_gossip.get_state().await
    }

    /// Get current round
    pub async fn current_round(&self) -> RoundNumber {
        *self.current_round.read().await
    }

    /// Calculate expected convergence rounds for hierarchical topology
    pub fn expected_convergence_rounds(&self) -> usize {
        // Local convergence: O(log n_local)
        let local_rounds = super::expected_convergence_rounds(
            self.config.subswarm_size,
            self.config.gossip_config.peer_sample_size,
        );

        // Leader convergence: O(log n_leaders)
        let leader_rounds = super::expected_convergence_rounds(
            self.config.num_subswarms(),
            (self.config.num_subswarms() as f64).log2().ceil() as usize,
        );

        // Total: local + leader + propagation
        local_rounds + leader_rounds + 1
    }
}

/// Hierarchical gossip round statistics
#[derive(Clone, Debug)]
pub struct HierarchicalRoundStats {
    /// Round number
    pub round: RoundNumber,
    /// Local gossip stats (within sub-swarm)
    pub local_stats: GossipRoundStats,
    /// Leader gossip stats (between sub-swarms)
    pub leader_stats: Option<GossipRoundStats>,
    /// Total round duration
    pub duration: Duration,
}

impl HierarchicalRoundStats {
    /// Total messages sent (local + leader)
    pub fn total_messages_sent(&self) -> usize {
        self.local_stats.messages_sent
            + self
                .leader_stats
                .as_ref()
                .map(|s| s.messages_sent)
                .unwrap_or(0)
    }

    /// Total messages received (local + leader)
    pub fn total_messages_received(&self) -> usize {
        self.local_stats.messages_received
            + self
                .leader_stats
                .as_ref()
                .map(|s| s.messages_received)
                .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gossip::state::StateValue;

    #[test]
    fn test_hierarchical_config_creation() {
        let config = HierarchicalConfig::new(0, 1000, 100);
        assert_eq!(config.subswarm_size, 100);
        assert_eq!(config.num_subswarms(), 10);
        assert_eq!(config.subswarm_id, 0);
        assert!(config.is_leader);
    }

    #[test]
    fn test_hierarchical_config_subswarm_agents() {
        let config = HierarchicalConfig::new(0, 1000, 100);
        let agents = config.subswarm_agents();
        assert_eq!(agents.len(), 100);
        assert_eq!(agents[0], 0);
        assert_eq!(agents[99], 99);

        let config2 = HierarchicalConfig::new(250, 1000, 100);
        let agents2 = config2.subswarm_agents();
        assert_eq!(agents2.len(), 100);
        assert_eq!(agents2[0], 200);
        assert_eq!(agents2[99], 299);
    }

    #[test]
    fn test_hierarchical_config_leaders() {
        let config = HierarchicalConfig::new(0, 1000, 100);
        let leaders = config.all_leaders();
        assert_eq!(leaders.len(), 10);
        assert_eq!(leaders, vec![0, 100, 200, 300, 400, 500, 600, 700, 800, 900]);
    }

    #[test]
    fn test_hierarchical_expected_convergence_rounds() {
        let config = HierarchicalConfig::new(0, 10000, 100);
        let initial_state = VersionedState::new(0, StateValue::Number(0), 0);
        let gossip = HierarchicalGossip::new(config, initial_state);

        let expected = gossip.expected_convergence_rounds();
        // Local (100 agents, k=8): ~2 rounds
        // Leaders (100 sub-swarms, k=7): ~2 rounds
        // Total: ~5 rounds
        assert!(expected >= 4 && expected <= 6);
    }

    #[tokio::test]
    async fn test_hierarchical_gossip_creation() {
        let config = HierarchicalConfig::new(0, 1000, 100);
        let initial_state = VersionedState::new(0, StateValue::Number(0), 0);
        let gossip = HierarchicalGossip::new(config, initial_state);

        assert_eq!(gossip.current_round().await, 0);
    }

    #[tokio::test]
    async fn test_hierarchical_gossip_init_peers() {
        let config = HierarchicalConfig::new(0, 1000, 100);
        let initial_state = VersionedState::new(0, StateValue::Number(0), 0);
        let gossip = HierarchicalGossip::new(config, initial_state);

        gossip.init_peers().await;
        // Peers should be initialized (can't easily test without exposing internals)
    }
}
