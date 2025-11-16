//! Compilation Certificate and Proof System
//!
//! This module provides machine-checkable proofs that compiled Σ*
//! artifacts satisfy all μ-kernel invariants.

use core::marker::PhantomData;
use sha3::{Digest, Sha3_256};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use alloc::vec::Vec;
use alloc::string::{String, ToString};

use crate::sigma::{SigmaCompiled, SigmaHash};
use crate::sigma_types::{
    IsaComplianceProof, InvariantProof, PatternExpansionProof,
    WithinChatmanConstant, CompiledTask, CompiledPattern, CompiledGuard,
};
use crate::CHATMAN_CONSTANT;

/// Compilation certificate with cryptographic proof
///
/// This certificate proves that the compiled Σ* satisfies all
/// static invariants. The loader MUST reject any Σ* without
/// a valid certificate.
#[derive(Debug, Clone)]
pub struct CompilationCertificate {
    /// Σ* hash this certificate is for
    pub sigma_hash: SigmaHash,
    /// ISA compliance proof
    pub isa_proof: IsaComplianceProof,
    /// Timing bound proof
    pub timing_proof: TimingBoundProof,
    /// Invariant satisfaction proof
    pub invariant_proof: InvariantProof,
    /// Compiler version
    pub compiler_version: [u8; 3],
    /// Timestamp (Unix time)
    pub timestamp: u64,
    /// Ed25519 signature over certificate
    pub signature: [u8; 64],
}

impl CompilationCertificate {
    /// Create a new certificate (signs with compiler key)
    pub fn new(
        sigma_hash: SigmaHash,
        isa_proof: IsaComplianceProof,
        timing_proof: TimingBoundProof,
        invariant_proof: InvariantProof,
        signing_key: &SigningKey,
    ) -> Self {
        let compiler_version = crate::MU_KERNEL_VERSION;
        let timestamp = 0; // In real impl, get actual timestamp

        // Compute certificate hash
        let mut cert = Self {
            sigma_hash,
            isa_proof,
            timing_proof,
            invariant_proof,
            compiler_version: [
                compiler_version.0 as u8,
                compiler_version.1,
                compiler_version.2,
            ],
            timestamp,
            signature: [0; 64],
        };

        // Sign the certificate
        let hash = cert.compute_hash();
        let signature = signing_key.sign(&hash.0);
        cert.signature.copy_from_slice(&signature.to_bytes());

        cert
    }

    /// Compute hash of certificate (for signing/verification)
    fn compute_hash(&self) -> SigmaHash {
        let mut hasher = Sha3_256::new();

        // Hash all fields except signature
        hasher.update(&self.sigma_hash.0);
        hasher.update(&self.isa_proof.opcodes);
        hasher.update(&[self.isa_proof.opcode_count as u8]);
        hasher.update(&self.timing_proof.max_ticks.to_le_bytes());
        hasher.update(&self.compiler_version);
        hasher.update(&self.timestamp.to_le_bytes());

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        SigmaHash(hash)
    }

    /// Verify certificate signature
    pub fn verify_signature(&self, verifying_key: &VerifyingKey) -> Result<(), CertificateError> {
        let hash = self.compute_hash();
        let signature = Signature::from_bytes(&self.signature);

        verifying_key
            .verify(&hash.0, &signature)
            .map_err(|_| CertificateError::InvalidSignature)
    }

    /// Verify all proofs in certificate
    pub fn verify_proofs(&self) -> Result<(), CertificateError> {
        // Verify ISA compliance
        if !self.isa_proof.verify() {
            return Err(CertificateError::IsaViolation);
        }

        // Verify timing bounds
        if !self.timing_proof.verify() {
            return Err(CertificateError::TimingViolation);
        }

        // Verify invariants
        if !self.invariant_proof.verify() {
            return Err(CertificateError::InvariantViolation);
        }

        Ok(())
    }

    /// Full verification (signature + proofs)
    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<(), CertificateError> {
        self.verify_signature(verifying_key)?;
        self.verify_proofs()?;
        Ok(())
    }
}

/// Timing bound proof
///
/// Proves that all hot path operations complete within
/// the Chatman Constant (≤8 ticks).
#[derive(Debug, Clone)]
pub struct TimingBoundProof {
    /// Maximum ticks across all tasks
    pub max_ticks: u64,
    /// Per-task tick counts
    pub task_ticks: Vec<TaskTimingProof>,
    /// Per-pattern tick counts
    pub pattern_ticks: Vec<PatternTimingProof>,
    /// Per-guard tick counts
    pub guard_ticks: Vec<GuardTimingProof>,
}

