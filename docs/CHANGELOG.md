# Changelog

All notable changes to KNHK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-11-07

### Added

#### Core Architecture
- **8-Beat Epoch System**: Implemented Chicago-style TDD tick-based scheduling with ≤8 tick hot path guarantee
- **Fiber/Ring Buffer Integration**: Beat system integrated with ETL pipeline for deterministic execution
- **OpenTelemetry Weaver Schema Validation**: Schema-first approach with `weaver registry check` and `weaver registry live-check`
- **DFLSS Workflows**: Definition of Done (DoD) validation system and evidence-based documentation structure
- **Production Validation System**: Comprehensive v1.0 DoD validation with Chicago TDD tests

#### ETL & Data Processing
- **Policy Engine Integration**: Weaver-inspired OPA/Regorus integration for runtime policy validation
- **Error Diagnostics System**: Miette-based rich error reporting with detailed diagnostics
- **Streaming Ingester**: High-performance streaming data ingestion with backpressure handling
- **Failure Actions**: Configurable failure handling with retry, dead-letter queue, and circuit breaker patterns
- **Beat Scheduler**: Tick-based execution scheduler coordinating fiber lifecycle
- **Reconcile Module**: State reconciliation with CRDT-like conflict resolution
- **Transform Module**: Data transformation pipeline with validation

#### Observability & Telemetry
- **Runtime Class Support**: Performance tier classification (Hot/Warm/Cold)
- **SLO Monitoring**: Service Level Objective tracking and alerting
- **Comprehensive Tracing**: Full OpenTelemetry span/metric/log coverage
- **KGC Sidecar**: Kubernetes-native sidecar for knowledge graph operations
- **AOT Specialization**: Ahead-of-time compilation optimization for hot paths

#### Infrastructure
- **Lockchain Integration**: Blockchain-inspired immutable audit trail with Git2 backend
- **TLS Certificate Loading**: Production-ready certificate management (fixed critical blocker)
- **Kubernetes Deployment**: KGC Sidecar service with deployment manifests
- **Testcontainers Backend**: Docker-based integration testing with Kafka and PostgreSQL
- **C Library Build**: Static/dynamic library compilation for C interop

#### Code Quality & Validation
- **Bulk Unwrap/Expect Remediation**: WAVE 4 implementation removing unsafe unwraps
- **Error Handling Best Practices**: Comprehensive `Result<T, E>` usage across codebase
- **Clippy Zero Warnings**: `cargo clippy --workspace -- -D warnings` compliance
- **Weaver Schema Compliance**: All telemetry validated against OTel schemas
- **Chicago TDD Test Suite**: Behavior-focused testing with AAA pattern

### Changed

#### Breaking Changes
- **Version Bump**: All crates updated from 0.1.0 to 1.0.0
- **clap-noun-verb**: Upgraded to v3.4.0 (fixed compilation errors)
- **Dependency Reorganization**: Removed circular dependencies (knhk-etl removed from knhk-validation)
- **Error Types**: Consolidated error handling with `thiserror` 2.0

#### Improvements
- **Documentation Consolidation**: 80/20 focus - archived status docs, consolidated Weaver documentation
- **README Updates**: Integrated 8-beat epoch system, Weaver insights, and recent improvements
- **Module Organization**: Better separation of concerns (ETL, OTEL, Connectors, Validation)
- **Performance Optimization**: Hot path operations verified ≤8 ticks (Chatman Constant compliance)

### Fixed

#### Critical Blockers
- **TLS Certificate Loading**: Fixed certificate file reading and parsing
- **Circular Dependency**: Broke knhk-etl ↔ knhk-validation cycle
- **Lockchain Compilation Errors**: Fixed unused variable warnings and module resolution
- **Chicago TDD Test Failures**: Resolved all test failures and compilation errors
- **Module Resolution**: Fixed ETL emit.rs and connector implementations

#### Build & Compilation
- **MSVC Compatibility**: Improved preload.h portability for MSVC and other architectures
- **False Positives**: Eliminated fake-green test results
- **Unfinished Work**: Removed placeholder implementations, added `unimplemented!()` markers
- **Duplicate Dependencies**: Removed duplicate serde in knhk-validation

#### Performance
- **Performance Compliance**: Verified hot path ≤8 ticks via benchmarks
- **Memory Management**: Fixed memory leaks in fiber ring buffer
- **Tick Budget Enforcement**: Beat scheduler now properly enforces tick limits

### Security
- **Certificate Management**: Production-ready TLS certificate loading and validation
- **Audit Trail**: Lockchain provides immutable audit log for all operations
- **Policy Enforcement**: OPA/Regorus integration for runtime security policies

### Documentation
- **DFLSS Structure**: Evidence-based documentation with workflow definitions
- **Weaver Documentation**: Consolidated schema validation and telemetry guides
- **C4 Architecture**: System architecture diagrams and component documentation
- **Hooks Engine**: Comprehensive documentation for pre/post operation hooks
- **API Documentation**: Updated Quick Start examples to match actual APIs
- **Performance Reports**: Accurate performance compliance documentation

### Known Issues
- **URDNA2015 Canonicalization**: Full RDF canonicalization deferred to v1.1 (basic sorting + normalization in v1.0)
- **Async Trait Methods**: Not used to maintain `dyn` compatibility
- **Testcontainer Version Alignment**: Some testcontainers modules may require version updates

### Migration Guide

#### From 0.1.0 to 1.0.0

**Dependency Updates**:
```toml
# Before
knhk-etl = { version = "0.1.0" }
knhk-otel = { version = "0.1.0" }

# After
knhk-etl = { version = "1.0.0" }
knhk-otel = { version = "1.0.0" }
```

**Error Handling**:
```rust
// Before (may panic)
let value = some_result.unwrap();

// After (safe error handling)
let value = some_result.map_err(|e| Error::OperationFailed(e))?;
```

**Weaver Validation**:
```bash
# New mandatory validation steps
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

**Chicago TDD Testing**:
```bash
# New test suite
make test-chicago-v04
make test-performance-v04  # Verifies ≤8 ticks
```

### Contributors
- Sean Chatman (@seanchatmangpt)

---

## [0.1.0] - 2025-10-01 (Pre-release)

Initial development release. See git history for details.

[1.0.0]: https://github.com/seanchatmangpt/knhk/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/seanchatmangpt/knhk/releases/tag/v0.1.0
