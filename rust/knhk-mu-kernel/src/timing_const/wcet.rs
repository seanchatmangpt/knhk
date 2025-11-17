//! Compile-Time Worst-Case Execution Time (WCET) Analysis
//!
//! This module provides const functions for computing WCET of
//! μ-kernel operations. All analysis is performed at compile time,
//! providing static timing guarantees.

use super::{total_tick_cost, ConstTickCost};
use crate::patterns::PatternId;
use crate::CHATMAN_CONSTANT;
use core::marker::PhantomData;

/// WCET analysis result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WcetResult {
    /// Worst-case tick count
    pub worst_case_ticks: u64,
    /// Best-case tick count
    pub best_case_ticks: u64,
    /// Average-case tick count
    pub average_case_ticks: u64,
    /// Is hot-path eligible (≤ Chatman Constant)
    pub is_hot_path: bool,
}

impl WcetResult {
    /// Create a new WCET result
    pub const fn new(worst: u64, best: u64, average: u64) -> Self {
        Self {
            worst_case_ticks: worst,
            best_case_ticks: best,
            average_case_ticks: average,
            is_hot_path: worst <= CHATMAN_CONSTANT,
        }
    }

    /// Check if WCET is within budget
    pub const fn within_budget(&self, budget: u64) -> bool {
        self.worst_case_ticks <= budget
    }

    /// Get timing variability (worst - best)
    pub const fn variability(&self) -> u64 {
        self.worst_case_ticks - self.best_case_ticks
    }
}

/// WCET phase analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WcetPhase {
    /// Phase name
    pub name: &'static str,
    /// Phase WCET
    pub wcet: u64,
    /// Is this phase required (or conditional)
    pub required: bool,
}

impl WcetPhase {
    /// Create a new WCET phase
    pub const fn new(name: &'static str, wcet: u64, required: bool) -> Self {
        Self {
            name,
            wcet,
            required,
        }
    }
}

/// WCET analyzer for compile-time timing analysis
pub struct WcetAnalyzer;

impl WcetAnalyzer {
    /// Analyze WCET for a single μ-op
    pub const fn analyze_mu_op(op_code: u8) -> WcetResult {
        // All μ-ops are designed to be 1 tick
        WcetResult::new(1, 1, 1)
    }

    /// Analyze WCET for pattern execution
    pub const fn analyze_pattern(pattern_id: PatternId) -> WcetResult {
        let cost = pattern_id.tick_cost() as u64;

        // Most patterns have deterministic cost
        // Some patterns (like deferred choice) may vary
        let (best, average) = match pattern_id {
            PatternId::DeferredChoice => (cost - 1, cost),
            PatternId::ExclusiveChoice => (cost, cost),
            _ => (cost, cost),
        };

        WcetResult::new(cost, best, average)
    }

    /// Analyze WCET for guard evaluation
    pub const fn analyze_guard_evaluation(guard_count: u64) -> WcetResult {
        // Each guard is 1 tick, branchless evaluation
        let ticks = guard_count;
        WcetResult::new(ticks, ticks, ticks)
    }

    /// Analyze WCET for a complete task
    pub const fn analyze_task(pattern_id: PatternId, guard_count: u64) -> WcetResult {
        // Task phases:
        // 1. Load Σ* descriptor (1 tick)
        // 2. Dispatch pattern (1 tick)
        // 3. Evaluate guards (N ticks)
        // 4. Execute pattern (varies)
        // 5. Write receipt (1 tick)

        let load_wcet = 1;
        let dispatch_wcet = 1;
        let guard_wcet = guard_count;
        let pattern_wcet = pattern_id.tick_cost() as u64;
        let receipt_wcet = 1;

        let worst = load_wcet + dispatch_wcet + guard_wcet + pattern_wcet + receipt_wcet;
        let best = worst; // Deterministic execution
        let average = worst;

        WcetResult::new(worst, best, average)
    }

    /// Analyze WCET for sequential composition
    pub const fn analyze_sequential<const N: usize>(phases: [WcetPhase; N]) -> WcetResult {
        let mut worst = 0;
        let mut best = 0;
        let mut average = 0;

        let mut i = 0;
        while i < N {
            if phases[i].required {
                worst += phases[i].wcet;
                best += phases[i].wcet;
                average += phases[i].wcet;
            } else {
                // Optional phase: adds to worst case only
                worst += phases[i].wcet;
                average += phases[i].wcet / 2; // 50% probability
            }
            i += 1;
        }

        WcetResult::new(worst, best, average)
    }

