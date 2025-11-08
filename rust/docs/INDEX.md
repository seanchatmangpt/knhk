# KNHK Documentation Index

**Version:** 1.0.0 | **Updated:** 2025-11-07

## 📊 Build & Validation

### Quick Start
- **[VALIDATION_QUICK_REFERENCE.md](VALIDATION_QUICK_REFERENCE.md)** ⭐ **Start here** - One-page command reference
- **[VALIDATION_MATRIX_SUMMARY.txt](VALIDATION_MATRIX_SUMMARY.txt)** - ASCII art summary (terminal-friendly)

### Comprehensive Guides
- **[BUILD_VALIDATION_MATRIX.md](BUILD_VALIDATION_MATRIX.md)** - Complete 78-scenario validation matrix
  - 13 individual package builds
  - 32 feature flag combinations
  - 12 integration scenarios
  - 21 test combinations
  - 4 CI/CD workflows

### Validation Scripts
- **[../scripts/README.md](../scripts/README.md)** - Script documentation
- `scripts/validate-pre-commit.sh` - Pre-commit checks (~3 min)
- `scripts/validate-pre-push.sh` - Pre-push checks (~6 min)
- `scripts/validate-feature-matrix.sh` - Feature validation (~10 min)
- `scripts/validate-integrations.sh` - Integration tests (~15 min)
- `scripts/validate-tests.sh` - All test suites (~20 min)
- `scripts/validate-release.sh` - Pre-release validation (~25 min)

## 🏗️ Architecture

### Core Documentation
- **[CHICAGO_TDD.md](CHICAGO_TDD.md)** - Chicago-style TDD methodology
- **[deprecated-apis.md](deprecated-apis.md)** - API deprecation tracking
- **[DOCUMENTATION_ORGANIZATION.md](DOCUMENTATION_ORGANIZATION.md)** - Doc structure guidelines
- **[DOCUMENTATION_POLICY.md](DOCUMENTATION_POLICY.md)** - Documentation standards

### Architecture Specifications
- **[architecture/](architecture/)** - System architecture documentation
- **[byteflow_hot_warm_cold_patterns.md](../docs/byteflow_hot_warm_cold_patterns.md)** - Performance tiers
- **[byteflow_nanosecond_optimizations.md](../docs/byteflow_nanosecond_optimizations.md)** - Optimization guide
- **[content_addressing.md](../docs/content_addressing.md)** - Content-addressable patterns

## 📋 Release Management

### Current Release
- **[RELEASE_NOTES_v1.0.0.md](RELEASE_NOTES_v1.0.0.md)** - Version 1.0.0 release notes
- **[CHANGELOG.md](CHANGELOG.md)** - Complete change history
- **[code-quality-analysis-v1.0.0.md](code-quality-analysis-v1.0.0.md)** - Quality metrics

### Evidence & Reports
- **[evidence/](evidence/)** - Production readiness evidence
  - `PRODUCTION_READINESS_VALIDATION_2025-11-07.md` - Latest validation
  - `V1_RELEASE_READINESS_REPORT.md` - Release certification
  - `PERFORMANCE_VALIDATION_V1.md` - Performance benchmarks
  - `PERFORMANCE_METRICS_SUMMARY.md` - Metrics dashboard
  - `CRITICAL_BLOCKERS_REMEDIATION_PLAN.md` - Risk mitigation

## 🧪 Testing & Quality

### Test Documentation
- **[FALSE_POSITIVES_RESOLVED.md](FALSE_POSITIVES_RESOLVED.md)** - False positive elimination
- **[production-validation-80-20.md](production-validation-80-20.md)** - 80/20 validation approach

