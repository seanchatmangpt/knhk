//! KNHK Change Engine - Autonomous Ontology Evolution
//!
//! This crate implements the Change Engine (Σ² → ΔΣ²) that automatically detects
//! schema drift and proposes validated ontology changes without human intervention.
//!
//! # Architecture
//!
//! The Change Engine consists of four main components:
//!
//! 1. **Pattern Miner** - Detects schema drift, repeated structures, and guard violations
//! 2. **Proposer** - Generates ΔΣ² change proposals (rule-based + optional LLM)
//! 3. **Validator** - Validates proposals against invariants Q
//! 4. **Executor** - Applies validated changes to create new snapshots
//!
//! # Example
//!
//! ```rust,no_run
//! use knhk_change_engine::ChangeExecutor;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create executor (includes pattern miner, proposer, and validator)
//! let executor = ChangeExecutor::new();
//!
//! // Execute full change cycle: detect → propose → validate → commit
//! let result = executor.execute_change_cycle().await?;
//!
//! println!("Proposals submitted: {}", result.proposals_submitted);
//! println!("Proposals validated: {}", result.proposals_validated);
//! println!("Proposals applied: {}", result.proposals_applied);
//! # Ok(())
//! # }
//! ```
//!
//! # Invariants Q
//!
//! The validator enforces five critical invariants:
//!
//! 1. **Type Soundness** - All triples conform to declared schema
//! 2. **No Retrocausation** - Immutability guarantees temporal consistency
//! 3. **Guard Preservation** - Security and business rules are maintained
//! 4. **SLO Preservation** - Performance remains ≤8 ticks for hot path
//! 5. **Determinism** - Projections produce consistent results

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod pattern_miner;
pub mod proposer;
pub mod validator;
pub mod executor;

// Re-export main types
pub use pattern_miner::{PatternMiner, DetectedPatterns, SchemaMismatch, RepeatedStructure, GuardViolation, PerfRegression};
pub use proposer::{DeltaSigmaProposal, DeltaSigmaProposer, PolicyRule};
pub use validator::{DeltaSigmaValidator, ValidationResult, InvariantsQ};
pub use executor::ChangeExecutor;

/// Result type used throughout the change engine
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the change engine
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Pattern detection failed
    #[error("Pattern detection failed: {0}")]
    PatternDetection(String),

    /// Proposal generation failed
    #[error("Proposal generation failed: {0}")]
    ProposalGeneration(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    Validation(String),

    /// Execution failed
    #[error("Execution failed: {0}")]
    Execution(String),

    /// LLM API error
    #[error("LLM API error: {0}")]
    LlmApi(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
