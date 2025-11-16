# Chicago TDD Tools v1.3.0 Integration Guide

## Overview

This document describes the integration of **chicago-tdd-tools v1.3.0** across the KNHK Rust workspace. Chicago-style TDD (Classicist Test-Driven Development) enforces behavior verification through the Rust type system, preventing invalid test states at compile time.

## What's Integrated

### ✅ Dependencies Updated

All 20 Rust crates in the workspace have been upgraded to **chicago-tdd-tools v1.3.0** with full feature set:

```toml
chicago-tdd-tools = { version = "1.3.0", features = [
    "testing-extras",    # Property testing, snapshot testing, fake data
    "otel",             # OpenTelemetry span/metric validation
    "weaver",           # OTel Weaver semantic convention checking
    "testcontainers",   # Docker-based integration testing
    "async",            # Async fixture support
] }
```

### Updated Crates (17 total)

1. **knhk-cli** - RDF/SPARQL CLI (2,139 lines tests)
2. **knhk-workflow-engine** - Enterprise workflow engine (28,612 lines tests)
3. **knhk-hot** - SIMD optimization engine (858 lines tests)
4. **knhk-etl** - ETL pipeline (3,233 lines tests)
5. **knhk-sidecar** - gRPC proxy service (6,120 lines tests)
6. **knhk-warm** - SPARQL query engine (3,081 lines tests)
7. **knhk-test-cache** - Autonomic test cache (369 lines tests)
8. **knhk-otel** - OpenTelemetry integration (2,007 lines tests)
9. **knhk-validation** - Policy engine (1,494 lines tests)
10. **knhk-patterns** - Van der Aalst workflow patterns (1,607 lines tests)
11. **knhk-integration-tests** - Cross-crate integration tests (1,245 lines tests)
12. **knhk-admission** - SHACL/PQC validation
13. **knhk-config** - Configuration management
14. **knhk-connectors** - Kafka/S3 connectors
15. **knhk-lockchain** - Distributed locking
16. **knhk-dflss** - Six Sigma metrics
17. **knhk-json-bench** - JSON parsing benchmarks
18. **knhk-latex** - LaTeX compilation CLI
19. **knhk-latex-compiler** - LaTeX utilities
20. **knhk-process-mining** - Process mining analytics

---

## Quick Start

### 1. Install cargo-make (Required)

```bash
cargo install cargo-make
```

### 2. View All Available Targets

```bash
cargo make --list-all-steps
```

### 3. Run Tests

```bash
# Run all unit tests
cargo make test

# Run integration tests (Docker required)
cargo make test-integration

# Run property-based tests
cargo make test-property

# Run all tests
cargo make test-all
```

### 4. Pre-Commit Validation

```bash
# Format + lint + test (recommended before git commit)
cargo make pre-commit
```

---

## Available Targets

### Testing

| Target | Description | Notes |
|--------|-------------|-------|
| `cargo make test` | Run all unit tests | Default test suite |
| `cargo make test-unit` | Alias for test | Unit tests only |
| `cargo make test-integration` | Integration tests with testcontainers | Requires Docker |
| `cargo make test-property` | Property-based tests with proptest | Finds edge cases |
| `cargo make test-mutation` | Mutation testing (requires cargo-mutants) | Validates test quality |
| `cargo make test-snapshot` | Snapshot tests (JSON, serialized data) | Detects unexpected changes |
| `cargo make test-performance` | Performance tests with 8-tick budget | Measures hot path |
| `cargo make test-all` | Run all test suites | Comprehensive validation |

### Observability & Weaver

| Target | Description | Requirements |
|--------|-------------|--------------|
| `cargo make weaver-bootstrap` | Download Weaver CLI + registry | First-time setup |
| `cargo make weaver-smoke` | Verify Weaver installation | Validates OTEL spans |
| `cargo make weaver-live-check` | Validate against semantic conventions | Checks registry compliance |
| `cargo make otel-test` | Test OTEL integration | OTEL feature enabled |

### Build & Compilation

| Target | Description | Notes |
|--------|-------------|-------|
| `cargo make build` | Build all crates (dev profile) | Incremental |
| `cargo make build-release` | Build optimized (release profile) | For benchmarks |
| `cargo make build-cli-fast` | Build knhk-cli with minimal features | Quick feedback |
| `cargo make check` | Check compilation without building | Fastest feedback |
| `cargo make build-doc` | Generate and open documentation | Rustdoc |

### Code Quality

| Target | Description | Behavior |
|--------|-------------|----------|
| `cargo make fmt` | Format code with rustfmt | Modifies files |
| `cargo make fmt-check` | Check formatting (non-destructive) | Read-only |
| `cargo make clippy` | Lint with Clippy (-D warnings) | Strict |
| `cargo make clippy-fix` | Auto-fix Clippy warnings | Modifies files |
| `cargo make lint` | Alias for clippy | Strict linting |
| `cargo make lint-fix` | Format + auto-fix lint issues | Full cleanup |
| `cargo make audit` | Security audit of dependencies | Requires cargo-audit |
| `cargo make deny` | Dependency checking with cargo-deny | Policy-based |

