# Expanded Chicago TDD Framework for Workflows

**Date**: 2025-01-XX  
**Status**: ✅ **COMPREHENSIVE FRAMEWORK COMPLETE**

---

## Overview

The expanded Chicago TDD framework provides comprehensive testing utilities, builders, helpers, and macros for writing production-ready workflow tests following Chicago TDD principles.

---

## 🎯 Framework Components

### 1. Test Fixture (`WorkflowTestFixture`)

Enhanced test fixture with unique database paths and comprehensive workflow operations.

**Features**:
- Unique test database paths (prevents conflicts)
- Workflow registration and management
- Case creation and execution
- Pattern execution
- Comprehensive assertion helpers
- Automatic cleanup

**Usage**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::WorkflowTestFixture;

#[tokio::test]
async fn test_workflow_execution() {
    // Arrange: Set up test fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_test_workflow_spec();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await.unwrap();
    
    // Act: Execute case
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Case completes successfully
    fixture.assert_case_completed(&case);
    fixture.assert_case_running(&case); // or assert_case_failed, assert_case_cancelled
}
```

### 2. Pattern Test Helpers

Comprehensive helpers for testing workflow patterns.

**Available Helpers**:
- `create_test_registry()` - Create registry with all 43 patterns
- `create_test_context()` - Create empty execution context
- `create_test_context_with_vars()` - Create context with variables
- `create_test_context_for_workflow()` - Create context for specific workflow
- `assert_pattern_success()` - Assert pattern execution succeeded
- `assert_pattern_failure()` - Assert pattern execution failed
- `assert_pattern_has_next_state()` - Assert pattern set next state
- `assert_pattern_has_variable()` - Assert pattern result has variable
- `assert_pattern_variable_equals()` - Assert variable equals expected value

**Usage**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

#[test]
fn test_pattern_execution() {
    // Arrange: Create registry and context
    let registry = create_test_registry();
    let mut vars = HashMap::new();
    vars.insert("condition".to_string(), "true".to_string());
    let ctx = create_test_context_with_vars(vars);
    
    // Act: Execute pattern
    let result = registry.execute(&PatternId(4), &ctx)
        .expect("Pattern should be registered");
    
    // Assert: Pattern executed successfully
    assert_pattern_success(&result);
    assert_pattern_has_next_state(&result);
    assert_pattern_variable_equals(&result, "condition", "true");
}
```

### 3. Workflow Builders

Builders for creating test workflows and tasks.

**WorkflowSpecBuilder**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::WorkflowSpecBuilder;

let spec = WorkflowSpecBuilder::new("Test Workflow")
    .with_start_condition("condition:start")
    .with_end_condition("condition:end")
    .add_task(create_test_task())
    .build();
```

**TaskBuilder**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::TaskBuilder;
use knhk_workflow_engine::parser::{TaskType, SplitType, JoinType};

let task = TaskBuilder::new("task:approve", "Approve Order")
    .with_type(TaskType::Atomic)
    .with_split_type(SplitType::And)
    .with_join_type(JoinType::And)
    .with_max_ticks(8)
    .add_outgoing_flow("task:notify")
    .build();
```

### 4. Test Data Builders

Builders for creating realistic test data.

**TestDataBuilder**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::TestDataBuilder;

// Simple data
let data = TestDataBuilder::new()
    .with_var("order_id", "ORD-001")
    .with_var("amount", "100.00")
    .build_json();

// Business scenario data
let order_data = TestDataBuilder::new()
    .with_order_data("ORD-2024-001234", "149.99")
    .with_customer_data("CUST-789456")
    .build_json();

// Approval data
let approval_data = TestDataBuilder::new()
    .with_approval_data("REQ-001", "5000.00")
    .build_json();
```

### 5. Resource Test Helpers

Helpers for testing resource allocation.

**Usage**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

let role = create_test_role("approver", "Approver");
let capability = create_test_capability("approval", "Approval", 100);
let resource = create_test_resource("User1", vec![role], vec![capability]);
```

### 6. Worklet Test Helpers

Helpers for testing worklets.

**Usage**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::create_test_worklet;

