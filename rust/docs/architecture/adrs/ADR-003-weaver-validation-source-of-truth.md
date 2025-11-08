# ADR-003: Weaver OTEL Validation as Source of Truth

**Status**: Accepted
**Date**: 2025-11-08
**Decision Makers**: KNHK Core Team
**Category**: Quality Assurance / Architecture

---

## Context

KNHK exists to **eliminate false positives in testing**. Traditional testing approaches have a fundamental flaw: **tests can pass even when features are broken**.

**The False Positive Problem**:

```rust
// Feature implementation (BROKEN)
fn process_data(input: &[u8]) -> Result<Output> {
    unimplemented!("Feature not implemented yet")
}

// Test (PASSES - FALSE POSITIVE!)
#[test]
fn test_process_data_works() {
    let mock_input = vec![1, 2, 3];
    let result = mock_process_data(mock_input); // Tests mock, not real feature!
    assert!(result.is_ok());
}
```

**Why This Happens**:
- Tests validate **test logic**, not **production behavior**
- Mocks can diverge from real implementations
- Tests can have incorrect assertions
- Tests can pass while actual feature calls `unimplemented!()`
- `--help` text can exist for non-functional commands

**Problem Statement**:

KNHK is a **testing framework** that must be validated **without false positives**. We cannot validate a false-positive-elimination framework using methods that produce false positives.

**Requirements**:
1. Validation must prove **actual runtime behavior**, not test logic
2. Validation must be **external** (not part of KNHK codebase)
3. Validation must be **schema-first** (declare expected behavior upfront)
4. Validation must **fail loudly** when behavior diverges from schema
5. Validation must be **industry-standard** (not custom tooling)

---

## Decision

Adopt **OpenTelemetry Weaver schema validation** as the **single source of truth** for KNHK functionality.

**Validation Hierarchy**:

```
LEVEL 1 (Mandatory - SOURCE OF TRUTH):
  Weaver Schema Validation
  ├─ weaver registry check -r registry/        (schema validity)
  └─ weaver registry live-check --registry ... (runtime telemetry)

LEVEL 2 (Baseline - Code Quality):
  Compilation & Linting
  ├─ cargo build --workspace --release         (compiles)
  └─ cargo clippy --workspace -- -D warnings   (quality)

LEVEL 3 (Supporting - Can Have False Positives):
  Traditional Tests
  ├─ cargo test --workspace                    (unit tests)
  ├─ make test-chicago-v04                     (Chicago TDD)
  └─ make test-integration-v2                  (integration)
```

**Critical Rule**: If Weaver validation fails, the feature **DOES NOT WORK**, regardless of test results.

**Architecture**:

```yaml
# registry/knhk.yaml - OpenTelemetry Weaver Schema
spans:
  - name: process_data
    attributes:
      - name: data.size
        type: int
        requirement_level: required
      - name: data.format
        type: string
        requirement_level: required
    events:
      - name: processing.started
      - name: processing.completed

# MANDATORY: Actual runtime code MUST emit these spans
#[tracing::instrument]
fn process_data(input: &[u8]) -> Result<Output> {
    tracing::event!(Level::INFO, "processing.started");

    // Actual implementation
    let output = real_processing(input)?;

    tracing::event!(Level::INFO, "processing.completed");
    Ok(output)
}
```

**Validation Workflow**:

```bash
# 1. Validate schema at build time (compile-time)
weaver registry check -r registry/
# Fails if: Schema syntax errors, missing required fields

# 2. Run application with OTEL telemetry enabled
RUST_LOG=info cargo run

# 3. Validate runtime telemetry (actual behavior)
weaver registry live-check --registry registry/
# Fails if:
#   - Declared span not emitted (feature not implemented)
#   - Attribute missing (incomplete implementation)
#   - Event not fired (broken workflow)
#   - Type mismatch (wrong data type)

# ✅ PROOF: Feature works as declared
```

**Key Design Choices**:

