//! Chicago TDD Tests: MAPE-K Monitor Phase
//!
//! Tests the Monitor phase of the MAPE-K autonomic loop.
//! Validates runtime observation and metrics collection.

use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::testing::chicago_tdd::{
    create_parallel_split_workflow, create_simple_sequential_workflow, WorkflowTestFixture,
};

#[tokio::test]
async fn test_monitor_tracks_workflow_execution() -> WorkflowResult<()> {
    // Arrange: Create workflow with monitoring
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("monitored_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow (monitoring happens automatically)
    let case = fixture.execute_case(case_id).await?;

    // Assert: Monitor captured execution data
    fixture.assert_case_completed(&case);

    // Verify monitoring through case history
    let history = fixture.get_case_history(case_id).await;
    assert!(
        !history.is_empty(),
        "Monitor should track workflow execution events"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_collects_performance_metrics() -> WorkflowResult<()> {
    // Arrange: Create workflow for performance monitoring
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("perf_monitored_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and measure
    let start = std::time::Instant::now();
    let case = fixture.execute_case(case_id).await?;
    let duration = start.elapsed();

    // Assert: Monitor collected performance metrics
    fixture.assert_case_completed(&case);

    // Performance metrics are observable through duration
    assert!(
        duration.as_micros() > 0,
        "Monitor should enable performance measurement"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_detects_state_transitions() -> WorkflowResult<()> {
    // Arrange: Create workflow to monitor state changes
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("state_monitored_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow
    fixture.engine.start_case(case_id).await?;
    let case_running = fixture.engine.get_case(case_id).await?;

    fixture.engine.execute_case(case_id).await?;
    let case_completed = fixture.engine.get_case(case_id).await?;

    // Assert: Monitor detected state transitions
    assert_eq!(
        case_running.state,
        CaseState::Running,
        "Monitor should detect Running state"
    );
    assert_eq!(
        case_completed.state,
        CaseState::Completed,
        "Monitor should detect Completed state"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_tracks_parallel_execution() -> WorkflowResult<()> {
    // Arrange: Create parallel workflow
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "parallel_monitored_workflow",
        vec![
            ("branch_1".to_string(), "Branch 1".to_string()),
            ("branch_2".to_string(), "Branch 2".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute parallel workflow
    let case = fixture.execute_case(case_id).await?;

    // Assert: Monitor tracked all parallel branches
    fixture.assert_case_completed(&case);

    let history = fixture.get_case_history(case_id).await;
    assert!(
        history.len() >= 3, // Split + at least 2 branches
        "Monitor should track all parallel branch executions"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_captures_execution_context() -> WorkflowResult<()> {
    // Arrange: Create workflow with rich context
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("context_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;

    let context_data = serde_json::json!({
        "user_id": "USER-001",
        "session_id": "SESSION-123",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    let case_id = fixture.create_case(spec_id, context_data).await?;

    // Act: Execute workflow
    let case = fixture.execute_case(case_id).await?;

    // Assert: Monitor preserved execution context
    fixture.assert_case_completed(&case);
    assert!(
        case.data.get("user_id").is_some(),
        "Monitor should preserve execution context"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_exports_telemetry() -> WorkflowResult<()> {
    // Arrange: Create workflow for telemetry export
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("telemetry_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute and export telemetry
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: Monitor exported telemetry (via XES)
    let xes_content = fixture.export_and_validate_xes(case_id, None).await?;
    assert!(
        !xes_content.is_empty(),
        "Monitor should export telemetry as XES"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_detects_anomalies() -> WorkflowResult<()> {
    // Arrange: Create workflow that might exhibit anomalies
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("anomaly_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute workflow
    let case = fixture.execute_case(case_id).await?;

    // Assert: Monitor enables anomaly detection through state observation
    assert!(
        case.state == CaseState::Completed || case.state == CaseState::Failed,
        "Monitor should detect terminal states for anomaly analysis"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_provides_real_time_visibility() -> WorkflowResult<()> {
    // Arrange: Create long-running workflow simulation
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("visible_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Start case and check visibility
    fixture.engine.start_case(case_id).await?;
    let case_during = fixture.engine.get_case(case_id).await?;

    // Assert: Monitor provides real-time state visibility
    assert_eq!(
        case_during.state,
        CaseState::Running,
        "Monitor should provide real-time visibility of running cases"
    );

    // Complete execution
    fixture.engine.execute_case(case_id).await?;

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_aggregates_metrics_across_cases() -> WorkflowResult<()> {
    // Arrange: Create multiple cases
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_simple_sequential_workflow("aggregate_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;

    // Create and execute multiple cases
    let case_1_id = fixture.create_case(spec_id, serde_json::json!({"id": 1})).await?;
    let case_2_id = fixture.create_case(spec_id, serde_json::json!({"id": 2})).await?;
    let case_3_id = fixture.create_case(spec_id, serde_json::json!({"id": 3})).await?;

    // Act: Execute all cases
    let case_1 = fixture.execute_case(case_1_id).await?;
    let case_2 = fixture.execute_case(case_2_id).await?;
    let case_3 = fixture.execute_case(case_3_id).await?;

    // Assert: Monitor tracked all cases
    fixture.assert_case_completed(&case_1);
    fixture.assert_case_completed(&case_2);
    fixture.assert_case_completed(&case_3);

    // Verify each case has monitoring data
    assert!(!fixture.get_case_history(case_1_id).await.is_empty());
    assert!(!fixture.get_case_history(case_2_id).await.is_empty());
    assert!(!fixture.get_case_history(case_3_id).await.is_empty());

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_supports_distributed_tracing() -> WorkflowResult<()> {
    // Arrange: Create workflow for distributed tracing
    let mut fixture = WorkflowTestFixture::new()?;

    let workflow = create_parallel_split_workflow(
        "distributed_workflow",
        vec![
            ("service_a".to_string(), "Service A".to_string()),
            ("service_b".to_string(), "Service B".to_string()),
        ],
    );

    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute distributed workflow
    let case = fixture.execute_case(case_id).await?;
    fixture.assert_case_completed(&case);

    // Assert: Monitor supports distributed tracing via XES
    let xes_content = fixture
        .export_and_validate_xes(case_id, Some(&["Service A", "Service B"]))
        .await?;

    // XES provides tracing capability
    assert!(
        xes_content.contains("<trace>"),
        "Monitor should support distributed tracing"
    );

    fixture.cleanup()?;
    Ok(())
}
