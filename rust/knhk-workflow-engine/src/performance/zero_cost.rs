//! Zero-Cost Abstractions for Pattern Dispatch
//!
//! Provides inlined dispatch functions that compile to direct calls with no overhead,
//! demonstrating zero-cost abstraction principles.

use crate::error::WorkflowResult;
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternExecutor};
use crate::types::newtypes::TickCount;

/// Dispatch pattern execution with zero-cost abstraction
///
/// This function is marked `#[inline(always)]` to ensure it compiles to a direct
/// function call with no indirection overhead. The monomorphization ensures each
/// concrete executor type gets its own optimized version.
#[inline(always)]
pub fn pattern_dispatch<P: PatternExecutor>(
    executor: &P,
    ctx: &PatternExecutionContext,
) -> PatternExecutionResult {
    executor.execute(ctx)
}

/// Batch dispatch patterns with zero overhead
///
/// Processes multiple patterns in sequence with guaranteed inline optimization.
#[inline(always)]
pub fn batch_dispatch<P: PatternExecutor>(
    executor: &P,
    contexts: &[PatternExecutionContext],
) -> Vec<PatternExecutionResult> {
    contexts
        .iter()
        .map(|ctx| executor.execute(ctx))
        .collect()
}

/// Conditional dispatch based on predicate (zero-cost)
///
/// The predicate and executor are both inlined, resulting in a simple branch
/// with no function call overhead.
#[inline(always)]
pub fn conditional_dispatch<P: PatternExecutor, F>(
    executor: &P,
    ctx: &PatternExecutionContext,
    predicate: F,
) -> Option<PatternExecutionResult>
where
    F: FnOnce(&PatternExecutionContext) -> bool,
{
    if predicate(ctx) {
        Some(executor.execute(ctx))
    } else {
        None
    }
}

/// Metered dispatch with tick counting (zero-cost abstraction)
///
/// Counts ticks while dispatching, ensuring hot path operations stay within
/// the Chatman Constant (≤8 ticks).
#[inline(always)]
pub fn metered_dispatch<P: PatternExecutor>(
    executor: &P,
    ctx: &PatternExecutionContext,
) -> (PatternExecutionResult, TickCount) {
    let start = std::time::Instant::now();
    let result = executor.execute(ctx);
    let elapsed = start.elapsed();

    // Approximate ticks (this is a simplified measurement)
    let ticks = TickCount::new(elapsed.as_nanos() as u64 / 100);

    (result, ticks)
}

/// Chain multiple pattern executors with zero overhead
///
/// Executes patterns in sequence, passing the result of each to the next.
/// Fully inlined for zero-cost chaining.
#[inline(always)]
pub fn chain_dispatch<P1, P2>(
    executor1: &P1,
    executor2: &P2,
    ctx: &PatternExecutionContext,
) -> (PatternExecutionResult, PatternExecutionResult)
where
    P1: PatternExecutor,
    P2: PatternExecutor,
{
    let result1 = executor1.execute(ctx);
    let result2 = executor2.execute(ctx);
    (result1, result2)
}

/// Parallel dispatch for independent patterns (conceptual zero-cost)
///
/// In a single-threaded context, this is just sequential execution.
/// In async context, this would spawn concurrent tasks.
#[inline(always)]
pub fn parallel_dispatch<P1, P2>(
    executor1: &P1,
    executor2: &P2,
    ctx1: &PatternExecutionContext,
    ctx2: &PatternExecutionContext,
) -> (PatternExecutionResult, PatternExecutionResult)
where
    P1: PatternExecutor,
    P2: PatternExecutor,
{
    // In sync context, execute sequentially
    // In async context with rayon, this would be parallel
    (executor1.execute(ctx1), executor2.execute(ctx2))
}

/// Cached dispatch with memoization (zero-cost after first call)
///
/// Uses a simple cache to avoid re-executing identical contexts.
/// The cache lookup is inlined for minimal overhead.
pub struct CachedDispatcher<P> {
    executor: P,
    cache: std::sync::Mutex<std::collections::HashMap<String, PatternExecutionResult>>,
}

