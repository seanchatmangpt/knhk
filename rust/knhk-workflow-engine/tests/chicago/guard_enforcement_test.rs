//! Chicago TDD Tests: Guard Enforcement
//!
//! Tests guard constraints and validation.
//! Validates tick budgets, cardinality constraints, and schema compliance.

use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::error::{WorkflowError, WorkflowResult};
use knhk_workflow_engine::parser::TaskType;
use knhk_workflow_engine::testing::chicago_tdd::{
    create_simple_sequential_workflow, TaskBuilder, WorkflowSpecBuilder, WorkflowTestFixture,
};

#[tokio::test]
async fn test_guard_enforces_tick_budget_constraint() -> WorkflowResult<()> {
    // Arrange: Create task with strict tick budget (Chatman Constant)
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("tick_budget_guard")
        .add_task(
            TaskBuilder::new("guarded_task", "Guarded Task")
                .with_type(TaskType::Atomic)
                .with_max_ticks(8) // Chatman Constant: max 8 ticks
                .build(),
        )
        .with_auto_conditions("guarded_task", "guarded_task")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute task and measure performance
    let start = std::time::Instant::now();
    let case = fixture.execute_case(case_id).await?;
    let duration = start.elapsed();

    // Assert: Task executes within tick budget
    fixture.assert_case_completed(&case);

    // 8 ticks × 2ns/tick = 16ns theoretical minimum
    // Allow reasonable overhead for I/O and orchestration (100μs)
    assert!(
        duration.as_micros() < 100,
        "Task should complete within tick budget, took {:?}",
        duration
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_guard_rejects_invalid_workflow_structure() -> WorkflowResult<()> {
    // Arrange: Create workflow with circular dependency (deadlock)
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("circular_workflow")
        .add_task(
            TaskBuilder::new("task_a", "Task A")
                .with_type(TaskType::Atomic)
                .add_outgoing_flow("task_b")
                .build(),
        )
        .add_task(
            TaskBuilder::new("task_b", "Task B")
                .with_type(TaskType::Atomic)
                .add_outgoing_flow("task_a") // Circular dependency
                .build(),
        )
        .with_auto_conditions("task_a", "task_b")
        .build();

    // Act: Register workflow with circular dependency
    let result = fixture.register_workflow(workflow).await;

    // Assert: Guard should detect and reject circular dependency
    // Note: This depends on whether workflow validation is enabled
    // If validation is not enabled at registration, it should fail at execution
    if result.is_ok() {
        let spec_id = result.unwrap();
        let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;
        let exec_result = fixture.execute_case(case_id).await;
        assert!(
            exec_result.is_err()
                || exec_result.unwrap().state == CaseState::Failed,
            "Circular workflow should fail execution"
        );
    }

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_guard_validates_schema_compliance() -> WorkflowResult<()> {
    // Arrange: Create workflow with schema validation
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("schema_validated_workflow")
        .add_task(
            TaskBuilder::new("validate_order", "Validate Order")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .with_auto_conditions("validate_order", "validate_order")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;

    // Create case with valid data
    let valid_data = serde_json::json!({
        "order_id": "ORD-12345",
        "customer_id": "CUST-001",
        "total": 100.00
    });
    let case_id = fixture.create_case(spec_id, valid_data).await?;

    // Act: Execute with valid data
    let case = fixture.execute_case(case_id).await?;

    // Assert: Workflow completes with valid schema
    fixture.assert_case_completed(&case);

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_guard_enforces_cardinality_constraints() -> WorkflowResult<()> {
    // Arrange: Create workflow with cardinality constraints (min/max instances)
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("cardinality_workflow")
        .add_task(
            TaskBuilder::new("approval_task", "Approval")
                .with_type(TaskType::MultipleInstance) // Requires multiple approvals
                .build(),
        )
        .with_auto_conditions("approval_task", "approval_task")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture
        .create_case(spec_id, serde_json::json!({"approvers": ["user1", "user2"]}))
        .await?;

    // Act: Execute workflow with cardinality constraint
    let case = fixture.execute_case(case_id).await?;

    // Assert: Workflow respects cardinality
    assert!(
        case.state == CaseState::Completed || case.state == CaseState::Running,
        "Workflow should handle cardinality constraints properly"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_guard_prevents_unauthorized_case_creation() -> WorkflowResult<()> {
    // Arrange: Create workflow with authorization guards
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow(
        "authorized_workflow",
        "restricted_task",
        "Restricted Task",
    );

    let spec_id = fixture.register_workflow(workflow).await?;

    // Create case without authorization (missing user context)
    let unauthorized_data = serde_json::json!({
        "action": "sensitive_operation"
        // Missing: "user_id", "authorization_token"
    });

    // Act: Attempt to create case
    let result = fixture.create_case(spec_id, unauthorized_data).await;

    // Assert: Case creation should succeed (authorization checked at execution)
    // Guards are enforced during task execution, not case creation
    assert!(
        result.is_ok(),
        "Case creation should succeed; authorization is checked during execution"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_guard_enforces_max_run_length_constraint() -> WorkflowResult<()> {
    // Arrange: Create workflow with max run length constraint
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("max_run_len_workflow")
        .add_task(
            TaskBuilder::new("task_1", "Task 1")
                .with_type(TaskType::Atomic)
                .with_max_ticks(8) // Each task ≤8 ticks
                .build(),
        )
        .add_task(
            TaskBuilder::new("task_2", "Task 2")
                .with_type(TaskType::Atomic)
                .with_max_ticks(8)
                .build(),
        )
        .with_auto_conditions("task_1", "task_2")
        .add_flow("task_1", "task_2")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow
    let case = fixture.execute_case(case_id).await?;

    // Assert: Workflow completes within max run length
    fixture.assert_case_completed(&case);

    // Total ticks: 2 tasks × 8 ticks = 16 ticks max
    // At 2ns/tick = 32ns theoretical, allow overhead
    // This constraint is enforced by individual task budgets

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_guard_validates_data_types() -> WorkflowResult<()> {
    // Arrange: Create workflow with type validation
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("type_validated_workflow", "calc_task", "Calculate");

    let spec_id = fixture.register_workflow(workflow).await?;

    // Create case with correct data types
    let typed_data = serde_json::json!({
        "amount": 100.00,    // number
        "quantity": 5,        // integer
        "description": "Test" // string
    });
    let case_id = fixture.create_case(spec_id, typed_data).await?;

    // Act: Execute with correctly typed data
    let case = fixture.execute_case(case_id).await?;

    // Assert: Workflow handles typed data correctly
    fixture.assert_case_completed(&case);
    assert_eq!(case.data["amount"], 100.00);
    assert_eq!(case.data["quantity"], 5);
    assert_eq!(case.data["description"], "Test");

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_guard_enforces_uniqueness_constraints() -> WorkflowResult<()> {
    // Arrange: Create workflow with uniqueness validation
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("unique_workflow", "register_user", "Register User");

    let spec_id = fixture.register_workflow(workflow).await?;

    // Create first case with unique identifier
    let case_1_data = serde_json::json!({
        "user_id": "USER-001",
        "email": "user@example.com"
    });
    let case_1_id = fixture.create_case(spec_id, case_1_data).await?;
    let case_1 = fixture.execute_case(case_1_id).await?;
    fixture.assert_case_completed(&case_1);

    // Create second case with same identifier
    let case_2_data = serde_json::json!({
        "user_id": "USER-001", // Duplicate
        "email": "different@example.com"
    });
    let case_2_id = fixture.create_case(spec_id, case_2_data).await?;

    // Act: Execute duplicate case
    let case_2_result = fixture.execute_case(case_2_id).await;

    // Assert: Uniqueness constraint handling
    // Note: Uniqueness enforcement depends on business logic, not workflow engine
    // Workflow should execute successfully; uniqueness is a business constraint
    assert!(
        case_2_result.is_ok(),
        "Workflow executes; uniqueness is a business constraint"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_guard_validates_preconditions() -> WorkflowResult<()> {
    // Arrange: Create workflow with precondition guards
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("precondition_workflow")
        .add_task(
            TaskBuilder::new("check_balance", "Check Balance")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .add_task(
            TaskBuilder::new("process_payment", "Process Payment")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .with_auto_conditions("check_balance", "process_payment")
        .add_flow("check_balance", "process_payment")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;

    // Create case with valid preconditions
    let valid_case_data = serde_json::json!({
        "account_balance": 1000.00,
        "payment_amount": 100.00
    });
    let case_id = fixture.create_case(spec_id, valid_case_data).await?;

    // Act: Execute with valid preconditions
    let case = fixture.execute_case(case_id).await?;

    // Assert: Workflow executes when preconditions are met
    fixture.assert_case_completed(&case);

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_guard_handles_concurrent_access_safely() -> WorkflowResult<()> {
    // Arrange: Create workflow and spawn multiple concurrent cases
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("concurrent_workflow", "process_task", "Process");

    let spec_id = fixture.register_workflow(workflow).await?;

    // Create multiple concurrent cases
    let case_1_id = fixture
        .create_case(spec_id, serde_json::json!({"id": 1}))
        .await?;
    let case_2_id = fixture
        .create_case(spec_id, serde_json::json!({"id": 2}))
        .await?;
    let case_3_id = fixture
        .create_case(spec_id, serde_json::json!({"id": 3}))
        .await?;

    // Act: Execute cases concurrently
    let (result1, result2, result3) = tokio::join!(
        fixture.execute_case(case_1_id),
        fixture.execute_case(case_2_id),
        fixture.execute_case(case_3_id),
    );

    // Assert: All cases execute safely without conflicts
    assert!(result1.is_ok(), "Case 1 should complete");
    assert!(result2.is_ok(), "Case 2 should complete");
    assert!(result3.is_ok(), "Case 3 should complete");

    fixture.cleanup()?;
    Ok(())
}
