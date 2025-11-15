# Testing Checklist

**Purpose**: One-page verification for comprehensive Chicago TDD testing
**Target**: ≥90% coverage, zero false positives, production-ready validation
**Validation**: All test suites must pass

---

## 1. Chicago TDD Principles

**State-based testing with real collaborators:**

- [ ] Tests verify outcomes and state changes (not interactions)
- [ ] Tests use real collaborators (no mocks for core logic)
- [ ] Tests follow AAA pattern (Arrange, Act, Assert)
- [ ] Tests have descriptive names (`test_hot_path_ask_query_within_budget`)
- [ ] Tests are independent (can run in any order)
- [ ] Tests are deterministic (same input → same output)
- [ ] Tests clean up after themselves (no side effects)
- [ ] Tests document behavior (serve as executable specifications)

**Pass Criteria**: All 8 items checked ✅

---

## 2. Unit Testing (≥90% Coverage)

**Test individual components in isolation:**

- [ ] All public functions have unit tests
- [ ] All error paths tested (Result::Err cases)
- [ ] All edge cases tested (empty inputs, boundary conditions)
- [ ] All success paths tested (Result::Ok cases)
- [ ] Code coverage ≥90% measured (`cargo tarpaulin`)
- [ ] Tests run quickly (≤10s for full suite)
- [ ] No `#[ignore]` tests (all tests active)
- [ ] Tests fail for the right reasons (not flaky)

**Pass Criteria**: All 8 items checked ✅

**Example**:
```rust
#[test]
fn test_hot_path_ask_query_within_budget() {
    // Arrange: Create executor and query
    let executor = WarmPathExecutor::new();
    let query = Query::new_ask("ASK { ?s ?p ?o }");

    // Act: Execute and measure
    let start = std::time::Instant::now();
    let result = executor.execute_ask(query);
    let duration = start.elapsed();

    // Assert: Verify outcome and performance
    assert!(result.is_ok(), "ASK query should succeed");
    let ticks = calculate_ticks(duration);
    assert!(ticks <= 8, "ASK query must complete within 8 ticks, got {}", ticks);
}
```

---

## 3. Integration Testing

**Test multiple components working together:**

- [ ] Integration tests cover happy paths (end-to-end success)
- [ ] Integration tests cover error paths (component failures)
- [ ] Integration tests use real databases/networks (not mocks)
- [ ] Integration tests verify data flow between components
- [ ] Integration tests validate performance (not just correctness)
- [ ] Integration tests run in CI/CD
- [ ] Integration tests clean up resources (temp files, DB records)
- [ ] `make test-integration-v2` passes

**Pass Criteria**: All 8 items checked ✅

---

## 4. Performance Testing (Chatman Constant)

**Verify hot path ≤8 ticks, warm path ≤500µs:**

- [ ] ASK operations ≤1.5 ns (`make test-performance-v04`)
- [ ] COUNT operations ≤1.5 ns
- [ ] COMPARE operations ≤1.5 ns
- [ ] VALIDATE operations ≤2.0 ns
- [ ] CONSTRUCT8 operations ≤500 µs (warm path)
- [ ] Performance tests run in isolation (no interference)
- [ ] Performance tests use real hardware (not VMs)
- [ ] Performance regression tests in CI

**Pass Criteria**: All 8 items checked ✅

**Command**: `make test-performance-v04`

---

## 5. Enterprise Use Case Testing

**Validate production scenarios:**

- [ ] `make test-enterprise` passes
- [ ] 18/19 enterprise use cases tested
- [ ] Hot path qualification verified (≤8 ticks)
- [ ] Warm path fallback tested (>8 ticks)
- [ ] Error handling for all enterprise scenarios
- [ ] Performance tested under load (concurrent users)
- [ ] Data integrity verified (receipts, provenance)
- [ ] Real-world data sizes tested (not toy examples)

**Pass Criteria**: All 8 items checked ✅

**Command**: `make test-enterprise`

---

## 6. Chicago TDD for C Hot Path

**Test branchless C engine:**

- [ ] `make test-chicago-v04` passes
- [ ] ASK, COUNT, COMPARE, VALIDATE operations tested
- [ ] CONSTRUCT8 routing to warm path tested
- [ ] FFI bindings tested (Rust ↔ C)
- [ ] Memory safety verified (no leaks, no corruption)
- [ ] SIMD operations tested (8-lane parallelism)
- [ ] Alignment verified (64-byte for AVX-512)
- [ ] Branchless code verified (no branch mispredicts)

**Pass Criteria**: All 8 items checked ✅

**Command**: `make test-chicago-v04`

---

## 7. Error Handling Testing

**Verify robust error handling:**

- [ ] All `Result<T, E>` types tested (both Ok and Err)
- [ ] Error messages are descriptive and actionable
- [ ] Errors propagate correctly (no lost context)
- [ ] Errors logged with tracing (not println)
- [ ] Recovery strategies tested (retry, fallback)
- [ ] Panic-free code (no unwrap/expect in prod)
- [ ] Error telemetry emitted (spans, metrics, logs)
- [ ] Error documentation complete (when/why errors occur)

**Pass Criteria**: All 8 items checked ✅

**Example**:
```rust
#[test]
fn test_query_parse_error_handling() {
    // Arrange: Invalid query
    let executor = WarmPathExecutor::new();
    let query = Query::new_ask("INVALID SPARQL");

    // Act: Execute
    let result = executor.execute_ask(query);

    // Assert: Returns descriptive error
    assert!(result.is_err(), "Invalid query should return error");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("parse"), "Error should mention parsing");
}
```

