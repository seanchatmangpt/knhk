# Migration Guide - KNHK v1.0.0

**Target Audience**: New users adopting KNHK v1.0.0
**Last Updated**: 2025-11-08

---

## Overview

This is the **initial v1.0.0 release** of KNHK. This guide is for users migrating from:

1. **Experimental/pre-release versions** (v0.x.x)
2. **Other knowledge graph frameworks** (RDF4J, Jena, etc.)
3. **Custom solutions** (starting fresh with KNHK)

If you're starting fresh with KNHK, skip to [Quick Start](#quick-start-for-new-users).

---

## Migration Scenarios

### Scenario 1: From KNHK v0.x.x (Pre-Release)

**Breaking Changes**: None (v1.0.0 is the first stable release)

**Recommended Actions**:

1. **Update to v1.0.0**:
   ```bash
   # Update Cargo.toml
   [dependencies]
   knhk-hot = "1.0.0"
   knhk-etl = "1.0.0"
   # ... other KNHK crates
   ```

2. **Rebuild C library**:
   ```bash
   cd c && make clean && make && cd ../rust
   ```

3. **Run validation**:
   ```bash
   cargo test --workspace
   ./scripts/validate-pre-push.sh
   ```

4. **Update OTEL schemas** (if you added custom telemetry):
   ```bash
   weaver registry check -r registry/
   ```

**Compatibility**:
- ✅ All v0.x.x APIs are compatible with v1.0.0
- ✅ No code changes required
- ✅ Existing OTEL schemas remain valid

---

### Scenario 2: From Other RDF/Knowledge Graph Frameworks

**Common Migrations**:

| From Framework | Migration Path | Compatibility |
|---------------|----------------|---------------|
| **RDF4J** | Use knhk-connectors + knhk-unrdf | ✅ RDF/XML, Turtle, N-Triples |
| **Apache Jena** | Use knhk-connectors + knhk-unrdf | ✅ SPARQL queries supported |
| **Neo4j** | Custom connector required | ⚠️ Graph model differs |
| **AllegroGraph** | Use knhk-connectors + SPARQL | ✅ RDF triple store compatible |

**Key Differences**:

| Feature | RDF4J/Jena | KNHK v1.0.0 |
|---------|-----------|-------------|
| **Hot path latency** | ms-scale | **≤8 ticks (μs-scale)** |
| **Allocation strategy** | Heap-based | **Zero-allocation buffer pools** |
| **SIMD acceleration** | No | **Yes (ARM64 NEON, x86_64 AVX2)** |
| **Validation** | Unit tests | **Weaver OTEL schema validation** |
| **Language** | Java | **Rust (+ C FFI)** |

**Migration Steps**:

1. **Export existing RDF data**:
   ```bash
   # From RDF4J
   rdf4j-console export myRepo output.ttl turtle

   # From Jena
   tdb2.tdbdump --loc /path/to/dataset > output.nq
   ```

2. **Ingest into KNHK**:
   ```bash
   # Use knhk-cli (to be implemented in v1.1)
   knhk admit --format turtle output.ttl

   # Or use knhk-etl programmatically
   use knhk_etl::Pipeline;
   let pipeline = Pipeline::new()?;
   pipeline.ingest_rdf("output.ttl", RdfFormat::Turtle)?;
   ```

3. **Migrate SPARQL queries**:
   ```rust
   // KNHK v1.0.0 (warm path)
   use knhk_warm::QueryEngine;

   let engine = QueryEngine::new()?;
   let results = engine.execute_sparql(
       "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"
   )?;
   ```

4. **Add OTEL telemetry** (KNHK-specific):
   ```rust
   use knhk_otel::telemetry;

   #[tracing::instrument]
   fn process_query(query: &str) -> Result<QueryResults> {
       // KNHK automatically emits telemetry
       // Validate with: weaver registry live-check
   }
   ```

---

### Scenario 3: Starting Fresh with KNHK

