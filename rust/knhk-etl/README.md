# knhk-etl

ETL (Extract, Transform, Load) pipeline with reflexive control for KNHK.

## Overview

`knhk-etl` implements a production-ready ETL pipeline that processes RDF data through five stages: Ingest → Transform → Load → Reflex → Emit. The pipeline enforces schema validation, invariant preservation, and generates cryptographic receipts for provenance tracking.

## Quick Start

```rust
use knhk_etl::{Pipeline, PipelineStage};

// Create pipeline with connectors
let mut pipeline = Pipeline::new(
    vec!["kafka_connector".to_string()],
    "urn:knhk:schema:test".to_string(),
    true,  // enable schema validation
    vec!["https://webhook.example.com".to_string()],
);

// Execute pipeline stages
let rdf_content = r#"
    <http://example.org/alice> <http://example.org/name> "Alice" .
    <http://example.org/bob> <http://example.org/name> "Bob" .
"#;

// Ingest: Parse RDF Turtle
let ingest_result = pipeline.ingest.parse_rdf_turtle(rdf_content)?;

// Transform: Hash IRIs and validate schema
let transform_result = pipeline.transform.transform(ingest_result)?;

// Load: Build SoA arrays (max_run_len ≤ 8)
let load_result = pipeline.load.load(transform_result)?;

// Reflex: Validate schema (O ⊨ Σ) and invariants (preserve(Q))
let reflex_result = pipeline.reflex.reflex(load_result)?;

// Emit: Send actions to downstream systems
let emit_result = pipeline.emit.emit(reflex_result)?;
```

## Pipeline Stages

### 1. Ingest Stage
- Parses RDF Turtle format
- Validates syntax
- Resolves prefixes and base URIs
- Handles blank nodes and literals

### 2. Transform Stage
- Converts raw triples to typed triples
- Hashes IRIs to u64 identifiers
- Validates against schema (O ⊨ Σ)
- Type checking and conversion

### 3. Load Stage
- Groups triples by predicate
- Builds Structure-of-Arrays (SoA) layout
- Enforces guard constraints (max_run_len ≤ 8)
- Creates predicate runs for hot path

### 4. Reflex Stage
- Schema validation (O ⊨ Σ)
- Invariant checking (preserve(Q))
- Receipt generation (hash(A) = hash(μ(O)))
- Tick budget enforcement (≤8 ticks)

### 5. Emit Stage
- Sends actions to downstream systems
- Writes receipts to lockchain
- Webhook notifications
- Error handling and retries

## Key Features

- **Path Selection**: Automatic routing (hot/warm/cold) based on query complexity
- **Guard Validation**: Enforces max_run_len ≤ 8 (Chatman Constant)
- **Schema Validation**: Validates observations against schema (O ⊨ Σ)
- **Invariant Preservation**: Ensures Q constraints are preserved
- **Receipt Generation**: Cryptographic provenance receipts
- **Runtime Classes**: R1/W1/C1 SLO monitoring
- **Failure Actions**: Automatic recovery and retry logic

## Dependencies

- `rio_turtle` - RDF parsing
- `knhk-hot` (optional) - Hot path operations
- `knhk-lockchain` (optional) - Receipt storage
- `knhk-otel` (optional) - Observability

## Performance

- **Hot Path**: ≤8 ticks (≤2ns) for guard operations
- **Warm Path**: ≤500ms for SPARQL queries
- **Cold Path**: Unbounded for complex queries
- **Guard Constraints**: max_run_len ≤ 8 enforced at Load stage

## Related Documentation

- [Technical Documentation](docs/README.md) - Detailed API reference
- [Architecture](../../docs/architecture.md) - System architecture
- [Integration Guide](../../docs/integration.md) - Integration examples
- [Path Selector](../../docs/ggen-integration-guide.md) - Path selection details