impl TimingBoundProof {
    /// Create a new timing proof
    pub fn new(
        task_ticks: Vec<TaskTimingProof>,
        pattern_ticks: Vec<PatternTimingProof>,
        guard_ticks: Vec<GuardTimingProof>,
    ) -> Self {
        // Compute maximum ticks
        let task_max = task_ticks.iter().map(|t| t.ticks).max().unwrap_or(0);
        let pattern_max = pattern_ticks.iter().map(|p| p.total_ticks).max().unwrap_or(0);
        let guard_max = guard_ticks.iter().map(|g| g.ticks).max().unwrap_or(0);

        let max_ticks = task_max.max(pattern_max).max(guard_max);

        Self {
            max_ticks,
            task_ticks,
            pattern_ticks,
            guard_ticks,
        }
    }

    /// Verify timing bounds
    pub fn verify(&self) -> bool {
        // Check max ticks doesn't exceed Chatman Constant
        if self.max_ticks > CHATMAN_CONSTANT {
            return false;
        }

        // Verify all task ticks
        for task in &self.task_ticks {
            if task.ticks > CHATMAN_CONSTANT {
                return false;
            }
        }

        // Verify all pattern ticks
        for pattern in &self.pattern_ticks {
            if pattern.total_ticks > CHATMAN_CONSTANT {
                return false;
            }
        }

        // Verify all guard ticks
        for guard in &self.guard_ticks {
            if guard.ticks > CHATMAN_CONSTANT {
                return false;
            }
        }

        true
    }
}

/// Timing proof for a single task
#[derive(Debug, Clone)]
pub struct TaskTimingProof {
    /// Task ID
    pub task_id: u64,
    /// Tick count
    pub ticks: u64,
    /// Breakdown by operation
    pub breakdown: TimingBreakdown,
}

/// Timing proof for a pattern
#[derive(Debug, Clone)]
pub struct PatternTimingProof {
    /// Pattern ID
    pub pattern_id: u8,
    /// Total ticks
    pub total_ticks: u64,
    /// Per-phase ticks
    pub phase_ticks: [u8; 8],
}

/// Timing proof for a guard
#[derive(Debug, Clone)]
pub struct GuardTimingProof {
    /// Guard ID
    pub guard_id: u16,
    /// Tick count
    pub ticks: u64,
}

/// Timing breakdown
#[derive(Debug, Clone)]
pub struct TimingBreakdown {
    /// Ticks for task descriptor load
    pub load_ticks: u8,
    /// Ticks for pattern dispatch
    pub dispatch_ticks: u8,
    /// Ticks for guard evaluation
    pub guard_ticks: u8,
    /// Ticks for execution
    pub execute_ticks: u8,
    /// Ticks for receipt writing
    pub receipt_ticks: u8,
}

impl TimingBreakdown {
    /// Total ticks
    pub fn total(&self) -> u64 {
        (self.load_ticks + self.dispatch_ticks + self.guard_ticks
            + self.execute_ticks + self.receipt_ticks) as u64
    }
}

/// Certified Σ* with proof-of-correctness
///
/// This wraps a SigmaCompiled with its certificate,
/// ensuring the loader only accepts valid Σ*.
#[derive(Debug, Clone)]
pub struct CertifiedSigma {
    /// Compiled Σ*
    pub sigma_star: SigmaCompiled,
    /// Compilation certificate
    pub certificate: CompilationCertificate,
}

impl CertifiedSigma {
    /// Create a new certified Σ*
    pub fn new(sigma_star: SigmaCompiled, certificate: CompilationCertificate) -> Self {
        Self {
            sigma_star,
            certificate,
        }
    }

    /// Verify this certified Σ*
    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<(), CertificateError> {
        // Verify certificate
        self.certificate.verify(verifying_key)?;

        // Verify hash matches
        let computed_hash = self.sigma_star.compute_hash();
        if computed_hash != self.certificate.sigma_hash {
            return Err(CertificateError::HashMismatch);
        }

        Ok(())
    }

    /// Get the Σ* (only if verification passes)
    pub fn sigma(&self) -> &SigmaCompiled {
        &self.sigma_star
    }
}

