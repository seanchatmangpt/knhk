//! Advanced Phase System for KNHK Validation Framework
//!
//! Provides a trait-based generic Phase<T> system with:
//! - HKT-style composition via phantom types
//! - Compile-time phase registration
//! - Async concurrent execution
//! - Type-safe phase composition
//! - Zero-cost abstractions via const generics

pub mod core;
pub mod executor;
pub mod registry;
pub mod telemetry;
pub mod validators;

// Re-export core types
pub use core::{Phase, PhaseContext, PhaseMetadata, PhaseResult, PhaseStatus};
pub use executor::PhaseExecutor;
pub use registry::{register_phase, PhaseRegistry};
pub use validators::{
    ConformanceMetricsPhase, FormalSoundnessPhase, LoadTestingPhase, PatternSemanticsPhase,
};
