# Chicago TDD Using chicago-tdd-tools

## Overview

This guide demonstrates how to write Chicago TDD tests using the `chicago-tdd-tools` framework in the workflow engine.

## Chicago TDD Principles

1. **State-Based Testing**: Tests verify outputs and state, not implementation
2. **Real Collaborators**: Uses actual dependencies (WorkflowEngine, StateStore), not mocks
3. **Behavior Verification**: Tests verify what code does, not how
4. **AAA Pattern**: All tests follow Arrange-Act-Assert structure

## Framework Components

### 1. Test Fixtures (`chicago_tdd_tools::TestFixture`)

Generic test fixture with unique test counters and metadata management.

```rust
use chicago_tdd_tools::prelude::*;

let fixture = TestFixture::new().expect("Failed to create fixture");
let counter = fixture.test_counter();
fixture.set_metadata("key".to_string(), "value".to_string());
```

### 2. Test Data Builder (`chicago_tdd_tools::TestDataBuilder`)

Fluent builder for creating test data structures.

```rust
use chicago_tdd_tools::prelude::*;

let test_data = TestDataBuilder::new()
    .with_order_data("ORD-001", "100.00")
    .with_customer_data("CUST-001")
    .with_var("status", "pending")
    .build_json();
```

### 3. Assertion Helpers (`chicago_tdd_tools::assertions`)

Comprehensive assertion utilities.

```rust
use chicago_tdd_tools::prelude::*;

assert_success(&result);
assert_error(&result);
assert_eq_with_msg(&actual, &expected, "Custom message");
assert_in_range(&value, &min, &max, "Value should be in range");
```

### 4. Workflow-Specific Fixture (`WorkflowTestFixture`)

Extends generic fixture with workflow operations.

```rust
use knhk_workflow_engine::testing::chicago_tdd::WorkflowTestFixture;

let mut fixture = WorkflowTestFixture::new()?;
let spec_id = fixture.register_workflow(spec).await?;
let case_id = fixture.create_case(spec_id, test_data).await?;
let case = fixture.execute_case(case_id).await?;
```

## Example Test

```rust
use chicago_tdd_tools::prelude::*;
use knhk_workflow_engine::testing::chicago_tdd::*;

#[tokio::test]
async fn test_workflow_execution() -> WorkflowResult<()> {
    // Arrange: Create fixtures
    let base_fixture = TestFixture::new()
        .map_err(|e| WorkflowError::Internal(e.to_string()))?;
    let mut workflow_fixture = WorkflowTestFixture::new()?;
    
    // Arrange: Build test data
    let test_data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .build_json();
    
    // Arrange: Create workflow
    let spec = WorkflowSpecBuilder::new("Test Workflow")
        .add_task(TaskBuilder::new("task:1", "Process Order").build())
        .build();
    
    // Act: Register and execute workflow
    let spec_id = workflow_fixture.register_workflow(spec).await?;
    let case_id = workflow_fixture.create_case(spec_id, test_data).await?;
    let case = workflow_fixture.execute_case(case_id).await?;
    
    // Assert: Verify results
    assert_success(&Ok(case.state.clone()));
    assert_eq_with_msg(
        &case.state,
        &CaseState::Completed,
        "Case should complete successfully",
    );
    
    Ok(())
}
```

## Best Practices

1. **Use Real Collaborators**: Always use actual WorkflowEngine, StateStore, etc.
2. **Follow AAA Pattern**: Clearly separate Arrange, Act, and Assert sections
3. **Descriptive Test Names**: Test names should explain what is being tested
4. **State Verification**: Verify state changes, not implementation details
5. **Error Handling**: Use proper error handling with `Result` types
6. **Test Isolation**: Each test should be independent and isolated

## Integration Points

- **Generic Tools**: `chicago_tdd_tools::prelude::*` provides fixtures, builders, assertions
- **Workflow Extensions**: `knhk_workflow_engine::testing::chicago_tdd::*` provides workflow-specific helpers
- **Combined Usage**: Use both together for comprehensive testing

## Test File Location

Tests using chicago-tdd-tools should be placed in:
- `tests/chicago_tdd_tools_integration.rs` - Integration tests
- `tests/chicago_tdd_*.rs` - Pattern-specific tests

## Running Tests

```bash
# Run all Chicago TDD tests
cargo test --test chicago_tdd_tools_integration

# Run specific test
cargo test --test chicago_tdd_tools_integration test_workflow_execution_with_chicago_tdd_tools

# Run with output
cargo test --test chicago_tdd_tools_integration -- --nocapture
```

## Current Status

All 12 tests in `tests/chicago_tdd_tools_integration.rs` are passing:
- ✅ Workflow registration and retrieval
- ✅ Case creation and state transitions
- ✅ Multi-task workflow execution
- ✅ Error handling (invalid workflows, missing cases)
- ✅ Case cancellation
- ✅ Multiple workflows and cases
- ✅ State persistence
- ✅ Admission gate integration
- ✅ Engine services access
- ✅ Complete workflow lifecycle

## Known Patterns

### StateStore Access

When accessing `StateStore` from `WorkflowEngine`, use dereferencing:
```rust
let store = engine.state_store().read().await;
(*store).save_case(case_id, &case)?;
```

### Test Database Paths

Use unique paths for each test to avoid conflicts:
```rust
let db_path = format!("./test_db_{}", std::process::id());
let state_store = StateStore::new(&db_path)?;
```

