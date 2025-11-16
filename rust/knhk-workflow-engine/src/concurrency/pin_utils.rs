//! Pin/Unpin Utilities
//!
//! Provides utilities for working with Pin<Box<T>> in async contexts.
//! Simplifies common patterns when working with self-referential futures.
//!
//! # Example
//! ```no_run
//! use knhk_workflow_engine::concurrency::PinExt;
//!
//! async fn example() {
//!     let future = async { 42 };
//!     let pinned = future.pinned();
//!
//!     // Use pinned future
//!     let result = pinned.await;
//!     assert_eq!(result, 42);
//! }
//! ```

use std::future::Future;
use std::pin::Pin;

/// Extension trait for pinning futures
pub trait PinExt: Sized {
    /// Pin this value in a Box
    fn pin(self) -> Pin<Box<Self>> {
        Box::pin(self)
    }

    /// Pin this value in a Box (alias for consistency)
    fn pinned(self) -> Pin<Box<Self>> {
        Box::pin(self)
    }
}

// Implement for all Sized types
impl<T: Sized> PinExt for T {}

/// Helper to create a pinned future from a function
pub fn pin_future<F, T>(f: F) -> Pin<Box<dyn Future<Output = T> + Send>>
where
    F: Future<Output = T> + Send + 'static,
{
    Box::pin(f)
}

/// Helper to create a pinned future (non-Send)
pub fn pin_future_local<F, T>(f: F) -> Pin<Box<dyn Future<Output = T>>>
where
    F: Future<Output = T> + 'static,
{
    Box::pin(f)
}

/// Macro to simplify pinning in async blocks
#[macro_export]
macro_rules! pin {
    ($val:expr) => {
        std::pin::Pin::new(&mut $val)
    };
}

/// Macro to create a pinned boxed future
#[macro_export]
macro_rules! pin_box {
    ($val:expr) => {
        Box::pin($val)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pin_ext() {
        let future = async { 42 };
        let pinned = future.pinned();

        let result = pinned.await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_pin_future() {
        let pinned = pin_future(async { "hello" });

        let result = pinned.await;
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_pin_macro() {
        let mut value = 42;
        let _pinned = pin!(value);
        // Compile test: ensures macro works
    }
}
