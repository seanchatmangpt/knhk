//! Merkle tree builder

use crate::lockchain::hash::HashGenerator;

/// Merkle tree builder - Builds Merkle tree from receipts
pub struct MerkleBuilder {
    hash_generator: HashGenerator,
}

impl MerkleBuilder {
    /// Create new Merkle builder
    pub fn new() -> Self {
        Self {
            hash_generator: HashGenerator::new(),
        }
    }

    /// Build Merkle tree from hashes
    pub fn build(&self, hashes: &[u64]) -> Result<u64, String> {
        if hashes.is_empty() {
            return Err("No hashes to build Merkle tree".to_string());
        }

        if hashes.len() == 1 {
            return Ok(hashes[0]);
        }

        // Build Merkle tree by hashing pairs
        let mut current_level = hashes.to_vec();
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    let combined = format!("{}{}", chunk[0], chunk[1]);
                    let hash = self.hash_generator.hash(combined.as_bytes())?;
                    next_level.push(hash);
                } else {
                    next_level.push(chunk[0]);
                }
            }
            current_level = next_level;
        }

        Ok(current_level[0])
    }
}

impl Default for MerkleBuilder {
    fn default() -> Self {
        Self::new()
    }
}
