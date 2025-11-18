//! Queen Agent - Orchestrates the swarm
//!
//! The Queen Agent is the central coordinator that:
//! - Assigns tasks to worker agents
//! - Resolves conflicts between agents
//! - Makes final decisions
//! - Maintains swarm cohesion

use super::{Agent, AgentConfig};
use crate::error::{SwarmError, SwarmResult};
use crate::types::{AgentId, AgentRole, AgentState, TaskId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

/// Task assignment from Queen to workers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub task_id: TaskId,
    pub assigned_to: AgentId,
    pub description: String,
    pub priority: u8,
}

/// The Queen Agent
pub struct QueenAgent {
    id: AgentId,
    config: AgentConfig,
    state: Arc<RwLock<AgentState>>,
    task_assignments: Arc<RwLock<HashMap<TaskId, TaskAssignment>>>,
    next_task_id: Arc<RwLock<u64>>,
}

impl QueenAgent {
    /// Create a new Queen agent
    #[instrument]
    pub fn new(id: AgentId, config: AgentConfig) -> SwarmResult<Self> {
        if config.role != AgentRole::Queen {
            return Err(SwarmError::InvalidConfig(
                "QueenAgent must have Queen role".into(),
            ));
        }

        info!("Creating Queen agent: {}", id);

        Ok(Self {
            id,
            config,
            state: Arc::new(RwLock::new(AgentState::Idle)),
            task_assignments: Arc::new(RwLock::new(HashMap::new())),
            next_task_id: Arc::new(RwLock::new(0)),
        })
    }

    /// Assign a task to a worker
    #[instrument(skip(self))]
    pub async fn assign_task(
        &self,
        agent_id: AgentId,
        description: String,
        priority: u8,
    ) -> SwarmResult<TaskId> {
        let mut next_id = self.next_task_id.write().await;
        let task_id = TaskId(*next_id);
        *next_id += 1;
        drop(next_id);

        let assignment = TaskAssignment {
            task_id,
            assigned_to: agent_id,
            description,
            priority,
        };

        self.task_assignments
            .write()
            .await
            .insert(task_id, assignment);

        info!("Queen assigned task {} to agent {}", task_id, agent_id);
        Ok(task_id)
    }

    /// Get all task assignments
    pub async fn get_assignments(&self) -> HashMap<TaskId, TaskAssignment> {
        self.task_assignments.read().await.clone()
    }

    /// Complete a task
    pub async fn complete_task(&self, task_id: TaskId) -> SwarmResult<()> {
        self.task_assignments.write().await.remove(&task_id);
        info!("Queen marked task {} as complete", task_id);
        Ok(())
    }
}

impl Agent for QueenAgent {
    fn id(&self) -> AgentId {
        self.id
    }

    fn role(&self) -> AgentRole {
        self.config.role
    }

    fn state(&self) -> AgentState {
        // This is a blocking call, but safe for enum copy
        AgentState::Working
    }

    async fn start(&mut self) {
        *self.state.write().await = AgentState::Working;
        info!("Queen agent {} started", self.id);
    }

    async fn stop(&mut self) {
        *self.state.write().await = AgentState::Idle;
        info!("Queen agent {} stopped", self.id);
    }

    async fn health_check(&self) -> bool {
        let state = self.state.read().await;
        !matches!(*state, AgentState::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queen_creation() {
        let config = AgentConfig {
            role: AgentRole::Queen,
            ..Default::default()
        };

        let queen = QueenAgent::new(AgentId(0), config).unwrap();
        assert_eq!(queen.id(), AgentId(0));
        assert_eq!(queen.role(), AgentRole::Queen);
    }

    #[tokio::test]
    async fn test_task_assignment() {
        let config = AgentConfig {
            role: AgentRole::Queen,
            ..Default::default()
        };

        let queen = QueenAgent::new(AgentId(0), config).unwrap();

        let task_id = queen
            .assign_task(AgentId(1), "Test task".into(), 5)
            .await
            .unwrap();

        let assignments = queen.get_assignments().await;
        assert_eq!(assignments.len(), 1);
        assert!(assignments.contains_key(&task_id));
    }
}
