// knhk-yawl/src/execution.rs
// Pattern execution utilities

use crate::error::{YawlError, YawlResult};
use std::time::Instant;

/// Measure execution time in ticks and microseconds
///
/// Returns (ticks, microseconds)
pub fn measure_execution<F, T>(f: F) -> YawlResult<(T, u32, u64)>
where
    F: FnOnce() -> YawlResult<T>,
{
    let start = Instant::now();
    let start_ticks = read_tsc();

    let result = f()?;

    let end_ticks = read_tsc();
    let duration = start.elapsed();

    let ticks = end_ticks.saturating_sub(start_ticks);
    let duration_us = duration.as_micros() as u64;

    Ok((result, ticks, duration_us))
}

/// Read CPU timestamp counter (TSC) for tick measurement
///
/// This uses RDTSC on x86/x86_64 for precise tick counting.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn read_tsc() -> u32 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::_rdtsc;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::_rdtsc;

    unsafe { _rdtsc() as u32 }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn read_tsc() -> u32 {
    // Fallback for non-x86 architectures
    0
}

/// Validate that execution did not exceed the Chatman constant (8 ticks)
pub fn validate_chatman_constant(ticks: u32) -> YawlResult<()> {
    const MAX_TICKS: u32 = 8;

    if ticks > MAX_TICKS {
        return Err(YawlError::PerformanceViolation(ticks));
    }

    Ok(())
}

/// Validate that loop iterations do not exceed the maximum (Q3: Bounded recursion)
pub fn validate_iteration_limit(iterations: u32, max: u32) -> YawlResult<()> {
    if iterations > max {
        return Err(YawlError::IterationLimitExceeded(max));
    }

    Ok(())
}
