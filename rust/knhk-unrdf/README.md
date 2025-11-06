# knhk-unrdf

Cold path integration layer for UNRDF knowledge hook engine.

## Overview

`knhk-unrdf` provides integration between KNHK and the UNRDF JavaScript knowledge graph engine for cold path operations. The crate supports both native Rust implementations (via oxigraph) and JavaScript integration (via Node.js FFI) for complex SPARQL queries and knowledge hook execution.

## Quick Start

```rust
use knhk_unrdf::{init_unrdf, register_hook, execute_hook, query_sparql};

// Initialize unrdf integration (requires unrdf directory)
init_unrdf("./vendors/unrdf")?;

// Register a hook
let hook = HookDefinition {
    id: "check-permission".to_string(),
    query: "ASK WHERE { ?s <http://example.org/hasPermission> ?o }".to_string(),
    query_type: SparqlQueryType::Ask,
};

register_hook(&hook)?;

// Execute hook
let result = execute_hook("check-permission", None)?;
println!("Hook result: {}", result.result);

// Execute SPARQL query
let query_result = query_sparql(
    "SELECT ?s ?o WHERE { ?s <http://example.org/name> ?o }",
    None,
)?;
```

## Key Features

### Native Rust Implementation (`native` feature)

- **Rust-Native Hooks Engine**: Pure Rust implementation using oxigraph
- **SPARQL Execution**: SELECT, ASK, CONSTRUCT, DESCRIBE queries
- **RDF Canonicalization**: SHA-256 and Blake3 hashing
- **Query Caching**: LRU cache for query results
- **Concurrent Execution**: Parallel hook evaluation

### JavaScript Integration (`unrdf` feature)

- **Node.js FFI**: Integration with UNRDF JavaScript engine
- **Cold Path Operations**: Complex SPARQL queries
- **Transaction Support**: Begin, commit, rollback transactions
- **SHACL Validation**: Shape constraint validation
- **RDF Serialization**: Multiple format support (Turtle, JSON-LD, N-Triples)

## Architecture

```
┌─────────────────────────────────────┐
│      knhk-unrdf (Rust Layer)        │
├─────────────────────────────────────┤
│  Native (oxigraph) │  FFI (Node.js) │
│  - Hooks Engine     │  - UNRDF Engine│
│  - Query Execution  │  - Transactions│
│  - Canonicalization │  - SHACL       │
└─────────────────────────────────────┘
```

## Usage Examples

### Native Rust Hooks Engine

```rust
#[cfg(feature = "native")]
use knhk_unrdf::hooks_native::{HookRegistry, execute_hook_native};

let mut registry = HookRegistry::new();

// Register hook
let hook = HookDefinition {
    id: "my-hook".to_string(),
    query: "ASK WHERE { ?s <http://example.org/pred> ?o }".to_string(),
    query_type: SparqlQueryType::Ask,
};

registry.register(hook)?;

// Execute hook
let result = execute_hook_native(&registry, "my-hook", None)?;
```

### JavaScript Integration

```rust
#[cfg(feature = "unrdf")]
use knhk_unrdf::{init_unrdf, store_turtle_data, query_sparql};

// Initialize
init_unrdf("./vendors/unrdf")?;

// Store RDF data
store_turtle_data(
    r#"
    <http://example.org/alice> <http://example.org/name> "Alice" .
    <http://example.org/bob> <http://example.org/name> "Bob" .
    "#,
)?;

// Query data
let result = query_sparql(
    "SELECT ?name WHERE { ?s <http://example.org/name> ?name }",
    None,
)?;
```

### Transaction Support

```rust
use knhk_unrdf::{begin_transaction, transaction_add, commit_transaction};

let tx_id = begin_transaction()?;

// Add triples to transaction
transaction_add(tx_id, 
    "<http://example.org/subject>",
    "<http://example.org/predicate>",
    "<http://example.org/object>",
)?;

// Commit transaction
commit_transaction(tx_id)?;
```

## Dependencies

### Native Feature
- `oxigraph` - Rust SPARQL engine and RDF parsing
- `sha2`, `blake3` - Hashing
- `lru` - Query caching
- `rayon` - Parallel execution

### UNRDF Feature
- `tokio` - Async runtime
- `tera` - Template rendering
- `serde`, `serde_json` - Serialization

## Performance

### Native Implementation
- **Hook Execution**: <100ms for simple ASK queries
- **Query Caching**: LRU cache with configurable size
- **Parallel Execution**: Concurrent hook evaluation

### JavaScript Integration
- **Cold Path**: Unbounded latency for complex queries
- **Transaction Overhead**: ~10-50ms per transaction
- **SHACL Validation**: Depends on graph size

## Feature Flags

- **`native`**: Enable Rust-native hooks engine (oxigraph)
- **`unrdf`**: Enable JavaScript integration (Node.js FFI)
- **`std`**: Enable standard library features

## Related Documentation

- [Technical Documentation](docs/README.md) - Detailed API reference
- [Architecture](../../docs/architecture.md) - System architecture
- [UNRDF Integration](../../docs/unrdf-integration-status.md) - Integration status
- [Cold Path](../../docs/architecture.md#cold-path) - Cold path operations

