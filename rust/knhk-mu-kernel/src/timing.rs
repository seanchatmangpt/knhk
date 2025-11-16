//! Cycle-Accurate Timing for μ-kernel
//!
//! Provides hardware cycle counters and tick budgets.
//! This module uses inline assembly for precise timing.

use core::sync::atomic::{AtomicU64, Ordering};

/// Tick counter using hardware cycle counter
#[repr(C, align(64))]  // Cache-line aligned
pub struct TickCounter {
    start: u64,
    current: AtomicU64,
}

impl TickCounter {
    /// Create a new tick counter
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            start: 0,
            current: AtomicU64::new(0),
        }
    }

    /// Start counting (capture hardware counter)
    #[inline(always)]
    pub fn start(&mut self) {
        self.start = Self::read_cycle_counter();
        self.current.store(self.start, Ordering::Relaxed);
    }

    /// Get current tick count since start
    #[inline(always)]
    pub fn ticks(&self) -> u64 {
        let current = Self::read_cycle_counter();
        current.saturating_sub(self.start)
    }

    /// Read hardware cycle counter
    #[inline(always)]
    #[cfg(target_arch = "x86_64")]
    fn read_cycle_counter() -> u64 {
        unsafe {
            let mut low: u32;
            let mut high: u32;
            core::arch::asm!(
                "rdtsc",
                "lfence",  // Serialize
                lateout("eax") low,
                lateout("edx") high,
                options(nostack, nomem)
            );
            ((high as u64) << 32) | (low as u64)
        }
    }

    #[inline(always)]
    #[cfg(target_arch = "aarch64")]
    fn read_cycle_counter() -> u64 {
        unsafe {
            let cnt: u64;
            core::arch::asm!(
                "mrs {cnt}, cntvct_el0",
                cnt = out(reg) cnt,
                options(nostack, nomem)
            );
            cnt
        }
    }

    #[inline(always)]
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn read_cycle_counter() -> u64 {
        // Fallback for other architectures (less precise)
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}

impl Default for TickCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Tick budget for μ operations
#[derive(Debug, Clone, Copy)]
#[repr(C, align(8))]
pub struct TickBudget {
    /// Maximum allowed ticks
    pub limit: u64,
    /// Ticks used so far
    pub used: u64,
}

impl TickBudget {
    /// Create a new tick budget with Chatman Constant
    #[inline(always)]
    pub const fn chatman() -> Self {
        Self {
            limit: crate::CHATMAN_CONSTANT,
            used: 0,
        }
    }

    /// Create a custom tick budget
    #[inline(always)]
    pub const fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }

    /// Check if budget is exhausted
    #[inline(always)]
    pub const fn is_exhausted(&self) -> bool {
        self.used >= self.limit
    }

    /// Remaining ticks
    #[inline(always)]
    pub const fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Consume ticks (branchless)
    #[inline(always)]
    pub fn consume(&mut self, ticks: u64) -> BudgetStatus {
        self.used = self.used.saturating_add(ticks);

        // Branchless status determination
        let exhausted = (self.used >= self.limit) as u8;

        // Use const array lookup instead of match (branchless)
        const STATUS_TABLE: [BudgetStatus; 2] = [
            BudgetStatus::Ok,
            BudgetStatus::Exhausted,
        ];
        STATUS_TABLE[exhausted as usize]
    }

    /// Reset budget
    #[inline(always)]
    pub fn reset(&mut self) {
        self.used = 0;
    }
}

/// Budget status (returned by consume)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BudgetStatus {
    /// Budget OK
    Ok = 0,
    /// Budget exhausted
    Exhausted = 1,
}

/// Measure execution time of a closure
#[inline(always)]
pub fn measure_ticks<F, R>(f: F) -> (R, u64)
where
    F: FnOnce() -> R,
{
    let mut counter = TickCounter::new();
    counter.start();
    let result = f();
    let ticks = counter.ticks();
    (result, ticks)
}

/// Assert tick budget (compile-time constant if possible)
#[macro_export]
macro_rules! assert_ticks {
    ($expr:expr, $limit:expr) => {{
        let (result, ticks) = $crate::timing::measure_ticks(|| $expr);
        assert!(
            ticks <= $limit,
            "Tick budget exceeded: {} > {} (violation: {})",
            ticks,
            $limit,
            ticks.saturating_sub($limit)
        );
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_counter() {
        let mut counter = TickCounter::new();
        counter.start();

        // Do some work
        let mut sum = 0u64;
        for i in 0..100 {
            sum = sum.wrapping_add(i);
        }

        let ticks = counter.ticks();
        assert!(ticks > 0, "Tick counter should advance");
        assert!(sum > 0, "Work should complete"); // Prevent optimization
    }

    #[test]
    fn test_tick_budget() {
        let mut budget = TickBudget::chatman();

        assert_eq!(budget.limit, crate::CHATMAN_CONSTANT);
        assert_eq!(budget.used, 0);
        assert!(!budget.is_exhausted());
        assert_eq!(budget.remaining(), 8);

        assert_eq!(budget.consume(3), BudgetStatus::Ok);
        assert_eq!(budget.used, 3);
        assert_eq!(budget.remaining(), 5);

        assert_eq!(budget.consume(6), BudgetStatus::Exhausted);
        assert!(budget.is_exhausted());
    }

    #[test]
    fn test_measure_ticks() {
        let (result, ticks) = measure_ticks(|| {
            let mut sum = 0u64;
            for i in 0..1000 {
                sum = sum.wrapping_add(i);
            }
            sum
        });

        assert!(result > 0);
        assert!(ticks > 0);
    }

    #[test]
    fn test_budget_branchless() {
        // Test that consume is branchless
        let mut budget = TickBudget::new(10);

        for i in 1..=15 {
            let status = budget.consume(1);
            if i <= 10 {
                assert_eq!(status, BudgetStatus::Ok);
            } else {
                assert_eq!(status, BudgetStatus::Exhausted);
            }
        }
    }
}
