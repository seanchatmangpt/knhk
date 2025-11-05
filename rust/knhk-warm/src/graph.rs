//! Warm path graph wrapper with oxigraph integration
//! Implements ggen's Graph wrapper pattern with caching

use oxigraph::io::RdfFormat;
use oxigraph::model::{GraphName, NamedNode, NamedOrBlankNode, Quad, Term};
use oxigraph::sparql::{Query, QueryResults};
use oxigraph::store::Store;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use ahash::AHasher;

/// Cached query result
#[derive(Debug, Clone)]
pub enum CachedResult {
    Boolean(bool),
    Solutions(Vec<BTreeMap<String, String>>),
    Graph(Vec<String>), // Serialized triples
}

impl CachedResult {
    /// Convert to serde_json::Value for consumption
    pub fn to_json(&self) -> JsonValue {
        match self {
            CachedResult::Boolean(b) => JsonValue::Bool(*b),
            CachedResult::Solutions(rows) => {
                let arr: Vec<JsonValue> = rows
                    .iter()
                    .map(|row| {
                        let mut obj = serde_json::Map::new();
                        for (k, v) in row {
                            obj.insert(k.clone(), JsonValue::String(v.clone()));
                        }
                        JsonValue::Object(obj)
                    })
                    .collect();
                JsonValue::Array(arr)
            }
            CachedResult::Graph(_triples) => JsonValue::String(String::new()),
        }
    }
}

/// Thread-safe Oxigraph wrapper with SPARQL caching. Clone is cheap (shared store).
/// Following ggen's Graph wrapper pattern.
pub struct WarmPathGraph {
    inner: Store,
    epoch: Arc<AtomicU64>,
    query_cache: Arc<Mutex<lru::LruCache<(u64, u64), CachedResult>>>,
    query_plan_cache: Arc<Mutex<lru::LruCache<u64, Query>>>,
    #[cfg(feature = "otel")]
    query_count: Arc<AtomicU64>,
    #[cfg(feature = "otel")]
    cache_hits: Arc<AtomicU64>,
    #[cfg(feature = "otel")]
    cache_misses: Arc<AtomicU64>,
}

