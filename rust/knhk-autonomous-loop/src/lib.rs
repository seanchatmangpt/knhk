//! # KNHK Autonomous Evolution Loop
//!
//! The self-aware, self-healing heart of KNHK that continuously evolves Σ without human intervention.
//!
//! ## Overview
//!
//! This crate implements two complementary autonomous loop systems:
//!
//! ### 1. Advanced Autonomous Loop (NEW - Recommended)
//!
//! Production-grade system with advanced features:
//! - **Self-triggering**: Feedback system drives changes based on metrics
//! - **Self-healing**: Recovers from promotion failures automatically
//! - **Adaptive**: Learns and adjusts change frequency based on success rate
//! - **Auditable**: Cryptographically signed logs with blockchain-style chaining
//! - **Observable**: Rich telemetry via OpenTelemetry
//!
//! ```text
//! Observe O → Detect patterns → Propose ΔΣ → Validate → Compile → Promote → Repeat
//!          ↑                                                              ↓
//!          └────────────────── Feedback Loop ─────────────────────────────┘
//! ```
//!
//! ### 2. Basic Evolution Loop (Legacy)
//!
//! Original six-step cycle implementation for backward compatibility:
//! ```text
//! observe (O, A)
//!   → detect patterns
//!   → propose ΔΣ
//!   → validate against Q
//!   → compile Π
//!   → promote Σ*
//!   → (loop)
//! ```
//!
//! ## Usage (Advanced Loop - Recommended)
//!
//! ```rust,no_run
//! use knhk_autonomous_loop::{AutonomousLoopController, LoopConfig};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = LoopConfig {
//!         max_proposals: 10,
//!         cycle_interval: Duration::from_secs(60),
//!         max_change_rate: 1.0,
//!         failure_threshold: 0.5,
//!         recovery_strategy: knhk_autonomous_loop::RecoveryStrategy::Rollback,
//!     };
//!
//!     let controller = AutonomousLoopController::new(config).await?;
//!     controller.run().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Usage (Basic Loop - Legacy)
//!
//! ```rust,no_run
//! use knhk_autonomous_loop::{LoopEngine, AutonomousLoopConfig, LoopDependencies};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = AutonomousLoopConfig {
//!         cycle_interval: Duration::from_secs(60),
//!         auto_promote: true,
//!         ..Default::default()
//!     };
//!
//!     let dependencies = LoopDependencies::new(/* ... */);
//!     let engine = LoopEngine::new(config, dependencies)?;
//!
//!     let handle = tokio::spawn(async move {
//!         engine.run().await
//!     });
//!
//!     handle.await??;
//!     Ok(())
//! }
//! ```

// ==================== Advanced Autonomous Loop (NEW) ====================
// Production-grade self-aware, self-healing autonomous loop

pub mod adaptive_strategy;
pub mod audit_trail;
pub mod feedback_system;
pub mod loop_controller;
pub mod self_healing;

// Re-exports for advanced loop
pub use adaptive_strategy::{AdaptiveStrategy, CycleOutcome, StrategyHistory};
pub use audit_trail::{AuditEntry, AuditEvent, AuditTrail};
pub use feedback_system::{FeedbackSystem, FeedbackThresholds, TriggerReason};
pub use loop_controller::{
    AutonomousLoopController, CycleResult, LoopConfig, LoopState,
};
pub use self_healing::{RecoveryStrategy, SelfHealer};

// ==================== Basic Evolution Loop (LEGACY) ====================
// Original six-step cycle implementation

pub mod config;
pub mod cycle;
pub mod dependencies;
pub mod health;
pub mod loop_engine;
pub mod telemetry;

// Legacy exports
pub use config::AutonomousLoopConfig;
pub use cycle::{CycleStep, EvolutionCycle};
pub use cycle::CycleResult as LegacyCycleResult; // Renamed to avoid conflict
pub use dependencies::LoopDependencies;
pub use health::LoopHealth;
pub use loop_engine::{start_autonomous_loop, LoopEngine, LoopHandle};
pub use telemetry::LoopTelemetry;

// ==================== Common Types ====================

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Result type for autonomous loop operations (Advanced Loop)
pub type Result<T> = std::result::Result<T, AutonomousLoopError>;

/// Legacy result type for backward compatibility
pub type LegacyResult<T> = std::result::Result<T, EvolutionError>;

/// Errors that can occur during autonomous evolution (Advanced Loop)
#[derive(Debug, thiserror::Error)]
pub enum AutonomousLoopError {
    #[error("Pattern mining failed: {0}")]
    PatternMiningFailed(String),

    #[error("Proposal generation failed: {0}")]
    ProposalGenerationFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Promotion failed: {0}")]
    PromotionFailed(String),

    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),

    #[error("Audit trail error: {0}")]
    AuditTrailError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Legacy errors (for backward compatibility)
