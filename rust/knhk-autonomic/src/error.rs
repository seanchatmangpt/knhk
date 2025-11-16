//! Error types for the autonomic system

use thiserror::Error;

/// Result type for autonomic operations
pub type Result<T> = std::result::Result<T, AutonomicError>;

/// Errors that can occur in the autonomic system
#[derive(Error, Debug)]
pub enum AutonomicError {
    /// Monitor component error
    #[error("Monitor error: {0}")]
    Monitor(String),

    /// Analyze component error
    #[error("Analyze error: {0}")]
    Analyze(String),

    /// Planner component error
    #[error("Planner error: {0}")]
    Planner(String),

    /// Execute component error
    #[error("Execute error: {0}")]
    Execute(String),

    /// Knowledge base error
    #[error("Knowledge base error: {0}")]
    Knowledge(String),

    /// Hook execution error
    #[error("Hook error: {0}")]
    Hook(String),

    /// SPARQL query error
    #[error("SPARQL query error: {0}")]
    Sparql(String),

    /// RDF store error
    #[error("RDF store error: {0}")]
    RdfStore(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Latency violation (exceeds Chatman Constant)
    #[error("Latency violation: {operation} took {actual_ticks} ticks (max: {max_ticks} ticks)")]
    LatencyViolation {
        /// Operation that violated latency bound
        operation: String,
        /// Actual ticks taken
        actual_ticks: u64,
        /// Maximum allowed ticks
        max_ticks: u64,
    },

    /// Generic error
    #[error("Autonomic error: {0}")]
    Other(#[from] anyhow::Error),
}
