# KNHK Build Validation Matrix

**Version:** 1.0.0
**Generated:** 2025-11-07
**Total Validation Scenarios:** 78

## Executive Summary

This matrix provides systematic validation of the KNHK monorepo across:
- **13 workspace packages** (12 active + 1 excluded)
- **32 feature flag combinations**
- **12 integration build scenarios**
- **21 test combination scenarios**

**Estimated Total Validation Time:** 45-60 minutes (parallel) / 180-240 minutes (sequential)

---

## 1. Individual Package Builds (13 packages)

### 1.1 Core System Packages

| Package | Build | Test | Clippy | Est. Time | Dependencies |
|---------|-------|------|--------|-----------|--------------|
| `knhk-hot` | `cargo build -p knhk-hot --release` | `cargo test -p knhk-hot --lib` | `cargo clippy -p knhk-hot -- -D warnings` | 60s | C library (optional) |
| `knhk-otel` | `cargo build -p knhk-otel --release` | `cargo test -p knhk-otel --lib` | `cargo clippy -p knhk-otel -- -D warnings` | 90s | OpenTelemetry 0.31 |
| `knhk-config` | `cargo build -p knhk-config --release` | `cargo test -p knhk-config --lib` | `cargo clippy -p knhk-config -- -D warnings` | 30s | None |

**Combined command:**
```bash
# Parallel validation (requires cargo-make or parallel)
for pkg in knhk-hot knhk-otel knhk-config; do
  (cargo build -p $pkg --release && \
   cargo test -p $pkg --lib && \
   cargo clippy -p $pkg -- -D warnings) &
done; wait
```

### 1.2 Data Processing Packages

| Package | Build | Test | Clippy | Est. Time | Dependencies |
|---------|-------|------|--------|-----------|--------------|
| `knhk-etl` | `cargo build -p knhk-etl --release` | `cargo test -p knhk-etl --lib` | `cargo clippy -p knhk-etl -- -D warnings` | 75s | otel, lockchain |
| `knhk-warm` | `cargo build -p knhk-warm --release` | `cargo test -p knhk-warm --lib` | `cargo clippy -p knhk-warm -- -D warnings` | 60s | Optional: otel, unrdf |
| `knhk-unrdf` | `cargo build -p knhk-unrdf --release` | `cargo test -p knhk-unrdf --lib` | `cargo clippy -p knhk-unrdf -- -D warnings` | 90s | Optional: oxigraph |
| `knhk-patterns` | `cargo build -p knhk-patterns --release` | `cargo test -p knhk-patterns --lib` | `cargo clippy -p knhk-patterns -- -D warnings` | 60s | C compiler (cc) |

**Combined command:**
```bash
for pkg in knhk-etl knhk-warm knhk-unrdf knhk-patterns; do
  (cargo build -p $pkg --release && \
   cargo test -p $pkg --lib && \
   cargo clippy -p $pkg -- -D warnings) &
done; wait
```

### 1.3 Integration & Infrastructure Packages

| Package | Build | Test | Clippy | Est. Time | Dependencies |
|---------|-------|------|--------|-----------|--------------|
| `knhk-connectors` | `cargo build -p knhk-connectors --release` | `cargo test -p knhk-connectors --lib` | `cargo clippy -p knhk-connectors -- -D warnings` | 75s | Optional: rdkafka, reqwest |
| `knhk-lockchain` | `cargo build -p knhk-lockchain --release` | `cargo test -p knhk-lockchain --lib` | `cargo clippy -p knhk-lockchain -- -D warnings` | 45s | None |
| `knhk-validation` | `cargo build -p knhk-validation --release` | `cargo test -p knhk-validation --lib` | `cargo clippy -p knhk-validation -- -D warnings` | 60s | Optional: miette, regorus |
| `knhk-aot` | `cargo build -p knhk-aot --release` | `cargo test -p knhk-aot --lib` | `cargo clippy -p knhk-aot -- -D warnings` | 45s | None |

**Combined command:**
```bash
for pkg in knhk-connectors knhk-lockchain knhk-validation knhk-aot; do
  (cargo build -p $pkg --release && \
   cargo test -p $pkg --lib && \
   cargo clippy -p $pkg -- -D warnings) &
done; wait
```

### 1.4 User-Facing Packages

