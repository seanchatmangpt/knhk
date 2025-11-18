//! Core Trait Hierarchy for YAWL Elements
//!
//! # DOCTRINE ALIGNMENT
//! - **Principle**: Σ (Ontology) - Type system mirrors RDF ontology
//! - **Covenant 2**: Invariants Are Law (all traits enforce Q constraints)
//!
//! This module defines the trait hierarchy that maps directly to the
//! YAWL ontology defined in `yawl-extended.ttl`.

use super::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Root trait for all workflow elements
///
/// All YAWL elements (tasks, arcs, conditions) implement this trait.
/// This provides uniform telemetry and lifecycle management.
pub trait WorkflowElement: Debug + Send + Sync {
    /// Unique identifier for this element
    fn id(&self) -> WorkflowId;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Element type (for telemetry and validation)
    fn element_type(&self) -> ElementType;

    /// Validate element against Q invariants
    fn validate(&self) -> Result<(), ValidationError>;
}

/// Element type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    AtomicTask,
    CompositeTask,
    SubflowTask,
    Condition,
    ControlFlow,
    DataFlow,
    Pattern,
}

/// Executable workflow element
///
/// Elements that can be executed by the workflow engine must implement this trait.
/// Execution is async and produces observable telemetry.
#[async_trait]
pub trait Executable: WorkflowElement {
    /// Execute this element within a case context
    ///
    /// # Chatman Constant Enforcement
    /// Implementations MUST complete within 8 ticks for hot path elements.
    /// Use `tick_counter` to track execution ticks.
    ///
    /// # Telemetry
    /// All executions MUST emit OTEL spans as defined in the schema.
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        tick_counter: &mut TickCounter,
    ) -> Result<ExecutionResult, ExecutionError>;

    /// Check if this element can execute (preconditions met)
    fn can_execute(&self, context: &ExecutionContext) -> bool;

    /// Estimated execution ticks (for scheduling)
    fn estimated_ticks(&self) -> u8 {
        1 // Default: most operations are 1 tick
    }
}

/// Split behavior trait
///
/// Elements that fan out to multiple branches (AND, OR, XOR splits)
#[async_trait]
pub trait Splittable: Executable {
    /// Get split type (AND, OR, XOR)
    fn split_type(&self) -> SplitType;

    /// Evaluate split and return active outgoing arcs
    ///
    /// # Split Types
    /// - AND: All outgoing arcs are activated
    /// - OR: One or more arcs activated based on predicates
    /// - XOR: Exactly one arc activated based on predicates
    async fn evaluate_split(
        &self,
        context: &ExecutionContext,
    ) -> Result<Vec<ArcId>, ExecutionError>;
}

/// Join behavior trait
///
/// Elements that synchronize multiple incoming branches
#[async_trait]
pub trait Joinable: Executable {
    /// Get join type (AND, OR, XOR, Discriminator)
    fn join_type(&self) -> JoinType;

    /// Check if join condition is satisfied
    ///
    /// # Join Types
    /// - AND: All incoming arcs must complete
    /// - OR: All *active* incoming arcs must complete
    /// - XOR: Any incoming arc triggers
    /// - Discriminator: First N incoming arcs (quorum)
    fn is_join_satisfied(
        &self,
        context: &ExecutionContext,
        completed_arcs: &[ArcId],
    ) -> bool;

    /// Reset join state (for loops)
    async fn reset_join(&mut self);
}

/// Conditional evaluation trait
///
/// Elements that make routing decisions based on data/predicates
pub trait Conditional: WorkflowElement {
    /// Evaluate predicate against current state
    fn evaluate(&self, context: &ExecutionContext) -> Result<bool, ExecutionError>;

    /// Get condition expression (for telemetry/debugging)
    fn condition_expression(&self) -> &str;
}

/// Cancellable element trait
///
/// Elements that support cancellation regions and exception handling
#[async_trait]
pub trait Cancellable: Executable {
    /// Cancel this element and its descendants
    async fn cancel(&mut self, context: &mut ExecutionContext) -> Result<(), ExecutionError>;

    /// Check if element is in cancellation region
    fn is_in_cancellation_region(&self) -> bool;

    /// Get cancellation set (elements to cancel together)
    fn cancellation_set(&self) -> Vec<TaskId>;
}

/// Resource allocation trait
///
/// Elements that require/allocate resources (agents, services, etc.)
#[async_trait]
pub trait ResourceAware: Executable {
    /// Required resource type
    fn required_resource(&self) -> ResourceType;

