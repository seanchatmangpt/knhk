//! Generic Associated Types (GATs) for Pattern Executors
//!
//! Provides zero-cost abstraction using GATs for pattern execution with compile-time
//! guarantees about input/output types and error handling.

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::{PatternExecutionContext, PatternExecutionResult};
use std::future::Future;
use std::marker::PhantomData;

/// Pattern executor using Generic Associated Types for zero-cost abstraction
///
/// GATs allow us to define associated types that are generic over a lifetime,
/// enabling zero-cost abstraction for pattern execution with proper lifetime tracking.
#[cfg(feature = "type-system-v2")]
pub trait PatternExecutor {
    /// Input type for the pattern execution (lifetime-generic)
    type Input<'a>
    where
        Self: 'a;

    /// Output type for the pattern execution (lifetime-generic)
    type Output<'a>
    where
        Self: 'a;

    /// Error type for the pattern execution (lifetime-generic)
    type Error<'a>: Into<WorkflowError>
    where
        Self: 'a;

    /// Execute the pattern with the given input
    ///
    /// This method uses GATs to ensure proper lifetime tracking of inputs and outputs
    /// while maintaining zero-cost abstraction through inlining.
    fn execute<'a>(
        &'a self,
        input: Self::Input<'a>,
    ) -> impl Future<Output = Result<Self::Output<'a>, Self::Error<'a>>> + 'a;
}

/// Async pattern executor using GATs for async workflows
///
/// Extends the basic pattern executor with async capabilities while maintaining
/// zero-cost abstraction through compiler optimizations.
#[cfg(feature = "type-system-v2")]
pub trait AsyncPatternExecutor {
    /// Input type for async execution
    type Input<'a>
    where
        Self: 'a;

    /// Output type for async execution
    type Output<'a>
    where
        Self: 'a;

    /// Error type for async execution
    type Error<'a>: Into<WorkflowError>
    where
        Self: 'a;

    /// Execute pattern asynchronously
    fn execute_async<'a>(
        &'a self,
        input: Self::Input<'a>,
    ) -> impl Future<Output = Result<Self::Output<'a>, Self::Error<'a>>> + Send + 'a;
}

/// Stateful pattern executor with compile-time state tracking
///
/// Uses GATs and phantom types to track execution state at compile time,
/// preventing invalid state transitions.
#[cfg(feature = "type-system-v2")]
pub trait StatefulPatternExecutor<State> {
    /// Input type parameterized by state
    type Input<'a, S>
    where
        Self: 'a,
        S: 'a;

    /// Output type parameterized by state
    type Output<'a, S>
    where
        Self: 'a,
        S: 'a;

    /// Next state after execution
    type NextState;

    /// Execute with state transition
    fn execute_with_state<'a, S>(
        &'a self,
        input: Self::Input<'a, S>,
        state: S,
    ) -> impl Future<Output = WorkflowResult<(Self::Output<'a, S>, Self::NextState)>> + 'a
    where
        S: 'a;
}

/// Basic pattern executor implementation using GATs
#[cfg(feature = "type-system-v2")]
pub struct BasicPatternExecutor<F> {
    executor_fn: F,
    _marker: PhantomData<fn()>,
}

#[cfg(feature = "type-system-v2")]
impl<F> BasicPatternExecutor<F> {
    /// Create a new basic pattern executor
    #[inline(always)]
    pub const fn new(executor_fn: F) -> Self {
        Self {
            executor_fn,
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "type-system-v2")]
impl<F> PatternExecutor for BasicPatternExecutor<F>
where
    F: for<'a> Fn(&'a PatternExecutionContext) -> WorkflowResult<PatternExecutionResult> + Send + Sync,
{
    type Input<'a> = &'a PatternExecutionContext where Self: 'a;
    type Output<'a> = PatternExecutionResult where Self: 'a;
    type Error<'a> = WorkflowError where Self: 'a;

    #[inline(always)]
    async fn execute<'a>(
        &'a self,
        input: Self::Input<'a>,
    ) -> Result<Self::Output<'a>, Self::Error<'a>> {
        (self.executor_fn)(input)
    }
}

/// Batch pattern executor for processing multiple patterns in parallel
#[cfg(feature = "type-system-v2")]
pub trait BatchPatternExecutor {
    /// Batch input type
    type BatchInput<'a>
    where
        Self: 'a;

    /// Batch output type
    type BatchOutput<'a>
    where
        Self: 'a;

    /// Execute batch of patterns
    fn execute_batch<'a>(
        &'a self,
        inputs: Self::BatchInput<'a>,
    ) -> impl Future<Output = WorkflowResult<Self::BatchOutput<'a>>> + 'a;
}

/// Pattern executor adapter for legacy compatibility
#[cfg(feature = "type-system-v2")]
pub struct LegacyPatternAdapter<T> {
    inner: T,
}

#[cfg(feature = "type-system-v2")]
impl<T> LegacyPatternAdapter<T> {
    /// Create a new legacy pattern adapter
    #[inline(always)]
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Unwrap the inner executor
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

#[cfg(feature = "type-system-v2")]
impl<T> PatternExecutor for LegacyPatternAdapter<T>
where
    T: crate::patterns::PatternExecutor + Send + Sync,
{
    type Input<'a> = &'a PatternExecutionContext where Self: 'a;
    type Output<'a> = PatternExecutionResult where Self: 'a;
    type Error<'a> = WorkflowError where Self: 'a;

    #[inline(always)]
    async fn execute<'a>(
        &'a self,
        input: Self::Input<'a>,
    ) -> Result<Self::Output<'a>, Self::Error<'a>> {
        Ok(self.inner.execute(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gat_size_of_executor() {
        // Ensure zero-cost: executor should be just the function pointer size
        let _executor = BasicPatternExecutor::new(|_ctx| {
            Ok(PatternExecutionResult::ok(vec![]))
        });

        // Size should be minimal (function pointer + phantom data = pointer size)
        assert!(std::mem::size_of_val(&_executor) <= std::mem::size_of::<usize>() * 2);
    }
}
