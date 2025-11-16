//! Workflow engine API module
//!
//! Provides unified API layer with service layer for business logic
//! and transport adapters for REST, gRPC, and CLI.

#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "grpc")]
pub mod grpc_server;
#[cfg(feature = "http")]
pub mod middleware;
pub mod models;
#[cfg(feature = "http")]
pub mod rest;
pub mod service;
pub mod transport;

// Re-export service layer for convenience
pub use service::{CaseService, PatternService, WorkflowService};

// Re-export gRPC server for convenience
#[cfg(feature = "grpc")]
pub use grpc_server::{GrpcServer, GrpcServerConfig};

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
