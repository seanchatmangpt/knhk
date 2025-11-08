// rust/knhk-patterns/src/hook_patterns.rs
// Hook-specific pattern types for orchestrating knowledge hook execution

use crate::patterns::{PatternError, PatternResult};
use knhk_etl::{
    hook_orchestration::{HookExecutionContext, HookExecutionPattern, HookExecutionResult},
    hook_registry::HookRegistry,
    load::{LoadResult, PredRun, SoAArrays},
};
use std::sync::Arc;

/// Hook condition function type: evaluates on execution context
pub type HookCondition = Arc<dyn Fn(&HookExecutionContext) -> bool + Send + Sync>;

/// Hook retry condition function type: evaluates on receipt
pub type HookRetryCondition = Arc<dyn Fn(&knhk_etl::Receipt) -> bool + Send + Sync>;

/// Hook sequence pattern: Execute hooks sequentially
pub struct HookSequencePattern {
    predicates: Vec<u64>,
}

impl HookSequencePattern {
    /// Create new hook sequence pattern
    pub fn new(predicates: Vec<u64>) -> PatternResult<Self> {
        if predicates.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Predicates list cannot be empty".to_string(),
            ));
        }

        Ok(Self { predicates })
    }

    /// Execute hooks sequentially
    pub fn execute_hooks(
        &self,
        context: &HookExecutionContext,
    ) -> Result<HookExecutionResult, PatternError> {
        use knhk_etl::hook_orchestration::HookOrchestrator;

        let orchestrator = HookOrchestrator::new();
        orchestrator
            .execute_with_pattern(context, HookExecutionPattern::Sequence(self.predicates.clone()))
            .map_err(|e| PatternError::ExecutionFailed(e.message().to_string()))
    }
}

/// Hook parallel pattern: Execute hooks in parallel
pub struct HookParallelPattern {
    predicates: Vec<u64>,
    use_simd: bool,
}

impl HookParallelPattern {
    /// Create new hook parallel pattern
    pub fn new(predicates: Vec<u64>) -> PatternResult<Self> {
        Self::with_simd(predicates, false)
    }

    /// Create new hook parallel pattern with SIMD option
    pub fn with_simd(predicates: Vec<u64>, use_simd: bool) -> PatternResult<Self> {
        if predicates.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Predicates list cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            predicates,
            use_simd,
        })
    }

    /// Execute hooks in parallel
    pub fn execute_hooks(
        &self,
        context: &HookExecutionContext,
    ) -> Result<HookExecutionResult, PatternError> {
        use knhk_etl::hook_orchestration::HookOrchestrator;

        let orchestrator = HookOrchestrator::new();
        orchestrator
            .execute_with_pattern(
                context,
                HookExecutionPattern::Parallel(self.predicates.clone()),
            )
            .map_err(|e| PatternError::ExecutionFailed(e.message().to_string()))
    }
}

/// Hook choice pattern: Conditional hook routing
pub struct HookChoicePattern {
    choices: Vec<(HookCondition, u64)>,
}

impl HookChoicePattern {
    /// Create new hook choice pattern
    pub fn new(choices: Vec<(HookCondition, u64)>) -> PatternResult<Self> {
        if choices.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Choices list cannot be empty".to_string(),
            ));
        }

        Ok(Self { choices })
    }

    /// Execute hooks conditionally
    pub fn execute_hooks(
        &self,
        context: &HookExecutionContext,
    ) -> Result<HookExecutionResult, PatternError> {
        use knhk_etl::hook_orchestration::HookOrchestrator;

        // Convert Arc conditions to boxed closures
        // Use type alias to reduce complexity for clippy compliance
        type BoxedHookCondition = Box<dyn Fn(&HookExecutionContext) -> bool + Send + Sync>;
        let boxed_choices: Vec<(BoxedHookCondition, u64)> = self
            .choices
            .iter()
            .map(|(cond, pred)| {
                let cond_clone = cond.clone();
                (
                    Box::new(move |ctx: &HookExecutionContext| (cond_clone)(ctx)) as BoxedHookCondition,
                    *pred,
                )
            })
            .collect();

        let orchestrator = HookOrchestrator::new();
        orchestrator
            .execute_with_pattern(context, HookExecutionPattern::Choice(boxed_choices))
            .map_err(|e| PatternError::ExecutionFailed(e.message().to_string()))
    }
}