let worklet = create_test_worklet(
    "Exception Handler",
    vec!["resource_unavailable".to_string()]
);
```

### 7. Performance Test Helpers

Helpers for performance testing and tick budget verification.

**Usage**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::PerformanceTestHelper;

#[test]
fn test_pattern_performance() {
    let perf = PerformanceTestHelper::new(8); // Max 8 ticks
    
    // Execute pattern
    let result = execute_pattern();
    
    // Verify tick budget
    perf.verify_tick_budget();
    assert!(perf.elapsed_ticks() <= 8);
}
```

### 8. Integration Test Helpers

Helpers for end-to-end integration testing.

**Usage**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::IntegrationTestHelper;

#[tokio::test]
async fn test_end_to_end_workflow() {
    let mut helper = IntegrationTestHelper::new().unwrap();
    
    let spec = create_test_workflow_spec();
    let data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .build_json();
    
    // Execute complete workflow
    let case = helper.execute_complete_workflow(spec, data).await.unwrap();
    
    // Verify result
    helper.fixture().assert_case_completed(&case);
}
```

### 9. Property-Based Testing

Property-based testing for workflow invariants.

**Usage**:
```rust
use knhk_workflow_engine::testing::chicago_tdd::WorkflowPropertyTester;

#[tokio::test]
async fn test_completion_property() {
    let mut tester = WorkflowPropertyTester::new(100).unwrap();
    let spec_id = register_test_workflow().await;
    
    // Test property: All cases eventually complete or fail
    let result = tester.test_completion_property(spec_id).await.unwrap();
    assert!(result, "All cases should complete or fail");
}
```

### 10. Test Macros

Macros for concise test writing.

**chicago_tdd_workflow_test!**:
```rust
use knhk_workflow_engine::chicago_tdd_workflow_test;

chicago_tdd_workflow_test!(test_approval_workflow, |fixture| async move {
    // Arrange
    let spec = create_approval_workflow();
    let spec_id = fixture.register_workflow(spec).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;
    
    // Act
    let case = fixture.execute_case(case_id).await?;
    
    // Assert
    fixture.assert_case_completed(&case);
    Ok(())
});
```

**chicago_tdd_pattern_test!**:
```rust
use knhk_workflow_engine::chicago_tdd_pattern_test;

chicago_tdd_pattern_test!(test_pattern_1_sequence, 1, |result| {
    assert_pattern_success(result);
    assert_pattern_has_next_state(result);
});
```

---

## 📚 Complete Example: Testing All 43 Patterns

```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

#[test]
fn test_all_patterns() {
    let registry = create_test_registry();
    
    for pattern_id in 1..=43 {
        let ctx = create_test_context();
        let result = registry.execute(&PatternId(pattern_id), &ctx)
            .expect(&format!("Pattern {} should be registered", pattern_id));
        
        assert_pattern_success(&result);
        assert_pattern_has_next_state(&result);
    }
}
```

---

## 📚 Complete Example: Business Acceptance Test

```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

#[tokio::test]
async fn test_order_processing_workflow() {
    // Arrange: Set up fixture and create realistic test data
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    let spec = WorkflowSpecBuilder::new("Order Processing")
        .with_start_condition("condition:start")
        .with_end_condition("condition:end")
        .add_task(
            TaskBuilder::new("task:validate", "Validate Order")
                .with_max_ticks(8)
                .build()
        )
        .add_task(
            TaskBuilder::new("task:process_payment", "Process Payment")
                .with_max_ticks(8)
                .build()
        )
        .build();
    
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    let test_data = TestDataBuilder::new()
        .with_order_data("ORD-2024-001234", "149.99")
        .with_customer_data("CUST-789456")
        .with_var("payment_method", "credit_card")
        .build_json();
    
    let case_id = fixture.create_case(spec_id, test_data).await.unwrap();
    
    // Act: Execute workflow
    let perf = PerformanceTestHelper::new(8);
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Verify completion and performance
    fixture.assert_case_completed(&case);
    perf.verify_tick_budget();
}
```

---

## 📚 Complete Example: Resource Allocation Test

```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

