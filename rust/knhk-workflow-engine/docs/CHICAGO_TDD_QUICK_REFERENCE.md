# Chicago TDD Framework Quick Reference

## Test Fixture

```rust
use knhk_workflow_engine::testing::chicago_tdd::WorkflowTestFixture;

let mut fixture = WorkflowTestFixture::new().unwrap();
let spec_id = fixture.register_workflow(spec).await.unwrap();
let case_id = fixture.create_case(spec_id, data).await.unwrap();
let case = fixture.execute_case(case_id).await.unwrap();
fixture.assert_case_completed(&case);
```

## Pattern Helpers

```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

let registry = create_test_registry();
let ctx = create_test_context();
let result = registry.execute(&PatternId(1), &ctx).unwrap();
assert_pattern_success(&result);
assert_pattern_has_next_state(&result);
```

## Workflow Builders

```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

let spec = WorkflowSpecBuilder::new("My Workflow")
    .with_start_condition("condition:start")
    .add_task(TaskBuilder::new("task:1", "Task 1").build())
    .build();
```

## Test Data Builders

```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

let data = TestDataBuilder::new()
    .with_order_data("ORD-001", "100.00")
    .with_customer_data("CUST-001")
    .build_json();
```

## Resource Helpers

```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

let role = create_test_role("approver", "Approver");
let capability = create_test_capability("approval", "Approval", 100);
let resource = create_test_resource("User1", vec![role], vec![capability]);
```

## Performance Testing

```rust
use knhk_workflow_engine::testing::chicago_tdd::PerformanceTestHelper;

let perf = PerformanceTestHelper::new(8);
// ... execute pattern ...
perf.verify_tick_budget();
```

## Test Macros

```rust
use knhk_workflow_engine::chicago_tdd_workflow_test;

chicago_tdd_workflow_test!(test_my_workflow, |fixture| async move {
    // Test code here
    Ok(())
});
```

## Complete Example

```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

#[tokio::test]
async fn test_complete_workflow() {
    // Arrange
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .build_json();
    let case_id = fixture.create_case(spec_id, data).await.unwrap();
    
    // Act
    let perf = PerformanceTestHelper::new(8);
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert
    fixture.assert_case_completed(&case);
    perf.verify_tick_budget();
}
```

For detailed documentation, see [CHICAGO_TDD_FRAMEWORK_EXPANDED.md](docs/CHICAGO_TDD_FRAMEWORK_EXPANDED.md)

