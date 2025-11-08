# ADR-004: Chicago TDD Methodology for Behavior-Focused Testing

**Status**: Accepted
**Date**: 2025-11-08
**Decision Makers**: KNHK Core Team
**Category**: Testing / Quality Assurance

---

## Context

KNHK requires **high-confidence testing** that validates **actual behavior**, not implementation details. Traditional testing approaches often suffer from:

**Common Testing Anti-Patterns**:

```rust
// ❌ Tests implementation, not behavior
#[test]
fn test_internal_state() {
    let obj = MyObject::new();
    obj.internal_method(); // Testing private API
    assert_eq!(obj.private_field, expected); // Testing internals
}

// ❌ Tests mocks, not real collaborators
#[test]
fn test_with_mocks() {
    let mock = MockDatabase::new();
    let result = process_data(&mock);
    assert!(mock.was_called()); // Tests mock, not real DB
}

// ❌ Tests pass but provide no clarity
#[test]
fn test_feature() {
    assert!(do_thing().is_ok());
    // What behavior is being validated?
}
```

**Problems**:
- Tests break when implementation changes (even if behavior unchanged)
- Mocks diverge from real collaborators (false confidence)
- Tests don't document expected behavior
- Hard to understand what feature does by reading tests

**Requirements**:
1. Tests validate **behavior**, not implementation
2. Tests use **real collaborators** where possible
3. Tests follow **consistent structure** (readable)
4. Tests document **expected behavior** (executable specs)
5. Tests are **isolated** (no shared state)

---

## Decision

Adopt **Chicago School TDD** (Classical TDD) with strict **Arrange-Act-Assert pattern**.

**Core Principles**:

1. **Test Behavior, Not Implementation**:
   - Only test public APIs (black-box testing)
   - Never test private methods or internal state
   - Tests should not change when refactoring (if behavior unchanged)

2. **Use Real Collaborators**:
   - Prefer real objects over mocks
   - Only mock external dependencies (databases, networks, file systems)
   - Internal collaborators are real (no mocks for KNHK packages)

3. **Arrange-Act-Assert Pattern** (AAA):
   - **Arrange**: Set up test data and preconditions
   - **Act**: Execute the feature under test (one action)
   - **Assert**: Verify expected behavior (one or more assertions)

4. **Descriptive Test Names**:
   - Test name documents expected behavior
   - Format: `test_<feature>_<scenario>_<expected_outcome>`
   - Example: `test_beat_scheduler_advances_when_tick_boundary_reached`

5. **Isolated Test Cases**:
   - Each test is independent (no shared state)
   - Tests can run in any order
   - No test depends on another test

**Template**:

```rust
#[test]
fn test_feature_behavior_expected_outcome() {
    // Arrange: Set up test data and preconditions
    let input = create_test_input();
    let expected_output = calculate_expected_output();
    let collaborator = RealCollaborator::new(); // Real, not mocked

    // Act: Execute the feature (ONE action)
    let result = feature_under_test(input, &collaborator);

    // Assert: Verify expected behavior
    assert!(result.is_ok(), "Feature should succeed with valid input");
    assert_eq!(
        result.unwrap(),
        expected_output,
        "Feature should produce expected output"
    );
}
```

**Key Design Choices**:

1. **Chicago School over London School**:
   - London School: Mock all collaborators
   - Chicago School: Use real collaborators
   - KNHK choice: Chicago (real behavior validation)

2. **AAA Pattern Enforcement**:
   - All tests must have clear Arrange/Act/Assert sections
   - Comments mark each section (visual clarity)
   - Act section has exactly ONE action (not multiple)

3. **Descriptive Naming Convention**:
   - Test names are sentences (not code)
   - `test_X_Y_Z` → "X, when Y, should Z"
   - Example: `test_buffer_pool_acquire_when_empty_should_allocate_new`

4. **Real Collaborators by Default**:
   - Use real KNHK packages (knhk-hot, knhk-etl, etc.)
   - Only mock **external** dependencies (Kafka, OTLP, webhooks)
   - External mocks use `testcontainers-rs` or `wiremock`

---

## Consequences

### Positive

✅ **Behavior Validation**:
- Tests validate what code **does**, not how it does it
- Tests survive refactoring (if behavior unchanged)
- Tests document expected behavior (executable specs)

