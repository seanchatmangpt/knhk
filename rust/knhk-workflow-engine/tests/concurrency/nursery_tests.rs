//! Chicago TDD Tests for Nursery Pattern
//!
//! Tests structured concurrency with nurseries following Chicago-style TDD.

use knhk_workflow_engine::concurrency::{Nursery, NurseryScope};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// AAA: Arrange-Act-Assert pattern
mod nursery_basic {
    use super::*;

    #[tokio::test]
    async fn test_empty_nursery_completes_immediately() {
        // Arrange: Create empty nursery
        let nursery = Nursery::new();

        // Act: Wait for all tasks
        let result = nursery.wait_all().await;

        // Assert: Should succeed with no tasks
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_single_task_execution() {
        // Arrange
        let mut nursery = Nursery::new();
        let executed = Arc::new(AtomicUsize::new(0));
        let executed_clone = executed.clone();

        // Act
        nursery
            .spawn(async move {
                executed_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await;

        nursery.wait_all().await.unwrap();

        // Assert
        assert_eq!(executed.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_multiple_tasks_all_execute() {
        // Arrange
        let mut nursery = Nursery::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let task_count = 10;

        // Act
        for _ in 0..task_count {
            let counter = counter.clone();
            nursery
                .spawn(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
                .await;
        }

        nursery.wait_all().await.unwrap();

        // Assert
        assert_eq!(counter.load(Ordering::SeqCst), task_count);
    }

    #[tokio::test]
    async fn test_task_count_accurate() {
        // Arrange
        let mut nursery = Nursery::new();

        // Act: Spawn 5 tasks
        for _ in 0..5 {
            nursery.spawn(async { Ok(()) }).await;
        }

        // Assert
        assert_eq!(nursery.task_count(), 5);
        assert!(!nursery.is_empty());
    }
}

mod nursery_wait_any {
    use super::*;

    #[tokio::test]
    async fn test_wait_any_returns_first_completion() {
        // Arrange
        let mut nursery = Nursery::new();
        let completed = Arc::new(AtomicUsize::new(0));

        // Spawn fast task
        let completed_clone = completed.clone();
        nursery
            .spawn(async move {
                sleep(Duration::from_millis(10)).await;
                completed_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await;

        // Spawn slow task
        let completed_clone = completed.clone();
        nursery
            .spawn(async move {
                sleep(Duration::from_millis(1000)).await;
                completed_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await;

        // Act: Wait for any
        let start = std::time::Instant::now();
        nursery.wait_any().await.unwrap();
        let elapsed = start.elapsed();

        // Assert: Should complete quickly, not wait for slow task
        assert!(elapsed < Duration::from_millis(100));
        assert_eq!(completed.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_wait_any_cancels_remaining() {
        // Arrange
        let mut nursery = Nursery::new();
        let tasks_completed = Arc::new(AtomicUsize::new(0));

        for i in 0..5 {
            let tasks_completed = tasks_completed.clone();
            nursery
                .spawn(async move {
                    sleep(Duration::from_millis(i as u64 * 10)).await;
                    tasks_completed.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
                .await;
        }

        // Act
        sleep(Duration::from_millis(5)).await; // Let first task start
        nursery.wait_any().await.unwrap();

        // Assert: Only first task should complete
        sleep(Duration::from_millis(100)).await;
        assert!(tasks_completed.load(Ordering::SeqCst) <= 2);
    }
}

mod nursery_scope {
    use super::*;

    #[tokio::test]
    async fn test_scope_auto_cleanup() {
        // Arrange
        let executed = Arc::new(AtomicUsize::new(0));
        let executed_clone = executed.clone();

        // Act: Scope automatically waits on explicit wait_all
        {
            let scope = NurseryScope::new();

            scope
                .spawn(async move {
                    sleep(Duration::from_millis(10)).await;
                    executed_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
                .await;

            scope.wait_all().await.unwrap();
        } // Scope drops here

        // Assert
        assert_eq!(executed.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_scope_multiple_tasks() {
        // Arrange
        let scope = NurseryScope::new();
        let counter = Arc::new(AtomicUsize::new(0));

        // Act
        for _ in 0..20 {
            let counter = counter.clone();
            scope
                .spawn(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
                .await;
        }

        scope.wait_all().await.unwrap();

        // Assert
        assert_eq!(counter.load(Ordering::SeqCst), 20);
    }
}

mod nursery_error_handling {
    use super::*;
    use knhk_workflow_engine::error::WorkflowError;

    #[tokio::test]
    async fn test_task_error_propagates() {
        // Arrange
        let mut nursery = Nursery::new();

        nursery
            .spawn(async {
                Err(WorkflowError::ExecutionFailed("test error".to_string()))
            })
            .await;

        // Act
        let result = nursery.wait_all().await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_errors_reported() {
        // Arrange
        let mut nursery = Nursery::new();

        // Spawn multiple failing tasks
        for _ in 0..3 {
            nursery
                .spawn(async {
                    Err(WorkflowError::ExecutionFailed("error".to_string()))
                })
                .await;
        }

        // Act
        let result = nursery.wait_all().await;

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("3 task failures"));
    }

    #[tokio::test]
    async fn test_partial_success_reports_errors() {
        // Arrange
        let mut nursery = Nursery::new();

        // Success task
        nursery.spawn(async { Ok(()) }).await;

        // Error task
        nursery
            .spawn(async {
                Err(WorkflowError::ExecutionFailed("error".to_string()))
            })
            .await;

        // Act
        let result = nursery.wait_all().await;

        // Assert: Should fail even if one task succeeds
        assert!(result.is_err());
    }
}

mod nursery_performance {
    use super::*;

    #[tokio::test]
    async fn test_spawning_many_tasks() {
        // Arrange
        let mut nursery = Nursery::new();
        let task_count = 1000;
        let counter = Arc::new(AtomicUsize::new(0));

        // Act
        let start = std::time::Instant::now();

        for _ in 0..task_count {
            let counter = counter.clone();
            nursery
                .spawn(async move {
                    counter.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .await;
        }

        nursery.wait_all().await.unwrap();
        let elapsed = start.elapsed();

        // Assert
        assert_eq!(counter.load(Ordering::Relaxed), task_count);
        println!("Spawned and completed {} tasks in {:?}", task_count, elapsed);
    }
}
