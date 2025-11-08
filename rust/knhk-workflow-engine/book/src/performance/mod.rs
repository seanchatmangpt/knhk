//! Performance optimization module
//!
//! Provides performance optimizations including hot path operations,
//! SIMD support, caching, and performance monitoring.

mod analyzer;
mod hot_path;
mod metrics;
pub mod simd;

pub use analyzer::{
    CaseMetrics, HotPathAnalysis, TickViolation, ViolationSeverity, WorkflowProfiler,
};
pub use hot_path::*;
pub use metrics::*;
pub use simd::*;
