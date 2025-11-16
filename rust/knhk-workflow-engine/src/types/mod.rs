//! Type system enhancements for compile-time safety and zero-cost abstractions
//!
//! This module provides advanced type system features:
//! - Generic Associated Types (GATs) for flexible pattern executors
//! - Phantom types for compile-time validation
//! - Newtypes for primitive type safety
//!
//! All types are designed for zero-cost abstraction with compile-time guarantees.

#[cfg(feature = "type-system-v2")]
pub mod gat;
pub mod newtypes;
pub mod phantom;

#[cfg(feature = "type-system-v2")]
pub use gat::{
    AsyncPatternExecutor, BasicPatternExecutor, BatchPatternExecutor, LegacyPatternAdapter,
    PatternExecutor, StatefulPatternExecutor,
};

pub use newtypes::{BatchSize, PriorityLevel, RetryCount, TickCount, TimeoutMs};

pub use phantom::{
    create_proof, NotValidated, SpecNotValidated, SpecValidated, Validate, Validatable,
    Validated, ValidationMarker, ValidationProof, WorkflowSpec,
};
