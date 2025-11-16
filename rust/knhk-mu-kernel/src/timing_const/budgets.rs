//! Const Generic Budgets with Type-Level Tracking
//!
//! This module provides compile-time budget tracking using const generics.
//! Budgets are tracked in the type system, making budget violations a
//! compile-time error.

use core::marker::PhantomData;
use crate::CHATMAN_CONSTANT;

/// Budget status marker - OK
pub struct BudgetOk;

/// Budget status marker - Exhausted
pub struct BudgetExhausted;

/// Const generic budget with type-level tracking
///
/// The budget is tracked through the type system using const generics.
/// INITIAL is the starting budget, USED is how much has been consumed.
///
/// # Example
///
/// ```ignore
/// let budget = ConstBudget::<8, 0>::new();
/// let budget = budget.spend::<3>(); // Now ConstBudget<8, 3>
/// let budget = budget.spend::<2>(); // Now ConstBudget<8, 5>
/// // budget.spend::<5>() // Would fail to compile (5 + 5 > 8)
/// ```
pub struct ConstBudget<const INITIAL: u64, const USED: u64 = 0> {
    _marker: PhantomData<[(); {
        // Compile-time assertions
        assert!(USED <= INITIAL, "Budget exceeded");
        assert!(INITIAL <= CHATMAN_CONSTANT, "Initial budget too high");
        0
    }]>,
}

impl<const I: u64, const U: u64> ConstBudget<I, U> {
    /// Create a new budget
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }

    /// Spend ticks from the budget
    ///
    /// Returns a new budget with updated USED count.
    /// Fails to compile if spending would exceed budget.
    pub const fn spend<const COST: u64>(self) -> ConstBudget<I, { U + COST }>
    where
        [(); (U + COST <= I) as usize]:,
    {
        ConstBudget { _marker: PhantomData }
    }

    /// Get remaining budget
    pub const fn remaining() -> u64 {
        I - U
    }

    /// Get used budget
    pub const fn used() -> u64 {
        U
    }

    /// Get initial budget
    pub const fn initial() -> u64 {
        I
    }

    /// Check if budget is exhausted
    pub const fn is_exhausted() -> bool {
        U >= I
    }

    /// Get budget utilization (0.0 to 1.0, scaled to u64)
    pub const fn utilization_percent() -> u64 {
        if I == 0 {
            100
        } else {
            (U * 100) / I
        }
    }
}

/// Chatman-constrained budget (≤8 ticks)
pub type ChatmanBudget<const USED: u64 = 0> = ConstBudget<CHATMAN_CONSTANT, USED>;

impl<const U: u64> ChatmanBudget<U> {
    /// Create a Chatman budget
    pub const fn chatman() -> Self {
        Self::new()
    }
}

/// Budget deduction result
pub struct BudgetDeduction<const REMAINING: u64> {
    _marker: PhantomData<[(); REMAINING as usize]>,
}

impl<const R: u64> BudgetDeduction<R> {
    /// Get remaining budget
    pub const fn remaining() -> u64 {
        R
    }
}

/// Const function to compute budget after deduction
pub const fn deduct_budget(budget: u64, cost: u64) -> u64 {
    if cost > budget {
        panic!("Budget deduction would go negative");
    }
    budget - cost
}

/// Const function to check if cost fits in budget
pub const fn fits_in_budget(cost: u64, budget: u64) -> bool {
    cost <= budget
}

/// Const function to merge budgets (take minimum)
pub const fn merge_budgets(budget1: u64, budget2: u64) -> u64 {
    if budget1 < budget2 { budget1 } else { budget2 }
}

/// Type-level budget state
pub trait BudgetState {
    const REMAINING: u64;
    const IS_EXHAUSTED: bool;
}

impl<const I: u64, const U: u64> BudgetState for ConstBudget<I, U> {
    const REMAINING: u64 = I - U;
    const IS_EXHAUSTED: bool = U >= I;
}

/// Budget split result for parallel operations
pub struct BudgetSplit<const BUDGET1: u64, const BUDGET2: u64> {
    _marker: PhantomData<(
        [(); BUDGET1 as usize],
        [(); BUDGET2 as usize],
    )>,
}

