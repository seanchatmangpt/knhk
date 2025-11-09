# Chicago TDD Tests for WorkflowEngine

**Status**: Production-Ready

---

## Overview

Comprehensive test suite using Chicago TDD methodology with `chicago-tdd-tools`.

---

## Test Coverage

### Core Tests

1. **Workflow Registration and Retrieval**
   - Register, retrieve, list workflows
   - State persistence

2. **Case Creation and State Transitions**
   - Create case, state transitions (Created → Running → Completed)
   - Case retrieval

3. **Multi-Task Workflow Execution**
   - Execute workflows with multiple tasks
   - Task execution order

4. **Error Handling**
   - Invalid workflows
   - Missing workflows/cases
   - Proper error types

5. **Case Cancellation**
   - Cancel running cases
   - State transition to Cancelled

6. **Complete Workflow Lifecycle**
   - Register → Create → Start → Execute
   - End-to-end execution

---

## Chicago TDD Principles

1. **State-Based Testing**: Verify state changes, not implementation details
2. **Real Collaborators**: Use actual `WorkflowEngine`, `StateStore` (no mocks)
3. **Behavior Verification**: Test what the engine does, not how
4. **AAA Pattern**: Arrange-Act-Assert structure

---

## Running Tests

```bash
# Run all tests
cargo test --test chicago_tdd_tools_integration

# Run specific test
cargo test --test chicago_tdd_tools_integration test_workflow_registration_and_retrieval

# Run with output
cargo test --test chicago_tdd_tools_integration -- --nocapture
```

---

## Test Example

```rust
#[tokio::test]
async fn test_workflow_registration_and_retrieval() {
    // Arrange
    let state_store = StateStore::new("./test_db").unwrap();
    let engine = Arc::new(WorkflowEngine::new(state_store));
    let spec = create_test_workflow_spec();
    
    // Act
    engine.register_workflow(spec.clone()).await.unwrap();
    let retrieved = engine.get_workflow(spec.id).await.unwrap();
    
    // Assert
    assert_eq!(retrieved.id, spec.id);
    assert_eq!(retrieved.name, spec.name);
}
```

---

## Framework Integration

Uses `chicago_tdd_tools::prelude::*`:
- `TestFixture` - Generic test fixtures
- `TestDataBuilder` - Fluent test data builders
- Assertion helpers (`assert_success`, `assert_error`)

---

## Key Behaviors Tested

- **Workflow Management**: Registration, retrieval, listing, persistence
- **Case Management**: Creation, execution, cancellation, state transitions
- **Error Handling**: Invalid workflows, missing resources, proper error types
- **Integration**: Admission gate, state persistence, service access

---

## Notes

- Each test uses a unique database path to avoid conflicts
- Tests use real collaborators (no mocks)
- Tests verify behavior, not implementation details
- State persistence test uses process ID in path for isolation

---

**License**: MIT
