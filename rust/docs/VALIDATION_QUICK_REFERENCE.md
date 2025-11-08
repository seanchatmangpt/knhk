# KNHK Build Validation Quick Reference

**Version:** 1.0.0 | **Updated:** 2025-11-07

## ⚡ Quick Command Reference

| Scenario | Command | Time | Description |
|----------|---------|------|-------------|
| **Before commit** | `./scripts/validate-pre-commit.sh` | 3 min | Format, clippy, lib tests, Chicago TDD |
| **Before push** | `./scripts/validate-pre-push.sh` | 6 min | All commits + full tests + perf + features |
| **Feature flags** | `./scripts/validate-feature-matrix.sh` | 10 min | Test 32 feature combinations |
| **Integrations** | `./scripts/validate-integrations.sh` | 15 min | Core, pipeline, validation, workspace |
| **All tests** | `./scripts/validate-tests.sh` | 20 min | Unit, integration, doc, Chicago, benchmarks |
| **Pre-release** | `./scripts/validate-release.sh` | 25 min | Comprehensive validation + docs + audit |

## 📊 Test Matrix Summary

### Individual Package Builds (13 packages)

```bash
# Build single package
cargo build -p <package> --release
cargo test -p <package> --lib
cargo clippy -p <package> -- -D warnings

# Packages
knhk-hot knhk-otel knhk-config knhk-etl knhk-warm knhk-unrdf knhk-patterns
knhk-validation knhk-lockchain knhk-connectors knhk-aot knhk-cli knhk-integration-tests
```

### Feature Flag Combinations (32 total)

| Package | Features | Combinations |
|---------|----------|--------------|
| `knhk-otel` | `std` | 3 |
| `knhk-connectors` | `kafka`, `salesforce` | 4 |
| `knhk-unrdf` | `native`, `unrdf` | 4 |
| `knhk-etl` | `grpc`, `tokio-runtime`, `parallel` | 6 |
| `knhk-warm` | `otel`, `unrdf` | 4 |
| `knhk-validation` | `advisor`, `policy-engine`, `schema-resolution`, `streaming` | 7 |
| `knhk-patterns` | `unrdf` | 2 |
| `knhk-cli` | `otel` | 3 |

```bash
# Example: Test all features for a package
cargo build -p knhk-connectors --all-features
cargo build -p knhk-connectors --no-default-features --features kafka
```

### Integration Scenarios (12 total)

| Scenario | Packages | Command |
|----------|----------|---------|
| **Core system** | hot, otel, config, lockchain | `cargo build -p knhk-hot -p knhk-otel -p knhk-config -p knhk-lockchain --release` |
| **Pipeline** | etl, warm, patterns, unrdf | `cargo build -p knhk-etl -p knhk-warm -p knhk-patterns -p knhk-unrdf --release --all-features` |
| **Validation** | validation, lockchain, connectors | `cargo build -p knhk-validation -p knhk-lockchain -p knhk-connectors --release --all-features` |
| **Full workspace** | all packages | `cargo build --workspace --release --all-features` |

### Test Suites (21 scenarios)

| Test Type | Command | Time | Purpose |
|-----------|---------|------|---------|
| **Lib tests** | `cargo test --workspace --lib` | 180s | Unit tests only |
| **Integration** | `cargo test --workspace --test '*'` | 300s | Integration tests |
| **Doc tests** | `cargo test --workspace --doc` | 120s | Documentation examples |
| **Chicago TDD** | `make test-chicago-v04` | 60s | Architecture compliance |
| **Performance** | `make test-performance-v04` | 30s | ≤8 ticks validation |
| **Benchmarks** | `cargo bench --workspace` | 300s | Performance benchmarks |

## 🎯 Common Workflows

### Local Development Loop

```bash
# Fast iteration (~30s)
cargo check --workspace
cargo test -p <your-package> --lib

# Before commit (~3 min)
./scripts/validate-pre-commit.sh

# Before push (~6 min)
./scripts/validate-pre-push.sh
```

### CI Pipeline Stages

```bash
# PR validation (5-10 min)
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace --lib
make test-chicago-v04

# Main branch (20-30 min)
./scripts/validate-pre-push.sh
./scripts/validate-feature-matrix.sh

# Nightly (60-90 min)
./scripts/validate-release.sh
cargo bench --workspace
cargo tarpaulin --workspace --out Html

# Pre-release (120-180 min)
./scripts/validate-release.sh
weaver registry check -r registry/
weaver registry live-check --registry registry/
cargo doc --workspace --all-features
```

### Release Checklist

- [ ] `./scripts/validate-release.sh` passes (25 min)
- [ ] `weaver registry check -r registry/` passes ✅ **CRITICAL**
- [ ] `weaver registry live-check --registry registry/` passes ✅ **CRITICAL**
- [ ] `cargo doc --workspace --all-features --no-deps` succeeds
- [ ] `cargo audit` shows no vulnerabilities
- [ ] Version bumped in all Cargo.toml files
- [ ] CHANGELOG.md updated
- [ ] Git tag created: `git tag -a v1.0.0 -m "Release v1.0.0"`