See [Quick Start](#quick-start-for-new-users) below.

---

## Quick Start for New Users

### Prerequisites

**Required**:
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# C compiler (choose one)
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf install gcc make
```

**Optional**:
```bash
# OpenTelemetry Weaver (for validation)
cargo install weaver-cli

# Criterion (for benchmarks)
cargo install cargo-criterion
```

### Installation

**1. Clone repository**:
```bash
git clone https://github.com/your-org/knhk.git
cd knhk
```

**2. Build C library** (REQUIRED):
```bash
cd c
make
cd ../rust
```

**3. Build Rust workspace**:
```bash
# Debug build (fast, for development)
cargo build --workspace

# Release build (optimized, for production)
cargo build --workspace --release
```

**4. Verify installation**:
```bash
# Run tests
cargo test --workspace

# Quick validation (~3 min)
./scripts/validate-pre-commit.sh

# Full validation (~25 min)
./scripts/validate-release.sh
```

**5. Validate with Weaver** (recommended):
```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

### Project Structure

```
knhk/
├── c/                          # C hot path library
│   ├── libknhk.a              # Static library (build artifact)
│   └── src/                   # C source files
├── rust/                       # Rust workspace
│   ├── Cargo.toml             # Workspace manifest
│   ├── knhk-hot/              # Hot path FFI (≤8 ticks)
│   ├── knhk-etl/              # Pipeline orchestration
│   ├── knhk-warm/             # Query optimization
│   ├── knhk-otel/             # OTEL integration
│   ├── knhk-cli/              # CLI interface
│   ├── docs/                  # Documentation
│   └── scripts/               # Validation scripts
├── docs/                       # Top-level documentation
│   └── book/                  # mdBook guide
└── registry/                   # OTEL Weaver schemas
```

### First Steps

**1. Read the documentation**:
```bash
cd docs/book
mdbook serve
# Open http://localhost:3000
```

**2. Explore examples** (to be added in v1.1):
```bash
cd rust/examples
cargo run --example basic_pipeline
cargo run --example simd_predicates
```

**3. Write your first integration**:
```rust
// my_app/src/main.rs
use knhk_hot::HotPath;
use knhk_etl::Pipeline;
use knhk_otel::telemetry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry
    telemetry::init()?;

    // Create hot path runtime
    let hot_path = HotPath::new();

    // Create pipeline
    let pipeline = Pipeline::new()?;

    // Process data (≤8 ticks per operation)
    hot_path.process_batch(&data)?;

    // Validate with Weaver
    // weaver registry live-check --registry registry/

    Ok(())
}
```

**4. Run validation**:
```bash
# Before committing code
./scripts/validate-pre-commit.sh

# Before pushing to remote
./scripts/validate-pre-push.sh
```

---

## API Changes (v0.x.x → v1.0.0)

**No breaking changes** (v1.0.0 is the first stable release).

For future versions, API changes will be documented here.

---

## Configuration Changes

### OTEL Configuration

**v1.0.0 introduces schema-first OTEL validation**:

**Before** (v0.x.x):
```rust
// No schema validation
tracing::info!("Processing data");
```

**After** (v1.0.0):
```rust
// Schema-validated telemetry
#[tracing::instrument]
fn process_data() {
    // MUST be declared in registry/knhk.yaml
    tracing::info!("Processing data");
}

// Validate at build time:
// weaver registry check -r registry/

// Validate at runtime:
// weaver registry live-check --registry registry/
```

### Feature Flags

**New feature flags in v1.0.0**:

```toml
# Cargo.toml
[dependencies]
knhk-etl = { version = "1.0.0", features = ["grpc", "tokio-runtime"] }
knhk-validation = { version = "1.0.0", features = ["advisor", "policy-engine"] }
knhk-warm = { version = "1.0.0", features = ["otel", "unrdf"] }
```

**Available features**:

| Crate | Feature | Description |
|-------|---------|-------------|
| **knhk-etl** | `grpc` | Enable gRPC server |
| **knhk-etl** | `tokio-runtime` | Enable async runtime |
| **knhk-etl** | `parallel` | Enable parallel processing |
| **knhk-validation** | `advisor` | Enable validation advisor |
| **knhk-validation** | `policy-engine` | Enable Rego policy engine |
| **knhk-validation** | `streaming` | Enable streaming validation |
| **knhk-warm** | `otel` | Enable OTEL telemetry |
| **knhk-warm** | `unrdf` | Enable RDF storage |
| **knhk-patterns** | `unrdf` | Enable RDF workflow patterns |

---

## Performance Migration

### From Heap Allocation to Buffer Pooling

**Before** (traditional approach):
```rust
fn process_data(input: &[u8]) -> Vec<u8> {
    // Heap allocation per call
    let mut output = Vec::new();
    // ... processing
    output
}
```

**After** (KNHK buffer pooling):
```rust
use knhk_hot::BufferPool;

fn process_data(input: &[u8], pool: &BufferPool) -> Result<Buffer> {
    // Zero-allocation hot path
    let mut buffer = pool.acquire()?;
    // ... processing
    Ok(buffer) // Returns to pool when dropped
}
```

### From Scalar to SIMD

**Before** (scalar):
```rust
fn matches_predicate(data: &[u8], threshold: u8) -> Vec<bool> {
    data.iter().map(|&b| b <= threshold).collect()
}
```

**After** (SIMD-accelerated):
```rust
use knhk_hot::simd::predicate_match;

fn matches_predicate(data: &[u8], threshold: u8) -> Vec<bool> {
    // 2-4x faster via SIMD intrinsics
    predicate_match(data, threshold)
}
```

---

## Testing Migration

### From Traditional Tests to Chicago TDD

**Before** (traditional):
```rust
#[test]
fn test_feature() {
    let result = feature();
    assert!(result.is_ok());
}
```

**After** (Chicago TDD with AAA pattern):
```rust
#[test]
fn test_feature_succeeds_with_valid_input() {
    // Arrange: Setup test data
    let input = valid_test_input();

    // Act: Execute the feature
    let result = feature(input);

    // Assert: Verify expected behavior
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_output());
}
```

### From Unit Tests to Weaver Validation

**Before** (unit tests only):
```rust
#[test]
fn test_telemetry_emitted() {
    // Mock telemetry collection
    let mock = MockTelemetry::new();
    emit_telemetry();
    assert_eq!(mock.count(), 1);
}
```

**After** (Weaver schema validation):
```yaml
# registry/knhk.yaml
spans:
  - name: process_data
    attributes:
      - name: data.size
        type: int
      - name: data.format
        type: string
```

```bash
# Validate at runtime (SOURCE OF TRUTH)
weaver registry live-check --registry registry/
```

**Key Insight**: Traditional telemetry tests can pass even when telemetry is broken. Weaver validates actual runtime behavior.

---

## Deployment Changes

### Build Process

**Before** (Rust only):
```bash
cargo build --release
```

**After** (C library + Rust):
```bash
# 1. Build C library FIRST
cd c && make && cd ../rust

# 2. Build Rust workspace
cargo build --workspace --release
```

### CI/CD Integration

**Minimal CI pipeline** (v1.0.0):
```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Build C library
      - name: Build C library
        run: cd c && make

      # Build Rust workspace
      - name: Build workspace
        run: cd rust && cargo build --workspace --release

      # Run tests
      - name: Run tests
        run: cd rust && cargo test --workspace

      # Clippy validation
      - name: Clippy
        run: cd rust && cargo clippy --workspace -- -D warnings

      # Weaver validation (optional but recommended)
      - name: Install Weaver
        run: cargo install weaver-cli
      - name: Validate schemas
        run: weaver registry check -r registry/
```

**Full CI pipeline** (with all validations):
```yaml
# Use provided validation scripts
- name: Pre-push validation
  run: cd rust/scripts && ./validate-pre-push.sh

- name: Feature matrix validation
  run: cd rust/scripts && ./validate-feature-matrix.sh

- name: Integration tests
  run: cd rust/scripts && ./validate-integrations.sh
```

---

## Troubleshooting

### Common Migration Issues

**Issue 1: "could not find native static library `knhk`"**

**Cause**: C library not built or not found

**Solution**:
```bash
cd c && make clean && make && cd ../rust
cargo clean -p knhk-hot
cargo build --workspace
```

---

**Issue 2: Weaver validation fails but tests pass**

**Cause**: Tests validate mocks, not actual runtime behavior (false positive)

**Solution**: Fix the actual feature implementation, not the test
```bash
# Identify missing telemetry
weaver registry live-check --registry registry/

# Add missing spans/metrics/logs to code
#[tracing::instrument]
fn feature() {
    tracing::info!("Processing started"); // Must be in schema
}

# Re-validate
weaver registry live-check --registry registry/
```

---

**Issue 3: Build performance degradation**

**Cause**: Workspace dependency inheritance includes unnecessary deps

**Solution**: Use feature flags to opt-in
```toml
# Before (pulls all dependencies)
[dependencies]
knhk-etl = "1.0.0"

# After (minimal dependencies)
[dependencies]
knhk-etl = { version = "1.0.0", default-features = false, features = ["grpc"] }
```

---

**Issue 4: Hot path exceeds 8 ticks**

**Cause**: Heap allocations or scalar operations in hot path

**Solution**: Use buffer pooling and SIMD
```rust
// Before (heap allocation)
let output = Vec::new();

// After (buffer pool)
let output = pool.acquire()?;

// Before (scalar)
data.iter().map(|b| b <= threshold)

// After (SIMD)
predicate_match(data, threshold)
```

---

## Support Resources

### Documentation

- **mdBook Guide**: `docs/book/` (comprehensive architecture and API docs)
- **CHANGELOG**: `rust/CHANGELOG.md` (all changes by version)
- **Release Notes**: `rust/docs/RELEASE_NOTES_v1.0.0.md` (this release)
- **API Docs**: `cargo doc --workspace --open`

### Validation Tools

- **Pre-commit**: `rust/scripts/validate-pre-commit.sh` (~3 min)
- **Pre-push**: `rust/scripts/validate-pre-push.sh` (~6 min)
- **Release**: `rust/scripts/validate-release.sh` (~25 min)

### Community

- **Issues**: https://github.com/your-org/knhk/issues (update with actual URL)
- **Discussions**: https://github.com/your-org/knhk/discussions (update with actual URL)
- **Discord**: (to be created)

---

## Next Steps

After migrating to v1.0.0:

1. **Read the mdBook guide** (`docs/book/`)
2. **Run validation scripts** (`scripts/validate-pre-push.sh`)
3. **Validate with Weaver** (`weaver registry live-check`)
4. **Review known issues** ([RELEASE_NOTES](RELEASE_NOTES_v1.0.0.md#known-issues))
5. **Plan for v1.1** ([roadmap](RELEASE_NOTES_v1.0.0.md#roadmap))

---

**Questions?** Open an issue or check the [FAQ](RELEASE_NOTES_v1.0.0.md#faq).

**Migration Guide Version**: 1.0.0
**Last Updated**: 2025-11-08
