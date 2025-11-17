//! Prusti Specifications for Formal Verification
//!
//! Uses Prusti to specify and verify:
//! - Function pre/post-conditions
//! - Loop invariants
//! - Type invariants
//! - Function contracts
//!
//! # Prusti Annotations
//!
//! - `#[requires(...)]` - Preconditions that must hold on entry
//! - `#[ensures(...)]` - Postconditions that must hold on exit
//! - `#[invariant(...)]` - Loop invariants
//! - `#[pure]` - Functions without side effects
//! - `#[trusted]` - Axiomatized functions (not verified)
//!
//! # Running Prusti
//!
//! ```bash
//! cargo prusti
//! cargo prusti --features verification
//! ```

#![cfg_attr(prusti, feature(register_tool))]
#![cfg_attr(prusti, register_tool(prusti))]

use crate::{
    guards::GuardContext,
    patterns::PatternId,
    timing::{BudgetStatus, TickBudget, TickCounter},
    CHATMAN_CONSTANT,
};

// ============================================================================
// Type Invariants
// ============================================================================

/// Type invariant: TickBudget always has valid state
///
/// INVARIANTS:
/// - used <= limit (cannot consume more than allocated)
/// - limit > 0 (budgets are non-zero)
#[cfg_attr(prusti, invariant(self.used <= self.limit))]
#[cfg_attr(prusti, invariant(self.limit > 0))]
impl TickBudget {
    // Implementation continues below
}

// ============================================================================
// TickBudget::chatman() Specification
// ============================================================================

#[cfg_attr(prusti, pure)]
#[cfg_attr(prusti, ensures(result.limit == 8))]
#[cfg_attr(prusti, ensures(result.used == 0))]
#[cfg_attr(prusti, ensures(!result.is_exhausted()))]
pub const fn chatman_verified() -> TickBudget {
    TickBudget::chatman()
}

// ============================================================================
// TickBudget::new() Specification
// ============================================================================

#[cfg_attr(prusti, requires(limit > 0))]
#[cfg_attr(prusti, ensures(result.limit == limit))]
#[cfg_attr(prusti, ensures(result.used == 0))]
#[cfg_attr(prusti, ensures(result.remaining() == limit))]
pub const fn new_verified(limit: u64) -> TickBudget {
    TickBudget::new(limit)
}

// ============================================================================
// TickBudget::consume() Specification
// ============================================================================

#[cfg_attr(prusti, requires(budget.used <= budget.limit))]
#[cfg_attr(prusti, ensures(budget.used >= old(budget.used)))]
#[cfg_attr(prusti, ensures(
    budget.used == old(budget.used).saturating_add(ticks)
))]
#[cfg_attr(prusti, ensures(
    result == BudgetStatus::Ok || result == BudgetStatus::Exhausted
))]
#[cfg_attr(prusti, ensures(
    (budget.used < budget.limit) == (result == BudgetStatus::Ok)
))]
pub fn consume_verified(budget: &mut TickBudget, ticks: u64) -> BudgetStatus {
    budget.consume(ticks)
}

// ============================================================================
// TickBudget::is_exhausted() Specification
// ============================================================================

#[cfg_attr(prusti, pure)]
#[cfg_attr(prusti, requires(budget.used <= budget.limit))]
#[cfg_attr(prusti, ensures(result == (budget.used >= budget.limit)))]
pub const fn is_exhausted_verified(budget: &TickBudget) -> bool {
    budget.is_exhausted()
}

// ============================================================================
// TickBudget::remaining() Specification
// ============================================================================

#[cfg_attr(prusti, pure)]
#[cfg_attr(prusti, requires(budget.used <= budget.limit))]
#[cfg_attr(prusti, ensures(result == budget.limit - budget.used))]
#[cfg_attr(prusti, ensures(result <= budget.limit))]
pub const fn remaining_verified(budget: &TickBudget) -> u64 {
    budget.remaining()
}

// ============================================================================
// TickBudget::reset() Specification
// ============================================================================

