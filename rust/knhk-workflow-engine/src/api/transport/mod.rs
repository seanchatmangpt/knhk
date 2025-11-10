//! Transport adapters
//!
//! Adapters that convert transport-specific types to unified models.

pub mod cli;
#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "http")]
pub mod rest;

// Re-export for convenience
pub use cli::CliAdapter;
#[cfg(feature = "grpc")]
pub use grpc::GrpcAdapter;
#[cfg(feature = "http")]
pub use rest::RestAdapter;
