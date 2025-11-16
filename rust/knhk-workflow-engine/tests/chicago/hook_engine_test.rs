//! Chicago TDD Tests: Hook Engine Behavior
//!
//! Tests workflow hook lifecycle and execution.
//! Validates before/after hooks for tasks, cases, and patterns.

use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::hooks::{HookContext, HookRegistry, HookResult, HookType, WorkflowHook};
use knhk_workflow_engine::parser::TaskType;
use knhk_workflow_engine::testing::chicago_tdd::{
    create_simple_sequential_workflow, TaskBuilder, WorkflowSpecBuilder, WorkflowTestFixture,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[tokio::test]
async fn test_hook_before_task_execution_is_called() -> WorkflowResult<()> {
    // Arrange: Create workflow with hook tracking
    let mut fixture = WorkflowTestFixture::new()?;
    let call_count = Arc::new(AtomicU32::new(0));
    let call_count_clone = call_count.clone();

    // Create hook that increments counter
    let hook = WorkflowHook {
        id: "before_task_hook".to_string(),
        hook_type: HookType::BeforeTaskExecution,
        name: "Before Task Hook".to_string(),
        description: "Counts task executions".to_string(),
        hook_fn: Arc::new(move |_ctx: &HookContext| {
            let count = call_count_clone.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                HookResult::success()
            })
        }),
        enabled: true,
        priority: 0,
    };

    // Register hook with engine
    fixture.engine.hook_registry.register(hook).await?;

    let workflow = create_simple_sequential_workflow("hook_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute case
    let case = fixture.execute_case(case_id).await?;

    // Assert: Hook was called
    fixture.assert_case_completed(&case);
    let hook_calls = call_count.load(Ordering::SeqCst);
    assert!(
        hook_calls >= 1,
        "Before task hook should be called at least once, got {}",
        hook_calls
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_hook_after_task_execution_is_called() -> WorkflowResult<()> {
    // Arrange: Create workflow with after-task hook
    let mut fixture = WorkflowTestFixture::new()?;
    let call_count = Arc::new(AtomicU32::new(0));
    let call_count_clone = call_count.clone();

    let hook = WorkflowHook {
        id: "after_task_hook".to_string(),
        hook_type: HookType::AfterTaskExecution,
        name: "After Task Hook".to_string(),
        description: "Tracks task completions".to_string(),
        hook_fn: Arc::new(move |_ctx: &HookContext| {
            let count = call_count_clone.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                HookResult::success()
            })
        }),
        enabled: true,
        priority: 0,
    };

    fixture.engine.hook_registry.register(hook).await?;

    let workflow = create_simple_sequential_workflow("after_hook_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute case
    let case = fixture.execute_case(case_id).await?;

    // Assert: After hook was called
    fixture.assert_case_completed(&case);
    assert!(
        call_count.load(Ordering::SeqCst) >= 1,
        "After task hook should be called at least once"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_hook_can_modify_case_data() -> WorkflowResult<()> {
    // Arrange: Create hook that modifies data
    let mut fixture = WorkflowTestFixture::new()?;

    let hook = WorkflowHook {
        id: "data_modifier_hook".to_string(),
        hook_type: HookType::BeforeTaskExecution,
        name: "Data Modifier".to_string(),
        description: "Adds timestamp to data".to_string(),
        hook_fn: Arc::new(move |ctx: &HookContext| {
            let mut data = ctx.data.clone();
            data["hook_timestamp"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
            Box::pin(async move { HookResult::success_with_data(data) })
        }),
        enabled: true,
        priority: 0,
    };

    fixture.engine.hook_registry.register(hook).await?;

    let workflow = create_simple_sequential_workflow("data_mod_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture
        .create_case(spec_id, serde_json::json!({"initial": "data"}))
        .await?;

    // Act: Execute case
    let case = fixture.execute_case(case_id).await?;

    // Assert: Hook modified the data
    fixture.assert_case_completed(&case);
    assert!(
        case.data.get("hook_timestamp").is_some(),
        "Hook should add timestamp to case data"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_hook_can_prevent_execution() -> WorkflowResult<()> {
    // Arrange: Create hook that blocks execution
    let mut fixture = WorkflowTestFixture::new()?;

    let hook = WorkflowHook {
        id: "blocking_hook".to_string(),
        hook_type: HookType::BeforeTaskExecution,
        name: "Blocking Hook".to_string(),
        description: "Prevents task execution".to_string(),
        hook_fn: Arc::new(move |_ctx: &HookContext| {
            Box::pin(async move {
                HookResult::failure("Task execution blocked by policy".to_string())
            })
        }),
        enabled: true,
        priority: 0,
    };

    fixture.engine.hook_registry.register(hook).await?;

    let workflow = create_simple_sequential_workflow("blocked_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Attempt to execute case
    let result = fixture.execute_case(case_id).await;

    // Assert: Execution was blocked
    assert!(
        result.is_err() || result.unwrap().state == CaseState::Failed,
        "Hook should prevent case execution"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_hooks_execute_in_priority_order() -> WorkflowResult<()> {
    // Arrange: Create multiple hooks with different priorities
    let mut fixture = WorkflowTestFixture::new()?;
    let execution_order = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let order_clone_1 = execution_order.clone();
    let order_clone_2 = execution_order.clone();

    let hook_low_priority = WorkflowHook {
        id: "low_priority_hook".to_string(),
        hook_type: HookType::BeforeTaskExecution,
        name: "Low Priority".to_string(),
        description: "Executes second".to_string(),
        hook_fn: Arc::new(move |_ctx: &HookContext| {
            let order = order_clone_1.clone();
            Box::pin(async move {
                order.lock().await.push("low");
                HookResult::success()
            })
        }),
        enabled: true,
        priority: 10, // Higher number = lower priority
    };

    let hook_high_priority = WorkflowHook {
        id: "high_priority_hook".to_string(),
        hook_type: HookType::BeforeTaskExecution,
        name: "High Priority".to_string(),
        description: "Executes first".to_string(),
        hook_fn: Arc::new(move |_ctx: &HookContext| {
            let order = order_clone_2.clone();
            Box::pin(async move {
                order.lock().await.push("high");
                HookResult::success()
            })
        }),
        enabled: true,
        priority: 0, // Lower number = higher priority
    };

    fixture.engine.hook_registry.register(hook_low_priority).await?;
    fixture.engine.hook_registry.register(hook_high_priority).await?;

    let workflow = create_simple_sequential_workflow("priority_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute case
    let case = fixture.execute_case(case_id).await?;

    // Assert: Hooks executed in priority order
    fixture.assert_case_completed(&case);
    let order = execution_order.lock().await;
    assert!(
        !order.is_empty() && order[0] == "high",
        "High priority hook should execute first, got: {:?}",
        *order
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_disabled_hook_does_not_execute() -> WorkflowResult<()> {
    // Arrange: Create disabled hook
    let mut fixture = WorkflowTestFixture::new()?;
    let call_count = Arc::new(AtomicU32::new(0));
    let call_count_clone = call_count.clone();

    let hook = WorkflowHook {
        id: "disabled_hook".to_string(),
        hook_type: HookType::BeforeTaskExecution,
        name: "Disabled Hook".to_string(),
        description: "Should not execute".to_string(),
        hook_fn: Arc::new(move |_ctx: &HookContext| {
            let count = call_count_clone.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                HookResult::success()
            })
        }),
        enabled: false, // Disabled
        priority: 0,
    };

    fixture.engine.hook_registry.register(hook).await?;

    let workflow = create_simple_sequential_workflow("disabled_hook_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute case
    let case = fixture.execute_case(case_id).await?;

    // Assert: Disabled hook was not called
    fixture.assert_case_completed(&case);
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        0,
        "Disabled hook should not execute"
    );

    fixture.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn test_hook_receives_correct_context() -> WorkflowResult<()> {
    // Arrange: Create hook that validates context
    let mut fixture = WorkflowTestFixture::new()?;
    let context_validated = Arc::new(AtomicU32::new(0));
    let context_clone = context_validated.clone();

    let hook = WorkflowHook {
        id: "context_validator_hook".to_string(),
        hook_type: HookType::BeforeTaskExecution,
        name: "Context Validator".to_string(),
        description: "Validates hook context".to_string(),
        hook_fn: Arc::new(move |ctx: &HookContext| {
            let validated = context_clone.clone();
            let has_case_id = ctx.case_id.is_some();
            let has_task_id = ctx.task_id.is_some();
            Box::pin(async move {
                if has_case_id && has_task_id {
                    validated.fetch_add(1, Ordering::SeqCst);
                }
                HookResult::success()
            })
        }),
        enabled: true,
        priority: 0,
    };

    fixture.engine.hook_registry.register(hook).await?;

    let workflow = create_simple_sequential_workflow("context_workflow", "task_1", "Task 1");
    let spec_id = fixture.register_workflow(workflow).await?;
    let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

    // Act: Execute case
    let case = fixture.execute_case(case_id).await?;

    // Assert: Hook received correct context
    fixture.assert_case_completed(&case);
    assert!(
        context_validated.load(Ordering::SeqCst) >= 1,
        "Hook should receive valid context with case_id and task_id"
    );

    fixture.cleanup()?;
    Ok(())
}
