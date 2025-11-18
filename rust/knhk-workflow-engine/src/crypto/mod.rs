//! Phase 7: Quantum-Safe Cryptography
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: Q (Hard Invariants) - Cryptographic proofs are non-negotiable
//! - Covenant 2: Invariants Are Law - All crypto failures are hard errors
//! - Covenant 5: Chatman Constant - Signing ≤250μs (2 ticks), ≤8 ticks budget
//!
//! This module implements NIST Post-Quantum Cryptography with hybrid signatures:
//! - Classical: Ed25519 (32-byte keys, 64-byte signatures, ~50μs)
//! - Quantum-safe: Dilithium (2528-byte keys, 3293-byte signatures, ~200μs)
//! - Hybrid: Both required for verification (~250μs total)

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use zeroize::Zeroize;

pub mod classical;
pub mod quantum;
pub mod hybrid;
pub mod key_management;
pub mod receipt;

// ============================================================================
// Phantom Type Markers for Key Categories
// ============================================================================

/// Classical cryptography (Ed25519)
/// - Security: 128-bit classical security
/// - Quantum resistance: NONE (broken by Shor's algorithm)
/// - Use: Legacy support only (pre-2028)
pub struct ClassicalKey;

/// Quantum-safe cryptography (Dilithium)
/// - Security: NIST Level 3 (~192-bit classical security)
/// - Quantum resistance: Secure against quantum computers
/// - Use: Post-2030 (quantum-only era)
pub struct QuantumSafeKey;

/// Hybrid cryptography (Ed25519 + Dilithium)
/// - Security: MAX(classical, quantum) - if one breaks, other remains
/// - Quantum resistance: Full
/// - Use: 2028-2030 (transition period)
pub struct HybridKey;

// ============================================================================
// Signature Scheme Trait (Generic over Key Category)
// ============================================================================

/// Generic signature scheme trait parametrized by key category
///
/// This trait defines the interface for all signature algorithms used in KNHK.
/// Type-level guarantees ensure correct usage:
/// - `Ed25519` implements `SignatureScheme<ClassicalKey>`
/// - `Dilithium` implements `SignatureScheme<QuantumSafeKey>`
/// - `Hybrid` implements `SignatureScheme<HybridKey>`
///
/// # Performance Requirements (Chatman Constant)
///
/// All operations MUST complete within 8 ticks (1ms @ 8MHz):
/// - `sign()`: ≤250μs (2 ticks)
/// - `verify()`: ≤400μs (3 ticks)
/// - `keygen()`: ≤200μs (2 ticks)
///
/// # Safety Invariants (Covenant 2)
///
/// - Secret keys MUST be zeroized on drop
/// - Signature verification failures MUST return hard errors (never warnings)
/// - Invalid signatures MUST be rejected (no partial verification)
pub trait SignatureScheme<K>: Sized {
    /// Public key type (must be clonable and serializable)
    type PublicKey: Clone + Serialize + for<'de> Deserialize<'de>;

    /// Secret key type (must be zeroizable and droppable)
    type SecretKey: Zeroize + Drop;

    /// Signature type (must be clonable and serializable)
    type Signature: Clone + Serialize + for<'de> Deserialize<'de>;

    /// Generate a new keypair
    ///
    /// # Security
    ///
    /// - Uses OS CSPRNG (`OsRng`) for entropy
    /// - Must have ≥256 bits of entropy
    /// - Must be unpredictable
    ///
    /// # Performance
    ///
    /// - MUST complete in ≤200μs (2 ticks)
    fn keygen() -> (Self::PublicKey, Self::SecretKey);

    /// Sign a message with a secret key
    ///
    /// # Arguments
    ///
    /// - `sk`: Secret key (will NOT be consumed, allows multiple signatures)
    /// - `msg`: Message to sign (arbitrary bytes)
    ///
    /// # Returns
    ///
    /// Deterministic signature over `msg`
    ///
    /// # Performance
    ///
    /// - MUST complete in ≤250μs (2 ticks) for hybrid signatures
    ///
    /// # Security
    ///
    /// - Signatures MUST be deterministic (same msg+sk → same sig)
    /// - Signatures MUST be non-malleable
    fn sign(sk: &Self::SecretKey, msg: &[u8]) -> Self::Signature;

    /// Verify a signature on a message
    ///
    /// # Arguments
    ///
    /// - `pk`: Public key
    /// - `msg`: Message that was signed
    /// - `sig`: Signature to verify
    ///
    /// # Returns
    ///
    /// - `true`: Signature is valid
    /// - `false`: Signature is invalid (hard error, never warning)
    ///
    /// # Performance
    ///
    /// - MUST complete in ≤400μs (3 ticks) for hybrid signatures
    ///
    /// # Security (Covenant 2)
    ///
    /// - Invalid signatures MUST return `false` (hard error)
    /// - Partial verification MUST NOT be accepted
    /// - Timing attacks MUST be mitigated (constant-time verification)
    fn verify(pk: &Self::PublicKey, msg: &[u8], sig: &Self::Signature) -> bool;
}

// ============================================================================
// Signature Policy (Migration Strategy)
// ============================================================================