impl<const B1: u64, const B2: u64> BudgetSplit<B1, B2> {
    /// Create a budget split
    pub const fn new() -> Self
    where
        [(); (B1 <= CHATMAN_CONSTANT) as usize]:,
        [(); (B2 <= CHATMAN_CONSTANT) as usize]:,
    {
        Self { _marker: PhantomData }
    }

    /// Get first budget
    pub const fn budget1() -> u64 {
        B1
    }

    /// Get second budget
    pub const fn budget2() -> u64 {
        B2
    }
}

/// Budget composition for sequential operations
pub struct BudgetComposition<const TOTAL: u64> {
    _marker: PhantomData<[(); TOTAL as usize]>,
}

impl<const T: u64> BudgetComposition<T> {
    /// Create a budget composition
    pub const fn new() -> Self
    where
        [(); (T <= CHATMAN_CONSTANT) as usize]:,
    {
        Self { _marker: PhantomData }
    }

    /// Get total budget
    pub const fn total() -> u64 {
        T
    }

    /// Check if within Chatman
    pub const fn within_chatman() -> bool {
        T <= CHATMAN_CONSTANT
    }
}

/// Const function to compute sequential budget composition
pub const fn compose_budgets<const N: usize>(costs: [u64; N]) -> u64 {
    let mut total = 0;
    let mut i = 0;
    while i < N {
        total += costs[i];
        i += 1;
    }
    total
}

/// Const function to compute parallel budget composition
pub const fn parallel_budgets<const N: usize>(costs: [u64; N]) -> u64 {
    let mut max = 0;
    let mut i = 0;
    while i < N {
        if costs[i] > max {
            max = costs[i];
        }
        i += 1;
    }
    max
}

/// Budget reservation for critical sections
pub struct BudgetReservation<const RESERVED: u64> {
    _marker: PhantomData<[(); RESERVED as usize]>,
}

impl<const R: u64> BudgetReservation<R> {
    /// Create a budget reservation
    pub const fn new() -> Self
    where
        [(); (R <= CHATMAN_CONSTANT) as usize]:,
    {
        Self { _marker: PhantomData }
    }

    /// Get reserved amount
    pub const fn reserved() -> u64 {
        R
    }

    /// Release reservation
    pub const fn release(self) -> u64 {
        R
    }
}

/// Const function to validate budget sequence
pub const fn validate_budget_sequence<const N: usize>(
    costs: [u64; N],
    total_budget: u64,
) -> bool {
    let total_cost = compose_budgets(costs);
    total_cost <= total_budget
}

/// Budget allocation table
pub struct BudgetAllocation<const PHASES: usize> {
    /// Budget per phase
    pub allocations: [u64; PHASES],
    /// Total allocated
    pub total: u64,
}

impl<const PHASES: usize> BudgetAllocation<PHASES> {
    /// Create a new budget allocation
    pub const fn new(allocations: [u64; PHASES]) -> Self {
        let total = compose_budgets(allocations);

        Self {
            allocations,
            total,
        }
    }

    /// Check if allocation is valid (within Chatman)
    pub const fn is_valid(&self) -> bool {
        self.total <= CHATMAN_CONSTANT
    }

    /// Get allocation for phase
    pub const fn get_allocation(&self, phase: usize) -> Option<u64> {
        if phase < PHASES {
            Some(self.allocations[phase])
        } else {
            None
        }
    }
}

/// Macro to create a compile-time budget assertion
#[macro_export]
macro_rules! assert_budget {
    ($used:expr, $limit:expr) => {
        const _: () = {
            if $used > $limit {
                panic!("Budget exceeded");
            }
        };
    };
}

/// Const function to compute budget overhead
pub const fn compute_overhead(base_cost: u64, actual_cost: u64) -> u64 {
    if actual_cost > base_cost {
        actual_cost - base_cost
    } else {
        0
    }
}

