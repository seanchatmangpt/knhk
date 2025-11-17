//! MAPE-K Protocol State Machine
//!
//! This module encodes the MAPE-K autonomic control loop as a type-level state machine.
//! The MAPE-K loop consists of:
//! - **Monitor** - Collect observations
//! - **Analyze** - Detect symptoms
//! - **Plan** - Generate proposals
//! - **Execute** - Apply changes
//! - **Knowledge** - Update knowledge base
//!
//! The type system enforces:
//! - Cannot skip phases (compile error)
//! - Must cycle through all phases
//! - Cannot repeat phase without cycling
//! - Integration with existing MAPE module
//!
//! ## Example
//! ```no_run
//! use knhk_mu_kernel::protocols::mape_protocol::*;
//!
//! // Create MAPE-K cycle
//! let cycle = MapeKCycle::new();
//!
//! // Must follow exact order
//! let cycle = cycle.monitor(receipt);
//! let cycle = cycle.analyze();
//! let cycle = cycle.plan();
//! let cycle = cycle.execute();
//! let _cycle = cycle.update_knowledge();
//!
//! // Invalid: cycle.plan(); // ERROR: no method `plan` on MonitorPhase
//! ```

use crate::mape::{AnalyzeResult, ExecuteResult, MonitorResult, PlanResult, Symptom};
use crate::overlay::DeltaSigma;
use crate::receipts::Receipt;
use core::marker::PhantomData;

/// MAPE-K cycle state machine
///
/// Parameterized by the current phase.
/// Type system ensures phases are executed in order.
pub struct MapeKCycle<Phase> {
    /// Type-level phase marker
    _phase: PhantomData<fn() -> Phase>,
}

impl<Phase> MapeKCycle<Phase> {
    /// Internal constructor
    #[inline(always)]
    const fn new() -> Self {
        Self {
            _phase: PhantomData,
        }
    }
}

// Zero-cost guarantee
const _: () = {
    assert!(core::mem::size_of::<MapeKCycle<()>>() == 0);
};

/// Phase marker types (zero-sized)

/// Monitor phase - collect observations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorPhase;

/// Analyze phase - detect symptoms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalyzePhase;

/// Plan phase - generate proposals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanPhase;

/// Execute phase - apply changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutePhase;

/// Knowledge phase - update knowledge base
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnowledgePhase;

// State transitions (enforce MAPE-K order)

impl MapeKCycle<MonitorPhase> {
    /// Create new MAPE-K cycle starting at Monitor phase
    #[inline(always)]
    pub const fn new() -> Self {
        Self::new()
    }

    /// Monitor phase - collect observations from receipts
    ///
    /// This is the first phase of MAPE-K.
    /// Type system ensures you start here.
    #[inline(always)]
    pub fn monitor(self, _receipt: Receipt) -> MapeKCycle<AnalyzePhase> {
        // In real implementation, would collect receipt
        MapeKCycle::new()
    }
}

impl Default for MapeKCycle<MonitorPhase> {
    fn default() -> Self {
        Self::new()
    }
}

impl MapeKCycle<AnalyzePhase> {
    /// Analyze phase - detect symptoms from observations
    ///
    /// Can only be called after Monitor.
    /// Type system enforces this.
    #[inline(always)]
    pub fn analyze(self) -> MapeKCycle<PlanPhase> {
        // In real implementation, would analyze observations
        MapeKCycle::new()
    }
}

impl MapeKCycle<PlanPhase> {
    /// Plan phase - generate ΔΣ proposals
    ///
    /// Can only be called after Analyze.
    /// Type system enforces this.
    #[inline(always)]
    pub fn plan(self) -> MapeKCycle<ExecutePhase> {
        // In real implementation, would generate proposals
        MapeKCycle::new()
    }
}

impl MapeKCycle<ExecutePhase> {
    /// Execute phase - apply ΔΣ via shadow deployment
    ///
    /// Can only be called after Plan.
    /// Type system enforces this.
    #[inline(always)]
    pub fn execute(self) -> MapeKCycle<KnowledgePhase> {
        // In real implementation, would apply overlay
        MapeKCycle::new()
    }
}

