//! Digital Signatures module
//!
//! Provides post-quantum digital signatures using Dilithium algorithm (NIST standardized).
//! Used for authentication and non-repudiation in quantum-safe systems.

use thiserror::Error;
use serde::{Deserialize, Serialize};
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{PublicKey, SecretKey};

/// Signature-related errors
#[derive(Error, Debug)]
pub enum SigError {
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Invalid signature")]
    InvalidSignature,
}

pub type Result<T> = std::result::Result<T, SigError>;

/// Trait for Post-Quantum Digital Signatures
pub trait QuantumSig {
    /// Generate a new keypair for signing
    fn keygen(&self) -> Result<(Vec<u8>, Vec<u8>)>;

    /// Sign a message with the secret key
    fn sign(&self, sk: &[u8], msg: &[u8]) -> Result<Vec<u8>>;

    /// Verify a signature with the public key
    fn verify(&self, pk: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool>;
}

/// Dilithium Signature implementation (NIST standardized, Lattice-based)
#[derive(Clone, Debug)]
pub struct DilithiumSig;

impl DilithiumSig {
    pub fn new() -> Self {
        Self
    }

    /// Serialize public key to standard format
    pub fn serialize_public_key(pk: &dilithium3::PublicKey) -> Result<Vec<u8>> {
        Ok(pk.as_bytes().to_vec())
    }

    /// Deserialize public key from standard format
    pub fn deserialize_public_key(bytes: &[u8]) -> Result<dilithium3::PublicKey> {
        dilithium3::PublicKey::from_bytes(bytes)
            .ok_or_else(|| SigError::InvalidKeyFormat("Invalid public key bytes".to_string()))
    }

    /// Serialize secret key to standard format
    pub fn serialize_secret_key(sk: &dilithium3::SecretKey) -> Result<Vec<u8>> {
        Ok(sk.as_bytes().to_vec())
    }

    /// Deserialize secret key from standard format
    pub fn deserialize_secret_key(bytes: &[u8]) -> Result<dilithium3::SecretKey> {
        dilithium3::SecretKey::from_bytes(bytes)
            .ok_or_else(|| SigError::InvalidKeyFormat("Invalid secret key bytes".to_string()))
    }

    /// Serialize signature to standard format
    pub fn serialize_signature(sig: &[u8]) -> Result<Vec<u8>> {
        Ok(sig.to_vec())
    }

    /// Deserialize signature from standard format
    pub fn deserialize_signature(bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(bytes.to_vec())
    }
}

impl Default for DilithiumSig {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantumSig for DilithiumSig {
    fn keygen(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let (pk, sk) = dilithium3::keypair();
        Ok((
            Self::serialize_public_key(&pk)?,
            Self::serialize_secret_key(&sk)?,
        ))
    }

    fn sign(&self, sk_bytes: &[u8], msg: &[u8]) -> Result<Vec<u8>> {
        let sk = Self::deserialize_secret_key(sk_bytes)?;
        let sig = dilithium3::sign(msg, &sk);
        Self::serialize_signature(&sig)
    }

    fn verify(&self, pk_bytes: &[u8], msg: &[u8], sig_bytes: &[u8]) -> Result<bool> {
        let pk = Self::deserialize_public_key(pk_bytes)?;
        let sig = Self::deserialize_signature(sig_bytes)?;
        Ok(dilithium3::open(msg, &sig, &pk).is_ok())
    }
}

/// Signature metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureMetadata {
    pub algorithm: String,      // "Dilithium3"
    pub public_key_size: usize, // ~1472 bytes for Dilithium3
    pub secret_key_size: usize, // ~4000 bytes for Dilithium3
    pub signature_size: usize,  // ~2701 bytes for Dilithium3
    pub created_at: String,
}

impl SignatureMetadata {
    pub fn dilithium3() -> Self {
        Self {
            algorithm: "Dilithium3".to_string(),
            public_key_size: 1472,
            secret_key_size: 4000,
            signature_size: 2701,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.algorithm != "Dilithium3" {
            return Err(SigError::InvalidKeyFormat("Unknown algorithm".to_string()));
        }
        if self.signature_size == 0 {
            return Err(SigError::InvalidSignature);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dilithium_keygen() {
        let sig = DilithiumSig::new();
        let (pk, sk) = sig.keygen().expect("keygen failed");
        assert_eq!(pk.len(), 1472, "Public key size mismatch");
        assert_eq!(sk.len(), 4000, "Secret key size mismatch");
    }

    #[test]
    fn test_dilithium_sign_verify() {
        let sig = DilithiumSig::new();
        let (pk, sk) = sig.keygen().expect("keygen failed");
        let msg = b"Test message";
        let signature = sig.sign(&sk, msg).expect("signing failed");
        assert_eq!(signature.len(), 2701, "Signature size mismatch");
        let valid = sig.verify(&pk, msg, &signature).expect("verification failed");
        assert!(valid, "Signature verification failed");
    }

    #[test]
    fn test_dilithium_wrong_message() {
        let sig = DilithiumSig::new();
        let (pk, sk) = sig.keygen().expect("keygen failed");
        let msg = b"Test message";
        let signature = sig.sign(&sk, msg).expect("signing failed");
        let wrong_msg = b"Wrong message";
        let valid = sig.verify(&pk, wrong_msg, &signature).expect("verification failed");
        assert!(!valid, "Invalid signature should not verify");
    }

    #[test]
    fn test_metadata_validation() {
        let meta = SignatureMetadata::dilithium3();
        meta.validate().expect("metadata validation failed");
    }

    #[test]
    fn test_invalid_metadata() {
        let mut meta = SignatureMetadata::dilithium3();
        meta.signature_size = 0;
        assert!(meta.validate().is_err());
    }
}
