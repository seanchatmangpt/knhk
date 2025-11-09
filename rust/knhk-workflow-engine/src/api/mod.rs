//! Workflow engine API module
//!
//! Provides unified API layer with service layer for business logic
//! and transport adapters for REST, gRPC, and CLI.

pub mod grpc;
pub mod middleware;
pub mod models;
pub mod rest;
pub mod service;
pub mod transport;

// Re-export service layer for convenience
pub use service::{CaseService, PatternService, WorkflowService};

/// Workflow engine API placeholder
pub struct WorkflowEngineApi;

impl WorkflowEngineApi {
    /// Create new workflow engine API
    pub fn new() -> Self {
        Self
    }
}

impl Default for WorkflowEngineApi {
    fn default() -> Self {
        Self::new()
    }
}
