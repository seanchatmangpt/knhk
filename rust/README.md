# KNHK Rust Workspace

This directory contains the Rust implementation of KNHK (Knowledge Graph Hot Path), organized as a Cargo workspace.

## Workspace Structure

### Core Crates

- **knhk-hot** (v1.0.0) - Hot path FFI layer (links to C library)
- **knhk-etl** (v0.1.0) - Extract, Transform, Load pipeline with beat scheduling
- **knhk-warm** (v0.1.0) - Warm path query optimization with caching
- **knhk-aot** (v0.1.0) - Ahead-of-time validation and policy enforcement

### Infrastructure Crates

- **knhk-otel** (v0.1.0) - OpenTelemetry instrumentation (schema-first validation)
- **knhk-connectors** (v0.1.0) - External system connectors (Kafka, Salesforce)
- **knhk-lockchain** (v0.1.0) - Immutable event log with Git-style tracking
- **knhk-unrdf** (v0.1.0) - Universal RDF hooks engine with Tera templates
- **knhk-validation** (v0.1.0) - Policy engine and validation framework
- **knhk-config** (v0.1.0) - Configuration management
- **knhk-sidecar** (v0.5.0) - gRPC sidecar service with batching/retries
- **knhk-cli** (v0.1.0) - Command-line interface
- **knhk-integration-tests** (v0.1.0) - End-to-end integration tests

## Prerequisites

**CRITICAL**: The C hot path layer must be built first before building the Rust workspace:

```bash
# From repository root
cd c && make && cd ..
```

This creates `c/libknhk.a` which is required by `knhk-hot`.

## Building

### Build Everything

```bash
# From this directory
cargo build --workspace

# Release build (optimized)
cargo build --workspace --release
```

### Build Specific Crates

```bash
# By crate name
cargo build -p knhk-etl
cargo build -p knhk-sidecar --release

# With features
cargo build -p knhk-unrdf --features native
cargo build -p knhk-cli --features otel
```

## Testing

### Run All Tests

```bash
cargo test --workspace
```

### Run Tests for Specific Crates

```bash
# Single crate
cargo test -p knhk-etl

# With features
cargo test -p knhk-unrdf --features native

# Specific test file
cargo test -p knhk-etl --test chicago_tdd_beat_scheduler
cargo test -p knhk-etl --test chicago_tdd_pipeline
```

### Chicago TDD Test Suite

```bash
# All Chicago TDD tests for ETL
cargo test -p knhk-etl --test chicago_tdd_beat_scheduler
cargo test -p knhk-etl --test chicago_tdd_pipeline
cargo test -p knhk-etl --test chicago_tdd_ring_conversion
cargo test -p knhk-etl --test chicago_tdd_hook_registry
cargo test -p knhk-etl --test chicago_tdd_runtime_class
```

## Code Quality

### Linting

```bash
# Check all crates for warnings
cargo clippy --workspace -- -D warnings

# Check specific crate
cargo clippy -p knhk-etl -- -D warnings
```

### Formatting

```bash
# Check formatting
cargo fmt --all --check

# Auto-format all code
cargo fmt --all
```

## Benchmarks

```bash
# Run benchmarks (from specific crate)
cd knhk-unrdf && cargo bench
cd knhk-warm && cargo bench
```

## Workspace Benefits

- **Unified dependency management**: Common dependencies shared via `[workspace.dependencies]`
- **Consistent versioning**: All crates use workspace-level version declarations
- **Faster builds**: Cargo reuses compiled dependencies across crates
- **Atomic operations**: `--workspace` flag applies commands to all crates
- **Better IDE support**: Single workspace for project-wide navigation

## Dependency Graph

```
knhk-cli → knhk-hot, knhk-warm, knhk-etl, knhk-connectors, knhk-lockchain, knhk-otel, knhk-config
knhk-sidecar → knhk-hot, knhk-etl, knhk-connectors, knhk-otel, knhk-config
knhk-validation → knhk-hot, knhk-connectors, knhk-lockchain, knhk-otel
knhk-etl → knhk-hot, knhk-connectors, knhk-lockchain, knhk-otel
knhk-warm → knhk-hot, knhk-etl, knhk-otel (optional), knhk-unrdf (optional)
knhk-aot → knhk-validation (optional)
knhk-hot → ../c/libknhk.a (C library)
```

## Troubleshooting

### "could not find native static library `knhk`"

The C library must be built first:

```bash
cd ../c && make && cd ../rust
cargo clean -p knhk-hot
cargo build --workspace
```

### Version conflicts

If you encounter dependency version conflicts:

```bash
cargo update
cargo build --workspace
```

### Disk space issues

Clean build artifacts:

```bash
cargo clean
cd ../c && make clean && cd ../rust
```

## OpenTelemetry Weaver Validation

**CRITICAL**: All KNHK functionality must be validated via OTel Weaver schemas:

```bash
# Validate schema definitions
weaver registry check -r registry/

# Validate runtime telemetry
weaver registry live-check --registry registry/
```

This is the ONLY source of truth for validation. Traditional test passes can be false positives.

## Contributing

When adding new crates:

1. Create crate directory under `rust/`
2. Add to `[workspace.members]` in `rust/Cargo.toml`
3. Use `[workspace.dependencies]` for common deps
4. Follow existing patterns for features and profiles

## License

MIT
