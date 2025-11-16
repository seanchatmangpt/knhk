//! Error types for the autonomous system

use thiserror::Error;

/// Result type for autonomous system operations
pub type Result<T> = std::result::Result<T, SystemError>;

/// Comprehensive error types for the autonomous ontology system
#[derive(Error, Debug)]
pub enum SystemError {
    /// Initialization failed
    #[error("System initialization failed: {0}")]
    InitializationFailed(String),

    /// Meta-ontology loading failed
    #[error("Meta-ontology loading failed: {0}")]
    MetaOntologyLoadFailed(String),

    /// Snapshot store error
    #[error("Snapshot store error: {0}")]
    SnapshotStore(#[from] knhk_ontology::PromotionError),

    /// Change engine error
    #[error("Change engine error: {0}")]
    ChangeEngine(#[from] knhk_change_engine::Error),

    /// Projection compiler error
    #[error("Projection compiler error: {0}")]
    ProjectionCompiler(#[from] knhk_projections::ProjectionError),

    /// Promotion pipeline error
    #[error("Promotion pipeline error: {0}")]
    PromotionPipeline(#[from] knhk_promotion::PromotionError),

    // /// Autonomous loop error
    // #[error("Autonomous loop error: {0}")]
    // AutonomousLoop(#[from] knhk_autonomous_loop::AutonomousLoopError),

    /// τ-axis (time) violation
    #[error("Time axis violation: {0}")]
    TimeAxisViolation(String),

    /// μ-axis (mapping) violation
    #[error("Mapping axis violation: {0}")]
    MappingAxisViolation(String),

    /// Γ-axis (glue) violation
    #[error("Glue axis violation: {0}")]
    GlueAxisViolation(String),

    /// Invariant Q violation
    #[error("Invariant Q violation: {0}")]
    InvariantViolation(String),

    /// Telemetry initialization failed
    #[error("Telemetry initialization failed: {0}")]
    TelemetryInitFailed(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic error
    #[error("System error: {0}")]
    Other(#[from] anyhow::Error),
}

impl SystemError {
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SystemError::TimeAxisViolation(_)
                | SystemError::ChangeEngine(_)
                | SystemError::ProjectionCompiler(_)
        )
    }

    /// Check if this error requires system shutdown
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            SystemError::InitializationFailed(_)
                | SystemError::MetaOntologyLoadFailed(_)
                | SystemError::InvariantViolation(_)
        )
    }
}
