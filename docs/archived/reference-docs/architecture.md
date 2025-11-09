# KNHK Architecture - 80/20 Guide

**Version**: 1.0  
**Status**: Production-Ready  
**Last Updated**: 2025-01-XX

---

## Overview

KNHK implements a multi-tier architecture with production-ready infrastructure, grounded in formal mathematical laws that enable safe parallelism, cryptographic verification, and deterministic execution.

**Key Features**:
- ✅ Hot Path Engine (C) - ≤8 tick query execution
- ✅ Warm Path Engine (Rust) - ≤500ms emit operations
- ✅ Connector Framework (Rust) - Enterprise data source integration
- ✅ ETL Pipeline (Rust) - Ingest → Transform → Load → Reflex → Emit
- ✅ Reflexive Control Layer (Erlang) - Schema, invariants, receipts, routing
- ✅ Observability (OTEL) - Metrics, tracing, span generation

---

## Quick Start (80% Use Case)

### System Overview

KNHK (v0.5.0) implements a multi-tier architecture:

1. **Hot Path Engine** (C) - ≤8 tick query execution (ASK, COUNT, COMPARE, VALIDATE)
2. **Warm Path Engine** (Rust) - ≤500ms emit operations (CONSTRUCT8)
3. **Connector Framework** (Rust) - Enterprise data source integration
4. **ETL Pipeline** (Rust) - Ingest → Transform → Load → Reflex → Emit
5. **Reflexive Control Layer** (Erlang) - Schema, invariants, receipts, routing
6. **Observability** (OTEL) - Metrics, tracing, span generation

Queries route to hot path (≤8 ticks), warm path (≤500ms), or cold path (full SPARQL engine) based on operation type and complexity.

---

## Core Architecture (80% Value)

### Three-Tier Path Architecture

**Hot Path** (≤8 ticks):
- ASK operations (existence checks)
- COUNT operations (cardinality)
- COMPARE operations (value comparison)
- VALIDATE operations (datatype validation)
- Branchless C implementation
- SIMD-optimized (4 elements per instruction)

**Warm Path** (≤500ms):
- CONSTRUCT8 operations (triple construction)
- Batch operations
- Rust implementation

**Cold Path** (full SPARQL):
- Complex queries
- Multi-predicate joins
- Full SPARQL engine

### Data Layer (SoA Layout)

Triples are stored in Structure-of-Arrays format:

```c
typedef struct {
    uint64_t s[8];  // Subject array (64-byte aligned)
    uint64_t p[8];  // Predicate array (64-byte aligned)
    uint64_t o[8];  // Object array (64-byte aligned)
} SoAArrays;
```

**Benefits**:
- Single cacheline loads (64-byte alignment)
- SIMD-friendly access patterns
- Zero-copy operations

### Branchless C Engine

**Key Design**: Zero branches in hot path operations.

**Implementation**:
- Function pointer table dispatch (O(1) lookup)
- Mask-based conditionals (no `if` statements)
- Branchless comparison operations
- Zero branch mispredicts

**Performance**:
- ≤8 ticks for all hot path operations
- ≤2ns per operation (Chatman Constant)
- Zero mispredicts

---

## Formal Mathematical Foundations

KNHK's architecture is grounded in formal mathematical laws:

**Key Formal Properties**:
- **Idempotence** (μ∘μ = μ): Safe retry semantics without coordination
- **Shard Distributivity** (μ(O ⊔ Δ) = μ(O) ⊔ μ(Δ)): Parallel evaluation equivalence
- **Sheaf Property** (glue(Cover(O)) = Γ(O)): Local-to-global consistency
- **Provenance** (hash(A) = hash(μ(O))): Cryptographic verification
- **Epoch Containment** (μ ⊂ τ): Time-bounded execution

See [Formal Mathematical Foundations](formal-foundations.md) for complete formal treatment.

---

## 8-Beat Epoch System

**Overview**: 8-beat epoch system for time-bounded execution.

**Components**:
- **Delta Ring**: Per-tick delta storage (8 slots)
- **Assertion Ring**: Per-tick assertion storage (8 slots)
- **Beat Scheduler**: Tick-based execution coordination
- **Fiber Executor**: C hot path execution

**Integration**:
- C/Rust integration complete
- Ring buffer FFI implemented
- Fiber execution integrated
- Receipt conversion module complete

See [8-Beat Integration Complete](8BEAT-INTEGRATION-COMPLETE.md) for details.

---

## RDF Workflow Architecture

**Three-Tier RDF Store Architecture**:
- `spec_rdf_store`: Workflow specifications (immutable, shared)
- `pattern_metadata_store`: 43 pattern metadata (immutable, shared)
- `case_rdf_stores`: Runtime state (mutable, per-case)

**Key Capabilities**:
- RDF-native execution (load `.ttl` workflows directly)
- SPARQL query capabilities
- SHACL soundness validation
- OTEL telemetry-first design

See [RDF Workflow Architecture](architecture/README-RDF-WORKFLOW-ARCHITECTURE.md) for details.

---

## Code Organization

### C Header Structure

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

### C Source Structure

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

