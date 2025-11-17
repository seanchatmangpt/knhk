//! AHI User Space Interface
//!
//! This module implements AHI as a constrained user space that must
//! request resources from the μ-kernel. AHI components cannot directly
//! modify Σ* or generate receipts - they must submit proven overlays.

use crate::core::{MuError, MuKernel};
use crate::overlay::DeltaSigma;
use crate::sigma::SigmaHash;
use core::marker::PhantomData;

/// Tick quota for AHI operations
#[derive(Debug, Clone, Copy)]
pub struct TickQuota {
    /// Maximum ticks allowed
    pub limit: u64,
    /// Ticks consumed so far
    pub consumed: u64,
}

impl TickQuota {
    /// Create a new tick quota
    pub const fn new(limit: u64) -> Self {
        Self { limit, consumed: 0 }
    }

    /// Check if quota is exhausted
    pub const fn is_exhausted(&self) -> bool {
        self.consumed >= self.limit
    }

    /// Remaining quota
    pub const fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    /// Consume ticks from quota
    pub fn consume(&mut self, ticks: u64) -> Result<(), QuotaExceeded> {
        if self.consumed + ticks > self.limit {
            return Err(QuotaExceeded {
                requested: ticks,
                available: self.remaining(),
            });
        }

        self.consumed += ticks;
        Ok(())
    }
}

/// Quota exceeded error
#[derive(Debug, Clone, Copy)]
pub struct QuotaExceeded {
    /// Ticks requested
    pub requested: u64,
    /// Ticks available
    pub available: u64,
}

/// AHI overlay proof trait (distinct from kernel OverlayProof)
///
/// Proofs that an AHI overlay is valid and safe to apply.
/// Different proof types provide different guarantees:
/// - TickProof: Guarantees tick budget not exceeded
/// - InvariantProof: Guarantees invariants preserved
/// - AuthorizationProof: Guarantees proper authorization
pub trait AhiOverlayProof: Sized {
    /// Type of proof
    const PROOF_TYPE: ProofType;

    /// Verify the proof is valid
    fn verify(&self) -> Result<(), ProofError>;

    /// Get proof hash (for receipt)
    fn proof_hash(&self) -> [u8; 32];
}

/// Proof types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProofType {
    /// Tick budget proof
    TickBudget = 0,
    /// Invariant preservation proof
    Invariant = 1,
    /// Authorization proof
    Authorization = 2,
    /// Composite proof (multiple proofs combined)
    Composite = 3,
}

/// Proof errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofError {
    /// Proof verification failed
    VerificationFailed,
    /// Invalid proof structure
    InvalidStructure,
    /// Proof expired
    Expired,
}

/// Tick budget proof (guarantees ≤ N ticks)
#[derive(Debug, Clone)]
pub struct TickBudgetProof<const MAX_TICKS: u64> {
    /// Measured tick count
    pub measured_ticks: u64,
    /// Timestamp of measurement
    pub timestamp: u64,
}

impl<const MAX_TICKS: u64> AhiOverlayProof for TickBudgetProof<MAX_TICKS> {
    const PROOF_TYPE: ProofType = ProofType::TickBudget;

    fn verify(&self) -> Result<(), ProofError> {
        if self.measured_ticks > MAX_TICKS {
            return Err(ProofError::VerificationFailed);
        }
        Ok(())
    }

    fn proof_hash(&self) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(&self.measured_ticks.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// Invariant preservation proof
#[derive(Debug, Clone)]
pub struct InvariantProof {
    /// Invariants checked
    pub invariants: heapless::Vec<u16, 16>,
    /// Hash of pre-state
    pub pre_hash: SigmaHash,
    /// Hash of post-state
    pub post_hash: SigmaHash,
}

impl AhiOverlayProof for InvariantProof {
    const PROOF_TYPE: ProofType = ProofType::Invariant;

    fn verify(&self) -> Result<(), ProofError> {
        // In production, would verify invariants hold
        // For now, just check we have invariants
        if self.invariants.is_empty() {
            return Err(ProofError::VerificationFailed);
        }
        Ok(())
    }

    fn proof_hash(&self) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(&self.pre_hash.0);
        hasher.update(&self.post_hash.0);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// AHI proven overlay (ΔΣ with proof)
#[derive(Debug)]
pub struct AhiProvenOverlay<P: AhiOverlayProof> {
    /// The overlay (ΔΣ)
    pub overlay: DeltaSigma,
    /// Proof that overlay is safe
    pub proof: P,
}

impl<P: AhiOverlayProof> AhiProvenOverlay<P> {
    /// Create a new proven overlay
    pub fn new(overlay: DeltaSigma, proof: P) -> Self {
        Self { overlay, proof }
    }

    /// Verify the overlay is valid
    pub fn verify(&self) -> Result<(), ProofError> {
        self.proof.verify()
    }
}

/// Submit token (returned after successful overlay submission)
#[derive(Debug, Clone, Copy)]
pub struct SubmitToken {
    /// Submission ID
    pub id: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Proof hash
    pub proof_hash: [u8; 32],
}

/// Tick grant (returned after requesting ticks)
#[derive(Debug, Clone, Copy)]
pub struct TickGrant {
    /// Granted ticks
    pub ticks: u64,
    /// Expiration timestamp
    pub expires_at: u64,
}

/// Proof factory for creating proofs
pub struct ProofFactory<'k> {
    _kernel: PhantomData<&'k MuKernel>,
}

impl<'k> ProofFactory<'k> {
    /// Create a new proof factory
    pub fn new() -> Self {
        Self {
            _kernel: PhantomData,
        }
    }

    /// Create a tick budget proof
    pub fn tick_proof<const MAX_TICKS: u64>(
        &self,
        measured_ticks: u64,
    ) -> Result<TickBudgetProof<MAX_TICKS>, ProofError> {
        if measured_ticks > MAX_TICKS {
            return Err(ProofError::VerificationFailed);
        }

        Ok(TickBudgetProof {
            measured_ticks,
            timestamp: 0, // Would get from kernel
        })
    }

    /// Create an invariant proof
    pub fn invariant_proof(
        &self,
        invariants: heapless::Vec<u16, 16>,
        pre_hash: SigmaHash,
        post_hash: SigmaHash,
    ) -> InvariantProof {
        InvariantProof {
            invariants,
            pre_hash,
            post_hash,
        }
    }
}

impl Default for ProofFactory<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// AHI context (per-operation context for AHI code)
pub struct AhiContext<'k> {
    /// Reference to kernel (read-only)
    kernel: &'k MuKernel,
    /// Tick quota for this context
    tick_quota: TickQuota,
    /// Proof factory
    proof_factory: ProofFactory<'k>,
    /// Submission counter
    submit_counter: u64,
}

impl<'k> AhiContext<'k> {
    /// Create a new AHI context
    pub fn new(kernel: &'k MuKernel, tick_quota: u64) -> Self {
        Self {
            kernel,
            tick_quota: TickQuota::new(tick_quota),
            proof_factory: ProofFactory::new(),
            submit_counter: 0,
        }
    }