| Package | Build | Test | Clippy | Est. Time | Dependencies |
|---------|-------|------|--------|-----------|--------------|
| `knhk-cli` | `cargo build -p knhk-cli --release` | `cargo test -p knhk-cli --lib` | `cargo clippy -p knhk-cli -- -D warnings` | 120s | All core packages |
| `knhk-integration-tests` | `cargo build -p knhk-integration-tests --release` | `cargo test -p knhk-integration-tests` | `cargo clippy -p knhk-integration-tests -- -D warnings` | 180s | All packages |

**Combined command:**
```bash
# Sequential (due to heavy dependencies)
cargo build -p knhk-cli --release && \
cargo test -p knhk-cli --lib && \
cargo clippy -p knhk-cli -- -D warnings

cargo build -p knhk-integration-tests --release && \
cargo test -p knhk-integration-tests && \
cargo clippy -p knhk-integration-tests -- -D warnings
```

### 1.5 Excluded Packages (Technical Debt)

| Package | Status | Reason | Est. Effort |
|---------|--------|--------|-------------|
| `knhk-sidecar` | ❌ Excluded | 53 async trait errors | Wave 5 |

---

## 2. Feature Flag Combinations (32 scenarios)

### 2.1 knhk-otel Features

| Feature Set | Command | Est. Time | Purpose |
|-------------|---------|-----------|---------|
| Default (std) | `cargo build -p knhk-otel --release` | 90s | Standard telemetry |
| No default features | `cargo build -p knhk-otel --release --no-default-features` | 60s | Minimal build |
| std only | `cargo build -p knhk-otel --release --no-default-features --features std` | 90s | Explicit std |

**Validation command:**
```bash
# Test all feature combinations
cargo build -p knhk-otel --release --all-features && \
cargo build -p knhk-otel --release --no-default-features && \
cargo test -p knhk-otel --all-features
```

### 2.2 knhk-connectors Features

| Feature Set | Command | Est. Time | Purpose |
|-------------|---------|-----------|---------|
| Default (std) | `cargo build -p knhk-connectors --release` | 60s | Minimal connectors |
| std + kafka | `cargo build -p knhk-connectors --release --features kafka` | 90s | Kafka integration |
| std + salesforce | `cargo build -p knhk-connectors --release --features salesforce` | 75s | Salesforce integration |
| All features | `cargo build -p knhk-connectors --release --all-features` | 105s | All connectors |

**Validation command:**
```bash
# Test feature matrix
for feat in "" "kafka" "salesforce" "kafka,salesforce"; do
  if [ -z "$feat" ]; then
    cargo build -p knhk-connectors --release
  else
    cargo build -p knhk-connectors --release --features "$feat"
  fi
done
```

### 2.3 knhk-unrdf Features

| Feature Set | Command | Est. Time | Purpose |
|-------------|---------|-----------|---------|
| Minimal (no default) | `cargo build -p knhk-unrdf --release --no-default-features` | 45s | Minimal build |
| native | `cargo build -p knhk-unrdf --release --no-default-features --features native` | 120s | Oxigraph + crypto |
| unrdf | `cargo build -p knhk-unrdf --release --no-default-features --features unrdf` | 60s | JS integration |
| native + unrdf | `cargo build -p knhk-unrdf --release --no-default-features --features native,unrdf` | 135s | Full features |

**Validation command:**
```bash
# Test all combinations
cargo build -p knhk-unrdf --release --no-default-features && \
cargo build -p knhk-unrdf --release --features native && \
cargo build -p knhk-unrdf --release --features unrdf && \
cargo build -p knhk-unrdf --release --all-features && \
cargo test -p knhk-unrdf --all-features
```

### 2.4 knhk-etl Features

| Feature Set | Command | Est. Time | Purpose |
|-------------|---------|-----------|---------|
| Default | `cargo build -p knhk-etl --release` | 75s | std + otel + lockchain + parallel |
| Minimal | `cargo build -p knhk-etl --release --no-default-features` | 45s | Core only |
| std only | `cargo build -p knhk-etl --release --no-default-features --features std` | 50s | Standard library |
| grpc | `cargo build -p knhk-etl --release --features grpc` | 90s | gRPC support |
| tokio-runtime | `cargo build -p knhk-etl --release --features tokio-runtime` | 85s | Async runtime |
| All features | `cargo build -p knhk-etl --release --all-features` | 105s | Full feature set |