---

## 8. Test Organization (AAA Pattern)

**Clear, readable, maintainable tests:**

- [ ] **Arrange**: Set up test data and dependencies
- [ ] **Act**: Execute the operation being tested
- [ ] **Assert**: Verify outcomes and state changes
- [ ] Comments separate AAA sections
- [ ] Test names describe scenario: `test_<component>_<scenario>_<expected>`
- [ ] One logical assertion per test (or related assertions)
- [ ] Helper functions for common setup (reduce duplication)
- [ ] Test files organized by module (`tests/module_name_test.rs`)

**Pass Criteria**: All 8 items checked ✅

---

## 9. Mocking Strategy (Use Sparingly)

**When to use mocks vs real collaborators:**

**Use Real Collaborators (Preferred)**:
- [ ] Core business logic (hot path, warm path)
- [ ] Database operations (use test database)
- [ ] Graph operations (use small test graphs)
- [ ] Workflow execution (use simple workflows)

**Use Mocks (When Necessary)**:
- [ ] External APIs (network calls to 3rd party)
- [ ] Slow operations (only if can't be made fast)
- [ ] Non-deterministic operations (time, randomness)
- [ ] Expensive resources (license servers, paid APIs)

**Pass Criteria**: Mocks used sparingly, real collaborators preferred ✅

---

## 10. Weaver Validation Testing

**Schema-validated telemetry testing:**

- [ ] `weaver registry check -r /home/user/knhk/registry/` passes (schema valid)
- [ ] `weaver registry live-check --registry /home/user/knhk/registry/` passes (runtime valid)
- [ ] Telemetry tests verify spans emitted
- [ ] Telemetry tests verify metrics recorded
- [ ] Telemetry tests verify logs generated
- [ ] Telemetry tests check context propagation
- [ ] Telemetry tests validate schema compliance
- [ ] Telemetry overhead tested (≤5% CPU)

**Pass Criteria**: All 8 items checked ✅ - Weaver validation is MANDATORY

---

## 11. Test Execution Commands

**Run all test suites:**

```bash
# Unit tests (Rust)
cd /home/user/knhk/rust && cargo test --workspace

# Chicago TDD (C hot path)
cd /home/user/knhk && make test-chicago-v04

# Performance tests
cd /home/user/knhk && make test-performance-v04

# Integration tests
cd /home/user/knhk && make test-integration-v2

# Enterprise use cases
cd /home/user/knhk && make test-enterprise

# Code coverage
cd /home/user/knhk/rust && cargo tarpaulin --workspace --out Html

# Weaver validation
weaver registry check -r /home/user/knhk/registry/
weaver registry live-check --registry /home/user/knhk/registry/
```

---

## 12. Test Coverage Metrics

**Target ≥90% coverage for production readiness:**

- [ ] Core modules ≥95% coverage (hot path, warm path)
- [ ] Business logic ≥90% coverage (workflows, policies)
- [ ] Integration ≥80% coverage (connectors, adapters)
- [ ] Infrastructure ≥70% coverage (CLI, config)
- [ ] Coverage report generated (`cargo tarpaulin`)
- [ ] Coverage tracked over time (CI/CD)
- [ ] Coverage enforced in CI (fail if below threshold)
- [ ] Uncovered lines documented (why not tested)

**Pass Criteria**: All 8 items checked ✅

**Command**: `cargo tarpaulin --workspace --out Html`

---

## 13. Test Quality Checklist

**High-quality, maintainable tests:**

- [ ] Tests are fast (unit tests ≤10s, integration tests ≤60s)
- [ ] Tests are focused (one scenario per test)
- [ ] Tests are isolated (no shared state between tests)
- [ ] Tests are readable (clear intent, good names)
- [ ] Tests are maintainable (DRY, helper functions)
- [ ] Tests document behavior (serve as examples)
- [ ] Tests fail clearly (descriptive assertion messages)
- [ ] Tests run in CI/CD (automated, gated)

**Pass Criteria**: All 8 items checked ✅

---

## Validation Hierarchy (CRITICAL)

**Remember the testing pyramid:**

1. **Weaver Validation** = Source of truth (proves telemetry works)
2. **Performance Tests** = Proves hot path ≤8 ticks
3. **Integration Tests** = Proves components work together
4. **Unit Tests** = Proves individual components work
5. **Code Coverage** = Supporting metric (not a goal itself)

**If Weaver validation fails, telemetry DOES NOT WORK, regardless of unit test results.**

---

## Final Sign-Off

- [ ] **All 13 sections completed** (104 total checks)
- [ ] **`cargo test --workspace` passes** (100% success)
- [ ] **`make test-chicago-v04` passes** (C hot path)
- [ ] **`make test-performance-v04` passes** (≤8 ticks)
- [ ] **`make test-integration-v2` passes** (integration)
- [ ] **`make test-enterprise` passes** (enterprise use cases)
- [ ] **Weaver validation passes** (telemetry)
- [ ] **Code coverage ≥90%** (measured)

**Testing Approved By**: ________________
**Date**: ________________
**Coverage**: ______%

---

**See Also**:
- [Testing Guide](/home/user/knhk/docs/TESTING.md)
- [Production Readiness Checklist](/home/user/knhk/docs/reference/cards/PRODUCTION_READINESS_CHECKLIST.md)
- [Performance Optimization Checklist](/home/user/knhk/docs/reference/cards/PERFORMANCE_OPTIMIZATION_CHECKLIST.md)
