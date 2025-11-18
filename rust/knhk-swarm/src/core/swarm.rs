//! Core swarm engine
//!
//! DOCTRINE Alignment:
//! - Covenant 2: Byzantine-safe consensus ensures invariants
//! - Covenant 3: MAPE-K loop integration
//! - Covenant 6: Full observability

use crate::agents::{
    Agent, AgentConfig, GuardianAgent, LearnerAgent, QueenAgent, ScoutAgent, WorkerAgent,
};
use crate::coordination::{ByzantineConsensus, SwarmCoordinator};
use crate::error::{SwarmError, SwarmResult};
use crate::learning::FederatedLearner;
use crate::monitoring::SwarmHealthMonitor;
use crate::types::{AgentId, AgentRole, ByzantineParams};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};

/// Configuration for the swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    /// Total number of agents in swarm
    pub total_agents: usize,
    /// Maximum Byzantine failures tolerated
    pub max_byzantine_failures: usize,
    /// Consensus timeout in milliseconds
    pub consensus_timeout_ms: u64,
    /// Enable federated learning
    pub enable_learning: bool,
    /// Enable health monitoring
    pub enable_monitoring: bool,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            total_agents: 7, // Allows f=2 Byzantine failures
            max_byzantine_failures: 2,
            consensus_timeout_ms: 5000,
            enable_learning: true,
            enable_monitoring: true,
        }
    }
}

impl SwarmConfig {
    /// Validate configuration
    pub fn validate(&self) -> SwarmResult<()> {
        // Byzantine fault tolerance requires n >= 3f + 1
        let min_agents = 3 * self.max_byzantine_failures + 1;
        if self.total_agents < min_agents {
            return Err(SwarmError::InvalidConfig(format!(
                "total_agents ({}) must be >= 3f+1 ({}) for f={} Byzantine failures",
                self.total_agents, min_agents, self.max_byzantine_failures
            )));
        }
        Ok(())
    }

    /// Get Byzantine parameters
    pub fn byzantine_params(&self) -> ByzantineParams {
        ByzantineParams::new(self.total_agents)
    }
}

/// The main swarm orchestrator
pub struct AgentSwarm {
    config: SwarmConfig,
    queen_agent: Arc<RwLock<QueenAgent>>,
    worker_agents: Arc<RwLock<HashMap<AgentId, WorkerAgent>>>,
    scout_agents: Arc<RwLock<HashMap<AgentId, ScoutAgent>>>,
    guardian_agents: Arc<RwLock<HashMap<AgentId, GuardianAgent>>>,
    learner_agents: Arc<RwLock<HashMap<AgentId, LearnerAgent>>>,
    coordinator: Arc<SwarmCoordinator>,
    consensus: Arc<ByzantineConsensus>,
    federated_learner: Option<Arc<RwLock<FederatedLearner>>>,
    health_monitor: Option<Arc<RwLock<SwarmHealthMonitor>>>,
    next_agent_id: Arc<RwLock<u64>>,
}

impl AgentSwarm {
    /// Create a new swarm
    #[instrument]
    pub async fn new(config: SwarmConfig) -> SwarmResult<Self> {
        info!("Creating new agent swarm with config: {:?}", config);

        // Validate configuration
        config.validate()?;

        // Create queen agent
        let queen_config = AgentConfig {
            role: AgentRole::Queen,
            ..Default::default()
        };
        let queen_agent = QueenAgent::new(AgentId(0), queen_config)?;

        // Create consensus engine
        let byzantine_params = config.byzantine_params();
        let consensus = ByzantineConsensus::new(byzantine_params)?;

        // Create coordinator
        let coordinator = SwarmCoordinator::new(Arc::new(consensus.clone()))?;

        // Create optional components
        let federated_learner = if config.enable_learning {
            Some(Arc::new(RwLock::new(FederatedLearner::new()?)))
        } else {
            None
        };

        let health_monitor = if config.enable_monitoring {
            Some(Arc::new(RwLock::new(SwarmHealthMonitor::new())))
        } else {
            None
        };

        let swarm = Self {
            config,
            queen_agent: Arc::new(RwLock::new(queen_agent)),
            worker_agents: Arc::new(RwLock::new(HashMap::new())),
            scout_agents: Arc::new(RwLock::new(HashMap::new())),
            guardian_agents: Arc::new(RwLock::new(HashMap::new())),
            learner_agents: Arc::new(RwLock::new(HashMap::new())),
            coordinator: Arc::new(coordinator),
            consensus: Arc::new(consensus),
            federated_learner,
            health_monitor,
            next_agent_id: Arc::new(RwLock::new(1)), // Queen is 0
        };

        info!("Agent swarm created successfully");
        Ok(swarm)
    }

    /// Spawn a worker agent
    #[instrument(skip(self))]
    pub async fn spawn_worker_agent(&self, config: AgentConfig) -> SwarmResult<AgentId> {
        let mut next_id = self.next_agent_id.write().await;
        let agent_id = AgentId(*next_id);
        *next_id += 1;
        drop(next_id);

        info!("Spawning worker agent: {}", agent_id);

        let agent = WorkerAgent::new(agent_id, config)?;
        self.worker_agents.write().await.insert(agent_id, agent);

        info!("Worker agent {} spawned successfully", agent_id);
        Ok(agent_id)
    }

    /// Spawn a scout agent
    #[instrument(skip(self))]
    pub async fn spawn_scout_agent(&self, config: AgentConfig) -> SwarmResult<AgentId> {
        let mut next_id = self.next_agent_id.write().await;
        let agent_id = AgentId(*next_id);
        *next_id += 1;
        drop(next_id);

        info!("Spawning scout agent: {}", agent_id);

        let agent = ScoutAgent::new(agent_id, config)?;
        self.scout_agents.write().await.insert(agent_id, agent);

        info!("Scout agent {} spawned successfully", agent_id);
        Ok(agent_id)
    }