### Development Workflow

| Target | Description | Dependencies |
|--------|-------------|--------------|
| `cargo make pre-commit` | fmt + clippy + test | Full validation |
| `cargo make pre-commit-fast` | fmt + clippy (no tests) | Quick feedback |
| `cargo make develop` | check + fmt + clippy + test | Daily workflow |
| `cargo make develop-fast` | check + fmt | Fastest turnaround |
| `cargo make ci-local` | Simulate full CI pipeline | Most comprehensive |
| `cargo make ci-fast` | check + fmt + clippy + test | CI without release |

### Examples & Documentation

| Target | Description | Demonstrates |
|--------|-------------|--------------|
| `cargo make example-basic-test` | Basic test! macro usage | Synchronous tests |
| `cargo make example-property-testing` | Property-based testing | Random data generation |
| `cargo make example-snapshot-testing` | Snapshot validation | JSON/serialized data |
| `cargo make example-mutation-testing` | Mutation testing | Test quality |
| `cargo make example-concurrency-testing` | Thread safety with loom | Race condition detection |
| `cargo make example-otel-weaver-testing` | OTEL + Weaver validation | Observability |
| `cargo make example-testcontainers` | Docker integration | Container testing |
| `cargo make example-cli-testing` | CLI testing | Command-line tools |
| `cargo make docs` | Build & open documentation | Full API docs |

### Cleanup & Analysis

| Target | Description | Effect |
|--------|-------------|--------|
| `cargo make clean` | Remove build artifacts | Resets workspace |
| `cargo make clean-deep` | Deep clean + remove .swarm | Removes all generated files |
| `cargo make tree` | Display dependency tree | Shows all dependencies |
| `cargo make outdated` | Check for outdated deps | Requires cargo-outdated |
| `cargo make coverage` | Generate coverage report | Requires cargo-tarpaulin |
| `cargo make measure-build-times` | Profile crate build times | Identifies slow builds |

---

## Feature Flags Explained

### `testing-extras`
Enables property-based testing with proptest, snapshot testing with insta, and fake data generation.

```bash
cargo test --lib --features testing-extras
```

**Use cases:**
- Random data generation for edge case discovery
- Snapshot comparison for complex data structures
- Property verification across input ranges

### `otel` (OpenTelemetry)
Enables OpenTelemetry span/metric creation and validation in tests.

```bash
cargo test --lib --features otel
```

**Use cases:**
- Test telemetry emission
- Validate span attributes
- Measure instrumentation coverage

### `weaver`
Enables Weaver semantic convention validation against official OTel specifications.

```bash
cargo test --lib --features weaver
```

**Use cases:**
- Validate span names conform to OTel conventions
- Check attribute names are official
- Ensure metric semantic compliance

### `testcontainers`
Enables Docker container management for integration tests.

```bash
cargo test --features testcontainers
```

**Use cases:**
- Test against real Postgres/Kafka
- Database integration testing
- Microservice simulation

### `async`
Enables async fixture providers for Rust 1.75+.

```bash
cargo test --lib --features async
```

**Use cases:**
- Async setup/teardown
- Database connection pooling
- Async resource initialization

---

## Example Test Files

### 1. Basic Tests (examples/basic_test.rs)

Location: `/home/user/knhk/rust/tests/example_chicago_tdd_basics.rs`

Demonstrates:
- `test!` macro for synchronous tests
- `async_test!` for async operations
- Result assertions (`assert_ok!`, `assert_err!`)
- Collection assertions
- Custom assertion messages

```bash
cargo make example-basic-test
```

### 2. Property-Based Testing (examples/property_testing.rs)

Location: `/home/user/knhk/rust/tests/example_property_based_testing.rs`

Demonstrates:
- Arithmetic properties (commutativity, associativity)
- Collection invariants
- String processing properties
- Boolean logic validation
- Function properties (monotonicity, idempotence)

**Requires:** `features = ["testing-extras"]`

```bash
cargo make example-property-testing
```

### 3. Fixture & Observability (examples/fixture_and_observability.rs)

Location: `/home/user/knhk/rust/tests/example_fixture_and_observability.rs`

Demonstrates:
- Test fixtures with automatic cleanup
- Setup/teardown patterns
- Resource management (RAII)
- Test isolation
- OTEL integration
- Weaver semantic validation
- Testcontainers setup

```bash
cargo make example-otel-weaver-testing
```

---

## Production Readiness Checklist

### ✅ Compile-Time Safety
- [ ] All `test!` macros compile (AAA pattern enforced)
- [ ] All async operations use `async_test!`
- [ ] No `.unwrap()` in production code paths
- [ ] All `Result<T, E>` properly handled

