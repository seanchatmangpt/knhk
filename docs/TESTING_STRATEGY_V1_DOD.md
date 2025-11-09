# KNHK v1.0 Testing Strategy & Definition of Done

## Executive Summary

This document defines the **testing hierarchy**, **quality gates**, and **validation standards** for KNHK v1.0 Definition of Done. KNHK eliminates false positives in testing, therefore our validation methodology must reflect this principle.

**Core Principle**: Tests can lie, schemas don't. Weaver validation is the source of truth.

---

## Testing Hierarchy (CRITICAL)

### Level 1: Weaver Schema Validation (MANDATORY - Source of Truth)

**Status**: P0 - REQUIRED FOR ALL FEATURES

**Why This Is Level 1**:
- Schema-first validation proves runtime behavior
- External tool eliminates circular dependency
- Industry standard (OpenTelemetry official validation)
- Detects "fake-green" tests that pass without validating actual behavior
- Cannot pass unless runtime telemetry matches declared schema

**Required Checks**:

```bash
# Schema Definition Validation
weaver registry check -r registry/

# Runtime Telemetry Validation
weaver registry live-check --registry registry/
```

**Quality Gates**:
- [ ] All schemas syntactically valid
- [ ] All schemas semantically correct
- [ ] Runtime telemetry matches schema declarations
- [ ] All spans/metrics/logs documented in schema
- [ ] No undocumented telemetry emissions

**Coverage Requirements**:
- **100% of claimed features** must have schema definitions
- **100% of runtime telemetry** must conform to schemas
- **Zero tolerance** for schema validation failures

**Integration Points**:
- Pre-commit: `weaver registry check -r registry/`
- CI/CD: Both `check` and `live-check` must pass
- Release gates: Full Weaver validation required

---

### Level 2: Compilation & Code Quality (Baseline)

**Status**: P0 - MINIMUM QUALITY THRESHOLD

**Why This Is Level 2**:
- Proves code is valid and follows best practices
- Catches common errors before runtime
- Establishes baseline code quality

**Required Checks**:

```bash
# Rust Compilation
cargo build --workspace --release

# Rust Linting (ZERO warnings tolerance)
cargo clippy --workspace -- -D warnings

# C Compilation
make build

# Code Formatting
cargo fmt --all -- --check
```

**Quality Gates**:
- [ ] Zero compilation errors
- [ ] Zero Clippy warnings
- [ ] All code formatted consistently
- [ ] No `.unwrap()` or `.expect()` in production paths
- [ ] All traits `dyn` compatible (no async trait methods)
- [ ] Proper `Result<T, E>` error handling
- [ ] No `println!` in production code (use `tracing`)
- [ ] No fake `Ok(())` returns from incomplete implementations

**Code Quality Metrics**:
- **Cyclomatic Complexity**: ≤15 per function
- **File Size**: ≤500 lines (modular design)
- **Function Size**: ≤50 lines (Chicago TDD principle)
- **Nesting Depth**: ≤4 levels

---

### Level 3: Traditional Tests (Supporting Evidence)

**Status**: P0 - COMPREHENSIVE COVERAGE REQUIRED