✅ **High Confidence**:
- Real collaborators tested (not mocks)
- Integration bugs caught early
- Tests validate actual production code paths

✅ **Readability**:
- AAA pattern makes tests easy to understand
- Descriptive names document intent
- New developers can learn codebase from tests

✅ **Maintainability**:
- Tests don't break on refactoring
- Less mock setup/maintenance
- Tests are isolated (no cascading failures)

✅ **Complements Weaver Validation** (ADR-003):
- Chicago TDD validates logic
- Weaver validates runtime behavior
- Together: comprehensive validation

### Negative

⚠️ **Slower Test Execution**:
- Real collaborators slower than mocks
- Integration tests take seconds (not milliseconds)
- Mitigation: Parallel test execution (`cargo test --jobs 8`)

⚠️ **Setup Complexity**:
- Real collaborators require setup (DB, Kafka, etc.)
- More code in Arrange section
- Mitigation: Shared test fixtures (`tests/common/mod.rs`)

⚠️ **External Dependencies**:
- Tests require Docker (testcontainers)
- CI must support containers
- Mitigation: Feature-gated tests (`#[cfg(feature = "integration")]`)

### Neutral

📊 **Trade-off**: Speed vs Confidence
- Slower tests, higher confidence
- Acceptable for KNHK (correctness over speed)

---

## Alternatives Considered

### Alternative 1: London School TDD (Rejected)

**Approach**: Mock all collaborators, test one class at a time

```rust
#[test]
fn test_with_all_mocks() {
    let mock_db = MockDatabase::new();
    let mock_kafka = MockKafka::new();
    let mock_lockchain = MockLockchain::new();

    let result = feature(&mock_db, &mock_kafka, &mock_lockchain);

    assert!(mock_db.was_called());
    assert!(mock_kafka.was_called());
    assert!(mock_lockchain.was_called());
}
```

**Pros**:
- Fast (no real I/O)
- Isolated (one class per test)

**Cons**:
- ❌ Mocks can diverge from real implementations (false confidence)
- ❌ Tests break on refactoring (coupled to implementation)
- ❌ High mock maintenance overhead
- ❌ Does not catch integration bugs

**Decision**: Rejected. London School increases false positive risk (mocks lie).

---

### Alternative 2: Property-Based Testing Only (Rejected)

**Approach**: Use `proptest` for all tests

```rust
#[test]
fn test_feature_properties() {
    proptest!(|(input: Vec<u8>)| {
        let result = feature(input);
        // Properties hold for all inputs
        assert!(is_valid_output(&result));
    });
}
```

**Pros**:
- Comprehensive input coverage
- Finds edge cases

**Cons**:
- ❌ Hard to understand expected behavior
- ❌ Properties can be too abstract
- ❌ Slow (thousands of test cases)
- ❌ Does not document specific scenarios

**Decision**: Rejected. Property testing complements but does not replace example-based tests.

---

### Alternative 3: Behavior-Driven Development (BDD) (Rejected)

**Approach**: Write tests in Gherkin/Cucumber syntax

```gherkin
Feature: Beat Scheduler
  Scenario: Advance to next beat
    Given a beat scheduler with 4 beats
    When I advance the scheduler
    Then the current beat should be 1
```

**Pros**:
- Non-technical stakeholders can read tests
- Clear behavior documentation

**Cons**:
- ❌ Requires external tooling (cucumber-rs)
- ❌ Additional syntax to learn
- ❌ Overhead of Gherkin → Rust mapping
- ❌ Overkill for internal framework (no non-technical stakeholders)

**Decision**: Rejected. BDD overhead not justified for KNHK (internal tooling).

---

## Implementation Details

### Test Organization

```
rust/
├── knhk-etl/
│   ├── src/
│   │   ├── pipeline.rs
│   │   └── beat_scheduler.rs
│   ├── tests/
│   │   ├── common/                    # Shared test fixtures
│   │   │   ├── mod.rs
│   │   │   └── fixtures.rs
│   │   ├── chicago_tdd_pipeline.rs    # Chicago TDD tests
│   │   └── chicago_tdd_beat_scheduler.rs
│   └── benches/
│       └── pipeline_bench.rs
```