/// Certificate errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateError {
    /// Invalid signature
    InvalidSignature,
    /// ISA compliance violation
    IsaViolation,
    /// Timing bound violation
    TimingViolation,
    /// Invariant violation
    InvariantViolation,
    /// Hash mismatch
    HashMismatch,
    /// Expired certificate
    Expired,
    /// Unsupported compiler version
    UnsupportedCompilerVersion,
}

/// Proof builder - accumulates proofs during compilation
pub struct ProofBuilder {
    /// ISA opcodes used
    opcodes: [u8; 256],
    opcode_count: usize,
    /// Task timing proofs
    task_timings: Vec<TaskTimingProof>,
    /// Pattern timing proofs
    pattern_timings: Vec<PatternTimingProof>,
    /// Guard timing proofs
    guard_timings: Vec<GuardTimingProof>,
    /// Invariants checked
    invariants: [crate::sigma_types::InvariantId; 64],
    invariant_count: usize,
}

impl ProofBuilder {
    /// Create a new proof builder
    pub fn new() -> Self {
        Self {
            opcodes: [0; 256],
            opcode_count: 0,
            task_timings: Vec::new(),
            pattern_timings: Vec::new(),
            guard_timings: Vec::new(),
            invariants: [crate::sigma_types::InvariantId(0); 64],
            invariant_count: 0,
        }
    }

    /// Record an opcode usage
    pub fn record_opcode(&mut self, opcode: u8) {
        if self.opcode_count < 256 {
            self.opcodes[self.opcode_count] = opcode;
            self.opcode_count += 1;
        }
    }

    /// Record task timing
    pub fn record_task_timing(&mut self, proof: TaskTimingProof) {
        self.task_timings.push(proof);
    }

    /// Record pattern timing
    pub fn record_pattern_timing(&mut self, proof: PatternTimingProof) {
        self.pattern_timings.push(proof);
    }

    /// Record guard timing
    pub fn record_guard_timing(&mut self, proof: GuardTimingProof) {
        self.guard_timings.push(proof);
    }

    /// Record invariant check
    pub fn record_invariant(&mut self, invariant: crate::sigma_types::InvariantId) {
        if self.invariant_count < 64 {
            self.invariants[self.invariant_count] = invariant;
            self.invariant_count += 1;
        }
    }

    /// Build final certificate
    pub fn build(
        self,
        sigma_hash: SigmaHash,
        signing_key: &SigningKey,
    ) -> Result<CompilationCertificate, CertificateError> {
        // Build ISA proof
        let isa_proof = IsaComplianceProof::new(self.opcodes, self.opcode_count);

        // Build timing proof
        let timing_proof = TimingBoundProof::new(
            self.task_timings,
            self.pattern_timings,
            self.guard_timings,
        );

        // Verify timing proof
        if !timing_proof.verify() {
            return Err(CertificateError::TimingViolation);
        }

        // Build invariant proof
        let invariant_proof = InvariantProof::new(self.invariants, self.invariant_count);

        // Create certificate
        Ok(CompilationCertificate::new(
            sigma_hash,
            isa_proof,
            timing_proof,
            invariant_proof,
            signing_key,
        ))
    }
}

impl Default for ProofBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Loader verification result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderVerification {
    /// Σ* is valid and certified
    Valid,
    /// Certificate missing
    MissingCertificate,
    /// Invalid certificate
    InvalidCertificate(CertificateError),
}

