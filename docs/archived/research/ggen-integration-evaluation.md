# ggen Techniques Integration Evaluation for KNHK

**Date**: December 2024  
**Purpose**: Evaluate how to best integrate ggen's RDF/SPARQL techniques into KNHK's three-tier architecture

## Executive Summary

**Recommendation**: **Hybrid Integration Strategy** - Use oxigraph for warm path operations (Rust-native performance), keep unrdf for cold path (complex operations, SHACL, reasoning).

### Key Findings

1. **Current State**: KNHK uses unrdf (JavaScript/Node.js) via FFI for cold path operations
2. **ggen Pattern**: Uses oxigraph (Rust-native) with caching wrapper for all RDF/SPARQL
3. **Integration Opportunity**: Replace warm path unrdf calls with oxigraph for better performance
4. **Preserve Cold Path**: Keep unrdf for complex operations that exceed oxigraph capabilities

## Architecture Comparison

### Current KNHK Architecture

```
┌─────────────────────────────────────┐
│      Hot Path (C)                   │
│      ≤8 ticks (2ns)                 │
│      SIMD-optimized                  │
│      18 micro-operations            │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Warm Path (Rust)               │
│      ≤500ms                         │
│      ETL Pipeline                   │
│      Connectors                     │
│      → knhk-unrdf FFI ──────────┐  │
└──────────────────────────────────┼──┘
                                    │
┌───────────────────────────────────▼──┐
│      Cold Path (unrdf/Node.js)      │
│      >500ms                         │
│      Complex SPARQL                 │
│      SHACL validation               │
│      Reasoning                      │
└──────────────────────────────────────┘
```

### Proposed Hybrid Architecture

```
┌─────────────────────────────────────┐
│      Hot Path (C)                   │
│      ≤8 ticks (2ns)                 │
│      SIMD-optimized                  │
│      18 micro-operations            │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Warm Path (Rust)               │
│      ≤500ms                         │
│      ETL Pipeline                   │
│      Connectors                     │
│      ┌──────────────────────────┐  │
│      │  oxigraph (Rust-native)  │  │
│      │  + Graph wrapper cache   │  │
│      │  Simple SPARQL queries   │  │
│      └──────────┬───────────────┘  │
│                 │                  │
│                 │ Complex queries  │
│                 ▼                  │
└─────────────────────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Cold Path (unrdf/Node.js)      │
│      >500ms                         │
│      Complex SPARQL                 │
│      SHACL validation               │
│      Reasoning                      │
└──────────────────────────────────────┘
```

## Integration Strategies

### Strategy 1: Full Replacement (NOT RECOMMENDED)

**Approach**: Replace unrdf entirely with oxigraph

**Pros**:
- Pure Rust, no FFI overhead
- Single codebase
- Better performance for warm path
- No Node.js dependency

**Cons**:
- Lose unrdf's SHACL validation capabilities
- Lose unrdf's reasoning (eyereasoner)
- Lose unrdf's lockchain integration
- Need to reimplement complex features
- Break existing cold path integrations

**Verdict**: ❌ **NOT RECOMMENDED** - Too much loss of functionality

---

### Strategy 2: Hybrid Approach (RECOMMENDED)

**Approach**: Use oxigraph for warm path, keep unrdf for cold path

#### Phase 1: Warm Path Optimization

**Add oxigraph to warm path** (`rust/knhk-warm/`):

```toml
[dependencies]
oxigraph = "0.5"
```

**Create Graph wrapper** (following ggen pattern):

```rust
// rust/knhk-warm/src/graph.rs
use oxigraph::store::Store;
use oxigraph::sparql::QueryResults;
use std::sync::{Arc, Mutex};
use lru::LruCache;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct WarmPathGraph {
    inner: Store,
    epoch: Arc<AtomicU64>,
    query_cache: Arc<Mutex<LruCache<u64, CachedResult>>>,
}

impl WarmPathGraph {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Store::new()?,
            epoch: Arc::new(AtomicU64::new(1)),
            query_cache: Arc::new(Mutex::new(LruCache::new(100))),
        })
    }
    
    /// Execute SPARQL query with caching
    pub fn query(&self, sparql: &str) -> Result<QueryResults> {
        let hash = self.hash_query(sparql);
        
        // Check cache
        if let Ok(cached) = self.query_cache.lock() {
            if let Some(result) = cached.get(&hash) {
                return Ok(result.clone());
            }
        }
        
        // Execute query
        let result = self.inner.query(sparql)?;
        
        // Cache result
        if let Ok(mut cache) = self.query_cache.lock() {
            cache.put(hash, result.clone());
        }
        
        Ok(result)
    }
    
    fn hash_query(&self, sparql: &str) -> u64 {
        // Simple hash for caching
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        sparql.hash(&mut hasher);
        hasher.finish()
    }
}
```

