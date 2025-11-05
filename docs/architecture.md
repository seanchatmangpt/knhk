# Architecture

## System Overview

KNHK (v0.5.0) implements a multi-tier architecture with production-ready infrastructure:

1. **Hot Path Engine** (C) - ≤8 tick query execution (ASK, COUNT, COMPARE, VALIDATE)
2. **Warm Path Engine** (Rust) - ≤500ms emit operations (CONSTRUCT8)
3. **Connector Framework** (Rust) - Enterprise data source integration
4. **ETL Pipeline** (Rust) - Ingest → Transform → Load → Reflex → Emit
5. **Reflexive Control Layer** (Erlang) - Schema, invariants, receipts, routing
6. **Observability** (OTEL) - Metrics, tracing, span generation

Queries route to hot path (≤8 ticks), warm path (≤500ms), or cold path (full SPARQL engine) based on operation type and complexity.

## Modular Code Organization

### Header Structure

The public API is organized into modular headers for maintainability:

```
include/
├── knhk.h              # Main umbrella header (includes all components)
└── knhk/
    ├── types.h          # Type definitions (enums, structs, constants)
    ├── eval.h           # Query evaluation functions (eval_bool, eval_construct8)
    ├── receipts.h       # Receipt operations (receipt_merge)
    └── utils.h          # Utility functions (init_ctx, load_rdf, clock utilities)
```

**Usage**: Include only `knhk.h` - it automatically includes all sub-modules:
```c
#include "knhk.h"  // Includes all API components
```

### Source Structure

SIMD operations are organized into focused modules:

```
src/
├── simd.h               # SIMD umbrella header (includes all SIMD modules)
├── simd/
│   ├── common.h         # Common infrastructure (includes, declarations)
│   ├── existence.h      # ASK operations (exists_8, exists_o_8, spo_exists_8)
│   ├── count.h          # COUNT operations (count_8)
│   ├── compare.h        # Comparison operations (compare_o_8)
│   ├── select.h         # SELECT operations (select_gather_8)
│   ├── validate.h       # Datatype validation (validate_datatype_sp_8)
│   └── construct.h      # CONSTRUCT8 operations (construct8_emit_8)
├── simd.c               # Variable-length SIMD implementations
├── core.c               # Core operations (batch execution)
├── rdf.c                # RDF parsing (Turtle format)
└── clock.c              # Timing utilities and span ID generation
```

### Test Structure

Tests are organized into focused suites:

```
tests/
├── chicago_enterprise_use_cases.c    # Main enterprise test runner
├── chicago_basic_operations.c        # Basic operations (tests 1,2,5)
├── chicago_cardinality.c             # Cardinality tests (tests 3,6,7,9)
├── chicago_object_operations.c       # Object operations (tests 8,10,11,12)
├── chicago_advanced.c                # Advanced operations (tests 4,13-19)
├── chicago_test_helpers.c           # Shared test infrastructure
├── chicago_test_helpers.h            # Test helper declarations
├── chicago_v1_test.c                 # v1.0 test runner
├── chicago_v1_receipts.c             # Receipt tests
├── chicago_v1_operations.c            # Operation tests (CONSTRUCT8, batch)
├── chicago_v1_validation.c           # Validation tests (guards, constants)
├── chicago_integration_v2.c          # Integration test runner
├── chicago_integration_core.c        # Core integration tests
├── chicago_integration_systems.c     # System integration (lockchain, OTEL)
└── chicago_integration_advanced.c    # Advanced integration tests
```

## Core Components

### 1. Data Layer (SoA Layout)

Triples are stored in Structure-of-Arrays format:
- `S[]` - Subject array (64-byte aligned)
- `P[]` - Predicate array (64-byte aligned)
- `O[]` - Object array (64-byte aligned)

All arrays are 64-byte aligned for optimal cache line access and SIMD operations.

### 2. Connector Framework (v0.4.0)

**Dark Matter 80/20 Connector Framework**:
- Kafka connector with rdkafka integration
- Salesforce connector with reqwest integration
- HTTP, File, SAP connector support
- Circuit breaker pattern for resilience
- Health checking and metrics
- Guard validation (max_run_len ≤ 8, max_batch_size, max_lag_ms)

### 3. ETL Pipeline (v0.4.0)

**Five-Stage Pipeline**:
- **Ingest**: Connector polling, RDF/Turtle parsing, JSON-LD support
- **Transform**: Schema validation (O ⊨ Σ), IRI hashing (FNV-1a), typed triples
- **Load**: Predicate run grouping, SoA conversion, 64-byte alignment
- **Reflex**: Hot path execution (≤8 ticks), receipt generation, receipt merging (⊕)
- **Emit**: Lockchain writing (Merkle-linked), downstream APIs (webhooks, Kafka, gRPC)

### 4. Query Layer

**Hot Path** (≤8 ticks, ≤2ns):
- Simple ASK queries
- COUNT queries (≤8 elements)
- Triple matching (S-P-O)
- Branchless SIMD execution
- Fully unrolled for NROWS=8
- Batch execution (≤8 hooks)