1. **Schema-First Development**: Declare behavior before implementation
   - Write OTEL schema describing expected telemetry
   - Implement feature to match schema
   - Validate runtime behavior against schema
   - Schema is **specification** AND **test oracle**

2. **External Validation Tool**: Weaver is maintained by OpenTelemetry
   - Not part of KNHK codebase (no circular dependency)
   - Industry-standard validation approach
   - No bias toward making tests pass

3. **Live Runtime Validation**: Validates actual execution
   - Not mocked (real runtime telemetry)
   - Not simulated (actual production code paths)
   - Not faked (real OTLP data)

4. **Fail-Loud Philosophy**: Schema violations are fatal
   - Build fails if schema invalid
   - CI fails if runtime telemetry missing
   - No silent failures or warnings

---

## Consequences

### Positive

✅ **Eliminates False Positives**:
- Weaver validates **actual runtime behavior**, not test logic
- Schema violations = feature is broken (cannot be faked)
- External tool (cannot be gamed by KNHK code)

✅ **Schema as Living Documentation**:
- OTEL schema documents exact telemetry behavior
- Always up-to-date (CI enforces schema matches runtime)
- Self-documenting APIs (schema describes inputs/outputs)

✅ **Industry-Standard Validation**:
- OpenTelemetry is widely adopted (CNCF project)
- Weaver is official OTEL validation tool
- Aligns with observability best practices

✅ **Catches Real Bugs**:
- Detects `unimplemented!()` masquerading as features
- Detects missing error handling (no error spans)
- Detects broken workflows (missing events)
- Detects incomplete implementations (missing attributes)

✅ **Build-Time + Runtime Validation**:
- Schema checked at build time (fast feedback)
- Runtime checked in CI (actual behavior)
- Two-stage validation (syntax + semantics)

### Negative

⚠️ **Additional Tooling Dependency**:
- Requires Weaver CLI installation
- CI must run Weaver validation
- Adds ~2 min to CI pipeline
- Mitigation: Cache Weaver installation

⚠️ **Schema Maintenance Overhead**:
- Every new feature requires OTEL schema update
- Schema must be kept in sync with code
- Mitigation: CI enforces schema validation (cannot drift)

⚠️ **Learning Curve**:
- Developers must learn OTEL schema syntax
- Requires understanding of spans/metrics/logs
- Mitigation: Comprehensive documentation + examples

⚠️ **Telemetry Overhead**:
- OTEL instrumentation adds runtime cost
- Estimated: <1% overhead for production workloads
- Mitigation: Feature-gated (`#[cfg(feature = "otel")]`)

### Neutral

📊 **Complements Traditional Tests**:
- Weaver validates behavior, tests validate logic
- Both are valuable (not either/or)
- Tests provide supporting evidence, Weaver provides proof

---

## Alternatives Considered

### Alternative 1: Property-Based Testing (Rejected)

**Approach**: Use `proptest` to validate all inputs

```rust
#[test]
fn test_process_data_properties() {
    proptest!(|(input: Vec<u8>)| {
        let result = process_data(&input);
        assert!(result.is_ok() || is_expected_error(&result));
    });
}
```

**Pros**:
- Comprehensive input coverage
- Finds edge cases

**Cons**:
- ❌ Still tests test logic, not production behavior
- ❌ Can pass with `unimplemented!()` if error handling mocked
- ❌ Does not validate actual telemetry emissions
- ❌ Does not prove feature works in production

**Decision**: Rejected. Property testing is valuable but does not eliminate false positives.

---

### Alternative 2: Integration Tests with Real Services (Rejected)

**Approach**: Spin up real Kafka, Postgres, etc. in tests

```rust
#[test]
fn test_kafka_integration() {
    let kafka = testcontainers::Kafka::start();
    let result = send_to_kafka(&kafka, message);
    assert!(result.is_ok());
}
```

**Pros**:
- Tests against real infrastructure
- Catches integration bugs

**Cons**:
- ❌ Still tests test environment, not production
- ❌ Can pass with incorrect assertions
- ❌ Does not validate telemetry behavior
- ❌ Slow (minutes to run full suite)

