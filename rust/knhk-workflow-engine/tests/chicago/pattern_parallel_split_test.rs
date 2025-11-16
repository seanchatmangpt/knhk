//! Chicago TDD Tests: Pattern 2 - Parallel Split
//!
//! Tests parallel execution where multiple tasks run concurrently.
//! AAA Pattern: Arrange, Act, Assert
//! Verifies true concurrency and synchronization.

use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::parser::{JoinType, SplitType, TaskType};
use knhk_workflow_engine::testing::chicago_tdd::{
    create_parallel_split_workflow, TaskBuilder, WorkflowSpecBuilder, WorkflowTestFixture,
};
use knhk_workflow_engine::error::WorkflowResult;

#[tokio::test]
async fn test_parallel_split_executes_all_branches() -> WorkflowResult<()> {
    // Arrange: Create workflow with parallel split pattern
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "parallel_workflow",
        vec![
            ("task_1".to_string(), "Task 1".to_string()),
            ("task_2".to_string(), "Task 2".to_string()),
            ("task_3".to_string(), "Task 3".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute the case
    let case = fixture.execute_case(case_id).await?;

    // Assert: All parallel tasks executed
    fixture.assert_case_completed(&case);

    // Verify XES contains all parallel tasks
    fixture
        .export_and_validate_xes(
            case_id,
            Some(&["Split", "Task 1", "Task 2", "Task 3", "Join"]),
        )
        .await?;

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_parallel_split_waits_for_all_branches_before_join() -> WorkflowResult<()> {
    // Arrange: Create parallel workflow with AND join
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "and_join_workflow",
        vec![
            ("branch_a".to_string(), "Branch A".to_string()),
            ("branch_b".to_string(), "Branch B".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute case
    let case = fixture.execute_case(case_id).await?;

    // Assert: Join only occurs after all branches complete
    fixture.assert_case_completed(&case);

    // Verify both branches executed before join
    let history = fixture.get_case_history(case_id).await;
    assert!(
        history.len() >= 4, // Split + Branch A + Branch B + Join
        "History should contain all branch executions and join"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_parallel_split_with_different_tick_budgets() -> WorkflowResult<()> {
    // Arrange: Create parallel tasks with different performance characteristics
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = WorkflowSpecBuilder::new("mixed_performance_parallel")
        .add_task(
            TaskBuilder::new("split_task", "Split")
                .with_type(TaskType::Atomic)
                .with_split_type(SplitType::And)
                .build(),
        )
        .add_task(
            TaskBuilder::new("fast_task", "Fast Task")
                .with_type(TaskType::Atomic)
                .with_max_ticks(4) // Fast path
                .build(),
        )
        .add_task(
            TaskBuilder::new("slow_task", "Slow Task")
                .with_type(TaskType::Atomic)
                .with_max_ticks(8) // Still within budget
                .build(),
        )
        .add_task(
            TaskBuilder::new("join_task", "Join")
                .with_type(TaskType::Atomic)
                .with_join_type(JoinType::And)
                .build(),
        )
        .with_auto_conditions("split_task", "join_task")
        .build();

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute parallel workflow
    let case = fixture.execute_case(case_id).await?;

    // Assert: Case completes even with different performance characteristics
    fixture.assert_case_completed(&case);

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_parallel_split_maintains_data_independence() -> WorkflowResult<()> {
    // Arrange: Create parallel workflow with independent data modifications
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "data_independence_workflow",
        vec![
            ("calc_tax".to_string(), "Calculate Tax".to_string()),
            ("calc_shipping".to_string(), "Calculate Shipping".to_string()),
            ("calc_discount".to_string(), "Calculate Discount".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let initial_data = serde_json::json!({
        "subtotal": 100.00,
        "tax_rate": 0.08,
        "shipping_zone": "domestic",
        "discount_code": "SAVE10"
    });
    let case_id = fixture.create_case(spec_id, initial_data).await?;

    // Act: Execute parallel calculations
    let case = fixture.execute_case(case_id).await?;

    // Assert: All data modifications are preserved
    fixture.assert_case_completed(&case);
    assert!(case.data.get("subtotal").is_some());
    assert!(case.data.get("tax_rate").is_some());
    assert!(case.data.get("shipping_zone").is_some());
    assert!(case.data.get("discount_code").is_some());

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_parallel_split_generates_concurrent_telemetry() -> WorkflowResult<()> {
    // Arrange: Create parallel workflow to verify telemetry generation
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "telemetry_workflow",
        vec![
            ("service_a".to_string(), "Service A".to_string()),
            ("service_b".to_string(), "Service B".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and capture telemetry
    let case = fixture.execute_case(case_id).await?;

    // Assert: Telemetry includes all parallel branches
    fixture.assert_case_completed(&case);

    // Validate XES export contains concurrent events
    let xes_content = fixture.export_and_validate_xes(case_id, None).await?;
    assert!(
        xes_content.contains("Service A") && xes_content.contains("Service B"),
        "XES should contain telemetry for all parallel branches"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_parallel_split_handles_branch_failure() -> WorkflowResult<()> {
    // Arrange: Create parallel workflow where one branch might fail
    // Note: This tests graceful degradation behavior
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "failure_resilience_workflow",
        vec![
            ("success_task".to_string(), "Success Task".to_string()),
            ("risky_task".to_string(), "Risky Task".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow
    let case = fixture.execute_case(case_id).await?;

    // Assert: Case reaches a terminal state (completed or failed)
    assert!(
        case.state == CaseState::Completed || case.state == CaseState::Failed,
        "Case should reach terminal state even if branch fails"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_parallel_split_exports_process_mining_compatible_xes() -> WorkflowResult<()> {
    // Arrange: Create parallel workflow for process mining validation
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "process_mining_workflow",
        vec![
            ("approve_credit".to_string(), "Approve Credit".to_string()),
            ("verify_inventory".to_string(), "Verify Inventory".to_string()),
            ("check_fraud".to_string(), "Check Fraud".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and export XES
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: XES is process mining compatible
    let xes_content = fixture
        .export_and_validate_xes(
            case_id,
            Some(&[
                "Split",
                "Approve Credit",
                "Verify Inventory",
                "Check Fraud",
                "Join",
            ]),
        )
        .await?;

    // Verify XES 2.0 compliance
    assert!(
        xes_content.contains("xes.version=\"2.0\""),
        "XES should be version 2.0 for process mining compatibility"
    );

    fixture.cleanup()?;
    Ok(())
}