**Why This Is Level 3**:
- Can have false positives (tests that pass but don't prove feature works)
- Provides supporting evidence, not proof
- Useful for regression detection and documentation

**Test Categories**:

#### 3.1 Chicago TDD Tests (Behavior-Focused)

**Location**: `tests/chicago_tdd_*.rs`

**Principles**:
- Test **what** code does, not **how** it does it
- Use real objects (no mocks unless testing interactions)
- Focus on observable behavior
- AAA pattern (Arrange, Act, Assert)

**Quality Standards**:
```rust
// ✅ GOOD: Tests behavior
#[test]
fn sequence_executes_branches_in_order() {
    // Arrange
    let pattern = SequencePattern::new(branches);
    let input = TestData::new(5);

    // Act
    let results = pattern.execute(input).unwrap();

    // Assert
    assert_eq!(results[0].value, 20); // Verifies output
}

// ❌ BAD: Tests implementation
#[test]
fn sequence_calls_execute_on_each_branch() {
    let mock_branch = MockBranch::new();
    // Tests internal calls, not behavior
}
```

**Coverage Requirements**:
- **Critical paths**: 100% behavior coverage
- **Edge cases**: All error conditions tested
- **Integration points**: All interfaces validated
- **Performance paths**: ≤8 ticks verified

**Test Suite Targets**:
```bash
make test-chicago-v04     # Chicago TDD test suite
cargo test chicago_tdd    # Rust Chicago TDD tests
```

---

#### 3.2 Performance Tests (τ ≤ 8 Validation)

**Location**: `tests/performance_*.rs`, `c/tests/performance/`

**Constraints**:
- **Hot path operations**: ≤8 ticks (Chatman Constant)
- **Cold path operations**: No strict limit (but document)
- **Memory allocation**: Zero allocations in hot path

**Validation Methodology**:
```rust
#[test]
fn hot_path_meets_8_tick_constraint() {
    let start = Instant::now();

    // Execute hot path operation
    let result = hot_path_operation(data);

    let duration = start.elapsed();
    let ticks = duration.as_nanos() / TICK_DURATION;

    assert!(ticks <= 8, "Hot path took {} ticks (max 8)", ticks);
}
```

**Performance Benchmarks**:
- Ring buffer operations: ≤8 ticks
- Beat scheduler: ≤8 ticks
- Memory reuse: ≤8 ticks
- Hook registry lookup: ≤8 ticks

**Test Execution**:
```bash
make test-performance-v04
cargo test --release performance
```

---

#### 3.3 Integration Tests (End-to-End Validation)

**Location**: `tests/integration/`

**Scope**:
- Multi-component workflows
- C ↔ Rust FFI boundaries
- OpenTelemetry instrumentation
- Data Gateway queries
- XES event log export

**Quality Standards**:
```rust
#[test]
fn e2e_workflow_execution_with_telemetry() {
    // Arrange: Initialize full system
    let engine = WorkflowEngine::new();
    let otel_subscriber = init_otel_tracing();

    // Act: Execute workflow
    let result = engine.execute_workflow(workflow_spec);

    // Assert: Verify behavior AND telemetry
    assert!(result.is_ok());
    assert_telemetry_emitted("workflow.start");
    assert_telemetry_emitted("workflow.complete");
}
```

**Integration Test Targets**:
```bash
make test-integration-v2
cargo test --test integration
```

---

#### 3.4 Unit Tests (Component Validation)

**Scope**: Individual functions and modules

**Coverage Requirements**:
- **Public APIs**: 100% coverage
- **Error paths**: All error conditions tested
- **Edge cases**: Boundary conditions validated

**Quality Standards**:
- Descriptive test names (not `test1`, `test2`)
- AAA pattern strictly followed
- No test interdependencies
- Fast execution (< 100ms per test)

---

## Functional Validation Standards

### Command Execution Validation (CRITICAL)

**The False Positive Problem**:
```bash
# ❌ FALSE POSITIVE: Help text exists, but command may be broken
knhk --help        # Returns help text
# Conclusion: "command works" ← WRONG!

# ✅ CORRECT VALIDATION: Execute with real arguments
knhk <command> <args>  # Actually run the command
# Verify: Does it produce expected output?
# Verify: Does it emit proper telemetry?
# Verify: Does Weaver validation pass?
```

**Validation Protocol**:
1. **Execute command** with realistic arguments
2. **Verify output** matches expected behavior
3. **Verify telemetry** is emitted correctly
4. **Run Weaver validation** on telemetry
5. **Check exit codes** and error handling

**Example**:
```bash
# Test XES export command
knhk workflow export --format xes workflow.yaml > output.xes

# Validate:
# 1. Exit code 0
# 2. output.xes is valid XES XML
# 3. Telemetry emitted: xes.export.start, xes.export.complete
# 4. Weaver validation passes
```

---

## Test Quality Gates

### Gate 1: Code Quality (Baseline)

**Automated Checks**:
```bash
cargo build --workspace --release
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
make build
```

**Manual Review**:
- No `unwrap()`/`expect()` in production code
- Proper error handling
- Modular design (files ≤500 lines)

**Gate Result**: PASS/FAIL

---

### Gate 2: Test Coverage (Supporting Evidence)

**Metrics**:
- Chicago TDD tests: 100% of critical paths
- Performance tests: All hot paths validated
- Integration tests: All features end-to-end tested
- Unit tests: 85%+ line coverage

**Execution**:
```bash
make test-all
# Includes:
# - cargo test --workspace
# - make test-chicago-v04
# - make test-performance-v04
# - make test-integration-v2
```

**Gate Result**: PASS/FAIL

---

### Gate 3: Weaver Validation (Source of Truth)

**Schema Validation**:
```bash
weaver registry check -r registry/
```

**Live Validation**:
```bash
weaver registry live-check --registry registry/
```

**Requirements**:
- All schemas valid
- All runtime telemetry conforms
- No undocumented telemetry
- No schema violations

**Gate Result**: PASS/FAIL (BLOCKING)

---

## Test Coverage Requirements

### Coverage by Component

| Component | Chicago TDD | Performance | Integration | Unit | Weaver |
|-----------|-------------|-------------|-------------|------|--------|
| **knhk-cli** | 100% commands | N/A | 100% workflows | 85% | 100% |
| **knhk-etl** | 100% paths | ≤8 ticks | 100% FFI | 90% | 100% |
| **knhk-workflow-engine** | 100% patterns | ≤8 ticks | 100% XES | 90% | 100% |
| **knhk-warm** | 100% hooks | ≤8 ticks | 100% lifecycle | 90% | 100% |
| **knhk-hot** | 100% ops | ≤8 ticks | 100% FFI | 90% | 100% |
| **knhk-patterns** | 100% patterns | N/A | 100% workflows | 90% | 100% |
| **C Library** | 100% API | ≤8 ticks | 100% Rust FFI | 85% | 100% |

### Coverage Thresholds

**P0 (Blocking)**:
- Weaver validation: 100%
- Critical paths (Chicago TDD): 100%
- Hot path performance: 100% (≤8 ticks)

**P1 (High Priority)**:
- Integration tests: 100% of features
- Unit test line coverage: 85%+

**P2 (Nice to Have)**:
- Branch coverage: 80%+
- Documentation coverage: 100%

---

## Test Execution Strategy

### Local Development

**Pre-Commit**:
```bash
# Fast validation
cargo clippy --workspace -- -D warnings
weaver registry check -r registry/
cargo test --workspace
```

**Pre-Push**:
```bash
# Comprehensive validation
make test-all
weaver registry live-check --registry registry/
```

---

### Continuous Integration (CI)

**PR Validation Pipeline**:
```yaml
1. Code Quality Gate
   - cargo build --workspace --release
   - cargo clippy --workspace -- -D warnings
   - cargo fmt --all -- --check
   - make build

2. Test Coverage Gate
   - make test-chicago-v04
   - make test-performance-v04
   - make test-integration-v2
   - cargo test --workspace

3. Weaver Validation Gate (BLOCKING)
   - weaver registry check -r registry/
   - weaver registry live-check --registry registry/

4. Functional Validation Gate
   - Execute all CLI commands with real arguments
   - Verify output and telemetry
   - Check exit codes
```

**Result**: All gates must PASS for merge approval

---

### Release Validation

**v1.0 Release Checklist**:
```bash
# Gate 1: Code Quality
✅ cargo build --workspace --release (zero warnings)
✅ cargo clippy --workspace -- -D warnings (zero issues)
✅ make build (C library compiles)

# Gate 2: Test Coverage
✅ make test-all (100% pass rate)
✅ Chicago TDD: 100% critical paths tested
✅ Performance: All hot paths ≤8 ticks
✅ Integration: All features end-to-end tested

# Gate 3: Weaver Validation (SOURCE OF TRUTH)
✅ weaver registry check -r registry/ (100% valid)
✅ weaver registry live-check --registry registry/ (100% conformance)

# Gate 4: Functional Validation
✅ All commands executed with real arguments
✅ All outputs verified
✅ All telemetry validated
✅ All workflows tested end-to-end
```

**Definition of Done**: ALL gates PASS

---

## Test Organization

### Directory Structure

```
knhk/
├── rust/
│   ├── knhk-*/
│   │   ├── src/
│   │   ├── tests/
│   │   │   ├── chicago_tdd_*.rs       # Chicago TDD tests
│   │   │   ├── performance_*.rs       # Performance tests
│   │   │   └── integration/           # Integration tests
│   │   │       └── *.rs
│   │   └── Cargo.toml
│   └── ...
├── c/
│   ├── tests/
│   │   ├── chicago/                   # Chicago TDD tests
│   │   ├── performance/               # Performance tests
│   │   └── integration/               # Integration tests
│   └── Makefile
├── registry/                          # Weaver schemas (SOURCE OF TRUTH)
│   ├── knhk-*.yaml
│   └── registry_manifest.yaml
└── scripts/
    ├── validate-v1.0-dod.sh          # DoD validation
    ├── run-performance-tests.sh       # Performance validation
    └── run-integration-tests.sh       # Integration validation
```

---

## Test Quality Standards

### Test Naming Conventions

**Chicago TDD Tests**:
```rust
// ✅ GOOD: Describes behavior
#[test]
fn sequence_executes_branches_in_order() { }

#[test]
fn parallel_split_executes_all_branches() { }

// ❌ BAD: Vague or implementation-focused
#[test]
fn test1() { }

#[test]
fn sequence_calls_next() { }
```

**Performance Tests**:
```rust
#[test]
fn hot_path_ring_buffer_meets_8_tick_constraint() { }

#[test]
fn beat_scheduler_processing_under_8_ticks() { }
```

---

### Test Documentation

**Every test file must have**:
```rust
// File: tests/chicago_tdd_patterns.rs
// Purpose: Chicago TDD tests for workflow patterns
// Tests behavior (what patterns do), not implementation (how)
```

**Every test function must have**:
```rust
#[test]
fn test_name_describes_behavior() {
    // Arrange: Setup preconditions

    // Act: Execute behavior

    // Assert: Verify outcomes
}
```

---

## Weaver Integration Standards

### Schema Requirements

**Every feature MUST have**:
1. Schema definition in `registry/`
2. All spans/metrics/logs documented
3. Semantic conventions followed
4. Schema validation passing

**Example Schema**:
```yaml
# registry/knhk-workflow-engine.yaml
groups:
  - id: workflow.execution
    type: span
    brief: Workflow execution span
    attributes:
      - id: workflow.id
        type: string
        requirement_level: required
      - id: workflow.status
        type: string
        requirement_level: required
```

---

### Live Validation Process

1. **Execute feature** with real arguments
2. **Capture telemetry** via OTEL collector
3. **Run Weaver live-check**
4. **Verify conformance** to schema
5. **Document deviations** (if any)

**Example**:
```bash
# 1. Execute workflow
knhk workflow run workflow.yaml

# 2. Weaver validates telemetry
weaver registry live-check --registry registry/

# 3. Expected output:
# ✅ All telemetry conforms to schema
# ✅ No undocumented spans
# ✅ No schema violations
```

---

## Validation Failure Handling

### Weaver Validation Failure

**Impact**: BLOCKING - Feature does NOT work

**Response**:
1. Review schema definition
2. Check runtime instrumentation
3. Verify telemetry emission
4. Fix schema or code
5. Re-validate

**No workarounds**: Weaver validation failure = feature broken

---

### Test Failure

**Impact**: Supporting evidence suggests issues

**Response**:
1. Investigate test failure
2. Determine if test is valid
3. Fix code or test
4. Verify Weaver validation still passes

**Important**: Test can fail even when feature works (false negative is OK)

---

## Performance Test Standards

### Hot Path Constraints

**Definition**: Operations in critical execution path

**Constraint**: τ ≤ 8 ticks (Chatman Constant)

**Measurement**:
```rust
const TICK_DURATION_NS: u64 = 1; // Platform-specific
let ticks = duration.as_nanos() / TICK_DURATION_NS;
assert!(ticks <= 8);
```

**Hot Paths**:
- Ring buffer operations
- Beat scheduler processing
- Hook registry lookup
- Memory reuse operations

---

### Cold Path Constraints

**Definition**: Operations outside critical path

**Constraint**: No strict limit (but document)

**Examples**:
- Initial setup
- Configuration loading
- Schema validation
- Report generation

---

## Integration Test Patterns

### Multi-Component Integration

```rust
#[test]
fn workflow_engine_with_etl_integration() {
    // Arrange: Initialize both components
    let engine = WorkflowEngine::new();
    let etl = EtlPipeline::new();

    // Act: Execute integrated workflow
    let result = engine.execute_with_etl(workflow, etl);

    // Assert: Verify integrated behavior
    assert!(result.is_ok());
    assert_telemetry_from_both_components();
}
```

---

### FFI Boundary Testing

```rust
#[test]
fn rust_to_c_ffi_workflow_execution() {
    // Arrange: Create C workflow handle
    let c_workflow = unsafe { knhk_workflow_new() };

    // Act: Call C API from Rust
    let result = unsafe { knhk_workflow_execute(c_workflow) };

    // Assert: Verify C API behavior
    assert_eq!(result, 0); // Success

    // Cleanup
    unsafe { knhk_workflow_free(c_workflow) };
}
```

---

## Test Maintenance Guidelines

### When to Update Tests

**ALWAYS update when**:
- Feature behavior changes
- New features added
- Bugs fixed
- Performance requirements change

**NEVER update to**:
- Make failing tests pass without fixing code
- Remove tests that expose real issues
- Reduce coverage to meet deadlines

---

### Test Refactoring

**Good reasons**:
- Improve test clarity
- Reduce test duplication
- Better test organization

**Bad reasons**:
- Tests are "too strict"
- Tests are "annoying"
- Tests slow down development

---

## Summary: Definition of Done v1.0

**A feature is DONE when**:

### ✅ LEVEL 1: Weaver Validation (MANDATORY)
- [ ] `weaver registry check -r registry/` passes
- [ ] `weaver registry live-check --registry registry/` passes
- [ ] All telemetry documented in schema
- [ ] All runtime telemetry conforms to schema

### ✅ LEVEL 2: Code Quality (BASELINE)
- [ ] `cargo build --workspace --release` succeeds (zero warnings)
- [ ] `cargo clippy --workspace -- -D warnings` passes (zero issues)
- [ ] `make build` succeeds (C library)
- [ ] No `unwrap()`/`expect()` in production code
- [ ] Proper error handling

### ✅ LEVEL 3: Test Coverage (SUPPORTING EVIDENCE)
- [ ] Chicago TDD tests: 100% critical paths
- [ ] Performance tests: All hot paths ≤8 ticks
- [ ] Integration tests: Feature tested end-to-end
- [ ] Unit tests: 85%+ coverage
- [ ] All tests pass: `make test-all`

### ✅ LEVEL 4: Functional Validation
- [ ] Command executed with real arguments (not just `--help`)
- [ ] Command produces expected output
- [ ] Command emits proper telemetry
- [ ] Workflows tested end-to-end
- [ ] Error handling validated

---

**Remember**: Tests can lie, schemas don't. Weaver validation is the source of truth.

**If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.**

---

## Appendix A: Test Execution Commands

```bash
# Full validation suite
make validate-v1.0

# Individual test suites
make test-chicago-v04      # Chicago TDD tests
make test-performance-v04  # Performance tests (≤8 ticks)
make test-integration-v2   # Integration tests
cargo test --workspace     # All Rust unit tests
make test                  # All C tests

# Code quality
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
make build

# Weaver validation (SOURCE OF TRUTH)
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

---

## Appendix B: Common Validation Failures

### Failure: "Help text exists but command doesn't work"

**Symptom**: `knhk <command> --help` succeeds, but `knhk <command> <args>` fails

**Root Cause**: Command registered in CLI but implementation calls `unimplemented!()`

**Fix**: Implement actual command logic, not just help text

**Prevention**: Always execute commands with real arguments during validation

---

### Failure: "Tests pass but Weaver validation fails"

**Symptom**: `cargo test` succeeds, but `weaver registry live-check` fails

**Root Cause**: Tests validate test logic, not actual runtime behavior

**Fix**: Fix runtime instrumentation to match schema

**Prevention**: Always run Weaver validation, not just tests

---

### Failure: "Hot path exceeds 8 ticks"

**Symptom**: Performance test shows τ > 8 ticks

**Root Cause**: Algorithm complexity or unnecessary allocations

**Fix**: Optimize algorithm, eliminate allocations

**Prevention**: Profile hot paths early and often

---

**End of Testing Strategy Document**
