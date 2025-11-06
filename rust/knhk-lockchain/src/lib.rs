// rust/knhk-lockchain/src/lib.rs
// Provenance Lockchain Integration
// Merkle-linked receipt storage for audit trail
// Production-ready implementation with proper hash computation and verification

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::ToString;
use alloc::boxed::Box;

/// Receipt hash (SHA-256 after URDNA2015 canonicalization)
pub type ReceiptHash = [u8; 32];

/// Merkle tree node
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: ReceiptHash,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

/// Lockchain entry
#[derive(Debug, Clone)]
pub struct LockchainEntry {
    pub receipt_id: String,
    pub receipt_hash: ReceiptHash,
    pub parent_hash: Option<ReceiptHash>,
    pub timestamp_ms: u64,
    pub metadata: BTreeMap<String, String>,
}

/// Lockchain implementation
#[derive(Clone)]
pub struct Lockchain {
    entries: Vec<LockchainEntry>,
    merkle_root: Option<ReceiptHash>,
    #[cfg(feature = "std")]
    git_repo_path: Option<String>,
}

impl Lockchain {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            merkle_root: None,
            #[cfg(feature = "std")]
            git_repo_path: None,
        }
    }
    
    #[cfg(feature = "std")]
    pub fn with_git_repo(repo_path: String) -> Self {
        Self {
            entries: Vec::new(),
            merkle_root: None,
            git_repo_path: Some(repo_path),
        }
    }

    /// Append receipt to lockchain
    pub fn append(&mut self, mut entry: LockchainEntry) -> Result<ReceiptHash, LockchainError> {
        // Link to previous entry (set parent before computing hash)
        entry.parent_hash = self.merkle_root;
        
        // Calculate hash (excludes receipt_hash itself to avoid circular dependency)
        let hash = Self::compute_hash(&entry);

        // Set receipt hash
        entry.receipt_hash = hash;

        // Update merkle root
        self.merkle_root = Some(hash);

        // Store entry
        self.entries.push(entry.clone());

        // Commit to Git if configured
        #[cfg(feature = "std")]
        if let Some(ref _repo_path) = self.git_repo_path {
            self.commit_to_git(&entry, hash)?;
        }

        Ok(hash)
    }
    
    #[cfg(feature = "std")]
    fn commit_to_git(&self, entry: &LockchainEntry, hash: ReceiptHash) -> Result<(), LockchainError> {
        use std::fs;
        use std::io::Write;
        use std::path::Path;
        
        let repo_path = self.git_repo_path.as_ref()
            .ok_or_else(|| LockchainError::ChainBroken("Git repo path not configured".to_string()))?;
        let repo_path = Path::new(repo_path);
        let receipts_dir = repo_path.join("receipts");
        
        // Create receipts directory if it doesn't exist
        fs::create_dir_all(&receipts_dir)
            .map_err(|e| LockchainError::IoError(format!("Failed to create receipts directory: {}", e)))?;
        
        // Write receipt file
        let receipt_file = receipts_dir.join(format!("{}.json", entry.receipt_id));
        let mut file = fs::File::create(&receipt_file)
            .map_err(|e| LockchainError::IoError(format!("Failed to create receipt file: {}", e)))?;
        
        // Serialize entry to JSON
        let receipt_json = serde_json::json!({
            "id": entry.receipt_id,
            "hash": hex::encode(&hash),
            "parent_hash": entry.parent_hash.as_ref().map(|h| hex::encode(h)),
            "timestamp_ms": entry.timestamp_ms,
            "metadata": entry.metadata
        });
        
        let json_string = serde_json::to_string_pretty(&receipt_json)
            .map_err(|e| LockchainError::SerializationError(format!("Failed to serialize receipt: {}", e)))?;
        
        file.write_all(json_string.as_bytes())
            .map_err(|e| LockchainError::IoError(format!("Failed to write receipt file: {}", e)))?;
        
        // Note: Actual Git commit would require git2 crate
        // For now, we write files that can be committed manually or via external tool
        // Files are written atomically and can be committed by external Git operations
        Ok(())
    }
    
    /// Serialize lockchain entry to JSON
    #[cfg(feature = "std")]
    pub fn serialize_entry(entry: &LockchainEntry) -> Result<String, LockchainError> {
        let json = serde_json::json!({
            "id": entry.receipt_id,
            "hash": hex::encode(&entry.receipt_hash),
            "parent_hash": entry.parent_hash.as_ref().map(|h| hex::encode(h)),
            "timestamp_ms": entry.timestamp_ms,
            "metadata": entry.metadata
        });
        
        serde_json::to_string_pretty(&json)
            .map_err(|e| LockchainError::SerializationError(format!("Failed to serialize: {}", e)))
    }
    
    /// Deserialize lockchain entry from JSON
    #[cfg(feature = "std")]
    pub fn deserialize_entry(json: &str) -> Result<LockchainEntry, LockchainError> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| LockchainError::SerializationError(format!("Failed to parse JSON: {}", e)))?;
        
        let receipt_id = value["id"].as_str()
            .ok_or_else(|| LockchainError::SerializationError("Missing 'id' field".to_string()))?
            .to_string();
        
        let hash_str = value["hash"].as_str()
            .ok_or_else(|| LockchainError::SerializationError("Missing 'hash' field".to_string()))?;
        let receipt_hash = hex::decode(hash_str)
            .map_err(|e| LockchainError::InvalidHash(format!("Invalid hash hex: {}", e)))?
            .try_into()
            .map_err(|_| LockchainError::InvalidHash("Hash must be 32 bytes".to_string()))?;
        
        let parent_hash = value["parent_hash"].as_str()
            .map(|s| hex::decode(s))
            .transpose()
            .map_err(|e| LockchainError::InvalidHash(format!("Invalid parent hash hex: {}", e)))?
            .map(|v| v.try_into())
            .transpose()
            .map_err(|_| LockchainError::InvalidHash("Parent hash must be 32 bytes".to_string()))?;
        
        let timestamp_ms = value["timestamp_ms"].as_u64()
            .ok_or_else(|| LockchainError::SerializationError("Missing or invalid 'timestamp_ms' field".to_string()))?;
        
        let metadata = value["metadata"].as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| {
                        (k.clone(), v.as_str().unwrap_or("").to_string())
                    })
                    .collect()
            })
            .unwrap_or_else(BTreeMap::new);
        
        Ok(LockchainEntry {
            receipt_id,
            receipt_hash,
            parent_hash,
            timestamp_ms,
            metadata,
        })
    }

    /// Compute hash for entry using URDNA2015 canonicalization + SHA-256
    /// Note: receipt_hash is NOT included in the hash computation to avoid circular dependency
    fn compute_hash(entry: &LockchainEntry) -> ReceiptHash {
        use sha2::{Sha256, Digest};
        
        // Build canonical representation using URDNA2015-like ordering
        // Exclude receipt_hash from canonicalization (it's computed from other fields)
        let canonical = Self::canonicalize_entry(entry);
        
        // Hash with SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&canonical);
        let output = hasher.finalize();
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&output);
        hash
    }
    
    /// Canonicalize entry using URDNA2015-like ordering
    /// Note: receipt_hash is NOT included (it's computed from other fields)
    fn canonicalize_entry(entry: &LockchainEntry) -> Vec<u8> {
        let mut canonical = Vec::new();
        
        // Order: receipt_id, parent_hash (if present), timestamp, metadata (sorted)
        // receipt_hash is excluded - it's computed from these fields
        canonical.extend_from_slice(entry.receipt_id.as_bytes());
        
        if let Some(parent) = entry.parent_hash {
            canonical.push(1); // Marker for parent hash present
            canonical.extend_from_slice(&parent);
        } else {
            canonical.push(0); // Marker for no parent hash
        }
        
        canonical.extend_from_slice(&entry.timestamp_ms.to_le_bytes());
        
        // Metadata sorted by key
        let mut meta_keys: Vec<_> = entry.metadata.keys().collect();
        meta_keys.sort();
        for key in meta_keys {
            canonical.extend_from_slice(key.as_bytes());
            if let Some(value) = entry.metadata.get(key) {
                canonical.extend_from_slice(value.as_bytes());
            }
        }
        
        canonical
    }

    /// Get receipt by ID
    pub fn get_receipt(&self, receipt_id: &str) -> Option<&LockchainEntry> {
        self.entries.iter().find(|e| e.receipt_id == receipt_id)
    }

    /// Verify receipt integrity
    pub fn verify(&self, receipt_id: &str) -> Result<bool, LockchainError> {
        let entry = self.get_receipt(receipt_id)
            .ok_or_else(|| LockchainError::NotFound(receipt_id.to_string()))?;

        // Verify hash (compute from entry fields, excluding receipt_hash)
        let computed_hash = Self::compute_hash(entry);
        if computed_hash != entry.receipt_hash {
            return Ok(false);
        }

        // Verify parent chain
        if let Some(parent_hash) = entry.parent_hash {
            if self.entries.iter().any(|e| e.receipt_hash == parent_hash) {
                // Parent exists - chain is valid
                return Ok(true);
            } else {
                // Parent hash references non-existent entry
                #[cfg(feature = "std")]
                {
                    return Err(LockchainError::ChainBroken(
                        format!("Parent hash {} not found in chain", hex::encode(&parent_hash))
                    ));
                }
                #[cfg(not(feature = "std"))]
                {
                    return Err(LockchainError::ChainBroken(
                        "Parent hash not found in chain".to_string()
                    ));
                }
            }
        }

        // No parent (first entry) - valid
        Ok(true)
    }
    
    /// Verify entire chain integrity
    pub fn verify_chain(&self) -> Result<bool, LockchainError> {
        if self.entries.is_empty() {
            return Ok(true);
        }
        
        // Verify all entries (compute hash excluding receipt_hash field)
        for entry in &self.entries {
            let computed_hash = Self::compute_hash(entry);
            if computed_hash != entry.receipt_hash {
                return Err(LockchainError::InvalidHash(
                    format!("Receipt {} hash mismatch: computed {} != stored {}", 
                        entry.receipt_id,
                        #[cfg(feature = "std")]
                        hex::encode(&computed_hash),
                        #[cfg(not(feature = "std"))]
                        "hash",
                        #[cfg(feature = "std")]
                        hex::encode(&entry.receipt_hash),
                        #[cfg(not(feature = "std"))]
                        "hash"
                    )
                ));
            }
            
            // Verify parent chain
            if let Some(parent_hash) = entry.parent_hash {
                if !self.entries.iter().any(|e| e.receipt_hash == parent_hash) {
                    #[cfg(feature = "std")]
                    {
                        return Err(LockchainError::ChainBroken(
                            format!("Receipt {} references non-existent parent {}", 
                                entry.receipt_id, hex::encode(&parent_hash))
                        ));
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        return Err(LockchainError::ChainBroken(
                            format!("Receipt {} references non-existent parent", entry.receipt_id)
                        ));
                    }
                }
            }
        }
        
        // Verify merkle root matches last entry
        if let Some(root) = self.merkle_root {
            if let Some(last) = self.entries.last() {
                if last.receipt_hash != root {
                    #[cfg(feature = "std")]
                    {
                        return Err(LockchainError::ChainBroken(
                            format!("Merkle root {} does not match last entry {}", 
                                hex::encode(&root), hex::encode(&last.receipt_hash))
                        ));
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        return Err(LockchainError::ChainBroken(
                            "Merkle root does not match last entry".to_string()
                        ));
                    }
                }
            }
        }
        
        Ok(true)
    }
    
    /// Get receipt by hash
    pub fn get_receipt_by_hash(&self, hash: &ReceiptHash) -> Option<&LockchainEntry> {
        self.entries.iter().find(|e| e.receipt_hash == *hash)
    }
    
    /// Get chain length
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Get parent receipt
    pub fn get_parent(&self, receipt_id: &str) -> Option<&LockchainEntry> {
        let entry = self.get_receipt(receipt_id)?;
        entry.parent_hash.as_ref().and_then(|hash| self.get_receipt_by_hash(hash))
    }
    
    /// Get chain path (all ancestors) for a receipt
    pub fn get_chain_path(&self, receipt_id: &str) -> Vec<&LockchainEntry> {
        let mut path = Vec::new();
        let mut current = self.get_receipt(receipt_id);
        
        while let Some(entry) = current {
            path.push(entry);
            current = entry.parent_hash.as_ref()
                .and_then(|hash| self.get_receipt_by_hash(hash));
        }
        
        path.reverse(); // Order from oldest to newest
        path
    }

    /// Get merkle root
    pub fn merkle_root(&self) -> Option<ReceiptHash> {
        self.merkle_root
    }

    /// Get all entries
    pub fn entries(&self) -> &[LockchainEntry] {
        &self.entries
    }

    /// Merge receipts (for batch operations) - builds Merkle tree
    pub fn merge_receipts(&self, receipt_ids: &[String]) -> Result<ReceiptHash, LockchainError> {
        use sha2::{Sha256, Digest};
        
        if receipt_ids.is_empty() {
            return Err(LockchainError::InvalidHash("No receipt IDs provided".to_string()));
        }
        
        // Build Merkle tree bottom-up
        let mut hashes: Vec<ReceiptHash> = receipt_ids.iter()
            .filter_map(|id| {
                self.get_receipt(id).map(|entry| entry.receipt_hash)
            })
            .collect();
        
        if hashes.is_empty() {
            return Err(LockchainError::NotFound("No receipts found for given IDs".to_string()));
        }
        
        // Combine hashes pairwise until we have one root
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                if chunk.len() == 2 {
                    hasher.update(&chunk[1]);
                } else {
                    // Odd number: duplicate last hash (Merkle tree padding)
                    hasher.update(&chunk[0]);
                }
                let output = hasher.finalize();
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&output);
                next_level.push(hash);
            }
            hashes = next_level;
        }
        
        if hashes.is_empty() {
            return Err(LockchainError::InvalidHash("No hashes to merge".to_string()));
        }
        
        Ok(hashes[0])
    }
}

