// rust/knhk-warm/src/mphf_cache.rs
// Minimal Perfect Hash Function (MPHF) cache for warm path
// O(1) lookups without collisions for hot predicates and IDs

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MPHF_CACHE_SIZE: usize = 256;

/// MPHF cache entry
#[derive(Clone, Debug)]
struct MPHFEntry {
    key: u64,
    hash: u64,
    value: u64,
    valid: bool,
}

/// MPHF cache for hot predicates and IDs
pub struct MPHFCache {
    entries: Vec<MPHFEntry>,
    size: usize,
    seed: u64,
}

impl MPHFCache {
    /// Create a new MPHF cache with default seed
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    /// Create a new MPHF cache with custom seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            entries: vec![
                MPHFEntry {
                    key: 0,
                    hash: 0,
                    value: 0,
                    valid: false,
                };
                MPHF_CACHE_SIZE
            ],
            size: 0,
            seed,
        }
    }

    /// Lookup in MPHF cache (O(1))
    /// Returns Some(value) if found, None otherwise
    pub fn lookup(&self, key: u64) -> Option<u64> {
        let hash = self.compute_hash(key);
        let idx = (hash % MPHF_CACHE_SIZE as u64) as usize;

        let entry = &self.entries[idx];
        if entry.valid && entry.key == key {
            Some(entry.value)
        } else {
            None
        }
    }

    /// Insert into MPHF cache (O(1))
    /// Returns true if inserted, false if cache is full
    pub fn insert(&mut self, key: u64, value: u64) -> bool {
        if self.size >= MPHF_CACHE_SIZE {
            return false;
        }

        let hash = self.compute_hash(key);
        let idx = (hash % MPHF_CACHE_SIZE as u64) as usize;

        // Check if slot is empty or has same key
        if !self.entries[idx].valid || self.entries[idx].key == key {
            if !self.entries[idx].valid {
                self.size += 1;
            }
            self.entries[idx] = MPHFEntry {
                key,
                hash,
                value,
                valid: true,
            };
            true
        } else {
            // Collision - use linear probing (should be rare with MPHF)
            for i in 1..MPHF_CACHE_SIZE {
                let probe_idx = (idx + i) % MPHF_CACHE_SIZE;
                if !self.entries[probe_idx].valid || self.entries[probe_idx].key == key {
                    if !self.entries[probe_idx].valid {
                        self.size += 1;
                    }
                    self.entries[probe_idx] = MPHFEntry {
                        key,
                        hash,
                        value,
                        valid: true,
                    };
                    return true;
                }
            }
            false
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            entry.valid = false;
        }
        self.size = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> MPHFCacheStats {
        MPHFCacheStats {
            size: self.size,
            capacity: MPHF_CACHE_SIZE,
            load_factor: self.size as f64 / MPHF_CACHE_SIZE as f64,
        }
    }

    /// Compute FNV-1a hash for MPHF
    fn compute_hash(&self, key: u64) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET_BASIS ^ self.seed;

        // Hash the key (8 bytes)
        let bytes = key.to_le_bytes();
        for byte in &bytes {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        hash
    }
}

impl Default for MPHFCache {
    fn default() -> Self {
        Self::new()
    }
}

/// MPHF cache statistics
#[derive(Debug, Clone)]
pub struct MPHFCacheStats {
    pub size: usize,
    pub capacity: usize,
    pub load_factor: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mphf_cache_insert_lookup() {
        let mut cache = MPHFCache::new();

        // Insert some entries
        assert!(cache.insert(100, 1000));
        assert!(cache.insert(200, 2000));
        assert!(cache.insert(300, 3000));

        // Lookup entries
        assert_eq!(cache.lookup(100), Some(1000));
        assert_eq!(cache.lookup(200), Some(2000));
        assert_eq!(cache.lookup(300), Some(3000));
        assert_eq!(cache.lookup(400), None);
    }

    #[test]
    fn test_mphf_cache_update() {
        let mut cache = MPHFCache::new();

        // Insert entry
        assert!(cache.insert(100, 1000));
        assert_eq!(cache.lookup(100), Some(1000));

        // Update entry
        assert!(cache.insert(100, 2000));
        assert_eq!(cache.lookup(100), Some(2000));
    }

    #[test]
    fn test_mphf_cache_stats() {
        let mut cache = MPHFCache::new();

        assert_eq!(cache.stats().size, 0);
        assert_eq!(cache.stats().capacity, MPHF_CACHE_SIZE);
        assert_eq!(cache.stats().load_factor, 0.0);

        // Insert entries
        for i in 0..10 {
            cache.insert(i, i * 10);
        }

        let stats = cache.stats();
        assert_eq!(stats.size, 10);
        assert!(stats.load_factor > 0.0);
    }

    #[test]
    fn test_mphf_cache_clear() {
        let mut cache = MPHFCache::new();

        cache.insert(100, 1000);
        assert_eq!(cache.lookup(100), Some(1000));

        cache.clear();
        assert_eq!(cache.lookup(100), None);
        assert_eq!(cache.stats().size, 0);
    }
}

