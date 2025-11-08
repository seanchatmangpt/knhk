//! Enterprise workflow engine with full 43-pattern YAWL support
//!
//! This crate provides a complete workflow engine that:
//! - Parses Turtle/YAWL workflow definitions
//! - Executes all 43 Van der Aalst workflow patterns
//! - Provides enterprise APIs (REST + gRPC)
//! - Manages workflow cases with state persistence
//! - Integrates with KNHK infrastructure (OTEL, lockchain, connectors)

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod api;
pub mod case;
pub mod error;
pub mod executor;
pub mod integration;
pub mod parser;
pub mod patterns;
pub mod state;

pub use case::{Case, CaseId, CaseState};
pub use error::{WorkflowError, WorkflowResult};
pub use executor::WorkflowEngine;
pub use parser::{WorkflowParser, WorkflowSpec, WorkflowSpecId};
// Pattern types are defined in patterns module
// pub use patterns::{PatternId, PatternRegistry};
pub use state::StateStore;
