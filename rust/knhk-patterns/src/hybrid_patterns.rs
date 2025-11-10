// rust/knhk-patterns/src/hybrid_patterns.rs
// Hybrid hot/cold path hook pattern types for orchestrating both hook systems

use crate::patterns::{PatternError, PatternResult};
use knhk_etl::{
    hook_orchestration::{HookExecutionContext, HookExecutionResult},
    hook_registry::SharedHookRegistry,
};
use std::marker::PhantomData;
use std::sync::Arc;

/// Hybrid hook condition function type: evaluates on execution context
pub type HybridHookCondition = Arc<dyn Fn(&HookExecutionContext) -> bool + Send + Sync>;

/// Hybrid hook sequence pattern: Execute hot path hooks
pub struct HybridSequencePattern {
    hot_predicates: Vec<u64>,
    _phantom: PhantomData<()>,
    hot_registry: SharedHookRegistry,
}

impl HybridSequencePattern {
    /// Create new hybrid sequence pattern
    pub fn new(
        hot_predicates: Vec<u64>,
        _cold_hook_ids: Vec<String>,
        hot_registry: SharedHookRegistry,
    ) -> PatternResult<Self> {
        if hot_predicates.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Hot predicates list cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            hot_predicates,
            _phantom: PhantomData,
            hot_registry,
        })
    }

    /// Execute hot path hooks, then cold path hooks sequentially
    pub fn execute(
        &self,
        context: &HookExecutionContext,
    ) -> Result<HybridExecutionResult, PatternError> {
        use crate::hook_patterns::HookSequencePattern;

        let mut hot_results = None;

        // Execute hot path hooks if predicates provided
        if !self.hot_predicates.is_empty() {
            let hot_context = HookExecutionContext {
                hook_registry: Arc::clone(&self.hot_registry),
                predicate_runs: context.predicate_runs.clone(),
                soa_arrays: context.soa_arrays.clone(),
                tick_budget: context.tick_budget,
            };

            let pattern = HookSequencePattern::new(self.hot_predicates.clone())?;
            let hot_result = pattern.execute_hooks(&hot_context)?;
            hot_results = Some(hot_result);
        }

        Ok(HybridExecutionResult {
            hot_results,
            cold_results: None,
        })
    }
}

/// Hybrid hook parallel pattern: Execute hot path hooks
pub struct HybridParallelPattern {
    hot_predicates: Vec<u64>,
    _phantom: PhantomData<()>,
    hot_registry: SharedHookRegistry,
}

impl HybridParallelPattern {
    /// Create new hybrid parallel pattern
    pub fn new(
        hot_predicates: Vec<u64>,
        _cold_hook_ids: Vec<String>,
        hot_registry: SharedHookRegistry,
    ) -> PatternResult<Self> {
        if hot_predicates.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Hot predicates list cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            hot_predicates,
            _phantom: PhantomData,
            hot_registry,
        })
    }

    /// Execute hot path hooks
    pub fn execute(
        &self,
        context: &HookExecutionContext,
    ) -> Result<HybridExecutionResult, PatternError> {
        use crate::hook_patterns::HookParallelPattern;

        let mut hot_results = None;

        // Execute hot path hooks
        if !self.hot_predicates.is_empty() {
            let hot_context = HookExecutionContext {
                hook_registry: Arc::clone(&self.hot_registry),
                predicate_runs: context.predicate_runs.clone(),
                soa_arrays: context.soa_arrays.clone(),
                tick_budget: context.tick_budget,
            };

            let pattern = HookParallelPattern::new(self.hot_predicates.clone())?;
            let hot_result = pattern.execute_hooks(&hot_context)?;
            hot_results = Some(hot_result);
        }

        Ok(HybridExecutionResult {
            hot_results,
            cold_results: None,
        })
    }
}

/// Hybrid hook choice pattern: Route to hot path based on condition
pub struct HybridChoicePattern {
    condition: HybridHookCondition,
    hot_predicates: Vec<u64>,
    _phantom: PhantomData<()>,
    hot_registry: SharedHookRegistry,
}

impl HybridChoicePattern {
    /// Create new hybrid choice pattern
    pub fn new(
        condition: HybridHookCondition,
        hot_registry: SharedHookRegistry,
        _cold_hook_ids: Vec<String>,
    ) -> PatternResult<Self> {
        Ok(Self {
            condition,
            hot_predicates: Vec::new(),
            _phantom: PhantomData,
            hot_registry,
        })
    }

    /// Execute hooks based on condition
    pub fn execute(
        &self,
        context: &HookExecutionContext,
    ) -> Result<HybridExecutionResult, PatternError> {
        // Evaluate condition - always use hot path (cold path removed)
        let use_hot_path = !(self.condition)(context);

        if use_hot_path {
            // Execute hot path hooks
            if !self.hot_predicates.is_empty() {
                use crate::hook_patterns::HookSequencePattern;

                let hot_context = HookExecutionContext {
                    hook_registry: Arc::clone(&self.hot_registry),
                    predicate_runs: context.predicate_runs.clone(),
                    soa_arrays: context.soa_arrays.clone(),
                    tick_budget: context.tick_budget,
                };

                let pattern = HookSequencePattern::new(self.hot_predicates.clone())?;
                let hot_result = pattern.execute_hooks(&hot_context)?;
                return Ok(HybridExecutionResult {
                    hot_results: Some(hot_result),
                    cold_results: None,
                });
            }

            // Hot path requested but no predicates
            Err(PatternError::ExecutionFailed(
                "Hot path requested but no predicates available".to_string(),
            ))
        } else {
            // Cold path removed - return error
            Err(PatternError::ExecutionFailed(
                "Cold path no longer available - unrdf project removed".to_string(),
            ))
        }
    }
}

/// Hybrid execution result: Contains results from hot path
pub struct HybridExecutionResult {
    /// Hot path hook execution results
    pub hot_results: Option<HookExecutionResult>,
    /// Cold path hook execution results (removed - unrdf project removed)
    pub cold_results: Option<()>,
}
