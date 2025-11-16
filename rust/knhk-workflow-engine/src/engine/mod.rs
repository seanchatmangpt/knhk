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

pub mod hook_engine;
pub mod pattern_library;
pub mod scheduler;

pub use hook_engine::{HookEngine, HookExecutionResult, MAX_HOT_PATH_TICKS};
pub use pattern_library::{PatternCategory, PatternLibrary, PatternMetadata};
pub use scheduler::{LatencyBoundedScheduler, Priority, SchedulerStats, TaskExecutionResult};