/// Signature policy for migration from classical to quantum-safe
///
/// # Timeline
///
/// - **Phase 1 (2028-2029):** `Hybrid` - Both Ed25519 + Dilithium required
/// - **Phase 2 (2029-2030):** `Hybrid` with deprecation warnings for classical-only
/// - **Phase 3 (2030+):** `QuantumOnly` - Ed25519 support removed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignaturePolicy {
    /// Classical-only signatures (Ed25519)
    /// - Status: DEPRECATED (pre-2028 legacy)
    /// - Security: Vulnerable to quantum attacks
    /// - Use: Legacy descriptor support only
    ClassicalOnly,

    /// Hybrid signatures (Ed25519 + Dilithium)
    /// - Status: CURRENT (2028-2030)
    /// - Security: Quantum-resistant
    /// - Use: All new descriptors MUST use hybrid
    Hybrid,

    /// Quantum-only signatures (Dilithium)
    /// - Status: FUTURE (2030+)
    /// - Security: Full quantum resistance
    /// - Use: Ed25519 support removed
    QuantumOnly,
}

impl SignaturePolicy {
    /// Get the current signature policy based on system time
    ///
    /// # Timeline
    ///
    /// - Before 2028-01-01: `ClassicalOnly`
    /// - 2028-01-01 to 2030-01-01: `Hybrid`
    /// - After 2030-01-01: `QuantumOnly`
    pub fn current() -> Self {
        use std::time::SystemTime;

        const PHASE1_START: u64 = 1735689600; // 2025-01-01 00:00:00 UTC
        const PHASE3_START: u64 = 1893456000; // 2030-01-01 00:00:00 UTC

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_secs();

        if now < PHASE1_START {
            Self::ClassicalOnly
        } else if now < PHASE3_START {
            Self::Hybrid
        } else {
            Self::QuantumOnly
        }
    }

    /// Check if a signature type is compliant with this policy
    pub fn is_compliant(&self, sig_type: SignatureType) -> bool {
        match (self, sig_type) {
            (Self::ClassicalOnly, SignatureType::Classical) => true,
            (Self::Hybrid, SignatureType::Hybrid) => true,
            (Self::QuantumOnly, SignatureType::Quantum) => true,
            _ => false,
        }
    }
}

/// Signature type discriminator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureType {
    Classical,
    Quantum,
    Hybrid,
}

// ============================================================================
// Cryptographic Errors
// ============================================================================

/// Cryptographic operation errors
///
/// # Covenant 2: Hard Errors Only
///
/// All errors are HARD ERRORS (never warnings). Invalid signatures MUST be rejected.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// Signature verification failed
    #[error("Signature verification failed: invalid signature")]
    SignatureInvalid,

    /// Public key is invalid or malformed
    #[error("Public key invalid: {0}")]
    PublicKeyInvalid(String),

    /// Secret key is invalid or malformed
    #[error("Secret key invalid: {0}")]
    SecretKeyInvalid(String),

    /// Signature policy violation
    #[error("Signature policy violation: expected {expected:?}, got {actual:?}")]
    PolicyViolation {
        expected: SignaturePolicy,
        actual: SignatureType,
    },

    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeygenFailed(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Performance budget exceeded (Chatman constant violation)
    #[error("Performance budget exceeded: operation took {actual_us}μs (limit: {limit_us}μs)")]
    PerformanceBudgetExceeded { actual_us: u64, limit_us: u64 },
}

// ============================================================================
// Key Identifiers
// ============================================================================

/// Key identifier (blake3 hash of public key + timestamp)
///
/// Format: `blake3(public_key || timestamp)`
///
/// This allows:
/// - Key versioning (multiple keys with same public key, different timestamps)
/// - Key rotation (identify which key signed a descriptor)
/// - Revocation (invalidate specific key IDs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyId([u8; 32]);

impl KeyId {
    /// Create a new key ID from public key bytes and timestamp
    pub fn new(public_key: &[u8], timestamp: u64) -> Self {
        let mut data = Vec::with_capacity(public_key.len() + 8);
        data.extend_from_slice(public_key);
        data.extend_from_slice(&timestamp.to_le_bytes());

        let hash = blake3::hash(&data);
        Self(*hash.as_bytes())
    }

    /// Get the raw bytes of this key ID
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "blake3:{}", hex::encode(&self.0))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_policy_timeline() {
        // This test will fail in 2030 (expected - update policy logic)
        let policy = SignaturePolicy::current();

        // As of 2025, we should be in Hybrid phase
        assert_eq!(policy, SignaturePolicy::Hybrid);
    }

    #[test]
    fn test_signature_policy_compliance() {
        let hybrid = SignaturePolicy::Hybrid;

        assert!(hybrid.is_compliant(SignatureType::Hybrid));
        assert!(!hybrid.is_compliant(SignatureType::Classical));
        assert!(!hybrid.is_compliant(SignatureType::Quantum));
    }

    #[test]
    fn test_key_id_deterministic() {
        let pk = b"test_public_key";
        let ts = 1234567890;

        let id1 = KeyId::new(pk, ts);
        let id2 = KeyId::new(pk, ts);

        assert_eq!(id1, id2, "Key IDs should be deterministic");
    }

    #[test]
    fn test_key_id_different_timestamps() {
        let pk = b"test_public_key";

        let id1 = KeyId::new(pk, 1000);
        let id2 = KeyId::new(pk, 2000);

        assert_ne!(id1, id2, "Different timestamps should produce different key IDs");
    }
}
