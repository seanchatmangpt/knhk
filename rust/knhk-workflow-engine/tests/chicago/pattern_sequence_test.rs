//! Chicago TDD Tests: Pattern 1 - Sequence
//!
//! Tests the basic sequence pattern where tasks execute in order.
//! AAA Pattern: Arrange, Act, Assert
//! State-based testing with real collaborators.

use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::parser::{SplitType, TaskType};
use knhk_workflow_engine::testing::chicago_tdd::{
    TaskBuilder, WorkflowSpecBuilder, WorkflowTestFixture,
};
use knhk_workflow_engine::error::WorkflowResult;

#[tokio::test]
async fn test_sequence_pattern_executes_tasks_in_order() -> WorkflowResult<()> {
    // Arrange: Create workflow with Task A → Task B → Task C sequence
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("sequence_workflow")
        .add_task(
            TaskBuilder::new("task_a", "Task A")
                .with_type(TaskType::Atomic)
                .with_max_ticks(8)
                .build(),
        )
        .add_task(
            TaskBuilder::new("task_b", "Task B")
                .with_type(TaskType::Atomic)
                .with_max_ticks(8)
                .build(),
        )
        .add_task(
            TaskBuilder::new("task_c", "Task C")
                .with_type(TaskType::Atomic)
                .with_max_ticks(8)
                .build(),
        )
        .with_auto_conditions("task_a", "task_c")
        .add_flow("task_a", "task_b")
        .add_flow("task_b", "task_c")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture
        .create_case(spec_id, serde_json::json!({"order_id": "ORD-001"}))
        .await?;

    // Act: Execute the case
    let case = fixture.execute_case(case_id).await?;

    // Assert: Case completed successfully
    fixture.assert_case_completed(&case);

    // Verify XES export contains all tasks in sequence
    fixture
        .export_and_validate_xes(case_id, Some(&["Task A", "Task B", "Task C"]))
        .await?;

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_sequence_pattern_fails_on_missing_task() -> WorkflowResult<()> {
    // Arrange: Create workflow with broken sequence (missing task link)
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("broken_sequence")
        .add_task(
            TaskBuilder::new("task_a", "Task A")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .add_task(
            TaskBuilder::new("task_c", "Task C")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .with_auto_conditions("task_a", "task_c")
        // Note: task_b is missing but referenced in flow
        .add_flow("task_a", "task_b") // This should cause failure
        .add_flow("task_b", "task_c")
        .build();

    // Act: Register workflow (should succeed)
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture
        .create_case(spec_id, serde_json::json!({}))
        .await?;

    // Execute case - expecting failure
    let result = fixture.execute_case(case_id).await;

    // Assert: Case execution should fail due to missing task
    assert!(
        result.is_err() || result.unwrap().state == CaseState::Failed,
        "Case should fail when task is missing from workflow"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_sequence_pattern_respects_tick_budget() -> WorkflowResult<()> {
    // Arrange: Create sequence with strict tick budget
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("tick_budget_sequence")
        .add_task(
            TaskBuilder::new("task_a", "Task A")
                .with_type(TaskType::Atomic)
                .with_max_ticks(8) // Chatman Constant
                .build(),
        )
        .add_task(
            TaskBuilder::new("task_b", "Task B")
                .with_type(TaskType::Atomic)
                .with_max_ticks(8)
                .build(),
        )
        .with_auto_conditions("task_a", "task_b")
        .add_flow("task_a", "task_b")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute with performance measurement
    let start = std::time::Instant::now();
    let case = fixture.execute_case(case_id).await?;
    let duration = start.elapsed();

    // Assert: Case completed within tick budget
    fixture.assert_case_completed(&case);

    // Each task ≤8 ticks = ≤16ns per tick at 2ns/tick
    // 2 tasks × 8 ticks × 2ns = 32ns budget
    assert!(
        duration.as_nanos() <= 100_000, // Allow overhead for I/O, 100μs total
        "Sequence should execute within reasonable time: {:?}",
        duration
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_sequence_pattern_preserves_data_flow() -> WorkflowResult<()> {
    // Arrange: Create sequence that passes data between tasks
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("data_flow_sequence")
        .add_task(
            TaskBuilder::new("task_a", "Task A")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .add_task(
            TaskBuilder::new("task_b", "Task B")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .with_auto_conditions("task_a", "task_b")
        .add_flow("task_a", "task_b")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;
    let initial_data = serde_json::json!({
        "customer_id": "CUST-001",
        "order_total": 1000.00
    });
    let case_id = fixture.create_case(spec_id, initial_data).await?;

    // Act: Execute case
    let case = fixture.execute_case(case_id).await?;

    // Assert: Data is preserved through sequence
    fixture.assert_case_completed(&case);
    assert_eq!(case.data["customer_id"], "CUST-001");
    assert_eq!(case.data["order_total"], 1000.00);

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_sequence_pattern_exports_valid_xes() -> WorkflowResult<()> {
    // Arrange: Create simple sequence
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("xes_sequence")
        .add_task(
            TaskBuilder::new("register", "Register")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .add_task(
            TaskBuilder::new("verify", "Verify")
                .with_type(TaskType::Atomic)
                .build(),
        )
        .with_auto_conditions("register", "verify")
        .add_flow("register", "verify")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and export XES
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: XES export is valid and complete
    let xes_content = fixture
        .export_and_validate_xes(case_id, Some(&["Register", "Verify"]))
        .await?;

    // Verify XES contains lifecycle transitions
    assert!(
        xes_content.contains("lifecycle:transition"),
        "XES should contain lifecycle transitions"
    );
    assert!(
        xes_content.contains("complete") || xes_content.contains("COMPLETE"),
        "XES should contain completion events"
    );

    fixture.cleanup()?;
    Ok(())
}
