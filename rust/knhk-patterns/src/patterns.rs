// rust/knhk-patterns/src/patterns.rs
// High-level Rust API for workflow patterns

use crate::ffi::PatternType;
use rayon::prelude::*;
use std::sync::Arc;

// ============================================================================
// Core Types
// ============================================================================

pub use crate::ffi::PatternType;

#[derive(Debug)]
pub enum PatternError {
    ValidationFailed(String),
    ExecutionFailed(String),
    TooManyBranches,
    InvalidConfiguration(String),
}

impl std::fmt::Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            PatternError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            PatternError::TooManyBranches => write!(f, "Too many branches (max 1024)"),
            PatternError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for PatternError {}

pub type PatternResult<T> = Result<T, PatternError>;

// Branch function type (pure Rust)
pub type BranchFn<T> = Arc<dyn Fn(T) -> PatternResult<T> + Send + Sync>;

// Condition function type (pure Rust)
pub type ConditionFn<T> = Arc<dyn Fn(&T) -> bool + Send + Sync>;

// ============================================================================
// Pattern Trait
// ============================================================================

pub trait Pattern<T>: Send + Sync {
    fn pattern_type(&self) -> PatternType;
    fn execute(&self, input: T) -> PatternResult<Vec<T>>;
    fn tick_budget(&self) -> u32 {
        self.pattern_type().tick_budget()
    }
}

// ============================================================================
// Pattern 1: Sequence
// ============================================================================

pub struct SequencePattern<T> {
    branches: Vec<BranchFn<T>>,
}

impl<T: Clone + Send + Sync> SequencePattern<T> {
    pub fn new(branches: Vec<BranchFn<T>>) -> PatternResult<Self> {
        // Ingress validation
        PatternType::Sequence.validate_ingress(branches.len() as u32)
            .map_err(PatternError::ValidationFailed)?;

        Ok(Self { branches })
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for SequencePattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::Sequence
    }

    fn execute(&self, mut input: T) -> PatternResult<Vec<T>> {
        // Execute branches sequentially
        for branch in &self.branches {
            input = branch(input)?;
        }
        Ok(vec![input])
    }
}

// ============================================================================
// Pattern 2: Parallel Split
// ============================================================================

pub struct ParallelSplitPattern<T> {
    branches: Vec<BranchFn<T>>,
    use_simd: bool,
}

impl<T: Clone + Send + Sync> ParallelSplitPattern<T> {
    pub fn new(branches: Vec<BranchFn<T>>) -> PatternResult<Self> {
        Self::with_simd(branches, false)
    }

    pub fn with_simd(branches: Vec<BranchFn<T>>, use_simd: bool) -> PatternResult<Self> {
        // Ingress validation
        PatternType::ParallelSplit.validate_ingress(branches.len() as u32)
            .map_err(PatternError::ValidationFailed)?;

        Ok(Self { branches, use_simd })
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for ParallelSplitPattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::ParallelSplit
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        // Execute all branches in parallel using Rayon
        let results: Result<Vec<_>, _> = self
            .branches
            .par_iter()
            .map(|branch| branch(input.clone()))
            .collect();

        results.map_err(|e| PatternError::ExecutionFailed(e.to_string()))
    }
}

// ============================================================================
// Pattern 3: Synchronization
// ============================================================================

pub struct SynchronizationPattern<T> {
    use_simd: bool,
}

impl<T: Clone + Send + Sync> SynchronizationPattern<T> {
    pub fn new() -> Self {
        Self { use_simd: false }
    }

    pub fn with_simd() -> Self {
        Self { use_simd: true }
    }
}

impl<T: Clone + Send + Sync> Default for SynchronizationPattern<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for SynchronizationPattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::Synchronization
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        // Synchronization just passes through
        // (actual synchronization happens in ParallelSplit execution)
        Ok(vec![input])
    }
}

// ============================================================================
// Pattern 4: Exclusive Choice (XOR-split)
// ============================================================================

pub struct ExclusiveChoicePattern<T> {
    choices: Vec<(ConditionFn<T>, BranchFn<T>)>,
}

impl<T: Clone + Send + Sync> ExclusiveChoicePattern<T> {
    pub fn new(choices: Vec<(ConditionFn<T>, BranchFn<T>)>) -> PatternResult<Self> {
        // Ingress validation
        PatternType::ExclusiveChoice.validate_ingress(choices.len() as u32)
            .map_err(PatternError::ValidationFailed)?;

        Ok(Self { choices })
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for ExclusiveChoicePattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::ExclusiveChoice
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        // Find first matching condition (XOR: only one should match)
        for (condition, branch) in &self.choices {
            if condition(&input) {
                let result = branch(input)?;
                return Ok(vec![result]);
            }
        }

        Err(PatternError::ExecutionFailed(
            "No condition matched".to_string(),
        ))
    }
}