    /// Allocate resource for execution
    async fn allocate_resource(
        &self,
        context: &mut ExecutionContext,
    ) -> Result<ResourceHandle, ExecutionError>;

    /// Release allocated resource
    async fn release_resource(&self, handle: ResourceHandle) -> Result<(), ExecutionError>;
}

/// Observable element trait
///
/// All elements must be observable via OpenTelemetry
pub trait Observable: WorkflowElement {
    /// Create OTEL span for this element's execution
    fn create_span(&self) -> tracing::Span;

    /// Record telemetry attributes
    fn record_attributes(&self, span: &tracing::Span);

    /// Emit completion event
    fn emit_completion(&self, result: &ExecutionResult);
}

/// Split type enumeration (from yawl-pattern-permutations.ttl)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitType {
    /// AND Split - all outgoing branches taken
    AND,
    /// OR Split - one or more branches taken based on predicates
    OR,
    /// XOR Split - exactly one branch taken
    XOR,
}

/// Join type enumeration (from yawl-pattern-permutations.ttl)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinType {
    /// AND Join - all incoming branches must complete
    AND,
    /// OR Join - all *active* incoming branches must complete
    OR,
    /// XOR Join - any incoming branch triggers
    XOR,
    /// Discriminator - first N incoming branches (quorum)
    Discriminator { quorum: usize },
}

/// Resource type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Human { role: String },
    Service { endpoint: String },
    Agent { agent_type: String },
    Computational { cores: usize, memory_mb: usize },
}

/// Resource allocation handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHandle {
    pub resource_id: Uuid,
    pub resource_type: ResourceType,
    pub allocated_at: std::time::Instant,
}

/// Execution context - shared state during workflow execution
#[derive(Debug)]
pub struct ExecutionContext {
    pub case_id: CaseId,
    pub workflow_id: WorkflowId,
    pub data: dashmap::DashMap<String, serde_json::Value>,
    pub active_tasks: dashmap::DashSet<TaskId>,
    pub completed_tasks: dashmap::DashSet<TaskId>,
    pub active_arcs: dashmap::DashSet<ArcId>,
    pub completed_arcs: dashmap::DashSet<ArcId>,
    pub allocated_resources: dashmap::DashMap<TaskId, ResourceHandle>,
}

impl ExecutionContext {
    pub fn new(case_id: CaseId, workflow_id: WorkflowId) -> Self {
        Self {
            case_id,
            workflow_id,
            data: dashmap::DashMap::new(),
            active_tasks: dashmap::DashSet::new(),
            completed_tasks: dashmap::DashSet::new(),
            active_arcs: dashmap::DashSet::new(),
            completed_arcs: dashmap::DashSet::new(),
            allocated_resources: dashmap::DashMap::new(),
        }
    }

    /// Get data value by key
    pub fn get_data(&self, key: &str) -> Option<serde_json::Value> {
        self.data.get(key).map(|v| v.clone())
    }

    /// Set data value
    pub fn set_data(&self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    /// Check if task is active
    pub fn is_task_active(&self, task_id: &TaskId) -> bool {
        self.active_tasks.contains(task_id)
    }

    /// Mark task as completed
    pub fn complete_task(&self, task_id: TaskId) {
        self.active_tasks.remove(&task_id);
        self.completed_tasks.insert(task_id);
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub ticks_used: u8,
    pub output_data: Option<serde_json::Value>,
    pub activated_arcs: Vec<ArcId>,
}

/// Validation error
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid split-join combination: {split:?} -> {join:?}")]
    InvalidSplitJoinCombination { split: SplitType, join: JoinType },

    #[error("Missing required property: {property}")]
    MissingProperty { property: String },

    #[error("Cycle detected in workflow graph")]
    CycleDetected,

    #[error("Unreachable task: {task_id}")]
    UnreachableTask { task_id: TaskId },

    #[error("Resource type mismatch: expected {expected:?}, got {actual:?}")]
    ResourceTypeMismatch {
        expected: ResourceType,
        actual: ResourceType,
    },
}

/// Execution error
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Chatman constant violated: {ticks} ticks exceeds limit of {limit}")]
    ChatmanConstantViolation { ticks: u8, limit: u8 },

    #[error("Precondition not satisfied: {condition}")]
    PreconditionNotSatisfied { condition: String },

    #[error("Resource allocation failed: {resource:?}")]
    ResourceAllocationFailed { resource: ResourceType },

    #[error("Data binding error: {message}")]
    DataBindingError { message: String },

    #[error("Task {task_id} not found")]
    TaskNotFound { task_id: TaskId },

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),

    #[error("Internal error: {0}")]
    Internal(String),
}
