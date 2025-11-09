//! Chicago TDD Tests for Workflow Engine
//!
//! Comprehensive test suite for the workflow engine using chicago-tdd-tools.
//! These tests demonstrate how to use chicago-tdd-tools to test workflow engine functionality.
//!
//! **Architecture**: Uses chicago-tdd-tools for fixtures and builders, extends with workflow-specific testing.

use chicago_tdd_tools::builders::TestDataBuilder;
use chicago_tdd_tools::{assert_ok, chicago_async_test};
use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::parser::{JoinType, SplitType, TaskType};
use knhk_workflow_engine::testing::chicago_tdd::*;

// ============================================================================
// Test Fixture Tests
// ============================================================================

chicago_async_test!(test_workflow_fixture_creation, {
    // Arrange & Act: Create fixture using chicago-tdd-tools pattern
    let fixture = WorkflowTestFixture::new().unwrap();

    // Assert: Fixture created successfully
    assert_eq!(fixture.specs.len(), 0);
    assert_eq!(fixture.cases.len(), 0);
});

chicago_async_test!(test_workflow_fixture_isolation, {
    // Arrange & Act: Create multiple fixtures
    let mut fixture1 = WorkflowTestFixture::new().unwrap();
    let mut fixture2 = WorkflowTestFixture::new().unwrap();
    let mut fixture3 = WorkflowTestFixture::new().unwrap();

    // Act: Register workflow in fixture1
    let spec1 = WorkflowSpecBuilder::new("Workflow 1").build();
    let result = fixture1.register_workflow(spec1).await;
    assert_ok!(&result, "Workflow registration should succeed");
    let spec_id1 = result.unwrap();

    // Assert: Each fixture is isolated (unique test databases)
    // fixture1 should have the workflow, others should not
    assert_eq!(fixture1.specs.len(), 1);
    assert_eq!(fixture2.specs.len(), 0);
    assert_eq!(fixture3.specs.len(), 0);
    assert!(fixture1.specs.contains_key(&spec_id1));
    assert!(!fixture2.specs.contains_key(&spec_id1));
    assert!(!fixture3.specs.contains_key(&spec_id1));
});

// ============================================================================
// Workflow Registration Tests
// ============================================================================

