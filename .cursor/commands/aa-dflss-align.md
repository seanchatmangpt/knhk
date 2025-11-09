# DFLSS Documentation Alignment

Read docs, write code, run tests, fix issues, loop.

## Architecture Principle

**Centralized Validation Architecture**:
- **knhk-workflow-engine**: ALL data ingress point. Domain logic, validation, guards.
- **knhk-hot**: Pure execution. NO checks. Assumes pre-validated inputs.

## Workflow

1. **Understand Requirements and Current State**:
   - Understand the relationship between code implementation and documented requirements
   - Identify requirements for each DMEDI phase (Define, Measure, Explore, Develop, Implement)
   - Review project context including scope, stakeholders, and customer needs
   - Review research-informed architectural decisions and their rationale

2. **Align Code and Documentation**:
   - Ensure code structure aligns with documented requirements
   - Update documentation to reflect current code structure when code changes
   - Update requirements documentation when requirements change
   - Update architectural documentation when architecture evolves
   - Write missing code implementations to fulfill documented requirements
   - **Use chicago-tdd-tools macros for tests**: `chicago_test!`, `chicago_async_test!`, `chicago_fixture_test!`, `chicago_performance_test!`
   - **Use assertion macros**: `assert_ok!`, `assert_err!`, `assert_within_tick_budget!`, `assert_guard_constraint!`

3. **Run Tests**:
   - Run `make test-rust` to verify code changes (tests run concurrently via scripts/run-all-rust-tests.sh)
   - Run `make test-chicago-v04` for Chicago TDD tests (concurrent execution)
   - Check for compilation errors
   - Verify documentation accuracy and consistency

4. **Fix Issues**:
   - Fix compilation errors
   - Fix failing tests
   - Fix documentation gaps and inconsistencies

5. **Loop**:
   - If tests fail or gaps found, repeat from step 1
