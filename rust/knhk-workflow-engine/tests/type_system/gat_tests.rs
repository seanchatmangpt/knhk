//! GAT (Generic Associated Types) integration tests

#![cfg(feature = "type-system-v2")]

use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::patterns::{PatternExecutionContext, PatternExecutionResult};
use knhk_workflow_engine::types::gat::{BasicPatternExecutor, LegacyPatternAdapter, PatternExecutor};

#[tokio::test]
async fn test_basic_pattern_executor_gat() {
    let executor = BasicPatternExecutor::new(|_ctx| {
        Ok(PatternExecutionResult::ok(vec!["next_task".to_string()]))
    });

    let ctx = PatternExecutionContext::default();
    let result = executor.execute(&ctx).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
    assert_eq!(result.next_activities.len(), 1);
}

#[tokio::test]
async fn test_gat_executor_with_different_lifetimes() {
    let executor = BasicPatternExecutor::new(|ctx| {
        // Access context fields with proper lifetime tracking
        let _case_id = &ctx.case_id;
        Ok(PatternExecutionResult::ok(vec![]))
    });

    // Test with different context lifetimes
    {
        let ctx = PatternExecutionContext::default();
        let _result = executor.execute(&ctx).await;
    }

    {
        let ctx = PatternExecutionContext::default();
        let _result = executor.execute(&ctx).await;
    }
}

#[tokio::test]
async fn test_legacy_adapter_compatibility() {
    // Create a legacy executor
    struct LegacyExecutor;

    impl knhk_workflow_engine::patterns::PatternExecutor for LegacyExecutor {
        fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
            PatternExecutionResult::ok(vec!["legacy_task".to_string()])
        }
    }

    // Wrap in new GAT adapter
    let adapter = LegacyPatternAdapter::new(LegacyExecutor);
    let ctx = PatternExecutionContext::default();
    let result = adapter.execute(&ctx).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.next_activities.len(), 1);
    assert_eq!(result.next_activities[0], "legacy_task");
}

#[tokio::test]
async fn test_gat_executor_error_handling() {
    use knhk_workflow_engine::error::WorkflowError;

    let executor = BasicPatternExecutor::new(|_ctx| {
        Err(WorkflowError::Validation("test error".to_string()))
    });

    let ctx = PatternExecutionContext::default();
    let result = executor.execute(&ctx).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_gat_executor_zero_cost_size() {
    let executor = BasicPatternExecutor::new(|_ctx| {
        Ok(PatternExecutionResult::ok(vec![]))
    });

    // Verify zero-cost: executor should be minimal size
    let size = std::mem::size_of_val(&executor);
    assert!(size <= std::mem::size_of::<usize>() * 2);
}

#[tokio::test]
async fn test_multiple_gat_executors_composition() {
    let executor1 = BasicPatternExecutor::new(|_ctx| {
        Ok(PatternExecutionResult::ok(vec!["task1".to_string()]))
    });

    let executor2 = BasicPatternExecutor::new(|_ctx| {
        Ok(PatternExecutionResult::ok(vec!["task2".to_string()]))
    });

    let ctx = PatternExecutionContext::default();

    let result1 = executor1.execute(&ctx).await.unwrap();
    let result2 = executor2.execute(&ctx).await.unwrap();

    assert_eq!(result1.next_activities[0], "task1");
    assert_eq!(result2.next_activities[0], "task2");
}
