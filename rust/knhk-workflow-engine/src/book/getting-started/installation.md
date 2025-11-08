# Installation

Installation instructions for the KNHK Workflow Engine.

## Cargo Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
knhk-workflow-engine = "1.0.0"
```

## Features

The workflow engine supports optional features:

```toml
[dependencies]
knhk-workflow-engine = { version = "1.0.0", features = ["unrdf"] }
```

### Available Features

- `unrdf` - Enable unrdf integration (default)
- `rdf` - Enable RDF processing

## System Requirements

- Rust 1.70 or later
- Tokio runtime (included in dependencies)

## Building from Source

```bash
git clone https://github.com/yourusername/knhk.git
cd knhk/rust/knhk-workflow-engine
cargo build --release
```

## Verification

Verify installation:

```bash
cargo test --lib
```

## Next Steps

- [Quick Start](quick-start.md) - Get started quickly
- [Basic Concepts](basic-concepts.md) - Learn core concepts