**Benefits**:
- ✅ Rust-native performance (no FFI overhead)
- ✅ Query caching (ggen pattern)
- ✅ Epoch-based cache invalidation
- ✅ Thread-safe operations
- ✅ Maintains ≤500ms warm path target

#### Phase 2: Path Selection Logic

**Route queries based on complexity**:

```rust
// rust/knhk-warm/src/path_selector.rs
pub enum QueryPath {
    Hot,      // ≤8 ticks, use C hot path
    Warm,     // ≤500ms, use oxigraph
    Cold,     // >500ms, use unrdf
}

pub fn select_path(query: &str, data_size: usize) -> QueryPath {
    // Check if query fits hot path constraints
    if is_hot_path_query(query) && data_size <= 8 {
        return QueryPath::Hot;
    }
    
    // Check if query fits warm path (oxigraph)
    if is_warm_path_query(query) && data_size <= 10000 {
        return QueryPath::Warm;
    }
    
    // Otherwise, use cold path (unrdf)
    QueryPath::Cold
}

fn is_hot_path_query(query: &str) -> bool {
    // Check for simple ASK/COUNT patterns
    query.contains("ASK") && !query.contains("OPTIONAL") &&
    !query.contains("FILTER") && !query.contains("UNION")
}

fn is_warm_path_query(query: &str) -> bool {
    // Check for SPARQL features oxigraph supports
    !query.contains("UPDATE") &&
    !query.contains("SHACL") &&
    !query.contains("reasoning")
}
```

**Benefits**:
- ✅ Automatic path selection
- ✅ Performance optimization
- ✅ Maintains architectural boundaries

#### Phase 3: Keep Cold Path for Complex Operations

**Use unrdf for**:
- Complex SPARQL (property paths, aggregates, ORDER BY, GROUP BY)
- SHACL validation
- OWL reasoning
- Lockchain integration
- Epistemology generation

**Benefits**:
- ✅ Preserve unrdf's full feature set
- ✅ Maintain existing integrations
- ✅ Best tool for complex operations

---

### Strategy 3: Incremental Migration (ALTERNATIVE)

**Approach**: Gradually migrate warm path operations to oxigraph

**Phase 1**: Add oxigraph alongside unrdf
**Phase 2**: Route simple queries to oxigraph
**Phase 3**: Optimize caching
**Phase 4**: Migrate more operations

**Benefits**:
- ✅ Low risk
- ✅ Gradual improvement
- ✅ Can rollback if needed

**Drawbacks**:
- ⚠️ Maintain two code paths temporarily
- ⚠️ More complex codebase

---

## Specific Integration Points

### 1. Warm Path Query Execution

**Current**: `rust/knhk-warm/src/warm_path.rs` → `knhk-unrdf` FFI

**Proposed**: Add oxigraph-based query execution

```rust
// rust/knhk-warm/src/warm_path.rs
use crate::graph::WarmPathGraph;

pub struct WarmPathExecutor {
    graph: WarmPathGraph,
    unrdf_client: Option<UnrdfClient>, // Keep for cold path fallback
}

impl WarmPathExecutor {
    pub fn execute_query(&self, query: &str) -> Result<QueryResult> {
        match select_path(query, self.graph.size()) {
            QueryPath::Hot => {
                // Route to C hot path
                self.execute_hot_path(query)
            }
            QueryPath::Warm => {
                // Use oxigraph
                self.graph.query(query)
            }
            QueryPath::Cold => {
                // Fall back to unrdf
                self.unrdf_client.as_ref()
                    .ok_or("unrdf not initialized")?
                    .query(query)
            }
        }
    }
}
```

### 2. RDF Parsing and Loading

**Current**: Custom RDF parser in `c/src/rdf.c`

**Proposed**: Use oxigraph for parsing in warm path

```rust
use oxigraph::io::RdfFormat;

pub fn load_rdf_turtle(&mut self, turtle_data: &str) -> Result<()> {
    self.graph.inner.load_from_reader(
        RdfFormat::Turtle,
        turtle_data.as_bytes()
    )?;
    self.bump_epoch(); // Invalidate cache
    Ok(())
}
```

**Benefits**:
- ✅ Better parsing performance
- ✅ More format support (N-Triples, RDF/XML, JSON-LD)
- ✅ Standard implementation

### 3. Query Result Caching

**Current**: No caching in warm path