// ============================================================================
// Pattern 5: Simple Merge (XOR-join)
// ============================================================================

pub struct SimpleMergePattern<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Send + Sync> SimpleMergePattern<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Clone + Send + Sync> Default for SimpleMergePattern<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for SimpleMergePattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::SimpleMerge
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        // Simple merge just passes through (XOR-join: continue immediately)
        Ok(vec![input])
    }
}

// ============================================================================
// Pattern 6: Multi-Choice (OR-split)
// ============================================================================

pub struct MultiChoicePattern<T> {
    choices: Vec<(ConditionFn<T>, BranchFn<T>)>,
    use_simd: bool,
}

impl<T: Clone + Send + Sync> MultiChoicePattern<T> {
    pub fn new(choices: Vec<(ConditionFn<T>, BranchFn<T>)>) -> PatternResult<Self> {
        Self::with_simd(choices, false)
    }

    pub fn with_simd(
        choices: Vec<(ConditionFn<T>, BranchFn<T>)>,
        use_simd: bool,
    ) -> PatternResult<Self> {
        // Ingress validation
        PatternType::MultiChoice.validate_ingress(choices.len() as u32)
            .map_err(PatternError::ValidationFailed)?;

        Ok(Self { choices, use_simd })
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for MultiChoicePattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::MultiChoice
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        // Execute all branches whose conditions match (OR-split)
        let results: Result<Vec<_>, _> = self
            .choices
            .par_iter()
            .filter_map(|(condition, branch)| {
                if condition(&input) {
                    Some(branch(input.clone()))
                } else {
                    None
                }
            })
            .collect();

        let results = results.map_err(|e| PatternError::ExecutionFailed(e.to_string()))?;

        if results.is_empty() {
            Err(PatternError::ExecutionFailed(
                "No condition matched".to_string(),
            ))
        } else {
            Ok(results)
        }
    }
}

// ============================================================================
// Pattern 10: Arbitrary Cycles (Retry/Loop)
// ============================================================================

pub struct ArbitraryCyclesPattern<T> {
    branch: BranchFn<T>,
    should_continue: ConditionFn<T>,
    max_iterations: u32,
}

impl<T: Clone + Send + Sync> ArbitraryCyclesPattern<T> {
    pub fn new(
        branch: BranchFn<T>,
        should_continue: ConditionFn<T>,
        max_iterations: u32,
    ) -> PatternResult<Self> {
        // Ingress validation
        PatternType::ArbitraryCycles.validate_ingress(1)
            .map_err(PatternError::ValidationFailed)?;

        if max_iterations == 0 {
            return Err(PatternError::InvalidConfiguration(
                "max_iterations must be > 0".to_string(),
            ));
        }

        Ok(Self {
            branch,
            should_continue,
            max_iterations,
        })
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for ArbitraryCyclesPattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::ArbitraryCycles
    }

    fn execute(&self, mut input: T) -> PatternResult<Vec<T>> {
        let mut iteration = 0;

        // Execute until condition met or max iterations reached
        while iteration < self.max_iterations && (self.should_continue)(&input) {
            input = (self.branch)(input)?;
            iteration += 1;
        }

        Ok(vec![input])
    }
}

// ============================================================================
// Pattern 16: Deferred Choice (Event-driven)
// ============================================================================

pub struct DeferredChoicePattern<T> {
    choices: Vec<(ConditionFn<T>, BranchFn<T>)>,
    timeout_ms: u64,
}

impl<T: Clone + Send + Sync> DeferredChoicePattern<T> {
    pub fn new(
        choices: Vec<(ConditionFn<T>, BranchFn<T>)>,
        timeout_ms: u64,
    ) -> PatternResult<Self> {
        // Ingress validation
        PatternType::DeferredChoice.validate_ingress(choices.len() as u32)
            .map_err(PatternError::ValidationFailed)?;

        Ok(Self {
            choices,
            timeout_ms,
        })
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for DeferredChoicePattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::DeferredChoice
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        use std::time::{Duration, Instant};

        let start = Instant::now();
        let timeout = Duration::from_millis(self.timeout_ms);

        // Poll conditions until one becomes true or timeout
        loop {
            // Check all conditions
            for (condition, branch) in &self.choices {
                if condition(&input) {
                    let result = branch(input)?;
                    return Ok(vec![result]);
                }
            }

            // Check timeout
            if start.elapsed() > timeout {
                return Err(PatternError::ExecutionFailed(
                    "Timeout waiting for condition".to_string(),
                ));
            }

            // Brief yield to avoid busy-waiting
            std::thread::yield_now();
        }
    }
}
