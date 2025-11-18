//! Worker Agent - Executes assigned tasks

use super::{Agent, AgentConfig};
use crate::error::{SwarmError, SwarmResult};
use crate::types::{AgentId, AgentRole, AgentState, TaskId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

/// Task result from worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub success: bool,
    pub output: String,
}

/// The Worker Agent
pub struct WorkerAgent {
    id: AgentId,
    config: AgentConfig,
    state: Arc<RwLock<AgentState>>,
    current_task: Arc<RwLock<Option<TaskId>>>,
}

impl WorkerAgent {
    /// Create a new Worker agent
    #[instrument]
    pub fn new(id: AgentId, config: AgentConfig) -> SwarmResult<Self> {
        info!("Creating Worker agent: {}", id);

        Ok(Self {
            id,
            config,
            state: Arc::new(RwLock::new(AgentState::Idle)),
            current_task: Arc::new(RwLock::new(None)),
        })
    }

    /// Start executing a task
    #[instrument(skip(self))]
    pub async fn start_task(&self, task_id: TaskId) -> SwarmResult<()> {
        *self.state.write().await = AgentState::Working;
        *self.current_task.write().await = Some(task_id);

        info!("Worker {} started task {}", self.id, task_id);
        Ok(())
    }

    /// Complete current task
    pub async fn complete_task(&self, success: bool, output: String) -> SwarmResult<TaskResult> {
        let task_id = self
            .current_task
            .write()
            .await
            .take()
            .ok_or_else(|| SwarmError::Internal("No active task".into()))?;

        *self.state.write().await = AgentState::Idle;

        info!(
            "Worker {} completed task {}: success={}",
            self.id, task_id, success
        );

        Ok(TaskResult {
            task_id,
            success,
            output,
        })
    }

    /// Get current task
    pub async fn current_task(&self) -> Option<TaskId> {
        *self.current_task.read().await
    }
}

impl Agent for WorkerAgent {
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
        *self.state.write().await = AgentState::Idle;
        info!("Worker agent {} started", self.id);
    }

    async fn stop(&mut self) {
        *self.state.write().await = AgentState::Idle;
        *self.current_task.write().await = None;
        info!("Worker agent {} stopped", self.id);
    }

    async fn health_check(&self) -> bool {
        let state = self.state.read().await;
        !matches!(*state, AgentState::Failed)
    }
}