### Rust Crate Structure

```
rust/
├── knhk-hot/            # Hot path operations (C FFI)
├── knhk-warm/           # Warm path operations
├── knhk-etl/            # ETL pipeline
├── knhk-connectors/     # Connector framework
├── knhk-lockchain/      # Provenance system
├── knhk-otel/           # Observability
└── knhk-workflow-engine/ # Workflow engine
```

---

## Performance Architecture

### Hot Path Performance

**Critical Design Decision**: C hot path contains **zero timing code**. All timing measurements are performed externally by the Rust framework.

**Performance Targets**:
- Hot path operations: ≤8 ticks (Chatman Constant)
- ≤2ns per operation
- Zero branch mispredicts

**Current Performance**:
- ASK operations: ~1.0-1.1 ns ✅
- COUNT operations: ~1.0-1.1 ns ✅
- COMPARE operations: ~0.9 ns ✅
- VALIDATE operations: ~1.5 ns ✅
- CONSTRUCT8: ~41-83 ticks ⚠️ (exceeds 8-tick budget)

### Optimization Strategies

1. **Structure-of-Arrays**: Separate S, P, O arrays for SIMD access
2. **64-byte alignment**: Single cacheline loads
3. **Fully unrolled SIMD**: Direct instruction sequence for NROWS=8
4. **Branchless operations**: Bitwise masks instead of conditionals
5. **Warm L1 cache**: Data assumed hot during measurement

---

## Integration Points

### C/Rust Integration

**FFI Interface**:
- Ring buffer operations (DeltaRing, AssertionRing)
- Fiber execution (FiberExecutor)
- Receipt conversion (C ↔ Rust)

**Integration Status**:
- ✅ Ring buffer FFI complete
- ✅ Fiber execution integrated
- ✅ Receipt conversion module complete

### Workflow Engine Integration

**RDF Workflow Execution**:
- Load workflows from Turtle files
- Parse with Oxigraph
- Execute with WorkflowEngine
- Persist state with Sled

**Integration Status**:
- ✅ RDF workflow loading complete
- ✅ Pattern execution complete
- ✅ State persistence complete

---

## Production Readiness

### ✅ Ready for Production

- **Hot Path Engine**: Fully functional (≤8 ticks)
- **Warm Path Engine**: Fully functional (≤500ms)
- **ETL Pipeline**: Complete (Ingest → Transform → Load → Reflex → Emit)
- **Connector Framework**: Enterprise data source integration
- **Observability**: OTEL integration complete
- **Provenance**: Lockchain integration complete

### ⚠️ Partial Production Readiness

- **CONSTRUCT8**: Exceeds 8-tick budget (41-83 ticks)
- **Cold Path**: Full SPARQL engine (not optimized)

---

## Troubleshooting

### Hot Path Performance Issues

**Problem**: Operations exceed 8-tick budget.

**Solution**: 
- Check for branches in hot path (should be zero)
- Verify SIMD alignment (64-byte alignment)
- Ensure warm L1 cache (data should be hot)

### C/Rust FFI Issues

**Problem**: FFI calls fail or crash.

**Solution**:
- Verify ring buffer initialization
- Check receipt conversion (C ↔ Rust)
- Ensure proper memory alignment

### Workflow Execution Issues

**Problem**: Workflows fail to execute.

**Solution**:
- Verify RDF parsing (Turtle format)
- Check pattern registry (43 patterns)
- Verify state persistence (Sled)

---

## Additional Resources

### Related Consolidated Guides
- **[Performance Guide](PERFORMANCE.md)** - Performance optimization and hot path details
- **[API Guide](API.md)** - API interfaces and usage patterns
- **[Integration Guide](INTEGRATION.md)** - Integration patterns and connectors
- **[Workflow Engine Guide](WORKFLOW_ENGINE.md)** - Workflow execution and patterns
- **[Production Guide](PRODUCTION.md)** - Production deployment and monitoring

### Detailed Documentation
- **Formal Foundations**: [Formal Mathematical Foundations](formal-foundations.md)
- **8-Beat System**: [8-Beat Integration Complete](8BEAT-INTEGRATION-COMPLETE.md)
- **Branchless Engine**: [Branchless C Engine Implementation](BRANCHLESS_C_ENGINE_IMPLEMENTATION.md)
- **RDF Workflow**: [RDF Workflow Architecture](architecture/README-RDF-WORKFLOW-ARCHITECTURE.md)
- **Performance**: [Performance Guide](PERFORMANCE.md)

### Architecture Decision Records
- **ADR-002**: [Turtle vs YAWL XML](architecture/ADR/ADR-002-turtle-vs-yawl-xml.md)
- **ADR-001**: [Interface B Work Item Lifecycle](architecture/ADR-001-interface-b-work-item-lifecycle.md)

### Code Examples
- **C Hot Path**: `c/src/simd/`
- **Rust ETL**: `rust/knhk-etl/src/`
- **Workflow Engine**: `rust/knhk-workflow-engine/src/`

---

## License

MIT License

---

**Last Updated**: 2025-01-XX  
**Version**: 1.0  
**Status**: Production-Ready
