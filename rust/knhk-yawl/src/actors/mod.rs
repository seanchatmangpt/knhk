//! Erlang-Style Actor System for YAWL Execution
//!
//! # DOCTRINE ALIGNMENT
//! - **Principle**: Fault tolerance through supervision trees
//! - **Covenant 2**: Invariants enforced by supervisors
//! - **Covenant 5**: Chatman Constant enforced per actor message
//!
//! # ARCHITECTURE
//!
//! Actor hierarchy:
//! ```text
//! WorkflowSupervisor (root)
//! ├── CaseActor (per workflow instance)
//! │   ├── TaskActor (per active task)
//! │   │   └── WorkerActor (execution unit)
//! │   └── PatternCoordinator (manages pattern execution)
//! └── ResourceManager (resource allocation)
//! ```
//!
//! ## Fault Tolerance
//!
//! Supervision strategies:
//! - **WorkflowSupervisor**: Restart failed cases
//! - **CaseActor**: Compensate failed tasks
//! - **TaskActor**: Retry with exponential backoff
//!
//! ## Concurrency Model
//!
//! - Each actor runs on its own Tokio task
//! - Message passing via channels (no shared state)
//! - Work-stealing scheduler for CPU-bound tasks
//! - Non-blocking I/O for external services

pub mod messages;
pub mod supervisor;
pub mod case_actor;
pub mod task_actor;
pub mod pattern_coordinator;
pub mod resource_manager;

pub use messages::*;
pub use supervisor::*;
pub use case_actor::*;
pub use task_actor::*;
pub use pattern_coordinator::*;
pub use resource_manager::*;

use crate::core::*;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{instrument, Span};

/// Actor trait - all actors implement this
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// Actor identifier
    fn actor_id(&self) -> ActorId;

    /// Actor type (for telemetry)
    fn actor_type(&self) -> ActorType;

    /// Start actor event loop
    async fn run(self);

    /// Graceful shutdown
    async fn shutdown(&mut self);
}

/// Actor identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorId(pub uuid::Uuid);

impl ActorId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for ActorId {
    fn default() -> Self {
        Self::new()
    }
}

/// Actor type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorType {
    Supervisor,
    CaseActor,
    TaskActor,
    PatternCoordinator,
    ResourceManager,
    Worker,
}

/// Supervision strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionStrategy {
    /// Restart failed child
    Restart,
    /// Resume execution (ignore error)
    Resume,
    /// Stop child permanently
    Stop,
    /// Escalate to parent supervisor
    Escalate,
}

/// Actor handle - reference to running actor
#[derive(Clone)]
pub struct ActorHandle<M: ActorMessage> {
    pub actor_id: ActorId,
    pub sender: mpsc::Sender<M>,
}

impl<M: ActorMessage> ActorHandle<M> {
    /// Send message to actor
    pub async fn send(&self, msg: M) -> Result<(), ExecutionError> {
        self.sender
            .send(msg)
            .await
            .map_err(|_| ExecutionError::Internal("Actor channel closed".to_string()))
    }

    /// Send message and wait for reply
    pub async fn ask<R>(&self, msg: M) -> Result<R, ExecutionError>
    where
        M: WithReply<R>,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let msg_with_reply = msg.with_reply(tx);

        self.send(msg_with_reply).await?;

        rx.await
            .map_err(|_| ExecutionError::Internal("Reply channel closed".to_string()))
    }
}

/// Actor message trait
pub trait ActorMessage: Send + 'static {}

/// Message with reply channel
pub trait WithReply<R>: ActorMessage {
    fn with_reply(self, reply: oneshot::Sender<R>) -> Self;
}

/// Actor lifecycle events
#[derive(Debug, Clone)]
pub enum ActorEvent {
    Started { actor_id: ActorId },
    MessageReceived { actor_id: ActorId, message_type: String },
    MessageProcessed { actor_id: ActorId, ticks: u8 },
    Failed { actor_id: ActorId, error: String },
    Restarted { actor_id: ActorId },
    Stopped { actor_id: ActorId },
}

/// Actor context - shared runtime state
pub struct ActorContext {
    pub actor_id: ActorId,
    pub actor_type: ActorType,
    pub parent: Option<ActorId>,
    pub children: Vec<ActorId>,
    pub span: Span,
}

impl ActorContext {
    pub fn new(actor_type: ActorType) -> Self {
        let actor_id = ActorId::new();
        let span = tracing::info_span!(
            "actor",
            actor_id = %actor_id.0,
            actor_type = ?actor_type
        );

        Self {
            actor_id,
            actor_type,
            parent: None,
            children: Vec::new(),
            span,
        }
    }

    pub fn with_parent(mut self, parent: ActorId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn add_child(&mut self, child: ActorId) {
        self.children.push(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_id_generation() {
        let id1 = ActorId::new();
        let id2 = ActorId::new();

        assert_ne!(id1, id2);
    }
}
