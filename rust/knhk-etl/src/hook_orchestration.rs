// rust/knhk-etl/src/hook_orchestration.rs
// Hook orchestration: Pattern-based hook execution within Reflex stage

extern crate alloc;

use alloc::format;
use alloc::vec::Vec;

use crate::error::PipelineError;
use crate::hook_registry::HookRegistry;
use crate::load::{LoadResult, PredRun, SoAArrays};
use crate::reflex::{Action, Receipt, ReflexStage};

/// Hook choice type: condition function and predicate ID
/// Factored out to reduce type complexity for clippy compliance
pub type HookChoice = (Box<dyn Fn(&HookExecutionContext) -> bool + Send + Sync>, u64);

/// Hook execution context: Contains all data needed for hook execution
pub struct HookExecutionContext {
    /// Hook registry mapping predicates to kernels
    pub hook_registry: HookRegistry,
    /// Predicate runs to execute
    pub predicate_runs: Vec<PredRun>,
    /// SoA arrays containing triple data
    pub soa_arrays: SoAArrays,
    /// Tick budget per hook (must be ≤8)
    pub tick_budget: u32,
}

impl HookExecutionContext {
    /// Create new hook execution context
    pub fn new(
        hook_registry: HookRegistry,
        predicate_runs: Vec<PredRun>,
        soa_arrays: SoAArrays,
        tick_budget: u32,
    ) -> Self {
        Self {
            hook_registry,
            predicate_runs,
            soa_arrays,
            tick_budget,
        }
    }

    /// Create from LoadResult
    pub fn from_load_result(
        hook_registry: HookRegistry,
        load_result: LoadResult,
        tick_budget: u32,
    ) -> Self {
        Self {
            hook_registry,
            predicate_runs: load_result.runs,
            soa_arrays: load_result.soa_arrays,
            tick_budget,
        }
    }
}

/// Hook execution pattern: Defines how hooks should be executed
pub enum HookExecutionPattern {
    /// Execute hooks sequentially
    Sequence(Vec<u64>),
    /// Execute hooks in parallel
    Parallel(Vec<u64>),
    /// Conditional routing: (condition, predicate_id)
    /// Condition evaluates on execution context
    Choice(Vec<HookChoice>),
    /// Retry logic: predicate, condition, max_attempts
    /// Condition evaluates on receipt to determine if retry needed
    Retry {
        predicate: u64,
        should_retry: Box<dyn Fn(&Receipt) -> bool + Send + Sync>,
        max_attempts: u32,
    },
}

/// Hook execution result: Aggregated results from pattern execution
pub struct HookExecutionResult {
    /// Receipts from hook execution
    pub receipts: Vec<Receipt>,
    /// Maximum ticks across all hooks
    pub max_ticks: u32,
    /// Actions generated from hooks
    pub actions: Vec<Action>,
    /// Aggregated receipt (merged via ⊕)
    pub aggregated_receipt: Receipt,
}

impl HookExecutionResult {
    /// Create new empty result
    pub fn new() -> Self {
        Self {
            receipts: Vec::new(),
            max_ticks: 0,
            actions: Vec::new(),
            aggregated_receipt: Receipt {
                id: "empty_receipt".to_string(),
                cycle_id: 0,
                shard_id: 0,
                hook_id: 0,
                ticks: 0,
                actual_ticks: 0,
                lanes: 0,
                span_id: 0,
                a_hash: 0,
            },
        }
    }

    /// Merge receipts using ⊕ operator
    pub fn merge_receipts(receipts: &[Receipt]) -> Receipt {
        ReflexStage::merge_receipts(receipts)
    }
}

impl Default for HookExecutionResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook orchestrator: Manages pattern-based hook execution
pub struct HookOrchestrator {
    /// Reflex stage for hook execution
    reflex_stage: ReflexStage,
}

impl HookOrchestrator {
    /// Create new hook orchestrator
    pub fn new() -> Self {
        Self {
            reflex_stage: ReflexStage::new(),
        }
    }

    /// Execute hooks with pattern
    pub fn execute_with_pattern(
        &self,
        context: &HookExecutionContext,
        pattern: HookExecutionPattern,
    ) -> Result<HookExecutionResult, PipelineError> {
        match pattern {
            HookExecutionPattern::Sequence(predicates) => {
                self.execute_sequence(context, &predicates)
            }
            HookExecutionPattern::Parallel(predicates) => {
                self.execute_parallel(context, &predicates)
            }
            HookExecutionPattern::Choice(choices) => {
                self.execute_choice(context, &choices)
            }
            HookExecutionPattern::Retry {
                predicate,
                should_retry,
                max_attempts,
            } => self.execute_retry(context, predicate, &should_retry, max_attempts),
        }
    }

