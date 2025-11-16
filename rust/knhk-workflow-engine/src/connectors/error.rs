//! Connector error types
//!
//! Re-exports error types from the main error module for backward compatibility.

// Re-export error types from main error module
pub use crate::error::sources::{
    CircuitBreakerError, ConnectorError, PoolError, RegistryError, RetryError,
};
