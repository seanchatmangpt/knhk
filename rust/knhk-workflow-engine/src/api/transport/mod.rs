//! Transport adapters
//!
//! Adapters that convert transport-specific types to unified models.

pub mod cli;
pub mod grpc;
pub mod rest;

// Re-export for convenience
pub use cli::CliAdapter;
pub use grpc::GrpcAdapter;
pub use rest::RestAdapter;