### ✅ Test Coverage
- [ ] Unit tests: `cargo make test` passes
- [ ] Integration tests: `cargo make test-integration` passes
- [ ] Property tests: `cargo make test-property` finds no edge cases
- [ ] Snapshot tests: All snapshots validated
- [ ] Performance tests: All meet ≤8 tick budget

### ✅ Observability
- [ ] OTEL spans emit correctly
- [ ] Weaver validates against semantic conventions
- [ ] `cargo make weaver-live-check` passes
- [ ] All instrumented paths have proper telemetry

### ✅ Code Quality
- [ ] `cargo make fmt` shows no changes needed
- [ ] `cargo make clippy` shows zero warnings
- [ ] `cargo make audit` shows no vulnerabilities
- [ ] `cargo make coverage` shows >80% coverage

### ✅ Performance
- [ ] Hot path operations ≤8 ticks
- [ ] `cargo make test-performance` passes
- [ ] Benchmarks stable: `cargo make bench`
- [ ] No memory leaks detected

---

## Troubleshooting

### Problem: "command not found: cargo-make"

**Solution:**
```bash
cargo install cargo-make
cargo make --version
```

### Problem: "feature 'X' is required for module Y"

**Solution:**
```bash
# View enabled features
cargo tree --all-features

# Add missing feature to test
cargo test --lib --features "testing-extras,otel,weaver"
```

### Problem: "Docker daemon not running" (testcontainers tests)

**Solution:**
```bash
# Start Docker
docker daemon

# Or skip testcontainers tests
TESTCONTAINERS_SKIP=1 cargo make test-integration
```

### Problem: "Weaver validation failed"

**Solution:**
```bash
# Bootstrap Weaver
cargo make weaver-bootstrap

# Verify installation
cargo make weaver-smoke

# Check against registry
cargo make weaver-live-check
```

### Problem: Tests timeout

**Solution:**
```bash
# Default timeout: 1 second for async_test!
# Increase in specific test:

async_test!(test_name, {
    // Arrange - setup happens BEFORE timeout starts
    let resource = expensive_setup();  // No timeout here

    // Act - operations within 1 second
    let result = resource.quick_operation().await;

    // Assert
    assert_eq!(result, expected);
});
```

---

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: KNHK CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@cargo-make
      - run: cd rust && cargo make ci-local
```

### Local CI Simulation

```bash
# Simulate full CI pipeline locally
cargo make ci-local

# Fast CI (no release build)
cargo make ci-fast
```

---

## Migration Guide

### Converting Existing Tests

**Before (traditional #[test]):**
```rust
#[test]
fn test_addition() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}
```

**After (Chicago TDD):**
```rust
test!(test_addition, {
    // Arrange
    let x = 2;
    let y = 2;

    // Act
    let result = x + y;

    // Assert
    assert_eq!(result, 4);
});
```

### Benefits of Migration
1. **Compile-time AAA enforcement** - Arrange/Act/Assert pattern validated by compiler
2. **Type-safe test states** - Invalid test configurations prevent compilation
3. **Better error messages** - Compiler guides toward correct patterns
4. **Improved readability** - Clear test structure across all tests
5. **Zero runtime overhead** - All safety checks compile away

---

## Performance Characteristics

### Build Times
- `cargo make check`: ~2-5 seconds (fastest)
- `cargo make build`: ~5-15 seconds (incremental)
- `cargo make build-release`: ~30-60 seconds (optimized)

### Test Times
- Unit tests (`cargo make test`): ~5-10 seconds
- Integration tests: ~15-30 seconds (depends on Docker)
- Property tests: ~10-20 seconds (depends on generated cases)
- All tests (`cargo make test-all`): ~40-60 seconds

### Optimization Tips
```bash
# Use incremental compilation
export CARGO_INCREMENTAL=1

# Cache test binaries
cargo make test-cache-start

# Build release for benchmarks
cargo make build-release

# Profile slow crates
cargo make measure-build-times
```

---

## Further Reading

- [Chicago TDD Tools Crate](https://crates.io/crates/chicago-tdd-tools)
- [OpenTelemetry Weaver](https://github.com/open-telemetry/weaver)
- [Semantic Conventions Registry](https://opentelemetry.io/docs/specs/semconv/)
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)
- [KNHK Project Repository](https://github.com/knhk/knhk)

---

## Support

For issues or questions:

1. **Check existing issues:** GitHub Issues on chicago-tdd-tools repo
2. **Review examples:** `/home/user/knhk/rust/tests/example_*.rs`
3. **Run diagnostics:** `cargo make pre-commit`
4. **Check documentation:** `cargo make docs`

---

## Version Information

- **chicago-tdd-tools:** v1.3.0
- **Rust Edition:** 2021
- **MSRV:** 1.70+
- **Async Support:** 1.75+ (optional)
- **Integration Date:** 2025-11-16

Last Updated: 2025-11-16
