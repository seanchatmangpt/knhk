//! Compile-time workflow validation and type-level programming
//!
//! This module provides utilities for compile-time workflow validation using
//! Rust's advanced type system and const evaluation.
//!
//! # Features
//!
//! - **Type-level state machines**: Encode workflow states as types
//! - **Const evaluation**: Analyze workflows at compile time
//! - **GADTs**: Generalized Algebraic Data Types for advanced type safety
//! - **Phantom types**: Zero-cost type-level state tracking
//!
//! # Example
//!
//! ```rust,ignore
//! use knhk_workflow_engine::compile_time::*;
//!
//! // Type-safe workflow that can only transition through valid states
//! let workflow = TypedWorkflow::<Initial>::new("workflow-1");
//! let workflow = workflow.validate_email()?;
//! let workflow = workflow.create_account()?;
//! let workflow = workflow.complete()?;
//! // workflow.validate_email()? // Compile error: can't go back!
//! ```

pub mod state_machine;
pub mod const_eval;
pub mod type_level;

pub use state_machine::*;
pub use const_eval::*;
pub use type_level::*;
