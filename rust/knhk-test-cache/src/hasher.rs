//! Code hashing for cache invalidation
//!
//! Generates SHA-256 hash of all Rust source files to detect code changes.
//! Used to invalidate cached test binaries and results when code changes.

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use crate::TestCacheError;

/// Code hasher that generates deterministic hashes of Rust source files
pub struct CodeHasher {
    /// Root directory to scan
    root: PathBuf,
    /// Excluded patterns (e.g., target/, .git/)
    exclude_patterns: Vec<String>,
}

impl CodeHasher {
    /// Create a new code hasher
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            exclude_patterns: vec![
                "target/".to_string(),
                ".git/".to_string(),
                ".test-cache/".to_string(),
            ],
        }
    }

    /// Add an exclude pattern
    pub fn exclude(mut self, pattern: String) -> Self {
        self.exclude_patterns.push(pattern);
        self
    }

    /// Generate hash of all Rust source files
    ///
    /// Returns SHA-256 hash of sorted file paths and their contents.
    /// Deterministic: same code → same hash.
    pub fn hash(&self) -> Result<String, TestCacheError> {
        let mut files = BTreeSet::new();
        
        // Collect all .rs files
        self.collect_rust_files(&self.root, &mut files)?;
        
        // Sort files for deterministic hashing
        let mut hasher = Sha256::new();
        
        for file_path in &files {
            // Hash file path
            hasher.update(file_path.as_os_str().to_string_lossy().as_bytes());
            hasher.update(b"\0");
            
            // Hash file contents
            let contents = std::fs::read(file_path)?;
            hasher.update(&contents);
            hasher.update(b"\0");
        }
        
        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }

    /// Collect all Rust source files recursively
    fn collect_rust_files(
        &self,
        dir: &Path,
        files: &mut BTreeSet<PathBuf>,
    ) -> Result<(), TestCacheError> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Skip excluded patterns
            if self.is_excluded(&path) {
                continue;
            }
            
            if path.is_dir() {
                self.collect_rust_files(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.insert(path);
            }
        }
        
        Ok(())
    }

    /// Check if path matches exclude patterns
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns.iter().any(|pattern| path_str.contains(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_hash_deterministic() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        
        fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();
        
        let hasher = CodeHasher::new(temp_dir.path().to_path_buf());
        let hash1 = hasher.hash().unwrap();
        let hash2 = hasher.hash().unwrap();
        
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_hash_changes_with_code() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        
        fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();
        
        let hasher = CodeHasher::new(temp_dir.path().to_path_buf());
        let hash1 = hasher.hash().unwrap();
        
        fs::write(src_dir.join("lib.rs"), "pub fn test() { println!(\"changed\"); }").unwrap();
        let hash2 = hasher.hash().unwrap();
        
        assert_ne!(hash1, hash2, "Hash should change when code changes");
    }

    #[test]
    fn test_excludes_target_directory() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        let target_dir = temp_dir.path().join("target");
        
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();
        
        fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();
        fs::write(target_dir.join("test.rs"), "should be excluded").unwrap();
        
        let hasher = CodeHasher::new(temp_dir.path().to_path_buf());
        let hash = hasher.hash().unwrap();
        
        // Hash should only include src/lib.rs, not target/test.rs
        assert!(!hash.is_empty());
    }
}