/// Const function to compute budget slack
pub const fn compute_slack(budget: u64, used: u64) -> u64 {
    if budget > used {
        budget - used
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_budget_creation() {
        let _budget = ConstBudget::<8, 0>::new();
        let _chatman = ChatmanBudget::<0>::chatman();

        assert_eq!(ConstBudget::<8, 0>::initial(), 8);
        assert_eq!(ConstBudget::<8, 0>::used(), 0);
        assert_eq!(ConstBudget::<8, 0>::remaining(), 8);
    }

    #[test]
    fn test_budget_spending() {
        let budget = ConstBudget::<8, 0>::new();
        let budget = budget.spend::<3>();
        let budget = budget.spend::<2>();

        assert_eq!(ConstBudget::<8, 5>::used(), 5);
        assert_eq!(ConstBudget::<8, 5>::remaining(), 3);

        // This would fail to compile:
        // let _budget = budget.spend::<5>(); // 5 + 5 > 8
    }

    #[test]
    fn test_budget_exhaustion() {
        assert!(!ConstBudget::<8, 0>::is_exhausted());
        assert!(!ConstBudget::<8, 7>::is_exhausted());
        assert!(ConstBudget::<8, 8>::is_exhausted());
    }

    #[test]
    fn test_budget_utilization() {
        assert_eq!(ConstBudget::<8, 0>::utilization_percent(), 0);
        assert_eq!(ConstBudget::<8, 4>::utilization_percent(), 50);
        assert_eq!(ConstBudget::<8, 8>::utilization_percent(), 100);
    }

    #[test]
    fn test_deduct_budget() {
        const REMAINING: u64 = deduct_budget(8, 3);
        assert_eq!(REMAINING, 5);
    }

    #[test]
    fn test_fits_in_budget() {
        assert!(fits_in_budget(5, 8));
        assert!(fits_in_budget(8, 8));
        assert!(!fits_in_budget(9, 8));
    }

    #[test]
    fn test_merge_budgets() {
        const MERGED: u64 = merge_budgets(5, 8);
        assert_eq!(MERGED, 5); // Minimum
    }

    #[test]
    fn test_compose_budgets() {
        const COSTS: [u64; 4] = [1, 2, 3, 2];
        const TOTAL: u64 = compose_budgets(COSTS);

        assert_eq!(TOTAL, 8);
    }

    #[test]
    fn test_parallel_budgets() {
        const COSTS: [u64; 4] = [2, 5, 3, 4];
        const MAX: u64 = parallel_budgets(COSTS);

        assert_eq!(MAX, 5); // Maximum
    }

    #[test]
    fn test_budget_allocation() {
        const ALLOC: BudgetAllocation<4> = BudgetAllocation::new([1, 2, 3, 2]);

        assert_eq!(ALLOC.total, 8);
        assert!(ALLOC.is_valid());
        assert_eq!(ALLOC.get_allocation(0).unwrap(), 1);
        assert_eq!(ALLOC.get_allocation(2).unwrap(), 3);
    }

    #[test]
    fn test_validate_budget_sequence() {
        const COSTS: [u64; 4] = [1, 2, 3, 1];
        const VALID: bool = validate_budget_sequence(COSTS, 8);

        assert!(VALID);

        const INVALID_COSTS: [u64; 3] = [5, 5, 5];
        const INVALID: bool = validate_budget_sequence(INVALID_COSTS, 8);

        assert!(!INVALID);
    }

    #[test]
    fn test_budget_overhead() {
        const OVERHEAD: u64 = compute_overhead(5, 7);
        assert_eq!(OVERHEAD, 2);

        const NO_OVERHEAD: u64 = compute_overhead(5, 5);
        assert_eq!(NO_OVERHEAD, 0);
    }

    #[test]
    fn test_budget_slack() {
        const SLACK: u64 = compute_slack(8, 5);
        assert_eq!(SLACK, 3);

        const NO_SLACK: u64 = compute_slack(5, 8);
        assert_eq!(NO_SLACK, 0);
    }

    #[test]
    fn test_chatman_budget() {
        let budget = ChatmanBudget::<0>::chatman();
        let budget = budget.spend::<3>();
        let budget = budget.spend::<2>();

        assert_eq!(ChatmanBudget::<5>::remaining(), 3);

        // This would fail to compile (exceeds Chatman Constant):
        // let _budget = ChatmanBudget::<9>::new();
    }
}
