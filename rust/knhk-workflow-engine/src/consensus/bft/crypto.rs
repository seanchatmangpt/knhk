//! Cryptographic primitives for BFT consensus
//!
//! This module provides:
//! - Digital signatures for message authentication
//! - Signature verification for Byzantine fault detection
//! - Key management for nodes

use super::*;
use ed25519_dalek::{Keypair as Ed25519Keypair, PublicKey, SecretKey, Signature as Ed25519Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cryptographic key pair
pub struct KeyPair {
    /// Ed25519 keypair
    keypair: Ed25519Keypair,
}

impl KeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let mut csprng = rand::rngs::OsRng;
        let keypair = Ed25519Keypair::generate(&mut csprng);
        Self { keypair }
    }

    /// Create from secret key bytes
    pub fn from_bytes(secret_bytes: &[u8]) -> ConsensusResult<Self> {
        let secret = SecretKey::from_bytes(secret_bytes)
            .map_err(|e| ConsensusError::Internal(format!("Invalid secret key: {}", e)))?;

        let public = PublicKey::from(&secret);
        let keypair = Ed25519Keypair { secret, public };

        Ok(Self { keypair })
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.keypair.public.to_bytes()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        let signature = self.keypair.sign(message);
        Signature {
            bytes: signature.to_bytes(),
        }
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        self.keypair.public
    }
}

/// Digital signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Signature bytes
    pub bytes: [u8; 64],
}

impl Signature {
    /// Verify signature against public key
    pub fn verify(&self, message: &[u8], public_key: &PublicKey) -> bool {
        let signature = match Ed25519Signature::from_bytes(&self.bytes) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        public_key.verify(message, &signature).is_ok()
    }
}

/// Signature verifier for BFT consensus
pub struct SignatureVerifier {
    /// Mapping from node ID to public key
    public_keys: HashMap<NodeId, PublicKey>,
}

impl SignatureVerifier {
    /// Create a new signature verifier
    pub fn new() -> Self {
        Self {
            public_keys: HashMap::new(),
        }
    }

    /// Add a public key for a node
    pub fn add_public_key(&mut self, node_id: NodeId, public_key: PublicKey) {
        self.public_keys.insert(node_id, public_key);
    }

    /// Verify a signature from a node
    pub fn verify(&self, node_id: NodeId, message: &[u8], signature: &Signature) -> bool {
        if let Some(public_key) = self.public_keys.get(&node_id) {
            signature.verify(message, public_key)
        } else {
            false
        }
    }

    /// Verify signatures from multiple nodes (quorum verification)
    pub fn verify_quorum(
        &self,
        message: &[u8],
        signatures: &HashMap<NodeId, Signature>,
        quorum_size: usize,
    ) -> bool {
        let mut valid_signatures = 0;

        for (node_id, signature) in signatures {
            if self.verify(*node_id, message, signature) {
                valid_signatures += 1;
            }
        }

        valid_signatures >= quorum_size
    }
}

impl Default for SignatureVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Cryptographic provider for BFT consensus
pub struct CryptoProvider {
    /// Our keypair
    keypair: KeyPair,

    /// Signature verifier
    verifier: SignatureVerifier,
}

impl CryptoProvider {
    /// Create a new crypto provider with random keypair
    pub fn new() -> Self {
        Self {
            keypair: KeyPair::generate(),
            verifier: SignatureVerifier::new(),
        }
    }

    /// Create from existing keypair
    pub fn with_keypair(keypair: KeyPair) -> Self {
        Self {
            keypair,
            verifier: SignatureVerifier::new(),
        }
    }

    /// Add a public key for a peer
    pub fn add_peer_key(&mut self, node_id: NodeId, public_key: PublicKey) {
        self.verifier.add_public_key(node_id, public_key);
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.keypair.sign(message)
    }

    /// Verify a signature from a peer
    pub fn verify(&self, node_id: NodeId, message: &[u8], signature: &Signature) -> bool {
        self.verifier.verify(node_id, message, signature)
    }

    /// Verify quorum of signatures
    pub fn verify_quorum(
        &self,
        message: &[u8],
        signatures: &HashMap<NodeId, Signature>,
        quorum_size: usize,
    ) -> bool {
        self.verifier.verify_quorum(message, signatures, quorum_size)
    }

    /// Get our public key
    pub fn public_key(&self) -> PublicKey {
        self.keypair.public_key()
    }
}

impl Default for CryptoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        let public_bytes = keypair.public_key_bytes();
        assert_eq!(public_bytes.len(), 32);
    }

    #[test]
    fn test_signature_sign_verify() {
        let keypair = KeyPair::generate();
        let message = b"Hello, Byzantine world!";

        let signature = keypair.sign(message);
        assert!(signature.verify(message, &keypair.public_key()));

        // Different message should fail
        let wrong_message = b"Wrong message";
        assert!(!signature.verify(wrong_message, &keypair.public_key()));
    }

    #[test]
    fn test_signature_verifier() {
        let keypair1 = KeyPair::generate();
        let keypair2 = KeyPair::generate();

        let mut verifier = SignatureVerifier::new();
        verifier.add_public_key(NodeId::new(1), keypair1.public_key());
        verifier.add_public_key(NodeId::new(2), keypair2.public_key());

        let message = b"Test message";
        let sig1 = keypair1.sign(message);
        let sig2 = keypair2.sign(message);

        assert!(verifier.verify(NodeId::new(1), message, &sig1));
        assert!(verifier.verify(NodeId::new(2), message, &sig2));

        // Wrong node ID should fail
        assert!(!verifier.verify(NodeId::new(2), message, &sig1));
    }

    #[test]
    fn test_quorum_verification() {
        let mut crypto = CryptoProvider::new();
        let keypair2 = KeyPair::generate();
        let keypair3 = KeyPair::generate();

        crypto.add_peer_key(NodeId::new(2), keypair2.public_key());
        crypto.add_peer_key(NodeId::new(3), keypair3.public_key());

        let message = b"Quorum test";
        let mut signatures = HashMap::new();
        signatures.insert(NodeId::new(2), keypair2.sign(message));
        signatures.insert(NodeId::new(3), keypair3.sign(message));

        // Quorum of 2 should succeed
        assert!(crypto.verify_quorum(message, &signatures, 2));

        // Quorum of 3 should fail (only have 2 signatures)
        assert!(!crypto.verify_quorum(message, &signatures, 3));
    }
}
