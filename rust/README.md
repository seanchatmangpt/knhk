# KNHK Rust Workspace

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/your-org/knhk/releases/tag/v1.0.0)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Validation](https://img.shields.io/badge/validation-Weaver%20OTEL-purple.svg)](https://github.com/open-telemetry/weaver)

This directory contains the Rust implementation of KNHK (Knowledge Graph Hot Path), organized as a Cargo workspace.

**v1.0.0 Release**: Production-ready knowledge graph framework with ≤8 tick hot path latency, zero-allocation buffer pooling, SIMD acceleration, and Weaver OTEL validation. See [CHANGELOG](CHANGELOG.md) and [Release Notes](docs/RELEASE_NOTES_v1.0.0.md).

## Workspace Structure

### Core Crates

- **knhk-hot** (v1.0.0) - Hot path FFI layer (links to C library)
- **knhk-etl** (v0.1.0) - Extract, Transform, Load pipeline with beat scheduling
- **knhk-warm** (v0.1.0) - Warm path query optimization with caching

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

**See**: [ADR-003: Weaver Validation as Source of Truth](docs/architecture/adrs/ADR-003-weaver-validation-source-of-truth.md)

## Performance Features (v1.0.0)

### Hot Path Compliance
- **≤8 ticks (Chatman Constant)**: Core operations meet latency constraint
- **Zero allocations**: Buffer pooling eliminates hot path allocations
- **SIMD acceleration**: 2-4x speedup via ARM64 NEON + x86_64 AVX2
- **>95% cache hit rate**: Buffer pool efficiency in production workloads

**See**:
- [ADR-001: Buffer Pooling Strategy](docs/architecture/adrs/ADR-001-buffer-pooling-strategy.md)
- [ADR-002: SIMD Implementation Approach](docs/architecture/adrs/ADR-002-simd-implementation-approach.md)

### Benchmark Results

```
hot_path_execution         time:   [1.2 ticks  1.3 ticks  1.4 ticks]
buffer_pool_allocation     time:   [0.1 ticks  0.1 ticks  0.1 ticks]
simd_predicate_match       time:   [0.3 ticks  0.3 ticks  0.4 ticks]
```

Run benchmarks: `cd knhk-hot && cargo bench`

## Release Documentation (v1.0.0)

### Essential Reading

- **[CHANGELOG](CHANGELOG.md)**: All changes by version
- **[Release Notes v1.0.0](docs/RELEASE_NOTES_v1.0.0.md)**: Executive summary, features, metrics
- **[Migration Guide](docs/MIGRATION_GUIDE_v1.0.0.md)**: Upgrade instructions and examples

### Architecture Decisions

- **[ADR-001](docs/architecture/adrs/ADR-001-buffer-pooling-strategy.md)**: Buffer pooling for zero allocations
- **[ADR-002](docs/architecture/adrs/ADR-002-simd-implementation-approach.md)**: SIMD cross-platform acceleration
- **[ADR-003](docs/architecture/adrs/ADR-003-weaver-validation-source-of-truth.md)**: Weaver validation eliminates false positives
- **[ADR-004](docs/architecture/adrs/ADR-004-chicago-tdd-methodology.md)**: Behavior-focused testing approach

### Validation Reports

- **[Production Readiness Summary](docs/PRODUCTION_READINESS_SUMMARY.md)**: 10/14 crates ready (71%)
- **[Benchmark Executive Summary](docs/BENCHMARK_EXECUTIVE_SUMMARY.md)**: Performance metrics and optimization roadmap
- **[Permutational Validation Report](docs/PERMUTATIONAL_VALIDATION_REPORT.md)**: 143 integration scenarios
- **[Code Quality Analysis v1.0.0](docs/code-quality-analysis-v1.0.0.md)**: 7.5/10 overall score

### Quick Start

```bash
# Build C library first
cd c && make && cd ../rust

# Build workspace
cargo build --workspace --release

# Run tests
cargo test --workspace

# Validate with Weaver (recommended)
weaver registry check -r registry/
weaver registry live-check --registry registry/

# Pre-push validation (~6 min)
./scripts/validate-pre-push.sh
```

## Contributing

When adding new crates:

1. Create crate directory under `rust/`
2. Add to `[workspace.members]` in `rust/Cargo.toml`
3. Use `[workspace.dependencies]` for common deps
4. Follow existing patterns for features and profiles

## License

MIT