**Validation command:**
```bash
# Critical feature combinations
cargo build -p knhk-etl --release --no-default-features && \
cargo build -p knhk-etl --release --features grpc && \
cargo build -p knhk-etl --release --all-features && \
cargo test -p knhk-etl --all-features
```

### 2.5 knhk-warm Features

| Feature Set | Command | Est. Time | Purpose |
|-------------|---------|-----------|---------|
| Default (std) | `cargo build -p knhk-warm --release` | 60s | Standard build |
| std + otel | `cargo build -p knhk-warm --release --features otel` | 75s | Telemetry support |
| std + unrdf | `cargo build -p knhk-warm --release --features unrdf` | 90s | RDF support |
| All features | `cargo build -p knhk-warm --release --all-features` | 105s | Full features |

**Validation command:**
```bash
cargo build -p knhk-warm --release --all-features && \
cargo test -p knhk-warm --all-features
```

### 2.6 knhk-validation Features

| Feature Set | Command | Est. Time | Purpose |
|-------------|---------|-----------|---------|
| Default (std + diagnostics) | `cargo build -p knhk-validation --release` | 60s | Standard validation |
| Minimal | `cargo build -p knhk-validation --release --no-default-features` | 40s | Core only |
| advisor | `cargo build -p knhk-validation --release --features advisor` | 90s | Policy engine |
| policy-engine | `cargo build -p knhk-validation --release --features policy-engine` | 85s | Regorus integration |
| schema-resolution | `cargo build -p knhk-validation --release --features schema-resolution` | 65s | Schema support |
| streaming | `cargo build -p knhk-validation --release --features streaming` | 70s | Streaming validation |
| All features | `cargo build -p knhk-validation --release --all-features` | 120s | Full feature set |

**Validation command:**
```bash
# Test critical combinations
cargo build -p knhk-validation --release --no-default-features && \
cargo build -p knhk-validation --release --features advisor,policy-engine && \
cargo build -p knhk-validation --release --all-features && \
cargo test -p knhk-validation --all-features
```

### 2.7 knhk-patterns Features

| Feature Set | Command | Est. Time | Purpose |
|-------------|---------|-----------|---------|
| Minimal | `cargo build -p knhk-patterns --release` | 60s | C patterns only |
| unrdf | `cargo build -p knhk-patterns --release --features unrdf` | 120s | RDF + patterns |

**Validation command:**
```bash
cargo build -p knhk-patterns --release && \
cargo build -p knhk-patterns --release --features unrdf && \
cargo test -p knhk-patterns --all-features
```

### 2.8 knhk-cli Features

| Feature Set | Command | Est. Time | Purpose |
|-------------|---------|-----------|---------|
| Default (std + otel) | `cargo build -p knhk-cli --release` | 120s | Full CLI |
| Minimal | `cargo build -p knhk-cli --release --no-default-features` | 75s | Basic CLI |
| std only | `cargo build -p knhk-cli --release --no-default-features --features std` | 80s | No telemetry |

**Validation command:**
```bash
cargo build -p knhk-cli --release --no-default-features && \
cargo build -p knhk-cli --release --all-features && \
cargo test -p knhk-cli --lib
```

---

## 3. Integration Build Scenarios (12 scenarios)

### 3.1 Core System Integration

**Purpose:** Validate foundational infrastructure works together

| Scenario | Command | Est. Time | Validates |
|----------|---------|-----------|-----------|
| Core minimal | `cargo build -p knhk-hot -p knhk-otel -p knhk-config --release` | 120s | Basic infrastructure |
| Core full | `cargo build -p knhk-hot -p knhk-otel -p knhk-config -p knhk-lockchain --release` | 180s | Infrastructure + consensus |
| Core tests | `cargo test -p knhk-hot -p knhk-otel -p knhk-config -p knhk-lockchain` | 240s | All core tests |

**Validation command:**
```bash
# Verify core system builds and tests
cargo build -p knhk-hot -p knhk-otel -p knhk-config -p knhk-lockchain --release && \
cargo test -p knhk-hot -p knhk-otel -p knhk-config -p knhk-lockchain
```

### 3.2 Pipeline System Integration

**Purpose:** Validate ETL pipeline and data processing