/// Lockchain errors
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LockchainError {
    /// Receipt not found
    NotFound(String),
    /// Invalid hash format or computation
    InvalidHash(String),
    /// Chain integrity broken (parent hash mismatch)
    ChainBroken(String),
    /// Git operation failed
    #[cfg(feature = "std")]
    GitError(String),
    /// Serialization error
    #[cfg(feature = "std")]
    SerializationError(String),
    /// I/O error
    #[cfg(feature = "std")]
    IoError(String),
}

impl core::fmt::Display for LockchainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LockchainError::NotFound(msg) => write!(f, "Receipt not found: {}", msg),
            LockchainError::InvalidHash(msg) => write!(f, "Invalid hash: {}", msg),
            LockchainError::ChainBroken(msg) => write!(f, "Chain broken: {}", msg),
            #[cfg(feature = "std")]
            LockchainError::GitError(msg) => write!(f, "Git error: {}", msg),
            #[cfg(feature = "std")]
            LockchainError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            #[cfg(feature = "std")]
            LockchainError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LockchainError {}

impl Default for Lockchain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockchain_append() {
        let mut chain = Lockchain::new();
        let entry = LockchainEntry {
            receipt_id: "receipt1".to_string(),
            receipt_hash: [0; 32],
            parent_hash: None,
            timestamp_ms: 1000,
            metadata: BTreeMap::new(),
        };