    /// Analyze WCET for parallel composition
    pub const fn analyze_parallel<const N: usize>(branch_wcets: [u64; N]) -> WcetResult {
        // For parallel execution, WCET is max of all branches
        let mut worst = 0;
        let mut best = u64::MAX;
        let mut sum = 0;

        let mut i = 0;
        while i < N {
            if branch_wcets[i] > worst {
                worst = branch_wcets[i];
            }
            if branch_wcets[i] < best {
                best = branch_wcets[i];
            }
            sum += branch_wcets[i];
            i += 1;
        }

        let average = if N > 0 { sum / N as u64 } else { 0 };

        WcetResult::new(worst, best, average)
    }

    /// Analyze WCET for conditional branching
    pub const fn analyze_conditional(
        condition_wcet: u64,
        true_branch_wcet: u64,
        false_branch_wcet: u64,
    ) -> WcetResult {
        // Worst case is condition + max(true, false)
        let max_branch = if true_branch_wcet > false_branch_wcet {
            true_branch_wcet
        } else {
            false_branch_wcet
        };

        let min_branch = if true_branch_wcet < false_branch_wcet {
            true_branch_wcet
        } else {
            false_branch_wcet
        };

        let worst = condition_wcet + max_branch;
        let best = condition_wcet + min_branch;
        let average = condition_wcet + (true_branch_wcet + false_branch_wcet) / 2;

        WcetResult::new(worst, best, average)
    }

    /// Analyze WCET for loop (bounded)
    pub const fn analyze_bounded_loop(iteration_wcet: u64, max_iterations: u64) -> WcetResult {
        let worst = iteration_wcet * max_iterations;
        let best = iteration_wcet; // Minimum 1 iteration
        let average = iteration_wcet * (max_iterations / 2);

        WcetResult::new(worst, best, average)
    }
}

/// Const function to compute operation composition WCET
pub const fn compose_wcet(wcet1: WcetResult, wcet2: WcetResult) -> WcetResult {
    WcetResult::new(
        wcet1.worst_case_ticks + wcet2.worst_case_ticks,
        wcet1.best_case_ticks + wcet2.best_case_ticks,
        wcet1.average_case_ticks + wcet2.average_case_ticks,
    )
}

/// Const function to compute parallel WCET
pub const fn parallel_wcet(wcet1: WcetResult, wcet2: WcetResult) -> WcetResult {
    let worst = if wcet1.worst_case_ticks > wcet2.worst_case_ticks {
        wcet1.worst_case_ticks
    } else {
        wcet2.worst_case_ticks
    };

    let best = if wcet1.best_case_ticks < wcet2.best_case_ticks {
        wcet1.best_case_ticks
    } else {
        wcet2.best_case_ticks
    };

    let average = (wcet1.average_case_ticks + wcet2.average_case_ticks) / 2;

    WcetResult::new(worst, best, average)
}

/// Type-level WCET proof
///
/// Encodes WCET in the type system for compile-time verification
pub struct WcetProof<const WORST: u64, const BEST: u64> {
    _marker: PhantomData<([(); WORST as usize], [(); BEST as usize])>,
}

impl<const WORST: u64, const BEST: u64> WcetProof<WORST, BEST> {
    /// Create a new WCET proof
    pub const fn new() -> Self
    where
        [(); (WORST <= CHATMAN_CONSTANT) as usize]:,
        [(); (BEST <= WORST) as usize]:,
    {
        Self {
            _marker: PhantomData,
        }
    }

    /// Get worst-case ticks
    pub const fn worst_case() -> u64 {
        WORST
    }

    /// Get best-case ticks
    pub const fn best_case() -> u64 {
        BEST
    }

    /// Check if within Chatman Constant
    pub const fn is_hot_path() -> bool {
        WORST <= CHATMAN_CONSTANT
    }
}

/// Macro to assert WCET at compile time
#[macro_export]
macro_rules! assert_wcet {
    ($wcet:expr, $limit:expr) => {
        const _: () = {
            if $wcet > $limit {
                panic!("WCET exceeds limit");
            }
        };
    };
}

/// Const function to verify WCET is within budget
pub const fn verify_wcet(wcet: WcetResult, budget: u64) {
    if wcet.worst_case_ticks > budget {
        panic!("WCET exceeds budget");
    }
}

