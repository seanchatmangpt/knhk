// knhk-unrdf: Rust-native caching layer
// LRU cache for query results and canonicalized data

#[cfg(feature = "native")]
use crate::canonicalize::get_canonical_hash;
#[cfg(feature = "native")]
use crate::error::{UnrdfError, UnrdfResult};
#[cfg(feature = "native")]
use crate::types::QueryResult;
#[cfg(feature = "native")]
use lru::LruCache;
#[cfg(feature = "native")]
use std::num::NonZeroUsize;
#[cfg(feature = "native")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "native")]
/// Cache key for query results
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    query_hash: String,
    data_hash: String,
}

#[cfg(feature = "native")]
impl CacheKey {
    pub fn new(query: &str, data: &str) -> UnrdfResult<Self> {
        let query_hash = get_canonical_hash(query)?;
        let data_hash = get_canonical_hash(data)?;
        Ok(Self {
            query_hash,
            data_hash,
        })
    }
}

#[cfg(feature = "native")]
/// Query result cache with LRU eviction
pub struct QueryCache {
    cache: Arc<Mutex<LruCache<CacheKey, QueryResult>>>,
    hits: Arc<Mutex<usize>>,
    misses: Arc<Mutex<usize>>,
}

#[cfg(feature = "native")]
impl QueryCache {
    /// Create a new cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        // SAFETY: capacity.max(1) mathematically guarantees the value is at least 1,
        // making NonZeroUsize::new infallible. Using unwrap_or as defensive programming.
        let cap = NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN);
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(cap))),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    /// Get cached query result
    pub fn get(&self, query: &str, data: &str) -> UnrdfResult<Option<QueryResult>> {
        let key = CacheKey::new(query, data)?;
        let mut cache = self.cache.lock().map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to acquire cache lock: {}", e))
        })?;

        match cache.get(&key) {
            Some(result) => {
                *self.hits.lock().map_err(|e| {
                    UnrdfError::LockPoisoned(format!("Failed to acquire hits lock: {}", e))
                })? += 1;
                Ok(Some(result.clone()))
            }
            None => {
                *self.misses.lock().map_err(|e| {
                    UnrdfError::LockPoisoned(format!("Failed to acquire misses lock: {}", e))
                })? += 1;
                Ok(None)
            }
        }
    }

    /// Store query result in cache
    pub fn put(&self, query: &str, data: &str, result: QueryResult) -> UnrdfResult<()> {
        let key = CacheKey::new(query, data)?;
        let mut cache = self.cache.lock().map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to acquire cache lock: {}", e))
        })?;

        cache.put(key, result);
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> UnrdfResult<CacheStats> {
        let hits = *self
            .hits
            .lock()
            .map_err(|e| UnrdfError::LockPoisoned(format!("Failed to acquire hits lock: {}", e)))?;
        let misses = *self.misses.lock().map_err(|e| {
            UnrdfError::LockPoisoned(format!("Failed to acquire misses lock: {}", e))
        })?;
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        let cache = self.cache.lock().map_err(|e| {
            UnrdfError::LockPoisoned(format!("Failed to acquire cache lock: {}", e))
        })?;
        Ok(CacheStats {
            hits,
            misses,
            hit_rate,
            size: cache.len(),
            capacity: cache.cap().get(),
        })
    }

    /// Clear the cache
    pub fn clear(&self) -> UnrdfResult<()> {
        let mut cache = self.cache.lock().map_err(|e| {
            UnrdfError::LockPoisoned(format!("Failed to acquire cache lock: {}", e))
        })?;

        cache.clear();
        *self.hits.lock().map_err(|e| {
            UnrdfError::LockPoisoned(format!("Failed to acquire hits lock: {}", e))
        })? = 0;
        *self.misses.lock().map_err(|e| {
            UnrdfError::LockPoisoned(format!("Failed to acquire misses lock: {}", e))
        })? = 0;
        Ok(())
    }
}

#[cfg(feature = "native")]
/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
    pub size: usize,
    pub capacity: usize,
}
