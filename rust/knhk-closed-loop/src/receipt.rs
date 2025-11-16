// Receipt System: Cryptographic Proof of Decisions
// Every decision in the closed loop is signed, timestamped, and immutable

use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey, Signer, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// A cryptographically signed receipt proving a decision was made
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    /// Unique ID (SHA-256 of content + timestamp)
    pub id: String,

    /// What operation produced this receipt
    pub operation: ReceiptOperation,

    /// When this decision was made
    pub timestamp: DateTime<Utc>,

    /// The decision outcome
    pub outcome: ReceiptOutcome,

    /// Evidence supporting the decision
    pub evidence: Vec<String>,

    /// Digital signature (ed25519)
    pub signature: String,

    /// Hash of parent receipt (forms immutable chain)
    pub parent_hash: Option<String>,

    /// Sector this receipt belongs to
    pub sector: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ReceiptOperation {
    /// Pattern detected in observation plane
    PatternDetected { pattern: String, confidence: f64 },

    /// ΔΣ (ontology change) proposed
    ProposalGenerated { delta_description: String },

    /// Multi-stage validation performed
    ValidationExecuted {
        stages_passed: Vec<String>,
        stages_failed: Vec<String>,
    },

    /// Hard invariant checked
    InvariantChecked { invariant: String, preserved: bool },

    /// Snapshot promoted to active
    SnapshotPromoted {
        snapshot_id: String,
        parent_id: Option<String>,
    },

    /// Loop cycle completed
    LoopCycleCompleted { duration_ms: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ReceiptOutcome {
    /// Decision was approved (e.g., change promoted)
    Approved,

    /// Decision was rejected (e.g., invalid change)
    Rejected { reason: String },

    /// Decision is pending (e.g., awaiting next validator stage)
    Pending { next_stage: String },

    /// Decision produced an error
    Error { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ReceiptError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Receipt verification failed: {0}")]
    VerificationFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Chain of custody broken")]
    ChainBroken,

    #[error("Receipt not found")]
    NotFound,
}

impl Receipt {
    /// Create and sign a new receipt
    pub fn create(
        operation: ReceiptOperation,
        outcome: ReceiptOutcome,
        evidence: Vec<String>,
        sector: String,
        signing_key: &SigningKey,
        parent_hash: Option<String>,
    ) -> Result<Self, ReceiptError> {
        let timestamp = Utc::now();

        // Create receipt content (deterministic)
        let content = format!(
            "{}|{:?}|{:?}|{}",
            timestamp.timestamp_millis(),
            operation,
            outcome,
            evidence.join("|")
        );

        // Hash content
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let id = hex::encode(hasher.finalize());

        // Sign with ed25519
        let signature = signing_key.sign(id.as_bytes());

        Ok(Receipt {
            id,
            operation,
            timestamp,
            outcome,
            evidence,
            signature: hex::encode(signature.to_bytes()),
            parent_hash,
            sector,
        })
    }

    /// Verify receipt signature
    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<(), ReceiptError> {
        let signature_bytes = hex::decode(&self.signature)
            .map_err(|_| ReceiptError::InvalidSignature)?;

        let signature = Signature::from_bytes(
            signature_bytes
                .as_slice()
                .try_into()
                .map_err(|_| ReceiptError::InvalidSignature)?,
        );

        verifying_key
            .verify(self.id.as_bytes(), &signature)
            .map_err(|_| ReceiptError::InvalidSignature)
    }

    /// Verify chain of custody (parent → child → ... → current)
    pub fn verify_chain(&self, store: &ReceiptStore) -> Result<(), ReceiptError> {
        if let Some(parent_hash) = &self.parent_hash {
            let parent = store
                .get(parent_hash)
                .map_err(|_| ReceiptError::ChainBroken)?;

            // Parent must have earlier timestamp
            if parent.timestamp >= self.timestamp {
                return Err(ReceiptError::ChainBroken);
            }

            // Recursively verify parent chain
            parent.verify_chain(store)?;
        }

        Ok(())
    }
}

/// Immutable append-only log of all receipts
pub struct ReceiptStore {
    receipts: dashmap::DashMap<String, Arc<Receipt>>,
    verifying_key: VerifyingKey,
}

impl ReceiptStore {
    /// Create new receipt store with verification key
    pub fn new(verifying_key: VerifyingKey) -> Self {
        ReceiptStore {
            receipts: dashmap::DashMap::new(),
            verifying_key,
        }
    }

    /// Append a receipt (immutable once added)
    pub async fn append(&self, receipt: Receipt) -> Result<String, ReceiptError> {
        // Verify signature before appending
        receipt.verify(&self.verifying_key)?;

        // Verify chain of custody
        receipt.verify_chain(self)?;

        let id = receipt.id.clone();
        self.receipts.insert(id.clone(), Arc::new(receipt));
        Ok(id)
    }

    /// Get receipt by ID
    pub fn get(&self, id: &str) -> Result<Arc<Receipt>, ReceiptError> {
        self.receipts
            .get(id)
            .map(|entry| entry.clone())
            .ok_or(ReceiptError::NotFound)
    }

    /// Get all receipts for a sector
    pub fn get_sector_receipts(&self, sector: &str) -> Vec<Arc<Receipt>> {
        self.receipts
            .iter()
            .filter(|entry| entry.value().sector == sector)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get receipt chain (parent → child → ... → current)
    pub fn get_chain(&self, receipt_id: &str) -> Result<Vec<Arc<Receipt>>, ReceiptError> {
        let receipt = self.get(receipt_id)?;
        let mut chain = vec![receipt.clone()];

        let mut current = receipt;
        while let Some(parent_hash) = &current.parent_hash {
            current = self.get(parent_hash)?;
            chain.push(current.clone());
        }

        chain.reverse();
        Ok(chain)
    }

    /// List all receipts (for monitoring)
    pub fn list_all(&self) -> Vec<Arc<Receipt>> {
        self.receipts.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Count receipts by sector
    pub fn count_by_sector(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for entry in self.receipts.iter() {
            *counts.entry(entry.value().sector.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_signing_key() -> SigningKey {
        let mut seed = [0u8; 32];
        seed[0] = 42; // Deterministic for testing
        SigningKey::from_bytes(&seed)
    }

    #[test]
    fn test_receipt_creation_and_verification() {
        let signing_key = create_signing_key();
        let verifying_key = signing_key.verifying_key();

        let receipt = Receipt::create(
            ReceiptOperation::PatternDetected {
                pattern: "test_pattern".to_string(),
                confidence: 0.95,
            },
            ReceiptOutcome::Approved,
            vec!["evidence1".to_string()],
            "test_sector".to_string(),
            &signing_key,
            None,
        )
        .expect("failed to create receipt");

        // Verify signature
        assert!(receipt.verify(&verifying_key).is_ok());
    }

    #[test]
    fn test_receipt_store_append_and_retrieve() {
        let signing_key = create_signing_key();
        let verifying_key = signing_key.verifying_key();
        let store = ReceiptStore::new(verifying_key);

        let receipt = Receipt::create(
            ReceiptOperation::PatternDetected {
                pattern: "test".to_string(),
                confidence: 0.9,
            },
            ReceiptOutcome::Approved,
            vec![],
            "sector1".to_string(),
            &signing_key,
            None,
        )
        .expect("failed to create receipt");

        let id = receipt.id.clone();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            store.append(receipt).await.expect("failed to append");
            let retrieved = store.get(&id).expect("failed to retrieve");
            assert_eq!(retrieved.id, id);
        });
    }

    #[test]
    fn test_receipt_chain_of_custody() {
        let signing_key = create_signing_key();
        let verifying_key = signing_key.verifying_key();
        let store = ReceiptStore::new(verifying_key);

        let receipt1 = Receipt::create(
            ReceiptOperation::PatternDetected {
                pattern: "p1".to_string(),
                confidence: 0.8,
            },
            ReceiptOutcome::Approved,
            vec![],
            "s1".to_string(),
            &signing_key,
            None,
        )
        .expect("r1");

        let id1 = receipt1.id.clone();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            store.append(receipt1).await.expect("append r1");

            let receipt2 = Receipt::create(
                ReceiptOperation::ProposalGenerated {
                    delta_description: "test_delta".to_string(),
                },
                ReceiptOutcome::Pending {
                    next_stage: "validation".to_string(),
                },
                vec![],
                "s1".to_string(),
                &signing_key,
                Some(id1.clone()),
            )
            .expect("r2");

            store.append(receipt2.clone()).await.expect("append r2");

            // Verify chain
            let chain = store.get_chain(&receipt2.id).expect("chain");
            assert_eq!(chain.len(), 2);
            assert_eq!(chain[0].id, id1);
        });
    }
}
