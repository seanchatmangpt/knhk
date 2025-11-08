# knhk-etl

ETL (Extract, Transform, Load) pipeline with reflexive control for KNHK.

## Overview

`knhk-etl` implements a complete ETL pipeline with five stages: Ingest → Transform → Load → Reflex → Emit. It provides automatic path selection (hot/warm/cold routing), guard validation, schema validation, and receipt generation.

## Quick Start

```rust
use knhk_etl::{Pipeline, PipelineError};

// Create pipeline with connectors, schema, and downstream endpoints
let pipeline = Pipeline::new(
    vec!["kafka_connector".to_string()],
    "urn:knhk:schema:test".to_string(),
    true,  // enable lockchain
    vec!["https://webhook.example.com".to_string()],
);

// Execute full pipeline (Ingest → Transform → Load → Reflex → Emit)
match pipeline.execute() {
    Ok(emit_result) => {
        println!("Pipeline executed successfully");
        println!("Actions emitted: {}", emit_result.actions_sent);
        println!("Receipts written: {}", emit_result.receipts_written);
    }
    Err(e) => eprintln!("Pipeline error: {}", e),
}
```

### Manual Stage Execution

For fine-grained control, you can access public stages:

```rust
use knhk_etl::{IngestStage, TransformStage, LoadStage, ReflexStage, EmitStage};

// Create stages individually
let ingest = IngestStage::new(vec!["connector".to_string()], "rdf/turtle".to_string());
let transform = TransformStage::new("urn:knhk:schema:test".to_string(), true);
let load = LoadStage::new();
let reflex = ReflexStage::new();
let emit = EmitStage::new(true, vec![]);

// Parse RDF content
let rdf_content = r#"
    <http://example.org/alice> <http://example.org/name> "Alice" .
"#;
let triples = ingest.parse_rdf_turtle(rdf_content)?;

// Execute stages sequentially
let ingest_result = ingest.ingest()?;
let transform_result = transform.transform(ingest_result)?;
let load_result = load.load(transform_result)?;
let reflex_result = reflex.reflex(load_result)?;
let emit_result = emit.emit(reflex_result)?;
```

## Key Features

- **Pipeline Stages**: Ingest, Transform, Load, Reflex, Emit
- **Ingester Pattern**: Unified interface for multiple input sources (file, stdin, memory, streaming)
  - Inspired by OpenTelemetry Weaver's ingester architecture
  - See `ingester` module for details
- **Path Selection**: Automatic routing based on query complexity
- **Guard Validation**: Enforces max_run_len ≤ 8 (Chatman Constant)
- **Schema Validation**: Validates observations against schema (O ⊨ Σ)
- **Receipt Generation**: Creates provenance receipts
- **Hook Orchestration**: Pattern-based hook execution (sequential, parallel, conditional, retry)
  - See `hook_orchestration` module for details
  - Integrates with `knhk-patterns` for workflow pattern support

## Hook Orchestration

The Reflex stage supports pattern-based hook orchestration using workflow patterns from `knhk-patterns`:

```rust
use knhk_etl::hook_orchestration::{HookOrchestrator, HookExecutionContext, HookExecutionPattern};
use knhk_etl::hook_registry::HookRegistry;

let registry = HookRegistry::new();
let load_result = pipeline.execute_to_load()?;

// Create execution context
let context = HookExecutionContext::from_load_result(registry, load_result, 8);

// Execute hooks in parallel
let pattern = HookExecutionPattern::Parallel(vec![pred1, pred2]);
let orchestrator = HookOrchestrator::new();
let results = orchestrator.execute_with_pattern(&context, pattern)?;

// Execute hooks conditionally
let choices = vec![
    (Box::new(|ctx| ctx.predicate_runs.len() > 1) as Box<dyn Fn(_) -> _ + Send + Sync>, pred1),
    (Box::new(|_| true) as Box<dyn Fn(_) -> _ + Send + Sync>, pred2),
];
let pattern = HookExecutionPattern::Choice(choices);
let results = orchestrator.execute_with_pattern(&context, pattern)?;

// Execute hooks with retry
let pattern = HookExecutionPattern::Retry {
    predicate: pred1,
    should_retry: Box::new(|receipt| receipt.ticks == 0),
    max_attempts: 3,
};
let results = orchestrator.execute_with_pattern(&context, pattern)?;
```

**Pattern Types:**
- **Sequence**: Execute hooks sequentially
- **Parallel**: Execute hooks concurrently (requires `parallel` feature)
- **Choice**: Conditional routing based on execution context
- **Retry**: Retry logic with exponential backoff

See [knhk-patterns HOOK_INTEGRATION.md](../knhk-patterns/HOOK_INTEGRATION.md) for comprehensive guide.

## Dependencies

### Required Dependencies
- `hashbrown` - Hash maps (no_std compatible)
- `hex` - Hex encoding

### Optional Dependencies (enabled via `std` feature)
- `oxigraph` - RDF parsing and SPARQL query engine
- `knhk-hot` - Hot path operations (requires C library `libknhk.a`)
- `knhk-lockchain` - Receipt storage (Merkle-linked chain)
- `knhk-otel` - OpenTelemetry observability
- `reqwest` - HTTP client for downstream APIs
- `serde_json` - JSON serialization

## Feature Flags

The crate supports conditional compilation via feature flags:

- **`std`** (default: disabled) - Enables standard library support and optional dependencies
  - Enables: `oxigraph`, `knhk-hot`, `knhk-lockchain`, `knhk-otel`, `reqwest`, `serde_json`
- **`kafka`** - Enables Kafka connector support (requires `std`)
- **`grpc`** - Enables gRPC support (requires `std`)

### Building with Features

```bash
# Build with std features (enables all optional dependencies)
cargo build --features std

# Build with specific features
cargo build --features std,kafka

# Build without std (no_std mode)
cargo build --no-default-features
```

### Build Requirements

When building with `knhk-hot` feature:
1. Build the C library first: `cd c && make lib`
2. Ensure `libknhk.a` exists in `c/` directory
3. The `knhk-hot` build script will link against it automatically

## Documentation

For detailed documentation, see [docs/README.md](docs/README.md).

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Integration](../../docs/integration.md) - Integration guide
- [Performance](../../docs/performance.md) - Performance guide
- [Weaver Integration](../../docs/WEAVER_INTEGRATION.md) - Weaver patterns integration (Ingester pattern)
- [Dependency Configuration](../../docs/dependency-configuration.md) - Feature flags and optional dependencies