#[tokio::test]
async fn test_resource_allocation() {
    // Arrange: Set up fixture and resources
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    // Register resources
    let approver_role = create_test_role("approver", "Approver");
    let approval_capability = create_test_capability("approval", "Approval", 100);
    let resource = create_test_resource("User1", vec![approver_role], vec![approval_capability]);
    
    fixture.engine.resource_allocator()
        .register_resource(resource)
        .await
        .unwrap();
    
    // Create workflow with resource requirements
    let spec = create_approval_workflow_with_resources();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Act: Execute workflow
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await.unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Workflow completed with resource allocation
    fixture.assert_case_completed(&case);
}
```

---

## 📚 Complete Example: Worklet Exception Handling Test

```rust
use knhk_workflow_engine::testing::chicago_tdd::*;

#[tokio::test]
async fn test_worklet_exception_handling() {
    // Arrange: Set up fixture and worklet
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    let worklet = create_test_worklet(
        "Resource Unavailable Handler",
        vec!["resource_unavailable".to_string()]
    );
    
    fixture.engine.worklet_repository()
        .register(worklet)
        .await
        .unwrap();
    
    // Create workflow with exception worklet
    let spec = create_workflow_with_exception_worklet();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Act: Execute workflow that triggers exception
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await.unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Exception handled via worklet
    fixture.assert_case_completed(&case);
}
```

---

## 🎨 Framework Features Summary

### Test Fixtures
- ✅ `WorkflowTestFixture` - Complete workflow testing fixture
- ✅ Unique database paths (no conflicts)
- ✅ Workflow registration and management
- ✅ Case creation and execution
- ✅ Pattern execution
- ✅ Comprehensive assertion helpers

### Pattern Helpers
- ✅ `create_test_registry()` - Registry with all 43 patterns
- ✅ `create_test_context()` - Empty context
- ✅ `create_test_context_with_vars()` - Context with variables
- ✅ `create_test_context_for_workflow()` - Context for workflow
- ✅ `assert_pattern_success()` - Assert success
- ✅ `assert_pattern_failure()` - Assert failure
- ✅ `assert_pattern_has_next_state()` - Assert next state
- ✅ `assert_pattern_has_variable()` - Assert variable exists
- ✅ `assert_pattern_variable_equals()` - Assert variable value

### Builders
- ✅ `WorkflowSpecBuilder` - Build workflow specifications
- ✅ `TaskBuilder` - Build tasks
- ✅ `TestDataBuilder` - Build test data (order, customer, approval)

### Resource Helpers
- ✅ `create_test_resource()` - Create test resources
- ✅ `create_test_role()` - Create test roles
- ✅ `create_test_capability()` - Create test capabilities

### Worklet Helpers
- ✅ `create_test_worklet()` - Create test worklets

### Performance Helpers
- ✅ `PerformanceTestHelper` - Performance testing and tick budget verification

### Integration Helpers
- ✅ `IntegrationTestHelper` - End-to-end integration testing

### Property Testing
- ✅ `WorkflowPropertyTester` - Property-based testing for invariants

### Macros
- ✅ `chicago_tdd_workflow_test!` - Macro for workflow tests
- ✅ `chicago_tdd_pattern_test!` - Macro for pattern tests

---

## 🚀 Benefits

### For Developers
- **Less Boilerplate**: Builders and helpers reduce code duplication
- **Consistent Patterns**: All tests follow same structure
- **Realistic Data**: Test data builders create realistic scenarios
- **Easy Assertions**: Comprehensive assertion helpers
- **Performance Testing**: Built-in tick budget verification

### For Teams
- **Faster Test Writing**: Less time writing boilerplate
- **Better Coverage**: Tools encourage comprehensive testing
- **Maintainability**: Centralized helpers are easy to maintain
- **Documentation**: Tests serve as executable documentation

### For Organizations
- **Quality Assurance**: Comprehensive testing framework
- **Risk Reduction**: Better test coverage reduces production risks
- **Efficiency**: Faster test creation and maintenance
- **Standards**: Consistent testing patterns across team

---

## 📖 Usage Patterns

### Pattern 1: Simple Pattern Test
```rust
#[test]
fn test_pattern_1_sequence() {
    let registry = create_test_registry();
    let ctx = create_test_context();
    let result = registry.execute(&PatternId(1), &ctx).unwrap();
    assert_pattern_success(&result);
}
```

### Pattern 2: Pattern Test with Variables
```rust
#[test]
fn test_pattern_4_exclusive_choice() {
    let registry = create_test_registry();
    let mut vars = HashMap::new();
    vars.insert("condition".to_string(), "true".to_string());
    let ctx = create_test_context_with_vars(vars);
    let result = registry.execute(&PatternId(4), &ctx).unwrap();
    assert_pattern_success(&result);
    assert_pattern_variable_equals(&result, "condition", "true");
}
```

### Pattern 3: Workflow Test with Fixture
```rust
#[tokio::test]
async fn test_workflow_execution() {
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await.unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();
    fixture.assert_case_completed(&case);
}
```

### Pattern 4: Business Acceptance Test
```rust
#[tokio::test]
async fn test_order_processing() {
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_order_processing_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    let data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .build_json();
    
    let case_id = fixture.create_case(spec_id, data).await.unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();
    fixture.assert_case_completed(&case);
}
```

### Pattern 5: Performance Test
```rust
#[test]
fn test_pattern_performance() {
    let perf = PerformanceTestHelper::new(8);
    let registry = create_test_registry();
    let ctx = create_test_context();
    let _result = registry.execute(&PatternId(1), &ctx).unwrap();
    perf.verify_tick_budget();
}
```

### Pattern 6: Property-Based Test
```rust
#[tokio::test]
async fn test_completion_property() {
    let mut tester = WorkflowPropertyTester::new(100).unwrap();
    let spec_id = register_test_workflow().await;
    let result = tester.test_completion_property(spec_id).await.unwrap();
    assert!(result);
}
```

---

## 🔧 Advanced Features

### Custom Assertions

Create custom assertions for specific test scenarios:

```rust
fn assert_order_processed(result: &PatternExecutionResult) {
    assert_pattern_success(result);
    assert_pattern_has_variable(result, "order_id");
    assert_pattern_has_variable(result, "order_status");
    assert_pattern_variable_equals(result, "order_status", "processed");
}
```

### Test Data Factories

Create factories for common test scenarios:

```rust
fn create_order_test_data(order_id: &str, amount: &str) -> serde_json::Value {
    TestDataBuilder::new()
        .with_order_data(order_id, amount)
        .with_customer_data("CUST-001")
        .with_var("payment_method", "credit_card")
        .build_json()
}
```

### Workflow Factories

Create factories for common workflows:

```rust
fn create_approval_workflow() -> WorkflowSpec {
    WorkflowSpecBuilder::new("Approval Workflow")
        .add_task(
            TaskBuilder::new("task:approve", "Approve")
                .with_max_ticks(8)
                .build()
        )
        .build()
}
```

---

## 📊 Framework Statistics

- **Helper Functions**: 20+
- **Builders**: 3 (WorkflowSpec, Task, TestData)
- **Assertion Helpers**: 10+
- **Test Macros**: 2
- **Integration Helpers**: 1
- **Property Testers**: 1

---

## Summary

The expanded Chicago TDD framework provides:

1. ✅ **Comprehensive Test Fixtures**: Complete workflow testing infrastructure
2. ✅ **Pattern Test Helpers**: All helpers needed for pattern testing
3. ✅ **Workflow Builders**: Easy workflow and task creation
4. ✅ **Test Data Builders**: Realistic test data generation
5. ✅ **Resource Helpers**: Resource allocation testing support
6. ✅ **Worklet Helpers**: Worklet testing support
7. ✅ **Performance Helpers**: Tick budget verification
8. ✅ **Integration Helpers**: End-to-end testing support
9. ✅ **Property Testing**: Invariant verification
10. ✅ **Test Macros**: Concise test writing

All features follow Chicago TDD principles:
- State-based testing (not interaction-based)
- Real collaborators (no mocks)
- Behavior verification (outputs and invariants)
- AAA pattern (Arrange-Act-Assert)
- Production-ready implementations

---

**Last Updated**: 2025-01-XX  
**Status**: ✅ **EXPANDED FRAMEWORK COMPLETE**

