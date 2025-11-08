# Workspace Setup

Complete workspace setup guide for KNHK development.

## Overview

KNHK uses a Rust workspace with multiple crates:
- **knhk-etl**: ETL pipeline with 8-beat epoch integration
- **knhk-hot**: C FFI bindings for hot path
- **knhk-unrdf**: Rust-native hooks engine
- **knhk-sidecar**: gRPC proxy service
- **knhk-connectors**: Data source connectors
- **knhk-validation**: Schema validation
- **knhk-otel**: OpenTelemetry integration
- **knhk-lockchain**: Receipt storage
- **knhk-warm**: Warm path operations
- **knhk-aot**: Ahead-of-time compilation
- **knhk-cli**: Command-line interface
- **knhk-config**: Configuration management
- **knhk-patterns**: Code patterns

## Workspace Structure

```
rust/
├── Cargo.toml          # Workspace configuration
├── knhk-etl/           # ETL pipeline
├── knhk-hot/           # C FFI bindings
├── knhk-unrdf/         # Hooks engine
├── knhk-sidecar/       # gRPC proxy
├── knhk-connectors/    # Connectors
├── knhk-validation/    # Validation
├── knhk-otel/          # OTEL integration
├── knhk-lockchain/     # Receipt storage
├── knhk-warm/          # Warm path
├── knhk-aot/           # AOT compilation
├── knhk-cli/           # CLI
├── knhk-config/        # Configuration
└── knhk-patterns/     # Patterns
```

## Setup Steps

### 1. Clone Repository

```bash
git clone https://github.com/seanchatmangpt/knhk.git
cd knhk
```

### 2. Build C Layer

```bash
cd c && make && cd ..
```

### 3. Build Rust Workspace

```bash
cd rust && cargo build --workspace && cd ..
```

### 4. Run Tests

```bash
cd rust && cargo test --workspace && cd ..
```

## Related Documentation

- [Installation](installation.md) - Installation guide
- [Building](building.md) - Build instructions
- [Testing](testing.md) - Test execution