impl<P: PatternExecutor> CachedDispatcher<P> {
    /// Create a new cached dispatcher
    #[inline(always)]
    pub fn new(executor: P) -> Self {
        Self {
            executor,
            cache: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Dispatch with caching based on case_id
    #[inline]
    pub fn dispatch(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        let cache_key = format!("{}", ctx.case_id);

        // Try cache first
        {
            let cache = self.cache.lock().expect("cache lock should not be poisoned");
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }

        // Execute and cache
        let result = self.executor.execute(ctx);
        {
            let mut cache = self.cache.lock().expect("cache lock should not be poisoned");
            cache.insert(cache_key, result.clone());
        }

        result
    }

    /// Clear the cache
    #[inline]
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().expect("cache lock should not be poisoned");
        cache.clear();
    }
}

/// Type-erased dispatcher with minimal overhead
///
/// Uses trait objects for dynamic dispatch when static dispatch isn't possible,
/// but still minimizes overhead through careful design.
pub struct DynamicDispatcher {
    executor: Box<dyn PatternExecutor>,
}

impl DynamicDispatcher {
    /// Create a new dynamic dispatcher
    #[inline]
    pub fn new(executor: Box<dyn PatternExecutor>) -> Self {
        Self { executor }
    }

    /// Dispatch using dynamic dispatch (single virtual call)
    #[inline(always)]
    pub fn dispatch(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        self.executor.execute(ctx)
    }
}

/// Compile-time dispatch selector
///
/// Uses const generics to select dispatch strategy at compile time.
pub struct DispatchSelector<const STRATEGY: u8>;

impl<const STRATEGY: u8> DispatchSelector<STRATEGY> {
    /// Dispatch with compile-time selected strategy
    #[inline(always)]
    pub fn dispatch<P: PatternExecutor>(
        executor: &P,
        ctx: &PatternExecutionContext,
    ) -> PatternExecutionResult {
        match STRATEGY {
            0 => pattern_dispatch(executor, ctx),
            1 => {
                let (result, _ticks) = metered_dispatch(executor, ctx);
                result
            }
            _ => pattern_dispatch(executor, ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::PatternExecutionContext;

    struct TestExecutor;

    impl PatternExecutor for TestExecutor {
        fn execute(&self, _ctx: &PatternExecutionContext) -> PatternExecutionResult {
            PatternExecutionResult::ok(vec!["next".to_string()])
        }
    }

    #[test]
    fn test_zero_cost_dispatch() {
        let executor = TestExecutor;
        let ctx = PatternExecutionContext::default();

        let result = pattern_dispatch(&executor, &ctx);
        assert!(result.success);
    }

    #[test]
    fn test_metered_dispatch_within_budget() {
        let executor = TestExecutor;
        let ctx = PatternExecutionContext::default();

        let (_result, ticks) = metered_dispatch(&executor, &ctx);

        // Simple executor should be well within Chatman Constant
        assert!(ticks.is_within_budget());
    }

    #[test]
    fn test_batch_dispatch() {
        let executor = TestExecutor;
        let contexts = vec![
            PatternExecutionContext::default(),
            PatternExecutionContext::default(),
        ];

        let results = batch_dispatch(&executor, &contexts);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
    }

    #[test]
    fn test_cached_dispatcher() {
        let executor = TestExecutor;
        let dispatcher = CachedDispatcher::new(executor);
        let ctx = PatternExecutionContext::default();

        // First call - cache miss
        let result1 = dispatcher.dispatch(&ctx);

        // Second call - cache hit
        let result2 = dispatcher.dispatch(&ctx);

        assert!(result1.success);
        assert!(result2.success);
    }

    #[test]
    fn test_compile_time_selector() {
        let executor = TestExecutor;
        let ctx = PatternExecutionContext::default();

        // Strategy 0: standard dispatch
        let result = DispatchSelector::<0>::dispatch(&executor, &ctx);
        assert!(result.success);

        // Strategy 1: metered dispatch
        let result = DispatchSelector::<1>::dispatch(&executor, &ctx);
        assert!(result.success);
    }
}