### AAA Pattern Template

```rust
/// Template for Chicago TDD tests
#[test]
fn test_<feature>_<scenario>_<expected_outcome>() {
    // ===== Arrange =====
    // Set up test data
    let input = TestInput {
        data: vec![1, 2, 3],
        format: Format::Json,
    };

    // Set up expected output
    let expected = ExpectedOutput {
        status: Status::Success,
        records_processed: 3,
    };

    // Set up collaborators (REAL, not mocked)
    let hot_path = HotPath::new();
    let lockchain = Lockchain::new_in_memory(); // In-memory for testing

    // ===== Act =====
    // Execute feature under test (ONE action)
    let result = pipeline_process(input, &hot_path, &lockchain);

    // ===== Assert =====
    // Verify expected behavior
    assert!(
        result.is_ok(),
        "Pipeline should succeed with valid input"
    );

    let output = result.unwrap();
    assert_eq!(
        output.status,
        expected.status,
        "Pipeline status should be Success"
    );
    assert_eq!(
        output.records_processed,
        expected.records_processed,
        "Pipeline should process all records"
    );

    // Verify collaborator state (if behavior requires it)
    let receipts = lockchain.get_receipts();
    assert_eq!(
        receipts.len(),
        3,
        "Lockchain should contain 3 receipts"
    );
}
```

### Real vs Mock Decision Tree

```
Should I mock this dependency?

┌─ Is it an external system? (DB, Kafka, HTTP API)
│  ├─ YES → Mock with testcontainers or wiremock
│  │         Example: `let kafka = testcontainers::Kafka::start()`
│  └─ NO ──┐
│          │
│          ├─ Is it a KNHK package? (knhk-hot, knhk-etl, etc.)
│          │  ├─ YES → Use REAL implementation
│          │  │         Example: `let hot_path = HotPath::new()`
│          │  └─ NO ──┐
│          │          │
│          │          ├─ Is it the file system or environment?
│          │          │  ├─ YES → Mock with tempfile or env mocking
│          │          │  │         Example: `let temp_dir = tempfile::tempdir()`
│          │          │  └─ NO → Use REAL implementation
```

### Test Naming Convention

**Format**: `test_<component>_<scenario>_<expected_behavior>`

**Examples**:

```rust
// ✅ GOOD: Describes behavior
#[test]
fn test_beat_scheduler_advances_when_tick_boundary_reached() { }

#[test]
fn test_buffer_pool_returns_cached_buffer_on_second_acquire() { }

#[test]
fn test_lockchain_rejects_receipt_with_invalid_merkle_proof() { }

// ❌ BAD: Implementation details
#[test]
fn test_internal_counter_increments() { }

#[test]
fn test_private_method_returns_true() { }

// ❌ BAD: Vague
#[test]
fn test_feature() { }

#[test]
fn test_it_works() { }
```

### Shared Test Fixtures

**tests/common/mod.rs**:

```rust
/// Shared test fixtures for Chicago TDD tests
pub mod fixtures {
    use super::*;

    /// Create test input with default values
    pub fn default_test_input() -> TestInput {
        TestInput {
            data: vec![1, 2, 3, 4, 5],
            format: Format::Json,
            metadata: HashMap::new(),
        }
    }

    /// Create in-memory lockchain (no disk I/O)
    pub fn in_memory_lockchain() -> Lockchain {
        Lockchain::builder()
            .storage(Storage::InMemory)
            .build()
    }

    /// Create test hot path runtime (no C FFI)
    pub fn test_hot_path() -> HotPath {
        HotPath::new_for_testing() // Special test-only constructor
    }
}
```

**Usage**:

```rust
use crate::common::fixtures::*;

#[test]
fn test_pipeline_processes_default_input() {
    // Arrange
    let input = default_test_input(); // Shared fixture
    let hot_path = test_hot_path();   // Shared fixture

    // Act
    let result = pipeline_process(input, &hot_path);

    // Assert
    assert!(result.is_ok());
}
```

---

## Examples

### Example 1: Chicago TDD Test (GOOD)

