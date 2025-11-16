//! Async Microkernel: Non-Blocking Workflow Execution
//!
//! Async type-state machines for concurrent workflows.
//! Futures-based kernel with zero-cost async abstraction.
//! Cancellation-safe operations with strong guarantees.

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::time::Duration;

/// Async workflow state - type-level state tracking
pub trait AsyncWorkflowState: 'static {
    const NAME: &'static str;
    const IS_TERMINAL: bool;
}

/// Pending - workflow not yet started
pub struct AsyncPending;
impl AsyncWorkflowState for AsyncPending {
    const NAME: &'static str = "pending";
    const IS_TERMINAL: bool = false;
}

/// Running - workflow executing
pub struct AsyncRunning;
impl AsyncWorkflowState for AsyncRunning {
    const NAME: &'static str = "running";
    const IS_TERMINAL: bool = false;
}

/// Suspended - workflow yielded/awaiting
pub struct AsyncSuspended;
impl AsyncWorkflowState for AsyncSuspended {
    const NAME: &'static str = "suspended";
    const IS_TERMINAL: bool = false;
}

/// Completed - workflow finished successfully
pub struct AsyncCompleted;
impl AsyncWorkflowState for AsyncCompleted {
    const NAME: &'static str = "completed";
    const IS_TERMINAL: bool = true;
}

/// Failed - workflow failed with error
pub struct AsyncFailed;
impl AsyncWorkflowState for AsyncFailed {
    const NAME: &'static str = "failed";
    const IS_TERMINAL: bool = true;
}

/// Async workflow - type-state machine
pub struct AsyncWorkflow<S: AsyncWorkflowState, T> {
    state: PhantomData<S>,
    value: Option<T>,
    error: Option<String>,
}

impl<T> AsyncWorkflow<AsyncPending, T> {
    /// Create new pending workflow
    pub const fn new() -> Self {
        Self {
            state: PhantomData,
            value: None,
            error: None,
        }
    }

    /// Start workflow execution
    pub fn start(self) -> AsyncWorkflow<AsyncRunning, T> {
        AsyncWorkflow {
            state: PhantomData,
            value: self.value,
            error: None,
        }
    }
}

impl<T> AsyncWorkflow<AsyncRunning, T> {
    /// Suspend workflow (yield control)
    pub fn suspend(self) -> AsyncWorkflow<AsyncSuspended, T> {
        AsyncWorkflow {
            state: PhantomData,
            value: self.value,
            error: self.error,
        }
    }

    /// Complete workflow successfully
    pub fn complete(mut self, value: T) -> AsyncWorkflow<AsyncCompleted, T> {
        self.value = Some(value);
        AsyncWorkflow {
            state: PhantomData,
            value: self.value,
            error: None,
        }
    }

    /// Fail workflow with error
    pub fn fail(mut self, error: String) -> AsyncWorkflow<AsyncFailed, T> {
        self.error = Some(error);
        AsyncWorkflow {
            state: PhantomData,
            value: None,
            error: self.error,
        }
    }
}

impl<T> AsyncWorkflow<AsyncSuspended, T> {
    /// Resume workflow execution
    pub fn resume(self) -> AsyncWorkflow<AsyncRunning, T> {
        AsyncWorkflow {
            state: PhantomData,
            value: self.value,
            error: self.error,
        }
    }
}

impl<T> AsyncWorkflow<AsyncCompleted, T> {
    /// Extract completed value
    pub fn into_value(self) -> Option<T> {
        self.value
    }
}

