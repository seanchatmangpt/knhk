//! Workflow engine API module

pub mod grpc;
pub mod middleware;
pub mod models;
pub mod rest;

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