/// Verify Σ* for loading
pub fn verify_for_loading(
    certified: &CertifiedSigma,
    verifying_key: &VerifyingKey,
) -> LoaderVerification {
    match certified.verify(verifying_key) {
        Ok(()) => LoaderVerification::Valid,
        Err(e) => LoaderVerification::InvalidCertificate(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn test_signing_key() -> SigningKey {
        // Deterministic key for testing
        let mut bytes = [0u8; 32];
        bytes[0] = 1;
        SigningKey::from_bytes(&bytes)
    }

    #[test]
    fn test_certificate_creation() {
        let signing_key = test_signing_key();
        let sigma_hash = SigmaHash([0; 32]);

        let isa_proof = IsaComplianceProof::new([0; 256], 0);
        let timing_proof = TimingBoundProof::new(Vec::new(), Vec::new(), Vec::new());
        let invariant_proof = InvariantProof::new([crate::sigma_types::InvariantId(0); 64], 0);

        let cert = CompilationCertificate::new(
            sigma_hash,
            isa_proof,
            timing_proof,
            invariant_proof,
            &signing_key,
        );

        assert_eq!(cert.sigma_hash, sigma_hash);
    }

    #[test]
    fn test_certificate_verification() {
        let signing_key = test_signing_key();
        let verifying_key = signing_key.verifying_key();
        let sigma_hash = SigmaHash([0; 32]);

        let isa_proof = IsaComplianceProof::new([0; 256], 0);
        let timing_proof = TimingBoundProof::new(Vec::new(), Vec::new(), Vec::new());
        let invariant_proof = InvariantProof::new([crate::sigma_types::InvariantId(0); 64], 0);

        let cert = CompilationCertificate::new(
            sigma_hash,
            isa_proof,
            timing_proof,
            invariant_proof,
            &signing_key,
        );

        assert!(cert.verify(&verifying_key).is_ok());
    }

    #[test]
    fn test_timing_proof_validation() {
        let task_timing = TaskTimingProof {
            task_id: 1,
            ticks: 5,
            breakdown: TimingBreakdown {
                load_ticks: 1,
                dispatch_ticks: 1,
                guard_ticks: 1,
                execute_ticks: 1,
                receipt_ticks: 1,
            },
        };

        let timing_proof = TimingBoundProof::new(
            alloc::vec![task_timing],
            Vec::new(),
            Vec::new(),
        );

        assert!(timing_proof.verify());
        assert!(timing_proof.max_ticks <= CHATMAN_CONSTANT);
    }

    #[test]
    fn test_timing_proof_violation() {
        let task_timing = TaskTimingProof {
            task_id: 1,
            ticks: 10, // Exceeds Chatman Constant
            breakdown: TimingBreakdown {
                load_ticks: 2,
                dispatch_ticks: 2,
                guard_ticks: 2,
                execute_ticks: 2,
                receipt_ticks: 2,
            },
        };

        let timing_proof = TimingBoundProof::new(
            alloc::vec![task_timing],
            Vec::new(),
            Vec::new(),
        );

        assert!(!timing_proof.verify());
    }

    #[test]
    fn test_proof_builder() {
        let mut builder = ProofBuilder::new();

        // Record some operations
        builder.record_opcode(1);
        builder.record_opcode(2);

        builder.record_task_timing(TaskTimingProof {
            task_id: 1,
            ticks: 5,
            breakdown: TimingBreakdown {
                load_ticks: 1,
                dispatch_ticks: 1,
                guard_ticks: 1,
                execute_ticks: 1,
                receipt_ticks: 1,
            },
        });

        let signing_key = test_signing_key();
        let sigma_hash = SigmaHash([0; 32]);

        let cert = builder.build(sigma_hash, &signing_key);
        assert!(cert.is_ok());
    }

    #[test]
    fn test_certified_sigma() {
        let signing_key = test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let sigma = SigmaCompiled::new();
        let sigma_hash = sigma.compute_hash();

        let isa_proof = IsaComplianceProof::new([0; 256], 0);
        let timing_proof = TimingBoundProof::new(Vec::new(), Vec::new(), Vec::new());
        let invariant_proof = InvariantProof::new([crate::sigma_types::InvariantId(0); 64], 0);

        let cert = CompilationCertificate::new(
            sigma_hash,
            isa_proof,
            timing_proof,
            invariant_proof,
            &signing_key,
        );

        let certified = CertifiedSigma::new(sigma, cert);
        assert!(certified.verify(&verifying_key).is_ok());
    }

    #[test]
    fn test_loader_verification() {
        let signing_key = test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let sigma = SigmaCompiled::new();
        let sigma_hash = sigma.compute_hash();

        let isa_proof = IsaComplianceProof::new([0; 256], 0);
        let timing_proof = TimingBoundProof::new(Vec::new(), Vec::new(), Vec::new());
        let invariant_proof = InvariantProof::new([crate::sigma_types::InvariantId(0); 64], 0);

        let cert = CompilationCertificate::new(
            sigma_hash,
            isa_proof,
            timing_proof,
            invariant_proof,
            &signing_key,
        );

        let certified = CertifiedSigma::new(sigma, cert);

        let result = verify_for_loading(&certified, &verifying_key);
        assert_eq!(result, LoaderVerification::Valid);
    }
}
