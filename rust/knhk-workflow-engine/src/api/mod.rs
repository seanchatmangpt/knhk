//! Workflow engine API module
//!
//! Provides unified API layer with service layer for business logic
//! and transport adapters for REST, gRPC, and CLI.

#[cfg(feature = "grpc")]
pub mod grpc;
pub mod interface_b;
pub mod interface_x;
#[cfg(feature = "http")]
pub mod middleware;
pub mod models;
#[cfg(feature = "http")]
pub mod rest;
pub mod service;
pub mod transport;

// Re-export service layer for convenience
pub use interface_b::{
    InterfaceB, LaunchMode, SessionHandle, UserId, WorkItemNotification, WorkItemRecord,
};
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
