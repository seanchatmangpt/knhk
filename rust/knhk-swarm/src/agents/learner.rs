//! Learner Agent - Runs neural models and discovers patterns

use super::{Agent, AgentConfig};
use crate::error::SwarmResult;
use crate::types::{AgentId, AgentRole, AgentState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

/// Learning recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: String,
    pub confidence: f64,
    pub suggestion: String,
}

/// The Learner Agent
pub struct LearnerAgent {
    id: AgentId,
    config: AgentConfig,
    state: Arc<RwLock<AgentState>>,
    recommendations: Arc<RwLock<Vec<Recommendation>>>,
}

impl LearnerAgent {
    /// Create a new Learner agent
    #[instrument]
    pub fn new(id: AgentId, config: AgentConfig) -> SwarmResult<Self> {
        info!("Creating Learner agent: {}", id);

        Ok(Self {
            id,
            config,
            state: Arc::new(RwLock::new(AgentState::Learning)),
            recommendations: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Generate a recommendation
    pub async fn generate_recommendation(&self, rec: Recommendation) {
        self.recommendations.write().await.push(rec.clone());
        info!(
            "Learner {} generated recommendation: {} (confidence: {:.2})",
            self.id, rec.category, rec.confidence
        );
    }

    /// Get all recommendations
    pub async fn get_recommendations(&self) -> Vec<Recommendation> {
        self.recommendations.read().await.clone()
    }
}

impl Agent for LearnerAgent {
    fn id(&self) -> AgentId {
        self.id
    }

    fn role(&self) -> AgentRole {
        self.config.role
    }

    fn state(&self) -> AgentState {
        AgentState::Learning
    }

    async fn start(&mut self) {
        *self.state.write().await = AgentState::Learning;
        info!("Learner agent {} started", self.id);
    }

    async fn stop(&mut self) {
        *self.state.write().await = AgentState::Idle;
        info!("Learner agent {} stopped", self.id);
    }

    async fn health_check(&self) -> bool {
        let state = self.state.read().await;
        !matches!(*state, AgentState::Failed)
    }
}