impl MapeKCycle<KnowledgePhase> {
    /// Knowledge phase - update knowledge base with results
    ///
    /// This completes one cycle.
    /// Must restart from Monitor for next cycle.
    #[inline(always)]
    pub fn update_knowledge(self) -> MapeKCycle<MonitorPhase> {
        // Cycle back to Monitor
        MapeKCycle::new()
    }
}

/// MAPE-K cycle with data
///
/// This variant carries the actual results through the phases.
pub struct MapeKCycleWithData<Phase, D> {
    /// Phase marker
    _phase: PhantomData<fn() -> Phase>,
    /// Accumulated data
    data: D,
}

impl<Phase, D> MapeKCycleWithData<Phase, D> {
    /// Internal constructor
    #[inline(always)]
    fn with_data(data: D) -> Self {
        Self {
            _phase: PhantomData,
            data,
        }
    }

    /// Get data reference
    #[inline(always)]
    pub fn data(&self) -> &D {
        &self.data
    }
}

/// Data accumulated during MAPE-K cycle
#[derive(Debug, Clone)]
pub struct MapeKData {
    /// Monitoring results
    pub monitor_result: Option<MonitorResult>,
    /// Analysis results
    pub analyze_result: Option<AnalyzeResult>,
    /// Planning results
    pub plan_result: Option<PlanResult>,
    /// Execution results
    pub execute_result: Option<ExecuteResult>,
}

impl MapeKData {
    /// Create new empty data
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            monitor_result: None,
            analyze_result: None,
            plan_result: None,
            execute_result: None,
        }
    }
}

impl Default for MapeKData {
    fn default() -> Self {
        Self::new()
    }
}

impl MapeKCycleWithData<MonitorPhase, MapeKData> {
    /// Create new cycle with data tracking
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_data(MapeKData::new())
    }

    /// Monitor with result tracking
    #[inline(always)]
    pub fn monitor(mut self, result: MonitorResult) -> MapeKCycleWithData<AnalyzePhase, MapeKData> {
        self.data.monitor_result = Some(result);
        MapeKCycleWithData::with_data(self.data)
    }
}

impl Default for MapeKCycleWithData<MonitorPhase, MapeKData> {
    fn default() -> Self {
        Self::new()
    }
}

impl MapeKCycleWithData<AnalyzePhase, MapeKData> {
    /// Analyze with result tracking
    #[inline(always)]
    pub fn analyze(mut self, result: AnalyzeResult) -> MapeKCycleWithData<PlanPhase, MapeKData> {
        self.data.analyze_result = Some(result);
        MapeKCycleWithData::with_data(self.data)
    }
}

impl MapeKCycleWithData<PlanPhase, MapeKData> {
    /// Plan with result tracking
    #[inline(always)]
    pub fn plan(mut self, result: PlanResult) -> MapeKCycleWithData<ExecutePhase, MapeKData> {
        self.data.plan_result = Some(result);
        MapeKCycleWithData::with_data(self.data)
    }
}

impl MapeKCycleWithData<ExecutePhase, MapeKData> {
    /// Execute with result tracking
    #[inline(always)]
    pub fn execute(
        mut self,
        result: ExecuteResult,
    ) -> MapeKCycleWithData<KnowledgePhase, MapeKData> {
        self.data.execute_result = Some(result);
        MapeKCycleWithData::with_data(self.data)
    }
}

impl MapeKCycleWithData<KnowledgePhase, MapeKData> {
    /// Complete cycle and return to Monitor with knowledge update
    #[inline(always)]
    pub fn update_knowledge(self) -> (MapeKCycleWithData<MonitorPhase, MapeKData>, MapeKData) {
        let data = self.data;
        (MapeKCycleWithData::new(), data)
    }

    /// Get accumulated data
    #[inline(always)]
    pub fn into_data(self) -> MapeKData {
        self.data
    }
}