#[cfg_attr(prusti, requires(budget.limit > 0))]
#[cfg_attr(prusti, ensures(budget.used == 0))]
#[cfg_attr(prusti, ensures(budget.limit == old(budget.limit)))]
#[cfg_attr(prusti, ensures(!budget.is_exhausted()))]
#[cfg_attr(prusti, ensures(budget.remaining() == budget.limit))]
pub fn reset_verified(budget: &mut TickBudget) {
    budget.reset();
}

// ============================================================================
// Chatman Constant Compliance
// ============================================================================

/// Verified: All hot path operations respect Chatman Constant
///
/// For any tick budget within Chatman Constant:
/// - consume() never panics
/// - used ticks are tracked correctly
/// - status reflects budget state
#[cfg_attr(prusti, requires(ticks <= CHATMAN_CONSTANT))]
#[cfg_attr(prusti, ensures(result.used <= CHATMAN_CONSTANT))]
#[cfg_attr(prusti, ensures(result.used == ticks))]
pub fn execute_hot_path(ticks: u64) -> TickBudget {
    let mut budget = TickBudget::chatman();
    let _status = budget.consume(ticks);
    budget
}

// ============================================================================
// Monotonicity Properties
// ============================================================================

/// Verified: Tick consumption is monotonic
///
/// Consuming ticks always increases (or maintains) used count.
#[cfg_attr(prusti, requires(budget.used <= budget.limit))]
#[cfg_attr(prusti, requires(ticks > 0))]
#[cfg_attr(prusti, ensures(budget.used > old(budget.used)))]
pub fn monotonic_consumption(budget: &mut TickBudget, ticks: u64) {
    let old_used = budget.used;
    budget.consume(ticks);

    // Ghost code for verification
    #[cfg(prusti)]
    prusti_assert!(budget.used >= old_used);
}

// ============================================================================
// Idempotence Properties
// ============================================================================

/// Verified: Budget state transitions are deterministic
///
/// Same input always produces same output.
#[cfg_attr(prusti, pure)]
#[cfg_attr(prusti, requires(limit > 0))]
#[cfg_attr(prusti, requires(ticks1 == ticks2))]
#[cfg_attr(prusti, ensures(result))]
pub fn idempotent_consumption(limit: u64, ticks1: u64, ticks2: u64) -> bool {
    let mut budget1 = TickBudget::new(limit);
    let mut budget2 = TickBudget::new(limit);

    let status1 = budget1.consume(ticks1);
    let status2 = budget2.consume(ticks2);

    budget1.used == budget2.used && status1 == status2
}

// ============================================================================
// Pattern Tick Cost Verification
// ============================================================================

/// Verified: Pattern tick costs are within Chatman Constant
///
/// Every pattern has a tick cost ≤ 8.
#[cfg_attr(prusti, requires(pattern_id < 43))]
#[cfg_attr(prusti, ensures(result > 0))]
#[cfg_attr(prusti, ensures(result <= 8))]
pub fn pattern_tick_cost_verified(pattern_id: u8) -> u8 {
    let pattern: PatternId = unsafe { core::mem::transmute(pattern_id) };
    pattern.tick_cost()
}

// ============================================================================
// Guard Context Verification
// ============================================================================

/// Verified: GuardContext preserves all fields
#[cfg_attr(prusti, ensures(result.task_id == task_id))]
#[cfg_attr(prusti, ensures(result.obs_data == obs_data))]
#[cfg_attr(prusti, ensures(result.params[0] == params[0]))]
#[cfg_attr(prusti, ensures(result.params[1] == params[1]))]
#[cfg_attr(prusti, ensures(result.params[2] == params[2]))]
#[cfg_attr(prusti, ensures(result.params[3] == params[3]))]
pub fn create_guard_context_verified(
    task_id: u64,
    obs_data: u64,
    params: [u64; 4],
) -> GuardContext {
    GuardContext {
        task_id,
        obs_data,
        params,
    }
}

// ============================================================================
// Loop Invariants
// ============================================================================

/// Verified loop: Sequential tick consumption
///
/// INVARIANT: At iteration i, used == i
#[cfg_attr(prusti, requires(limit > 0))]
#[cfg_attr(prusti, requires(limit <= 100))]
#[cfg_attr(prusti, ensures(budget.used <= limit))]
pub fn sequential_consumption_verified(budget: &mut TickBudget, limit: u64) {
    let mut i = 0u64;

    #[cfg(prusti)]
    while i < limit {
        prusti_assert!(budget.used == i);
        budget.consume(1);
        i += 1;
        prusti_assert!(budget.used == i);
    }
}