**Warm Path** (≤500ms, Rust + oxigraph):
- SPARQL SELECT, ASK, CONSTRUCT, DESCRIBE queries
- Data size ≤10K triples
- Basic SPARQL features (FILTER, OPTIONAL, UNION)
- Query result caching (LRU cache, 1000 entries)
- Query plan caching (parsed SPARQL queries)
- OTEL metrics and observability
- Automatic path selection based on query complexity

**Cold Path** (unrdf):
- Complex queries (JOINs, OPTIONAL, UNION)
- Multi-predicate queries
- UPDATE queries
- SHACL validation
- OWL reasoning
- Full SPARQL compliance
- Lockchain integration

### 5. Evaluation Layer

- **Hook IR**: Lightweight query representation
- **Context**: SoA arrays + predicate run metadata
- **SIMD Operations**: ARM NEON / x86 AVX2
- **Receipts**: Timing, provenance, span IDs (OTEL-compatible)

### 6. Reflexive Control Layer (v0.4.0)

**Erlang Supervision Tree**:
- **knhk_sigma**: Schema registry (Σ management)
- **knhk_q**: Invariant registry (Q constraints, preserve(Q))
- **knhk_ingest**: Delta ingestion (O ⊔ Δ)
- **knhk_lockchain**: Receipt storage (Merkle-linked)
- **knhk_hooks**: Hook installation and management
- **knhk_epoch**: Epoch scheduling (Λ ≺-total, τ ≤ 8)
- **knhk_route**: Action routing to downstream systems

## Architecture Diagram

See `architecture.mmd` for visual representation.

## Data Structures

### knhk_context_t
```c
typedef struct {
  const uint64_t *S;        // Subject array (KNHK_ALIGN aligned, KNHK_NROWS sized)
  const uint64_t *P;        // Predicate array
  const uint64_t *O;        // Object array
  size_t triple_count;      // Number of loaded triples
  knhk_pred_run_t run;     // Predicate run metadata
} knhk_context_t;
```

### knhk_hook_ir_t
```c
typedef struct {
  knhk_op_t op;            // Operation type
  uint64_t s, p, o, k;      // Subject, predicate, object, threshold
  
  // For CONSTRUCT8 only: preallocated output spans (8 rows max)
  uint64_t *out_S;          // may be NULL for non-CONSTRUCT8
  uint64_t *out_P;
  uint64_t *out_O;
  uint64_t out_mask;        // per-lane bitmask result (returned by μ)
  
  // Legacy SELECT support (cold path only, not in hot v1.0)
  uint64_t *select_out;     // SELECT output buffer
  size_t select_capacity;
} knhk_hook_ir_t;
```

### knhk_receipt_t
```c
typedef struct {
  uint32_t ticks;    // ≤ 8
  uint32_t lanes;    // SIMD width used
  uint64_t span_id;  // OTEL-compatible id
  uint64_t a_hash;   // hash(A) = hash(μ(O)) fragment
} knhk_receipt_t;
```

## Execution Flow

### Basic Query Flow
1. **RDF Loading**: Parse RDF/Turtle files → SoA arrays
2. **Predicate Run Detection**: Group triples by predicate (len ≤ 8)
3. **Query Compilation**: SPARQL → Hook IR
4. **Path Selection**: Hot path vs warm path vs cold path routing
   - Hot path: Simple ASK, data size ≤8
   - Warm path: SPARQL queries, data size ≤10K
   - Cold path: Complex queries, SHACL, reasoning
5. **Evaluation**: Branchless SIMD execution (hot) or SPARQL engine (warm/cold)
6. **Result Return**: Boolean, count, or solution set result
7. **Caching**: Query results cached for warm path (epoch-based invalidation)

### Enterprise Pipeline Flow (v0.3.0)
1. **Connect**: Register connectors (Kafka, Salesforce, etc.)
2. **Ingest**: Poll connectors → Raw triples
3. **Transform**: Validate against Σ schema → Typed triples (IRI → u64)
4. **Load**: Group by predicate → SoA arrays (64-byte aligned)
5. **Reflex**: Execute hooks (μ) → Actions (A) + Receipts
6. **Emit**: Write receipts to lockchain → Send actions to downstream APIs
7. **Provenance**: hash(A) = hash(μ(O)) verified via receipts

## Hot Path Requirements (v0.5.0)

**Hot Path Operations** (≤8 ticks, ≤2ns):
- ASK operations (ASK_SP, ASK_SPO, ASK_OP) - Existence checks
- COUNT operations (COUNT_SP_GE/LE/EQ, COUNT_OP variants) - Cardinality validation
- COMPARE operations (COMPARE_O_EQ/GT/LT/GE/LE) - Value comparisons
- VALIDATION operations (UNIQUE_SP, VALIDATE_DATATYPE_SP/SPO) - Property validation

**Warm Path Operations** (≤500ms):
- CONSTRUCT8 operations (fixed-template emit) - Moved from hot path in v0.5.0

