//! Execution module - Improved execution architecture
//!
//! Provides:
//! - Execution engine with async pattern execution
//! - Execution pipeline for pattern composition
//! - Execution context management
//! - Execution tracking and monitoring

pub mod engine;
pub mod pipeline;
pub mod queue;

pub use engine::{ExecutionEngine, ExecutionHandle, ExecutionStatus};
pub use pipeline::ExecutionPipeline;
pub use queue::WorkQueue;
