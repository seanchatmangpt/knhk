// rust/knhk-patterns/src/patterns.rs
// High-level Rust API for workflow patterns

use rayon::prelude::*;
use std::sync::Arc;

// ============================================================================
// Core Types
// ============================================================================

// Re-export PatternType from ffi
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
        PatternType::Sequence
            .validate_ingress(branches.len() as u32)
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
    #[allow(dead_code)]
    use_simd: bool,
}

impl<T: Clone + Send + Sync> ParallelSplitPattern<T> {
    pub fn new(branches: Vec<BranchFn<T>>) -> PatternResult<Self> {
        Self::with_simd(branches, false)
    }

    pub fn with_simd(branches: Vec<BranchFn<T>>, use_simd: bool) -> PatternResult<Self> {
        // Ingress validation
        PatternType::ParallelSplit
            .validate_ingress(branches.len() as u32)
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
    #[allow(dead_code)]
    use_simd: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Send + Sync> SynchronizationPattern<T> {
    pub fn new() -> Self {
        Self {
            use_simd: false,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_simd() -> Self {
        Self {
            use_simd: true,
            _phantom: std::marker::PhantomData,
        }
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
        PatternType::ExclusiveChoice
            .validate_ingress(choices.len() as u32)
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
    #[allow(dead_code)]
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
        PatternType::MultiChoice
            .validate_ingress(choices.len() as u32)
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
        PatternType::ArbitraryCycles
            .validate_ingress(1)
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
        PatternType::DeferredChoice
            .validate_ingress(choices.len() as u32)
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

// ============================================================================
// Pattern 9: Discriminator (First-Wins / Race Condition)
// ============================================================================

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc as StdArc;

pub struct DiscriminatorPattern<T> {
    branches: Vec<BranchFn<T>>,
    #[allow(dead_code)]
    use_simd: bool,
}

impl<T: Clone + Send + Sync + 'static> DiscriminatorPattern<T> {
    pub fn new(branches: Vec<BranchFn<T>>) -> PatternResult<Self> {
        Self::with_simd(branches, false)
    }

    pub fn with_simd(branches: Vec<BranchFn<T>>, use_simd: bool) -> PatternResult<Self> {
        // Ingress validation
        PatternType::Discriminator
            .validate_ingress(branches.len() as u32)
            .map_err(PatternError::ValidationFailed)?;

        if branches.is_empty() {
            return Err(PatternError::InvalidConfiguration(
                "Discriminator requires at least one branch".to_string(),
            ));
        }

        Ok(Self { branches, use_simd })
    }
}

impl<T: Clone + Send + Sync + 'static> Pattern<T> for DiscriminatorPattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::Discriminator
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        use crossbeam_channel::bounded;

        // Create channel for first result
        let (tx, rx) = bounded(1);
        let won = StdArc::new(AtomicBool::new(false));

        // Execute all branches in parallel
        self.branches.par_iter().for_each(|branch| {
            let tx = tx.clone();
            let won = won.clone();
            let input = input.clone();

            // Execute branch
            if let Ok(result) = branch(input) {
                // Try to be first to send result
                if !won.swap(true, Ordering::SeqCst) {
                    let _ = tx.send(result); // First one wins, others ignored
                }
            }
        });

        // Wait for first result
        drop(tx); // Drop original sender
        match rx.recv() {
            Ok(result) => Ok(vec![result]),
            Err(_) => Err(PatternError::ExecutionFailed(
                "All branches failed in discriminator".to_string(),
            )),
        }
    }
}

// ============================================================================
// Pattern 11: Implicit Termination (Workflow Completion Detection)
// ============================================================================

pub struct ImplicitTerminationPattern<T> {
    branches: Vec<BranchFn<T>>,
}

impl<T: Clone + Send + Sync> ImplicitTerminationPattern<T> {
    pub fn new(branches: Vec<BranchFn<T>>) -> PatternResult<Self> {
        // Ingress validation
        PatternType::ImplicitTermination
            .validate_ingress(branches.len() as u32)
            .map_err(PatternError::ValidationFailed)?;

        Ok(Self { branches })
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for ImplicitTerminationPattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::ImplicitTermination
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        // Track active branches
        let active_count = StdArc::new(AtomicUsize::new(self.branches.len()));
        let results = StdArc::new(std::sync::Mutex::new(Vec::new()));

        // Execute all branches in parallel
        self.branches.par_iter().for_each(|branch| {
            let input = input.clone();
            let active = active_count.clone();
            let results_lock = results.clone();

            // Execute branch
            if let Ok(result) = branch(input) {
                if let Ok(mut results) = results_lock.lock() {
                    results.push(result);
                }
            }

            // Decrement active count
            active.fetch_sub(1, Ordering::SeqCst);
        });

        // Wait for all branches to complete
        while active_count.load(Ordering::SeqCst) > 0 {
            std::thread::yield_now();
        }

        // Extract results
        let final_results = results
            .lock()
            .map_err(|_| PatternError::ExecutionFailed("Lock poisoned".to_string()))?
            .clone();

        if final_results.is_empty() {
            Err(PatternError::ExecutionFailed(
                "All branches failed in implicit termination".to_string(),
            ))
        } else {
            Ok(final_results)
        }
    }
}

// ============================================================================
// Pattern 20: Timeout (Production-Critical)
// ============================================================================

pub struct TimeoutPattern<T> {
    branch: BranchFn<T>,
    timeout_ms: u64,
    fallback: Option<BranchFn<T>>,
}

impl<T: Clone + Send + Sync + 'static> TimeoutPattern<T> {
    pub fn new(branch: BranchFn<T>, timeout_ms: u64) -> PatternResult<Self> {
        Self::with_fallback(branch, timeout_ms, None)
    }

    pub fn with_fallback(
        branch: BranchFn<T>,
        timeout_ms: u64,
        fallback: Option<BranchFn<T>>,
    ) -> PatternResult<Self> {
        // Ingress validation
        PatternType::Timeout
            .validate_ingress(1)
            .map_err(PatternError::ValidationFailed)?;

        if timeout_ms == 0 {
            return Err(PatternError::InvalidConfiguration(
                "Timeout must be > 0".to_string(),
            ));
        }

        Ok(Self {
            branch,
            timeout_ms,
            fallback,
        })
    }
}

impl<T: Clone + Send + Sync + 'static> Pattern<T> for TimeoutPattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::Timeout
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        use crossbeam_channel::{bounded, select};
        use std::time::Duration;

        let (tx, rx) = bounded(1);
        let branch = self.branch.clone();
        let input_clone = input.clone();

        // Execute branch in thread
        std::thread::spawn(move || {
            if let Ok(result) = branch(input_clone) {
                let _ = tx.send(Ok(result));
            } else {
                let _ = tx.send(Err(()));
            }
        });

        // Wait for result or timeout
        select! {
            recv(rx) -> result => {
                match result {
                    Ok(Ok(value)) => Ok(vec![value]),
                    Ok(Err(_)) | Err(_) => {
                        // Branch failed, try fallback
                        if let Some(fallback) = &self.fallback {
                            let result = fallback(input)?;
                            Ok(vec![result])
                        } else {
                            Err(PatternError::ExecutionFailed("Branch failed".to_string()))
                        }
                    }
                }
            }
            default(Duration::from_millis(self.timeout_ms)) => {
                // Timeout occurred, try fallback
                if let Some(fallback) = &self.fallback {
                    let result = fallback(input)?;
                    Ok(vec![result])
                } else {
                    Err(PatternError::ExecutionFailed(
                        format!("Timeout after {}ms", self.timeout_ms)
                    ))
                }
            }
        }
    }
}