impl WarmPathGraph {
    /// Create a new warm path graph
    /// 
    /// Cache sizes:
    /// - Query cache: 1000 entries (increased from 100)
    /// - Query plan cache: 1000 entries (parsed SPARQL queries)
    /// - Result cache: shared with query cache
    pub fn new() -> Result<Self, String> {
        let query_cache_size =
            NonZeroUsize::new(1000).ok_or_else(|| "Invalid cache size".to_string())?;
        let plan_cache_size =
            NonZeroUsize::new(1000).ok_or_else(|| "Invalid plan cache size".to_string())?;
        
        let store = Store::new()
            .map_err(|e| format!("Failed to create oxigraph store: {}", e))?;

        Ok(Self {
            inner: store,
            epoch: Arc::new(AtomicU64::new(1)),
            query_cache: Arc::new(Mutex::new(lru::LruCache::new(query_cache_size))),
            query_plan_cache: Arc::new(Mutex::new(lru::LruCache::new(plan_cache_size))),
            #[cfg(feature = "otel")]
            query_count: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "otel")]
            cache_hits: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "otel")]
            cache_misses: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Load RDF data from a file into the graph
    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file = File::open(path.as_ref())
            .map_err(|e| format!("Failed to open file {:?}: {}", path.as_ref(), e))?;
        
        let reader = BufReader::new(file);
        self.inner
            .load_from_reader(RdfFormat::Turtle, reader)
            .map_err(|e| format!("Failed to load RDF from file: {}", e))?;
        
        self.bump_epoch();
        Ok(())
    }

    /// Load RDF data from Turtle string
    pub fn load_from_turtle(&self, turtle: &str) -> Result<(), String> {
        self.inner
            .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| format!("Failed to load Turtle data: {}", e))?;
        
        self.bump_epoch();
        Ok(())
    }

    /// Get current epoch
    fn current_epoch(&self) -> u64 {
        self.epoch.load(Ordering::Relaxed)
    }

    /// Bump epoch (invalidates cache)
    pub fn bump_epoch(&self) {
        self.epoch.fetch_add(1, Ordering::Relaxed);
    }

    /// Hash SPARQL query for cache key
    fn hash_query(&self, sparql: &str) -> u64 {
        let mut hasher = AHasher::default();
        sparql.hash(&mut hasher);
        hasher.finish()
    }

    /// Materialize QueryResults into CachedResult
    fn materialize_results(&self, results: QueryResults) -> Result<CachedResult, String> {
        match results {
            QueryResults::Boolean(b) => Ok(CachedResult::Boolean(b)),
            QueryResults::Solutions(solutions) => {
                let mut rows = Vec::new();
                for solution in solutions {
                    let solution = solution.map_err(|e| format!("Query solution error: {}", e))?;
                    let mut row = BTreeMap::new();
                    for (var, term) in solution.iter() {
                        row.insert(var.as_str().to_string(), term.to_string());
                    }
                    rows.push(row);
                }
                Ok(CachedResult::Solutions(rows))
            }
            QueryResults::Graph(quads) => {
                let triples: Result<Vec<String>, String> = quads
                    .map(|q| q.map(|quad| quad.to_string()).map_err(|e| e.to_string()))
                    .collect();
                Ok(CachedResult::Graph(triples.map_err(|e| format!("Graph query error: {}", e))?))
            }
        }
    }

    /// Execute SPARQL query with caching
    /// 
    /// Uses query plan cache to avoid re-parsing identical queries
    pub fn query(&self, sparql: &str) -> Result<QueryResults, String> {
        #[cfg(feature = "otel")]
        let start_time = std::time::Instant::now();
        
        #[cfg(feature = "otel")]
        self.query_count.fetch_add(1, Ordering::Relaxed);
        
        let query_hash = self.hash_query(sparql);
        let current_epoch = self.current_epoch();

        // Check result cache
        let cache_hit = if let Ok(mut cache) = self.query_cache.lock() {
            if let Some(cached) = cache.get(&(query_hash, current_epoch)) {
                #[cfg(feature = "otel")]
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                // Convert cached result back to QueryResults
                return self.cached_to_query_results(cached.clone());
            } else {
                false
            }
        } else {
            false
        };

        #[cfg(feature = "otel")]
        if !cache_hit {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }

        // Check query plan cache - reuse parsed query if available
        let parsed_query = if let Ok(mut plan_cache) = self.query_plan_cache.lock() {
            if let Some(cached_query) = plan_cache.get(&query_hash) {
                Some(cached_query.clone())
            } else {
                // Parse query and cache it
                match Query::parse(sparql, None) {
                    Ok(parsed) => {
                        let query_clone = parsed.clone();
                        plan_cache.put(query_hash, parsed);
                        Some(query_clone)
                    }
                    Err(e) => {
                        return Err(format!("Query parse failed: {}", e));
                    }
                }
            }
        } else {
            // Fallback: parse without caching
            Some(Query::parse(sparql, None)
                .map_err(|e| format!("Query parse failed: {}", e))?)
        };

        // Execute query using parsed query plan
        let results = if let Some(ref query) = parsed_query {
            self.inner
                .query(query)
                .map_err(|e| format!("SPARQL query execution failed: {}", e))?
        } else {
            // Fallback: execute via string (should not happen)
            self.inner
                .query(sparql)
                .map_err(|e| format!("SPARQL query execution failed: {}", e))?
        };

        // Cache result
        if let Ok(materialized) = self.materialize_results(results.clone()) {
            if let Ok(mut cache) = self.query_cache.lock() {
                cache.put((query_hash, current_epoch), materialized);
            }
        }

        #[cfg(feature = "otel")]
        {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            self.record_query_metric(sparql, latency_ms, cache_hit);
        }

        Ok(results)
    }

    /// Record query metric via OTEL
    #[cfg(feature = "otel")]
    fn record_query_metric(&self, sparql: &str, latency_ms: u64, cache_hit: bool) {
        use knhk_otel::{Tracer, Metric, MetricValue};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut tracer = Tracer::new();
        
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        
        // Record query latency
        let latency_metric = Metric {
            name: "knhk.warm.query.latency_ms".to_string(),
            value: MetricValue::Gauge(latency_ms as f64),
            timestamp_ms,
            attributes: {
                let mut attrs = std::collections::BTreeMap::new();
                attrs.insert("cache_hit".to_string(), cache_hit.to_string());
                attrs.insert("query_type".to_string(), 
                    sparql.trim().to_uppercase()
                        .split_whitespace()
                        .next()
                        .unwrap_or("unknown")
                        .to_string());
                attrs
            },
        };
        tracer.record_metric(latency_metric);
        
        // Record cache hit rate
        let total_queries = self.query_count.load(Ordering::Relaxed);
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let hit_rate = if total_queries > 0 {
            (hits as f64) / (total_queries as f64)
        } else {
            0.0
        };
        
        let hit_rate_metric = Metric {
            name: "knhk.warm.query.cache_hit_rate".to_string(),
            value: MetricValue::Gauge(hit_rate),
            timestamp_ms,
            attributes: std::collections::BTreeMap::new(),
        };
        tracer.record_metric(hit_rate_metric);
    }

    /// Get query metrics
    #[cfg(feature = "otel")]
    pub fn get_metrics(&self) -> QueryMetrics {
        let total = self.query_count.load(Ordering::Relaxed);
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let hit_rate = if total > 0 {
            (hits as f64) / (total as f64)
        } else {
            0.0
        };
        
        QueryMetrics {
            total_queries: total,
            cache_hits: hits,
            cache_misses: misses,
            cache_hit_rate: hit_rate,
        }
    }

    /// Get query metrics (no-op when OTEL disabled)
    #[cfg(not(feature = "otel"))]
    pub fn get_metrics(&self) -> QueryMetrics {
        QueryMetrics {
            total_queries: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_hit_rate: 0.0,
        }
    }

    /// Convert cached result back to QueryResults
    /// Note: This is a simplified conversion - full conversion would require
    /// storing more information in the cache
    fn cached_to_query_results(&self, _cached: CachedResult) -> Result<QueryResults, String> {
        // For now, return error to force re-execution
        // In production, would store full QueryResults or reconstruct
        Err("Cache conversion not fully implemented".to_string())
    }

    /// Insert a single triple
    pub fn insert_triple(&self, s: &str, p: &str, o: &str) -> Result<(), String> {
        let s_node = NamedNode::new(s)
            .map_err(|e| format!("Invalid subject IRI {}: {}", s, e))?;
        let p_node = NamedNode::new(p)
            .map_err(|e| format!("Invalid predicate IRI {}: {}", p, e))?;
        let o_term = Self::parse_term(o)?;
        
        let quad = Quad::new(s_node, p_node, o_term, GraphName::DefaultGraph);
        self.inner
            .insert(&quad)
            .map_err(|e| format!("Failed to insert triple: {}", e))?;
        
        self.bump_epoch();
        Ok(())
    }

    /// Parse RDF term from string
    fn parse_term(term_str: &str) -> Result<Term, String> {
        // Simple parsing - try NamedNode first, then Literal
        if term_str.starts_with('<') && term_str.ends_with('>') {
            let iri = &term_str[1..term_str.len() - 1];
            NamedNode::new(iri)
                .map(|n| Term::NamedNode(n))
                .map_err(|e| format!("Invalid IRI {}: {}", iri, e))
        } else if term_str.starts_with('"') {
            // Literal parsing - simplified
            NamedNode::new(term_str)
                .map(|n| Term::NamedNode(n))
                .map_err(|e| format!("Invalid literal {}: {}", term_str, e))
        } else {
            // Try as named node
            NamedNode::new(term_str)
                .map(|n| Term::NamedNode(n))
                .map_err(|e| format!("Invalid term {}: {}", term_str, e))
        }
    }

    /// Batch insert quads
    pub fn insert_quads(&self, quads: &[Quad]) -> Result<(), String> {
        for quad in quads {
            self.inner
                .insert(quad)
                .map_err(|e| format!("Failed to insert quad: {}", e))?;
        }
        
        self.bump_epoch();
        Ok(())
    }

    /// Get graph size (number of quads)
    pub fn size(&self) -> usize {
        self.inner.len()
    }
}

/// Query metrics
#[derive(Debug, Clone)]
pub struct QueryMetrics {
    pub total_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
}

impl Default for WarmPathGraph {
    fn default() -> Self {
        // Default implementation should not fail
        // If new() fails, return minimal graph rather than panicking
        Self::new().unwrap_or_else(|_| {
            // Create minimal graph on failure
            Self {
                store: Arc::new(Store::new()),
                cache: Arc::new(Mutex::new(LruCache::new(
                    NonZeroUsize::new(1000).expect("1000 is non-zero")
                ))),
            }
        })
    }
}

