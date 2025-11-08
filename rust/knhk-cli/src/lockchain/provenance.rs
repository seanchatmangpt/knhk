//! Provenance tracker - Tracks operation provenance

use crate::lockchain::merkle::MerkleBuilder;
use crate::receipt_store::store::ReceiptEntry;

/// Provenance tracker - Tracks operation provenance
pub struct ProvenanceTracker {
    merkle_builder: MerkleBuilder,
}

impl ProvenanceTracker {
    /// Create new provenance tracker
    pub fn new() -> Self {
        Self {
            merkle_builder: MerkleBuilder::new(),
        }
    }

    /// Track operation provenance
    pub fn track(&self, receipts: &[ReceiptEntry]) -> Result<u64, String> {
        let hashes: Vec<u64> = receipts.iter().map(|r| r.a_hash).collect();
        self.merkle_builder.build(&hashes)
    }
}

impl Default for ProvenanceTracker {
    fn default() -> Self {
        Self::new()
    }
}