| Scenario | Command | Est. Time | Validates |
|----------|---------|-----------|-----------|
| Pipeline minimal | `cargo build -p knhk-etl -p knhk-warm --release` | 90s | ETL + caching |
| Pipeline + patterns | `cargo build -p knhk-etl -p knhk-warm -p knhk-patterns --release` | 150s | Workflow patterns |
| Pipeline + unrdf | `cargo build -p knhk-etl -p knhk-warm -p knhk-unrdf --release --features unrdf` | 210s | RDF processing |
| Pipeline full | `cargo build -p knhk-etl -p knhk-warm -p knhk-patterns -p knhk-unrdf --release --all-features` | 270s | Complete pipeline |

**Validation command:**
```bash
# Verify pipeline builds with all features
cargo build -p knhk-etl -p knhk-warm -p knhk-patterns -p knhk-unrdf --release --all-features && \
cargo test -p knhk-etl -p knhk-warm -p knhk-patterns -p knhk-unrdf
```

### 3.3 Validation System Integration

**Purpose:** Validate schema validation and policy enforcement

| Scenario | Command | Est. Time | Validates |
|----------|---------|-----------|-----------|
| Validation minimal | `cargo build -p knhk-validation --release` | 60s | Basic validation |
| Validation + policy | `cargo build -p knhk-validation --release --features advisor,policy-engine` | 90s | Policy engine |
| Validation + lockchain | `cargo build -p knhk-validation -p knhk-lockchain --release` | 105s | Consensus validation |
| Validation full | `cargo build -p knhk-validation -p knhk-lockchain -p knhk-connectors --release --all-features` | 180s | Complete validation |

**Validation command:**
```bash
# Verify validation system
cargo build -p knhk-validation -p knhk-lockchain -p knhk-connectors --release --all-features && \
cargo test -p knhk-validation -p knhk-lockchain -p knhk-connectors
```

### 3.4 Full Workspace Build

**Purpose:** Validate entire workspace compiles together

| Scenario | Command | Est. Time | Validates |
|----------|---------|-----------|-----------|
| Workspace minimal | `cargo build --workspace --release` | 360s | All packages compile |
| Workspace all features | `cargo build --workspace --release --all-features` | 480s | All feature combinations |
| Workspace tests | `cargo test --workspace` | 600s | All tests pass |

**Validation command:**
```bash
# Full workspace validation (recommended nightly run)
cargo build --workspace --release --all-features && \
cargo test --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings
```

---

## 4. Test Combination Scenarios (21 scenarios)

### 4.1 Unit Tests

| Scenario | Command | Est. Time | Coverage |
|----------|---------|-----------|----------|
| All lib tests | `cargo test --workspace --lib` | 180s | Library code only |
| Core lib tests | `cargo test -p knhk-hot -p knhk-otel -p knhk-config --lib` | 60s | Core libraries |
| Pipeline lib tests | `cargo test -p knhk-etl -p knhk-warm -p knhk-patterns --lib` | 90s | Pipeline libraries |
| Validation lib tests | `cargo test -p knhk-validation -p knhk-lockchain --lib` | 45s | Validation libraries |

**Validation command:**
```bash
# Run all library tests in parallel
cargo test --workspace --lib -- --test-threads=4
```

### 4.2 Integration Tests

| Scenario | Command | Est. Time | Coverage |
|----------|---------|-----------|----------|
| All integration tests | `cargo test --workspace --test '*'` | 300s | All integration tests |
| ETL integration | `cargo test -p knhk-etl --test '*'` | 90s | ETL pipeline tests |
| Warm integration | `cargo test -p knhk-warm --test '*'` | 75s | Cache system tests |
| CLI integration | `cargo test -p knhk-integration-tests` | 180s | End-to-end tests |

**Validation command:**
```bash
# Run integration tests by package
for pkg in knhk-etl knhk-warm knhk-validation knhk-integration-tests; do
  cargo test -p $pkg --test '*'
done
```

### 4.3 Doc Tests

| Scenario | Command | Est. Time | Coverage |
|----------|---------|-----------|----------|
| All doc tests | `cargo test --workspace --doc` | 120s | Documentation examples |
| Core doc tests | `cargo test -p knhk-hot -p knhk-otel --doc` | 40s | Core docs |
| Pipeline doc tests | `cargo test -p knhk-etl -p knhk-warm --doc` | 50s | Pipeline docs |

