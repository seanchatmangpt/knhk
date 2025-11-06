// rust/knhk-aot/src/mphf.rs
// Minimal Perfect Hash Function generation for predicate lookups

use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Minimal Perfect Hash Function for predicates
/// Optimizes predicate lookups in hot path
pub struct Mphf {
    predicates: Vec<u64>,
    hash_table: BTreeMap<u64, usize>, // Predicate -> index mapping
}

impl Mphf {
    /// Create MPHF from predicate list
    pub fn new(predicates: Vec<u64>) -> Self {
        let mut hash_table = BTreeMap::new();
        
        // Simple hash table construction
        // In production, would use proper MPHF algorithm (e.g., CHD)
        for (idx, &pred) in predicates.iter().enumerate() {
            hash_table.insert(pred, idx);
        }

        Self {
            predicates,
            hash_table,
        }
    }

    /// Lookup predicate index
    pub fn lookup(&self, predicate: u64) -> Option<usize> {
        self.hash_table.get(&predicate).copied()
    }

    /// Get predicate count
    pub fn len(&self) -> usize {
        self.predicates.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.predicates.is_empty()
    }
}

impl Default for Mphf {
    fn default() -> Self {
        Self {
            predicates: Vec::new(),
            hash_table: BTreeMap::new(),
        }
    }
}

/// MPHF cache for common predicates
pub struct MphfCache {
    cache: BTreeMap<Vec<u64>, Mphf>, // Predicate set -> MPHF mapping
}

impl MphfCache {
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
        }
    }

    /// Get or create MPHF for predicate set
    pub fn get_or_create(&mut self, predicates: &[u64]) -> &Mphf {
        let key: Vec<u64> = predicates.to_vec();
        
        if !self.cache.contains_key(&key) {
            let mphf = Mphf::new(key.clone());
            self.cache.insert(key.clone(), mphf);
        }

        self.cache.get(&key).unwrap()
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for MphfCache {
    fn default() -> Self {
        Self::new()
    }
}

