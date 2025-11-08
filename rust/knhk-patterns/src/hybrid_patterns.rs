// rust/knhk-patterns/src/hybrid_patterns.rs
// Hybrid hot/cold path hook pattern types for orchestrating both hook systems

use crate::patterns::{PatternError, PatternResult};
use knhk_etl::{
    hook_orchestration::{HookExecutionContext, HookExecutionResult},
    hook_registry::HookRegistry,
    load::SoAArrays,
};
use std::sync::Arc;

#[cfg(feature = "unrdf")]
use crate::unrdf_patterns::{UnrdfParallelPattern, UnrdfSequencePattern};
#[cfg(feature = "unrdf")]
use knhk_unrdf::{hooks_native::NativeHookRegistry, types::HookResult};

#[cfg(not(feature = "unrdf"))]
use std::marker::PhantomData;

/// Hybrid hook condition function type: evaluates on execution context
pub type HybridHookCondition = Arc<dyn Fn(&HookExecutionContext) -> bool + Send + Sync>;

/// Hybrid hook sequence pattern: Execute hot path hooks, then cold path hooks
pub struct HybridSequencePattern {
    hot_predicates: Vec<u64>,
    #[cfg(feature = "unrdf")]
    cold_hook_ids: Vec<String>,
    #[cfg(not(feature = "unrdf"))]
    _phantom: PhantomData<()>,
    hot_registry: HookRegistry,
    #[cfg(feature = "unrdf")]
    cold_registry: Arc<NativeHookRegistry>,
}

impl HybridSequencePattern {
    /// Create new hybrid sequence pattern
    #[cfg(feature = "unrdf")]
    pub fn new(
        hot_predicates: Vec<u64>,
        cold_hook_ids: Vec<String>,
        hot_registry: HookRegistry,
    ) -> PatternResult<Self> {
        Self::with_registries(
            hot_predicates,
            cold_hook_ids,
            hot_registry,
            Arc::new(NativeHookRegistry::new()),
        )
    }

    /// Create new hybrid sequence pattern with registries
    #[cfg(feature = "unrdf")]
    pub fn with_registries(
        hot_predicates: Vec<u64>,
        cold_hook_ids: Vec<String>,
        hot_registry: HookRegistry,
        cold_registry: Arc<NativeHookRegistry>,
    ) -> PatternResult<Self> {
        if hot_predicates.is_empty() && cold_hook_ids.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "At least one hot predicate or cold hook ID required".to_string(),
            ));
        }

        Ok(Self {
            hot_predicates,
            cold_hook_ids,
            hot_registry,
            cold_registry,
        })
    }

    #[cfg(not(feature = "unrdf"))]
    pub fn new(
        hot_predicates: Vec<u64>,
        _cold_hook_ids: Vec<String>,
        hot_registry: HookRegistry,
    ) -> PatternResult<Self> {
        if hot_predicates.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Hot predicates list cannot be empty when unrdf feature is disabled".to_string(),
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
        let mut cold_results = None;

        // Execute hot path hooks if predicates provided
        if !self.hot_predicates.is_empty() {
            let hot_context = HookExecutionContext {
                hook_registry: self.hot_registry.clone(),
                predicate_runs: context.predicate_runs.clone(),
                soa_arrays: context.soa_arrays.clone(),
                tick_budget: context.tick_budget,
            };

            let pattern = HookSequencePattern::new(self.hot_predicates.clone())?;
            let hot_result = pattern.execute_hooks(&hot_context)?;
            hot_results = Some(hot_result);
        }

        // Execute cold path hooks if hook IDs provided
        #[cfg(feature = "unrdf")]
        {
            if !self.cold_hook_ids.is_empty() {
                // Convert SoAArrays to turtle data for cold path
                let turtle_data = self.soa_to_turtle(&context.soa_arrays)?;

                let pattern = UnrdfSequencePattern::with_registry(
                    self.cold_hook_ids.clone(),
                    self.cold_registry.clone(),
                )?;
                let cold_result = pattern.execute_hooks(&turtle_data)?;
                cold_results = Some(cold_result);
            }
        }

        Ok(HybridExecutionResult {
            hot_results,
            cold_results,
        })
    }

    #[cfg(feature = "unrdf")]
    /// Convert SoAArrays to turtle data (simplified - in production would use proper serialization)
    fn soa_to_turtle(&self, _soa: &SoAArrays) -> Result<String, PatternError> {
        // TODO: Implement proper SoA to Turtle conversion
        // For now, return empty turtle data
        Ok(String::new())
    }
}

