# knhk-warm

Warm path query engine with SPARQL support for the KNHK framework.

## Features

- SPARQL query execution via Oxigraph
- LRU caching with ahash for fast lookups
- Integration with hot path for optimized queries
- Optional OpenTelemetry instrumentation
- Memory-efficient query processing

## Usage

```rust
use knhk_warm::{WarmPathEngine, Query};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = WarmPathEngine::new()?;

    let query = Query::parse("SELECT ?s ?p ?o WHERE { ?s ?p ?o }")?;
    let results = engine.execute(query)?;

    Ok(())
}
```

## Features

Enable OpenTelemetry instrumentation:

```toml
knhk-warm = { version = "1.0.0", features = ["otel"] }
```

## Performance

- Query caching for repeated queries
- Efficient memory usage with LRU eviction
- Integration with hot path for sub-8-tick operations

## License

Licensed under MIT license.
