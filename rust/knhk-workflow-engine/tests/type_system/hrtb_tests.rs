//! HRTB (Higher-Ranked Trait Bounds) callback integration tests

use knhk_workflow_engine::callbacks::{CallbackExecutor, CallbackRegistry, CallbackRegistryBuilder};
use knhk_workflow_engine::error::WorkflowResult;
use knhk_workflow_engine::patterns::{PatternExecutionContext, PatternExecutionResult};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_hrtb_pre_callback() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let mut registry = CallbackRegistry::new();
    registry.register_pre("test_pre", move |_ctx| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });

    let ctx = PatternExecutionContext::default();
    registry.execute_pre(&ctx).expect("pre callback should succeed");

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_hrtb_post_callback() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let mut registry = CallbackRegistry::new();
    registry.register_post("test_post", move |_ctx| {
        counter_clone.fetch_add(10, Ordering::SeqCst);
        Ok(())
    });

    let ctx = PatternExecutionContext::default();
    registry.execute_post(&ctx).expect("post callback should succeed");

    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[test]
fn test_hrtb_error_callback() {
    use knhk_workflow_engine::error::WorkflowError;

    let error_caught = Arc::new(AtomicUsize::new(0));
    let error_clone = error_caught.clone();

    let mut registry = CallbackRegistry::new();
    registry.register_error("test_error", move |_ctx, _error| {
        error_clone.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });

    let ctx = PatternExecutionContext::default();
    let error = WorkflowError::Validation("test error".to_string());
    registry.execute_error(&ctx, &error).expect("error callback should succeed");

    assert_eq!(error_caught.load(Ordering::SeqCst), 1);
}

#[test]
fn test_hrtb_callback_registry_builder() {
    let pre_counter = Arc::new(AtomicUsize::new(0));
    let post_counter = Arc::new(AtomicUsize::new(0));

    let pre_clone = pre_counter.clone();
    let post_clone = post_counter.clone();

    let registry = CallbackRegistryBuilder::new()
        .with_pre("pre_callback", move |_ctx| {
            pre_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .with_post("post_callback", move |_ctx| {
            post_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .build();

    let (pre_count, post_count, error_count) = registry.count();
    assert_eq!(pre_count, 1);
    assert_eq!(post_count, 1);
    assert_eq!(error_count, 0);
}

#[test]
fn test_hrtb_callback_executor_success() {
    let pre_counter = Arc::new(AtomicUsize::new(0));
    let post_counter = Arc::new(AtomicUsize::new(0));

    let pre_clone = pre_counter.clone();
    let post_clone = post_counter.clone();

    let registry = CallbackRegistryBuilder::new()
        .with_pre("pre", move |_ctx| {
            pre_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .with_post("post", move |_ctx| {
            post_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .build();

    let executor = CallbackExecutor::new(Arc::new(registry));
    let ctx = PatternExecutionContext::default();

    let result = executor.execute_with_callbacks(&ctx, |_ctx| {
        Ok(PatternExecutionResult::ok(vec!["next".to_string()]))
    });

    assert!(result.is_ok());
    assert_eq!(pre_counter.load(Ordering::SeqCst), 1);
    assert_eq!(post_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_hrtb_callback_executor_error_handling() {
    use knhk_workflow_engine::error::WorkflowError;

    let error_counter = Arc::new(AtomicUsize::new(0));
    let error_clone = error_counter.clone();

    let registry = CallbackRegistryBuilder::new()
        .with_error("error", move |_ctx, _error| {
            error_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .build();

    let executor = CallbackExecutor::new(Arc::new(registry));
    let ctx = PatternExecutionContext::default();

    let result = executor.execute_with_callbacks(&ctx, |_ctx| {
        Err(WorkflowError::Validation("test error".to_string()))
    });

    assert!(result.is_err());
    assert_eq!(error_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_hrtb_multiple_callbacks_same_hook() {
    let counter = Arc::new(AtomicUsize::new(0));

    let counter1 = counter.clone();
    let counter2 = counter.clone();
    let counter3 = counter.clone();

    let mut registry = CallbackRegistry::new();

    registry.register_pre("callback1", move |_ctx| {
        counter1.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });

    registry.register_pre("callback2", move |_ctx| {
        counter2.fetch_add(2, Ordering::SeqCst);
        Ok(())
    });

    registry.register_pre("callback3", move |_ctx| {
        counter3.fetch_add(4, Ordering::SeqCst);
        Ok(())
    });

    let ctx = PatternExecutionContext::default();
    registry.execute_pre(&ctx).expect("callbacks should succeed");

    // 1 + 2 + 4 = 7
    assert_eq!(counter.load(Ordering::SeqCst), 7);
}

#[test]
fn test_hrtb_callback_removal() {
    let mut registry = CallbackRegistry::new();

    registry.register_pre("test", |_ctx| Ok(()));
    assert_eq!(registry.count().0, 1);

    let removed = registry.remove_pre("test");
    assert!(removed);
    assert_eq!(registry.count().0, 0);

    let removed_again = registry.remove_pre("test");
    assert!(!removed_again);
}

#[test]
fn test_hrtb_callback_clear() {
    let mut registry = CallbackRegistry::new();

    registry.register_pre("pre", |_ctx| Ok(()));
    registry.register_post("post", |_ctx| Ok(()));
    registry.register_error("error", |_ctx, _error| Ok(()));

    assert_eq!(registry.count(), (1, 1, 1));

    registry.clear();
    assert_eq!(registry.count(), (0, 0, 0));
}

#[test]
fn test_hrtb_callback_with_context_data() {
    let mut registry = CallbackRegistry::new();

    registry.register_pre("context_reader", |ctx| {
        // Verify we can access context data through HRTB
        let _ = &ctx.case_id;
        let _ = &ctx.workflow_id;
        let _ = &ctx.variables;
        Ok(())
    });

    let ctx = PatternExecutionContext::default();
    let result = registry.execute_pre(&ctx);
    assert!(result.is_ok());
}
