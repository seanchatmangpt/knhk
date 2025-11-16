//! Cancellation Tokens and Scopes
//!
//! Provides graceful cancellation for async tasks with hierarchical scopes.
//! Inspired by .NET's CancellationToken and Trio's cancel scopes.
//!
//! # Example
//! ```no_run
//! use knhk_workflow_engine::concurrency::CancelToken;
//!
//! async fn example() {
//!     let token = CancelToken::new();
//!     let child_token = token.child_token();
//!
//!     tokio::spawn({
//!         let token = child_token.clone();
//!         async move {
//!             loop {
//!                 tokio::select! {
//!                     _ = token.cancelled() => {
//!                         println!("Task cancelled");
//!                         break;
//!                     }
//!                     _ = do_work() => {
//!                         println!("Work done");
//!                     }
//!                 }
//!             }
//!         }
//!     });
//!
//!     // Later: cancel all child tasks
//!     token.cancel();
//! }
//!
//! async fn do_work() {
//!     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
//! }
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Notify;
use parking_lot::RwLock;

/// A token for cooperative task cancellation
///
/// The token can be cloned and shared across tasks. When cancelled,
/// all instances of the token are notified.
#[derive(Clone)]
pub struct CancelToken {
    inner: Arc<CancelTokenInner>,
}

struct CancelTokenInner {
    /// Whether this token has been cancelled
    cancelled: RwLock<bool>,

    /// Notifier for cancellation
    notify: Notify,

    /// Parent token (if this is a child)
    parent: Option<CancelToken>,

    /// Child tokens
    children: RwLock<Vec<CancelToken>>,
}

impl CancelToken {
    /// Create a new cancellation token
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CancelTokenInner {
                cancelled: RwLock::new(false),
                notify: Notify::new(),
                parent: None,
                children: RwLock::new(Vec::new()),
            }),
        }
    }

    /// Create a child token
    ///
    /// When the parent is cancelled, all children are automatically cancelled.
    pub fn child_token(&self) -> Self {
        let child = Self {
            inner: Arc::new(CancelTokenInner {
                cancelled: RwLock::new(false),
                notify: Notify::new(),
                parent: Some(self.clone()),
                children: RwLock::new(Vec::new()),
            }),
        };

        self.inner.children.write().push(child.clone());
        child
    }

    /// Cancel this token and all children
    pub fn cancel(&self) {
        // Mark as cancelled
        *self.inner.cancelled.write() = true;

        // Notify all waiters
        self.inner.notify.notify_waiters();

        // Cancel all children
        let children = self.inner.children.read().clone();
        for child in children {
            child.cancel();
        }
    }

    /// Check if this token has been cancelled
    pub fn is_cancelled(&self) -> bool {
        // Check self
        if *self.inner.cancelled.read() {
            return true;
        }

        // Check parent recursively
        if let Some(parent) = &self.inner.parent {
            return parent.is_cancelled();
        }

        false
    }

    /// Wait for this token to be cancelled
    ///
    /// Returns immediately if already cancelled.
    pub async fn cancelled(&self) {
        // Fast path: already cancelled
        if self.is_cancelled() {
            return;
        }

        // Wait for notification
        let notified = self.inner.notify.notified();

        // Check again after getting notified future (race condition)
        if self.is_cancelled() {
            return;
        }

        notified.await;
    }

    /// Create a future that completes when cancelled
    pub fn cancelled_owned(self) -> CancelledFuture {
        CancelledFuture { token: self }
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Future that completes when a token is cancelled
pub struct CancelledFuture {
    token: CancelToken,
}

impl Future for CancelledFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            Poll::Ready(())
        } else {
            // Re-register waker
            let notified = self.token.inner.notify.notified();
            tokio::pin!(notified);
            notified.poll(cx)
        }
    }
}

/// A cancellation scope that automatically cancels on drop
///
/// Provides RAII-style cancellation guarantees.
pub struct CancelScope {
    token: CancelToken,
    cancel_on_drop: bool,
}

impl CancelScope {
    /// Create a new cancel scope
    pub fn new() -> Self {
        Self {
            token: CancelToken::new(),
            cancel_on_drop: true,
        }
    }

    /// Create a cancel scope with a parent token
    pub fn with_parent(parent: &CancelToken) -> Self {
        Self {
            token: parent.child_token(),
            cancel_on_drop: true,
        }
    }

    /// Get the cancellation token for this scope
    pub fn token(&self) -> &CancelToken {
        &self.token
    }

    /// Cancel this scope
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Disable automatic cancellation on drop
    pub fn leak(mut self) -> CancelToken {
        self.cancel_on_drop = false;
        self.token.clone()
    }
}

impl Default for CancelScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CancelScope {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.token.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_cancel_token_basic() {
        let token = CancelToken::new();

        assert!(!token.is_cancelled());

        token.cancel();

        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancel_token_child() {
        let parent = CancelToken::new();
        let child = parent.child_token();

        assert!(!parent.is_cancelled());
        assert!(!child.is_cancelled());

        parent.cancel();

        assert!(parent.is_cancelled());
        assert!(child.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancel_token_wait() {
        let token = CancelToken::new();
        let token_clone = token.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            token_clone.cancel();
        });

        timeout(Duration::from_millis(100), token.cancelled())
            .await
            .expect("Should be cancelled");
    }

    #[tokio::test]
    async fn test_cancel_scope() {
        let token = {
            let scope = CancelScope::new();
            scope.token().clone()
        }; // scope dropped here

        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancel_scope_leak() {
        let token = {
            let scope = CancelScope::new();
            scope.leak()
        }; // scope dropped here, but token not cancelled

        assert!(!token.is_cancelled());
    }
}
