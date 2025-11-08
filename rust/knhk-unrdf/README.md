# knhk-unrdf

RDF utilities and helpers for the KNHK framework.

## Features

- Template-based RDF generation with Tera
- Oxigraph integration for native RDF processing
- BLAKE3 hashing for content addressing
- LRU caching for query results
- Parallel processing with Rayon

## Usage

```rust
use knhk_unrdf::{TemplateEngine, RdfProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = TemplateEngine::new()?;
    let rdf = engine.render_template("template.tera", &context)?;

    Ok(())
}
```

## Features

Enable native RDF support:

```toml
knhk-unrdf = { version = "1.0.0", features = ["native"] }
```

## License

Licensed under MIT license.