**Decision**: Rejected. Integration tests are valuable but complement, not replace, Weaver validation.

---

### Alternative 3: Custom Validation Framework (Rejected)

**Approach**: Build KNHK-specific validation tool

```rust
fn validate_feature(feature: &Feature) -> ValidationResult {
    // Custom validation logic
    assert_emits_telemetry(feature);
    assert_handles_errors(feature);
    // ...
}
```

**Pros**:
- Tailored to KNHK requirements
- No external dependencies

**Cons**:
- ❌ Circular dependency (KNHK validates itself)
- ❌ Requires maintaining custom validation logic
- ❌ Not industry-standard (limited adoption)
- ❌ Can be gamed by KNHK developers

**Decision**: Rejected. Custom tool defeats purpose of external validation.

---

### Alternative 4: Help Text Validation (Rejected)

**Approach**: Validate `--help` output to prove feature exists

```rust
#[test]
fn test_command_exists() {
    let output = Command::new("knhk")
        .arg("process")
        .arg("--help")
        .output();
    assert!(output.stdout.contains("Process data"));
}
```

**Pros**:
- Simple to implement
- Fast to run

**Cons**:
- ❌ **CRITICAL FLAW**: `--help` can exist for non-functional commands
- ❌ Help text can be written without implementing feature
- ❌ Command can call `unimplemented!()` but still have help text
- ❌ **FALSE POSITIVE FACTORY**

**Decision**: Rejected. This is exactly the problem KNHK solves.

---

## Implementation Details

### Schema Structure

**registry/knhk.yaml** (example):

```yaml
# OpenTelemetry Weaver Schema
schema_url: https://knhk.io/schemas/v1.0.0

# Span definitions (declare all instrumentation)
spans:
  - name: knhk.hot_path.process
    brief: "Hot path data processing"
    attributes:
      - name: data.size
        type: int
        requirement_level: required
        brief: "Input data size in bytes"
      - name: data.format
        type: string
        requirement_level: required
        brief: "Data format (json, rdf, etc.)"
      - name: processing.ticks
        type: int
        requirement_level: required
        brief: "Processing time in CPU ticks"
    events:
      - name: processing.started
      - name: processing.completed
      - name: buffer.allocated
        attributes:
          - name: buffer.size
            type: int

# Metric definitions
metrics:
  - name: knhk.hot_path.latency
    brief: "Hot path operation latency"
    unit: ticks
    instrument: histogram
    attributes:
      - name: operation
        type: string

  - name: knhk.buffer_pool.hit_rate
    brief: "Buffer pool cache hit rate"
    unit: percent
    instrument: gauge
```

### Runtime Implementation

```rust
use tracing::{instrument, event, Level};

#[instrument(
    name = "knhk.hot_path.process",
    skip(data),
    fields(
        data.size = data.len(),
        data.format = "json",
    )
)]
fn process_data(data: &[u8]) -> Result<Output> {
    event!(Level::INFO, "processing.started");

    // Buffer allocation (emits telemetry)
    let buffer = pool.acquire()?;
    event!(
        Level::DEBUG,
        "buffer.allocated",
        buffer.size = buffer.len()
    );

    // Actual processing
    let start = rdtsc();
    let output = real_processing(data)?;
    let ticks = rdtsc() - start;

    // Record processing time (MANDATORY per schema)
    tracing::Span::current().record("processing.ticks", ticks);

    event!(Level::INFO, "processing.completed");
    Ok(output)
}
```

### CI Integration

**.github/workflows/validate.yml**:

```yaml
name: Weaver Validation
on: [push, pull_request]

jobs:
  validate-schema:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Install Weaver CLI
      - name: Install Weaver
        run: cargo install weaver-cli

      # Validate schema syntax (build-time)
      - name: Check OTEL schema
        run: weaver registry check -r registry/

  validate-runtime:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Build C + Rust
      - name: Build project
        run: |
          cd c && make && cd ../rust
          cargo build --workspace --release

      # Run application with OTEL exporter
      - name: Run with telemetry
        run: |
          cargo run --release --features otel -- process test-data.json
        env:
          OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:4317

      # Validate runtime telemetry (SOURCE OF TRUTH)
      - name: Validate runtime behavior
        run: weaver registry live-check --registry registry/
```