    /// Submit a proven overlay to the kernel
    ///
    /// # Arguments
    /// - overlay: The proven overlay to submit
    ///
    /// # Returns
    /// A submit token on success
    pub fn submit_overlay<P: AhiOverlayProof>(
        &mut self,
        overlay: AhiProvenOverlay<P>,
    ) -> Result<SubmitToken, AhiError> {
        // Verify the proof
        overlay.verify().map_err(AhiError::ProofError)?;

        // Estimate tick cost (simplified)
        let estimated_ticks = 10; // Would analyze overlay

        // Consume quota
        self.tick_quota
            .consume(estimated_ticks)
            .map_err(AhiError::QuotaExceeded)?;

        // Generate submit token
        self.submit_counter += 1;
        let token = SubmitToken {
            id: self.submit_counter,
            timestamp: 0, // Would get from kernel
            proof_hash: overlay.proof.proof_hash(),
        };

        Ok(token)
    }

    /// Request additional tick quota
    ///
    /// # Arguments
    /// - count: Number of ticks requested
    ///
    /// # Returns
    /// A tick grant on success
    pub fn request_ticks(&mut self, count: u64) -> Result<TickGrant, QuotaExceeded> {
        // Check if request is reasonable
        if count > crate::ahi::AHI_DEFAULT_QUOTA {
            return Err(QuotaExceeded {
                requested: count,
                available: crate::ahi::AHI_DEFAULT_QUOTA,
            });
        }

        // Grant ticks (in production, would check with kernel)
        Ok(TickGrant {
            ticks: count,
            expires_at: 0, // Would set expiration
        })
    }

    /// Get current tick quota
    pub fn quota(&self) -> &TickQuota {
        &self.tick_quota
    }

    /// Get proof factory
    pub fn proof_factory(&self) -> &ProofFactory<'k> {
        &self.proof_factory
    }
}

/// AHI errors
#[derive(Debug)]
pub enum AhiError {
    /// Kernel error
    Kernel(MuError),
    /// Proof error
    ProofError(ProofError),
    /// Quota exceeded
    QuotaExceeded(QuotaExceeded),
    /// Invalid overlay
    InvalidOverlay,
    /// Unauthorized operation
    Unauthorized,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::MuKernel;
    use crate::sigma::{SigmaCompiled, SigmaPointer};

    #[test]
    fn test_tick_quota() {
        let mut quota = TickQuota::new(100);

        assert_eq!(quota.remaining(), 100);
        assert!(!quota.is_exhausted());

        quota.consume(30).unwrap();
        assert_eq!(quota.remaining(), 70);

        quota.consume(80).unwrap_err();
        assert_eq!(quota.consumed, 30);
    }

    #[test]
    fn test_tick_budget_proof() {
        let proof = TickBudgetProof::<8> {
            measured_ticks: 5,
            timestamp: 123,
        };

        assert!(proof.verify().is_ok());
        assert_eq!(proof.proof_hash().len(), 32);
    }

    #[test]
    fn test_tick_budget_proof_exceeds() {
        let proof = TickBudgetProof::<8> {
            measured_ticks: 10,
            timestamp: 123,
        };

        assert!(proof.verify().is_err());
    }

    #[test]
    fn test_invariant_proof() {
        let mut invariants = heapless::Vec::new();
        invariants.push(1).unwrap();

        let proof = InvariantProof {
            invariants,
            pre_hash: crate::sigma::SigmaHash([0; 32]),
            post_hash: crate::sigma::SigmaHash([1; 32]),
        };

        assert!(proof.verify().is_ok());
    }

    #[test]
    fn test_proof_factory() {
        let factory = ProofFactory::new();

        let tick_proof = factory.tick_proof::<8>(5).unwrap();
        assert_eq!(tick_proof.measured_ticks, 5);

        let tick_proof_fail = factory.tick_proof::<8>(10);
        assert!(tick_proof_fail.is_err());
    }

    #[test]
    fn test_ahi_context() {
        let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
        let kernel = MuKernel::new(sigma_ptr);

        let mut ctx = AhiContext::new(&kernel, 1000);

        assert_eq!(ctx.quota().remaining(), 1000);

        let grant = ctx.request_ticks(100).unwrap();
        assert_eq!(grant.ticks, 100);
    }
}
