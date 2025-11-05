# Architecture Overview

KNHK implements a three-tier architecture optimized for ≤8 tick performance on critical path operations:

1. **Hot Path (C)** - ≤8 tick operations using SIMD
2. **Warm Path (Rust)** - Safe abstractions over hot path
3. **Cold Path (Erlang)** - Complex queries and validation

## System Overview

KNHK (v0.4.0) implements a multi-tier architecture with production-ready infrastructure:

1. **Hot Path Engine** (C) - ≤8 tick query execution
2. **Connector Framework** (Rust) - Enterprise data source integration
3. **ETL Pipeline** (Rust) - Ingest → Transform → Load → Reflex → Emit
4. **Reflexive Control Layer** (Erlang) - Schema, invariants, receipts, routing
5. **Observability** (OTEL) - Metrics, tracing, span generation

Queries route to either hot path (≤8 ticks) or cold path (full SPARQL engine) based on complexity and data characteristics.

## Key Components

- **ETL Pipeline**: Ingest → Transform → Load → Reflex → Emit
- **Connectors**: Kafka, Salesforce (with circuit breaker pattern)
- **Lockchain**: Merkle-linked provenance storage (URDNA2015 + SHA-256)
- **OTEL Integration**: Spans, metrics, traces
- **CLI Tool**: 13 command modules, 20+ commands

## See Also

- [Three-Tier Architecture](three-tier.md) - Detailed architecture breakdown
- [Key Components](components.md) - Component descriptions
- [Data Flow](data-flow.md) - Execution flow

