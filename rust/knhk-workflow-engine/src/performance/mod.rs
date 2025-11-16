//! Performance optimization module
//!
//! Provides performance optimizations including hot path operations,
//! SIMD support, caching, and performance monitoring.

mod analyzer;
pub mod aot;
pub mod benchmark;
mod hot_path;
mod metrics;
pub mod simd;
pub mod tick_budget;
/// Phase 1: Type-System Mastery - Zero-cost dispatch abstractions
pub mod zero_cost;

pub use analyzer::{
    CaseMetrics, HotPathAnalysis, TickViolation, ViolationSeverity, WorkflowProfiler,
};
pub use hot_path::*;
pub use metrics::*;
pub use simd::*;
