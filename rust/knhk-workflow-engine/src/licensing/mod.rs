//! License enforcement module for KNHK workflow engine
//!
//! Provides type-level license enforcement with zero runtime overhead.
//!
//! # DOCTRINE ALIGNMENT
//! - Principle: Π (Projection/Market), Q (Invariants)
//! - Covenant 2: Invariants Are Law (license limits enforced at compile-time)
//! - Covenant 6: Observations Drive Everything (audit trail for compliance)
//!
//! # Architecture
//! ```
//! Free:       $0/month      (10 workflows, 1 concurrent, 1k/day, CPU+SIMD)
//! Pro:        $2k/month     (1k workflows, 100 concurrent, 1M/day, +GPU)
//! Enterprise: $50k+/year    (unlimited, +FPGA, +Byzantine, +on-prem)
//! ```

use std::marker::PhantomData;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

pub mod tiers;
pub mod token;
pub mod enforcement;
pub mod audit;
pub mod validation;

/// License trait (implemented by each tier)
pub trait License: Sized + Send + Sync + 'static {
    /// Maximum number of workflows
    const MAX_WORKFLOWS: usize;

    /// Maximum concurrent executions
    const MAX_CONCURRENT: usize;

    /// Maximum daily executions
    const MAX_DAILY_EXECUTIONS: usize;

    /// Support SLA in hours
    const SUPPORT_SLA_HOURS: u32;

    /// GPU acceleration included
    const INCLUDES_GPU: bool;

    /// FPGA acceleration included
    const INCLUDES_FPGA: bool;

    /// Quantum-safe cryptography included
    const INCLUDES_QUANTUM: bool;

    /// Byzantine consensus included
    const INCLUDES_BYZANTINE: bool;

    /// Validate license token
    fn validate(&self, token: &LicenseToken) -> Result<(), LicenseError>;
}

/// Free tier (personal/learning)
pub struct FreeTier;

impl License for FreeTier {
    const MAX_WORKFLOWS: usize = 10;
    const MAX_CONCURRENT: usize = 1;
    const MAX_DAILY_EXECUTIONS: usize = 1_000;
    const SUPPORT_SLA_HOURS: u32 = 24;

    const INCLUDES_GPU: bool = false;
    const INCLUDES_FPGA: bool = false;
    const INCLUDES_QUANTUM: bool = false;
    const INCLUDES_BYZANTINE: bool = false;

    fn validate(&self, token: &LicenseToken) -> Result<(), LicenseError> {
        if token.tier != LicenseTier::Free {
            return Err(LicenseError::TierMismatch);
        }
        token.verify_signature()?;
        token.check_expiration()?;
        Ok(())
    }
}

/// Pro tier (startups/SMBs)
pub struct ProTier;

impl License for ProTier {
    const MAX_WORKFLOWS: usize = 1_000;
    const MAX_CONCURRENT: usize = 100;
    const MAX_DAILY_EXECUTIONS: usize = 1_000_000;
    const SUPPORT_SLA_HOURS: u32 = 4;

    const INCLUDES_GPU: bool = true;
    const INCLUDES_FPGA: bool = false;
    const INCLUDES_QUANTUM: bool = true;
    const INCLUDES_BYZANTINE: bool = false;

    fn validate(&self, token: &LicenseToken) -> Result<(), LicenseError> {
        if token.tier != LicenseTier::Pro {
            return Err(LicenseError::TierMismatch);
        }
        token.verify_signature()?;
        token.check_expiration()?;
        Ok(())
    }
}

/// Enterprise tier (Fortune 500)
pub struct EnterpriseTier;

impl License for EnterpriseTier {
    const MAX_WORKFLOWS: usize = usize::MAX;  // Unlimited
    const MAX_CONCURRENT: usize = usize::MAX;
    const MAX_DAILY_EXECUTIONS: usize = usize::MAX;
    const SUPPORT_SLA_HOURS: u32 = 1;

    const INCLUDES_GPU: bool = true;
    const INCLUDES_FPGA: bool = true;
    const INCLUDES_QUANTUM: bool = true;
    const INCLUDES_BYZANTINE: bool = true;

