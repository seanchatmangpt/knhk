// rust/knhk-patterns/src/unrdf_patterns.rs
// Unrdf (cold path) hook pattern types for orchestrating SPARQL-based hooks

#[cfg(feature = "unrdf")]
use crate::patterns::{PatternError, PatternResult};
#[cfg(feature = "unrdf")]
use knhk_unrdf::{
    hooks_native::{evaluate_hook_native, evaluate_hooks_batch_native, NativeHookRegistry},
    types::{HookDefinition, HookResult},
    UnrdfError, UnrdfResult,
};
#[cfg(feature = "unrdf")]
use std::sync::Arc;

#[cfg(feature = "unrdf")]
/// Unrdf hook condition function type: evaluates on turtle data
pub type UnrdfHookCondition = Arc<dyn Fn(&str) -> bool + Send + Sync>;

#[cfg(feature = "unrdf")]
/// Unrdf hook retry condition function type: evaluates on hook result
pub type UnrdfHookRetryCondition = Arc<dyn Fn(&HookResult) -> bool + Send + Sync>;

#[cfg(feature = "unrdf")]
/// Unrdf hook sequence pattern: Execute hooks sequentially
pub struct UnrdfSequencePattern {
    hook_ids: Vec<String>,
    registry: Arc<NativeHookRegistry>,
}

#[cfg(feature = "unrdf")]
impl UnrdfSequencePattern {
    /// Create new unrdf hook sequence pattern
    pub fn new(hook_ids: Vec<String>) -> PatternResult<Self> {
        Self::with_registry(hook_ids, Arc::new(NativeHookRegistry::new()))
    }

    /// Create new unrdf hook sequence pattern with registry
    pub fn with_registry(
        hook_ids: Vec<String>,
        registry: Arc<NativeHookRegistry>,
    ) -> PatternResult<Self> {
        if hook_ids.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Hook IDs list cannot be empty".to_string(),
            ));
        }

        // Validate epoch order is ≺-total
        NativeHookRegistry::validate_epoch_order(&hook_ids)
            .map_err(|e| PatternError::InvalidConfiguration(e.to_string()))?;

        Ok(Self { hook_ids, registry })
    }

    /// Execute hooks sequentially
    pub fn execute_hooks(&self, turtle_data: &str) -> Result<Vec<HookResult>, PatternError> {
        // Select hooks in epoch order
        let hooks = self
            .registry
            .select_by_epoch_order(&self.hook_ids)
            .map_err(|e| PatternError::ExecutionFailed(e.to_string()))?;

        // Execute hooks sequentially
        let mut results = Vec::new();
        for hook in hooks {
            let result = evaluate_hook_native(&hook, turtle_data)
                .map_err(|e| PatternError::ExecutionFailed(e.to_string()))?;
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(feature = "unrdf")]
/// Unrdf hook parallel pattern: Execute hooks in parallel
pub struct UnrdfParallelPattern {
    hook_ids: Vec<String>,
    registry: Arc<NativeHookRegistry>,
}

#[cfg(feature = "unrdf")]
impl UnrdfParallelPattern {
    /// Create new unrdf hook parallel pattern
    pub fn new(hook_ids: Vec<String>) -> PatternResult<Self> {
        Self::with_registry(hook_ids, Arc::new(NativeHookRegistry::new()))
    }

    /// Create new unrdf hook parallel pattern with registry
    pub fn with_registry(
        hook_ids: Vec<String>,
        registry: Arc<NativeHookRegistry>,
    ) -> PatternResult<Self> {
        if hook_ids.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Hook IDs list cannot be empty".to_string(),
            ));
        }

        Ok(Self { hook_ids, registry })
    }

    /// Execute hooks in parallel using batch evaluation
    pub fn execute_hooks(&self, turtle_data: &str) -> Result<Vec<HookResult>, PatternError> {
        // Get hooks from registry
        let hooks: Vec<HookDefinition> = self
            .hook_ids
            .iter()
            .filter_map(|hook_id| self.registry.get(hook_id).ok().flatten())
            .collect();

        if hooks.is_empty() {
            return Err(PatternError::ExecutionFailed(
                "No hooks found in registry".to_string(),
            ));
        }

        // Execute hooks in parallel using batch evaluation
        evaluate_hooks_batch_native(&hooks, turtle_data)
            .map_err(|e| PatternError::ExecutionFailed(e.to_string()))
    }
}

#[cfg(feature = "unrdf")]
/// Unrdf hook choice pattern: Conditional hook routing
pub struct UnrdfChoicePattern {
    choices: Vec<(UnrdfHookCondition, String)>,
    registry: Arc<NativeHookRegistry>,
}

