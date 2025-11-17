//! KNHK Hook Engine and Execution Layer
//!
//! Implements μ via KNHK with:
//! - Hook execution engine
//! - Pattern library (43 YAWL patterns)
//! - Latency-bounded scheduler (≤8 ticks)
//! - OTEL integration
//! - Receipt system
//! - Guard enforcement
//! - Snapshot management
//!
//! # YAWL Engine Components
//!
//! YAWL-specific implementations with TRIZ hyper-advanced patterns:
//! - `y_engine` - Core YAWL engine (TRIZ Principle 28: Mechanics Substitution)
//! - `net_runner` - Net execution (TRIZ Principle 24: Intermediary)
//! - `y_work_item` - Work item lifecycle (TRIZ Principle 32: Color Changes)

pub mod case_store;
pub mod hook_engine;
pub mod net_runner;
pub mod pattern_library;
pub mod scheduler;
pub mod y_engine;
pub mod y_work_item;

pub use case_store::CaseStore;
pub use hook_engine::{HookEngine, HookExecutionResult, MAX_HOT_PATH_TICKS};
pub use net_runner::{ExecutionPlan, ExecutionStatus, YNetRunner};
pub use pattern_library::{PatternCategory, PatternLibrary, PatternMetadata};
pub use scheduler::{LatencyBoundedScheduler, Priority, SchedulerStats, TaskExecutionResult};
pub use y_engine::{EngineStatus, ExecutionStrategy, YEngine};
pub use y_work_item::{
    Allocated, Cancelled, Completed, Enabled, Executing, WorkItem, WorkItemId, WorkItemRepository,
    WorkItemStatus,
};
