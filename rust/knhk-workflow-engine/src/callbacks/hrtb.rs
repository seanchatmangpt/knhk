//! Higher-Ranked Trait Bounds (HRTBs) for Callback Functions
//!
//! Provides flexible callback types using HRTBs for pattern execution hooks,
//! allowing callbacks to work with any lifetime.

use crate::error::WorkflowResult;
use crate::patterns::PatternExecutionContext;
use std::collections::HashMap;
use std::sync::Arc;

/// Pattern callback function type using HRTB
///
/// The `for<'a>` syntax creates a higher-ranked trait bound, meaning this function
/// works for any lifetime 'a. This allows the callback to be called with contexts
/// of any lifetime without requiring the callback itself to be parameterized.
pub type PatternCallback = for<'a> fn(&'a PatternExecutionContext) -> WorkflowResult<()>;

/// Async pattern callback using HRTB
pub type AsyncPatternCallback =
    for<'a> fn(&'a PatternExecutionContext) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = WorkflowResult<()>> + Send + 'a>,
    >;

/// Callback registry for pattern execution hooks
///
/// Uses HRTBs to store callbacks that can work with any lifetime, providing
/// maximum flexibility while maintaining type safety.
#[derive(Default)]
pub struct CallbackRegistry {
    /// Pre-execution callbacks (run before pattern execution)
    pre_callbacks: HashMap<String, Arc<dyn CallbackFn>>,

    /// Post-execution callbacks (run after pattern execution)
    post_callbacks: HashMap<String, Arc<dyn CallbackFn>>,

    /// Error callbacks (run on pattern execution failure)
    error_callbacks: HashMap<String, Arc<dyn ErrorCallbackFn>>,
}

/// Trait for callback functions using HRTB
trait CallbackFn: Send + Sync {
    fn call(&self, ctx: &PatternExecutionContext) -> WorkflowResult<()>;
}

/// Trait for error callback functions
trait ErrorCallbackFn: Send + Sync {
    fn call(&self, ctx: &PatternExecutionContext, error: &crate::error::WorkflowError) -> WorkflowResult<()>;
}

/// Implementation wrapper for function pointers
struct FnCallback<F> {
    f: F,
}

impl<F> CallbackFn for FnCallback<F>
where
    F: for<'a> Fn(&'a PatternExecutionContext) -> WorkflowResult<()> + Send + Sync,
{
    fn call(&self, ctx: &PatternExecutionContext) -> WorkflowResult<()> {
        (self.f)(ctx)
    }
}

/// Implementation wrapper for error callbacks
struct ErrorFnCallback<F> {
    f: F,
}

impl<F> ErrorCallbackFn for ErrorFnCallback<F>
where
    F: for<'a> Fn(&'a PatternExecutionContext, &'a crate::error::WorkflowError) -> WorkflowResult<()> + Send + Sync,
{
    fn call(&self, ctx: &PatternExecutionContext, error: &crate::error::WorkflowError) -> WorkflowResult<()> {
        (self.f)(ctx, error)
    }
}

impl CallbackRegistry {
    /// Create a new callback registry
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            pre_callbacks: HashMap::new(),
            post_callbacks: HashMap::new(),
            error_callbacks: HashMap::new(),
        }
    }

    /// Register a pre-execution callback
    ///
    /// The HRTB on `F` allows any function that works for all lifetimes.
    #[inline]
    pub fn register_pre<F>(&mut self, name: impl Into<String>, callback: F)
    where
        F: for<'a> Fn(&'a PatternExecutionContext) -> WorkflowResult<()> + Send + Sync + 'static,
    {
        self.pre_callbacks.insert(
            name.into(),
            Arc::new(FnCallback { f: callback }),
        );
    }

    /// Register a post-execution callback
    #[inline]
    pub fn register_post<F>(&mut self, name: impl Into<String>, callback: F)
    where
        F: for<'a> Fn(&'a PatternExecutionContext) -> WorkflowResult<()> + Send + Sync + 'static,
    {
        self.post_callbacks.insert(
            name.into(),
            Arc::new(FnCallback { f: callback }),
        );
    }

    /// Register an error callback
    #[inline]
    pub fn register_error<F>(&mut self, name: impl Into<String>, callback: F)
    where
        F: for<'a> Fn(&'a PatternExecutionContext, &'a crate::error::WorkflowError) -> WorkflowResult<()> + Send + Sync + 'static,
    {
        self.error_callbacks.insert(
            name.into(),
            Arc::new(ErrorFnCallback { f: callback }),
        );
    }

    /// Execute all pre-callbacks
    #[inline]
    pub fn execute_pre(&self, ctx: &PatternExecutionContext) -> WorkflowResult<()> {
        for callback in self.pre_callbacks.values() {
            callback.call(ctx)?;
        }
        Ok(())
    }

    /// Execute all post-callbacks
    #[inline]
    pub fn execute_post(&self, ctx: &PatternExecutionContext) -> WorkflowResult<()> {
        for callback in self.post_callbacks.values() {
            callback.call(ctx)?;
        }
        Ok(())
    }

    /// Execute all error callbacks
    #[inline]
    pub fn execute_error(&self, ctx: &PatternExecutionContext, error: &crate::error::WorkflowError) -> WorkflowResult<()> {
        for callback in self.error_callbacks.values() {
            callback.call(ctx, error)?;
        }
        Ok(())
    }

    /// Remove a pre-callback
    #[inline]
    pub fn remove_pre(&mut self, name: &str) -> bool {
        self.pre_callbacks.remove(name).is_some()
    }

    /// Remove a post-callback
    #[inline]
    pub fn remove_post(&mut self, name: &str) -> bool {
        self.post_callbacks.remove(name).is_some()
    }

    /// Remove an error callback
    #[inline]
    pub fn remove_error(&mut self, name: &str) -> bool {
        self.error_callbacks.remove(name).is_some()
    }

    /// Clear all callbacks
    #[inline]
    pub fn clear(&mut self) {
        self.pre_callbacks.clear();
        self.post_callbacks.clear();
        self.error_callbacks.clear();
    }

    /// Get count of registered callbacks
    #[inline]
    pub fn count(&self) -> (usize, usize, usize) {
        (
            self.pre_callbacks.len(),
            self.post_callbacks.len(),
            self.error_callbacks.len(),
        )
    }
}

