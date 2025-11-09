# Implement Missing DFLSS Requirements

Read docs, write code, run tests, fix issues, loop.

## Architecture Principle

**Centralized Validation Architecture**:
- **knhk-workflow-engine**: ALL data ingress point. Domain logic, validation, guards.
- **knhk-hot**: Pure execution. NO checks. Assumes pre-validated inputs.

## Workflow

1. **Identify Gaps**:
   - Identify gaps between documented requirements and code implementation
   - Review requirements for each DMEDI phase (Define, Measure, Explore, Develop, Implement)
   - Review architectural decisions and design patterns that inform implementation

2. **Implement Missing Functionality**:
   - Implement missing functionality based on documented requirements
   - Ensure all documented requirements have corresponding code implementations
   - Ensure all validation logic is implemented in knhk-workflow-engine (ingress point)
   - Ensure knhk-hot contains only pure execution with NO validation checks
   - Follow architecture principles: centralized validation at ingress, pure execution in hot path
   - **Use chicago-tdd-tools macros for tests**: `chicago_test!`, `chicago_async_test!`, `chicago_fixture_test!`, `chicago_performance_test!`
   - **Use assertion macros**: `assert_ok!`, `assert_err!`, `assert_within_tick_budget!`, `assert_guard_constraint!`

3. **Run Tests**:
   - Run `make test-rust` to verify implementations (tests run concurrently via scripts/run-all-rust-tests.sh)
   - Run `make test-chicago-v04` for Chicago TDD tests (concurrent execution)
   - Check for compilation errors
   - Verify architecture compliance (validation at ingress, no checks in hot path)

4. **Fix Issues**:
   - Fix compilation errors
   - Fix failing tests
   - Fix architecture violations

5. **Loop**:
   - If tests fail or more requirements found, repeat from step 1
