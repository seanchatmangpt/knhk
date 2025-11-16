//! Zero-cost abstraction integration tests

use knhk_workflow_engine::patterns::{PatternExecutionContext, PatternExecutionResult, PatternExecutor};
use knhk_workflow_engine::performance::zero_cost::{
    batch_dispatch, cached_dispatch, chain_dispatch, conditional_dispatch, metered_dispatch,
    pattern_dispatch, CachedDispatcher, DispatchSelector, DynamicDispatcher,
};
use knhk_workflow_engine::types::newtypes::TickCount;

struct SimpleExecutor;

impl PatternExecutor for SimpleExecutor {
    fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
        PatternExecutionResult::ok(vec!["next".to_string()])
    }
}

struct CountingExecutor {
    count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl PatternExecutor for CountingExecutor {
    fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        PatternExecutionResult::ok(vec!["next".to_string()])
    }
}

#[test]
fn test_pattern_dispatch() {
    let executor = SimpleExecutor;
    let ctx = PatternExecutionContext::default();

    let result = pattern_dispatch(&executor, &ctx);
    assert!(result.success);
    assert_eq!(result.next_activities.len(), 1);
}

#[test]
fn test_batch_dispatch() {
    let executor = SimpleExecutor;
    let contexts = vec![
        PatternExecutionContext::default(),
        PatternExecutionContext::default(),
        PatternExecutionContext::default(),
    ];

    let results = batch_dispatch(&executor, &contexts);
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.success));
}

#[test]
fn test_conditional_dispatch_true() {
    let executor = SimpleExecutor;
    let ctx = PatternExecutionContext::default();

    let result = conditional_dispatch(&executor, &ctx, |_| true);
    assert!(result.is_some());
    assert!(result.unwrap().success);
}

#[test]
fn test_conditional_dispatch_false() {
    let executor = SimpleExecutor;
    let ctx = PatternExecutionContext::default();

    let result = conditional_dispatch(&executor, &ctx, |_| false);
    assert!(result.is_none());
}

#[test]
fn test_metered_dispatch() {
    let executor = SimpleExecutor;
    let ctx = PatternExecutionContext::default();

    let (result, ticks) = metered_dispatch(&executor, &ctx);

    assert!(result.success);
    // Simple executor should be within Chatman Constant (≤8 ticks)
    assert!(ticks.is_within_budget());
}

#[test]
fn test_chain_dispatch() {
    let executor1 = SimpleExecutor;
    let executor2 = SimpleExecutor;
    let ctx = PatternExecutionContext::default();

    let (result1, result2) = chain_dispatch(&executor1, &executor2, &ctx);

    assert!(result1.success);
    assert!(result2.success);
}

#[test]
fn test_cached_dispatcher() {
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let executor = CountingExecutor {
        count: count.clone(),
    };
    let dispatcher = CachedDispatcher::new(executor);

    let ctx = PatternExecutionContext::default();

    // First call - cache miss
    let result1 = dispatcher.dispatch(&ctx);
    assert!(result1.success);
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);

    // Second call - cache hit (should not increment counter)
    let result2 = dispatcher.dispatch(&ctx);
    assert!(result2.success);
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn test_cached_dispatcher_clear() {
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let executor = CountingExecutor {
        count: count.clone(),
    };
    let dispatcher = CachedDispatcher::new(executor);

    let ctx = PatternExecutionContext::default();

    // First call
    let _result1 = dispatcher.dispatch(&ctx);
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);

    // Clear cache
    dispatcher.clear_cache();

    // Third call - cache miss again
    let _result2 = dispatcher.dispatch(&ctx);
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
}

#[test]
fn test_dynamic_dispatcher() {
    let executor = Box::new(SimpleExecutor) as Box<dyn PatternExecutor>;
    let dispatcher = DynamicDispatcher::new(executor);

    let ctx = PatternExecutionContext::default();
    let result = dispatcher.dispatch(&ctx);

    assert!(result.success);
}

#[test]
fn test_dispatch_selector_strategy_0() {
    let executor = SimpleExecutor;
    let ctx = PatternExecutionContext::default();

    let result = DispatchSelector::<0>::dispatch(&executor, &ctx);
    assert!(result.success);
}

#[test]
fn test_dispatch_selector_strategy_1() {
    let executor = SimpleExecutor;
    let ctx = PatternExecutionContext::default();

    let result = DispatchSelector::<1>::dispatch(&executor, &ctx);
    assert!(result.success);
}

#[test]
fn test_zero_cost_inline_optimization() {
    // This test verifies that inlining is happening correctly
    // by measuring execution time for a batch of operations
    let executor = SimpleExecutor;
    let contexts: Vec<_> = (0..1000)
        .map(|_| PatternExecutionContext::default())
        .collect();

    let start = std::time::Instant::now();
    let results = batch_dispatch(&executor, &contexts);
    let elapsed = start.elapsed();

    assert_eq!(results.len(), 1000);

    // With proper inlining, this should be very fast
    // (this is a smoke test, actual performance depends on hardware)
    assert!(elapsed.as_millis() < 100);
}

#[test]
fn test_metered_dispatch_tick_counting() {
    let executor = SimpleExecutor;
    let contexts: Vec<_> = (0..10)
        .map(|_| PatternExecutionContext::default())
        .collect();

    let tick_counts: Vec<TickCount> = contexts
        .iter()
        .map(|ctx| {
            let (_result, ticks) = metered_dispatch(&executor, ctx);
            ticks
        })
        .collect();

    // All simple executions should be within budget
    assert!(tick_counts.iter().all(|t| t.is_within_budget()));
}