**Validation command:**
```bash
# Verify all documentation examples compile and run
cargo test --workspace --doc
```

### 4.4 Performance Tests

| Scenario | Command | Est. Time | Coverage |
|----------|---------|-----------|----------|
| Chicago TDD performance | `make test-performance-v04` | 30s | ≤8 tick validation |
| Warm path benchmarks | `cargo bench -p knhk-warm` | 120s | Query performance |
| UnRDF benchmarks | `cargo bench -p knhk-unrdf` | 90s | Native hooks performance |

**Validation command:**
```bash
# Run performance benchmarks
make test-performance-v04 && \
cargo bench -p knhk-warm --no-fail-fast && \
cargo bench -p knhk-unrdf --no-fail-fast
```

### 4.5 Specialized Test Suites

| Scenario | Command | Est. Time | Coverage |
|----------|---------|-----------|----------|
| Chicago TDD suite | `make test-chicago-v04` | 60s | Architecture compliance |
| Enterprise tests | `make test-enterprise` | 180s | Fortune 5 use cases |
| Integration v2 | `make test-integration-v2` | 120s | Cross-package integration |

**Validation command:**
```bash
# Run all specialized test suites
make test-chicago-v04 && \
make test-enterprise && \
make test-integration-v2
```

### 4.6 Test Coverage Analysis

| Scenario | Command | Est. Time | Output |
|----------|---------|-----------|--------|
| Generate coverage | `cargo tarpaulin --workspace --out Html` | 480s | HTML report |
| Core coverage | `cargo tarpaulin -p knhk-hot -p knhk-otel -p knhk-config` | 120s | Core coverage % |
| Pipeline coverage | `cargo tarpaulin -p knhk-etl -p knhk-warm -p knhk-patterns` | 180s | Pipeline coverage % |

**Note:** Requires `cargo-tarpaulin` installed: `cargo install cargo-tarpaulin`

---

## 5. Continuous Integration Matrix

### 5.1 Quick Validation (CI Pull Request)

**Time Budget:** 5-10 minutes

```bash
#!/bin/bash
# Quick PR validation script
set -e

echo "=== Quick Validation (PR Check) ==="

# 1. Format check (10s)
cargo fmt --all -- --check

# 2. Clippy on changed packages only (60s)
cargo clippy --workspace -- -D warnings

# 3. Core tests only (120s)
cargo test -p knhk-hot -p knhk-otel -p knhk-config -p knhk-etl --lib

# 4. Chicago TDD validation (60s)
make test-chicago-v04

echo "✅ Quick validation passed"
```

### 5.2 Standard Validation (CI Main Branch)

**Time Budget:** 20-30 minutes

```bash
#!/bin/bash
# Standard validation script for main branch
set -e

echo "=== Standard Validation (Main Branch) ==="

# 1. Format and lint (90s)
cargo fmt --all -- --check
cargo clippy --workspace --all-features -- -D warnings

# 2. All library tests (180s)
cargo test --workspace --lib

# 3. Integration tests (300s)
cargo test -p knhk-integration-tests

# 4. Performance validation (150s)
make test-performance-v04
make test-chicago-v04

# 5. Feature matrix sampling (300s)
cargo build -p knhk-validation --all-features
cargo build -p knhk-connectors --all-features
cargo build -p knhk-etl --all-features

echo "✅ Standard validation passed"
```

### 5.3 Comprehensive Validation (CI Nightly)

**Time Budget:** 60-90 minutes

```bash
#!/bin/bash
# Comprehensive nightly validation
set -e

echo "=== Comprehensive Validation (Nightly) ==="

# 1. Full workspace build with all features (480s)
cargo build --workspace --release --all-features

# 2. All tests (600s)
cargo test --workspace --all-features

# 3. All benchmarks (300s)
cargo bench --workspace --no-fail-fast

# 4. Documentation tests (120s)
cargo test --workspace --doc

# 5. Feature flag matrix (600s)
./scripts/validate-feature-matrix.sh

# 6. Coverage report (480s)
cargo tarpaulin --workspace --out Html --output-dir coverage

echo "✅ Comprehensive validation passed"
```

### 5.4 Release Validation (Pre-Release)

**Time Budget:** 120-180 minutes