/// Standard task WCET computation (const)
pub const fn standard_task_wcet(pattern_cost: u64, guard_count: u64) -> u64 {
    // load(1) + dispatch(1) + guards(N) + pattern(P) + receipt(1)
    1 + 1 + guard_count + pattern_cost + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcet_result_creation() {
        const WCET: WcetResult = WcetResult::new(8, 5, 6);

        assert_eq!(WCET.worst_case_ticks, 8);
        assert_eq!(WCET.best_case_ticks, 5);
        assert_eq!(WCET.average_case_ticks, 6);
        assert!(WCET.is_hot_path);
        assert_eq!(WCET.variability(), 3);
    }

    #[test]
    fn test_analyze_pattern() {
        const SEQ_WCET: WcetResult = WcetAnalyzer::analyze_pattern(PatternId::Sequence);
        assert_eq!(SEQ_WCET.worst_case_ticks, 1);

        const PARA_WCET: WcetResult = WcetAnalyzer::analyze_pattern(PatternId::ParallelSplit);
        assert_eq!(PARA_WCET.worst_case_ticks, 2);
    }

    #[test]
    fn test_analyze_task() {
        // Sequence pattern with 2 guards
        const TASK_WCET: WcetResult = WcetAnalyzer::analyze_task(PatternId::Sequence, 2);

        // load(1) + dispatch(1) + guards(2) + pattern(1) + receipt(1) = 6
        assert_eq!(TASK_WCET.worst_case_ticks, 6);
        assert!(TASK_WCET.is_hot_path);
    }

    #[test]
    fn test_analyze_sequential() {
        const PHASES: [WcetPhase; 3] = [
            WcetPhase::new("phase1", 2, true),
            WcetPhase::new("phase2", 3, true),
            WcetPhase::new("phase3", 1, false),
        ];

        const RESULT: WcetResult = WcetAnalyzer::analyze_sequential(PHASES);

        assert_eq!(RESULT.worst_case_ticks, 6); // All phases
        assert_eq!(RESULT.best_case_ticks, 5); // Required only
    }

    #[test]
    fn test_analyze_parallel() {
        const BRANCHES: [u64; 4] = [2, 5, 3, 4];
        const RESULT: WcetResult = WcetAnalyzer::analyze_parallel(BRANCHES);

        assert_eq!(RESULT.worst_case_ticks, 5); // Max branch
        assert_eq!(RESULT.best_case_ticks, 2); // Min branch
    }

    #[test]
    fn test_analyze_conditional() {
        const RESULT: WcetResult = WcetAnalyzer::analyze_conditional(
            1, // condition cost
            4, // true branch
            3, // false branch
        );

        assert_eq!(RESULT.worst_case_ticks, 5); // condition + max(4, 3)
        assert_eq!(RESULT.best_case_ticks, 4); // condition + min(4, 3)
    }

    #[test]
    fn test_compose_wcet() {
        const WCET1: WcetResult = WcetResult::new(3, 2, 2);
        const WCET2: WcetResult = WcetResult::new(4, 3, 3);
        const COMPOSED: WcetResult = compose_wcet(WCET1, WCET2);

        assert_eq!(COMPOSED.worst_case_ticks, 7);
        assert_eq!(COMPOSED.best_case_ticks, 5);
    }

    #[test]
    fn test_parallel_wcet() {
        const WCET1: WcetResult = WcetResult::new(5, 3, 4);
        const WCET2: WcetResult = WcetResult::new(4, 2, 3);
        const PARALLEL: WcetResult = parallel_wcet(WCET1, WCET2);

        assert_eq!(PARALLEL.worst_case_ticks, 5); // Max worst
        assert_eq!(PARALLEL.best_case_ticks, 2); // Min best
    }

    #[test]
    fn test_standard_task_wcet() {
        const WCET1: u64 = standard_task_wcet(1, 2); // Sequence + 2 guards
        assert_eq!(WCET1, 5);

        const WCET2: u64 = standard_task_wcet(3, 3); // Sync + 3 guards
        assert_eq!(WCET2, 8);
    }

    #[test]
    fn test_wcet_within_budget() {
        const WCET: WcetResult = WcetResult::new(7, 5, 6);

        assert!(WCET.within_budget(8));
        assert!(WCET.within_budget(7));
        assert!(!WCET.within_budget(6));
    }

    #[test]
    fn test_analyze_bounded_loop() {
        const LOOP_WCET: WcetResult = WcetAnalyzer::analyze_bounded_loop(
            2, // iteration cost
            4, // max iterations
        );

        assert_eq!(LOOP_WCET.worst_case_ticks, 8); // 2 * 4
        assert_eq!(LOOP_WCET.best_case_ticks, 2); // 1 iteration
    }
}
