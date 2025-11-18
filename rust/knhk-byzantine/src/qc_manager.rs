//! Quorum Certificate Management
//!
//! Manages creation, verification, and storage of quorum certificates (QCs).
//! A QC proves that a quorum (2f+1 nodes) agreed on a block.

use crate::{
    errors::{ByzantineError, Result},
    protocols::Signature,
    Hash, NodeId,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Quorum certificate proving consensus on a block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuorumCertificate {
    pub block_hash: Hash,
    pub view: u64,
    pub signatures: Vec<(NodeId, Signature)>,
}

impl QuorumCertificate {
    /// Verify the QC has sufficient signatures
    pub fn verify_threshold(&self, threshold: usize) -> Result<()> {
        if self.signatures.len() < threshold {
            return Err(ByzantineError::InsufficientQuorum {
                got: self.signatures.len(),
                need: threshold,
            });
        }
        Ok(())
    }

    /// Get signature count
    pub fn signature_count(&self) -> usize {
        self.signatures.len()
    }

    /// Check if a node signed this QC
    pub fn has_signature(&self, node_id: NodeId) -> bool {
        self.signatures.iter().any(|(id, _)| *id == node_id)
    }
}

/// Manages quorum certificates
pub struct QuorumCertificateManager {
    threshold: usize,
    certificates: Arc<DashMap<Hash, QuorumCertificate>>,
    verified_qcs: Arc<DashMap<Hash, bool>>,
}

impl QuorumCertificateManager {
    /// Create new QC manager
    pub fn new(threshold: usize) -> Self {
        info!("QC manager initialized with threshold {}", threshold);

        Self {
            threshold,
            certificates: Arc::new(DashMap::new()),
            verified_qcs: Arc::new(DashMap::new()),
        }
    }

    /// Create a quorum certificate
    pub fn create_qc(
        &self,
        block_hash: Hash,
        view: u64,
        signatures: Vec<(NodeId, Signature)>,
    ) -> QuorumCertificate {
        debug!(
            "Creating QC: block={}, view={}, sigs={}",
            block_hash,
            view,
            signatures.len()
        );

        let qc = QuorumCertificate {
            block_hash,
            view,
            signatures,
        };

        self.certificates.insert(block_hash, qc.clone());
        qc
    }

    /// Verify a quorum certificate
    pub async fn verify_qc(&self, qc: &QuorumCertificate) -> Result<()> {
        // Check if already verified
        if self.verified_qcs.get(&qc.block_hash).is_some() {
            return Ok(());
        }

        // Verify threshold
        qc.verify_threshold(self.threshold)?;

        // Verify no duplicate signatures
        let mut seen = std::collections::HashSet::new();
        for (node_id, _) in &qc.signatures {
            if !seen.insert(node_id) {
                return Err(ByzantineError::DuplicateMessage {
                    node_id: node_id.0,
                });
            }
        }

        // In production, verify each signature cryptographically
        // For now, we trust the signatures

        debug!("QC verified: block={}, sigs={}", qc.block_hash, qc.signatures.len());
        self.verified_qcs.insert(qc.block_hash, true);

        Ok(())
    }

    /// Get a stored QC
    pub fn get_qc(&self, block_hash: &Hash) -> Option<QuorumCertificate> {
        self.certificates.get(block_hash).map(|qc| qc.clone())
    }

    /// Check if QC exists and is verified
    pub fn is_verified(&self, block_hash: &Hash) -> bool {
        self.verified_qcs.get(block_hash).is_some()
    }

    /// Get all stored QCs
    pub fn all_qcs(&self) -> Vec<QuorumCertificate> {
        self.certificates.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Get QC count
    pub fn qc_count(&self) -> usize {
        self.certificates.len()
    }

    /// Get verified QC count
    pub fn verified_count(&self) -> usize {
        self.verified_qcs.len()
    }

    /// Clear all QCs (for testing)
    pub fn clear(&self) {
        self.certificates.clear();
        self.verified_qcs.clear();
    }

    /// Update threshold
    pub fn set_threshold(&mut self, threshold: usize) {
        info!("Updating QC threshold to {}", threshold);
        self.threshold = threshold;
    }

    /// Get current threshold
    pub fn threshold(&self) -> usize {
        self.threshold
    }
}

/// QC aggregator for collecting signatures
pub struct QCAggregator {
    block_hash: Hash,
    view: u64,
    signatures: DashMap<NodeId, Signature>,
    threshold: usize,
}

impl QCAggregator {
    /// Create new aggregator
    pub fn new(block_hash: Hash, view: u64, threshold: usize) -> Self {
        Self {
            block_hash,
            view,
            signatures: DashMap::new(),
            threshold,
        }
    }

    /// Add a signature
    pub fn add_signature(&self, node_id: NodeId, signature: Signature) -> Result<()> {
        if self.signatures.contains_key(&node_id) {
            return Err(ByzantineError::DuplicateMessage {
                node_id: node_id.0,
            });
        }

        self.signatures.insert(node_id, signature);
        debug!("Signature added: node={}, total={}", node_id, self.signatures.len());

        Ok(())
    }

    /// Check if threshold reached
    pub fn has_quorum(&self) -> bool {
        self.signatures.len() >= self.threshold
    }

    /// Build QC if threshold reached
    pub fn try_build_qc(&self) -> Option<QuorumCertificate> {
        if self.has_quorum() {
            let signatures: Vec<_> = self.signatures.iter()
                .map(|entry| (*entry.key(), entry.value().clone()))
                .collect();

            Some(QuorumCertificate {
                block_hash: self.block_hash,
                view: self.view,
                signatures,
            })
        } else {
            None
        }
    }

    /// Get current signature count
    pub fn signature_count(&self) -> usize {
        self.signatures.len()
    }

    /// Reset aggregator
    pub fn reset(&self) {
        self.signatures.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qc_creation() {
        let manager = QuorumCertificateManager::new(3);
        let block_hash = Hash([1u8; 32]);
        let signatures = vec![
            (NodeId(0), Signature(vec![0u8; 64])),
            (NodeId(1), Signature(vec![1u8; 64])),
            (NodeId(2), Signature(vec![2u8; 64])),
        ];

        let qc = manager.create_qc(block_hash, 1, signatures);

        assert_eq!(qc.block_hash, block_hash);
        assert_eq!(qc.view, 1);
        assert_eq!(qc.signature_count(), 3);
    }

    #[tokio::test]
    async fn test_qc_verification() {
        let manager = QuorumCertificateManager::new(3);
        let block_hash = Hash([1u8; 32]);
        let signatures = vec![
            (NodeId(0), Signature(vec![0u8; 64])),
            (NodeId(1), Signature(vec![1u8; 64])),
            (NodeId(2), Signature(vec![2u8; 64])),
        ];

        let qc = manager.create_qc(block_hash, 1, signatures);
        assert!(manager.verify_qc(&qc).await.is_ok());
        assert!(manager.is_verified(&block_hash));
    }

    #[tokio::test]
    async fn test_insufficient_quorum() {
        let manager = QuorumCertificateManager::new(3);
        let block_hash = Hash([1u8; 32]);
        let signatures = vec![
            (NodeId(0), Signature(vec![0u8; 64])),
            (NodeId(1), Signature(vec![1u8; 64])),
        ];

        let qc = QuorumCertificate {
            block_hash,
            view: 1,
            signatures,
        };

        assert!(manager.verify_qc(&qc).await.is_err());
    }

    #[test]
    fn test_qc_aggregator() {
        let block_hash = Hash([1u8; 32]);
        let aggregator = QCAggregator::new(block_hash, 1, 3);

        aggregator.add_signature(NodeId(0), Signature(vec![0u8; 64])).unwrap();
        aggregator.add_signature(NodeId(1), Signature(vec![1u8; 64])).unwrap();
        assert!(!aggregator.has_quorum());

        aggregator.add_signature(NodeId(2), Signature(vec![2u8; 64])).unwrap();
        assert!(aggregator.has_quorum());

        let qc = aggregator.try_build_qc().unwrap();
        assert_eq!(qc.signature_count(), 3);
    }

    #[test]
    fn test_duplicate_signature() {
        let block_hash = Hash([1u8; 32]);
        let aggregator = QCAggregator::new(block_hash, 1, 3);

        aggregator.add_signature(NodeId(0), Signature(vec![0u8; 64])).unwrap();
        let result = aggregator.add_signature(NodeId(0), Signature(vec![0u8; 64]));
        assert!(result.is_err());
    }
}
