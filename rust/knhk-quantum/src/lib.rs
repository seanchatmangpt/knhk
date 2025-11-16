//! Phase 7: Quantum-Safe Cryptography Module
//!
//! This module provides post-quantum cryptography implementations for the KNHK platform,
//! ensuring security against both classical and quantum adversaries.
//!
//! # Principles
//!
//! - **NIST Standardization**: All algorithms follow NIST PQC Round 3 standards
//! - **Hybrid Approach**: Combines classical and quantum algorithms for interim deployment
//! - **Graceful Fallback**: Operates with classical crypto if quantum unavailable
//! - **Observable**: Full OpenTelemetry integration for production observability
//! - **Zero Trust**: No assumptions about adversary computational power
//!
//! # Modules
//!
//! - `kem`: Key Encapsulation Mechanism (Kyber-based)
//! - `sig`: Digital Signatures (Dilithium-based)
//! - `hybrid`: Hybrid classical+quantum algorithms
//! - `nist`: NIST PQC compliance validation
//! - `integration`: Integration with Phase 5 platform

pub mod kem;
pub mod sig;
pub mod hybrid;
pub mod nist;
pub mod integration;

// Re-exports for common use
pub use kem::{QuantumKEM, KyberKEM, KEMError};
pub use sig::{QuantumSig, DilithiumSig, SigError};
pub use hybrid::{HybridEncryption, HybridSignature};
pub use nist::{NISTLevel, validate_nist_compliance};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// DOCTRINE alignment: Q (Hard Invariants)
/// Post-quantum security is NOT optional - it's mandatory for future-facing systems
pub struct QuantumSecurityContext {
    /// Minimum NIST level for acceptable algorithms
    pub min_nist_level: nist::NISTLevel,
    /// Enable hybrid mode (classical + quantum)
    pub hybrid_enabled: bool,
    /// Enable telemetry for quantum operations
    pub telemetry_enabled: bool,
}

impl Default for QuantumSecurityContext {
    fn default() -> Self {
        Self {
            min_nist_level: nist::NISTLevel::Level1,
            hybrid_enabled: true,
            telemetry_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_defaults() {
        let ctx = QuantumSecurityContext::default();
        assert_eq!(ctx.min_nist_level, nist::NISTLevel::Level1);
        assert!(ctx.hybrid_enabled);
        assert!(ctx.telemetry_enabled);
    }
}