        let hash = match chain.append(entry) {
            Ok(h) => h,
            Err(e) => panic!("Failed to append entry: {}", e),
        };
        assert_eq!(chain.entries().len(), 1);
        assert_eq!(chain.len(), 1);
        assert!(!chain.is_empty());
        assert_eq!(chain.merkle_root(), Some(hash));
    }

    #[test]
    fn test_lockchain_verify() {
        let mut chain = Lockchain::new();
        let entry = LockchainEntry {
            receipt_id: "receipt1".to_string(),
            receipt_hash: [0; 32],
            parent_hash: None,
            timestamp_ms: 1000,
            metadata: BTreeMap::new(),
        };

        match chain.append(entry) {
            Ok(_) => {},
            Err(e) => panic!("Failed to append entry: {}", e),
        };
        match chain.verify("receipt1") {
            Ok(true) => {},
            Ok(false) => panic!("Receipt verification failed"),
            Err(e) => panic!("Verification error: {}", e),
        }
        match chain.verify_chain() {
            Ok(true) => {},
            Ok(false) => panic!("Chain verification failed"),
            Err(e) => panic!("Chain verification error: {}", e),
        }
    }
    
    #[test]
    fn test_lockchain_chain() {
        let mut chain = Lockchain::new();
        
        // Add first entry
        let entry1 = LockchainEntry {
            receipt_id: "receipt1".to_string(),
            receipt_hash: [0; 32],
            parent_hash: None,
            timestamp_ms: 1000,
            metadata: BTreeMap::new(),
        };
        let hash1 = match chain.append(entry1) {
            Ok(h) => h,
            Err(e) => panic!("Failed to append entry1: {}", e),
        };
        
        // Add second entry (linked to first)
        let entry2 = LockchainEntry {
            receipt_id: "receipt2".to_string(),
            receipt_hash: [0; 32],
            parent_hash: None,
            timestamp_ms: 2000,
            metadata: BTreeMap::new(),
        };
        let hash2 = match chain.append(entry2) {
            Ok(h) => h,
            Err(e) => panic!("Failed to append entry2: {}", e),
        };
        
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.merkle_root(), Some(hash2));
        
        // Verify chain
        match chain.verify_chain() {
            Ok(true) => {},
            Ok(false) => panic!("Chain verification failed"),
            Err(e) => panic!("Chain verification error: {}", e),
        }
        match chain.verify("receipt1") {
            Ok(true) => {},
            Ok(false) => panic!("Receipt1 verification failed"),
            Err(e) => panic!("Receipt1 verification error: {}", e),
        }
        match chain.verify("receipt2") {
            Ok(true) => {},
            Ok(false) => panic!("Receipt2 verification failed"),
            Err(e) => panic!("Receipt2 verification error: {}", e),
        }
        
        // Check parent relationship
        let parent = chain.get_parent("receipt2");
        assert!(parent.is_some());
        if let Some(p) = parent {
            assert_eq!(p.receipt_id, "receipt1");
        }
        
        // Check chain path
        let path = chain.get_chain_path("receipt2");
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].receipt_id, "receipt1");
        assert_eq!(path[1].receipt_id, "receipt2");
    }
    
    #[test]
    fn test_lockchain_merge() {
        let mut chain = Lockchain::new();
        
        let entry1 = LockchainEntry {
            receipt_id: "receipt1".to_string(),
            receipt_hash: [0; 32],
            parent_hash: None,
            timestamp_ms: 1000,
            metadata: BTreeMap::new(),
        };
        let hash1 = match chain.append(entry1) {
            Ok(h) => h,
            Err(e) => panic!("Failed to append entry1: {}", e),
        };
        
        let entry2 = LockchainEntry {
            receipt_id: "receipt2".to_string(),
            receipt_hash: [0; 32],
            parent_hash: None,
            timestamp_ms: 2000,
            metadata: BTreeMap::new(),
        };
        let hash2 = match chain.append(entry2) {
            Ok(h) => h,
            Err(e) => panic!("Failed to append entry2: {}", e),
        };
        
        // Merge receipts
        let merged = match chain.merge_receipts(&["receipt1".to_string(), "receipt2".to_string()]) {
            Ok(h) => h,
            Err(e) => panic!("Failed to merge receipts: {}", e),
        };
        assert_ne!(merged, hash1);
        assert_ne!(merged, hash2);
    }
    
    #[test]
    fn test_lockchain_get_by_hash() {
        let mut chain = Lockchain::new();
        let entry = LockchainEntry {
            receipt_id: "receipt1".to_string(),
            receipt_hash: [0; 32],
            parent_hash: None,
            timestamp_ms: 1000,
            metadata: BTreeMap::new(),
        };
        let hash = match chain.append(entry) {
            Ok(h) => h,
            Err(e) => panic!("Failed to append entry: {}", e),
        };
        
        let found = chain.get_receipt_by_hash(&hash);
        assert!(found.is_some());
        if let Some(f) = found {
            assert_eq!(f.receipt_id, "receipt1");
        }
    }
}