**Path Selection**:
- Hot path: Query operations that fit ≤8 tick budget
- Warm path: Emit operations (CONSTRUCT8) that exceed 8-tick budget but complete in <500ms
- Cold path: Complex queries requiring full SPARQL engine

**Requirements**:
- Predicate run size ≤8 elements (guard constraint enforced)
- Simple operations (ASK, COUNT, triple match)
- Data hot in L1 cache
- Single predicate queries
- Branchless operations (constant-time execution)
- ≤8 ticks (Chatman Constant: 2ns = 8 ticks) for hot path
- ≤500ms (p95) for warm path

## Path Selection Logic

KNHK automatically routes queries to the appropriate execution path based on query complexity and data size:

### Hot Path Selection
- Simple ASK queries (no FILTER, OPTIONAL, UNION)
- COUNT queries (≤8 elements)
- Data size ≤8 triples
- Single predicate queries

### Warm Path Selection (oxigraph)
- SPARQL SELECT, ASK, CONSTRUCT, DESCRIBE queries
- Data size ≤10K triples
- Basic SPARQL features (FILTER, OPTIONAL, UNION)
- No UPDATE queries
- No SHACL validation
- No OWL reasoning

### Cold Path Selection (unrdf)
- Complex queries exceeding warm path constraints
- UPDATE queries
- SHACL validation
- OWL reasoning
- Data size >10K triples
- Very complex property paths or UNION patterns

## Warm Path Integration (oxigraph)

The warm path uses [oxigraph](https://github.com/oxigraph/oxigraph), a Rust-native RDF store and SPARQL query engine, providing:

- **Performance**: 10-14x faster than unrdf FFI (5-50ms vs 150-700ms)
- **Caching**: LRU cache for query results (1000 entries)
- **Query Plan Caching**: Parsed SPARQL queries cached to reduce parsing overhead
- **Epoch-based Invalidation**: Cache automatically invalidated when data changes
- **OTEL Integration**: Query latency, cache hit rate, and query count metrics
- **Thread-safe**: Arc-based shared store for concurrent access
- **Hot Path Integration**: Automatic routing to C hot path (≤2ns) for simple queries

### Hot Path Routing

Simple queries are automatically routed to C hot path functions for ≤2ns execution:

- **ASK queries**: Routed to `knhk_eval_bool` C function
- **COUNT queries**: Routed to C hot path COUNT operations
- **Validation**: Tick budget verified (≤8 ticks) with automatic fallback
- **Fallback**: Seamless fallback to warm path if hot path fails

### Test Coverage

Comprehensive test suite with 83 Chicago TDD tests:

- **Path Selector**: 100% coverage
- **Query Module**: ~90% coverage  
- **Graph Module**: ~80% coverage
- **Executor**: ~80% coverage
- **Overall**: ~80%+ coverage

See [Testing Documentation](testing.md) for details.

### Performance Characteristics
- **Query Execution**: 5-50ms (depending on data size and query complexity)
- **Cache Hit**: 2-6μs (100x faster than cold execution)
- **Target**: ≤500ms for warm path queries
- **Cache Hit Rate**: Typically 60-80% for repeated queries

## Cold Path Fallback

Queries that exceed hot path or warm path constraints automatically fall back to unrdf (cold path) for full SPARQL engine execution.

## Production Infrastructure (v0.4.0)

**v0.4.0 Status**: Production-ready critical path features complete. See [v0.4.0 Status](v0.4.0-status.md) for complete details.

**v0.5.0 Updates**:
- ✅ CONSTRUCT8 moved to warm path (≤500ms budget)
- ✅ Warm path engine implemented (`knhk-warm` crate)
- ✅ Clear separation between hot path (query) and warm path (emit)
- ✅ Warm path metrics and observability

**Known Limitations**:
- ⚠️ Configuration management incomplete - Deferred to v0.5.0 Phase 2
- ⚠️ CLI documentation pending - Deferred to v0.5.0 Phase 3
- ⚠️ Examples directory missing - Deferred to v0.5.0 Phase 3

### Connector Framework
- Real library integrations (rdkafka, reqwest)
- Circuit breaker pattern for resilience
- Health checking and metrics
- Guard validation (max_run_len ≤ 8, max_batch_size, max_lag_ms)

### ETL Pipeline
- Production-ready stages (Ingest, Transform, Load, Reflex, Emit)
- Schema validation (O ⊨ Σ)
- Invariant checking (preserve(Q))
- Receipt generation and merging (⊕)

### Erlang Reflexive Control
- Schema registry (knhk_sigma)
- Invariant management (knhk_q)
- Delta ingestion (knhk_ingest)
- Lockchain (knhk_lockchain) with Merkle linking

### Observability
- OTEL span ID generation (not placeholders)
- Metrics collection
- Tracing support
- Receipt provenance tracking

## Modular Design Benefits

- **Maintainability**: Clear separation of concerns
- **Testability**: Focused test suites for each component
- **Performance**: Hot path isolated in inline headers
- **Extensibility**: Easy to add new operations or test suites
- **Code Size**: Reduced compilation unit sizes improve build times

