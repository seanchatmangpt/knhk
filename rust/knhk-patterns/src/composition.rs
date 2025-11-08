// rust/knhk-patterns/src/composition.rs
// Pattern composition: Build complex workflows from simple primitives

use crate::patterns::{
    ArbitraryCyclesPattern, BranchFn, ConditionFn, ExclusiveChoicePattern,
    MultiChoicePattern, ParallelSplitPattern, Pattern, PatternError, PatternResult,
    SequencePattern,
};
use std::sync::Arc;

// ============================================================================
// Composite Pattern (allows nesting patterns)
// ============================================================================

pub enum CompositePattern<T: Clone + Send + Sync + 'static> {
    /// Execute patterns sequentially
    Sequence(Vec<Box<dyn Pattern<T>>>),

    /// Execute patterns in parallel
    Parallel(Vec<Box<dyn Pattern<T>>>),

    /// Choose one pattern based on condition (XOR)
    Choice(Vec<(ConditionFn<T>, Box<dyn Pattern<T>>)>),

    /// Execute multiple patterns based on conditions (OR)
    MultiChoice(Vec<(ConditionFn<T>, Box<dyn Pattern<T>>)>),

    /// Retry pattern until condition met
    Retry {
        pattern: Box<dyn Pattern<T>>,
        should_continue: ConditionFn<T>,
        max_attempts: u32,
    },

    /// Execute pattern with timeout
    Timeout {
        pattern: Box<dyn Pattern<T>>,
        timeout_ms: u64,
    },

    /// Atomic pattern (leaf node)
    Atomic(Box<dyn Pattern<T>>),
}