#[derive(Debug, thiserror::Error)]
pub enum EvolutionError {
    #[error("No validation receipt found for snapshot")]
    NoReceipt,

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("Pattern mining failed: {0}")]
    PatternMiningFailed(String),

    #[error("Proposal generation failed: {0}")]
    ProposalFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Promotion failed: {0}")]
    PromotionFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Telemetry error: {0}")]
    TelemetryError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Conversion from AutonomousLoopError to EvolutionError (for backward compatibility)
impl From<AutonomousLoopError> for EvolutionError {
    fn from(err: AutonomousLoopError) -> Self {
        match err {
            AutonomousLoopError::PatternMiningFailed(s) => EvolutionError::PatternMiningFailed(s),
            AutonomousLoopError::ProposalGenerationFailed(s) => EvolutionError::ProposalFailed(s),
            AutonomousLoopError::ValidationFailed(s) => EvolutionError::ValidationFailed(s),
            AutonomousLoopError::CompilationFailed(s) => EvolutionError::CompilationFailed(s),
            AutonomousLoopError::PromotionFailed(s) => EvolutionError::PromotionFailed(s),
            AutonomousLoopError::ConfigurationError(s) => EvolutionError::Unknown(s),
            AutonomousLoopError::RecoveryFailed(s) => EvolutionError::RollbackFailed(s),
            AutonomousLoopError::AuditTrailError(s) => EvolutionError::TelemetryError(s),
            AutonomousLoopError::IoError(e) => EvolutionError::IoError(e),
            AutonomousLoopError::SerializationError(e) => EvolutionError::SerializationError(e),
            AutonomousLoopError::Other(e) => EvolutionError::Unknown(e.to_string()),
        }
    }
}

/// Unique identifier for a Σ snapshot (Advanced Loop)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SigmaSnapshotId(pub String);

impl SigmaSnapshotId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Legacy snapshot identifier (32-byte hash)
pub type LegacySigmaSnapshotId = [u8; 32];

/// Helper to create snapshot ID from string (Legacy)
pub fn snapshot_id_from_str(s: &str) -> std::result::Result<LegacySigmaSnapshotId, EvolutionError> {
    let bytes = hex::decode(s).map_err(|e| EvolutionError::Unknown(e.to_string()))?;
    if bytes.len() != 32 {
        return Err(EvolutionError::Unknown(format!(
            "Invalid snapshot ID length: {} (expected 32)",
            bytes.len()
        )));
    }
    let mut id = [0u8; 32];
    id.copy_from_slice(&bytes);
    Ok(id)
}

/// Helper to convert snapshot ID to hex string (Legacy)
pub fn snapshot_id_to_string(id: &LegacySigmaSnapshotId) -> String {
    hex::encode(id)
}

// ==================== Advanced Loop Common Types ====================

/// Represents a detected pattern from observations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_id: String,
    pub confidence: f64,
    pub frequency: u32,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub metadata: serde_json::Value,
}

/// Collection of detected patterns
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectedPatterns {
    pub patterns: Vec<DetectedPattern>,
}

impl DetectedPatterns {
    pub fn count(&self) -> usize {
        self.patterns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

/// A proposed change to Σ (ΔΣ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaSigmaProposal {
    pub proposal_id: String,
    pub snapshot_id: SigmaSnapshotId,
    pub changes: Vec<SchemaChange>,
    pub justification: String,
    pub patterns: Vec<String>,
    pub created_at: SystemTime,
}

/// Individual schema change within a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaChange {
    AddClass { name: String, properties: Vec<String> },
    RemoveClass { name: String },
    AddProperty { class: String, property: String, range: String },
    RemoveProperty { class: String, property: String },
    AddRelation { subject: String, predicate: String, object: String },
}

/// Result of validating a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub snapshot_id: SigmaSnapshotId,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
    pub validated_at: SystemTime,
}

/// Compiled projection ready for promotion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledProjection {
    pub snapshot_id: SigmaSnapshotId,
    pub projection_type: String,
    pub artifact_path: String,
    pub compiled_at: SystemTime,
}

/// Current metrics for feedback system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CurrentMetrics {
    pub schema_mismatches: u32,
    pub guard_violations: u32,
    pub performance_regression_detected: bool,
    pub new_patterns: DetectedPatterns,
    pub total_observations: u64,
    pub error_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_id_roundtrip() {
        let original = [42u8; 32];
        let string = snapshot_id_to_string(&original);
        let recovered = snapshot_id_from_str(&string).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_snapshot_id_invalid_length() {
        let result = snapshot_id_from_str("aabbcc");
        assert!(result.is_err());
    }
}