// ============================================================================
// Pattern 21: Cancellation (Production-Critical)
// ============================================================================

pub struct CancellationPattern<T> {
    branch: BranchFn<T>,
    should_cancel: Arc<dyn Fn() -> bool + Send + Sync>,
}

impl<T: Clone + Send + Sync> CancellationPattern<T> {
    pub fn new(
        branch: BranchFn<T>,
        should_cancel: Arc<dyn Fn() -> bool + Send + Sync>,
    ) -> PatternResult<Self> {
        // Ingress validation
        PatternType::Cancellation
            .validate_ingress(1)
            .map_err(PatternError::ValidationFailed)?;

        Ok(Self {
            branch,
            should_cancel,
        })
    }
}

impl<T: Clone + Send + Sync> Pattern<T> for CancellationPattern<T> {
    fn pattern_type(&self) -> PatternType {
        PatternType::Cancellation
    }

    fn execute(&self, input: T) -> PatternResult<Vec<T>> {
        // Check for cancellation before execution
        if (self.should_cancel)() {
            return Err(PatternError::ExecutionFailed(
                "Operation cancelled before execution".to_string(),
            ));
        }

        // Execute branch
        let result = (self.branch)(input)?;

        // Check for cancellation after execution
        if (self.should_cancel)() {
            return Err(PatternError::ExecutionFailed(
                "Operation cancelled after execution".to_string(),
            ));
        }

        Ok(vec![result])
    }
}
