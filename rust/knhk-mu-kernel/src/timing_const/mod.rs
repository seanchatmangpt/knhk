//! Const-Eval Timing Analysis for Compile-Time Budgets
//!
//! This module provides compile-time timing analysis using const evaluation
//! and const generics. All timing constraints are verified at compile time,
//! providing zero-overhead runtime guarantees.
//!
//! # Core Concepts
//!
//! - **ConstTickCost**: Trait for compile-time tick cost calculation
//! - **TickBudget<N>**: Type-level budget tracking with const generics
//! - **WCET Analysis**: Worst-Case Execution Time computed at compile time
//! - **TimingProof**: Compile-time certificates of timing guarantees
//!
//! # Example
//!
//! ```ignore
//! // Compile-time budget that proves timing constraints
//! const fn analyze_task<P: ConstTickCost>() -> u64 {
//!     let budget = TickBudget::<8>::new();
//!     let cost = P::TICK_COST;
//!
//!     if cost > 8 {
//!         panic!("Task exceeds Chatman Constant");
//!     }
//!
//!     cost
//! }
//! ```

use core::marker::PhantomData;
use crate::patterns::PatternId;
use crate::CHATMAN_CONSTANT;

pub mod wcet;
pub mod budgets;
pub mod proofs;

pub use wcet::{WcetAnalyzer, WcetResult, WcetPhase};
pub use budgets::{ConstBudget, BudgetExhausted, BudgetOk};
pub use proofs::{TimingProof, TimingCertificate, ProofStrength};

/// Trait for compile-time tick cost calculation
///
/// Types implementing this trait can have their tick costs
/// computed at compile time, enabling compile-time budget verification.
pub trait ConstTickCost {
    /// The tick cost as a compile-time constant
    const TICK_COST: u64;

    /// Check if this operation is hot-path eligible (≤ Chatman Constant)
    const IS_HOT_PATH: bool = Self::TICK_COST <= CHATMAN_CONSTANT;

    /// Get a descriptive name for this operation
    const NAME: &'static str;
}

/// Sequence pattern - simplest workflow pattern
pub struct SequencePattern;

impl ConstTickCost for SequencePattern {
    const TICK_COST: u64 = 1;
    const NAME: &'static str = "Sequence";
}

/// Parallel split pattern - fork workflow
pub struct ParallelSplitPattern;

impl ConstTickCost for ParallelSplitPattern {
    const TICK_COST: u64 = 2;
    const NAME: &'static str = "ParallelSplit";
}

/// Synchronization pattern - join parallel flows
pub struct SynchronizationPattern;

impl ConstTickCost for SynchronizationPattern {
    const TICK_COST: u64 = 3;
    const NAME: &'static str = "Synchronization";
}

/// Exclusive choice pattern - conditional branch
pub struct ExclusiveChoicePattern;

impl ConstTickCost for ExclusiveChoicePattern {
    const TICK_COST: u64 = 1;
    const NAME: &'static str = "ExclusiveChoice";
}

/// Multi-choice pattern - multiple parallel branches
pub struct MultiChoicePattern;

impl ConstTickCost for MultiChoicePattern {
    const TICK_COST: u64 = 3;
    const NAME: &'static str = "MultiChoice";
}

/// Guard evaluation operation
pub struct GuardEvalOp;

impl ConstTickCost for GuardEvalOp {
    const TICK_COST: u64 = 1;
    const NAME: &'static str = "GuardEval";
}

/// Receipt write operation
pub struct ReceiptWriteOp;

impl ConstTickCost for ReceiptWriteOp {
    const TICK_COST: u64 = 1;
    const NAME: &'static str = "ReceiptWrite";
}

/// Σ* load operation
pub struct SigmaLoadOp;

impl ConstTickCost for SigmaLoadOp {
    const TICK_COST: u64 = 1;
    const NAME: &'static str = "SigmaLoad";
}

/// Pattern dispatch operation
pub struct PatternDispatchOp;

impl ConstTickCost for PatternDispatchOp {
    const TICK_COST: u64 = 1;
    const NAME: &'static str = "PatternDispatch";
}

/// Compute tick cost for a pattern at compile time
#[inline(always)]
pub const fn pattern_tick_cost(pattern_id: PatternId) -> u64 {
    pattern_id.tick_cost() as u64
}