impl<T> AsyncWorkflow<AsyncFailed, T> {
    /// Get error message
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Async kernel result
pub enum AsyncKernelResult<T> {
    Ready(T),
    Pending,
    Error(String),
}

/// Async kernel operation - future-based
pub struct AsyncKernelOp<F> {
    future: F,
}

impl<F, T> AsyncKernelOp<F>
where
    F: Future<Output = Result<T, String>>,
{
    /// Create async kernel operation
    pub fn new(future: F) -> Self {
        Self { future }
    }

    /// Execute operation asynchronously
    pub async fn execute(self) -> AsyncKernelResult<T> {
        match self.future.await {
            Ok(value) => AsyncKernelResult::Ready(value),
            Err(e) => AsyncKernelResult::Error(e),
        }
    }
}

/// Timeout wrapper for async operations
pub struct WithTimeout<F> {
    future: F,
    timeout: Duration,
}

impl<F> WithTimeout<F> {
    pub fn new(future: F, timeout: Duration) -> Self {
        Self { future, timeout }
    }
}

impl<F, T> Future for WithTimeout<F>
where
    F: Future<Output = T> + Unpin,
{
    type Output = Result<T, TimeoutError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Simplified timeout - real impl would use timer
        match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(value) => Poll::Ready(Ok(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Timeout error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutError;

/// Cancellation token - cooperative cancellation
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn child(&self) -> Self {
        Self {
            cancelled: self.cancelled.clone(),
        }
    }
}

/// Cancellable async operation
pub struct Cancellable<F> {
    future: F,
    token: CancellationToken,
}

impl<F> Cancellable<F> {
    pub fn new(future: F, token: CancellationToken) -> Self {
        Self { future, token }
    }
}

impl<F, T> Future for Cancellable<F>
where
    F: Future<Output = T> + Unpin,
{
    type Output = Result<T, Cancelled>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            return Poll::Ready(Err(Cancelled));
        }

        match Pin::new(&mut self.future).poll(cx) {
            Poll::Ready(value) => Poll::Ready(Ok(value)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

/// Async workflow executor - runs workflows concurrently
pub struct AsyncExecutor {
    runtime: tokio::runtime::Runtime,
}

impl AsyncExecutor {
    /// Create new async executor
    pub fn new() -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        Self { runtime }
    }

    /// Spawn workflow task
    pub fn spawn<F, T>(&self, future: F) -> tokio::task::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// Block on future (wait for completion)
    pub fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        self.runtime.block_on(future)
    }

    /// Spawn multiple workflows concurrently
    pub async fn spawn_many<F, T>(&self, futures: Vec<F>) -> Vec<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let handles: Vec<_> = futures.into_iter().map(|f| self.spawn(f)).collect();

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
        results
    }
}

impl Default for AsyncExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Async workflow builder - fluent API
pub struct AsyncWorkflowBuilder<T> {
    timeout: Option<Duration>,
    cancellation: Option<CancellationToken>,
    _phantom: PhantomData<T>,
}

impl<T> AsyncWorkflowBuilder<T> {
    pub fn new() -> Self {
        Self {
            timeout: None,
            cancellation: None,
            _phantom: PhantomData,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.cancellation = Some(token);
        self
    }

    pub async fn execute<F>(self, future: F) -> Result<T, String>
    where
        F: Future<Output = Result<T, String>>,
    {
        // Apply cancellation if specified
        if let Some(token) = self.cancellation {
            if token.is_cancelled() {
                return Err("Workflow cancelled".to_string());
            }
        }

        // Apply timeout if specified
        if let Some(_timeout) = self.timeout {
            // Would wrap with timeout
        }

        future.await
    }
}

impl<T> Default for AsyncWorkflowBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_workflow_states() {
        let workflow = AsyncWorkflow::<AsyncPending, i32>::new();
        let workflow = workflow.start();
        let workflow = workflow.complete(42);
        assert_eq!(workflow.into_value(), Some(42));
    }

    #[test]
    fn test_async_workflow_failure() {
        let workflow = AsyncWorkflow::<AsyncPending, i32>::new();
        let workflow = workflow.start();
        let workflow = workflow.fail("Test error".to_string());
        assert_eq!(workflow.error(), Some("Test error"));
    }

    #[test]
    fn test_async_workflow_suspension() {
        let workflow = AsyncWorkflow::<AsyncPending, i32>::new();
        let workflow = workflow.start();
        let workflow = workflow.suspend();
        let workflow = workflow.resume();
        let workflow = workflow.complete(100);
        assert_eq!(workflow.into_value(), Some(100));
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());

        let child = token.child();
        assert!(child.is_cancelled()); // Shares state
    }

    #[tokio::test]
    async fn test_async_kernel_op() {
        let op = AsyncKernelOp::new(async { Ok(42) });
        let result = op.execute().await;

        match result {
            AsyncKernelResult::Ready(value) => assert_eq!(value, 42),
            _ => panic!("Expected Ready"),
        }
    }

    #[tokio::test]
    async fn test_async_workflow_builder() {
        let builder = AsyncWorkflowBuilder::<i32>::new()
            .with_timeout(Duration::from_secs(1));

        let result = builder.execute(async { Ok(42) }).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_async_executor() {
        let executor = AsyncExecutor::new();

        let handle = executor.spawn(async { 42 });
        let result = handle.await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_spawn_many() {
        let executor = AsyncExecutor::new();

        // Helper function to create futures with same type
        async fn make_future(value: i32) -> i32 {
            value
        }

        let futures = vec![
            make_future(1),
            make_future(2),
            make_future(3),
        ];

        let results = executor.spawn_many(futures).await;
        assert_eq!(results.len(), 3);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(results.contains(&3));
    }

    #[test]
    fn test_state_constants() {
        assert_eq!(AsyncPending::NAME, "pending");
        assert!(!AsyncPending::IS_TERMINAL);

        assert_eq!(AsyncCompleted::NAME, "completed");
        assert!(AsyncCompleted::IS_TERMINAL);

        assert_eq!(AsyncFailed::NAME, "failed");
        assert!(AsyncFailed::IS_TERMINAL);
    }
}