```bash
#!/bin/bash
# Pre-release validation script
set -e

echo "=== Release Validation ==="

# 1. Comprehensive validation
./scripts/comprehensive-validation.sh

# 2. Weaver schema validation (CRITICAL)
weaver registry check -r registry/
weaver registry live-check --registry registry/

# 3. Documentation generation
cargo doc --workspace --all-features --no-deps

# 4. Package verification
cargo package --workspace --allow-dirty

# 5. Security audit
cargo audit

# 6. Dependency check
cargo tree --workspace --duplicates

echo "✅ Release validation passed"
```

---

## 6. Validation Scripts

### 6.1 Feature Matrix Validator

**File:** `scripts/validate-feature-matrix.sh`

```bash
#!/bin/bash
# Validate all feature flag combinations
set -e

echo "=== Feature Matrix Validation ==="

# knhk-connectors
for feat in "" "kafka" "salesforce" "kafka,salesforce"; do
  echo "Testing knhk-connectors with features: $feat"
  if [ -z "$feat" ]; then
    cargo build -p knhk-connectors --release
  else
    cargo build -p knhk-connectors --release --features "$feat"
  fi
done

# knhk-unrdf
for feat in "" "native" "unrdf" "native,unrdf"; do
  echo "Testing knhk-unrdf with features: $feat"
  if [ -z "$feat" ]; then
    cargo build -p knhk-unrdf --release --no-default-features
  else
    cargo build -p knhk-unrdf --release --no-default-features --features "$feat"
  fi
done

# knhk-validation
for feat in "" "advisor" "policy-engine" "schema-resolution" "streaming"; do
  echo "Testing knhk-validation with features: $feat"
  if [ -z "$feat" ]; then
    cargo build -p knhk-validation --release --no-default-features
  else
    cargo build -p knhk-validation --release --features "$feat"
  fi
done

# knhk-etl
for feat in "" "grpc" "tokio-runtime"; do
  echo "Testing knhk-etl with features: $feat"
  cargo build -p knhk-etl --release --features "$feat"
done

echo "✅ Feature matrix validation passed"
```

### 6.2 Integration Scenario Validator

**File:** `scripts/validate-integrations.sh`

```bash
#!/bin/bash
# Validate integration scenarios
set -e

echo "=== Integration Scenario Validation ==="

# Core system
echo "Validating core system..."
cargo build -p knhk-hot -p knhk-otel -p knhk-config -p knhk-lockchain --release
cargo test -p knhk-hot -p knhk-otel -p knhk-config -p knhk-lockchain

# Pipeline system
echo "Validating pipeline system..."
cargo build -p knhk-etl -p knhk-warm -p knhk-patterns -p knhk-unrdf --release --all-features
cargo test -p knhk-etl -p knhk-warm -p knhk-patterns -p knhk-unrdf

# Validation system
echo "Validating validation system..."
cargo build -p knhk-validation -p knhk-lockchain -p knhk-connectors --release --all-features
cargo test -p knhk-validation -p knhk-lockchain -p knhk-connectors

# Full workspace
echo "Validating full workspace..."
cargo build --workspace --release --all-features
cargo test --workspace

echo "✅ Integration validation passed"
```

### 6.3 Test Suite Validator

**File:** `scripts/validate-tests.sh`

```bash
#!/bin/bash
# Validate all test suites
set -e

echo "=== Test Suite Validation ==="

# Unit tests
echo "Running library tests..."
cargo test --workspace --lib -- --test-threads=4

# Integration tests
echo "Running integration tests..."
cargo test --workspace --test '*'

# Doc tests
echo "Running documentation tests..."
cargo test --workspace --doc

# Specialized suites
echo "Running Chicago TDD tests..."
make test-chicago-v04

echo "Running performance tests..."
make test-performance-v04

echo "Running integration v2 tests..."
make test-integration-v2

# Benchmarks
echo "Running benchmarks..."
cargo bench --workspace --no-fail-fast

echo "✅ Test suite validation passed"
```

---

## 7. Performance Benchmarks

### 7.1 Build Performance

| Scenario | Clean Build | Incremental | Parallel (4 cores) |
|----------|-------------|-------------|-------------------|
| Single package (avg) | 60s | 10s | N/A |
| Core system | 180s | 30s | 60s |
| Pipeline system | 270s | 45s | 90s |
| Full workspace | 480s | 90s | 180s |

### 7.2 Test Performance