/// Compute total tick cost for multiple operations at compile time
#[inline(always)]
pub const fn total_tick_cost<const N: usize>(costs: [u64; N]) -> u64 {
    let mut total = 0;
    let mut i = 0;
    while i < N {
        total += costs[i];
        i += 1;
    }
    total
}

/// Verify tick budget at compile time
///
/// Panics at compile time if budget exceeds limit
#[inline(always)]
pub const fn verify_tick_budget(used: u64, limit: u64) {
    if used > limit {
        panic!("Tick budget exceeded");
    }
}

/// Check if operation sequence fits within Chatman Constant
#[inline(always)]
pub const fn within_chatman<const N: usize>(costs: [u64; N]) -> bool {
    total_tick_cost(costs) <= CHATMAN_CONSTANT
}

/// Compose two tick costs
#[inline(always)]
pub const fn compose_costs(cost1: u64, cost2: u64) -> u64 {
    cost1 + cost2
}

/// Maximum of two tick costs
#[inline(always)]
pub const fn max_cost(cost1: u64, cost2: u64) -> u64 {
    if cost1 > cost2 { cost1 } else { cost2 }
}

/// Minimum of two tick costs
#[inline(always)]
pub const fn min_cost(cost1: u64, cost2: u64) -> u64 {
    if cost1 < cost2 { cost1 } else { cost2 }
}

/// Type-level representation of tick costs
///
/// Allows encoding tick costs in the type system for
/// compile-time verification.
pub struct TickCost<const N: u64>;

impl<const N: u64> TickCost<N> {
    /// Get the tick cost value
    pub const COST: u64 = N;

    /// Check if within Chatman Constant
    pub const WITHIN_CHATMAN: bool = N <= CHATMAN_CONSTANT;

    /// Create a new tick cost marker
    pub const fn new() -> Self {
        Self
    }
}

/// Marker trait for costs within Chatman Constant
pub trait WithinChatman {
    const COST: u64;
}

impl<const N: u64> WithinChatman for TickCost<N>
where
    [(); (N <= CHATMAN_CONSTANT) as usize]:,
{
    const COST: u64 = N;
}

/// Const function to compute WCET for a standard task
///
/// Standard task structure:
/// 1. Load Σ* descriptor (1 tick)
/// 2. Dispatch pattern (1 tick)
/// 3. Evaluate guards (N ticks, N ≤ 4)
/// 4. Execute pattern (varies by pattern)
/// 5. Write receipt (1 tick)
#[inline(always)]
pub const fn compute_task_wcet(pattern_cost: u64, guard_count: u64) -> u64 {
    let load_cost = 1;
    let dispatch_cost = 1;
    let guard_cost = guard_count;
    let receipt_cost = 1;

    load_cost + dispatch_cost + guard_cost + pattern_cost + receipt_cost
}

/// Verify task WCET at compile time
#[inline(always)]
pub const fn verify_task_wcet(pattern_cost: u64, guard_count: u64) {
    let wcet = compute_task_wcet(pattern_cost, guard_count);
    verify_tick_budget(wcet, CHATMAN_CONSTANT);
}

/// Operation cost table for all μ-ops
pub struct OpCostTable;

impl OpCostTable {
    /// Load Σ* descriptor
    pub const LOAD_SIGMA: u64 = 1;

    /// Dispatch pattern by ID
    pub const DISPATCH_PATTERN: u64 = 1;

    /// Evaluate guard
    pub const EVAL_GUARD: u64 = 1;

    /// Read observation field
    pub const READ_OBS: u64 = 1;

    /// Write receipt field
    pub const WRITE_RECEIPT: u64 = 1;

    /// Check budget
    pub const CHECK_BUDGET: u64 = 0; // Inline, zero cost
}

/// Const function to validate a tick budget array
#[inline(always)]
pub const fn validate_budget_array<const N: usize>(costs: [u64; N]) -> bool {
    let total = total_tick_cost(costs);
    total <= CHATMAN_CONSTANT
}

/// Type-level proof that a sequence of operations is within budget
pub struct BudgetProof<const TOTAL: u64> {
    _marker: PhantomData<[(); TOTAL as usize]>,
}

impl<const TOTAL: u64> BudgetProof<TOTAL> {
    /// Create a new budget proof (const)
    pub const fn new() -> Self
    where
        [(); (TOTAL <= CHATMAN_CONSTANT) as usize]:,
    {
        Self {
            _marker: PhantomData,
        }
    }

