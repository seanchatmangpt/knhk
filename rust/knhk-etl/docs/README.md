# knhk-etl Documentation

ETL (Extract, Transform, Load) pipeline with reflexive control.

## Overview

The `knhk-etl` crate provides:
- Pipeline stages (Ingest, Transform, Load, Reflex, Emit)
- Path selection logic (hot/warm/cold routing)
- Schema validation (O ⊨ Σ)
- Invariant checking (preserve(Q))
- Receipt generation and merging

## Architecture

- **Pipeline**: Multi-stage ETL pipeline
- **Path Selector**: Routes queries to optimal execution path
- **Connectors**: Data source integration
- **Lockchain**: Merkle-linked receipt storage

## Key Features

- **Path Selection**: Automatic routing based on query complexity
- **Guard Validation**: Enforces constraints (max_run_len ≤ 8)
- **Schema Validation**: Validates observations against schema
- **Invariant Preservation**: Ensures Q constraints are preserved

## Path Selection

The path selector (`path_selector.rs`) routes queries:
- **Hot Path**: Simple ASK queries, data size ≤8
- **Warm Path**: SPARQL queries, data size ≤10K
- **Cold Path**: Complex queries, SHACL, reasoning

## Related Documentation

- [Architecture](../../../docs/architecture.md) - System architecture
- [Integration](../../../docs/integration.md) - Integration guide

