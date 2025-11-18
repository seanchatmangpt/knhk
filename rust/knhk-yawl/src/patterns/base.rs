// knhk-yawl/src/patterns/base.rs
// Base trait definitions for YAWL patterns

use crate::error::{YawlError, YawlResult};
use crate::triz::TrizPrinciple;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Split behavior type (from YAWL ontology)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitType {
    /// AND split - all branches taken in parallel
    And,
    /// OR split - one or more branches taken based on predicate
    Or,
    /// XOR split - exactly one branch taken
    Xor,
    /// No split - sequential execution
    None,
}

/// Join behavior type (from YAWL ontology)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinType {
    /// AND join - wait for all incoming branches
    And,
    /// OR join - wait for all active branches
    Or,
    /// XOR join - continue on first incoming branch
    Xor,
    /// Discriminator - continue after first N branches (quorum)
    Discriminator,
    /// No join - no synchronization
    None,
}

/// Execution context for pattern execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Workflow instance ID
    pub instance_id: String,
    /// Task ID within the workflow
    pub task_id: String,
    /// Input data as JSON value
    pub data: serde_json::Value,
    /// Execution variables
    pub variables: HashMap<String, serde_json::Value>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Maximum iterations for loops (Q3: Bounded recursion)
    pub max_iterations: u32,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(instance_id: impl Into<String>, task_id: impl Into<String>) -> Self {
        Self {
            instance_id: instance_id.into(),
            task_id: task_id.into(),
            data: serde_json::Value::Null,
            variables: HashMap::new(),
            timeout_ms: None,
            max_iterations: 8, // Default: Chatman constant
        }
    }

    /// Set input data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Set a variable
    pub fn set_variable(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.variables.insert(key.into(), value);
    }

    /// Get a variable
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }
}

/// Output from pattern execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternOutput {
    /// Output data from execution
    pub data: serde_json::Value,
    /// Number of branches that executed
    pub branch_count: usize,
    /// Execution metrics
    pub metrics: ExecutionMetrics,
}

/// Execution metrics for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Execution time in ticks (for Chatman constant validation)
    pub ticks: u32,
    /// Execution time in microseconds
    pub duration_us: u64,
    /// Number of iterations (for loop patterns)
    pub iterations: u32,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            ticks: 0,
            duration_us: 0,
            iterations: 0,
        }
    }
}

/// Base trait for all YAWL workflow patterns
///
/// Each pattern must implement this trait to be executable within the KNHK workflow engine.
pub trait YawlPattern: Send + Sync {
    /// Get the name of this pattern
    fn name(&self) -> &str;

    /// Get the TRIZ principle(s) this pattern embodies
    fn triz_principles(&self) -> &[TrizPrinciple];

    /// Get the split type for this pattern
    fn split_type(&self) -> SplitType {
        SplitType::None
    }

    /// Get the join type for this pattern
    fn join_type(&self) -> JoinType {
        JoinType::None
    }

    /// Decompose this pattern into sub-patterns (TRIZ decomposition)
    ///
    /// Returns a list of simpler patterns that compose this pattern.
    /// Leaf patterns return an empty vector.
    fn decompose(&self) -> Vec<Arc<dyn YawlPattern>> {
        Vec::new()
    }

    /// Execute the pattern with the given context
    ///
    /// # Errors
    ///
    /// Returns `YawlError` if execution fails or constraints are violated.
    ///
    /// # Performance
    ///
    /// Hot path operations MUST complete within 8 ticks (Q3: Chatman constant).
    fn execute(&self, context: &ExecutionContext) -> YawlResult<PatternOutput>;

    /// Validate pattern configuration
    ///
    /// Called before execution to ensure the pattern is properly configured.
    fn validate(&self) -> YawlResult<()> {
        Ok(())
    }
}
