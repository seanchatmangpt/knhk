//! Hash generator - SHA-256 hash generation

use sha2::{Digest, Sha256};

/// Hash generator - Generates SHA-256 hashes
pub struct HashGenerator;

impl HashGenerator {
    /// Create new hash generator
    pub fn new() -> Self {
        Self
    }

    /// Generate hash from bytes
    pub fn hash(&self, data: &[u8]) -> Result<u64, String> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();

        // Convert first 8 bytes to u64
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[..8]);
        Ok(u64::from_be_bytes(bytes))
    }
}

impl Default for HashGenerator {
    fn default() -> Self {
        Self::new()
    }
}
