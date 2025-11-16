//! # Hooks System - Integration Points
//!
//! **Covenant 3**: Feedback loops run at machine speed
//!
//! The Hooks system provides integration points for customizing autonomic behavior.
//! Hooks are called at specific points in the MAPE-K loop to enable extension
//! without modifying core code.
//!
//! ## Hook Types
//!
//! - **PreMonitor**: Before metrics collection
//! - **PostMonitor**: After metrics collected and anomalies detected
//! - **PreAnalyze**: Before pattern matching
//! - **PostAnalyze**: After root cause analysis
//! - **PrePlan**: Before policy evaluation
//! - **PostPlan**: After plan generated
//! - **PreExecute**: Before action execution
//! - **PostExecute**: After action execution and feedback capture
//! - **PreFeedback**: Before knowledge update
//! - **PostFeedback**: After learning complete
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_autonomic::hooks::{HookRegistry, HookType, HookContext};
//! use knhk_autonomic::Result;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut registry = HookRegistry::new();
//!
//! // Register hook
//! registry.register(HookType::PostAnalyze, |ctx| {
//!     async move {
//!         println!("Analysis complete: {:?}", ctx);
//!         Ok(())
//!     }
//! });
//!
//! // Execute hooks
//! let ctx = HookContext::default();
//! registry.execute(HookType::PostAnalyze, &ctx).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{instrument, debug};

/// Hook execution point in MAPE-K loop
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    /// Before monitoring phase
    PreMonitor,
    /// After monitoring phase
    PostMonitor,
    /// Before analysis phase
    PreAnalyze,
    /// After analysis phase
    PostAnalyze,
    /// Before planning phase
    PrePlan,
    /// After planning phase
    PostPlan,
    /// Before execution phase
    PreExecute,
    /// After execution phase
    PostExecute,
    /// Before feedback to knowledge
    PreFeedback,
    /// After feedback to knowledge
    PostFeedback,
}

/// Context passed to hook functions
#[derive(Debug, Clone, Default)]
pub struct HookContext {
    /// Arbitrary data for hooks
    pub data: HashMap<String, serde_json::Value>,
}

impl HookContext {
    /// Create new hook context
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Set value in context
    pub fn set(&mut self, key: impl Into<String>, value: impl serde::Serialize) -> Result<()> {
        self.data.insert(
            key.into(),
            serde_json::to_value(value)?,
        );
        Ok(())
    }

    /// Get value from context
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        if let Some(value) = self.data.get(key) {
            Ok(Some(serde_json::from_value(value.clone())?))
        } else {
            Ok(None)
        }
    }
}

/// Hook function type
pub type HookFn = Arc<dyn Fn(HookContext) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

/// Registry for autonomic hooks
#[derive(Clone)]
pub struct HookRegistry {
    hooks: Arc<RwLock<HashMap<HookType, Vec<HookFn>>>>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HookRegistry {
    /// Create a new hook registry
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a hook
    #[instrument(skip(self, hook))]
    pub async fn register<F, Fut>(&mut self, hook_type: HookType, hook: F)
    where
        F: Fn(HookContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let hook_fn: HookFn = Arc::new(move |ctx| Box::pin(hook(ctx)));

        let mut hooks = self.hooks.write().await;
        hooks.entry(hook_type).or_insert_with(Vec::new).push(hook_fn);

        debug!("Registered hook for {:?}", hook_type);
    }

    /// Execute all hooks of a given type
    #[instrument(skip(self, context))]
    pub async fn execute(&self, hook_type: HookType, context: &HookContext) -> Result<()> {
        let hooks = self.hooks.read().await;

        if let Some(hook_fns) = hooks.get(&hook_type) {
            debug!("Executing {} hooks for {:?}", hook_fns.len(), hook_type);

            for hook_fn in hook_fns {
                // Execute hook
                hook_fn(context.clone()).await?;
            }
        }

        Ok(())
    }

    /// Check if hooks are registered for a type
    pub async fn has_hooks(&self, hook_type: HookType) -> bool {
        let hooks = self.hooks.read().await;
        hooks.get(&hook_type).map(|v| !v.is_empty()).unwrap_or(false)
    }

    /// Get count of hooks for a type
    pub async fn hook_count(&self, hook_type: HookType) -> usize {
        let hooks = self.hooks.read().await;
        hooks.get(&hook_type).map(|v| v.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_execute_hook() {
        let mut registry = HookRegistry::new();

        // Register hook
        registry.register(HookType::PostMonitor, |_ctx| async {
            Ok(())
        }).await;

        // Check registration
        assert!(registry.has_hooks(HookType::PostMonitor).await);
        assert_eq!(registry.hook_count(HookType::PostMonitor).await, 1);

        // Execute hooks
        let ctx = HookContext::new();
        registry.execute(HookType::PostMonitor, &ctx).await.unwrap();
    }

    #[tokio::test]
    async fn test_hook_context() {
        let mut ctx = HookContext::new();

        // Set value
        ctx.set("test_key", "test_value").unwrap();

        // Get value
        let value: Option<String> = ctx.get("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }
}
