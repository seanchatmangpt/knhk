//! Tick-budget accounting for Chatman Constant compliance (≤8 ticks)
//!
//! Provides cycle counting and tick-budget validation for hot path operations.
//! Ensures all hot path operations stay within the Chatman Constant (≤8 ticks = 2ns).

use crate::error::{WorkflowError, WorkflowResult};

/// Tick budget for hot path operations (Chatman Constant: 8 ticks = 2ns)
pub const HOT_PATH_TICK_BUDGET: u64 = 8;

/// Tick counter using RDTSC (Read Time-Stamp Counter)
///
/// On x86_64, uses `rdtsc` instruction for cycle counting.
/// On other platforms, uses `std::time::Instant` as fallback.
pub struct TickCounter {
    /// Start tick count
    start_ticks: u64,
}

impl TickCounter {
    /// Create a new tick counter and start counting
    pub fn start() -> Self {
        Self {
            start_ticks: Self::read_ticks(),
        }
    }

    /// Read current tick count
    fn read_ticks() -> u64 {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { std::arch::x86_64::_rdtsc() }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            // Fallback: use SystemTime for non-x86_64 platforms
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        }
    }

    /// Get elapsed ticks since start
    pub fn elapsed_ticks(&self) -> u64 {
        Self::read_ticks().saturating_sub(self.start_ticks)
    }

    /// Check if elapsed ticks exceed budget
    pub fn exceeds_budget(&self, budget: u64) -> bool {
        self.elapsed_ticks() > budget
    }

    /// Assert that elapsed ticks are within budget
    pub fn assert_within_budget(&self, budget: u64) -> WorkflowResult<()> {
        let elapsed = self.elapsed_ticks();
        if elapsed > budget {
            Err(WorkflowError::Internal(format!(
                "Tick budget exceeded: {} ticks > {} ticks (Chatman Constant)",
                elapsed, budget
            )))
        } else {
            Ok(())
        }
    }
}

/// Measure tick count for an operation
///
/// Returns the number of ticks elapsed during the operation.
pub fn measure_ticks<F, R>(f: F) -> (R, u64)
where
    F: FnOnce() -> R,
{
    let counter = TickCounter::start();
    let result = f();
    let ticks = counter.elapsed_ticks();
    (result, ticks)
}

/// Measure tick count for an async operation
///
/// Returns the result and the number of ticks elapsed during the operation.
pub async fn measure_ticks_async<F, Fut, R>(f: F) -> (R, u64)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let counter = TickCounter::start();
    let result = f().await;
    let ticks = counter.elapsed_ticks();
    (result, ticks)
}

/// Assert that an operation stays within tick budget
///
/// Returns an error if the operation exceeds the budget.
pub fn assert_within_budget<F, R>(budget: u64, f: F) -> WorkflowResult<R>
where
    F: FnOnce() -> R,
{
    let counter = TickCounter::start();
    let result = f();
    counter.assert_within_budget(budget)?;
    Ok(result)
}

/// Assert that an async operation stays within tick budget
///
/// Returns an error if the operation exceeds the budget.
pub async fn assert_within_budget_async<F, Fut, R>(budget: u64, f: F) -> WorkflowResult<R>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let counter = TickCounter::start();
    let result = f().await;
    counter.assert_within_budget(budget)?;
    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_counter() {
        let counter = TickCounter::start();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let elapsed = counter.elapsed_ticks();
        assert!(elapsed > 0);
    }

    #[test]
    fn test_measure_ticks() {
        let (result, ticks) = measure_ticks(|| {
            std::thread::sleep(std::time::Duration::from_millis(1));
            42
        });
        assert_eq!(result, 42);
        assert!(ticks > 0);
    }

    #[test]
    fn test_assert_within_budget() {
        // Fast operation should pass
        let result = assert_within_budget(1000, || 42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
}