/// Pattern execution wrapper with callback support
///
/// Wraps pattern execution with pre/post/error callbacks using HRTB callbacks.
pub struct CallbackExecutor {
    registry: Arc<CallbackRegistry>,
}

impl CallbackExecutor {
    /// Create a new callback executor
    #[inline(always)]
    pub fn new(registry: Arc<CallbackRegistry>) -> Self {
        Self { registry }
    }

    /// Execute a pattern with callbacks
    ///
    /// Runs pre-callbacks, executes the pattern, runs post-callbacks,
    /// and handles errors with error callbacks.
    #[inline]
    pub fn execute_with_callbacks<F>(
        &self,
        ctx: &PatternExecutionContext,
        executor: F,
    ) -> WorkflowResult<crate::patterns::PatternExecutionResult>
    where
        F: FnOnce(&PatternExecutionContext) -> WorkflowResult<crate::patterns::PatternExecutionResult>,
    {
        // Execute pre-callbacks
        if let Err(e) = self.registry.execute_pre(ctx) {
            let _ = self.registry.execute_error(ctx, &e);
            return Err(e);
        }

        // Execute the pattern
        let result = executor(ctx);

        // Handle result
        match result {
            Ok(res) => {
                // Execute post-callbacks on success
                if let Err(e) = self.registry.execute_post(ctx) {
                    let _ = self.registry.execute_error(ctx, &e);
                    return Err(e);
                }
                Ok(res)
            }
            Err(e) => {
                // Execute error callbacks on failure
                let _ = self.registry.execute_error(ctx, &e);
                Err(e)
            }
        }
    }
}

/// Builder for callback registry using HRTB
pub struct CallbackRegistryBuilder {
    registry: CallbackRegistry,
}

impl CallbackRegistryBuilder {
    /// Create a new builder
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            registry: CallbackRegistry::new(),
        }
    }

    /// Add a pre-callback
    #[inline]
    pub fn with_pre<F>(mut self, name: impl Into<String>, callback: F) -> Self
    where
        F: for<'a> Fn(&'a PatternExecutionContext) -> WorkflowResult<()> + Send + Sync + 'static,
    {
        self.registry.register_pre(name, callback);
        self
    }

    /// Add a post-callback
    #[inline]
    pub fn with_post<F>(mut self, name: impl Into<String>, callback: F) -> Self
    where
        F: for<'a> Fn(&'a PatternExecutionContext) -> WorkflowResult<()> + Send + Sync + 'static,
    {
        self.registry.register_post(name, callback);
        self
    }

    /// Add an error callback
    #[inline]
    pub fn with_error<F>(mut self, name: impl Into<String>, callback: F) -> Self
    where
        F: for<'a> Fn(&'a PatternExecutionContext, &'a crate::error::WorkflowError) -> WorkflowResult<()> + Send + Sync + 'static,
    {
        self.registry.register_error(name, callback);
        self
    }

    /// Build the registry
    #[inline(always)]
    pub fn build(self) -> CallbackRegistry {
        self.registry
    }
}

impl Default for CallbackRegistryBuilder {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_hrtb_callback_execution() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let mut registry = CallbackRegistry::new();
        registry.register_pre("counter", move |_ctx| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        let ctx = PatternExecutionContext::default();
        registry.execute_pre(&ctx).expect("pre-callback should succeed");

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_callback_registry_builder() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pre_counter = counter.clone();
        let post_counter = counter.clone();

        let registry = CallbackRegistryBuilder::new()
            .with_pre("pre", move |_ctx| {
                pre_counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .with_post("post", move |_ctx| {
                post_counter.fetch_add(10, Ordering::SeqCst);
                Ok(())
            })
            .build();

        let (pre_count, post_count, _) = registry.count();
        assert_eq!(pre_count, 1);
        assert_eq!(post_count, 1);
    }
}
