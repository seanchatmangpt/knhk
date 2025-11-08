// rust/knhk-patterns/src/pipeline_ext.rs
// Extension trait to add workflow patterns to KNHK Pipeline

use crate::patterns::{BranchFn, ConditionFn, PatternResult};
use knhk_etl::{LoadResult, Pipeline};

// ============================================================================
// Pipeline Extension Trait
// ============================================================================

pub trait PipelinePatternExt {
    /// Execute pipeline with parallel split pattern
    fn execute_parallel<F>(&mut self, processors: Vec<F>) -> PatternResult<Vec<LoadResult>>
    where
        F: Fn(LoadResult) -> PatternResult<LoadResult> + Send + Sync + 'static;

    /// Execute pipeline with conditional routing
    fn execute_conditional<F, C>(
        &mut self,
        choices: Vec<(C, F)>,
    ) -> PatternResult<Vec<LoadResult>>
    where
        F: Fn(LoadResult) -> PatternResult<LoadResult> + Send + Sync + 'static,
        C: Fn(&LoadResult) -> bool + Send + Sync + 'static;

    /// Execute pipeline with retry pattern
    fn execute_with_retry<F, C>(
        &mut self,
        processor: F,
        should_retry: C,
        max_attempts: u32,
    ) -> PatternResult<LoadResult>
    where
        F: Fn(LoadResult) -> PatternResult<LoadResult> + Send + Sync + 'static,
        C: Fn(&LoadResult) -> bool + Send + Sync + 'static;
}

impl PipelinePatternExt for Pipeline {
    fn execute_parallel<F>(&mut self, processors: Vec<F>) -> PatternResult<Vec<LoadResult>>
    where
        F: Fn(LoadResult) -> PatternResult<LoadResult> + Send + Sync + 'static,
    {
        use crate::patterns::ParallelSplitPattern;
        use std::sync::Arc;

        // Execute standard pipeline first
        let result = self
            .execute()
            .map_err(|e| crate::patterns::PatternError::ExecutionFailed(e.to_string()))?;

        // Create branch functions from processors
        let branches: Vec<BranchFn<LoadResult>> = processors
            .into_iter()
            .map(|p| Arc::new(p) as BranchFn<LoadResult>)
            .collect();

        // Execute parallel split pattern
        let pattern = ParallelSplitPattern::new(branches)?;
        pattern.execute(result)
    }

    fn execute_conditional<F, C>(
        &mut self,
        choices: Vec<(C, F)>,
    ) -> PatternResult<Vec<LoadResult>>
    where
        F: Fn(LoadResult) -> PatternResult<LoadResult> + Send + Sync + 'static,
        C: Fn(&LoadResult) -> bool + Send + Sync + 'static,
    {
        use crate::patterns::ExclusiveChoicePattern;
        use std::sync::Arc;

        // Execute standard pipeline first
        let result = self
            .execute()
            .map_err(|e| crate::patterns::PatternError::ExecutionFailed(e.to_string()))?;

        // Create condition-branch pairs
        let pattern_choices: Vec<(ConditionFn<LoadResult>, BranchFn<LoadResult>)> = choices
            .into_iter()
            .map(|(c, f)| {
                let condition = Arc::new(c) as ConditionFn<LoadResult>;
                let branch = Arc::new(f) as BranchFn<LoadResult>;
                (condition, branch)
            })
            .collect();

        // Execute exclusive choice pattern
        let pattern = ExclusiveChoicePattern::new(pattern_choices)?;
        pattern.execute(result)
    }

    fn execute_with_retry<F, C>(
        &mut self,
        processor: F,
        should_retry: C,
        max_attempts: u32,
    ) -> PatternResult<LoadResult>
    where
        F: Fn(LoadResult) -> PatternResult<LoadResult> + Send + Sync + 'static,
        C: Fn(&LoadResult) -> bool + Send + Sync + 'static,
    {
        use crate::patterns::ArbitraryCyclesPattern;
        use std::sync::Arc;

        // Execute standard pipeline first
        let result = self
            .execute()
            .map_err(|e| crate::patterns::PatternError::ExecutionFailed(e.to_string()))?;

        // Create retry pattern
        let branch = Arc::new(processor) as BranchFn<LoadResult>;
        let condition = Arc::new(should_retry) as ConditionFn<LoadResult>;

        let pattern = ArbitraryCyclesPattern::new(branch, condition, max_attempts)?;
        let results = pattern.execute(result)?;

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
            Arc::new(|result: LoadResult| Ok(result)) as BranchFn<LoadResult>,
            Arc::new(|result: LoadResult| Ok(result)) as BranchFn<LoadResult>,
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
                Arc::new(|result: &LoadResult| result.runs.len() > 100)
                    as ConditionFn<LoadResult>,
                Arc::new(|result: LoadResult| Ok(result)) as BranchFn<LoadResult>,
            ),
            (
                Arc::new(|_result: &LoadResult| true) as ConditionFn<LoadResult>,
                Arc::new(|result: LoadResult| Ok(result)) as BranchFn<LoadResult>,
            ),
        ];

        let _pattern = crate::patterns::ExclusiveChoicePattern::new(choices);
    }

    #[test]
    fn example_retry_with_backoff() {
        // Example: Retry validation with exponential backoff

        let processor = Arc::new(|result: LoadResult| Ok(result)) as BranchFn<LoadResult>;

        let should_retry =
            Arc::new(|result: &LoadResult| result.runs.is_empty()) as ConditionFn<LoadResult>;

        let _pattern = crate::patterns::ArbitraryCyclesPattern::new(processor, should_retry, 3);
    }
}