    /// Get the total cost
    pub const fn total() -> u64 {
        TOTAL
    }

    /// Get remaining budget
    pub const fn remaining() -> u64 {
        CHATMAN_CONSTANT - TOTAL
    }
}

/// Const assertion macro for tick budgets
#[macro_export]
macro_rules! const_assert_ticks {
    ($cost:expr) => {
        const _: () = {
            if $cost > $crate::CHATMAN_CONSTANT {
                panic!("Tick cost exceeds Chatman Constant");
            }
        };
    };
}

/// Const function to compute parallel execution cost
///
/// For parallel operations, the cost is the maximum of all branches,
/// not the sum.
#[inline(always)]
pub const fn parallel_cost<const N: usize>(branch_costs: [u64; N]) -> u64 {
    let mut max = 0;
    let mut i = 0;
    while i < N {
        if branch_costs[i] > max {
            max = branch_costs[i];
        }
        i += 1;
    }
    max
}

/// Const function to compute sequential execution cost
#[inline(always)]
pub const fn sequential_cost<const N: usize>(op_costs: [u64; N]) -> u64 {
    total_tick_cost(op_costs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_tick_cost_trait() {
        assert_eq!(SequencePattern::TICK_COST, 1);
        assert_eq!(ParallelSplitPattern::TICK_COST, 2);
        assert_eq!(SynchronizationPattern::TICK_COST, 3);

        assert!(SequencePattern::IS_HOT_PATH);
        assert!(ParallelSplitPattern::IS_HOT_PATH);
    }

    #[test]
    fn test_total_tick_cost() {
        const COSTS: [u64; 5] = [1, 2, 1, 3, 1];
        const TOTAL: u64 = total_tick_cost(COSTS);

        assert_eq!(TOTAL, 8);
    }

    #[test]
    fn test_within_chatman() {
        const VALID: bool = within_chatman([1, 2, 3, 2]);
        const INVALID: bool = within_chatman([5, 5, 5]);

        assert!(VALID);
        assert!(!INVALID);
    }

    #[test]
    fn test_compute_task_wcet() {
        // Sequence pattern with 2 guards
        const WCET1: u64 = compute_task_wcet(1, 2);
        assert_eq!(WCET1, 5); // load(1) + dispatch(1) + guards(2) + pattern(1) + receipt(1)

        // Parallel split with 3 guards
        const WCET2: u64 = compute_task_wcet(2, 3);
        assert_eq!(WCET2, 7); // load(1) + dispatch(1) + guards(3) + pattern(2) + receipt(1)
    }

    #[test]
    fn test_compose_costs() {
        const COMPOSED: u64 = compose_costs(3, 4);
        assert_eq!(COMPOSED, 7);
    }

    #[test]
    fn test_parallel_cost() {
        const BRANCHES: [u64; 4] = [2, 5, 3, 4];
        const MAX: u64 = parallel_cost(BRANCHES);

        assert_eq!(MAX, 5); // Maximum branch cost
    }

    #[test]
    fn test_sequential_cost() {
        const OPS: [u64; 4] = [1, 2, 3, 1];
        const TOTAL: u64 = sequential_cost(OPS);

        assert_eq!(TOTAL, 7); // Sum of all ops
    }

    #[test]
    fn test_tick_cost_type() {
        const COST1: TickCost<3> = TickCost::new();
        const COST2: TickCost<8> = TickCost::new();

        assert_eq!(TickCost::<3>::COST, 3);
        assert_eq!(TickCost::<8>::COST, 8);

        assert!(TickCost::<3>::WITHIN_CHATMAN);
        assert!(TickCost::<8>::WITHIN_CHATMAN);
    }

    #[test]
    fn test_op_cost_table() {
        assert_eq!(OpCostTable::LOAD_SIGMA, 1);
        assert_eq!(OpCostTable::DISPATCH_PATTERN, 1);
        assert_eq!(OpCostTable::EVAL_GUARD, 1);
        assert_eq!(OpCostTable::WRITE_RECEIPT, 1);
        assert_eq!(OpCostTable::CHECK_BUDGET, 0);
    }

    #[test]
    fn test_max_min_cost() {
        const MAX: u64 = max_cost(5, 3);
        const MIN: u64 = min_cost(5, 3);

        assert_eq!(MAX, 5);
        assert_eq!(MIN, 3);
    }
}
