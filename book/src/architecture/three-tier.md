# Three-Tier Architecture

## Hot Path (C)

**Goal**: ≤8 tick operations using SIMD optimization

### Characteristics
- Structure-of-Arrays (SoA) layout
- 64-byte alignment for SIMD
- Branchless operations
- 19 query operations (ASK, COUNT, COMPARE, SELECT, CONSTRUCT8)

### Operations
- Simple ASK queries
- COUNT queries (≤8 elements)
- Triple matching (S-P-O)
- Branchless SIMD execution
- Fully unrolled for NROWS=8
- Batch execution (≤8 hooks)

### Performance
- **Hot Path**: ≤8 ticks (Chatman Constant: 2ns = 8 ticks)
- **Critical Path**: Separated from receipt generation overhead
- **SoA Layout**: 64-byte alignment for SIMD operations
- **Branchless**: Constant-time execution on hot path

### Selection Criteria
- Simple ASK queries (no FILTER, OPTIONAL, UNION)
- Data size ≤8 triples
- Single predicate queries

## Warm Path (Rust + oxigraph)

**Goal**: ≤500ms SPARQL query execution with caching

### Components
- **oxigraph**: Rust-native RDF store and SPARQL query engine
- **Query Caching**: LRU cache for query results (1000 entries)
- **Query Plan Caching**: Parsed SPARQL queries cached to reduce parsing overhead
- **Epoch-based Invalidation**: Cache automatically invalidated when data changes
- **OTEL Integration**: Query latency, cache hit rate, and query count metrics
- ETL Pipeline (Ingest → Transform → Load → Reflex → Emit)
- Connector framework (Kafka, Salesforce)
- Lockchain integration (Merkle-linked receipts)

### ETL Pipeline Stages
1. **Ingest**: Connector polling, RDF/Turtle parsing, JSON-LD support
2. **Transform**: Schema validation (O ⊨ Σ), IRI hashing (FNV-1a), typed triples
3. **Load**: Predicate run grouping, SoA conversion, 64-byte alignment
4. **Reflex**: Hot path execution (≤8 ticks), receipt generation, receipt merging (⊕)
5. **Emit**: Lockchain writing (Merkle-linked), downstream APIs (webhooks, Kafka, gRPC)

### Warm Path Query Execution
- SPARQL SELECT, ASK, CONSTRUCT, DESCRIBE queries
- Data size ≤10K triples
- Basic SPARQL features (FILTER, OPTIONAL, UNION)
- No UPDATE queries
- No SHACL validation
- No OWL reasoning

### Performance Characteristics
- **Query Execution**: 5-50ms (depending on data size and query complexity)
- **Cache Hit**: 2-6μs (100x faster than cold execution)
- **Target**: ≤500ms for warm path queries
- **Cache Hit Rate**: Typically 60-80% for repeated queries
- **Speedup**: 10-14x faster than unrdf FFI (5-50ms vs 150-700ms)

### Selection Criteria
- SPARQL queries (SELECT, ASK, CONSTRUCT, DESCRIBE)
- Data size ≤10K triples
- Basic SPARQL features (FILTER, OPTIONAL, UNION)
- No UPDATE/SHACL/reasoning operations

## Cold Path (Erlang/unrdf)

**Goal**: Complex queries, SHACL validation, and reasoning

### Components
- **unrdf**: JavaScript-based knowledge engine
- SPARQL query execution (full compliance)
- SHACL validation
- OWL reasoning
- Schema registry (knhk_sigma)
- Invariant registry (knhk_q)
- Lockchain integration
- Epistemology generation

### Reflexive Control Layer
- **knhk_sigma**: Schema registry (Σ management)
- **knhk_q**: Invariant registry (Q constraints, preserve(Q))
- **knhk_ingest**: Delta ingestion (O ⊔ Δ)
- **knhk_lockchain**: Receipt storage (Merkle-linked)
- **knhk_hooks**: Hook installation and management
- **knhk_epoch**: Epoch scheduling (Λ ≺-total, τ ≤ 8)
- **knhk_route**: Action routing to downstream systems

### Selection Criteria
- Complex queries exceeding warm path constraints
- UPDATE queries
- SHACL validation
- OWL reasoning
- Data size >10K triples
- Very complex property paths or UNION patterns

## Path Selection

Queries automatically route to the appropriate path based on:

### Query Complexity Analysis
- **Simple ASK queries** → Hot path
- **SPARQL queries** → Warm path (oxigraph)
- **Complex queries** → Cold path (unrdf)

### Data Size Constraints
- **≤8 triples** → Hot path
- **≤10K triples** → Warm path
- **>10K triples** → Cold path

### Operation Type
- **Simple operations** (ASK, COUNT) → Hot path
- **SPARQL queries** (SELECT, CONSTRUCT) → Warm path
- **Complex operations** (UPDATE, SHACL, reasoning) → Cold path

### Automatic Routing
The path selector (`knhk-etl::path_selector`) analyzes queries and routes them automatically:
- Checks query features (FILTER, OPTIONAL, UNION, UPDATE, SHACL)
- Validates data size constraints
- Routes to optimal execution path

## Performance Benchmarks

### Hot Path
- **Target**: ≤8 ticks (≤2ns)
- **Current**: Most operations ~1-1.5ns
- **Measurement**: External timing by Rust framework

### Warm Path
- **Target**: ≤500ms
- **Typical**: 5-50ms for SELECT queries
- **Cache Hit**: 2-6μs
- **Speedup**: 10-14x faster than unrdf FFI

### Cold Path
- **Target**: Acceptable for complex operations
- **Typical**: 150-700ms (including FFI overhead)
- **Use Case**: Complex queries, SHACL, reasoning

## See Also

- [Key Components](components.md) - Detailed component descriptions
- [Data Flow](data-flow.md) - Execution flow diagrams
- [Performance Guide](../reference/performance.md) - Performance optimization guide
- [ggen Integration Guide](../../docs/ggen-integration-guide.md) - Integration guide