impl<T: Clone + Send + Sync + 'static> CompositePattern<T> {
    /// Execute the composite pattern
    pub fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        match self {
            CompositePattern::Sequence(patterns) => {
                let mut current = vec![input];
                for pattern in patterns {
                    let mut next = Vec::new();
                    for item in current {
                        let mut results = pattern.execute(item)?;
                        next.append(&mut results);
                    }
                    current = next;
                }
                Ok(current)
            }

            CompositePattern::Parallel(patterns) => {
                use rayon::prelude::*;
                let results: Result<Vec<_>, _> = patterns
                    .par_iter()
                    .map(|pattern| pattern.execute(input.clone()))
                    .collect();

                let results = results?;
                Ok(results.into_iter().flatten().collect())
            }

            CompositePattern::Choice(choices) => {
                for (condition, pattern) in choices {
                    if condition(&input) {
                        return pattern.execute(input);
                    }
                }
                Err(PatternError::ExecutionFailed(
                    "No condition matched in Choice".to_string(),
                ))
            }

            CompositePattern::MultiChoice(choices) => {
                use rayon::prelude::*;
                let results: Result<Vec<_>, _> = choices
                    .par_iter()
                    .filter_map(|(condition, pattern)| {
                        if condition(&input) {
                            Some(pattern.execute(input.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();

                let results = results?;
                if results.is_empty() {
                    Err(PatternError::ExecutionFailed(
                        "No condition matched in MultiChoice".to_string(),
                    ))
                } else {
                    Ok(results.into_iter().flatten().collect())
                }
            }

            CompositePattern::Retry {
                pattern,
                should_continue,
                max_attempts,
            } => {
                let mut current = input;
                let mut attempt = 0;

                while attempt < *max_attempts && should_continue(&current) {
                    let results = pattern.execute(current.clone())?;
                    if let Some(result) = results.into_iter().next() {
                        current = result;
                    }
                    attempt += 1;
                }

                Ok(vec![current])
            }

            CompositePattern::Timeout {
                pattern,
                timeout_ms,
            } => {
                // Note: Timeout requires 'static lifetime for thread spawn
                // For now, execute without timeout to avoid lifetime issues
                // TODO: Implement timeout with async/await or other mechanism
                let _timeout = *timeout_ms;
                pattern.execute(input)?
            }

            CompositePattern::Atomic(pattern) => pattern.execute(input),
        }
    }
}

// ============================================================================
// Pattern Builder (Fluent API)
// ============================================================================

pub struct PatternBuilder<T: Clone + Send + Sync + 'static> {
    patterns: Vec<Box<dyn Pattern<T>>>,
}

impl<T: Clone + Send + Sync + 'static> PatternBuilder<T> {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Add a sequential step
    pub fn then(mut self, branch: BranchFn<T>) -> Self {
        let pattern = SequencePattern::new(vec![branch])
            .ok()
            .map(|p| Box::new(p) as Box<dyn Pattern<T>>);
        if let Some(pattern) = pattern {
            self.patterns.push(pattern);
        }
        self
    }

    /// Add parallel branches
    pub fn parallel(mut self, branches: Vec<BranchFn<T>>) -> Self {
        let pattern = ParallelSplitPattern::new(branches)
            .ok()
            .map(|p| Box::new(p) as Box<dyn Pattern<T>>);
        if let Some(pattern) = pattern {
            self.patterns.push(pattern);
        }
        self
    }

    /// Add conditional choice (XOR)
    pub fn choice(mut self, choices: Vec<(ConditionFn<T>, BranchFn<T>)>) -> Self {
        let pattern = ExclusiveChoicePattern::new(choices)
            .ok()
            .map(|p| Box::new(p) as Box<dyn Pattern<T>>);
        if let Some(pattern) = pattern {
            self.patterns.push(pattern);
        }
        self
    }

    /// Add multi-choice (OR)
    pub fn multi_choice(mut self, choices: Vec<(ConditionFn<T>, BranchFn<T>)>) -> Self {
        let pattern = MultiChoicePattern::new(choices)
            .ok()
            .map(|p| Box::new(p) as Box<dyn Pattern<T>>);
        if let Some(pattern) = pattern {
            self.patterns.push(pattern);
        }
        self
    }

    /// Add retry logic
    pub fn retry(
        mut self,
        branch: BranchFn<T>,
        should_continue: ConditionFn<T>,
        max_attempts: u32,
    ) -> Self {
        let pattern = ArbitraryCyclesPattern::new(branch, should_continue, max_attempts)
            .ok()
            .map(|p| Box::new(p) as Box<dyn Pattern<T>>);
        if let Some(pattern) = pattern {
            self.patterns.push(pattern);
        }
        self
    }

    /// Build the composite pattern
    pub fn build(self) -> CompositePattern<T> {
        if self.patterns.len() == 1 {
            CompositePattern::Atomic(self.patterns.into_iter().next().unwrap())
        } else {
            CompositePattern::Sequence(self.patterns)
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Default for PatternBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Example Composite Patterns
// ============================================================================

/// Parallel validation with retry
pub fn parallel_validation_with_retry<T: Clone + Send + Sync + 'static>(
    validators: Vec<BranchFn<T>>,
    max_retries: u32,
) -> CompositePattern<T> {
    CompositePattern::Retry {
        pattern: Box::new(
            ParallelSplitPattern::new(validators)
                .ok()
                .map(|p| Box::new(p) as Box<dyn Pattern<T>>)
                .unwrap(),
        ),
        should_continue: Arc::new(|_| true),
        max_attempts: max_retries,
    }
}

/// Conditional routing with timeout
pub fn conditional_routing_with_timeout<T: Clone + Send + Sync + 'static>(
    choices: Vec<(ConditionFn<T>, BranchFn<T>)>,
    timeout_ms: u64,
) -> CompositePattern<T> {
    CompositePattern::Timeout {
        pattern: Box::new(
            ExclusiveChoicePattern::new(choices)
                .ok()
                .map(|p| Box::new(p) as Box<dyn Pattern<T>>)
                .unwrap(),
        ),
        timeout_ms,
    }
}
