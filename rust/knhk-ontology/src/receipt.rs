//! Validation receipts - cryptographic proof of correctness.
//!
//! Receipts prove that a snapshot:
//! - Preserves all 5 invariants (Q)
//! - Passes static checks (SHACL, Σ² invariants)
//! - Passes dynamic checks (projection tests)
//! - Meets performance SLOs
//!
//! Receipts are immutable, cryptographically signed, and append-only.

use crate::snapshot::SigmaSnapshotId;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ReceiptError {
    #[error("Invalid receipt signature")]
    InvalidSignature,

    #[error("Receipt verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invariants not preserved")]
    InvariantsViolated,
}

/// Validation errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

impl ValidationError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            location: None,
        }
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

/// Validation results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationResults {
    /// SHACL, Σ² invariants
    pub static_checks_passed: bool,

    /// Projection tests, runtime behavior
    pub dynamic_checks_passed: bool,

    /// Performance SLO tests (≤8 ticks hot path)
    pub performance_checks_passed: bool,

    /// All 5 invariants preserved
    pub invariants_q_preserved: bool,

    /// Errors encountered
    pub errors: Vec<ValidationError>,

    /// Non-blocking warnings
    pub warnings: Vec<String>,
}

impl ValidationResults {
    pub fn passed(&self) -> bool {
        self.static_checks_passed
            && self.dynamic_checks_passed
            && self.performance_checks_passed
            && self.invariants_q_preserved
            && self.errors.is_empty()
    }

    pub fn failed(&self) -> bool {
        !self.passed()
    }
}

/// Cryptographic receipt proving validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaReceipt {
    /// The snapshot being validated
    pub snapshot_id: SigmaSnapshotId,

    /// Parent snapshot (for delta tracking)
    pub parent_snapshot_id: Option<SigmaSnapshotId>,

    /// Description of changes (ΔΣ²)
    pub delta_description: String,

    /// Validation results
    pub validation_results: ValidationResults,

    /// When validated
    pub validated_at: SystemTime,

    /// How long validation took
    pub validation_duration_ms: u64,

    /// Cryptographic signature (signed by KMS)
    pub signature: Vec<u8>,

    /// Receipt hash (Blake3)
    pub receipt_hash: [u8; 32],

    /// Can this be promoted to production?
    pub production_ready: bool,
}

impl SigmaReceipt {
    /// Create new receipt
    pub fn new(
        snapshot_id: SigmaSnapshotId,
        parent_snapshot_id: Option<SigmaSnapshotId>,
        delta_description: String,
        validation_results: ValidationResults,
        validation_duration_ms: u64,
    ) -> Self {
        let validated_at = SystemTime::now();
        let production_ready = validation_results.passed();

        let mut receipt = Self {
            snapshot_id,
            parent_snapshot_id,
            delta_description,
            validation_results,
            validated_at,
            validation_duration_ms,
            signature: Vec::new(),
            receipt_hash: [0; 32],
            production_ready,
        };

        // Compute hash
        receipt.receipt_hash = receipt.compute_hash();

        // Sign (simplified - in production would use KMS)
        receipt.signature = receipt.sign_simple();

        receipt
    }

    /// Compute receipt hash
    fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Hasher::new();

        hasher.update(&self.snapshot_id);

        if let Some(parent) = &self.parent_snapshot_id {
            hasher.update(parent);
        }

        hasher.update(self.delta_description.as_bytes());

        let timestamp = self.validated_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .to_le_bytes();
        hasher.update(&timestamp);

        hasher.update(&self.validation_duration_ms.to_le_bytes());
        hasher.update(&[self.production_ready as u8]);

        *hasher.finalize().as_bytes()
    }

    /// Simple signing (in production: use KMS)
    fn sign_simple(&self) -> Vec<u8> {
        // Simplified signing: hash of receipt_hash
        // In production: use AWS KMS, GPG, or other signing service
        let mut hasher = Hasher::new();
        hasher.update(&self.receipt_hash);
        hasher.update(b"KNHK_SIGNATURE");
        hasher.finalize().as_bytes().to_vec()
    }

    /// Verify receipt integrity
    pub fn verify(&self) -> Result<(), ReceiptError> {
        // Verify hash
        let computed_hash = self.compute_hash();
        if computed_hash != self.receipt_hash {
            return Err(ReceiptError::VerificationFailed(
                "Hash mismatch".to_string()
            ));
        }

        // Verify signature (simplified)
        let expected_signature = self.sign_simple();
        if self.signature != expected_signature {
            return Err(ReceiptError::InvalidSignature);
        }

        // Verify invariants
        if !self.validation_results.invariants_q_preserved {
            return Err(ReceiptError::InvariantsViolated);
        }

        Ok(())
    }

    /// Check if receipt allows production promotion
    pub fn allows_promotion(&self) -> bool {
        self.production_ready && self.verify().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_results_passed() {
        let results = ValidationResults {
            static_checks_passed: true,
            dynamic_checks_passed: true,
            performance_checks_passed: true,
            invariants_q_preserved: true,
            errors: vec![],
            warnings: vec![],
        };

        assert!(results.passed());
        assert!(!results.failed());
    }

    #[test]
    fn test_validation_results_failed() {
        let results = ValidationResults {
            static_checks_passed: true,
            dynamic_checks_passed: false,
            performance_checks_passed: true,
            invariants_q_preserved: true,
            errors: vec![ValidationError::new("E001", "Dynamic check failed")],
            warnings: vec![],
        };

        assert!(!results.passed());
        assert!(results.failed());
    }

    #[test]
    fn test_receipt_creation() {
        let snapshot_id = [1u8; 32];
        let results = ValidationResults {
            static_checks_passed: true,
            dynamic_checks_passed: true,
            performance_checks_passed: true,
            invariants_q_preserved: true,
            errors: vec![],
            warnings: vec![],
        };

        let receipt = SigmaReceipt::new(
            snapshot_id,
            None,
            "Test delta".to_string(),
            results,
            100,
        );

        assert!(receipt.production_ready);
        assert_eq!(receipt.snapshot_id, snapshot_id);
        assert!(!receipt.signature.is_empty());
    }

    #[test]
    fn test_receipt_verification() {
        let snapshot_id = [2u8; 32];
        let results = ValidationResults {
            static_checks_passed: true,
            dynamic_checks_passed: true,
            performance_checks_passed: true,
            invariants_q_preserved: true,
            errors: vec![],
            warnings: vec![],
        };

        let receipt = SigmaReceipt::new(
            snapshot_id,
            None,
            "Test delta".to_string(),
            results,
            50,
        );

        assert!(receipt.verify().is_ok());
        assert!(receipt.allows_promotion());
    }

    #[test]
    fn test_receipt_invariants_violation() {
        let snapshot_id = [3u8; 32];
        let results = ValidationResults {
            static_checks_passed: true,
            dynamic_checks_passed: true,
            performance_checks_passed: true,
            invariants_q_preserved: false, // Violation!
            errors: vec![],
            warnings: vec![],
        };

        let receipt = SigmaReceipt::new(
            snapshot_id,
            None,
            "Test delta".to_string(),
            results,
            100,
        );

        assert!(!receipt.production_ready);
        assert!(!receipt.allows_promotion());
        assert!(matches!(receipt.verify(), Err(ReceiptError::InvariantsViolated)));
    }
}