## 🚨 Critical Validation Rules

### The Hierarchy (MUST FOLLOW)

1. **LEVEL 1: Weaver Schema Validation** ✅ **SOURCE OF TRUTH**
   ```bash
   weaver registry check -r registry/          # Schema is valid
   weaver registry live-check --registry registry/  # Runtime conforms to schema
   ```

2. **LEVEL 2: Compilation & Code Quality**
   ```bash
   cargo build --release                       # Compiles
   cargo clippy --workspace -- -D warnings     # Zero warnings
   ```

3. **LEVEL 3: Traditional Tests** (Supporting evidence only)
   ```bash
   cargo test --workspace                      # Tests pass
   make test-chicago-v04                       # Architecture tests
   ```

**⚠️ REMEMBER:** Tests can pass with false positives. Only Weaver validation proves features work.

### The False Positive Paradox

```
❌ WRONG VALIDATION:
  knhk --help  →  "Help text exists"  →  ✅  →  ASSUME FEATURE WORKS
  └─ Help text can exist for unimplemented!() functions

✅ CORRECT VALIDATION:
  knhk <command> <args>  →  "Command executes"  →  Check output/telemetry
  weaver registry live-check  →  "Telemetry matches schema"  →  ✅ FEATURE WORKS
```

## 📦 Package Build Times (Approximate)

| Package | Clean Build | Incremental | With Tests |
|---------|-------------|-------------|------------|
| `knhk-hot` | 60s | 10s | 90s |
| `knhk-otel` | 90s | 15s | 120s |
| `knhk-config` | 30s | 5s | 45s |
| `knhk-etl` | 75s | 12s | 150s |
| `knhk-warm` | 60s | 10s | 120s |
| `knhk-unrdf` | 90s | 15s | 180s |
| `knhk-patterns` | 60s | 10s | 90s |
| `knhk-validation` | 60s | 10s | 120s |
| `knhk-lockchain` | 45s | 8s | 75s |
| `knhk-connectors` | 75s | 12s | 105s |
| `knhk-aot` | 45s | 8s | 60s |
| `knhk-cli` | 120s | 20s | 180s |
| `knhk-integration-tests` | 180s | 30s | 300s |
| **Full workspace** | **480s** | **90s** | **600s** |

## 🔧 Performance Optimization

### Use Incremental Builds

```bash
export CARGO_INCREMENTAL=1  # Default, keep enabled
```

### Use Build Cache

```bash
# Install and configure sccache
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### Parallel Test Execution

```bash
# Library tests (fast, high parallelism)
cargo test --workspace --lib -- --test-threads=8

# Integration tests (slow, low parallelism)
cargo test --workspace --test '*' -- --test-threads=2
```

### Selective Testing

```bash
# Test only changed packages
cargo test -p knhk-etl -p knhk-warm

# Test only specific test names
cargo test --workspace -- pattern_matching

# Skip expensive tests
cargo test --workspace -- --skip integration_
```

## 🐛 Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| **Script not executable** | Missing permissions | `chmod +x scripts/*.sh` |
| **Makefile not found** | Running from wrong dir | `cd /Users/sac/knhk/rust` |
| **C compiler error** | Missing build tools | Install build-essential / Xcode |
| **OTel version conflict** | Version 0.21 vs 0.31 | Check feature flags |
| **Async trait errors** | knhk-sidecar | Excluded, Wave 5 fix |
| **Clippy fails** | `-D warnings` in CI | Fix all warnings |
| **Tests timeout** | Slow CI runners | Increase timeout or parallelize |

## 📚 Related Documentation

- **[BUILD_VALIDATION_MATRIX.md](BUILD_VALIDATION_MATRIX.md)** - Complete 78-scenario matrix
- **[scripts/README.md](../scripts/README.md)** - Script documentation
- **[CLAUDE.md](../../CLAUDE.md)** - Project guidelines
- **[Definition of Done](DEFINITION_OF_DONE.md)** - Release criteria

## 🚀 One-Line Validators

```bash
# Quick pre-commit check
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace --lib

# Quick pre-push check
cargo test --workspace && make test-performance-v04

# Quick feature check
cargo build --workspace --all-features

# Quick release check
./scripts/validate-release.sh && echo "✅ Ready for Weaver validation"
```

## 📊 Validation Coverage

| Area | Scenarios | Commands | Scripts |
|------|-----------|----------|---------|
| **Individual builds** | 13 | 39 | - |
| **Feature flags** | 32 | 32 | `validate-feature-matrix.sh` |
| **Integrations** | 12 | 12 | `validate-integrations.sh` |
| **Tests** | 21 | 21 | `validate-tests.sh` |
| **CI workflows** | 4 | - | All scripts |
| **Total** | **78** | **104+** | **6 scripts** |

---

**Last Updated:** 2025-11-07
**Maintainer:** KNHK Team
**Version:** 1.0.0