### Test Execution
```bash
# Quick validation
./scripts/validate-pre-commit.sh

# Full validation
./scripts/validate-release.sh

# Weaver validation (source of truth)
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

## 📦 Package Structure

### Active Packages (13)

**Core System (4)**
- `knhk-hot` - C/Rust FFI, workflow patterns
- `knhk-otel` - OpenTelemetry integration (v0.31)
- `knhk-config` - Configuration management
- `knhk-lockchain` - Merkle chain, quorum consensus

**Data Processing (4)**
- `knhk-etl` - ETL pipeline, beats, fibers
- `knhk-warm` - Cache layer, query optimization
- `knhk-unrdf` - RDF processing, native hooks
- `knhk-patterns` - Van der Aalst workflow patterns

**Integration (3)**
- `knhk-connectors` - Kafka, Salesforce integration
- `knhk-validation` - Schema validation, policy engine
- `knhk-aot` - Ahead-of-time optimization

**User-Facing (2)**
- `knhk-cli` - Command-line interface
- `knhk-integration-tests` - End-to-end tests

### Excluded (1)
- `knhk-sidecar` - ❌ 53 async trait errors (Wave 5)

## 🗃️ Archive

- **[archived/](archived/)** - Historical documentation
  - `planning/` - Old planning documents
  - `product/` - Old product specs
  - `status-reports/` - Historical status updates
  - `weaver-docs/` - Legacy Weaver documentation

## 🚀 Quick Commands

### Development
```bash
# Fast iteration
cargo check --workspace
cargo test -p <package> --lib

# Before commit
./scripts/validate-pre-commit.sh

# Before push
./scripts/validate-pre-push.sh
```

### CI/CD
```bash
# PR validation
cargo fmt --check && cargo clippy --workspace -- -D warnings

# Main branch
./scripts/validate-pre-push.sh

# Nightly
./scripts/validate-release.sh

# Pre-release
./scripts/validate-release.sh && weaver registry live-check --registry registry/
```

### Feature Testing
```bash
# Test all features for a package
cargo build -p <package> --all-features
cargo test -p <package> --all-features

# Test specific feature combination
cargo build -p knhk-connectors --features kafka,salesforce
cargo test -p knhk-etl --features grpc,tokio-runtime
```

## 📊 Validation Hierarchy

### Level 1: Weaver Schema Validation (SOURCE OF TRUTH)
```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

### Level 2: Compilation & Code Quality (BASELINE)
```bash
cargo build --release
cargo clippy --workspace -- -D warnings
```

### Level 3: Traditional Tests (SUPPORTING EVIDENCE)
```bash
cargo test --workspace
make test-chicago-v04
make test-performance-v04
```

**⚠️ CRITICAL:** Only Weaver validation proves features work. Traditional tests can produce false positives.

## 🔗 External References

### Main Project
- **[Root CLAUDE.md](../../CLAUDE.md)** - Project-wide configuration
- **[Root README.md](../../README.md)** - Project overview

### Package READMEs
- [knhk-otel/README.md](../knhk-otel/README.md) - Telemetry documentation
- [knhk-unrdf/README.md](../knhk-unrdf/README.md) - RDF processing guide

## 🆘 Troubleshooting

### Build Issues
See [BUILD_VALIDATION_MATRIX.md § 9](BUILD_VALIDATION_MATRIX.md#9-common-issues--solutions)

### Test Failures
See [VALIDATION_QUICK_REFERENCE.md § Troubleshooting](VALIDATION_QUICK_REFERENCE.md#-troubleshooting)

### CI/CD Problems
See [scripts/README.md § Troubleshooting](../scripts/README.md#troubleshooting)

## 📝 Contributing

1. Read [DOCUMENTATION_POLICY.md](DOCUMENTATION_POLICY.md)
2. Run `./scripts/validate-pre-commit.sh` before committing
3. Run `./scripts/validate-pre-push.sh` before pushing
4. Ensure all tests pass: `cargo test --workspace`
5. Check Weaver validation for production changes

## 📅 Last Updated

- **Documentation Index:** 2025-11-07
- **Build Matrix:** 2025-11-07
- **Validation Scripts:** 2025-11-07
- **Release:** v1.0.0

---

**Quick Navigation:**
[Build Matrix](BUILD_VALIDATION_MATRIX.md) |
[Quick Reference](VALIDATION_QUICK_REFERENCE.md) |
[Scripts](../scripts/README.md) |
[Architecture](architecture/) |
[Evidence](evidence/) |
[Archive](archived/)
