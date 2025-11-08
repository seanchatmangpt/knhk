# How ggen Handles RDF/OWL/SHACL/SPARQL

## Overview

Based on research of `~/ggen` Cargo.toml files and source code, here's how ggen integrates RDF/OWL/SHACL/SPARQL technologies:

## Core Dependencies

### Workspace-Level (`Cargo.toml`)
```toml
oxigraph = "0.5.1"  # RDF/SPARQL engine
```

### Crate-Level (`crates/ggen-core/Cargo.toml`)
```toml
oxigraph = "0.5"              # RDF store and SPARQL query engine
shacl_validation = "0.1"      # SHACL validation (declared but not actively used)
srdf = "0.1"                  # RDF serialization (declared but not actively used)
```

## Primary Library: Oxigraph

**Oxigraph** is the core RDF/SPARQL library used throughout ggen:

### Key Features Used:
1. **RDF Store** (`oxigraph::store::Store`)
   - In-memory RDF graph storage
   - Thread-safe, cloneable for sharing
   - Supports multiple RDF formats (Turtle, N-Triples, RDF/XML, JSON-LD)

2. **SPARQL Querying** (`oxigraph::sparql::QueryResults`)
   - SELECT queries (returns solutions/bindings)
   - ASK queries (returns boolean)
   - CONSTRUCT queries (returns graph)
   - DESCRIBE queries (returns graph)

3. **RDF Models** (`oxigraph::model`)
   - `NamedNode` - URIs
   - `Quad` - RDF triples/quads
   - `Term` - RDF terms (named nodes, blank nodes, literals)
   - `GraphName` - Named graph support

4. **RDF I/O** (`oxigraph::io::RdfFormat`)
   - Turtle parsing/serialization
   - N-Triples format
   - RDF/XML
   - JSON-LD

## Usage Patterns

### 1. Graph Creation and Loading
```rust
use oxigraph::store::Store;
use oxigraph::io::RdfFormat;

// Create empty store
let store = Store::new()?;

// Load from Turtle file
store.load_from_reader(RdfFormat::Turtle, turtle_bytes)?;
```

### 2. SPARQL Query Execution
```rust
use oxigraph::sparql::QueryResults;

// Execute query
let results: QueryResults = store.query(sparql_query)?;

// Handle results
match results {
    QueryResults::Boolean(b) => { /* boolean result */ }
    QueryResults::Solutions(solutions) => {
        for solution in solutions {
            // Process bindings
        }
    }
    QueryResults::Graph(quads) => {
        // Process quads
    }
}
```

### 3. Triple Insertion
```rust
use oxigraph::model::{NamedNode, Quad, GraphName};

let s = NamedNode::new("http://example.org/subject")?;
let p = NamedNode::new("http://example.org/predicate")?;
let o = NamedNode::new("http://example.org/object")?;
let quad = Quad::new(s, p, o, GraphName::DefaultGraph);

store.insert(&quad)?;
```

## Example Projects

### 1. `sparql-engine` Example
**Dependencies:**
```toml
oxigraph = "0.5"
```

**Features:**
- Natural language to SPARQL conversion
- JSON to SPARQL conversion
- Query streaming
- Multiple query types (SELECT, CONSTRUCT, ASK, DESCRIBE)

### 2. `knowledge-graph-builder` Example
**Dependencies:**
```toml
oxigraph = "0.5"
```

**Features:**
- AI-powered ontology generation
- Template-driven code generation from graphs
- SPARQL query generation and execution
- Multi-format export (RDF, JSON-LD, Turtle)

### 3. `advanced-sparql-graph` Example
**Dependencies:**
- Uses `ggen-core` which provides `oxigraph` integration

## Architecture: Graph Wrapper

ggen-core provides a `Graph` wrapper around `oxigraph::store::Store`:

```rust
pub struct Graph {
    inner: Store,
    epoch: Arc<AtomicU64>,
    plan_cache: Arc<Mutex<LruCache<u64, String>>>,
    result_cache: Arc<Mutex<LruCache<(u64, u64), CachedResult>>>,
}
```

**Features:**
- Thread-safe with shared store
- SPARQL query plan caching
- Result caching (Boolean, Solutions, Graph)
- Epoch-based cache invalidation

## SHACL and OWL Support

### SHACL (`shacl_validation = "0.1"`)
- **Status:** Declared in Cargo.toml but **not actively used** in source code
- **Potential:** Could be used for schema validation
- **File patterns:** `templates/**/graphs/shapes/*.shacl.ttl` (mentioned in `gpack.rs`)

### OWL
- **Status:** No explicit OWL dependencies
- **Handling:** Oxigraph provides basic RDF/OWL support through standard RDF primitives
- **Reasoning:** Not explicitly supported (would need OWL reasoner like Pellet, Hermit, etc.)

### SRDF (`srdf = "0.1"`)
- **Status:** Declared but **not actively used** in source code
- **Purpose:** Likely intended for RDF serialization/parsing utilities

## Comparison with KNHK/unrdf Approach

### ggen (Rust):
- **Primary:** `oxigraph` (Rust-native, pure Rust implementation)
- **SPARQL:** Built-in query engine
- **Store:** In-memory with optional persistence
- **Validation:** SHACL support declared but unused

### unrdf (JavaScript/Node.js):
- **Primary:** `n3.js` (RDF store), `@comunica/query-sparql` (SPARQL)
- **Validation:** `rdf-validate-shacl`
- **Reasoning:** `eyereasoner` (N3 reasoning)
- **Canonicalization:** `rdf-canonize` (URDNA2015)

## Recommendations for KNHK Integration

### Option 1: Use Oxigraph (Rust-native)
```toml
[dependencies]
oxigraph = "0.5"
```

**Pros:**
- Pure Rust, no FFI overhead
- Good performance
- Active maintenance
- SPARQL 1.1 support

**Cons:**
- Different from unrdf's JavaScript stack
- Would need separate implementations

### Option 2: Bridge to unrdf (Current Approach)
- Keep `knhk-unrdf` FFI layer
- Use unrdf's JavaScript ecosystem
- Unified knowledge graph across languages

**Pros:**
- Single knowledge graph implementation
- Leverages unrdf's full feature set
- Consistent with cold path architecture

**Cons:**
- FFI overhead
- Node.js runtime dependency

### Option 3: Hybrid Approach
- Hot path: Rust-native (oxigraph for simple queries)
- Cold path: unrdf (complex SPARQL, SHACL, reasoning)
- Best of both worlds

## Key Takeaways

1. **Oxigraph is the standard** for Rust RDF/SPARQL
2. **SHACL validation is available** but not actively used in ggen
3. **Store pattern:** Thread-safe wrapper with caching
4. **Format support:** Turtle primary, others available
5. **Query types:** Full SPARQL 1.1 support (SELECT, ASK, CONSTRUCT, DESCRIBE)

For KNHK's autonomous DoD validator, consider:
- Using `oxigraph` for Rust-native RDF operations
- Keeping `knhk-unrdf` for cold path complex operations
- Leveraging ggen's `Graph` wrapper pattern for caching/performance

