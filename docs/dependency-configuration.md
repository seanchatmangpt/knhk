# Dependency Configuration Guide

## Overview

`knhk-etl` uses standard library dependencies that are always enabled. All features are always available - there are no optional features or feature gates.

## Dependencies

All dependencies are always enabled:

- **knhk-hot**: Hot path operations (C FFI)
- **knhk-lockchain**: Merkle-linked receipt storage
- **knhk-otel**: OpenTelemetry observability integration
- **reqwest**: HTTP client for downstream APIs
- **rdkafka**: Kafka client for message streaming
- **oxigraph**: RDF/Turtle parsing
- **serde_json**: JSON serialization

## Building

```bash
# Build library
cargo build --release

# Build with tests
cargo test

# Check compilation
cargo check
```

## C Library Dependency (knhk-hot)

**Purpose**: Hot path operations (C FFI)

**Build Requirements**:
1. Build the C library: `cd c && make libknhk.a`
2. Ensure `libknhk.a` exists in `c/` directory
3. The build script (`build.rs`) will automatically link against it

**Configuration**: The `build.rs` file sets the library search path to `../../c` to locate `libknhk.a`.

## Troubleshooting

### C Library Not Found (knhk-hot)

If you see errors about `libknhk.a` not found:

1. **Build the C library**:
   ```bash
   cd c
   make libknhk.a
   ```

2. **Verify library exists**:
   ```bash
   ls -la c/libknhk.a
   ```

3. **Check build.rs**: The build script should set the search path to `../../c`

### Compilation Errors

If you see compilation errors:

1. **Clean and rebuild**:
   ```bash
   cargo clean
   cargo build
   ```

2. **Check dependencies**: All dependencies should be listed in `Cargo.toml` without `optional = true`

3. **Verify C library**: Ensure `libknhk.a` is built and accessible

