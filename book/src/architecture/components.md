# Key Components

## Data Layer (SoA Layout)

Triples are stored in Structure-of-Arrays format:
- `S[]` - Subject array (64-byte aligned)
- `P[]` - Predicate array (64-byte aligned)
- `O[]` - Object array (64-byte aligned)

All arrays are 64-byte aligned for optimal cache line access and SIMD operations.

## Connector Framework

**Dark Matter 80/20 Connector Framework**:
- Kafka connector with rdkafka integration
- Salesforce connector with reqwest integration
- HTTP, File, SAP connector support
- Circuit breaker pattern for resilience
- Health checking and metrics
- Guard validation (max_run_len ≤ 8, max_batch_size, max_lag_ms)

## ETL Pipeline

**Five-Stage Pipeline**:
- **Ingest**: Connector polling, RDF/Turtle parsing, JSON-LD support
- **Transform**: Schema validation (O ⊨ Σ), IRI hashing (FNV-1a), typed triples
- **Load**: Predicate run grouping, SoA conversion, 64-byte alignment
- **Reflex**: Hot path execution (≤8 ticks), receipt generation, receipt merging (⊕)
- **Emit**: Lockchain writing (Merkle-linked), downstream APIs (webhooks, Kafka, gRPC)

## Query Layer

**Hot Path** (≤8 ticks):
- Simple ASK queries
- COUNT queries (≤8 elements)
- Triple matching (S-P-O)
- Branchless SIMD execution
- Fully unrolled for NROWS=8
- CONSTRUCT8 operations
- Batch execution (≤8 hooks)

**Cold Path**:
- Complex queries (JOINs, OPTIONAL, UNION)
- Multi-predicate queries
- Full SPARQL compliance

## Evaluation Layer

- **Hook IR**: Lightweight query representation
- **Context**: SoA arrays + predicate run metadata
- **SIMD Operations**: ARM NEON / x86 AVX2
- **Receipts**: Timing, provenance, span IDs (OTEL-compatible)

## Reflexive Control Layer

**Erlang Supervision Tree**:
- **knhk_sigma**: Schema registry (Σ management)
- **knhk_q**: Invariant registry (Q constraints, preserve(Q))
- **knhk_ingest**: Delta ingestion (O ⊔ Δ)
- **knhk_lockchain**: Receipt storage (Merkle-linked)
- **knhk_hooks**: Hook installation and management
- **knhk_epoch**: Epoch scheduling (Λ ≺-total, τ ≤ 8)
- **knhk_route**: Action routing to downstream systems

## Lockchain

Merkle-linked provenance storage:
- URDNA2015 canonicalization + SHA-256 hashing
- Receipt storage with parent hash linking
- Git integration for audit trail
- Provenance verification: hash(A) = hash(μ(O))

## Observability

- OTEL span ID generation (not placeholders)
- Metrics collection
- Tracing support
- Receipt provenance tracking

## See Also

- [Three-Tier Architecture](three-tier.md) - Architecture overview
- [Data Flow](data-flow.md) - Execution flow