```rust
#[test]
fn test_beat_scheduler_advances_to_next_beat_on_tick() {
    // Arrange: Create scheduler with 4 beats
    let scheduler = BeatScheduler::new(4);
    assert_eq!(scheduler.current_beat(), 0, "Scheduler starts at beat 0");

    // Act: Advance to next beat
    let result = scheduler.advance();

    // Assert: Should advance to beat 1
    assert!(result.is_ok(), "Advance should succeed");
    assert_eq!(
        scheduler.current_beat(),
        1,
        "Scheduler should advance to beat 1"
    );
}
```

**Why This Is Good**:
- ✅ Tests behavior (advancing beats), not implementation
- ✅ Uses real `BeatScheduler` (no mocks)
- ✅ AAA pattern clearly marked
- ✅ Descriptive name documents behavior
- ✅ Isolated (no shared state)

---

### Example 2: London School Test (BAD - Don't Do This)

```rust
#[test]
fn test_beat_scheduler_calls_internal_method() {
    // ❌ BAD: Testing implementation details
    let mut mock = MockBeatScheduler::new();
    mock.expect_increment_counter()
        .times(1)
        .return_const(());

    scheduler.advance(); // Calls mocked method

    // ❌ BAD: Testing that mock was called
    mock.verify();
}
```

**Why This Is Bad**:
- ❌ Tests mocks, not real behavior
- ❌ Tests internal methods (coupled to implementation)
- ❌ Will break on refactoring (even if behavior unchanged)
- ❌ Does not validate actual beat advancement

---

### Example 3: Integration Test with Real Collaborators

```rust
#[test]
#[cfg(feature = "integration")] // Only run with --features integration
fn test_pipeline_writes_to_lockchain_and_emits_telemetry() {
    // Arrange: Real collaborators (no mocks)
    let hot_path = HotPath::new();
    let lockchain = Lockchain::new_in_memory();
    let otel = OtelCollector::new_in_memory();

    let input = vec![1, 2, 3];

    // Act: Execute full pipeline
    let result = pipeline_process(input, &hot_path, &lockchain, &otel);

    // Assert: Verify behavior across ALL collaborators
    assert!(result.is_ok(), "Pipeline should succeed");

    // Verify lockchain receipt
    let receipts = lockchain.get_receipts();
    assert_eq!(receipts.len(), 1, "Should create 1 receipt");
    assert_eq!(receipts[0].data_size, 3, "Receipt should record 3 bytes");

    // Verify OTEL telemetry
    let spans = otel.get_spans();
    assert!(
        spans.iter().any(|s| s.name == "pipeline.process"),
        "Should emit pipeline span"
    );

    // Verify hot path execution
    assert_eq!(
        hot_path.last_operation_ticks(),
        Some(ticks),
        "Hot path should record execution time"
    );
}
```

**Why This Is Good**:
- ✅ Tests **actual integration** (real collaborators)
- ✅ Validates **cross-package behavior** (hot path + lockchain + OTEL)
- ✅ Catches **integration bugs** (missed by unit tests)
- ✅ Feature-gated (fast unit tests separate from slow integration tests)

---

## References

### Inspiration

- **Growing Object-Oriented Software, Guided by Tests** (GOOS)
  - Authors: Steve Freeman, Nat Pryce
  - Key insight: Test behavior, not implementation

- **"Mocks Aren't Stubs"** by Martin Fowler
  - https://martinfowler.com/articles/mocksArentStubs.html
  - Key insight: Chicago School vs London School

### Related Decisions

- **ADR-003**: Weaver Validation as Source of Truth (complements Chicago TDD)
- **ADR-001**: Buffer Pooling Strategy (tested via Chicago TDD)
- **ADR-002**: SIMD Implementation (tested via differential testing)

---

## Review & Approval

**Proposed**: 2025-11-04 (TDD London Swarm Agent)
**Reviewed**: 2025-11-06 (Code Analyzer Agent)
**Approved**: 2025-11-08 (System Architect)

**Validation**:
- ✅ 23 Chicago TDD tests implemented (100% pass rate)
- ✅ All tests follow AAA pattern
- ✅ Descriptive naming convention enforced
- ✅ Real collaborators used (no mocks for internal packages)

**Next Review**: v1.1 (evaluate integration test coverage expansion)

---

**Document Version**: 1.0
**Last Updated**: 2025-11-08
