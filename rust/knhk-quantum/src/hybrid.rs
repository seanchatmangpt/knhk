//! Hybrid Classical + Quantum Cryptography
//!
//! Provides transitional cryptographic schemes that combine classical and post-quantum
//! algorithms for defense-in-depth. This enables interim security during quantum migration.

use thiserror::Error;
use serde::{Deserialize, Serialize};
use crate::kem::{QuantumKEM, KyberKEM, KEMError};
use crate::sig::{QuantumSig, DilithiumSig, SigError};
use hex;

/// Hybrid cryptography errors
#[derive(Error, Debug)]
pub enum HybridError {
    #[error("KEM error: {0}")]
    KEM(#[from] KEMError),
    #[error("Signature error: {0}")]
    Signature(#[from] SigError),
    #[error("Hybrid encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Hybrid decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Fallback failed: classical crypto unavailable")]
    FallbackFailed,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, HybridError>;

/// Hybrid encryption combining X25519 (classical) + Kyber (quantum-safe)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridEncryption {
    /// Classical public key (X25519)
    pub classical_pk: Vec<u8>,
    /// Quantum public key (Kyber)
    pub quantum_pk: Vec<u8>,
    /// Combined shared secret (XOR of classical and quantum components)
    pub combined_secret: Vec<u8>,
}

impl HybridEncryption {
    /// Generate hybrid keypair (classical + quantum)
    pub fn keygen() -> Result<(HybridEncryption, Vec<u8>, Vec<u8>)> {
        // Generate classical keypair (X25519)
        let classical_sk = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
        let classical_pk = x25519_dalek::PublicKey::from(&classical_sk);

        // Generate quantum keypair (Kyber)
        let kem = KyberKEM::new();
        let (quantum_pk, quantum_sk) = kem.keygen()?;

        let hybrid = HybridEncryption {
            classical_pk: classical_pk.as_bytes().to_vec(),
            quantum_pk,
            combined_secret: vec![],
        };

        let classical_sk_bytes = classical_sk.as_bytes().to_vec();

        Ok((hybrid, classical_sk_bytes, quantum_sk))
    }

    /// Encapsulate under hybrid keypair
    pub fn encapsulate(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let kem = KyberKEM::new();

        // Quantum encapsulation
        let (quantum_ss, quantum_ct) = kem.encapsulate(&self.quantum_pk)?;

        // For classical component, we use the public key directly as ephemeral
        let ephemeral_sk = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
        let ephemeral_pk = x25519_dalek::PublicKey::from(&ephemeral_sk);

        let classical_pk =
            x25519_dalek::PublicKey::from(
                <[u8; 32]>::try_from(self.classical_pk.as_slice())
                    .map_err(|_| HybridError::EncryptionFailed("Invalid classical key".to_string()))?
            );

        let classical_ss = ephemeral_sk.diffie_hellman(&classical_pk);

        // Combine: XOR both shared secrets for defense-in-depth
        let combined = xor_secrets(classical_ss.as_bytes(), &quantum_ss);

        // Ciphertext includes both ephemeral classical PK and quantum ciphertext
        let mut ciphertext = Vec::new();
        ciphertext.extend_from_slice(ephemeral_pk.as_bytes());
        ciphertext.extend_from_slice(&quantum_ct);

        Ok((combined, ciphertext))
    }

    /// Decapsulate under hybrid keypair
    pub fn decapsulate(
        &self,
        classical_sk: &[u8],
        quantum_sk: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>> {
        if ciphertext.len() < 32 {
            return Err(HybridError::DecryptionFailed(
                "Ciphertext too short".to_string(),
            ));
        }

        // Extract ephemeral classical public key and quantum ciphertext
        let ephemeral_pk_bytes = &ciphertext[0..32];
        let quantum_ct = &ciphertext[32..];

        // Classical decapsulation
        let classical_sk_obj = x25519_dalek::StaticSecret::from(
            <[u8; 32]>::try_from(classical_sk)
                .map_err(|_| HybridError::DecryptionFailed("Invalid SK format".to_string()))?
        );

        let ephemeral_pk = x25519_dalek::PublicKey::from(
            <[u8; 32]>::try_from(ephemeral_pk_bytes)
                .map_err(|_| HybridError::DecryptionFailed("Invalid ephemeral key".to_string()))?
        );

        let classical_ss = classical_sk_obj.diffie_hellman(&ephemeral_pk);

        // Quantum decapsulation
        let kem = KyberKEM::new();
        let quantum_ss = kem.decapsulate(quantum_sk, quantum_ct)?;

        // Combine: XOR both shared secrets
        let combined = xor_secrets(classical_ss.as_bytes(), &quantum_ss);

        Ok(combined)
    }
}

/// Hybrid signature combining Ed25519 (classical) + Dilithium (quantum-safe)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridSignature {
    /// Classical public key (Ed25519)
    pub classical_pk: Vec<u8>,
    /// Quantum public key (Dilithium)
    pub quantum_pk: Vec<u8>,
}

impl HybridSignature {
    /// Generate hybrid keypair for signing
    pub fn keygen() -> Result<(HybridSignature, Vec<u8>, Vec<u8>)> {
        // Generate classical keypair (Ed25519)
        let classical_sk = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
        let classical_pk = classical_sk.verifying_key();

        // Generate quantum keypair (Dilithium)
        let sig = DilithiumSig::new();
        let (quantum_pk, quantum_sk) = sig.keygen()?;

        let hybrid = HybridSignature {
            classical_pk: classical_pk.to_bytes().to_vec(),
            quantum_pk,
        };

        Ok((hybrid, classical_sk.to_bytes().to_vec(), quantum_sk))
    }

    /// Sign message with hybrid keypair (both classical and quantum)
    pub fn sign(
        &self,
        classical_sk: &[u8],
        quantum_sk: &[u8],
        msg: &[u8],
    ) -> Result<Vec<u8>> {
        // Classical signature
        let classical_sk_obj = ed25519_dalek::SigningKey::from_bytes(
            <&[u8; 32]>::try_from(classical_sk)
                .map_err(|_| HybridError::Signature(SigError::InvalidKeyFormat(
                    "Invalid classical SK".to_string()
                )))?
        );
        let classical_sig = classical_sk_obj.sign(msg).to_bytes().to_vec();

        // Quantum signature
        let sig = DilithiumSig::new();
        let quantum_sig = sig.sign(quantum_sk, msg)?;

        // Combine both signatures
        let mut combined = Vec::new();
        combined.extend_from_slice(&(classical_sig.len() as u32).to_le_bytes());
        combined.extend_from_slice(&classical_sig);
        combined.extend_from_slice(&quantum_sig);

        Ok(combined)
    }

    /// Verify hybrid signature (both must verify for acceptance)
    pub fn verify(&self, msg: &[u8], combined_sig: &[u8]) -> Result<bool> {
        if combined_sig.len() < 4 {
            return Ok(false);
        }

        // Extract classical and quantum signatures
        let classical_sig_len = u32::from_le_bytes(
            <[u8; 4]>::try_from(&combined_sig[0..4])
                .map_err(|_| HybridError::Signature(SigError::InvalidSignature))?
        ) as usize;

        if combined_sig.len() < 4 + classical_sig_len {
            return Ok(false);
        }

        let classical_sig = &combined_sig[4..4 + classical_sig_len];
        let quantum_sig = &combined_sig[4 + classical_sig_len..];

        // Verify classical signature
        let classical_pk = ed25519_dalek::VerifyingKey::from_bytes(
            <&[u8; 32]>::try_from(self.classical_pk.as_slice())
                .map_err(|_| HybridError::Signature(SigError::InvalidKeyFormat(
                    "Invalid classical PK".to_string()
                )))?
        ).map_err(|_| HybridError::Signature(SigError::InvalidSignature))?;

        let classical_sig_obj = ed25519_dalek::Signature::from_slice(classical_sig)
            .map_err(|_| HybridError::Signature(SigError::InvalidSignature))?;

        let classical_valid = classical_pk.verify(msg, &classical_sig_obj).is_ok();

        // Verify quantum signature
        let sig = DilithiumSig::new();
        let quantum_valid = sig.verify(&self.quantum_pk, msg, quantum_sig)?;

        // Both must verify for acceptance (AND logic)
        Ok(classical_valid && quantum_valid)
    }
}

/// Helper function to XOR two byte vectors for combined secrets
fn xor_secrets(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x ^ y)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_encryption_keygen() {
        let (hybrid, _sk_classical, _sk_quantum) = HybridEncryption::keygen()
            .expect("keygen failed");
        assert_eq!(hybrid.classical_pk.len(), 32);
        assert_eq!(hybrid.quantum_pk.len(), 1184);
    }

    #[test]
    fn test_hybrid_encryption_full_flow() {
        let (hybrid, sk_classical, sk_quantum) = HybridEncryption::keygen()
            .expect("keygen failed");
        let (combined_secret, ciphertext) = hybrid.encapsulate().expect("encapsulation failed");
        let recovered = hybrid.decapsulate(&sk_classical, &sk_quantum, &ciphertext)
            .expect("decapsulation failed");
        assert_eq!(combined_secret, recovered);
    }

    #[test]
    fn test_hybrid_signature_keygen() {
        let (hybrid, _sk_classical, _sk_quantum) = HybridSignature::keygen()
            .expect("keygen failed");
        assert_eq!(hybrid.classical_pk.len(), 32);
        assert_eq!(hybrid.quantum_pk.len(), 1472);
    }

    #[test]
    fn test_hybrid_signature_full_flow() {
        let (hybrid, sk_classical, sk_quantum) = HybridSignature::keygen()
            .expect("keygen failed");
        let msg = b"Test hybrid signature";
        let sig = hybrid.sign(&sk_classical, &sk_quantum, msg).expect("signing failed");
        let valid = hybrid.verify(msg, &sig).expect("verification failed");
        assert!(valid);
    }

    #[test]
    fn test_hybrid_signature_wrong_message() {
        let (hybrid, sk_classical, sk_quantum) = HybridSignature::keygen()
            .expect("keygen failed");
        let msg = b"Test hybrid signature";
        let sig = hybrid.sign(&sk_classical, &sk_quantum, msg).expect("signing failed");
        let wrong_msg = b"Wrong message";
        let valid = hybrid.verify(wrong_msg, &sig).expect("verification failed");
        assert!(!valid);
    }
}
