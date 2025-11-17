//! Key Encapsulation Mechanism (KEM) module
//!
//! Provides post-quantum key agreement using Kyber algorithm (NIST standardized).
//! KEM is used for secure key establishment in encryption schemes.

use thiserror::Error;
use serde::{Deserialize, Serialize};
use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{PublicKey, SecretKey};

/// KEM-related errors
#[derive(Error, Debug)]
pub enum KEMError {
    #[error("Encapsulation failed: {0}")]
    EncapsulationFailed(String),
    #[error("Decapsulation failed: {0}")]
    DecapsulationFailed(String),
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Invalid shared secret")]
    InvalidSharedSecret,
}

pub type Result<T> = std::result::Result<T, KEMError>;

/// Trait for Key Encapsulation Mechanisms
pub trait QuantumKEM {
    /// Generate a new keypair
    fn keygen(&self) -> Result<(Vec<u8>, Vec<u8>)>;

    /// Encapsulate: generate shared secret using public key
    fn encapsulate(&self, pk: &[u8]) -> Result<(Vec<u8>, Vec<u8>)>;

    /// Decapsulate: recover shared secret using secret key
    fn decapsulate(&self, sk: &[u8], ct: &[u8]) -> Result<Vec<u8>>;
}

/// Kyber KEM implementation (NIST standardized, Lattice-based)
#[derive(Clone, Debug)]
pub struct KyberKEM;

impl KyberKEM {
    pub fn new() -> Self {
        Self
    }

    /// Serialize public key to standard format
    pub fn serialize_public_key(pk: &kyber768::PublicKey) -> Result<Vec<u8>> {
        Ok(pk.as_bytes().to_vec())
    }

    /// Deserialize public key from standard format
    pub fn deserialize_public_key(bytes: &[u8]) -> Result<kyber768::PublicKey> {
        kyber768::PublicKey::from_bytes(bytes)
            .ok_or(KEMError::InvalidKeyFormat("Invalid public key bytes".to_string()))
    }

    /// Serialize secret key to standard format
    pub fn serialize_secret_key(sk: &kyber768::SecretKey) -> Result<Vec<u8>> {
        Ok(sk.as_bytes().to_vec())
    }

    /// Deserialize secret key from standard format
    pub fn deserialize_secret_key(bytes: &[u8]) -> Result<kyber768::SecretKey> {
        kyber768::SecretKey::from_bytes(bytes)
            .ok_or(KEMError::InvalidKeyFormat("Invalid secret key bytes".to_string()))
    }
}

impl Default for KyberKEM {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantumKEM for KyberKEM {
    fn keygen(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let (pk, sk) = kyber768::keypair();
        Ok((
            Self::serialize_public_key(&pk)?,
            Self::serialize_secret_key(&sk)?,
        ))
    }

    fn encapsulate(&self, pk_bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let pk = Self::deserialize_public_key(pk_bytes)?;
        let (ss, ct) = kyber768::encapsulate(&pk);
        Ok((ss.as_bytes().to_vec(), ct.as_bytes().to_vec()))
    }

    fn decapsulate(&self, sk_bytes: &[u8], ct_bytes: &[u8]) -> Result<Vec<u8>> {
        let sk = Self::deserialize_secret_key(sk_bytes)?;
        let ct = pqcrypto_kyber::kyber768::Ciphertext::from_bytes(ct_bytes)
            .ok_or(KEMError::InvalidKeyFormat("Invalid ciphertext".to_string()))?;
        let ss = kyber768::decapsulate(&ct, &sk);
        Ok(ss.as_bytes().to_vec())
    }
}

/// KEM Key metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KEMKeyMetadata {
    pub algorithm: String,      // "Kyber768"
    pub public_key_size: usize, // ~1184 bytes for Kyber768
    pub secret_key_size: usize, // ~2400 bytes for Kyber768
    pub shared_secret_size: usize, // 32 bytes
    pub ciphertext_size: usize, // ~1088 bytes for Kyber768
    pub created_at: String,
}

impl KEMKeyMetadata {
    pub fn kyber768() -> Self {
        Self {
            algorithm: "Kyber768".to_string(),
            public_key_size: 1184,
            secret_key_size: 2400,
            shared_secret_size: 32,
            ciphertext_size: 1088,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.algorithm != "Kyber768" {
            return Err(KEMError::InvalidKeyFormat("Unknown algorithm".to_string()));
        }
        if self.shared_secret_size == 0 {
            return Err(KEMError::InvalidSharedSecret);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyber_keygen() {
        let kem = KyberKEM::new();
        let (pk, sk) = kem.keygen().expect("keygen failed");
        assert_eq!(pk.len(), 1184, "Public key size mismatch");
        assert_eq!(sk.len(), 2400, "Secret key size mismatch");
    }

    #[test]
    fn test_kyber_encapsulation() {
        let kem = KyberKEM::new();
        let (pk, _sk) = kem.keygen().expect("keygen failed");
        let (ss1, ct) = kem.encapsulate(&pk).expect("encapsulation failed");
        assert_eq!(ss1.len(), 32, "Shared secret size mismatch");
        assert_eq!(ct.len(), 1088, "Ciphertext size mismatch");
    }

    #[test]
    fn test_kyber_full_flow() {
        let kem = KyberKEM::new();
        let (pk, sk) = kem.keygen().expect("keygen failed");
        let (ss_alice, ct) = kem.encapsulate(&pk).expect("encapsulation failed");
        let ss_bob = kem.decapsulate(&sk, &ct).expect("decapsulation failed");
        assert_eq!(ss_alice, ss_bob, "Shared secrets don't match");
    }

    #[test]
    fn test_metadata_validation() {
        let meta = KEMKeyMetadata::kyber768();
        meta.validate().expect("metadata validation failed");
    }

    #[test]
    fn test_invalid_metadata() {
        let mut meta = KEMKeyMetadata::kyber768();
        meta.shared_secret_size = 0;
        assert!(meta.validate().is_err());
    }
}
