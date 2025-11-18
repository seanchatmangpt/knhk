//! Agent implementations

pub mod queen;
pub mod worker;
pub mod scout;
pub mod guardian;
pub mod learner;

pub use queen::QueenAgent;
pub use worker::WorkerAgent;
pub use scout::ScoutAgent;
pub use guardian::GuardianAgent;
pub use learner::LearnerAgent;

use crate::types::{AgentId, AgentRole, AgentState};
use serde::{Deserialize, Serialize};

/// Configuration for spawning a new agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub role: AgentRole,
    pub initial_state: AgentState,
    pub capabilities: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            role: AgentRole::Worker,
            initial_state: AgentState::Idle,
            capabilities: vec![],
        }
    }
}

/// Base trait for all agents
#[allow(async_fn_in_trait)]
pub trait Agent: Send + Sync {
    fn id(&self) -> AgentId;
    fn role(&self) -> AgentRole;
    fn state(&self) -> AgentState;

    async fn start(&mut self);
    async fn stop(&mut self);
    async fn health_check(&self) -> bool;
}