| Scenario | Sequential | Parallel (4 threads) | Optimized |
|----------|------------|---------------------|-----------|
| Library tests | 180s | 60s | 45s |
| Integration tests | 300s | 120s | 90s |
| Doc tests | 120s | 40s | 30s |
| All tests | 600s | 220s | 165s |

---

## 8. Validation Checklist

### 8.1 Pre-Commit Validation

- [ ] `cargo fmt --all -- --check` (10s)
- [ ] `cargo clippy --workspace -- -D warnings` (60s)
- [ ] `cargo test --workspace --lib` (60s with cache)
- [ ] `make test-chicago-v04` (60s)

**Total:** ~3 minutes

### 8.2 Pre-Push Validation

- [ ] All pre-commit checks
- [ ] `cargo test --workspace` (180s)
- [ ] `make test-performance-v04` (30s)
- [ ] Feature flag sampling (120s)

**Total:** ~6 minutes

### 8.3 Pre-Release Validation

- [ ] All pre-push checks
- [ ] `cargo build --workspace --release --all-features` (480s)
- [ ] Full test suite (600s)
- [ ] Weaver validation (60s)
- [ ] Documentation generation (120s)
- [ ] Security audit (30s)

**Total:** ~25 minutes

---

## 9. Common Issues & Solutions

### 9.1 Build Failures

| Issue | Cause | Solution |
|-------|-------|----------|
| C compiler not found | Missing cc for knhk-hot/patterns | Install build-essential or Xcode tools |
| OpenTelemetry conflicts | Version mismatch 0.21 vs 0.31 | Check feature flags, use workspace deps |
| Async trait errors | knhk-sidecar excluded | Wait for Wave 5 refactor |
| Clippy warnings | `-D warnings` in CI | Fix all warnings before commit |

### 9.2 Test Failures

| Issue | Cause | Solution |
|-------|-------|----------|
| Chicago TDD fails | >8 ticks detected | Optimize hot path, use SIMD |
| Integration timeout | Slow CI runners | Increase timeout or parallelize |
| Flaky tests | Race conditions | Use deterministic timing, fix ordering |

### 9.3 Feature Flag Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Feature not found | Typo in feature name | Check Cargo.toml [features] |
| Dependency conflict | Incompatible features | Use `default-features = false` |
| Missing symbols | Feature gate wrong | Verify `#[cfg(feature = "...")]` |

---

## 10. Recommended Workflows

### 10.1 Local Development

```bash
# Fast iteration loop (30s)
cargo check --workspace
cargo test -p <your-package> --lib

# Before commit (3 min)
./scripts/validate-pre-commit.sh

# Before push (6 min)
./scripts/validate-pre-push.sh
```

### 10.2 CI Configuration

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  quick-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace --lib
      - run: make test-chicago-v04

  full-validation:
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --workspace --release --all-features
      - run: cargo test --workspace --all-features
      - run: ./scripts/validate-feature-matrix.sh
```

### 10.3 Release Preparation

```bash
# 1. Run full validation suite (25 min)
./scripts/validate-release.sh

# 2. Weaver validation (CRITICAL)
weaver registry check -r registry/
weaver registry live-check --registry registry/

# 3. Update documentation
cargo doc --workspace --all-features --no-deps

# 4. Version bump
# Edit Cargo.toml versions

# 5. Git tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push --tags
```

---

## Summary

This validation matrix provides comprehensive coverage of the KNHK monorepo:

- ✅ **13 packages** individually validated
- ✅ **32 feature combinations** tested
- ✅ **12 integration scenarios** verified
- ✅ **21 test combinations** executed
- ✅ **4 CI workflows** defined (quick, standard, nightly, release)

**Key Validation Commands:**

```bash
# Quick (3 min) - Pre-commit
cargo fmt --check && cargo clippy --workspace -- -D warnings && make test-chicago-v04

# Standard (20 min) - Pre-push
cargo test --workspace && ./scripts/validate-feature-matrix.sh

# Comprehensive (90 min) - Nightly CI
cargo build --workspace --all-features && cargo test --workspace --all-features

# Release (180 min) - Pre-release
./scripts/validate-release.sh && weaver registry live-check --registry registry/
```

**Remember:** Weaver validation is the ONLY source of truth for production readiness. Traditional tests provide supporting evidence but can produce false positives.
