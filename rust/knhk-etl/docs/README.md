# knhk-etl Documentation

ETL (Extract, Transform, Load) pipeline with reflexive control.

## File Structure

```
rust/knhk-etl/
├── src/
│   ├── lib.rs              # Module exports and public API
│   ├── types.rs            # Pipeline types (PipelineStage, PipelineMetrics)
│   ├── error.rs            # PipelineError enum
│   ├── ingest.rs           # IngestStage - RDF parsing and ingestion
│   ├── transform.rs        # TransformStage - Type conversion and validation
│   ├── load.rs             # LoadStage - SoA array construction
│   ├── reflex.rs           # ReflexStage - Schema/invariant checking, receipt generation
│   ├── emit.rs             # EmitStage - Downstream emission
│   ├── pipeline.rs         # Pipeline orchestration
│   ├── integration.rs      # Integration layer (connectors, lockchain, OTEL)
│   └── path_selector.rs    # Path selection logic (hot/warm/cold routing)
└── Cargo.toml
```

## Core Components

### Pipeline Stages
- **IngestStage** (`ingest.rs`): Parses RDF (Turtle format), validates syntax
- **TransformStage** (`transform.rs`): Converts raw triples to typed triples
- **LoadStage** (`load.rs`): Builds SoA arrays with guard validation (max_run_len ≤ 8)
- **ReflexStage** (`reflex.rs`): Schema validation (O ⊨ Σ), invariant checking (preserve(Q))
- **EmitStage** (`emit.rs`): Emits results to downstream systems

### Path Selector (`path_selector.rs`)
- **QueryPath enum**: Hot, Warm, Cold
- **select_path()**: Routes queries based on complexity and data size
- **is_hot_path_query()**: Checks if query fits hot path constraints
- **is_warm_path_query()**: Checks if query fits warm path constraints

### Integration (`integration.rs`)
- Connects pipeline with connectors, lockchain, OTEL
- Warm path executor integration
- Unified result types

## Key Features

- **Path Selection**: Automatic routing based on query complexity
- **Guard Validation**: Enforces constraints (max_run_len ≤ 8)
- **Schema Validation**: Validates observations against schema (O ⊨ Σ)
- **Invariant Preservation**: Ensures Q constraints are preserved
- **Receipt Generation**: Creates provenance receipts

## Dependencies

- `oxigraph` - RDF parsing and SPARQL query engine (optional, via `std` feature)
- `knhk-hot` - Hot path operations
- `knhk-lockchain` - Receipt storage
- `knhk-otel` - Observability (optional)

## Usage

```rust
use knhk_etl::{Pipeline, IngestStage, TransformStage, LoadStage};

let pipeline = Pipeline::new();
// Execute pipeline stages...
```

## Related Documentation

- [Architecture](../../../docs/architecture.md) - System architecture
- [Integration](../../../docs/integration.md) - Integration guide
- [Path Selector](../../../docs/ggen-integration-guide.md) - Path selection details