    /// Spawn a guardian agent
    #[instrument(skip(self))]
    pub async fn spawn_guardian_agent(&self, config: AgentConfig) -> SwarmResult<AgentId> {
        let mut next_id = self.next_agent_id.write().await;
        let agent_id = AgentId(*next_id);
        *next_id += 1;
        drop(next_id);

        info!("Spawning guardian agent: {}", agent_id);

        let agent = GuardianAgent::new(agent_id, config)?;
        self.guardian_agents.write().await.insert(agent_id, agent);

        info!("Guardian agent {} spawned successfully", agent_id);
        Ok(agent_id)
    }

    /// Spawn a learner agent
    #[instrument(skip(self))]
    pub async fn spawn_learner_agent(&self, config: AgentConfig) -> SwarmResult<AgentId> {
        let mut next_id = self.next_agent_id.write().await;
        let agent_id = AgentId(*next_id);
        *next_id += 1;
        drop(next_id);

        info!("Spawning learner agent: {}", agent_id);

        let agent = LearnerAgent::new(agent_id, config)?;
        self.learner_agents.write().await.insert(agent_id, agent);

        info!("Learner agent {} spawned successfully", agent_id);
        Ok(agent_id)
    }

    /// Get total number of active agents
    pub async fn agent_count(&self) -> usize {
        let workers = self.worker_agents.read().await.len();
        let scouts = self.scout_agents.read().await.len();
        let guardians = self.guardian_agents.read().await.len();
        let learners = self.learner_agents.read().await.len();
        1 + workers + scouts + guardians + learners // +1 for queen
    }

    /// Perform health check on the swarm
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> SwarmHealth {
        let total_agents = self.agent_count().await;

        // Check queen
        let queen = self.queen_agent.read().await;
        let queen_healthy = queen.health_check().await;
        drop(queen);

        // Count healthy agents
        let mut healthy_count = if queen_healthy { 1 } else { 0 };

        // Check workers
        let workers = self.worker_agents.read().await;
        for agent in workers.values() {
            if agent.health_check().await {
                healthy_count += 1;
            }
        }
        drop(workers);

        // Check scouts
        let scouts = self.scout_agents.read().await;
        for agent in scouts.values() {
            if agent.health_check().await {
                healthy_count += 1;
            }
        }
        drop(scouts);

        // Check guardians
        let guardians = self.guardian_agents.read().await;
        for agent in guardians.values() {
            if agent.health_check().await {
                healthy_count += 1;
            }
        }
        drop(guardians);

        // Check learners
        let learners = self.learner_agents.read().await;
        for agent in learners.values() {
            if agent.health_check().await {
                healthy_count += 1;
            }
        }
        drop(learners);

        let overall_health = if total_agents > 0 {
            healthy_count as f64 / total_agents as f64
        } else {
            0.0
        };

        if overall_health < 0.7 {
            warn!(
                "Swarm health degraded: {}/{} agents healthy ({:.1}%)",
                healthy_count,
                total_agents,
                overall_health * 100.0
            );
        }

        SwarmHealth {
            total_agents,
            healthy_agents: healthy_count,
            overall_health,
            has_quorum: self.consensus.has_quorum(healthy_count).await,
        }
    }

    /// Shutdown the swarm
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> SwarmResult<()> {
        info!("Shutting down swarm...");

        // Stop all agents
        self.queen_agent.write().await.stop().await;

        for agent in self.worker_agents.write().await.values_mut() {
            agent.stop().await;
        }

        for agent in self.scout_agents.write().await.values_mut() {
            agent.stop().await;
        }

        for agent in self.guardian_agents.write().await.values_mut() {
            agent.stop().await;
        }

        for agent in self.learner_agents.write().await.values_mut() {
            agent.stop().await;
        }

        info!("Swarm shut down successfully");
        Ok(())
    }
}

/// Health status of the swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmHealth {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub overall_health: f64,
    pub has_quorum: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_swarm_creation() {
        let config = SwarmConfig::default();
        let swarm = AgentSwarm::new(config).await.unwrap();

        // Should have 1 agent (queen) initially
        assert_eq!(swarm.agent_count().await, 1);
    }

    #[tokio::test]
    async fn test_spawn_worker_agent() {
        let config = SwarmConfig::default();
        let swarm = AgentSwarm::new(config).await.unwrap();

        let agent_config = AgentConfig {
            role: AgentRole::Worker,
            ..Default::default()
        };

        let agent_id = swarm.spawn_worker_agent(agent_config).await.unwrap();
        assert_eq!(agent_id.0, 1); // Queen is 0, first worker is 1

        assert_eq!(swarm.agent_count().await, 2);
    }

    #[tokio::test]
    async fn test_config_validation() {
        // Valid config: n=7, f=2 (7 >= 3*2+1 = 7)
        let valid_config = SwarmConfig {
            total_agents: 7,
            max_byzantine_failures: 2,
            ..Default::default()
        };
        assert!(valid_config.validate().is_ok());

        // Invalid config: n=6, f=2 (6 < 3*2+1 = 7)
        let invalid_config = SwarmConfig {
            total_agents: 6,
            max_byzantine_failures: 2,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = SwarmConfig::default();
        let swarm = AgentSwarm::new(config).await.unwrap();

        let health = swarm.health_check().await;
        assert_eq!(health.total_agents, 1);
        assert_eq!(health.healthy_agents, 1);
        assert_eq!(health.overall_health, 1.0);
    }
}
