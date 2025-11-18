//! Merkle Tree Proofs for Byzantine Detection
//!
//! Compact cryptographic proofs that verify state correctness without revealing full state.
//! Used for Byzantine-robust gossip where agents verify peer states before merging.

use super::{AgentId, Result};
use blake3::Hash as Blake3Hash;
use serde::{Deserialize, Serialize};
use tracing::trace;

/// Merkle proof path (list of sibling hashes from leaf to root)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleProof {
    /// Merkle path from leaf to root (depth = log2(n))
    pub path: Vec<Blake3Hash>,
    /// Leaf index in tree
    pub leaf_index: usize,
}

impl MerkleProof {
    /// Create new Merkle proof
    pub fn new(path: Vec<Blake3Hash>, leaf_index: usize) -> Self {
        Self { path, leaf_index }
    }

    /// Verify proof against root hash
    pub fn verify(&self, leaf_hash: &Blake3Hash, root_hash: &Blake3Hash) -> bool {
        let mut current_hash = *leaf_hash;
        let mut index = self.leaf_index;

        for sibling_hash in &self.path {
            // Compute parent hash: hash(left || right)
            current_hash = if index % 2 == 0 {
                // Current is left child
                hash_pair(&current_hash, sibling_hash)
            } else {
                // Current is right child
                hash_pair(sibling_hash, &current_hash)
            };
            index /= 2;
        }

        current_hash == *root_hash
    }

    /// Proof depth (tree height)
    pub fn depth(&self) -> usize {
        self.path.len()
    }
}

/// State proof (Merkle proof + signature)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateProof {
    /// State hash (leaf in Merkle tree)
    pub state_hash: Blake3Hash,
    /// State version
    pub version: u64,
    /// Merkle path to root
    pub merkle_path: Vec<Blake3Hash>,
    /// Leaf index
    pub leaf_index: usize,
    /// Aggregator signature (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregator_signature: Option<Vec<u8>>,
}

impl StateProof {
    /// Create new state proof
    pub fn new(
        state_hash: Blake3Hash,
        version: u64,
        merkle_path: Vec<Blake3Hash>,
        leaf_index: usize,
    ) -> Self {
        Self {
            state_hash,
            version,
            merkle_path,
            leaf_index,
            aggregator_signature: None,
        }
    }

    /// Verify proof against root hash
    pub fn verify(&self, root_hash: &Blake3Hash) -> bool {
        let merkle_proof = MerkleProof::new(self.merkle_path.clone(), self.leaf_index);
        merkle_proof.verify(&self.state_hash, root_hash)
    }

    /// Proof size in bytes
    pub fn size(&self) -> usize {
        32 + // state_hash
        8 + // version
        self.merkle_path.len() * 32 + // merkle path
        8 + // leaf_index
        self.aggregator_signature.as_ref().map(|s| s.len()).unwrap_or(0)
    }
}

/// Hash a pair of hashes (used in Merkle tree)
fn hash_pair(left: &Blake3Hash, right: &Blake3Hash) -> Blake3Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    hasher.finalize()
}

/// Build Merkle tree from list of hashes
pub fn build_merkle_tree(leaves: &[Blake3Hash]) -> Vec<Vec<Blake3Hash>> {
    if leaves.is_empty() {
        return vec![];
    }

    let mut tree = vec![leaves.to_vec()];
    let mut current_level = leaves.to_vec();

    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        for i in (0..current_level.len()).step_by(2) {
            let left = current_level[i];
            let right = if i + 1 < current_level.len() {
                current_level[i + 1]
            } else {
                current_level[i] // Duplicate if odd number
            };
            next_level.push(hash_pair(&left, &right));
        }
        tree.push(next_level.clone());
        current_level = next_level;
    }

    tree
}

/// Get Merkle root from tree
pub fn merkle_root(tree: &[Vec<Blake3Hash>]) -> Option<Blake3Hash> {
    tree.last().and_then(|level| level.first().copied())
}

/// Generate Merkle proof for leaf at index
pub fn generate_merkle_proof(
    tree: &[Vec<Blake3Hash>],
    leaf_index: usize,
) -> Option<MerkleProof> {
    if tree.is_empty() || leaf_index >= tree[0].len() {
        return None;
    }

    let mut path = Vec::new();
    let mut index = leaf_index;

    for level in tree.iter().take(tree.len() - 1) {
        let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
        let sibling_hash = if sibling_index < level.len() {
            level[sibling_index]
        } else {
            level[index] // Duplicate if no sibling
        };
        path.push(sibling_hash);
        index /= 2;
    }

    Some(MerkleProof::new(path, leaf_index))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_data(data: &[u8]) -> Blake3Hash {
        blake3::hash(data)
    }

    #[test]
    fn test_hash_pair() {
        let hash1 = hash_data(b"hello");
        let hash2 = hash_data(b"world");
        let combined = hash_pair(&hash1, &hash2);

        // Verify deterministic
        let combined2 = hash_pair(&hash1, &hash2);
        assert_eq!(combined, combined2);

        // Verify order matters
        let reversed = hash_pair(&hash2, &hash1);
        assert_ne!(combined, reversed);
    }

    #[test]
    fn test_build_merkle_tree() {
        let leaves = vec![
            hash_data(b"a"),
            hash_data(b"b"),
            hash_data(b"c"),
            hash_data(b"d"),
        ];

        let tree = build_merkle_tree(&leaves);
        assert_eq!(tree.len(), 3); // 4 leaves -> 2 intermediate -> 1 root
        assert_eq!(tree[0].len(), 4); // Leaf level
        assert_eq!(tree[1].len(), 2); // Intermediate level
        assert_eq!(tree[2].len(), 1); // Root level
    }

    #[test]
    fn test_merkle_root() {
        let leaves = vec![hash_data(b"a"), hash_data(b"b"), hash_data(b"c")];
        let tree = build_merkle_tree(&leaves);
        let root = merkle_root(&tree);
        assert!(root.is_some());
    }

    #[test]
    fn test_merkle_proof_generation_and_verification() {
        let leaves = vec![
            hash_data(b"a"),
            hash_data(b"b"),
            hash_data(b"c"),
            hash_data(b"d"),
        ];
        let tree = build_merkle_tree(&leaves);
        let root = merkle_root(&tree).unwrap();

        // Generate and verify proof for leaf 0
        let proof = generate_merkle_proof(&tree, 0).unwrap();
        assert!(proof.verify(&leaves[0], &root));

        // Generate and verify proof for leaf 2
        let proof2 = generate_merkle_proof(&tree, 2).unwrap();
        assert!(proof2.verify(&leaves[2], &root));

        // Verify wrong leaf fails
        assert!(!proof.verify(&leaves[1], &root));
    }

    #[test]
    fn test_state_proof() {
        let state_hash = hash_data(b"state");
        let merkle_path = vec![hash_data(b"sibling1"), hash_data(b"sibling2")];
        let proof = StateProof::new(state_hash, 1, merkle_path, 0);

        assert_eq!(proof.version, 1);
        assert_eq!(proof.leaf_index, 0);
        assert!(proof.size() > 0);
    }
}
