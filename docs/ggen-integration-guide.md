# ggen Integration Guide

This guide explains how to use KNHK's warm path with oxigraph integration, when to use warm vs cold path, performance tuning, and migration from unrdf-only configurations.

## Overview

KNHK now supports a three-tier architecture:
- **Hot Path** (C): ≤8 ticks, simple ASK queries
- **Warm Path** (Rust + oxigraph): ≤500ms, SPARQL queries
- **Cold Path** (Erlang/unrdf): Complex operations, SHACL, reasoning

The warm path integration uses [oxigraph](https://github.com/oxigraph/oxigraph), a Rust-native RDF store and SPARQL query engine, providing 10-14x performance improvement over unrdf FFI.

## Quick Start

### Basic Usage

```rust
use knhk_warm::{WarmPathGraph, execute_select, execute_ask};

// Create a warm path graph
let graph = WarmPathGraph::new()?;

// Load RDF data
let turtle_data = r#"
    <s1> <p1> <o1> .
    <s2> <p1> <o2> .
"#;
graph.load_from_turtle(turtle_data)?;

// Execute SELECT query
let query = "SELECT ?s ?o WHERE { ?s <p1> ?o }";
let result = execute_select(&graph, query)?;
println!("Found {} bindings", result.bindings.len());

// Execute ASK query
let ask_query = "ASK { <s1> <p1> <o1> }";
let ask_result = execute_ask(&graph, ask_query)?;
println!("Result: {}", ask_result.result);
```

### Using Warm Path Executor

```rust
use knhk_warm::WarmPathExecutor;

// Create executor with automatic path selection
let mut executor = WarmPathExecutor::new()?;

// Load RDF data
executor.load_rdf(turtle_data)?;

// Execute query (automatically routes to warm or cold path)
let result = executor.execute_query("SELECT ?s WHERE { ?s <p1> ?o }")?;

match result {
    QueryExecutionResult::Select(select) => {
        println!("Found {} bindings", select.bindings.len());
    }
    QueryExecutionResult::Ask(ask) => {
        println!("Result: {}", ask.result);
    }
    // ... other result types
}
```

## When to Use Warm vs Cold Path

### Use Warm Path (oxigraph) When:
- ✅ SPARQL SELECT, ASK, CONSTRUCT, DESCRIBE queries
- ✅ Data size ≤10K triples
- ✅ Basic SPARQL features (FILTER, OPTIONAL, UNION)
- ✅ Need fast query execution (5-50ms)
- ✅ Want caching for repeated queries

### Use Cold Path (unrdf) When:
- ✅ UPDATE queries (INSERT, DELETE, LOAD, CLEAR)
- ✅ SHACL validation
- ✅ OWL reasoning
- ✅ Data size >10K triples
- ✅ Very complex property paths
- ✅ Lockchain integration required
- ✅ Epistemology generation

### Use Hot Path (C) When:
- ✅ Simple ASK queries (no FILTER, OPTIONAL, UNION)
- ✅ COUNT queries (≤8 elements)
- ✅ Data size ≤8 triples
- ✅ Need ≤2ns execution time

## Path Selection

KNHK automatically selects the appropriate path based on query analysis:

```rust
use knhk_etl::path_selector::{select_path, QueryPath};

let query = "SELECT ?s WHERE { ?s <p1> ?o }";
let data_size = 100; // Number of triples
let path = select_path(query, data_size);

match path {
    QueryPath::Hot => println!("Routing to hot path"),
    QueryPath::Warm => println!("Routing to warm path (oxigraph)"),
    QueryPath::Cold => println!("Routing to cold path (unrdf)"),
}
```

### Path Selection Logic

1. **Hot Path**: Simple ASK, data size ≤8
2. **Warm Path**: SPARQL queries, data size ≤10K, no UPDATE/SHACL/reasoning
3. **Cold Path**: Complex queries, SHACL validation, reasoning, updates

## Performance Tuning

### Cache Configuration

The warm path uses two caches:
- **Query Result Cache**: 1000 entries (LRU)
- **Query Plan Cache**: 1000 entries (parsed SPARQL queries)

Cache sizes are configured in `WarmPathGraph::new()`:

```rust
// Cache sizes are hardcoded in the implementation
// Query cache: 1000 entries
// Query plan cache: 1000 entries
```

### Cache Invalidation

Cache is automatically invalidated when data changes:

```rust
// Inserting triples invalidates cache
graph.insert_triple("s", "p", "o")?; // Cache invalidated

// Loading data invalidates cache
graph.load_from_turtle(turtle_data)?; // Cache invalidated

// Manual cache invalidation
graph.bump_epoch(); // Increments epoch, invalidates cache
```

### Monitoring Cache Performance

```rust
use knhk_warm::WarmPathGraph;

let graph = WarmPathGraph::new()?;

// Execute queries...
// ...

// Check cache metrics
let metrics = graph.get_metrics();
println!("Total queries: {}", metrics.total_queries);
println!("Cache hits: {}", metrics.cache_hits);
println!("Cache misses: {}", metrics.cache_misses);
println!("Cache hit rate: {:.1}%", metrics.cache_hit_rate * 100.0);
```

### OTEL Metrics

Enable OTEL feature for detailed metrics:

```toml
[dependencies]
knhk-warm = { path = "../knhk-warm", features = ["otel"] }
```

Metrics include:
- `knhk.warm.query.latency_ms`: Query execution latency
- `knhk.warm.query.cache_hit_rate`: Cache hit rate
- Query count, cache hits, cache misses

## Migration Guide

### From unrdf-Only to Hybrid (oxigraph + unrdf)

#### Step 1: Update Dependencies

```toml
[dependencies]
knhk-warm = { path = "../knhk-warm", features = ["std", "otel"] }
knhk-etl = { path = "../knhk-etl", features = ["std"] }
```

#### Step 2: Update Query Execution

**Before (unrdf-only)**:
```rust
use knhk_unrdf::query_sparql;

let result = query_sparql(sparql_query)?;
```

**After (hybrid with path selection)**:
```rust
use knhk_warm::WarmPathExecutor;

let executor = WarmPathExecutor::new()?;
executor.load_rdf(turtle_data)?;
let result = executor.execute_query(sparql_query)?;
```

#### Step 3: Configure Cold Path Fallback

```rust
use knhk_warm::WarmPathExecutor;

let mut executor = WarmPathExecutor::new()?;

// Initialize unrdf for cold path fallback
#[cfg(feature = "unrdf")]
executor.init_unrdf("/path/to/unrdf")?;

// Queries automatically route to warm or cold path
let result = executor.execute_query(query)?;
```

#### Step 4: Update ETL Pipeline Integration

```rust
use knhk_etl::integration::IntegratedPipeline;
use knhk_warm::WarmPathExecutor;

let mut pipeline = IntegratedPipeline::new(
    connectors,
    schema_iri,
    lockchain_enabled,
    downstream_endpoints,
);

// Set warm path executor
#[cfg(feature = "std")]
{
    let executor = WarmPathExecutor::new()?;
    pipeline.set_warm_path_executor(Box::new(executor));
}

// Execute pipeline
let result = pipeline.execute()?;
```

## Performance Benchmarks

### Warm Path (oxigraph)
- **Query Execution**: 5-50ms (depending on data size)
- **Cache Hit**: 2-6μs (100x faster)
- **Target**: ≤500ms
- **Speedup**: 10-14x faster than unrdf FFI

### Cold Path (unrdf)
- **Query Execution**: 150-700ms (including FFI overhead)
- **Use Case**: Complex queries, SHACL, reasoning

### Cache Performance
- **Hit Rate**: Typically 60-80% for repeated queries
- **Cache Hit Latency**: 2-6μs
- **Cache Miss Latency**: 5-50ms

## Best Practices

### 1. Use Warm Path for Repeated Queries
Warm path caching is most effective for queries that are executed multiple times.

### 2. Batch Similar Queries
Group similar queries together to maximize cache hit rate.

### 3. Monitor Cache Metrics
Regularly check cache hit rates and adjust query patterns if needed.

### 4. Use Cold Path for Complex Operations
Don't try to force complex queries through warm path - use cold path for UPDATE, SHACL, reasoning.

### 5. Load Data Efficiently
Load RDF data once and reuse the graph instance for multiple queries.

### 6. Enable OTEL for Production
Enable OTEL feature for detailed performance monitoring in production.

## Troubleshooting

### Low Cache Hit Rate
- Check if queries are being reused
- Verify epoch isn't being bumped unnecessarily
- Consider increasing cache size (requires code changes)

### Slow Query Execution
- Check data size (should be ≤10K triples for warm path)
- Verify query complexity (should not include UPDATE/SHACL)
- Check OTEL metrics for latency breakdown

### Cache Not Working
- Verify cache is enabled (default: enabled)
- Check epoch invalidation isn't too frequent
- Ensure query strings are identical (whitespace matters)

## Examples

See `rust/knhk-warm/examples/warm_path_query.rs` for complete examples including:
- SELECT queries
- ASK queries
- CONSTRUCT queries
- Cache performance demonstration
- Query metrics
- Batch query execution

## References

- [Architecture Documentation](../docs/architecture.md)
- [Three-Tier Architecture](../book/src/architecture/three-tier.md)
- [oxigraph Documentation](https://docs.rs/oxigraph/)
- [ggen Integration Evaluation](../docs/ggen-integration-evaluation.md)

