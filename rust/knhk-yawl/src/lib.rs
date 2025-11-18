//! KNHK YAWL - Core YAWL Data Structures and Execution Engine
//!
//! # Doctrine Alignment
//!
//! This package implements core YAWL (Yet Another Workflow Language) structures
//! following DOCTRINE_2027 principles:
//!
//! - **Covenant 1** (O ⊨ Σ): All workflow definitions are observable via telemetry
//! - **Covenant 2** (Q ⊨ Implementation): Respects Q3 (max_run_length ≤ 8 ticks)
//! - **Covenant 4** (Σ ⊨ Completeness): All 43 W3C patterns expressible via permutations
//! - **Covenant 5** (Q3 ⊨ Boundedness): Hot path operations ≤ 8 ticks (Chatman constant)
//! - **Covenant 6** (O ⊨ Discovery): Full OpenTelemetry instrumentation
//!
//! # Module Structure
//!
//! - `core`: Core data types (Workflow, Task, Transition, NetState, ExecutionContext)
//! - `patterns`: Pattern trait hierarchy for all 43+ W3C workflow patterns
//! - `engine`: Execution engine (WorkflowExecutor, TaskActor, TokenManager)
//! - `telemetry`: OpenTelemetry integration for observability
//!
//! # Example
//!
//! ```rust
//! use knhk_yawl::core::{Workflow, Task, TaskType};
//! use knhk_yawl::patterns::PatternType;
//!
//! let task = Task::new("task1", "Process Order", TaskType::Atomic);
//! let workflow = Workflow::builder()
//!     .id("order-processing")
//!     .name("Order Processing Workflow")
//!     .version("1.0.0")
//!     .add_task(task)
//!     .build();
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

pub mod core;
pub mod patterns;
pub mod engine;
pub mod telemetry;

// Re-export commonly used types
pub use core::{
    workflow::{Workflow, WorkflowBuilder},
    task::{Task, TaskType, TaskBuilder},
    transition::{Transition, SplitType, JoinType},
    net::{NetState, Arc as YawlArc},
    context::ExecutionContext,
};

pub use patterns::{YawlPattern, PatternType, PatternError};

pub use engine::{
    executor::WorkflowExecutor,
    token::TokenManager,
};

/// Result type for YAWL operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for YAWL operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Pattern execution error
    #[error("Pattern execution failed: {0}")]
    PatternExecution(String),

    /// Invalid workflow structure
    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),

    /// Task execution error
    #[error("Task execution failed: {0}")]
    TaskExecution(String),

    /// Timeout error (Covenant 5: Chatman constant violation)
    #[error("Operation exceeded time bound: {0} ticks (max: 8)")]
    TimeoutViolation(u64),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Other errors
    #[error("Error: {0}")]
    Other(#[from] anyhow::Error),
}
