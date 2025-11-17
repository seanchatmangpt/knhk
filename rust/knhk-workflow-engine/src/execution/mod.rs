//! Execution module - Improved execution architecture
//!
//! Provides:
//! - Execution engine with async pattern execution
//! - Execution pipeline for pattern composition
//! - Execution context management
//! - Execution tracking and monitoring
//! - Workflow snapshot management (Σ*)
//! - Cryptographic receipts (Γ(O))
//! - Hook evaluation framework (μ)

pub mod engine;
pub mod hooks;
pub mod pipeline;
pub mod queue;
pub mod receipt;
pub mod self_executing;
pub mod snapshot;

pub use engine::{ExecutionEngine, ExecutionHandle, ExecutionStatus};
pub use hooks::{patterns, HookContext, HookEngine, HookFn, HookRegistry, HookResult};
pub use pipeline::ExecutionPipeline;
pub use queue::WorkQueue;
pub use receipt::{Receipt, ReceiptId, ReceiptStatistics, ReceiptStore};
pub use self_executing::SelfExecutingWorkflow;
pub use snapshot::{OntologyFile, SnapshotId, SnapshotManifest, SnapshotStore};
