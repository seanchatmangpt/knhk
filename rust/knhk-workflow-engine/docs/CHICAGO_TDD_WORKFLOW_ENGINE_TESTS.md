# Chicago TDD Tests for WorkflowEngine

## Overview

This test suite (`tests/chicago_tdd_tools_integration.rs`) contains comprehensive tests for the `WorkflowEngine` using Chicago TDD methodology with `chicago-tdd-tools`.

## Test Coverage

### 1. Workflow Registration and Retrieval
- **Test**: `test_workflow_registration_and_retrieval`
- **What it tests**: 
  - Workflow registration with `register_workflow()`
  - Workflow retrieval with `get_workflow()`
  - Workflow listing with `list_workflows()`
  - State persistence of workflow specifications

### 2. Case Creation and State Transitions
- **Test**: `test_case_creation_and_state_transitions`
- **What it tests**:
  - Case creation with `create_case()`
  - Case state transitions (Created → Running → Completed)
  - Case retrieval with `get_case()`
  - State verification at each transition

### 3. Multi-Task Workflow Execution
- **Test**: `test_case_execution_with_multiple_tasks`
- **What it tests**:
  - Workflow execution with multiple tasks
  - Case execution with `execute_case()`
  - Task execution order and completion
  - Case state after multi-task execution

### 4. Error Handling - Invalid Workflow
- **Test**: `test_error_handling_invalid_workflow`
- **What it tests**:
  - Error handling for invalid workflow structures
  - Proper error types (Validation, Parse)
  - Graceful error handling

### 5. Error Handling - Missing Workflow
- **Test**: `test_error_handling_missing_workflow`
- **What it tests**:
  - Error handling when retrieving non-existent workflow
  - Proper `InvalidSpecification` error type
  - Error message validation

### 6. Error Handling - Missing Case
- **Test**: `test_error_handling_missing_case`
- **What it tests**:
  - Error handling when retrieving non-existent case
  - Proper `CaseNotFound` error type
  - Error message validation

### 7. Case Cancellation
- **Test**: `test_case_cancellation`
- **What it tests**:
  - Case cancellation with `cancel_case()`
  - State transition to Cancelled
  - Cancellation from Running state
  - State persistence after cancellation

### 8. Multiple Workflows and Cases
- **Test**: `test_multiple_workflows_and_cases`
- **What it tests**:
  - Multiple workflow registration
  - Multiple case creation
  - Case listing per workflow with `list_cases()`
  - Workflow isolation

### 9. State Persistence
- **Test**: `test_state_persistence`
- **What it tests**:
  - State persistence across engine instances
  - Workflow retrieval from persisted state
  - Database persistence verification

### 10. Admission Gate Integration
- **Test**: `test_admission_gate_integration`
- **What it tests**:
  - Admission gate validation during case creation
  - Valid data passing admission gate
  - Case creation with validated data

### 11. Engine Services Access
- **Test**: `test_engine_services_access`
- **What it tests**:
  - Access to pattern registry
  - Access to resource allocator
  - Access to worklet repository and executor
  - Access to timer service, work item service, admission gate, event sidecar

### 12. Complete Workflow Lifecycle
- **Test**: `test_complete_workflow_lifecycle`
- **What it tests**:
  - Complete workflow lifecycle: register → create → start → execute
  - State transitions throughout lifecycle
  - Data preservation
  - End-to-end workflow execution

## Chicago TDD Principles Applied

1. **State-Based Testing**: All tests verify state changes, not implementation details
2. **Real Collaborators**: Uses actual `WorkflowEngine`, `StateStore`, etc. (no mocks)
3. **Behavior Verification**: Tests verify what the engine does, not how
4. **AAA Pattern**: All tests follow Arrange-Act-Assert structure

## Framework Integration

- Uses `chicago_tdd_tools::prelude::*` for:
  - `TestFixture` - Generic test fixtures
  - `TestDataBuilder` - Fluent test data builders
  - Assertion helpers (`assert_success`, `assert_error`, `assert_eq_with_msg`, `assert_in_range`)

- Uses workflow-specific helpers from `knhk_workflow_engine::testing::chicago_tdd`:
  - `WorkflowTestFixture` (not used in these tests - we test engine directly)
  - Pattern helpers (for pattern-specific tests)

## Running Tests

```bash
# Run all tests
cargo test --test chicago_tdd_tools_integration

# Run specific test
cargo test --test chicago_tdd_tools_integration test_workflow_registration_and_retrieval

# Run with output
cargo test --test chicago_tdd_tools_integration -- --nocapture
```

## Test Data

Tests use `TestDataBuilder` from `chicago-tdd-tools` to create realistic test data:
- Order data (`with_order_data`)
- Customer data (`with_customer_data`)
- Approval data (`with_approval_data`)
- Custom variables (`with_var`)

## Key Behaviors Tested

1. **Workflow Management**:
   - Registration, retrieval, listing
   - State persistence
   - Error handling

2. **Case Management**:
   - Creation, starting, execution, cancellation
   - State transitions
   - Data preservation

3. **Error Handling**:
   - Invalid workflows
   - Missing workflows
   - Missing cases
   - Proper error types

4. **Integration**:
   - Admission gate
   - State persistence
   - Service access
   - Multi-workflow scenarios

## Notes

- Each test uses a unique database path to avoid conflicts
- Tests follow Chicago TDD principles strictly
- All tests use real collaborators (no mocks)
- Tests verify behavior, not implementation details

## Implementation Details

### StateStore Access Pattern

The `WorkflowEngine` uses `Arc<RwLock<Arc<StateStore>>>` for state store access:
- Outer `Arc<RwLock<...>>`: Allows concurrent read access across async tasks
- Inner `Arc<StateStore>`: Allows sharing the StateStore with services (e.g., TimerService)
- Access pattern: `(*store).method()` where `store` is obtained via `self.state_store.read().await`

Example:
```rust
let store_arc = self.state_store.read().await;
(*store_arc).save_spec(&spec_clone)?;
```

### Test Database Isolation

- Each test uses a unique database path to prevent conflicts
- State persistence test uses process ID in path: `format!("./test_workflow_db_persistence_{}", std::process::id())`
- Tests must drop engine instances before creating new ones with the same database path to release locks

### Runtime Issues Fixed

1. **Database Lock Conflicts**: Fixed `test_state_persistence` by dropping the first engine instance before creating the second one
2. **StateStore Type Mismatch**: Fixed compilation errors by properly dereferencing `Arc<StateStore>` when accessing methods
3. **Compiler Module Imports**: Fixed by using public re-exports from `parser` module instead of private modules

