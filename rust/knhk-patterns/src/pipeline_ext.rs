// rust/knhk-patterns/src/pipeline_ext.rs
// Extension trait to add workflow patterns to KNHK Pipeline

use crate::patterns::{BranchFn, ConditionFn, Pattern, PatternResult};
use knhk_etl::{EmitResult, Pipeline};

// ============================================================================
// Pipeline Extension Trait
// ============================================================================

pub trait PipelinePatternExt {
    /// Execute pipeline with parallel split pattern
    fn execute_parallel<F>(&mut self, processors: Vec<F>) -> PatternResult<Vec<EmitResult>>
    where
        F: Fn(EmitResult) -> PatternResult<EmitResult> + Send + Sync + 'static;

    /// Execute pipeline with conditional routing
    fn execute_conditional<F, C>(
        &mut self,
        choices: Vec<(C, F)>,
    ) -> PatternResult<Vec<EmitResult>>
    where
        F: Fn(EmitResult) -> PatternResult<EmitResult> + Send + Sync + 'static,
        C: Fn(&EmitResult) -> bool + Send + Sync + 'static;

    /// Execute pipeline with retry pattern
    fn execute_with_retry<F, C>(
        &mut self,
        processor: F,
        should_retry: C,
        max_attempts: u32,
    ) -> PatternResult<EmitResult>
    where
        F: Fn(EmitResult) -> PatternResult<EmitResult> + Send + Sync + 'static,
        C: Fn(&EmitResult) -> bool + Send + Sync + 'static;
}

impl PipelinePatternExt for Pipeline {
    fn execute_parallel<F>(&mut self, processors: Vec<F>) -> PatternResult<Vec<EmitResult>>
    where
        F: Fn(EmitResult) -> PatternResult<EmitResult> + Send + Sync + 'static,
    {
        use crate::patterns::ParallelSplitPattern;
        use std::sync::Arc;

        // Execute standard pipeline first
        let result = self
            .execute()
            .map_err(|e| crate::patterns::PatternError::ExecutionFailed(e.message().to_string()))?;

        // Create branch functions from processors
        let branches: Vec<BranchFn<EmitResult>> = processors
            .into_iter()
            .map(|p| Arc::new(p) as BranchFn<EmitResult>)
            .collect();

        // Execute parallel split pattern
        let pattern = ParallelSplitPattern::new(branches)?;
        Pattern::execute(&pattern, result)
    }

    fn execute_conditional<F, C>(
        &mut self,
        choices: Vec<(C, F)>,
    ) -> PatternResult<Vec<EmitResult>>
    where
        F: Fn(EmitResult) -> PatternResult<EmitResult> + Send + Sync + 'static,
        C: Fn(&EmitResult) -> bool + Send + Sync + 'static,
    {
        use crate::patterns::ExclusiveChoicePattern;
        use std::sync::Arc;

        // Execute standard pipeline first
        let result = self
            .execute()
            .map_err(|e| crate::patterns::PatternError::ExecutionFailed(e.message().to_string()))?;

        // Create condition-branch pairs
        let pattern_choices: Vec<(ConditionFn<EmitResult>, BranchFn<EmitResult>)> = choices
            .into_iter()
            .map(|(c, f)| {
                let condition = Arc::new(c) as ConditionFn<EmitResult>;
                let branch = Arc::new(f) as BranchFn<EmitResult>;
                (condition, branch)
            })
            .collect();

        // Execute exclusive choice pattern
        let pattern = ExclusiveChoicePattern::new(pattern_choices)?;
        Pattern::execute(&pattern, result)
    }

    fn execute_with_retry<F, C>(
        &mut self,
        processor: F,
        should_retry: C,
        max_attempts: u32,
    ) -> PatternResult<EmitResult>
    where
        F: Fn(EmitResult) -> PatternResult<EmitResult> + Send + Sync + 'static,
        C: Fn(&EmitResult) -> bool + Send + Sync + 'static,
    {
        use crate::patterns::ArbitraryCyclesPattern;
        use std::sync::Arc;

        // Execute standard pipeline first
        let result = self
            .execute()
            .map_err(|e| crate::patterns::PatternError::ExecutionFailed(e.message().to_string()))?;

        // Create retry pattern
        let branch = Arc::new(processor) as BranchFn<EmitResult>;
        let condition = Arc::new(should_retry) as ConditionFn<EmitResult>;

        let pattern = ArbitraryCyclesPattern::new(branch, condition, max_attempts)?;
        let results = Pattern::execute(&pattern, result)?;

        results
            .into_iter()
            .next()
            .ok_or_else(|| crate::patterns::PatternError::ExecutionFailed("No result".to_string()))
    }
}

// ============================================================================
// Example Usage Patterns
// ============================================================================

#[cfg(test)]
mod examples {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn example_parallel_validation() {
        // Example: Validate data through multiple validators in parallel
        use crate::patterns::ParallelSplitPattern;

        let validators = vec![
            Arc::new(|result: EmitResult| Ok(result)) as BranchFn<EmitResult>,
            Arc::new(|result: EmitResult| Ok(result)) as BranchFn<EmitResult>,
        ];

        let _pattern = ParallelSplitPattern::new(validators);

        // This would be used with a pipeline:
        // let results = pattern.execute(pipeline_result)?;
    }

    #[test]
    fn example_conditional_routing() {
        // Example: Route to different processors based on data characteristics

        let choices = vec![
            (
                Arc::new(|result: &EmitResult| result.receipts_written > 100)
                    as ConditionFn<EmitResult>,
                Arc::new(|result: EmitResult| Ok(result)) as BranchFn<EmitResult>,
            ),
            (
                Arc::new(|_result: &EmitResult| true) as ConditionFn<EmitResult>,
                Arc::new(|result: EmitResult| Ok(result)) as BranchFn<EmitResult>,
            ),
        ];

        let _pattern = crate::patterns::ExclusiveChoicePattern::new(choices);
    }

    #[test]
    fn example_retry_with_backoff() {
        // Example: Retry validation with exponential backoff

        let processor = Arc::new(|result: EmitResult| Ok(result)) as BranchFn<EmitResult>;

        let should_retry =
            Arc::new(|result: &EmitResult| result.receipts_written == 0) as ConditionFn<EmitResult>;

        let _pattern = crate::patterns::ArbitraryCyclesPattern::new(processor, should_retry, 3);
    }
}
