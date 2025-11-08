//! Receipt linker - Links receipts in Merkle tree

use crate::lockchain::merkle::MerkleBuilder;
use crate::receipt_store::store::ReceiptEntry;

/// Receipt linker - Links receipts in Merkle tree
pub struct ReceiptLinker {
    merkle_builder: MerkleBuilder,
}

impl ReceiptLinker {
    /// Create new receipt linker
    pub fn new() -> Self {
        Self {
            merkle_builder: MerkleBuilder::new(),
        }
    }

    /// Link receipts in Merkle tree
    pub fn link(&self, receipts: &[ReceiptEntry]) -> Result<u64, String> {
        let hashes: Vec<u64> = receipts.iter().map(|r| r.a_hash).collect();
        self.merkle_builder.build(&hashes)
    }
}

impl Default for ReceiptLinker {
    fn default() -> Self {
        Self::new()
    }
}