    /// Execute hooks sequentially
    fn execute_sequence(
        &self,
        context: &HookExecutionContext,
        predicates: &[u64],
    ) -> Result<HookExecutionResult, PipelineError> {
        let mut receipts = Vec::new();
        let mut max_ticks = 0u32;
        let mut actions = Vec::new();

        // Find runs for each predicate and execute in order
        for predicate in predicates {
            if let Some(run) = context
                .predicate_runs
                .iter()
                .find(|r| r.pred == *predicate)
            {
                // Validate tick budget
                if run.len > context.tick_budget as u64 {
                    return Err(PipelineError::ReflexError(format!(
                        "Run length {} exceeds tick budget {}",
                        run.len, context.tick_budget
                    )));
                }

                let receipt = self.reflex_stage.execute_hook(&context.soa_arrays, run)?;

                // Validate receipt ticks
                if receipt.ticks > context.tick_budget {
                    return Err(PipelineError::ReflexError(format!(
                        "Hook execution {} ticks exceeds budget {} ticks",
                        receipt.ticks, context.tick_budget
                    )));
                }

                max_ticks = max_ticks.max(receipt.ticks);
                receipts.push(receipt.clone());

                // Generate action if hook succeeded
                if receipt.ticks > 0 {
                    actions.push(Action {
                        id: format!("action_{}", receipt.id),
                        payload: Vec::new(),
                        receipt_id: receipt.id.clone(),
                    });
                }
            }
        }

        let aggregated_receipt = HookExecutionResult::merge_receipts(&receipts);

        Ok(HookExecutionResult {
            receipts,
            max_ticks,
            actions,
            aggregated_receipt,
        })
    }

    /// Execute hooks in parallel
    fn execute_parallel(
        &self,
        context: &HookExecutionContext,
        predicates: &[u64],
    ) -> Result<HookExecutionResult, PipelineError> {
        // Find runs for predicates
        let runs: Vec<&PredRun> = predicates
            .iter()
            .filter_map(|pred| {
                context
                    .predicate_runs
                    .iter()
                    .find(|r| r.pred == *pred)
            })
            .collect();

        #[cfg(feature = "parallel")]
        {
            // Note: Cannot use parallel iteration here because ReflexStage contains RefCell<SloMonitor>
            // which is not Sync. RefCell is not thread-safe, so we use sequential iteration.
            // Execute hooks sequentially (RefCell is not Sync, so parallel iteration is not safe)
            let results: Result<Vec<_>, _> = runs
                .iter()
                .map(|run| {
                    // Validate tick budget
                    if run.len > context.tick_budget as u64 {
                        return Err(PipelineError::ReflexError(format!(
                            "Run length {} exceeds tick budget {}",
                            run.len, context.tick_budget
                        )));
                    }

                    let receipt = self.reflex_stage.execute_hook(&context.soa_arrays, run)?;

                    // Validate receipt ticks
                    if receipt.ticks > context.tick_budget {
                        return Err(PipelineError::ReflexError(format!(
                            "Hook execution {} ticks exceeds budget {} ticks",
                            receipt.ticks, context.tick_budget
                        )));
                    }

                    Ok(receipt)
                })
                .collect();

            let receipts = results?;
            let max_ticks = receipts.iter().map(|r| r.ticks).max().unwrap_or(0);

            // Generate actions for successful hooks
            let actions: Vec<Action> = receipts
                .iter()
                .filter(|r| r.ticks > 0)
                .map(|receipt| Action {
                    id: format!("action_{}", receipt.id),
                    payload: Vec::new(),
                    receipt_id: receipt.id.clone(),
                })
                .collect();

            let aggregated_receipt = HookExecutionResult::merge_receipts(&receipts);

            Ok(HookExecutionResult {
                receipts,
                max_ticks,
                actions,
                aggregated_receipt,
            })
        }

        #[cfg(not(feature = "parallel"))]
        {
            // Fallback to sequential execution when parallel feature is disabled
            let mut receipts = Vec::new();
            let mut max_ticks = 0u32;

            for run in runs {
                // Validate tick budget
                if run.len > context.tick_budget as u64 {
                    return Err(PipelineError::ReflexError(format!(
                        "Run length {} exceeds tick budget {}",
                        run.len, context.tick_budget
                    )));
                }

                let receipt = self.reflex_stage.execute_hook(&context.soa_arrays, run)?;

                // Validate receipt ticks
                if receipt.ticks > context.tick_budget {
                    return Err(PipelineError::ReflexError(format!(
                        "Hook execution {} ticks exceeds budget {} ticks",
                        receipt.ticks, context.tick_budget
                    )));
                }

                max_ticks = max_ticks.max(receipt.ticks);
                receipts.push(receipt);
            }

            // Generate actions for successful hooks
            let actions: Vec<Action> = receipts
                .iter()
                .filter(|r| r.ticks > 0)
                .map(|receipt| Action {
                    id: format!("action_{}", receipt.id),
                    payload: Vec::new(),
                    receipt_id: receipt.id.clone(),
                })
                .collect();

            let aggregated_receipt = HookExecutionResult::merge_receipts(&receipts);

            Ok(HookExecutionResult {
                receipts,
                max_ticks,
                actions,
                aggregated_receipt,
            })
        }
    }