**Proposed**: Implement ggen's caching pattern

```rust
pub struct CachedResult {
    boolean: Option<bool>,
    solutions: Option<Vec<BTreeMap<String, String>>>,
    graph: Option<Vec<String>>,
}

pub struct WarmPathGraph {
    inner: Store,
    epoch: Arc<AtomicU64>,
    query_cache: Arc<Mutex<LruCache<(u64, u64), CachedResult>>>, // (query_hash, epoch)
}
```

**Benefits**:
- ✅ Reduced query execution time
- ✅ Better warm path performance
- ✅ Epoch-based invalidation

### 4. SHACL Validation

**Current**: Only via unrdf (cold path)

**Options**:
1. **Option A**: Use `shacl_validation = "0.1"` crate (if available)
2. **Option B**: Keep unrdf for SHACL (recommended)

**Recommendation**: Keep unrdf for SHACL - it's mature and feature-complete

---

## Performance Impact Analysis

### Current Performance (unrdf FFI)

```
Warm Path Query:
  - FFI overhead: ~50-100μs
  - Node.js startup: ~100-500ms (first call)
  - Query execution: ~10-100ms
  - Total: ~150-700ms
```

### Proposed Performance (oxigraph)

```
Warm Path Query:
  - Rust-native: ~0μs overhead
  - Query execution: ~5-50ms
  - Cache hit: ~1-5μs
  - Total: ~5-50ms (10-14x faster)
```

### Cache Hit Performance

```
Cached Query:
  - Cache lookup: ~1-5μs
  - Result return: ~1μs
  - Total: ~2-6μs (100x faster than cold execution)
```

---

## Implementation Plan

### Phase 1: Foundation (Week 1-2)

1. **Add oxigraph dependency**
   ```toml
   # rust/knhk-warm/Cargo.toml
   [dependencies]
   oxigraph = "0.5"
   lru = "0.16"
   ```

2. **Create Graph wrapper** (`rust/knhk-warm/src/graph.rs`)
   - Implement Store wrapper
   - Add caching layer
   - Add epoch tracking

3. **Add path selection** (`rust/knhk-warm/src/path_selector.rs`)
   - Query complexity analysis
   - Path routing logic

### Phase 2: Integration (Week 3-4)

1. **Update warm path executor**
   - Add oxigraph query execution
   - Implement fallback to unrdf
   - Add metrics/observability

2. **Update ETL pipeline**
   - Use oxigraph for RDF parsing
   - Route queries appropriately
   - Maintain backward compatibility

### Phase 3: Optimization (Week 5-6)

1. **Cache tuning**
   - Optimize cache size
   - Tune eviction policies
   - Add cache metrics

2. **Performance testing**
   - Benchmark warm path queries
   - Compare with unrdf baseline
   - Validate ≤500ms target

### Phase 4: Documentation (Week 7)

1. **Update architecture docs**
2. **Add usage examples**
3. **Document path selection logic**

---

## Risk Assessment

### Low Risk ✅
- Adding oxigraph alongside unrdf
- Implementing Graph wrapper
- Adding caching layer

### Medium Risk ⚠️
- Path selection logic (may route incorrectly)
- Cache invalidation (may return stale data)
- Performance regression (if misconfigured)

### High Risk ❌
- Removing unrdf entirely (lose features)
- Breaking existing cold path integrations

---

## Recommendations

### Immediate Actions (v0.5.0)

1. ✅ **Add oxigraph to warm path** - Low risk, high reward
2. ✅ **Implement Graph wrapper** - Follow ggen pattern
3. ✅ **Add query caching** - Significant performance gain
4. ✅ **Keep unrdf for cold path** - Preserve functionality

### Future Enhancements (v1.0+)

1. **SHACL validation**: Evaluate `shacl_validation` crate vs unrdf
2. **OWL reasoning**: Keep unrdf (no Rust-native option)
3. **Lockchain integration**: Keep unrdf (tightly coupled)
4. **Epistemology generation**: Keep unrdf (complex logic)

---

## Conclusion

**Recommended Strategy**: **Hybrid Integration (Strategy 2)**

- Use oxigraph for warm path operations (Rust-native performance)
- Keep unrdf for cold path (complex operations, SHACL, reasoning)
- Implement ggen's Graph wrapper pattern for caching
- Add path selection logic for automatic routing

**Expected Benefits**:
- 10-14x performance improvement for warm path queries
- 100x improvement for cached queries
- Reduced FFI overhead
- Maintains architectural boundaries
- Preserves unrdf's full feature set

**Implementation Priority**: **HIGH** - Significant performance gains with low risk

