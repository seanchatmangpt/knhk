//! Core YAWL data structures
//!
//! This module contains the fundamental data types for YAWL workflows:
//! - Workflows and their metadata
//! - Tasks and their properties
//! - Transitions and control flow
//! - Net states and tokens
//! - Execution contexts

pub mod workflow;
pub mod task;
pub mod transition;
pub mod net;
pub mod context;

// Re-export core types
pub use workflow::{Workflow, WorkflowBuilder};
pub use task::{Task, TaskType, TaskBuilder};
pub use transition::{Transition, SplitType, JoinType, TransitionBuilder};
pub use net::{NetState, Arc as YawlArc, Token};
pub use context::{ExecutionContext, ContextBuilder};