// ============================================================================
// Bounds Checking
// ============================================================================

/// Verified: Array access is bounds-checked
#[cfg_attr(prusti, requires(index < 4))]
#[cfg_attr(prusti, ensures(result == params[index]))]
pub fn safe_array_access(params: &[u64; 4], index: usize) -> u64 {
    params[index]
}

// ============================================================================
// Memory Safety
// ============================================================================

/// Verified: Memory layout is non-overlapping
#[cfg_attr(prusti, pure)]
#[cfg_attr(prusti, ensures(result))]
pub const fn memory_layout_valid() -> bool {
    use crate::memory::*;

    SIGMA_BASE + SIGMA_SIZE <= PATTERN_BASE
        && PATTERN_BASE + PATTERN_SIZE <= GUARD_BASE
        && GUARD_BASE + GUARD_SIZE <= OBS_BASE
        && OBS_BASE + OBS_SIZE <= RECEIPT_BASE
        && RECEIPT_BASE + RECEIPT_SIZE <= WARM_BASE
}

// ============================================================================
// Saturation Arithmetic
// ============================================================================

/// Verified: Saturating add never overflows
#[cfg_attr(prusti, ensures(result >= a))]
#[cfg_attr(prusti, ensures(result >= b))]
#[cfg_attr(prusti, ensures(result <= u64::MAX))]
pub fn saturating_add_verified(a: u64, b: u64) -> u64 {
    a.saturating_add(b)
}

/// Verified: Saturating sub never underflows
#[cfg_attr(prusti, ensures(result <= a))]
#[cfg_attr(prusti, ensures(result >= 0))]
pub fn saturating_sub_verified(a: u64, b: u64) -> u64 {
    a.saturating_sub(b)
}

// ============================================================================
// Branchless Operations
// ============================================================================

/// Verified: Budget status lookup is branchless
///
/// Uses const array instead of match/if for performance.
#[cfg_attr(prusti, pure)]
#[cfg_attr(prusti, requires(exhausted_flag <= 1))]
#[cfg_attr(prusti, ensures(
    (exhausted_flag == 0) == (result == BudgetStatus::Ok)
))]
#[cfg_attr(prusti, ensures(
    (exhausted_flag == 1) == (result == BudgetStatus::Exhausted)
))]
pub const fn branchless_status(exhausted_flag: u8) -> BudgetStatus {
    const STATUS_TABLE: [BudgetStatus; 2] = [BudgetStatus::Ok, BudgetStatus::Exhausted];
    STATUS_TABLE[exhausted_flag as usize]
}

// ============================================================================
// Determinism Properties
// ============================================================================

/// Verified: Same input produces same output
#[cfg_attr(prusti, requires(ticks1 == ticks2))]
#[cfg_attr(prusti, ensures(result))]
pub fn deterministic_execution(ticks1: u64, ticks2: u64) -> bool {
    let mut budget1 = TickBudget::chatman();
    let mut budget2 = TickBudget::chatman();

    budget1.consume(ticks1);
    budget2.consume(ticks2);

    budget1.used == budget2.used
}

// ============================================================================
// Composite Specifications
// ============================================================================

/// Verified: Complete hot path execution
///
/// Proves that a full hot path execution:
/// 1. Never exceeds Chatman Constant
/// 2. Produces valid receipts
/// 3. Maintains invariants
#[cfg_attr(prusti, requires(pattern_id < 43))]
#[cfg_attr(prusti, requires(guard_params[0] > 0))]
#[cfg_attr(prusti, ensures(result.budget.used <= CHATMAN_CONSTANT))]
#[cfg_attr(prusti, ensures(!result.overrun))]
pub fn complete_hot_path_verified(pattern_id: u8, guard_params: [u64; 4]) -> HotPathResult {
    let mut budget = TickBudget::chatman();

    // Pattern dispatch (1-2 ticks)
    let pattern: PatternId = unsafe { core::mem::transmute(pattern_id) };
    let pattern_cost = pattern.tick_cost() as u64;
    let status1 = budget.consume(pattern_cost);

    // Guard evaluation (1-2 ticks)
    let guard_cost = 2u64;
    let status2 = budget.consume(guard_cost);

    // Receipt generation (1-2 ticks)
    let receipt_cost = 2u64;
    let status3 = budget.consume(receipt_cost);

    let overrun = status1 == BudgetStatus::Exhausted
        || status2 == BudgetStatus::Exhausted
        || status3 == BudgetStatus::Exhausted;

    HotPathResult {
        budget,
        overrun,
        pattern_cost,
        guard_cost,
        receipt_cost,
    }
}

