//! Scout Agent - Gathers information and detects patterns

use super::{Agent, AgentConfig};
use crate::error::SwarmResult;
use crate::types::{AgentId, AgentRole, AgentState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

/// Pattern detected by scout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_type: String,
    pub confidence: f64,
    pub description: String,
}

/// The Scout Agent
pub struct ScoutAgent {
    id: AgentId,
    config: AgentConfig,
    state: Arc<RwLock<AgentState>>,
    detected_patterns: Arc<RwLock<Vec<DetectedPattern>>>,
}

impl ScoutAgent {
    /// Create a new Scout agent
    #[instrument]
    pub fn new(id: AgentId, config: AgentConfig) -> SwarmResult<Self> {
        info!("Creating Scout agent: {}", id);

        Ok(Self {
            id,
            config,
            state: Arc::new(RwLock::new(AgentState::Idle)),
            detected_patterns: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Report a detected pattern
    pub async fn report_pattern(&self, pattern: DetectedPattern) {
        self.detected_patterns.write().await.push(pattern.clone());
        info!(
            "Scout {} detected pattern: {}",
            self.id, pattern.pattern_type
        );
    }

    /// Get all detected patterns
    pub async fn get_patterns(&self) -> Vec<DetectedPattern> {
        self.detected_patterns.read().await.clone()
    }
}

impl Agent for ScoutAgent {
    fn id(&self) -> AgentId {
        self.id
    }

    fn role(&self) -> AgentRole {
        self.config.role
    }

    fn state(&self) -> AgentState {
        AgentState::Working
    }

    async fn start(&mut self) {
        *self.state.write().await = AgentState::Working;
        info!("Scout agent {} started", self.id);
    }

    async fn stop(&mut self) {
        *self.state.write().await = AgentState::Idle;
        info!("Scout agent {} stopped", self.id);
    }

    async fn health_check(&self) -> bool {
        let state = self.state.read().await;
        !matches!(*state, AgentState::Failed)
    }
}
