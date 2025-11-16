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
pub mod pipeline;
pub mod queue;
pub mod snapshot;
pub mod receipt;
pub mod hooks;
pub mod self_executing;

pub use engine::{ExecutionEngine, ExecutionHandle, ExecutionStatus};
pub use pipeline::ExecutionPipeline;
pub use queue::WorkQueue;
pub use snapshot::{SnapshotId, SnapshotManifest, SnapshotStore, OntologyFile};
pub use receipt::{Receipt, ReceiptId, ReceiptStore, ReceiptStatistics};
pub use hooks::{HookContext, HookResult, HookRegistry, HookEngine, HookFn, patterns};
pub use self_executing::SelfExecutingWorkflow;
