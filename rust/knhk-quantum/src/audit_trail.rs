//! Quantum-Safe Audit Trail Module
//!
//! Provides cryptographically signed audit trails for workflow receipts using ML-DSA
//! (Dilithium) post-quantum signatures. All receipts are signed and verifiable.
//!
//! # DOCTRINE ALIGNMENT
//! - Principle: Q (Hard Invariants) - Security ≥128-bit quantum-safe
//! - Covenant: 2 (Invariants Are Law) + 6 (Observable telemetry)
//! - Validation: Weaver schema + signature verification

use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use crate::sig::{QuantumSig, DilithiumSig, SigError};

/// Audit trail errors
#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Signature error: {0}")]
    SignatureError(#[from] SigError),
    #[error("Invalid receipt: {0}")]
    InvalidReceipt(String),
    #[error("Merkle proof verification failed")]
    MerkleProofFailed,
    #[error("Receipt not found: {0}")]
    ReceiptNotFound(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AuditError>;

/// A receipt representing a workflow operation or event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub workflow_id: String,
    pub operation: String,
    pub timestamp: DateTime<Utc>,
    pub data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Receipt {
    /// Create a new receipt
    pub fn new(
        id: String,
        workflow_id: String,
        operation: String,
        data: Vec<u8>,
    ) -> Self {
        Self {
            id,
            workflow_id,
            operation,
            timestamp: Utc::now(),
            data,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Compute hash of receipt for signing
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(&self.workflow_id);
        hasher.update(&self.operation);
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(&self.data);
        hasher.finalize().to_vec()
    }

    /// Serialize receipt to bytes for signing
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

/// A signed receipt with quantum-safe signature
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedReceipt {
    pub receipt: Receipt,
    pub signature: Vec<u8>,
    pub signer_public_key_id: String,
    pub algorithm: String, // "Dilithium3"
    pub signed_at: DateTime<Utc>,
}

impl SignedReceipt {
    /// Verify the signature on this receipt
    pub fn verify(&self, public_key: &[u8]) -> Result<bool> {
        let sig_engine = DilithiumSig::new();
        let receipt_hash = self.receipt.hash();
        Ok(sig_engine.verify(public_key, &receipt_hash, &self.signature)?)
    }

    /// Get receipt hash for Merkle tree construction
    pub fn merkle_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.receipt.hash());
        hasher.update(&self.signature);
        hasher.finalize().to_vec()
    }
}

/// Merkle proof for receipt existence in audit trail
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    pub receipt_hash: Vec<u8>,
    pub siblings: Vec<Vec<u8>>,
    pub root: Vec<u8>,
    pub path_indices: Vec<bool>, // true = right, false = left
}

impl MerkleProof {
    /// Verify this Merkle proof
    pub fn verify(&self) -> bool {
        let mut current_hash = self.receipt_hash.clone();

        for (sibling, &is_right) in self.siblings.iter().zip(self.path_indices.iter()) {
            let mut hasher = Sha256::new();
            if is_right {
                hasher.update(&current_hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(&current_hash);
            }
            current_hash = hasher.finalize().to_vec();
        }

        current_hash == self.root
    }
}

/// Quantum-safe audit trail for workflow receipts
pub struct QuantumSafeAuditTrail {
    receipts: Vec<SignedReceipt>,
    private_key: Vec<u8>,
    public_key: Vec<u8>,
    public_key_id: String,
    sig_engine: DilithiumSig,
}

impl QuantumSafeAuditTrail {
    /// Create a new audit trail with a generated key pair
    pub fn new() -> Result<Self> {
        let sig_engine = DilithiumSig::new();
        let (public_key, private_key) = sig_engine.keygen()?;
        let public_key_id = format!("pk_{}", hex::encode(&public_key[..16]));

        Ok(Self {
            receipts: Vec::new(),
            private_key,
            public_key,
            public_key_id,
            sig_engine,
        })
    }

    /// Create audit trail with existing keys
    pub fn with_keys(
        private_key: Vec<u8>,
        public_key: Vec<u8>,
        public_key_id: String,
    ) -> Self {
        Self {
            receipts: Vec::new(),
            private_key,
            public_key,
            public_key_id,
            sig_engine: DilithiumSig::new(),
        }
    }

    /// Record a receipt by signing it
    pub fn record_receipt(&mut self, receipt: Receipt) -> Result<SignedReceipt> {
        let receipt_hash = receipt.hash();
        let signature = self.sig_engine.sign(&self.private_key, &receipt_hash)?;

        let signed_receipt = SignedReceipt {
            receipt,
            signature,
            signer_public_key_id: self.public_key_id.clone(),
            algorithm: "Dilithium3".to_string(),
            signed_at: Utc::now(),
        };

        self.receipts.push(signed_receipt.clone());
        Ok(signed_receipt)
    }

    /// Verify integrity of all receipts in the trail
    pub fn verify_trail_integrity(&self) -> Result<()> {
        for signed_receipt in &self.receipts {
            if !signed_receipt.verify(&self.public_key)? {
                return Err(AuditError::InvalidReceipt(
                    signed_receipt.receipt.id.clone()
                ));
            }
        }
        Ok(())
    }

    /// Export Merkle proof for a specific receipt
    pub fn export_merkle_proof(&self, receipt_id: &str) -> Result<MerkleProof> {
        // Find receipt index
        let receipt_index = self.receipts
            .iter()
            .position(|r| r.receipt.id == receipt_id)
            .ok_or_else(|| AuditError::ReceiptNotFound(receipt_id.to_string()))?;

        // Build Merkle tree
        let mut tree_layer: Vec<Vec<u8>> = self.receipts
            .iter()
            .map(|r| r.merkle_hash())
            .collect();

        let mut siblings = Vec::new();
        let mut path_indices = Vec::new();
        let mut current_index = receipt_index;

        while tree_layer.len() > 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < tree_layer.len() {
                siblings.push(tree_layer[sibling_index].clone());
                path_indices.push(current_index % 2 == 0);
            }

            // Build next layer
            let mut next_layer = Vec::new();
            for i in (0..tree_layer.len()).step_by(2) {
                let mut hasher = Sha256::new();
                hasher.update(&tree_layer[i]);
                if i + 1 < tree_layer.len() {
                    hasher.update(&tree_layer[i + 1]);
                }
                next_layer.push(hasher.finalize().to_vec());
            }

            tree_layer = next_layer;
            current_index /= 2;
        }

        let root = tree_layer[0].clone();
        let receipt_hash = self.receipts[receipt_index].merkle_hash();

        Ok(MerkleProof {
            receipt_hash,
            siblings,
            root,
            path_indices,
        })
    }

    /// Get all receipts
    pub fn receipts(&self) -> &[SignedReceipt] {
        &self.receipts
    }

    /// Get public key
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Get public key ID
    pub fn public_key_id(&self) -> &str {
        &self.public_key_id
    }
}

impl Default for QuantumSafeAuditTrail {
    fn default() -> Self {
        Self::new().expect("Failed to create default audit trail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_creation() {
        let receipt = Receipt::new(
            "receipt_001".to_string(),
            "workflow_001".to_string(),
            "process_task".to_string(),
            b"task data".to_vec(),
        );
        assert_eq!(receipt.id, "receipt_001");
        assert_eq!(receipt.workflow_id, "workflow_001");
    }

    #[test]
    fn test_receipt_hash() {
        let receipt = Receipt::new(
            "receipt_001".to_string(),
            "workflow_001".to_string(),
            "process_task".to_string(),
            b"task data".to_vec(),
        );
        let hash1 = receipt.hash();
        let hash2 = receipt.hash();
        assert_eq!(hash1, hash2, "Hash should be deterministic");
        assert_eq!(hash1.len(), 32, "SHA256 hash should be 32 bytes");
    }

    #[test]
    fn test_audit_trail_creation() {
        let trail = QuantumSafeAuditTrail::new().expect("Failed to create trail");
        assert!(trail.receipts().is_empty());
        assert!(!trail.public_key_id().is_empty());
    }

    #[test]
    fn test_record_and_verify_receipt() {
        let mut trail = QuantumSafeAuditTrail::new().expect("Failed to create trail");
        let receipt = Receipt::new(
            "receipt_001".to_string(),
            "workflow_001".to_string(),
            "process_task".to_string(),
            b"task data".to_vec(),
        );

        let signed = trail.record_receipt(receipt).expect("Failed to record receipt");
        assert_eq!(signed.algorithm, "Dilithium3");
        assert!(signed.verify(trail.public_key()).expect("Failed to verify"));
    }

    #[test]
    fn test_trail_integrity() {
        let mut trail = QuantumSafeAuditTrail::new().expect("Failed to create trail");

        for i in 0..5 {
            let receipt = Receipt::new(
                format!("receipt_{:03}", i),
                "workflow_001".to_string(),
                "process_task".to_string(),
                format!("task data {}", i).into_bytes(),
            );
            trail.record_receipt(receipt).expect("Failed to record receipt");
        }

        trail.verify_trail_integrity().expect("Trail integrity check failed");
    }

    #[test]
    fn test_merkle_proof() {
        let mut trail = QuantumSafeAuditTrail::new().expect("Failed to create trail");

        // Record multiple receipts
        for i in 0..4 {
            let receipt = Receipt::new(
                format!("receipt_{:03}", i),
                "workflow_001".to_string(),
                "process_task".to_string(),
                format!("task data {}", i).into_bytes(),
            );
            trail.record_receipt(receipt).expect("Failed to record receipt");
        }

        // Export and verify Merkle proof
        let proof = trail.export_merkle_proof("receipt_002")
            .expect("Failed to export proof");
        assert!(proof.verify(), "Merkle proof verification failed");
    }

    #[test]
    fn test_merkle_proof_not_found() {
        let trail = QuantumSafeAuditTrail::new().expect("Failed to create trail");
        let result = trail.export_merkle_proof("nonexistent");
        assert!(result.is_err());
    }
}
