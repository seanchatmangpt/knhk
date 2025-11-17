// knhk-kernel: RDTSC-based tick counter for hot path measurement
// Provides cycle-accurate timing with ≤8 tick validation

use std::sync::atomic::{AtomicU64, Ordering};

/// CPU frequency in Hz (cached after calibration)
static CPU_FREQUENCY: AtomicU64 = AtomicU64::new(0);

/// Overhead of RDTSC instruction itself (in ticks)
static RDTSC_OVERHEAD: AtomicU64 = AtomicU64::new(0);

/// Read Time Stamp Counter (x86-64 only)
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn read_tsc() -> u64 {
    unsafe { std::arch::x86_64::_rdtsc() }
}

/// Read Time Stamp Counter with serialization (more accurate but slower)
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn read_tsc_serialized() -> u64 {
    unsafe {
        // CPUID serializes instruction stream
        std::arch::x86_64::__cpuid(0);
        std::arch::x86_64::_rdtsc()
    }
}

/// Read Time Stamp Counter with fence
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn read_tsc_fenced() -> u64 {
    unsafe {
        std::arch::x86_64::_mm_mfence();
        let tsc = std::arch::x86_64::_rdtsc();
        std::arch::x86_64::_mm_mfence();
        tsc
    }
}

/// Fallback for non-x86_64 platforms
#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn read_tsc() -> u64 {
    // Use high-resolution monotonic clock as fallback
    let now = std::time::Instant::now();
    now.elapsed().as_nanos() as u64
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn read_tsc_serialized() -> u64 {
    read_tsc()
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn read_tsc_fenced() -> u64 {
    read_tsc()
}

/// Calibration result
#[derive(Debug, Clone, Copy)]
pub struct CalibrationResult {
    pub cpu_freq_hz: u64,
    pub rdtsc_overhead_ticks: u64,
    pub ticks_per_ns: f64,
    pub confidence: f64,
}

/// Timer for hot path measurements
pub struct HotPathTimer {
    start_ticks: u64,
    overhead: u64,
}

impl HotPathTimer {
    /// Create a new timer and start measuring immediately
    #[inline(always)]
    pub fn start() -> Self {
        let overhead = RDTSC_OVERHEAD.load(Ordering::Relaxed);
        Self {
            start_ticks: read_tsc(),
            overhead,
        }
    }

    /// Create a timer with serialized TSC read (more accurate)
    #[inline(always)]
    pub fn start_serialized() -> Self {
        let overhead = RDTSC_OVERHEAD.load(Ordering::Relaxed);
        Self {
            start_ticks: read_tsc_serialized(),
            overhead,
        }
    }

    /// Get elapsed ticks (compensated for overhead)
    #[inline(always)]
    pub fn elapsed_ticks(&self) -> u64 {
        let end = read_tsc();
        end.saturating_sub(self.start_ticks)
            .saturating_sub(self.overhead)
    }

    /// Check if elapsed time is within budget (≤8 ticks)
    #[inline(always)]
    pub fn within_budget(&self) -> bool {
        self.elapsed_ticks() <= 8
    }

    /// Reset timer to current time
    #[inline(always)]
    pub fn reset(&mut self) {
        self.start_ticks = read_tsc();
    }
}

/// Calibrate the TSC by measuring against wall clock
pub fn calibrate_tsc() -> CalibrationResult {
    const CALIBRATION_ROUNDS: usize = 100;
    const CALIBRATION_DURATION_MS: u64 = 10;

    // Measure RDTSC overhead first
    let overhead = measure_rdtsc_overhead();
    RDTSC_OVERHEAD.store(overhead, Ordering::Relaxed);

    let mut frequencies = Vec::with_capacity(CALIBRATION_ROUNDS);

    for _ in 0..CALIBRATION_ROUNDS {
        let start_wall = std::time::Instant::now();
        let start_tsc = read_tsc_serialized();

        // Busy wait for calibration duration
        while start_wall.elapsed().as_millis() < CALIBRATION_DURATION_MS as u128 {
            std::hint::spin_loop();
        }

        let end_tsc = read_tsc_serialized();
        let elapsed_wall = start_wall.elapsed();

        let ticks = end_tsc - start_tsc;
        let nanos = elapsed_wall.as_nanos() as u64;
        let freq_hz = (ticks as f64 * 1_000_000_000.0 / nanos as f64) as u64;

        frequencies.push(freq_hz);
    }

    // Calculate median frequency (more robust than mean)
    frequencies.sort_unstable();
    let median_freq = frequencies[frequencies.len() / 2];

    // Calculate confidence (inverse of variance)
    let mean = frequencies.iter().sum::<u64>() as f64 / frequencies.len() as f64;
    let variance = frequencies
        .iter()
        .map(|&f| {
            let diff = f as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / frequencies.len() as f64;

    let std_dev = variance.sqrt();
    let confidence = if std_dev > 0.0 {
        1.0 / (1.0 + std_dev / mean)
    } else {
        1.0
    };

    CPU_FREQUENCY.store(median_freq, Ordering::Relaxed);

    CalibrationResult {
        cpu_freq_hz: median_freq,
        rdtsc_overhead_ticks: overhead,
        ticks_per_ns: median_freq as f64 / 1_000_000_000.0,
        confidence,
    }
}

/// Measure the overhead of RDTSC instruction itself
fn measure_rdtsc_overhead() -> u64 {
    const ROUNDS: usize = 10000;
    let mut measurements = Vec::with_capacity(ROUNDS);

    // Warm up
    for _ in 0..100 {
        let _ = read_tsc();
    }

    // Measure overhead
    for _ in 0..ROUNDS {
        let start = read_tsc_serialized();
        std::hint::black_box(read_tsc());
        let end = read_tsc_serialized();
        measurements.push(end - start);
    }

    // Use 10th percentile to avoid outliers
    measurements.sort_unstable();
    measurements[ROUNDS / 10]
}

/// Tick budget tracker for enforcing Chatman constant
#[derive(Debug)]
pub struct TickBudget {
    budget: u64,
    spent: u64,
    operations: Vec<(&'static str, u64)>,
}

impl TickBudget {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom budget
    #[inline]
    pub fn with_budget(budget: u64) -> Self {
        Self {
            budget,
            spent: 0,
            operations: Vec::with_capacity(8),
        }
    }

    /// Charge ticks for an operation
    #[inline]
    pub fn charge(&mut self, operation: &'static str, ticks: u64) -> Result<(), String> {
        let new_spent = self.spent.saturating_add(ticks);
        if new_spent > self.budget {
            return Err(format!(
                "Tick budget exceeded: {} + {} = {} > {}",
                self.spent, ticks, new_spent, self.budget
            ));
        }
        self.spent = new_spent;
        self.operations.push((operation, ticks));
        Ok(())
    }

    /// Get remaining budget
    #[inline(always)]
    pub fn remaining(&self) -> u64 {
        self.budget.saturating_sub(self.spent)
    }

    /// Check if budget is exhausted
    #[inline(always)]
    pub fn exhausted(&self) -> bool {
        self.spent >= self.budget
    }

    /// Reset budget for reuse
    #[inline]
    pub fn reset(&mut self) {
        self.spent = 0;
        self.operations.clear();
    }

    /// Get breakdown of operations
    pub fn breakdown(&self) -> &[(&'static str, u64)] {
        &self.operations
    }
}

impl Default for TickBudget {
    fn default() -> Self {
        Self::with_budget(8)
    }
}

/// Scoped tick measurement with automatic charging
pub struct ScopedTickMeasurement<'a> {
    budget: &'a mut TickBudget,
    operation: &'static str,
    start: u64,
}

impl<'a> ScopedTickMeasurement<'a> {
    #[inline]
    pub fn new(budget: &'a mut TickBudget, operation: &'static str) -> Self {
        Self {
            budget,
            operation,
            start: read_tsc(),
        }
    }
}

impl<'a> Drop for ScopedTickMeasurement<'a> {
    #[inline]
    fn drop(&mut self) {
        let elapsed = read_tsc().saturating_sub(self.start);
        let _ = self.budget.charge(self.operation, elapsed);
    }
}

/// Macro for easy tick measurement
#[macro_export]
macro_rules! measure_ticks {
    ($budget:expr, $op:literal, $code:block) => {{
        let _guard = $crate::timer::ScopedTickMeasurement::new($budget, $op);
        $code
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_basic() {
        let timer = HotPathTimer::start();
        // Do some work
        let mut sum = 0u64;
        for i in 0..10 {
            sum = sum.wrapping_add(i);
        }
        let elapsed = timer.elapsed_ticks();

        // Should be some non-zero value
        assert!(elapsed > 0);
        println!("Basic operation took {} ticks", elapsed);
    }

    #[test]
    fn test_calibration() {
        let result = calibrate_tsc();
        println!("Calibration result: {:?}", result);

        // Sanity checks
        assert!(result.cpu_freq_hz > 1_000_000_000); // > 1 GHz
        assert!(result.cpu_freq_hz < 10_000_000_000); // < 10 GHz
        assert!(result.confidence > 0.9); // High confidence
    }

    #[test]
    fn test_tick_budget() {
        let mut budget = TickBudget::new();

        // Should succeed
        assert!(budget.charge("op1", 2).is_ok());
        assert!(budget.charge("op2", 3).is_ok());
        assert_eq!(budget.remaining(), 3);

        // Should fail - would exceed budget
        assert!(budget.charge("op3", 4).is_err());

        // But this should succeed
        assert!(budget.charge("op3", 3).is_ok());
        assert_eq!(budget.remaining(), 0);
        assert!(budget.exhausted());
    }

    #[test]
    fn test_rdtsc_overhead() {
        let overhead = measure_rdtsc_overhead();
        println!("RDTSC overhead: {} ticks", overhead);

        // RDTSC should be very fast
        assert!(overhead < 100);
    }
}