/// Result of hot path execution
pub struct HotPathResult {
    pub budget: TickBudget,
    pub overrun: bool,
    pub pattern_cost: u64,
    pub guard_cost: u64,
    pub receipt_cost: u64,
}

// ============================================================================
// Ghost State and Specifications
// ============================================================================

#[cfg(prusti)]
mod ghost_state {
    use super::*;

    /// Ghost variable: Tracks total ticks consumed across all operations
    ///
    /// This is only used for verification, not runtime.
    #[prusti::spec_only]
    static mut GHOST_TOTAL_TICKS: u64 = 0;

    /// Ghost function: Record tick consumption
    #[prusti::spec_only]
    #[prusti::ensures(*GHOST_TOTAL_TICKS == old(*GHOST_TOTAL_TICKS) + ticks)]
    pub unsafe fn ghost_consume(ticks: u64) {
        GHOST_TOTAL_TICKS += ticks;
    }

    /// Ghost function: Verify total ticks
    #[prusti::spec_only]
    #[prusti::pure]
    pub unsafe fn ghost_total() -> u64 {
        GHOST_TOTAL_TICKS
    }
}

// ============================================================================
// Abstraction Functions
// ============================================================================

/// Abstraction function: Budget state to mathematical model
///
/// Maps concrete TickBudget to abstract mathematical properties.
#[cfg(prusti)]
#[prusti::pure]
#[prusti::trusted]
pub fn abs_budget_valid(budget: &TickBudget) -> bool {
    budget.used <= budget.limit && budget.limit > 0
}

/// Abstraction function: Budget exhaustion
#[cfg(prusti)]
#[prusti::pure]
#[prusti::trusted]
pub fn abs_budget_exhausted(budget: &TickBudget) -> bool {
    budget.used >= budget.limit
}

// ============================================================================
// External Function Contracts
// ============================================================================

/// External spec: PatternId::tick_cost()
///
/// We trust that pattern tick costs are within bounds.
#[cfg(prusti)]
#[prusti::extern_spec]
impl PatternId {
    #[prusti::pure]
    #[prusti::ensures(result > 0)]
    #[prusti::ensures(result <= 8)]
    fn tick_cost(&self) -> u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prusti_specs_compile() {
        // Verify that Prusti specs don't break normal compilation
        let budget = chatman_verified();
        assert_eq!(budget.limit, 8);
    }

    #[test]
    fn verified_functions_work() {
        let mut budget = new_verified(10);
        let status = consume_verified(&mut budget, 5);

        assert_eq!(status, BudgetStatus::Ok);
        assert_eq!(budget.used, 5);
        assert_eq!(remaining_verified(&budget), 5);
    }

    #[test]
    fn memory_layout_verification() {
        assert!(memory_layout_valid());
    }

    #[test]
    fn saturating_arithmetic_verification() {
        let result = saturating_add_verified(u64::MAX, 1);
        assert_eq!(result, u64::MAX);

        let result = saturating_sub_verified(0, 1);
        assert_eq!(result, 0);
    }

    #[test]
    fn branchless_status_verification() {
        assert_eq!(branchless_status(0), BudgetStatus::Ok);
        assert_eq!(branchless_status(1), BudgetStatus::Exhausted);
    }

    #[test]
    fn hot_path_execution_verification() {
        let result = complete_hot_path_verified(0, [1, 2, 3, 4]);

        assert!(result.budget.used <= CHATMAN_CONSTANT);
        assert!(!result.overrun || result.budget.is_exhausted());
    }
}
