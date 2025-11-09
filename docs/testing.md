# Testing - 80/20 Guide

**Version**: 1.0  
**Status**: Production-Ready  
**Last Updated**: 2025-01-XX

---

## Overview

KNHK follows Chicago TDD (Classicist Test-Driven Development) methodology, focusing on state-based testing with real collaborators. Tests verify behavior, not implementation details.

**Key Features**:
- ✅ Chicago TDD methodology (state-based testing)
- ✅ Real collaborators (no mocks)
- ✅ Behavior verification (not implementation details)
- ✅ AAA pattern (Arrange, Act, Assert)
- ✅ Comprehensive test coverage (32 test suites)
- ✅ Weaver validation (source of truth)

---

## Quick Start (80% Use Case)

### Chicago TDD Principles

**State-Based Tests** (Not Interaction-Based):
- Tests verify outputs and invariants
- No mocking of internal implementation
- Focus on what code does, not how

**Real Collaborators** (No Mocks):
- Use actual dependencies (knhk-lockchain, knhk-otel, oxigraph)
- No test doubles or stubs
- Production code paths tested

**Behavior Verification**:
- Test outcomes and state changes
- Verify guard constraints
- Verify error handling

**AAA Pattern**:
- **Arrange**: Set up real state
- **Act**: Execute operation
- **Assert**: Verify state change

---

## Core Testing (80% Value)

### Test Coverage

**Runtime Classes & SLOs** (50 tests):
- **runtime_class.rs**: 5 inline tests + 11 separate tests
  - Coverage: R1/W1/C1 classification, metadata, edge cases
- **slo_monitor.rs**: 10 inline tests + 10 separate tests
  - Coverage: p99 calculation, violation detection, window management
- **failure_actions.rs**: 7 inline tests + 7 separate tests
  - Coverage: R1 escalation, W1 retry/degrade, C1 async finalization

**ETL Pipeline Tests**:
- **Ingest Stage**: RDF/Turtle parsing, prefix resolution, blank nodes, literals
- **Transform Stage**: IRI hashing, schema validation
- **Load Stage**: SoA array construction, predicate grouping, guard constraint enforcement
- **Reflex Stage**: Receipt generation, tick budget enforcement, span ID generation
- **Emit Stage**: Lockchain integration, downstream API calls, W1 cache degradation, C1 async finalization

### Test Execution

**Run All Tests**:
```bash
cargo test --workspace
```

**Run Chicago TDD Tests**:
```bash
cargo test --test chicago_tdd_tools_integration
```

**Run with Output**:
```bash
cargo test --test chicago_tdd_tools_integration -- --nocapture
```

**Run C Tests**:
```bash
cd c && make test
```

### Weaver Validation

**Critical**: All features MUST pass Weaver validation (source of truth).

**Weaver Schema Validation**:
```bash
weaver registry check -r registry/
```

**Weaver Runtime Validation**:
```bash
weaver registry live-check --registry registry/
```

**Test Hierarchy**:
1. **Level 1: Weaver Schema Validation** (MANDATORY)
   - Schema is valid
   - Runtime telemetry conforms
2. **Level 2: Compilation & Code Quality** (Baseline)
   - Code compiles
   - Zero linting warnings
3. **Level 3: Traditional Tests** (Supporting Evidence)
   - Rust unit tests
   - C Chicago TDD tests

**⚠️ WARNING**: Tests at Level 3 can pass even when features are broken (false positives). Only Weaver validation (Level 1) is the source of truth.

---

## Test Categories

### Unit Tests

**Purpose**: Test individual functions and modules in isolation.

**Examples**:
- Function input/output validation
- Error handling
- Edge cases

### Integration Tests

**Purpose**: Test interactions between components.

**Examples**:
- ETL pipeline end-to-end
- Workflow engine execution
- Connector framework integration

### Chicago TDD Tests

**Purpose**: Test behavior with real collaborators.

**Examples**:
- Runtime class classification
- SLO monitoring
- Failure actions

### Property-Based Tests

**Purpose**: Test invariants across many inputs.

**Examples**:
- Pattern execution properties
- Receipt generation properties
- State transition properties

---

## Production Readiness

### ✅ Ready for Production

- **Test Infrastructure**: Complete (Chicago TDD framework)
- **Test Coverage**: Comprehensive (32 test suites)
- **Weaver Validation**: Integrated (source of truth)
- **Test Execution**: Automated (CI/CD integration)

### ⚠️ Partial Production Readiness

- **Property-Based Tests**: Some tests blocked by compilation errors
- **Performance Tests**: Not executed (dependent on compilation)

---

## Troubleshooting

### Test Failures

**Problem**: Tests fail after code changes.

**Solution**:
- Run tests individually to identify failures
- Check test dependencies
- Verify test data setup
- Review test output for specific errors

### Weaver Validation Failures

**Problem**: Weaver validation fails.

**Solution**:
- Check OTEL schema definitions
- Verify telemetry instrumentation
- Review Weaver validation output
- Fix schema violations

### Compilation Errors in Tests

**Problem**: Tests fail to compile.

**Solution**:
- Fix missing fields in test data structures
- Fix trait bound issues
- Review error messages for specific fixes

---

## Additional Resources

### Detailed Documentation
- **Chicago TDD**: [Chicago TDD Guide](CHICAGO_TDD.md)
- **Testing Guide**: [Testing Documentation](testing.md)
- **Test Results**: [Actual Test Results](ACTUAL_TEST_RESULTS.md)
- **Validation**: [Validation Complete README](VALIDATION_COMPLETE_README.md)
- **Weaver**: [Weaver Integration](WEAVER.md)

### Test Examples
- **Chicago TDD**: `rust/knhk-workflow-engine/tests/chicago_tdd_*.rs`
- **C Tests**: `c/tests/chicago_*.c`
- **Integration Tests**: `rust/knhk-workflow-engine/tests/integration_*.rs`

### Test Infrastructure
- **Test Helpers**: `rust/knhk-workflow-engine/src/testing/`
- **Test Utilities**: `c/tests/chicago_test_helpers.c`

---

## License

MIT License

---

**Last Updated**: 2025-01-XX  
**Version**: 1.0  
**Status**: Production-Ready
