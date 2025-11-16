//! Projection cache for incremental compilation
//!
//! Avoids recompiling unchanged snapshots by caching compiled artifacts.

use crate::compiler::CompiledProjections;
use knhk_ontology::SigmaSnapshotId;
use std::collections::HashMap;
use tracing::debug;

/// Thread-safe cache for compiled projections
pub struct ProjectionCache {
    /// Map: snapshot_id -> compiled projections
    entries: HashMap<SigmaSnapshotId, CompiledProjections>,

    /// Total cache hits
    hits: usize,

    /// Total cache misses
    misses: usize,
}

impl ProjectionCache {
    /// Create new empty cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Insert compiled projections into cache
    pub fn insert(&mut self, snapshot_id: SigmaSnapshotId, compiled: CompiledProjections) {
        debug!("Caching compilation for snapshot {:?}", snapshot_id);
        self.entries.insert(snapshot_id, compiled);
    }

    /// Get compiled projections from cache
    pub fn get(&mut self, snapshot_id: &SigmaSnapshotId) -> Option<CompiledProjections> {
        match self.entries.get(snapshot_id) {
            Some(compiled) => {
                self.hits += 1;
                debug!("Cache hit for snapshot {:?}", snapshot_id);
                Some(compiled.clone())
            }
            None => {
                self.misses += 1;
                debug!("Cache miss for snapshot {:?}", snapshot_id);
                None
            }
        }
    }

    /// Remove entry from cache
    pub fn remove(&mut self, snapshot_id: &SigmaSnapshotId) -> Option<CompiledProjections> {
        self.entries.remove(snapshot_id)
    }

    /// Clear all cached entries
    pub fn clear(&mut self) {
        debug!("Clearing projection cache ({} entries)", self.entries.len());
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Get cache statistics (hits, misses)
    pub fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }

    /// Get number of cached entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl Default for ProjectionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::*;
    use std::time::SystemTime;

    fn create_test_compiled() -> CompiledProjections {
        CompiledProjections {
            snapshot_id: [1u8; 32],
            snapshot_hash: [2u8; 32],
            rust_models: RustModelsOutput {
                models_code: "test".to_string(),
                hash: [3u8; 32],
            },
            openapi_spec: OpenApiOutput {
                openapi_spec: "test".to_string(),
                paths: vec![],
                schemas: vec![],
                hash: [4u8; 32],
            },
            hooks_config: HooksOutput {
                hooks_config: "test".to_string(),
                operators: vec![],
                guards: vec![],
                hash: [5u8; 32],
            },
            markdown_docs: MarkdownOutput {
                markdown: "test".to_string(),
                sections: vec![],
                hash: [6u8; 32],
            },
            otel_schema: OtelOutput {
                otel_schema: "test".to_string(),
                spans: vec![],
                metrics: vec![],
                hash: [7u8; 32],
            },
            compiled_at: SystemTime::now(),
        }
    }

    #[test]
    fn test_cache_insert_get() {
        let mut cache = ProjectionCache::new();
        let snapshot_id = [1u8; 32];
        let compiled = create_test_compiled();

        cache.insert(snapshot_id, compiled.clone());

        let retrieved = cache.get(&snapshot_id).unwrap();
        assert_eq!(retrieved.snapshot_id, compiled.snapshot_id);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = ProjectionCache::new();
        let snapshot_id = [1u8; 32];

        assert!(cache.get(&snapshot_id).is_none());
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = ProjectionCache::new();
        let snapshot_id = [1u8; 32];
        let compiled = create_test_compiled();

        cache.insert(snapshot_id, compiled);
        assert!(cache.get(&snapshot_id).is_some());

        cache.remove(&snapshot_id);
        assert!(cache.get(&snapshot_id).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ProjectionCache::new();

        cache.insert([1u8; 32], create_test_compiled());
        cache.insert([2u8; 32], create_test_compiled());

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = ProjectionCache::new();
        let snapshot_id = [1u8; 32];
        let compiled = create_test_compiled();

        cache.insert(snapshot_id, compiled);

        // 2 hits
        cache.get(&snapshot_id);
        cache.get(&snapshot_id);

        // 1 miss
        cache.get(&[2u8; 32]);

        let (hits, misses) = cache.stats();
        assert_eq!(hits, 2);
        assert_eq!(misses, 1);

        assert_eq!(cache.hit_rate(), 2.0 / 3.0);
    }
}
