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
- CONSTRUCT8 operations
- Batch execution (≤8 hooks)

### Performance
- **Hot Path**: ≤8 ticks (Chatman Constant: 2ns = 8 ticks)
- **Critical Path**: Separated from receipt generation overhead
- **SoA Layout**: 64-byte alignment for SIMD operations
- **Branchless**: Constant-time execution on hot path

## Warm Path (Rust)

**Goal**: Safe abstractions over hot path

### Components
- ETL Pipeline (Ingest → Transform → Load → Reflex → Emit)
- Connector framework (Kafka, Salesforce)
- Lockchain integration (Merkle-linked receipts)
- OTEL observability

### ETL Pipeline Stages
1. **Ingest**: Connector polling, RDF/Turtle parsing, JSON-LD support
2. **Transform**: Schema validation (O ⊨ Σ), IRI hashing (FNV-1a), typed triples
3. **Load**: Predicate run grouping, SoA conversion, 64-byte alignment
4. **Reflex**: Hot path execution (≤8 ticks), receipt generation, receipt merging (⊕)
5. **Emit**: Lockchain writing (Merkle-linked), downstream APIs (webhooks, Kafka, gRPC)

## Cold Path (Erlang)

**Goal**: Complex queries and validation

### Components
- SPARQL query execution
- SHACL validation
- Schema registry (knhk_sigma)
- Invariant registry (knhk_q)

### Reflexive Control Layer
- **knhk_sigma**: Schema registry (Σ management)
- **knhk_q**: Invariant registry (Q constraints, preserve(Q))
- **knhk_ingest**: Delta ingestion (O ⊔ Δ)
- **knhk_lockchain**: Receipt storage (Merkle-linked)
- **knhk_hooks**: Hook installation and management
- **knhk_epoch**: Epoch scheduling (Λ ≺-total, τ ≤ 8)
- **knhk_route**: Action routing to downstream systems

## Path Selection

Queries route to either hot path or cold path based on:
- **Complexity**: Simple queries → hot path, complex queries → cold path
- **Data characteristics**: Predicate run size ≤ 8 → hot path
- **Operation type**: Operations in H_hot set → hot path

## See Also

- [Key Components](components.md) - Detailed component descriptions
- [Data Flow](data-flow.md) - Execution flow diagrams