    fn validate(&self, token: &LicenseToken) -> Result<(), LicenseError> {
        if token.tier != LicenseTier::Enterprise {
            return Err(LicenseError::TierMismatch);
        }
        token.verify_signature()?;
        token.check_expiration()?;
        Ok(())
    }
}

/// License tier enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LicenseTier {
    Free,
    Pro,
    Enterprise,
}

/// License token (Ed25519-signed, tamper-proof)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseToken {
    /// Customer ID (SHA-256 hash of email/org)
    pub customer_id: [u8; 32],

    /// License tier
    pub tier: LicenseTier,

    /// Issued timestamp
    pub issued: SystemTime,

    /// Expiration timestamp
    pub expires: SystemTime,

    /// Feature flags
    pub quantum_safe_enabled: bool,
    pub byzantine_consensus_enabled: bool,
    pub gpu_acceleration_enabled: bool,
    pub fpga_acceleration_enabled: bool,

    /// Usage limits (overrides for custom contracts)
    pub max_workflows: Option<usize>,
    pub max_concurrent: Option<usize>,
    pub max_daily_executions: Option<usize>,

    /// Audit trail
    pub issued_by: [u8; 32],
    pub nonce: [u8; 32],

    /// Cryptographic proof
    pub signature: [u8; 64],
}

impl LicenseToken {
    /// Verify Ed25519 signature
    pub fn verify_signature(&self) -> Result<(), LicenseError> {
        // TODO: Implement Ed25519 verification
        // This is a stub - real implementation uses ed25519_dalek
        Ok(())
    }

    /// Check if token is expired
    pub fn check_expiration(&self) -> Result<(), LicenseError> {
        let now = SystemTime::now();
        if now > self.expires {
            return Err(LicenseError::Expired {
                expired_at: self.expires,
            });
        }
        Ok(())
    }

    /// Validate token (signature + expiration)
    pub fn validate(&self) -> Result<(), LicenseError> {
        self.verify_signature()?;
        self.check_expiration()?;
        Ok(())
    }
}

/// License error types
#[derive(Debug, thiserror::Error)]
pub enum LicenseError {
    #[error("License tier mismatch")]
    TierMismatch,

    #[error("License expired at {expired_at:?}")]
    Expired { expired_at: SystemTime },

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid public key")]
    InvalidPublicKey,

    #[error("Signature verification failed")]
    SignatureVerificationFailed,
}

// Re-exports
pub use tiers::{FreeTier, ProTier, EnterpriseTier};
pub use token::LicenseToken;
pub use enforcement::EnforcedEngine;
pub use audit::ExecutionAuditLog;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_ordering() {
        // Tiers should be ordered by capability
        assert!(LicenseTier::Pro > LicenseTier::Free);
        assert!(LicenseTier::Enterprise > LicenseTier::Pro);
    }

    #[test]
    fn test_free_tier_limits() {
        assert_eq!(FreeTier::MAX_WORKFLOWS, 10);
        assert_eq!(FreeTier::MAX_CONCURRENT, 1);
        assert_eq!(FreeTier::MAX_DAILY_EXECUTIONS, 1_000);
        assert!(!FreeTier::INCLUDES_GPU);
        assert!(!FreeTier::INCLUDES_FPGA);
    }

    #[test]
    fn test_pro_tier_limits() {
        assert_eq!(ProTier::MAX_WORKFLOWS, 1_000);
        assert_eq!(ProTier::MAX_CONCURRENT, 100);
        assert_eq!(ProTier::MAX_DAILY_EXECUTIONS, 1_000_000);
        assert!(ProTier::INCLUDES_GPU);
        assert!(!ProTier::INCLUDES_FPGA);
    }

    #[test]
    fn test_enterprise_tier_unlimited() {
        assert_eq!(EnterpriseTier::MAX_WORKFLOWS, usize::MAX);
        assert_eq!(EnterpriseTier::MAX_CONCURRENT, usize::MAX);
        assert_eq!(EnterpriseTier::MAX_DAILY_EXECUTIONS, usize::MAX);
        assert!(EnterpriseTier::INCLUDES_GPU);
        assert!(EnterpriseTier::INCLUDES_FPGA);
    }
}
