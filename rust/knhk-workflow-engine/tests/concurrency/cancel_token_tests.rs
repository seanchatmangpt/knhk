//! Chicago TDD Tests for CancelToken
//!
//! Tests cancellation tokens and scopes following Chicago-style TDD.

use knhk_workflow_engine::concurrency::{CancelScope, CancelToken};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

mod cancel_token_basic {
    use super::*;

    #[tokio::test]
    async fn test_new_token_not_cancelled() {
        // Arrange & Act
        let token = CancelToken::new();

        // Assert
        assert!(!token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancel_sets_cancelled_flag() {
        // Arrange
        let token = CancelToken::new();

        // Act
        token.cancel();

        // Assert
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancelled_future_completes_immediately_when_cancelled() {
        // Arrange
        let token = CancelToken::new();
        token.cancel();

        // Act
        let result = timeout(Duration::from_millis(10), token.cancelled()).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cancelled_future_waits_until_cancelled() {
        // Arrange
        let token = CancelToken::new();
        let token_clone = token.clone();

        // Spawn task to cancel after delay
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            token_clone.cancel();
        });

        // Act
        let start = std::time::Instant::now();
        token.cancelled().await;
        let elapsed = start.elapsed();

        // Assert
        assert!(elapsed >= Duration::from_millis(40));
        assert!(token.is_cancelled());
    }
}

mod cancel_token_hierarchical {
    use super::*;

    #[tokio::test]
    async fn test_child_token_created() {
        // Arrange
        let parent = CancelToken::new();

        // Act
        let child = parent.child_token();

        // Assert
        assert!(!parent.is_cancelled());
        assert!(!child.is_cancelled());
    }

    #[tokio::test]
    async fn test_parent_cancel_cancels_children() {
        // Arrange
        let parent = CancelToken::new();
        let child1 = parent.child_token();
        let child2 = parent.child_token();

        // Act
        parent.cancel();

        // Assert
        assert!(parent.is_cancelled());
        assert!(child1.is_cancelled());
        assert!(child2.is_cancelled());
    }

    #[tokio::test]
    async fn test_child_cancel_does_not_affect_parent() {
        // Arrange
        let parent = CancelToken::new();
        let child = parent.child_token();

        // Act
        child.cancel();

        // Assert
        assert!(!parent.is_cancelled());
        assert!(child.is_cancelled());
    }

    #[tokio::test]
    async fn test_grandchild_cancelled_by_grandparent() {
        // Arrange
        let grandparent = CancelToken::new();
        let parent = grandparent.child_token();
        let child = parent.child_token();

        // Act
        grandparent.cancel();

        // Assert
        assert!(grandparent.is_cancelled());
        assert!(parent.is_cancelled());
        assert!(child.is_cancelled());
    }
}

mod cancel_scope {
    use super::*;

    #[tokio::test]
    async fn test_scope_auto_cancels_on_drop() {
        // Arrange
        let token = {
            let scope = CancelScope::new();
            scope.token().clone()
        }; // scope drops here

        // Assert
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_scope_leak_prevents_auto_cancel() {
        // Arrange
        let token = {
            let scope = CancelScope::new();
            scope.leak()
        }; // scope drops here

        // Assert
        assert!(!token.is_cancelled());
    }

    #[tokio::test]
    async fn test_scope_manual_cancel() {
        // Arrange
        let scope = CancelScope::new();

        // Act
        scope.cancel();

        // Assert
        assert!(scope.is_cancelled());
        assert!(scope.token().is_cancelled());
    }

    #[tokio::test]
    async fn test_scope_with_parent() {
        // Arrange
        let parent = CancelToken::new();
        let scope = CancelScope::with_parent(&parent);

        // Act
        parent.cancel();

        // Assert
        assert!(scope.is_cancelled());
    }
}

mod cancel_token_async_integration {
    use super::*;

    #[tokio::test]
    async fn test_select_with_cancellation() {
        // Arrange
        let token = CancelToken::new();
        let token_clone = token.clone();
        let work_done = Arc::new(AtomicBool::new(false));
        let work_done_clone = work_done.clone();

        // Spawn task that can be cancelled
        tokio::spawn(async move {
            tokio::select! {
                _ = token_clone.cancelled() => {
                    // Cancelled
                }
                _ = sleep(Duration::from_secs(10)) => {
                    work_done_clone.store(true, Ordering::SeqCst);
                }
            }
        });

        // Act: Cancel immediately
        sleep(Duration::from_millis(10)).await;
        token.cancel();

        sleep(Duration::from_millis(50)).await;

        // Assert: Work should not have completed
        assert!(!work_done.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_multiple_waiters_all_notified() {
        // Arrange
        let token = CancelToken::new();
        let completed = Arc::new(AtomicUsize::new(0));

        // Spawn multiple waiters
        for _ in 0..10 {
            let token = token.clone();
            let completed = completed.clone();
            tokio::spawn(async move {
                token.cancelled().await;
                completed.fetch_add(1, Ordering::SeqCst);
            });
        }

        sleep(Duration::from_millis(10)).await;

        // Act
        token.cancel();

        sleep(Duration::from_millis(50)).await;

        // Assert: All waiters should be notified
        assert_eq!(completed.load(Ordering::SeqCst), 10);
    }
}

mod cancel_token_stress {
    use super::*;

    #[tokio::test]
    async fn test_many_children_cancelled() {
        // Arrange
        let parent = CancelToken::new();
        let children: Vec<_> = (0..100).map(|_| parent.child_token()).collect();

        // Act
        parent.cancel();

        // Assert
        for child in &children {
            assert!(child.is_cancelled());
        }
    }

    #[tokio::test]
    async fn test_rapid_cancel_and_check() {
        // Arrange
        let token = CancelToken::new();

        // Act: Rapidly cancel and check from multiple tasks
        let handles: Vec<_> = (0..50)
            .map(|i| {
                let token = token.clone();
                tokio::spawn(async move {
                    if i == 25 {
                        token.cancel();
                    } else {
                        token.cancelled().await;
                    }
                })
            })
            .collect();

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Assert
        assert!(token.is_cancelled());
    }
}
