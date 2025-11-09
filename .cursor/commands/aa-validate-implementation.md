# Validate Implementation Against DFLSS

Read docs, write validation code, run tests, fix issues, loop.

## Workflow

1. **Understand Requirements**:
   - Understand documented requirements and their expected code structure
   - Review requirements for each DMEDI phase (Define, Measure, Explore, Develop, Implement)
   - Review architectural principles and validation rules

2. **Write Validation Code**:
   - Write validation code to verify code implementation matches documented requirements
   - Verify code structure matches documented requirements
   - Verify implementation fulfills all phase requirements
   - Verify architecture compliance (centralized validation at ingress, pure execution in hot path)
   - **Use chicago-tdd-tools validation utilities**: OTEL validation, performance validation (RDTSC), guard constraint enforcement, Weaver live validation
   - **Use chicago-tdd-tools macros for tests**: `chicago_test!`, `chicago_async_test!`, `chicago_fixture_test!`, `chicago_performance_test!`
   - **Use assertion macros**: `assert_ok!`, `assert_err!`, `assert_within_tick_budget!`, `assert_guard_constraint!`

3. **Run Tests**:
   - Run `make test-rust` to verify validation code (tests run concurrently via scripts/run-all-rust-tests.sh)
   - Run `make test-chicago-v04` for Chicago TDD validation tests (concurrent execution)
   - Run `make test-performance-v04` for performance validation (concurrent execution)
   - Check for compilation errors

4. **Fix Issues**:
   - Fix validation gaps found
   - Fix failing tests
   - Fix compilation errors

5. **Loop**:
   - If validation gaps found, repeat from step 1