/// Hybrid hook parallel pattern: Execute hot and cold path hooks concurrently
pub struct HybridParallelPattern {
    hot_predicates: Vec<u64>,
    #[cfg(feature = "unrdf")]
    cold_hook_ids: Vec<String>,
    #[cfg(not(feature = "unrdf"))]
    _phantom: PhantomData<()>,
    hot_registry: HookRegistry,
    #[cfg(feature = "unrdf")]
    cold_registry: Arc<NativeHookRegistry>,
}

impl HybridParallelPattern {
    /// Create new hybrid parallel pattern
    #[cfg(feature = "unrdf")]
    pub fn new(
        hot_predicates: Vec<u64>,
        cold_hook_ids: Vec<String>,
        hot_registry: HookRegistry,
    ) -> PatternResult<Self> {
        Self::with_registries(
            hot_predicates,
            cold_hook_ids,
            hot_registry,
            Arc::new(NativeHookRegistry::new()),
        )
    }

    /// Create new hybrid parallel pattern with registries
    #[cfg(feature = "unrdf")]
    pub fn with_registries(
        hot_predicates: Vec<u64>,
        cold_hook_ids: Vec<String>,
        hot_registry: HookRegistry,
        cold_registry: Arc<NativeHookRegistry>,
    ) -> PatternResult<Self> {
        if hot_predicates.is_empty() && cold_hook_ids.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "At least one hot predicate or cold hook ID required".to_string(),
            ));
        }

        Ok(Self {
            hot_predicates,
            cold_hook_ids,
            hot_registry,
            cold_registry,
        })
    }

    #[cfg(not(feature = "unrdf"))]
    pub fn new(
        hot_predicates: Vec<u64>,
        _cold_hook_ids: Vec<String>,
        hot_registry: HookRegistry,
    ) -> PatternResult<Self> {
        if hot_predicates.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Hot predicates list cannot be empty when unrdf feature is disabled".to_string(),
            ));
        }

        Ok(Self {
            hot_predicates,
            _phantom: PhantomData,
            hot_registry,
        })
    }

    /// Execute hot and cold path hooks in parallel
    pub fn execute(
        &self,
        context: &HookExecutionContext,
    ) -> Result<HybridExecutionResult, PatternError> {
        use crate::hook_patterns::HookParallelPattern;
        use rayon::prelude::*;

        let mut hot_results = None;
        let mut cold_results = None;

        // Execute hot and cold paths in parallel using rayon
        let mut results_vec = Vec::new();

        // Hot path execution
        if !self.hot_predicates.is_empty() {
            let hot_context = HookExecutionContext {
                hook_registry: self.hot_registry.clone(),
                predicate_runs: context.predicate_runs.clone(),
                soa_arrays: context.soa_arrays.clone(),
                tick_budget: context.tick_budget,
            };

            let pattern = HookParallelPattern::new(self.hot_predicates.clone())?;
            match pattern.execute_hooks(&hot_context) {
                Ok(result) => {
                    hot_results = Some(result);
                    results_vec.push(Ok(()));
                }
                Err(e) => results_vec.push(Err(e)),
            }
        } else {
            results_vec.push(Ok(()));
        }

        // Cold path execution
        #[cfg(feature = "unrdf")]
        {
            if !self.cold_hook_ids.is_empty() {
                let turtle_data = self.soa_to_turtle(&context.soa_arrays)?;
                let pattern = UnrdfParallelPattern::with_registry(
                    self.cold_hook_ids.clone(),
                    self.cold_registry.clone(),
                )?;
                match pattern.execute_hooks(&turtle_data) {
                    Ok(result) => {
                        cold_results = Some(result);
                        results_vec.push(Ok(()));
                    }
                    Err(e) => results_vec.push(Err(e)),
                }
            } else {
                results_vec.push(Ok(()));
            }
        }

        let results: Vec<Result<(), PatternError>> = results_vec.into_par_iter().collect();

        // Check for errors
        for result in results {
            result?;
        }

        Ok(HybridExecutionResult {
            hot_results,
            cold_results,
        })
    }

    #[cfg(feature = "unrdf")]
    /// Convert SoAArrays to turtle data (simplified - in production would use proper serialization)
    fn soa_to_turtle(&self, _soa: &SoAArrays) -> Result<String, PatternError> {
        // TODO: Implement proper SoA to Turtle conversion
        // For now, return empty turtle data
        Ok(String::new())
    }
}