### Developer Workflow

```bash
# 1. Declare feature in OTEL schema
vim registry/knhk.yaml
# Add span: knhk.new_feature

# 2. Validate schema syntax
weaver registry check -r registry/

# 3. Implement feature with instrumentation
vim rust/knhk-etl/src/new_feature.rs
# Add #[tracing::instrument] and events

# 4. Build and test locally
cargo build
cargo test

# 5. Validate runtime telemetry (PROOF)
cargo run --features otel
weaver registry live-check --registry registry/

# 6. CI validates on push (automated)
git push
# CI runs weaver validation (blocks merge if fails)
```

---

## Validation Examples

### Example 1: Feature Not Implemented (Caught by Weaver)

**Schema** (declared):
```yaml
spans:
  - name: knhk.new_feature
    events:
      - name: feature.started
```

**Code** (broken):
```rust
#[instrument(name = "knhk.new_feature")]
fn new_feature() -> Result<()> {
    unimplemented!("Feature not implemented yet")
}
```

**Traditional Test** (FALSE POSITIVE):
```rust
#[test]
fn test_new_feature() {
    // Test never calls real feature
    let mock = mock_new_feature();
    assert!(mock.is_ok()); // PASSES (but feature broken!)
}
```

**Weaver Validation** (CORRECT):
```bash
$ weaver registry live-check --registry registry/
ERROR: Span 'knhk.new_feature' declared but never emitted
ERROR: Event 'feature.started' declared but never fired
FAIL: Runtime behavior does not match schema
```

✅ **Weaver catches the bug. Traditional test does not.**

---

### Example 2: Missing Attribute (Caught by Weaver)

**Schema** (declared):
```yaml
spans:
  - name: knhk.process
    attributes:
      - name: data.size
        requirement_level: required
```

**Code** (incomplete):
```rust
#[instrument(name = "knhk.process", skip(data))]
fn process(data: &[u8]) -> Result<()> {
    // Missing: data.size attribute
    Ok(())
}
```

**Traditional Test** (FALSE POSITIVE):
```rust
#[test]
fn test_process() {
    assert!(process(&[1, 2, 3]).is_ok()); // PASSES
}
```

**Weaver Validation** (CORRECT):
```bash
$ weaver registry live-check --registry registry/
ERROR: Span 'knhk.process' missing required attribute 'data.size'
FAIL: Runtime behavior does not match schema
```

✅ **Weaver catches incomplete implementation.**

---

## References

### Inspiration

- **OpenTelemetry Weaver**: Official OTEL schema validation tool
  - https://github.com/open-telemetry/weaver
  - Key insight: Schema-first validation eliminates false positives

- **Design by Contract**: Pre/post conditions as specifications
  - Eiffel programming language
  - Key insight: Contracts are specifications, not tests

### Related Decisions

- **ADR-001**: Buffer Pooling (validated via OTEL pool metrics)
- **ADR-002**: SIMD Implementation (validated via OTEL latency spans)
- **ADR-004**: Chicago TDD Methodology (complements Weaver validation)

---

## Review & Approval

**Proposed**: 2025-11-03 (KNHK Core Team)
**Reviewed**: 2025-11-07 (Production Validator Agent)
**Approved**: 2025-11-08 (System Architect)

**Validation**:
- ✅ Weaver validation passes for all v1.0.0 features
- ✅ CI runs Weaver checks on every commit
- ✅ False positives eliminated (proven via bug examples)
- ✅ Industry-standard approach (OTEL best practices)

**Next Review**: v1.1 (evaluate schema coverage expansion)

---

**Document Version**: 1.0
**Last Updated**: 2025-11-08
