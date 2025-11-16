//! Formal Verification Module for KNHK Governance Layer
//!
//! Provides mathematical proof capabilities using SMT solvers and theorem proving
//! to verify that autonomic adaptations are lawful and maintain system invariants.
//!
//! **Architecture**:
//! - SMT solver integration for policy verification
//! - Runtime invariant checking
//! - Proof certificate storage and validation
//! - Type-level compile-time verification
//!
//! **Key Guarantees**:
//! - Policy lattice operations preserve lattice laws
//! - Doctrine projection correctness: Q ∧ policy → policy'
//! - μ-kernel constraints: τ ≤ 8, max_run_len ≤ 8
//! - ΔΣ overlay safety before application
//! - Session isolation and trace determinism
//!
//! # Example
//!
//! ```rust
//! use knhk_workflow_engine::verification::*;
//!
//! // Create verifier
//! let verifier = PolicyVerifier::new()?;
//!
//! // Verify policy at compile-time where possible
//! const POLICY: VerifiedPolicy = VerifiedPolicy::new(
//!     PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Hard).unwrap())
//! );
//!
//! // Runtime verification with proof
//! async fn verify_overlay(overlay: DeltaSigma<ProofPending>) -> Result<DeltaSigma<Proven>, Error> {
//!     let proof = verifier.verify_overlay(&overlay).await?;
//!
//!     if proof.is_valid() {
//!         Ok(overlay.into_proven_with_proof(proof)?)
//!     } else {
//!         Err(Error::ProofFailed(proof.counterexample()))
//!     }
//! }
//! ```

pub mod invariants;
pub mod proof_certificates;
pub mod smt_solver;
pub mod type_level;

// Re-exports for convenience
pub use invariants::{
    Invariant, InvariantChecker, InvariantViolation, RuntimeInvariant, SessionInvariant,
};
pub use proof_certificates::{
    ProofCache, ProofCertificate, ProofCertificateStore, ProofMetadata, ProofStatus,
};
pub use smt_solver::{
    PolicyVerifier, SmtFormula, SmtProof, SmtResult, SmtSolver, VerificationError,
};
pub use type_level::{Bounds, Proven as TypeProven, Unverified, VerificationState, Verified};

use crate::error::{WorkflowError, WorkflowResult};

/// Verification configuration
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Enable SMT solver verification
    pub enable_smt: bool,
    /// SMT solver timeout (milliseconds)
    pub smt_timeout_ms: u64,
    /// Enable proof caching
    pub enable_cache: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Enable runtime invariant checking
    pub enable_invariants: bool,
    /// Verification strictness level
    pub strictness: VerificationStrictness,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            enable_smt: true,
            smt_timeout_ms: 100, // 100ms for new proofs
            enable_cache: true,
            max_cache_size: 10_000,
            enable_invariants: true,
            strictness: VerificationStrictness::Production,
        }
    }
}

/// Verification strictness level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStrictness {
    /// Development mode - warnings only
    Development,
    /// Testing mode - fail on critical violations
    Testing,
    /// Production mode - fail on any violation
    Production,
}

/// Verification result with proof
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether verification succeeded
    pub valid: bool,
    /// Proof certificate (if successful)
    pub proof: Option<ProofCertificate>,
    /// Counterexample (if failed)
    pub counterexample: Option<String>,
    /// Verification duration (milliseconds)
    pub duration_ms: u64,
    /// Cached proof (if from cache)
    pub from_cache: bool,
}

impl VerificationResult {
    /// Create successful result
    pub fn success(proof: ProofCertificate, duration_ms: u64, from_cache: bool) -> Self {
        Self {
            valid: true,
            proof: Some(proof),
            counterexample: None,
            duration_ms,
            from_cache,
        }
    }

    /// Create failed result
    pub fn failure(counterexample: String, duration_ms: u64) -> Self {
        Self {
            valid: false,
            proof: None,
            counterexample: Some(counterexample),
            duration_ms,
            from_cache: false,
        }
    }

    /// Check if result is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get proof or error
    pub fn proof_or_err(&self) -> WorkflowResult<&ProofCertificate> {
        self.proof.as_ref().ok_or_else(|| {
            WorkflowError::Validation(format!(
                "Verification failed: {}",
                self.counterexample.as_deref().unwrap_or("unknown error")
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_config_default() {
        let config = VerificationConfig::default();
        assert!(config.enable_smt);
        assert!(config.enable_cache);
        assert!(config.enable_invariants);
        assert_eq!(config.strictness, VerificationStrictness::Production);
    }

    #[test]
    fn test_verification_result_success() {
        let proof = ProofCertificate::mock();
        let result = VerificationResult::success(proof, 50, false);
        assert!(result.is_valid());
        assert!(result.proof.is_some());
        assert!(result.counterexample.is_none());
    }

    #[test]
    fn test_verification_result_failure() {
        let result = VerificationResult::failure("Policy violation".to_string(), 50);
        assert!(!result.is_valid());
        assert!(result.proof.is_none());
        assert!(result.counterexample.is_some());
    }
}
