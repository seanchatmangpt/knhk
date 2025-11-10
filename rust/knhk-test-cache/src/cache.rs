//! Test result caching
//!
//! Caches test results by code hash to enable instant test execution
//! when code hasn't changed.

use crate::TestCacheError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Cached test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheResult {
    /// Code hash when test was run
    pub code_hash: String,
    /// Test execution timestamp
    pub timestamp: u64,
    /// Test execution status
    pub status: TestStatus,
    /// Test output (if available)
    pub output: Option<String>,
    /// Execution duration in seconds
    pub duration_secs: f64,
}

/// Test execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

/// Test cache manager
#[derive(Clone)]
pub struct Cache {
    /// Cache directory
    cache_dir: PathBuf,
    /// Result cache directory
    result_dir: PathBuf,
    /// Maximum cache age in seconds (default: 1 hour)
    max_age_secs: u64,
}

impl Cache {
    /// Create a new cache manager
    pub fn new(cache_dir: PathBuf) -> Self {
        let result_dir = cache_dir.join("results");
        std::fs::create_dir_all(&result_dir).ok();

        Self {
            cache_dir,
            result_dir,
            max_age_secs: 3600, // 1 hour default
        }
    }

    /// Set maximum cache age
    pub fn with_max_age(mut self, max_age_secs: u64) -> Self {
        self.max_age_secs = max_age_secs;
        self
    }

    /// Get cached test result for code hash
    pub fn get(&self, code_hash: &str) -> Result<Option<CacheResult>, TestCacheError> {
        let result_file = self.result_dir.join(format!("{}.json", code_hash));

        if !result_file.exists() {
            return Ok(None);
        }

        let metadata = std::fs::metadata(&result_file)?;
        let modified = metadata.modified()?;
        let age = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if cache is expired
        if now.saturating_sub(age) > self.max_age_secs {
            std::fs::remove_file(&result_file).ok();
            return Ok(None);
        }

        // Load cached result
        let contents = std::fs::read_to_string(&result_file)?;
        let result: CacheResult = serde_json::from_str(&contents)
            .map_err(|e| TestCacheError::CacheError(format!("Invalid cache format: {}", e)))?;

        Ok(Some(result))
    }

    /// Store test result in cache
    pub fn store(
        &self,
        code_hash: &str,
        status: TestStatus,
        output: Option<String>,
        duration_secs: f64,
    ) -> Result<(), TestCacheError> {
        let result = CacheResult {
            code_hash: code_hash.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status,
            output,
            duration_secs,
        };

        let result_file = self.result_dir.join(format!("{}.json", code_hash));
        let contents = serde_json::to_string_pretty(&result)
            .map_err(|e| TestCacheError::CacheError(format!("Serialization error: {}", e)))?;

        std::fs::write(&result_file, contents)?;

        // Clean old cache entries (keep last 10)
        self.clean_old_entries(10)?;

        Ok(())
    }

    /// Clean old cache entries, keeping only the most recent N
    fn clean_old_entries(&self, keep: usize) -> Result<(), TestCacheError> {
        let mut entries: Vec<(PathBuf, u64)> = Vec::new();

        for entry in std::fs::read_dir(&self.result_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        let timestamp = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
                        entries.push((path, timestamp));
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove old entries
        for (path, _) in entries.iter().skip(keep) {
            std::fs::remove_file(path).ok();
        }

        Ok(())
    }

    /// Clear all cached results
    pub fn clear(&self) -> Result<(), TestCacheError> {
        for entry in std::fs::read_dir(&self.result_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                std::fs::remove_file(path).ok();
            }
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats, TestCacheError> {
        let mut total_size = 0;
        let mut count = 0;

        for entry in std::fs::read_dir(&self.result_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    total_size += metadata.len();
                    count += 1;
                }
            }
        }

        Ok(CacheStats {
            entry_count: count,
            total_size_bytes: total_size,
        })
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let cache = Cache::new(temp_dir.path().to_path_buf());

        let code_hash = "test_hash_123";
        cache
            .store(code_hash, TestStatus::Passed, None, 1.5)
            .unwrap();

        let result = cache.get(code_hash).unwrap();
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.code_hash, code_hash);
        assert_eq!(result.status, TestStatus::Passed);
        assert_eq!(result.duration_secs, 1.5);
    }

    #[test]
    fn test_cache_expiration() {
        let temp_dir = TempDir::new().unwrap();
        let cache = Cache::new(temp_dir.path().to_path_buf()).with_max_age(1); // 1 second expiration

        let code_hash = "test_hash_456";
        cache
            .store(code_hash, TestStatus::Passed, None, 1.0)
            .unwrap();

        // Should be available immediately
        assert!(cache.get(code_hash).unwrap().is_some());

        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Should be expired
        assert!(cache.get(code_hash).unwrap().is_none());
    }
}
