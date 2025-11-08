//! Workflow specification types
//!
//! Core types for workflow specifications including tasks, conditions, and specifications.

use uuid::Uuid;

use crate::error::{WorkflowError, WorkflowResult};

/// Unique identifier for a workflow specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct WorkflowSpecId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl WorkflowSpecId {
    /// Generate a new spec ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse from string
    pub fn parse_str(s: &str) -> WorkflowResult<Self> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|e| WorkflowError::Parse(format!("Invalid spec ID: {}", e)))
    }
}

impl Default for WorkflowSpecId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WorkflowSpecId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Split type (AND, XOR, OR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SplitType {
    /// AND-split: all branches execute
    And,
    /// XOR-split: exactly one branch executes
    Xor,
    /// OR-split: one or more branches execute
    Or,
}

/// Join type (AND, XOR, OR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JoinType {
    /// AND-join: wait for all branches
    And,
    /// XOR-join: wait for one branch
    Xor,
    /// OR-join: wait for all active branches
    Or,
}

/// Task type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    /// Atomic task (cannot be decomposed)
    Atomic,
    /// Composite task (contains sub-workflow)
    Composite,
    /// Multiple instance task
    MultipleInstance,
}

/// Workflow task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    /// Task identifier (IRI)
    pub id: String,
    /// Task name/label
    pub name: String,
    /// Task type
    pub task_type: TaskType,
    /// Split type
    pub split_type: SplitType,
    /// Join type
    pub join_type: JoinType,
    /// Maximum execution ticks (≤8 for hot path)
    pub max_ticks: Option<u32>,
    /// Priority (0-255)
    pub priority: Option<u32>,
    /// Use SIMD optimization
    pub use_simd: bool,
    /// Input conditions
    pub input_conditions: Vec<String>,
    /// Output conditions
    pub output_conditions: Vec<String>,
    /// Outgoing flows (task IDs)
    pub outgoing_flows: Vec<String>,
    /// Incoming flows (task IDs)
    pub incoming_flows: Vec<String>,
    /// Resource allocation policy
    pub allocation_policy: Option<crate::resource::AllocationPolicy>,
    /// Required roles for task execution
    pub required_roles: Vec<String>,
    /// Required capabilities for task execution
    pub required_capabilities: Vec<String>,
    /// Worklet ID for exception handling (optional)
    pub exception_worklet: Option<crate::worklets::WorkletId>,
}

/// Workflow condition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Condition {
    /// Condition identifier (IRI)
    pub id: String,
    /// Condition name/label
    pub name: String,
    /// Outgoing flows (task IDs)
    pub outgoing_flows: Vec<String>,
    /// Incoming flows (task IDs)
    pub incoming_flows: Vec<String>,
}

/// Workflow specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowSpec {
    /// Specification ID
    pub id: WorkflowSpecId,
    /// Specification name
    pub name: String,
    /// Tasks in the workflow
    pub tasks: std::collections::HashMap<String, Task>,
    /// Conditions in the workflow
    pub conditions: std::collections::HashMap<String, Condition>,
    /// Start condition ID
    pub start_condition: Option<String>,
    /// End condition ID
    pub end_condition: Option<String>,
}
