# KNHK - Knowledge Hook System

**Version**: 0.4.0  
**Status**: Production Ready  
**Architecture**: 8-Tick Hot Path Knowledge Graph Query System

## Overview

KNHK (Knowledge Hook System) is a high-performance knowledge graph query system designed for enterprise-scale RDF data processing. The system achieves **≤8 tick performance** (Chatman Constant) on critical path operations through SIMD-optimized C hot path, safe Rust warm path, and Erlang cold path architecture.

## Key Features

### Core Features (80% Value)

- **Hot Path Operations** - 19 operations achieving ≤8 ticks  
- **ETL Pipeline** - Complete pipeline with guard enforcement  
- **Connector Framework** - Kafka, Salesforce with circuit breakers  
- **Lockchain** - Merkle-linked receipts with URDNA2015 + SHA-256  
- **CLI Tool** - Production-ready command-line interface  
- **OTEL Integration** - Observability and metrics  
- **Guard Constraints** - max_run_len ≤ 8, τ ≤ 8 enforced  

### Performance

- **Hot Path**: ≤8 ticks (Chatman Constant: 2ns = 8 ticks)
- **Critical Path**: Separated from receipt generation overhead
- **SoA Layout**: 64-byte alignment for SIMD operations
- **Branchless**: Constant-time execution on hot path

## Three-Tier Architecture

1. **Hot Path (C)** - ≤8 tick operations using SIMD
   - Structure-of-Arrays (SoA) layout
   - 64-byte alignment for SIMD
   - Branchless operations
   - 19 query operations (ASK, COUNT, COMPARE, SELECT, CONSTRUCT8)

2. **Warm Path (Rust)** - Safe abstractions over hot path
   - ETL Pipeline (Ingest → Transform → Load → Reflex → Emit)
   - Connector framework (Kafka, Salesforce)
   - Lockchain integration (Merkle-linked receipts)
   - OTEL observability

3. **Cold Path (Erlang)** - Complex queries and validation
   - SPARQL query execution
   - SHACL validation
   - Schema registry (knhk_sigma)
   - Invariant registry (knhk_q)

## Quick Start

### Build

```bash
# Build C library
make lib

# Build CLI
cd rust/knhk-cli
cargo build --release

# Run tests
make test
```

### CLI Usage

```bash
# Initialize system
knhk boot init schema.ttl invariants.sparql

# Register connector
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples

# Run pipeline
knhk pipeline run --connectors kafka-prod
```

## Documentation

This book provides comprehensive documentation for KNHK:

- **[Quick Start](getting-started.md)** - Get up and running in 5 minutes
- **[CLI Guide](cli/README.md)** - Complete command reference
- **[Architecture](architecture/README.md)** - System architecture overview
- **[API Reference](api/README.md)** - API documentation
- **[Integration Guide](integration.md)** - Integration examples

## 80/20 Philosophy

KNHK follows the **80/20 principle**: Focus on the critical 20% that delivers 80% of value.

### Critical Path (80% Value)
- Basic triple pattern matching (ASK, SELECT on single predicate)
- Simple property constraints (minCount, maxCount, unique)
- Datatype validation (basic type checks)
- Existence checks (ASK_SP, ASK_SPO)
- Count aggregations (COUNT_SP_GE, COUNT_SP_EQ)

### Deferred (20% Edge Cases)
- Complex JOINs across multiple predicates
- OPTIONAL patterns
- Transitive property paths
- Full OWL inference
- Complex SPARQL queries (multi-predicate, nested)

## Project Structure

```
vendors/knhk/
├── src/              # C hot path implementation
├── include/          # C headers
├── rust/             # Rust warm path crates
│   ├── knhk-cli/    # CLI tool
│   ├── knhk-etl/    # ETL pipeline
│   ├── knhk-connectors/  # Connector framework
│   ├── knhk-lockchain/   # Provenance lockchain
│   └── knhk-otel/   # OTEL integration
├── erlang/           # Erlang cold path
├── tests/            # Test suite
├── docs/             # Documentation
└── Makefile          # Build system
```

## Release Status

**Current Version**: v0.4.0  
**Release Date**: December 2024  
**Status**: Production Ready

See [Release Notes](../RELEASE_NOTES_v0.4.0.md) for full release details.

---

**80/20 Philosophy**: Focus on the critical 20% that delivers 80% of value.