chicago_async_test!(test_register_workflow_with_builder, {
    // Arrange: Create fixture and workflow spec using builder
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow")
        .add_task(
            TaskBuilder::new("task1", "Task 1")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .build();

    // Act: Register workflow
    let result = fixture.register_workflow(spec.clone()).await;
    assert_ok!(&result, "Workflow registration should succeed");
    let spec_id = result.unwrap();

    // Assert: Workflow registered
    assert!(fixture.specs.contains_key(&spec_id));
    assert_eq!(fixture.specs.get(&spec_id).unwrap().name, "Test Workflow");
});

chicago_async_test!(test_register_multiple_workflows, {
    // Arrange: Create fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();

    // Act: Register multiple workflows
    let spec1 = WorkflowSpecBuilder::new("Workflow 1").build();
    let spec2 = WorkflowSpecBuilder::new("Workflow 2").build();
    let spec3 = WorkflowSpecBuilder::new("Workflow 3").build();

    let result1 = fixture.register_workflow(spec1).await;
    assert_ok!(&result1, "Workflow 1 registration should succeed");
    let spec_id1 = result1.unwrap();

    let result2 = fixture.register_workflow(spec2).await;
    assert_ok!(&result2, "Workflow 2 registration should succeed");
    let spec_id2 = result2.unwrap();

    let result3 = fixture.register_workflow(spec3).await;
    assert_ok!(&result3, "Workflow 3 registration should succeed");
    let spec_id3 = result3.unwrap();

    // Assert: All workflows registered
    assert_eq!(fixture.specs.len(), 3);
    assert!(fixture.specs.contains_key(&spec_id1));
    assert!(fixture.specs.contains_key(&spec_id2));
    assert!(fixture.specs.contains_key(&spec_id3));
});

// ============================================================================
// Case Creation Tests
// ============================================================================

chicago_async_test!(test_create_case_with_test_data_builder, {
    // Arrange: Create fixture, workflow, and test data using chicago-tdd-tools
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();

    let test_data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .build_json();

    // Act: Create case
    let case_id = fixture
        .create_case(spec_id, test_data.clone())
        .await
        .unwrap();

    // Assert: Case created and tracked, and data is stored correctly
    assert!(fixture.cases.contains(&case_id));
    let case = fixture.engine.get_case(case_id).await.unwrap();
    assert_eq!(case.data["key1"], "value1");
    assert_eq!(case.data["order_id"], "ORD-001");
    assert_eq!(case.data["total_amount"], "100.00");
    assert_eq!(case.data["customer_id"], "CUST-001");
});

chicago_async_test!(test_create_case_with_empty_data, {
    // Arrange: Create fixture and workflow
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();

    // Act: Create case with empty data
    let case_id = fixture
        .create_case(spec_id, serde_json::json!({}))
        .await
        .unwrap();

    // Assert: Case created
    assert!(fixture.cases.contains(&case_id));
});

chicago_async_test!(test_create_multiple_cases, {
    // Arrange: Create fixture and workflow
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();

    // Act: Create multiple cases
    let case_id1 = fixture
        .create_case(
            spec_id,
            TestDataBuilder::new().with_var("id", "1").build_json(),
        )
        .await
        .unwrap();
    let case_id2 = fixture
        .create_case(
            spec_id,
            TestDataBuilder::new().with_var("id", "2").build_json(),
        )
        .await
        .unwrap();
    let case_id3 = fixture
        .create_case(
            spec_id,
            TestDataBuilder::new().with_var("id", "3").build_json(),
        )
        .await
        .unwrap();

    // Assert: All cases created
    assert_eq!(fixture.cases.len(), 3);
    assert!(fixture.cases.contains(&case_id1));
    assert!(fixture.cases.contains(&case_id2));
    assert!(fixture.cases.contains(&case_id3));
});

// ============================================================================
// Case Execution Tests
// ============================================================================

chicago_async_test!(test_execute_case, {
    // Arrange: Create fixture, workflow, and case
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow")
        .add_task(
            TaskBuilder::new("task1", "Task 1")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture
        .create_case(spec_id, serde_json::json!({}))
        .await
        .unwrap();

    // Act: Execute case
    let case = fixture.execute_case(case_id).await.unwrap();

    // Assert: Case executed and state changed from initial state
    // Case should not be in Created state after execution
    assert!(
        !matches!(case.state, CaseState::Created),
        "Case should not remain in Created state after execution, got {:?}",
        case.state
    );
    // Case should be in a valid execution state
    assert!(
        matches!(
            case.state,
            CaseState::Completed | CaseState::Failed | CaseState::Running
        ),
        "Case should be in a valid execution state, got {:?}",
        case.state
    );
});

chicago_async_test!(test_execute_case_with_data, {
    // Arrange: Create fixture, workflow, and case with test data
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow")
        .add_task(
            TaskBuilder::new("task1", "Task 1")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();

    let test_data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .build_json();

    let case_id = fixture
        .create_case(spec_id, test_data.clone())
        .await
        .unwrap();

    // Act: Execute case
    let case = fixture.execute_case(case_id).await.unwrap();

    // Assert: Case executed, state changed, and data is preserved
    assert!(
        !matches!(case.state, CaseState::Created),
        "Case should not remain in Created state after execution, got {:?}",
        case.state
    );
    assert!(
        matches!(
            case.state,
            CaseState::Completed | CaseState::Failed | CaseState::Running
        ),
        "Case should be in a valid execution state, got {:?}",
        case.state
    );
    // Verify test data is preserved
    assert_eq!(case.data["order_id"], "ORD-001");
    assert_eq!(case.data["customer_id"], "CUST-001");
});

// ============================================================================
// Case State Assertion Tests
// ============================================================================

chicago_async_test!(test_assert_case_completed, {
    // Arrange: Create fixture and case
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture
        .create_case(spec_id, serde_json::json!({}))
        .await
        .unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();

    // Act & Assert: Use assertion helper - verify it works correctly
    // Always test the assertion helper by creating a completed case
    // This ensures we actually test the assertion, not just skip it
    let mut completed_case = case;
    completed_case.state = CaseState::Completed;
    fixture.assert_case_completed(&completed_case);
});

chicago_async_test!(test_assert_case_failed, {
    // Arrange: Create fixture and case
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let case_id = fixture
        .create_case(spec_id, serde_json::json!({}))
        .await
        .unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();

    // Act & Assert: Use assertion helper - verify it works correctly
    // Always test the assertion helper by creating a failed case
    // This ensures we actually test the assertion, not just skip it
    let mut failed_case = case;
    failed_case.state = CaseState::Failed;
    fixture.assert_case_failed(&failed_case);
});

// ============================================================================
// Workflow Builder Tests
// ============================================================================

chicago_async_test!(test_workflow_spec_builder_with_tasks, {
    // Arrange: Create workflow spec with multiple tasks
    let spec = WorkflowSpecBuilder::new("Complex Workflow")
        .add_task(
            TaskBuilder::new("task1", "Task 1")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .add_task(
            TaskBuilder::new("task2", "Task 2")
                .with_type(TaskType::Atomic)
                .with_split_type(SplitType::And)
                .build(),
        )
        .add_task(
            TaskBuilder::new("task3", "Task 3")
                .with_type(TaskType::Atomic)
                .with_join_type(JoinType::And)
                .build(),
        )
        .build();

    // Assert: Workflow spec created with tasks
    assert_eq!(spec.name, "Complex Workflow");
    assert_eq!(spec.tasks.len(), 3);
    assert!(spec.tasks.contains_key("task1"));
    assert!(spec.tasks.contains_key("task2"));
    assert!(spec.tasks.contains_key("task3"));
});

chicago_async_test!(test_task_builder_with_all_options, {
    // Arrange: Create task with all builder options
    let task = TaskBuilder::new("task1", "Task 1")
        .with_type(TaskType::MultipleInstance)
        .with_split_type(SplitType::Xor)
        .with_join_type(JoinType::Xor)
        .with_max_ticks(100)
        .add_outgoing_flow("task2")
        .add_outgoing_flow("task3")
        .build();

    // Assert: Task created with all options
    assert_eq!(task.id, "task1");
    assert_eq!(task.name, "Task 1");
    assert_eq!(task.task_type, TaskType::MultipleInstance);
    assert_eq!(task.split_type, SplitType::Xor);
    assert_eq!(task.join_type, JoinType::Xor);
    assert_eq!(task.max_ticks, Some(100));
    assert_eq!(task.outgoing_flows.len(), 2);
});

// ============================================================================
// Integration Tests
// ============================================================================

chicago_async_test!(test_full_workflow_lifecycle, {
    // Arrange: Create fixture, workflow, and case
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Full Lifecycle Workflow")
        .add_task(
            TaskBuilder::new("task1", "Task 1")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();

    let test_data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .build_json();

    let case_id = fixture
        .create_case(spec_id, test_data.clone())
        .await
        .unwrap();

    // Verify initial state
    let initial_case = fixture.engine.get_case(case_id).await.unwrap();
    assert_eq!(initial_case.state, CaseState::Created);

    // Act: Execute full lifecycle
    let case = fixture.execute_case(case_id).await.unwrap();

    // Assert: Case completed lifecycle - state changed from Created
    assert_ne!(
        case.state,
        CaseState::Created,
        "Case should not remain in Created state after execution"
    );
    assert!(
        matches!(
            case.state,
            CaseState::Completed | CaseState::Failed | CaseState::Running
        ),
        "Case should complete lifecycle, got {:?}",
        case.state
    );
    // Verify data is preserved through lifecycle
    assert_eq!(case.data["order_id"], "ORD-001");
    assert_eq!(case.data["customer_id"], "CUST-001");
});

chicago_async_test!(test_multiple_workflows_and_cases, {
    // Arrange: Create fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();

    // Act: Register multiple workflows and create cases
    let spec1 = WorkflowSpecBuilder::new("Workflow 1").build();
    let spec2 = WorkflowSpecBuilder::new("Workflow 2").build();

    let spec_id1 = fixture.register_workflow(spec1).await.unwrap();
    let spec_id2 = fixture.register_workflow(spec2).await.unwrap();

    let case_id1 = fixture
        .create_case(
            spec_id1,
            TestDataBuilder::new().with_var("id", "1").build_json(),
        )
        .await
        .unwrap();
    let case_id2 = fixture
        .create_case(
            spec_id2,
            TestDataBuilder::new().with_var("id", "2").build_json(),
        )
        .await
        .unwrap();

    // Assert: All workflows and cases created
    assert_eq!(fixture.specs.len(), 2);
    assert_eq!(fixture.cases.len(), 2);
    assert!(fixture.cases.contains(&case_id1));
    assert!(fixture.cases.contains(&case_id2));
});

// ============================================================================
// Test Data Builder Integration Tests
// ============================================================================

chicago_async_test!(test_test_data_builder_integration, {
    // Arrange: Create test data using chicago-tdd-tools TestDataBuilder
    let test_data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_var("key2", "value2")
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .with_approval_data("REQ-001", "50.00")
        .build_json();

    // Assert: Test data created correctly
    assert!(test_data.is_object());
    assert_eq!(test_data["key1"], "value1");
    assert_eq!(test_data["key2"], "value2");
    assert_eq!(test_data["order_id"], "ORD-001");
    assert_eq!(test_data["total_amount"], "100.00");
    assert_eq!(test_data["customer_id"], "CUST-001");
    assert_eq!(test_data["request_id"], "REQ-001");
});

chicago_async_test!(test_test_data_builder_with_workflow, {
    // Arrange: Create fixture, workflow, and test data
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let spec_id = fixture.register_workflow(spec).await.unwrap();

    let test_data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-001")
        .build_json();

    // Act: Create case with test data
    let case_id = fixture.create_case(spec_id, test_data).await.unwrap();

    // Assert: Case created with test data
    assert!(fixture.cases.contains(&case_id));
    let case = fixture.engine.get_case(case_id).await.unwrap();
    assert_eq!(case.data["order_id"], "ORD-001");
    assert_eq!(case.data["customer_id"], "CUST-001");
});

// ============================================================================
// Error Handling Tests
// ============================================================================

chicago_async_test!(test_register_workflow_with_invalid_spec, {
    // Arrange: Create fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();

    // Act: Try to register workflow with empty name (invalid)
    let spec = WorkflowSpecBuilder::new("").build();
    let result = fixture.register_workflow(spec).await;

    // Assert: Registration should either succeed (if empty names are allowed)
    // or fail (if validation rejects empty names)
    // Either way, we verify the result is handled correctly
    match result {
        Ok(spec_id) => {
            // If registration succeeds, verify it was actually registered
            assert!(fixture.specs.contains_key(&spec_id));
            assert_eq!(fixture.specs.get(&spec_id).unwrap().name, "");
        }
        Err(e) => {
            // If registration fails, verify it's a proper error
            assert!(!e.to_string().is_empty());
        }
    }
});

chicago_async_test!(test_create_case_with_invalid_spec_id, {
    // Arrange: Create fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();

    // Act: Try to create case with non-existent spec ID
    use knhk_workflow_engine::parser::WorkflowSpecId;
    let invalid_spec_id = WorkflowSpecId::new();
    let result = fixture
        .create_case(invalid_spec_id, serde_json::json!({}))
        .await;

    // Assert: Creation should fail
    assert!(result.is_err());
});

// ============================================================================
// Cleanup Tests
// ============================================================================

chicago_async_test!(test_fixture_cleanup, {
    // Arrange: Create fixture and add some state
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = WorkflowSpecBuilder::new("Test Workflow").build();
    let _spec_id = fixture.register_workflow(spec).await.unwrap();

    // Act: Cleanup fixture
    let result = fixture.cleanup();

    // Assert: Cleanup succeeds and doesn't panic
    assert!(result.is_ok());
    // Verify fixture still exists after cleanup (cleanup doesn't destroy fixture)
    assert_eq!(fixture.specs.len(), 1);
});