#[cfg(feature = "unrdf")]
impl UnrdfChoicePattern {
    /// Create new unrdf hook choice pattern
    pub fn new(choices: Vec<(UnrdfHookCondition, String)>) -> PatternResult<Self> {
        Self::with_registry(choices, Arc::new(NativeHookRegistry::new()))
    }

    /// Create new unrdf hook choice pattern with registry
    pub fn with_registry(
        choices: Vec<(UnrdfHookCondition, String)>,
        registry: Arc<NativeHookRegistry>,
    ) -> PatternResult<Self> {
        if choices.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Choices list cannot be empty".to_string(),
            ));
        }

        Ok(Self { choices, registry })
    }

    /// Execute hooks conditionally
    pub fn execute_hooks(&self, turtle_data: &str) -> Result<Option<HookResult>, PatternError> {
        // Evaluate conditions in order, execute first matching hook
        for (condition, hook_id) in &self.choices {
            if condition(turtle_data) {
                // Get hook from registry
                let hook = self
                    .registry
                    .get(hook_id)
                    .map_err(|e| PatternError::ExecutionFailed(e.to_string()))?
                    .ok_or_else(|| {
                        PatternError::ExecutionFailed(format!("Hook '{}' not found", hook_id))
                    })?;

                // Execute hook
                let result = evaluate_hook_native(&hook, turtle_data)
                    .map_err(|e| PatternError::ExecutionFailed(e.to_string()))?;

                return Ok(Some(result));
            }
        }

        // No condition matched
        Ok(None)
    }
}

#[cfg(feature = "unrdf")]
/// Unrdf hook retry pattern: Retry failed hooks
pub struct UnrdfRetryPattern {
    hook_id: String,
    should_retry: UnrdfHookRetryCondition,
    max_attempts: u32,
    registry: Arc<NativeHookRegistry>,
}

#[cfg(feature = "unrdf")]
impl UnrdfRetryPattern {
    /// Create new unrdf hook retry pattern
    pub fn new(
        hook_id: String,
        should_retry: UnrdfHookRetryCondition,
        max_attempts: u32,
    ) -> PatternResult<Self> {
        Self::with_registry(
            hook_id,
            should_retry,
            max_attempts,
            Arc::new(NativeHookRegistry::new()),
        )
    }

    /// Create new unrdf hook retry pattern with registry
    pub fn with_registry(
        hook_id: String,
        should_retry: UnrdfHookRetryCondition,
        max_attempts: u32,
        registry: Arc<NativeHookRegistry>,
    ) -> PatternResult<Self> {
        if max_attempts == 0 {
            return Err(PatternError::InvalidConfiguration(
                "Max attempts must be > 0".to_string(),
            ));
        }

        Ok(Self {
            hook_id,
            should_retry,
            max_attempts,
            registry,
        })
    }

    /// Execute hook with retry logic
    pub fn execute_hooks(&self, turtle_data: &str) -> Result<HookResult, PatternError> {
        // Get hook from registry
        let hook = self
            .registry
            .get(&self.hook_id)
            .map_err(|e| PatternError::ExecutionFailed(e.to_string()))?
            .ok_or_else(|| {
                PatternError::ExecutionFailed(format!("Hook '{}' not found", self.hook_id))
            })?;

        // Retry loop
        let mut last_error: Option<UnrdfError> = None;
        for attempt in 1..=self.max_attempts {
            // Execute hook
            match evaluate_hook_native(&hook, turtle_data) {
                Ok(result) => {
                    // Check if we should retry
                    if !(self.should_retry)(&result) {
                        return Ok(result);
                    }

                    // Should retry, but check if we've exceeded max attempts
                    if attempt >= self.max_attempts {
                        return Err(PatternError::ExecutionFailed(format!(
                            "Hook '{}' failed after {} attempts",
                            self.hook_id, self.max_attempts
                        )));
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt >= self.max_attempts {
                        break;
                    }
                }
            }

            // Exponential backoff (yield to avoid blocking)
            // Note: In no_std environments, this is a no-op
            // Yield to allow other threads to run
            #[cfg(feature = "unrdf")]
            {
                // Use yield_now if available, otherwise no-op
                #[cfg(not(no_std))]
                {
                    std::thread::yield_now();
                }
            }
        }

        // All attempts failed
        Err(PatternError::ExecutionFailed(format!(
            "Hook '{}' failed after {} attempts: {}",
            self.hook_id,
            self.max_attempts,
            last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Unknown error".to_string())
        )))
    }
}

#[cfg(not(feature = "unrdf"))]
/// Placeholder types when unrdf feature is disabled
pub struct UnrdfSequencePattern;
#[cfg(not(feature = "unrdf"))]
pub struct UnrdfParallelPattern;
#[cfg(not(feature = "unrdf"))]
pub struct UnrdfChoicePattern;
#[cfg(not(feature = "unrdf"))]
pub struct UnrdfRetryPattern;