/// Hybrid hook choice pattern: Route between hot and cold paths based on condition
pub struct HybridChoicePattern {
    condition: HybridHookCondition,
    hot_predicates: Vec<u64>,
    #[cfg(feature = "unrdf")]
    cold_hook_ids: Vec<String>,
    #[cfg(not(feature = "unrdf"))]
    _phantom: PhantomData<()>,
    hot_registry: HookRegistry,
    #[cfg(feature = "unrdf")]
    cold_registry: Arc<NativeHookRegistry>,
}

impl HybridChoicePattern {
    /// Create new hybrid choice pattern
    #[cfg(feature = "unrdf")]
    pub fn new(
        condition: HybridHookCondition,
        hot_registry: HookRegistry,
        cold_hook_ids: Vec<String>,
    ) -> PatternResult<Self> {
        Self::with_registries(
            condition,
            Vec::new(),
            cold_hook_ids,
            hot_registry,
            Arc::new(NativeHookRegistry::new()),
        )
    }

    /// Create new hybrid choice pattern with hot predicates
    #[cfg(feature = "unrdf")]
    pub fn with_hot_predicates(
        condition: HybridHookCondition,
        hot_predicates: Vec<u64>,
        hot_registry: HookRegistry,
        cold_hook_ids: Vec<String>,
    ) -> PatternResult<Self> {
        Self::with_registries(
            condition,
            hot_predicates,
            cold_hook_ids,
            hot_registry,
            Arc::new(NativeHookRegistry::new()),
        )
    }

    /// Create new hybrid choice pattern with registries
    #[cfg(feature = "unrdf")]
    pub fn with_registries(
        condition: HybridHookCondition,
        hot_predicates: Vec<u64>,
        cold_hook_ids: Vec<String>,
        hot_registry: HookRegistry,
        cold_registry: Arc<NativeHookRegistry>,
    ) -> PatternResult<Self> {
        Ok(Self {
            condition,
            hot_predicates,
            cold_hook_ids,
            hot_registry,
            cold_registry,
        })
    }

    #[cfg(not(feature = "unrdf"))]
    pub fn new(
        condition: HybridHookCondition,
        hot_registry: HookRegistry,
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
        // Evaluate condition
        let use_cold_path = (self.condition)(context);

        if use_cold_path {
            // Execute cold path hooks
            #[cfg(feature = "unrdf")]
            {
                if !self.cold_hook_ids.is_empty() {
                    let turtle_data = self.soa_to_turtle(&context.soa_arrays)?;
                    let pattern = UnrdfSequencePattern::with_registry(
                        self.cold_hook_ids.clone(),
                        self.cold_registry.clone(),
                    )?;
                    let cold_result = pattern.execute_hooks(&turtle_data)?;
                    return Ok(HybridExecutionResult {
                        hot_results: None,
                        cold_results: Some(cold_result),
                    });
                }
            }

            // Cold path requested but no hooks or feature disabled
            Err(PatternError::ExecutionFailed(
                "Cold path requested but no hooks available or unrdf feature disabled".to_string(),
            ))
        } else {
            // Execute hot path hooks
            if !self.hot_predicates.is_empty() {
                use crate::hook_patterns::HookSequencePattern;

                let hot_context = HookExecutionContext {
                    hook_registry: self.hot_registry.clone(),
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
        }
    }

    #[cfg(feature = "unrdf")]
    /// Convert SoAArrays to turtle data (simplified - in production would use proper serialization)
    fn soa_to_turtle(&self, _soa: &SoAArrays) -> Result<String, PatternError> {
        // TODO: Implement proper SoA to Turtle conversion
        // For now, return empty turtle data
        Ok(String::new())
    }
}

/// Hybrid execution result: Contains results from both hot and cold paths
pub struct HybridExecutionResult {
    /// Hot path hook execution results
    pub hot_results: Option<HookExecutionResult>,
    /// Cold path hook execution results
    #[cfg(feature = "unrdf")]
    pub cold_results: Option<Vec<HookResult>>,
    #[cfg(not(feature = "unrdf"))]
    pub cold_results: Option<()>,
}