/// Conditional MAPE-K transitions
///
/// Allows conditional execution while maintaining type safety.
pub enum MapeKBranch<P1, P2> {
    /// Normal path
    Normal(MapeKCycle<P1>),
    /// Alternative path (e.g., early exit)
    Alternative(MapeKCycle<P2>),
}

impl<P1, P2> MapeKBranch<P1, P2> {
    /// Match on branch
    #[inline(always)]
    pub fn match_branch<F, G, R>(self, f: F, g: G) -> R
    where
        F: FnOnce(MapeKCycle<P1>) -> R,
        G: FnOnce(MapeKCycle<P2>) -> R,
    {
        match self {
            MapeKBranch::Normal(cycle) => f(cycle),
            MapeKBranch::Alternative(cycle) => g(cycle),
        }
    }
}

/// MAPE-K with timing guarantees
///
/// Tracks ticks to ensure Chatman Constant compliance.
pub struct TimedMapeK<Phase> {
    _phase: PhantomData<fn() -> Phase>,
    /// Ticks spent in current phase
    ticks: u64,
}

impl<Phase> TimedMapeK<Phase> {
    /// Create timed cycle
    #[inline(always)]
    fn new() -> Self {
        Self {
            _phase: PhantomData,
            ticks: 0,
        }
    }

    /// Get tick count
    #[inline(always)]
    pub fn ticks(&self) -> u64 {
        self.ticks
    }
}

impl TimedMapeK<MonitorPhase> {
    /// Create new timed MAPE-K cycle
    #[inline(always)]
    pub fn new() -> Self {
        Self::new()
    }

    /// Monitor with tick counting
    #[inline(always)]
    pub fn monitor(self, ticks: u64) -> TimedMapeK<AnalyzePhase> {
        TimedMapeK {
            _phase: PhantomData,
            ticks,
        }
    }
}

impl Default for TimedMapeK<MonitorPhase> {
    fn default() -> Self {
        Self::new()
    }
}

impl TimedMapeK<AnalyzePhase> {
    /// Analyze with tick counting
    #[inline(always)]
    pub fn analyze(self, ticks: u64) -> TimedMapeK<PlanPhase> {
        TimedMapeK {
            _phase: PhantomData,
            ticks: self.ticks.saturating_add(ticks),
        }
    }
}

impl TimedMapeK<PlanPhase> {
    /// Plan with tick counting
    #[inline(always)]
    pub fn plan(self, ticks: u64) -> TimedMapeK<ExecutePhase> {
        TimedMapeK {
            _phase: PhantomData,
            ticks: self.ticks.saturating_add(ticks),
        }
    }
}

impl TimedMapeK<ExecutePhase> {
    /// Execute with tick counting
    #[inline(always)]
    pub fn execute(self, ticks: u64) -> TimedMapeK<KnowledgePhase> {
        TimedMapeK {
            _phase: PhantomData,
            ticks: self.ticks.saturating_add(ticks),
        }
    }
}

impl TimedMapeK<KnowledgePhase> {
    /// Update knowledge and get total ticks
    #[inline(always)]
    pub fn update_knowledge(self) -> (TimedMapeK<MonitorPhase>, u64) {
        let total_ticks = self.ticks;
        (TimedMapeK::new(), total_ticks)
    }

    /// Check if within Chatman Constant
    #[inline(always)]
    pub fn within_chatman_constant(&self) -> bool {
        self.ticks <= crate::CHATMAN_CONSTANT
    }
}

/// MAPE-K cycle counter
///
/// Tracks how many complete cycles have been executed.
pub struct CycleCounter<Phase> {
    _phase: PhantomData<fn() -> Phase>,
    /// Number of completed cycles
    count: u64,
}

impl CycleCounter<MonitorPhase> {
    /// Create new cycle counter
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            _phase: PhantomData,
            count: 0,
        }
    }

    /// Get cycle count
    #[inline(always)]
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Monitor phase
    #[inline(always)]
    pub fn monitor(self) -> CycleCounter<AnalyzePhase> {
        CycleCounter {
            _phase: PhantomData,
            count: self.count,
        }
    }
}

