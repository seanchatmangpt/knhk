//! Actor Message Types

use super::*;
use crate::core::*;
use tokio::sync::oneshot;

/// Case actor messages
#[derive(Debug)]
pub enum CaseMessage {
    /// Start case execution
    Start {
        workflow: Arc<Workflow>,
        initial_data: serde_json::Value,
        reply: oneshot::Sender<Result<CaseId, ExecutionError>>,
    },

    /// Execute task
    ExecuteTask {
        task_id: TaskId,
        reply: oneshot::Sender<Result<ExecutionResult, ExecutionError>>,
    },

    /// Suspend case
    Suspend {
        reply: oneshot::Sender<Result<(), ExecutionError>>,
    },

    /// Resume case
    Resume {
        reply: oneshot::Sender<Result<(), ExecutionError>>,
    },

    /// Cancel case
    Cancel {
        reply: oneshot::Sender<Result<(), ExecutionError>>,
    },

    /// Get case state
    GetState {
        reply: oneshot::Sender<CaseSnapshot>,
    },

    /// Shutdown actor
    Shutdown,
}

impl ActorMessage for CaseMessage {}

/// Task actor messages
#[derive(Debug)]
pub enum TaskMessage {
    /// Execute task
    Execute {
        context: Arc<parking_lot::RwLock<ExecutionContext>>,
        reply: oneshot::Sender<Result<ExecutionResult, ExecutionError>>,
    },

    /// Suspend task
    Suspend {
        reply: oneshot::Sender<Result<(), ExecutionError>>,
    },

    /// Resume task
    Resume {
        reply: oneshot::Sender<Result<(), ExecutionError>>,
    },

    /// Cancel task
    Cancel {
        reply: oneshot::Sender<Result<(), ExecutionError>>,
    },

    /// Shutdown actor
    Shutdown,
}

impl ActorMessage for TaskMessage {}

/// Resource manager messages
#[derive(Debug)]
pub enum ResourceMessage {
    /// Allocate resource
    Allocate {
        resource_type: ResourceType,
        task_id: TaskId,
        reply: oneshot::Sender<Result<ResourceHandle, ExecutionError>>,
    },

    /// Release resource
    Release {
        handle: ResourceHandle,
        reply: oneshot::Sender<Result<(), ExecutionError>>,
    },

    /// Get available resources
    GetAvailable {
        resource_type: ResourceType,
        reply: oneshot::Sender<usize>,
    },

    /// Shutdown actor
    Shutdown,
}

impl ActorMessage for ResourceMessage {}

/// Pattern coordinator messages
#[derive(Debug)]
pub enum PatternMessage {
    /// Execute pattern
    ExecutePattern {
        pattern_type: PatternType,
        context: Arc<parking_lot::RwLock<ExecutionContext>>,
        reply: oneshot::Sender<Result<ExecutionResult, ExecutionError>>,
    },

    /// Shutdown actor
    Shutdown,
}

impl ActorMessage for PatternMessage {}

/// Pattern type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    Sequence,
    ParallelSplit,
    Synchronization,
    ExclusiveChoice,
    SimpleMerge,
    MultiChoice,
    SynchronizingMerge,
    MultiMerge,
    Discriminator,
    ORJoin,
    // ... 33 more patterns
}

/// Supervisor messages
#[derive(Debug)]
pub enum SupervisorMessage {
    /// Register child actor
    RegisterChild {
        child_id: ActorId,
        child_type: ActorType,
    },

    /// Child actor failed
    ChildFailed {
        child_id: ActorId,
        error: String,
    },

    /// Restart child
    RestartChild {
        child_id: ActorId,
    },

    /// Shutdown all children
    ShutdownAll {
        reply: oneshot::Sender<()>,
    },
}

impl ActorMessage for SupervisorMessage {}