    /// Execute hooks conditionally
    fn execute_choice(
        &self,
        context: &HookExecutionContext,
        choices: &[HookChoice],
    ) -> Result<HookExecutionResult, PipelineError> {
        if choices.is_empty() {
            return Err(PipelineError::ReflexError(
                "No choices provided for conditional execution".to_string(),
            ));
        }

        // Evaluate conditions in order
        for (condition, predicate) in choices {
            if condition(context) {
                // Execute matching hook
                if let Some(run) = context
                    .predicate_runs
                    .iter()
                    .find(|r| r.pred == *predicate)
                {
                    // Validate tick budget
                    if run.len > context.tick_budget as u64 {
                        return Err(PipelineError::ReflexError(format!(
                            "Run length {} exceeds tick budget {}",
                            run.len, context.tick_budget
                        )));
                    }

                    let receipt = self.reflex_stage.execute_hook(&context.soa_arrays, run)?;

                    // Validate receipt ticks
                    if receipt.ticks > context.tick_budget {
                        return Err(PipelineError::ReflexError(format!(
                            "Hook execution {} ticks exceeds budget {} ticks",
                            receipt.ticks, context.tick_budget
                        )));
                    }

                    let actions = if receipt.ticks > 0 {
                        vec![Action {
                            id: format!("action_{}", receipt.id),
                            payload: Vec::new(),
                            receipt_id: receipt.id.clone(),
                        }]
                    } else {
                        Vec::new()
                    };

                    return Ok(HookExecutionResult {
                        receipts: vec![receipt.clone()],
                        max_ticks: receipt.ticks,
                        actions,
                        aggregated_receipt: receipt,
                    });
                }
            }
        }

        // No condition matched
        Err(PipelineError::ReflexError(
            "No condition matched in choice pattern".to_string(),
        ))
    }

    /// Execute hook with retry logic
    fn execute_retry(
        &self,
        context: &HookExecutionContext,
        predicate: u64,
        should_retry: &dyn Fn(&Receipt) -> bool,
        max_attempts: u32,
    ) -> Result<HookExecutionResult, PipelineError> {
        if let Some(run) = context
            .predicate_runs
            .iter()
            .find(|r| r.pred == predicate)
        {
            let mut attempt = 0;
            let mut last_receipt = None;

            while attempt < max_attempts {
                let receipt = self.reflex_stage.execute_hook(&context.soa_arrays, run)?;

                // Check if we should retry
                if !should_retry(&receipt) {
                    // Success, return result
                    let actions = if receipt.ticks > 0 {
                        vec![Action {
                            id: format!("action_{}", receipt.id),
                            payload: Vec::new(),
                            receipt_id: receipt.id.clone(),
                        }]
                    } else {
                        Vec::new()
                    };

                    return Ok(HookExecutionResult {
                        receipts: vec![receipt.clone()],
                        max_ticks: receipt.ticks,
                        actions,
                        aggregated_receipt: receipt,
                    });
                }

                last_receipt = Some(receipt);
                attempt += 1;

                // Exponential backoff (simplified: just yield if std available)
                #[cfg(feature = "std")]
                {
                    std::thread::yield_now();
                }
            }

            // Max attempts reached
            if let Some(receipt) = last_receipt {
                Ok(HookExecutionResult {
                    receipts: vec![receipt.clone()],
                    max_ticks: receipt.ticks,
                    actions: Vec::new(),
                    aggregated_receipt: receipt,
                })
            } else {
                Err(PipelineError::ReflexError(
                    "Retry failed: no receipt generated".to_string(),
                ))
            }
        } else {
            Err(PipelineError::ReflexError(format!(
                "Predicate {} not found in runs",
                predicate
            )))
        }
    }
}

impl Default for HookOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