impl Default for CycleCounter<MonitorPhase> {
    fn default() -> Self {
        Self::new()
    }
}

impl CycleCounter<AnalyzePhase> {
    /// Analyze phase
    #[inline(always)]
    pub fn analyze(self) -> CycleCounter<PlanPhase> {
        CycleCounter {
            _phase: PhantomData,
            count: self.count,
        }
    }
}

impl CycleCounter<PlanPhase> {
    /// Plan phase
    #[inline(always)]
    pub fn plan(self) -> CycleCounter<ExecutePhase> {
        CycleCounter {
            _phase: PhantomData,
            count: self.count,
        }
    }
}

impl CycleCounter<ExecutePhase> {
    /// Execute phase
    #[inline(always)]
    pub fn execute(self) -> CycleCounter<KnowledgePhase> {
        CycleCounter {
            _phase: PhantomData,
            count: self.count,
        }
    }
}

impl CycleCounter<KnowledgePhase> {
    /// Update knowledge and increment counter
    #[inline(always)]
    pub fn update_knowledge(self) -> CycleCounter<MonitorPhase> {
        CycleCounter {
            _phase: PhantomData,
            count: self.count.saturating_add(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma::SigmaHash;

    #[test]
    fn test_basic_mape_k_cycle() {
        // Must follow exact order
        let cycle = MapeKCycle::new();

        let receipt = Receipt::new(0, SigmaHash([0; 32]), [0; 32], [0; 32], 5, 0, 0);

        let cycle = cycle.monitor(receipt);
        let cycle = cycle.analyze();
        let cycle = cycle.plan();
        let cycle = cycle.execute();
        let _cycle = cycle.update_knowledge();

        // Type system ensures correct order
    }

    #[test]
    fn test_mape_k_cycle_with_data() {
        let cycle = MapeKCycleWithData::new();

        let monitor_result = MonitorResult {
            receipt_id: 1,
            observations_count: 10,
            avg_tau: 5.5,
        };

        let analyze_result = AnalyzeResult {
            symptoms: alloc::vec![],
        };

        let plan_result = PlanResult {
            proposals: alloc::vec![],
        };

        let execute_result = ExecuteResult {
            success: true,
            new_sigma_id: None,
            error: None,
        };

        let cycle = cycle.monitor(monitor_result);
        let cycle = cycle.analyze(analyze_result);
        let cycle = cycle.plan(plan_result);
        let cycle = cycle.execute(execute_result);
        let (_cycle, data) = cycle.update_knowledge();

        assert!(data.monitor_result.is_some());
        assert!(data.analyze_result.is_some());
    }

    #[test]
    fn test_timed_mape_k() {
        let cycle = TimedMapeK::new();

        let cycle = cycle.monitor(2);
        let cycle = cycle.analyze(1);
        let cycle = cycle.plan(2);
        let cycle = cycle.execute(3);

        assert_eq!(cycle.ticks(), 8); // 2 + 1 + 2 + 3
        assert!(!cycle.within_chatman_constant()); // > 8 ticks

        let (_cycle, total) = cycle.update_knowledge();
        assert_eq!(total, 8);
    }

    #[test]
    fn test_cycle_counter() {
        let mut counter = CycleCounter::new();

        // First cycle
        counter = counter
            .monitor()
            .analyze()
            .plan()
            .execute()
            .update_knowledge();
        assert_eq!(counter.count(), 1);

        // Second cycle
        counter = counter
            .monitor()
            .analyze()
            .plan()
            .execute()
            .update_knowledge();
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_zero_size() {
        // Verify zero runtime overhead
        assert_eq!(core::mem::size_of::<MapeKCycle<MonitorPhase>>(), 0);
        assert_eq!(core::mem::size_of::<MapeKCycle<AnalyzePhase>>(), 0);
        assert_eq!(core::mem::size_of::<MapeKCycle<PlanPhase>>(), 0);
        assert_eq!(core::mem::size_of::<MapeKCycle<ExecutePhase>>(), 0);
        assert_eq!(core::mem::size_of::<MapeKCycle<KnowledgePhase>>(), 0);
    }
}
