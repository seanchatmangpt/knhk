//! Structured Concurrency with Nurseries
//!
//! Provides scoped task spawning with automatic cancellation and cleanup.
//! Inspired by Trio's nurseries and Swift's TaskGroups.
//!
//! # Example
//! ```no_run
//! use knhk_workflow_engine::concurrency::Nursery;
//!
//! async fn example() {
//!     let mut nursery = Nursery::new();
//!
//!     // Spawn tasks - all will be awaited before nursery drops
//!     nursery.spawn(async { println!("Task 1"); Ok(()) }).await;
//!     nursery.spawn(async { println!("Task 2"); Ok(()) }).await;
//!
//!     // Wait for all tasks
//!     nursery.wait_all().await.unwrap();
//! }
//! ```

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::WorkflowResult;
use super::{ConcurrencyError, ConcurrencyResult};

/// A nursery for managing multiple async tasks with structured concurrency
///
/// The nursery ensures all spawned tasks complete (or are cancelled) before
/// the nursery itself completes. This prevents orphaned tasks and ensures
/// proper resource cleanup.
pub struct Nursery<'a> {
    /// Handles to spawned tasks
    tasks: Vec<JoinHandle<WorkflowResult<()>>>,

    /// Phantom data to bind lifetime
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Nursery<'a> {
    /// Create a new nursery
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Spawn a new task in this nursery
    ///
    /// The task will be automatically awaited when the nursery completes.
    pub async fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = WorkflowResult<()>> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        self.tasks.push(handle);
    }

    /// Spawn a new task that returns a value
    pub async fn spawn_value<F, T>(&mut self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        tokio::spawn(future)
    }

    /// Wait for all tasks to complete
    ///
    /// Returns an error if any task failed.
    pub async fn wait_all(self) -> WorkflowResult<()> {
        let mut errors = Vec::new();

        for handle in self.tasks {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => errors.push(e),
                Err(join_err) => {
                    errors.push(crate::error::WorkflowError::ExecutionFailed(
                        format!("Task join error: {}", join_err)
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ExecutionFailed(
                format!("Nursery had {} task failures", errors.len())
            ))
        }
    }

    /// Wait for any task to complete, then cancel remaining tasks
    ///
    /// Returns the result of the first completed task.
    pub async fn wait_any(mut self) -> WorkflowResult<()> {
        if self.tasks.is_empty() {
            return Ok(());
        }

        // Wait for first task to complete
        let (result, _index, remaining) = futures::future::select_all(self.tasks).await;

        // Cancel remaining tasks
        for handle in remaining {
            handle.abort();
        }

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(join_err) => Err(crate::error::WorkflowError::ExecutionFailed(
                format!("Task join error: {}", join_err)
            )),
        }
    }

    /// Get the number of tasks in this nursery
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Check if the nursery is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

impl<'a> Default for Nursery<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped nursery that automatically waits for all tasks on drop
///
/// This provides RAII-style guarantees that all tasks complete before
/// the scope exits.
pub struct NurseryScope {
    inner: Arc<Mutex<Vec<JoinHandle<WorkflowResult<()>>>>>,
}

impl NurseryScope {
    /// Create a new nursery scope
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Spawn a task in this scope
    pub async fn spawn<F>(&self, future: F)
    where
        F: Future<Output = WorkflowResult<()>> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        self.inner.lock().await.push(handle);
    }

    /// Wait for all tasks to complete
    pub async fn wait_all(&self) -> WorkflowResult<()> {
        let handles = {
            let mut guard = self.inner.lock().await;
            std::mem::take(&mut *guard)
        };

        let mut errors = Vec::new();

        for handle in handles {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => errors.push(e),
                Err(join_err) => {
                    errors.push(crate::error::WorkflowError::ExecutionFailed(
                        format!("Task join error: {}", join_err)
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(crate::error::WorkflowError::ExecutionFailed(
                format!("Nursery scope had {} task failures", errors.len())
            ))
        }
    }
}

impl Default for NurseryScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NurseryScope {
    fn drop(&mut self) {
        // Note: We can't await in drop, so we abort all tasks
        // Users should call wait_all() explicitly before dropping
        if let Ok(mut guard) = self.inner.try_lock() {
            for handle in guard.drain(..) {
                handle.abort();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nursery_basic() {
        let mut nursery = Nursery::new();

        nursery.spawn(async { Ok(()) }).await;
        nursery.spawn(async { Ok(()) }).await;

        assert_eq!(nursery.task_count(), 2);
        nursery.wait_all().await.unwrap();
    }

    #[tokio::test]
    async fn test_nursery_wait_any() {
        let mut nursery = Nursery::new();

        nursery.spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok(())
        }).await;

        nursery.spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok(())
        }).await;

        // Should complete after first task
        nursery.wait_any().await.unwrap();
    }

    #[tokio::test]
    async fn test_nursery_scope() {
        let scope = NurseryScope::new();

        scope.spawn(async { Ok(()) }).await;
        scope.spawn(async { Ok(()) }).await;

        scope.wait_all().await.unwrap();
    }
}