/// Hook retry pattern: Retry logic for hooks
pub struct HookRetryPattern {
    predicate: u64,
    should_retry: HookRetryCondition,
    max_attempts: u32,
}

impl HookRetryPattern {
    /// Create new hook retry pattern
    pub fn new(
        predicate: u64,
        should_retry: HookRetryCondition,
        max_attempts: u32,
    ) -> PatternResult<Self> {
        if max_attempts == 0 {
            return Err(PatternError::InvalidConfiguration(
                "max_attempts must be > 0".to_string(),
            ));
        }

        Ok(Self {
            predicate,
            should_retry,
            max_attempts,
        })
    }

    /// Execute hook with retry logic
    pub fn execute_hooks(
        &self,
        context: &HookExecutionContext,
    ) -> Result<HookExecutionResult, PatternError> {
        use knhk_etl::hook_orchestration::HookOrchestrator;

        // Convert Arc condition to boxed closure
        let retry_cond = self.should_retry.clone();
        let boxed_retry = Box::new(move |receipt: &knhk_etl::Receipt| (retry_cond)(receipt))
            as Box<dyn Fn(&knhk_etl::Receipt) -> bool + Send + Sync>;

        let orchestrator = HookOrchestrator::new();
        orchestrator
            .execute_with_pattern(
                context,
                HookExecutionPattern::Retry {
                    predicate: self.predicate,
                    should_retry: boxed_retry,
                    max_attempts: self.max_attempts,
                },
            )
            .map_err(|e| PatternError::ExecutionFailed(e.message().to_string()))
    }
}

/// Helper function to create hook execution context from LoadResult
pub fn create_hook_context(
    hook_registry: HookRegistry,
    load_result: LoadResult,
    tick_budget: u32,
) -> HookExecutionContext {
    HookExecutionContext::from_load_result(hook_registry, load_result, tick_budget)
}

/// Helper function to create hook execution context from components
pub fn create_hook_context_from_components(
    hook_registry: HookRegistry,
    predicate_runs: Vec<PredRun>,
    soa_arrays: SoAArrays,
    tick_budget: u32,
) -> HookExecutionContext {
    HookExecutionContext::new(hook_registry, predicate_runs, soa_arrays, tick_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_sequence_pattern_creation() {
        let pattern = HookSequencePattern::new(vec![1, 2, 3]);
        assert!(pattern.is_ok());

        let pattern = HookSequencePattern::new(vec![]);
        assert!(pattern.is_err());
    }

    #[test]
    fn test_hook_parallel_pattern_creation() {
        let pattern = HookParallelPattern::new(vec![1, 2, 3]);
        assert!(pattern.is_ok());

        let pattern = HookParallelPattern::new(vec![]);
        assert!(pattern.is_err());
    }

    #[test]
    fn test_hook_choice_pattern_creation() {
        let choices = vec![
            (
                Arc::new(|_ctx: &HookExecutionContext| true) as HookCondition,
                1u64,
            ),
            (
                Arc::new(|_ctx: &HookExecutionContext| false) as HookCondition,
                2u64,
            ),
        ];

        let pattern = HookChoicePattern::new(choices);
        assert!(pattern.is_ok());

        let pattern = HookChoicePattern::new(vec![]);
        assert!(pattern.is_err());
    }

    #[test]
    fn test_hook_retry_pattern_creation() {
        let pattern = HookRetryPattern::new(
            1u64,
            Arc::new(|_receipt: &knhk_etl::Receipt| true) as HookRetryCondition,
            3,
        );
        assert!(pattern.is_ok());

        let pattern = HookRetryPattern::new(
            1u64,
            Arc::new(|_receipt: &knhk_etl::Receipt| true) as HookRetryCondition,
            0,
        );
        assert!(pattern.is_err());
    }
}

