//! Precision timing using CPU cycle counters (RDTSC)
//!
//! Provides sub-nanosecond precision measurement for hot path operations.

use crate::ChicagoResult;
use std::time::Instant;

/// Tick measurement result
#[derive(Debug, Clone, Copy)]
pub struct TickMeasurement {
    /// Raw tick count or nanoseconds
    pub value: u64,
    /// Whether this is ticks (true) or nanoseconds (false)
    pub is_ticks: bool,
}

/// Precision timer using RDTSC on x86_64 or fallback to Instant
pub struct PrecisionTimer {
    /// Overhead in ticks (calibrated on initialization)
    overhead: u64,
    /// CPU frequency for tick-to-ns conversion
    #[allow(dead_code)]
    cpu_freq_mhz: f64,
    /// Whether RDTSC is available
    rdtsc_available: bool,
}

impl PrecisionTimer {
    /// Create and calibrate a new precision timer
    pub fn new() -> ChicagoResult<Self> {
        let rdtsc_available = Self::check_rdtsc_available();

        let cpu_freq_mhz = if rdtsc_available {
            Self::estimate_cpu_frequency()
        } else {
            0.0
        };

        let mut timer = Self {
            overhead: 0,
            cpu_freq_mhz,
            rdtsc_available,
        };

        // Calibrate overhead
        timer.calibrate_overhead()?;

        Ok(timer)
    }

    /// Check if RDTSC instruction is available
    #[cfg(target_arch = "x86_64")]
    fn check_rdtsc_available() -> bool {
        use raw_cpuid::CpuId;
        let cpuid = CpuId::new();
        cpuid
            .get_feature_info()
            .map(|f| f.has_tsc())
            .unwrap_or(false)
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_rdtsc_available() -> bool {
        false
    }

    /// Estimate CPU frequency by measuring RDTSC over a known time interval
    #[cfg(target_arch = "x86_64")]
    fn estimate_cpu_frequency() -> f64 {
        use std::thread;
        use std::time::Duration;

        let start_tsc = unsafe { Self::rdtsc() };
        let start_time = Instant::now();

        thread::sleep(Duration::from_millis(100));

        let end_tsc = unsafe { Self::rdtsc() };
        let elapsed = start_time.elapsed();

        let ticks = end_tsc.saturating_sub(start_tsc);
        let elapsed_ns = elapsed.as_nanos() as f64;

        // MHz = (ticks / ns) * 1000
        (ticks as f64 / elapsed_ns) * 1000.0
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn estimate_cpu_frequency() -> f64 {
        0.0
    }

    /// Read the Time Stamp Counter (TSC) using RDTSC instruction
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn rdtsc() -> u64 {
        std::arch::x86_64::_rdtsc()
    }

    #[cfg(not(target_arch = "x86_64"))]
    #[inline(always)]
    unsafe fn rdtsc() -> u64 {
        0
    }

    /// Calibrate measurement overhead by measuring empty operations
    fn calibrate_overhead(&mut self) -> ChicagoResult<()> {
        if !self.rdtsc_available {
            self.overhead = 0;
            return Ok(());
        }

        const CALIBRATION_SAMPLES: usize = 1000;
        let mut overheads = Vec::with_capacity(CALIBRATION_SAMPLES);

        for _ in 0..CALIBRATION_SAMPLES {
            let start = unsafe { Self::rdtsc() };
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            let end = unsafe { Self::rdtsc() };

            let overhead = end.saturating_sub(start);
            overheads.push(overhead);
        }

        // Use median overhead to avoid outliers
        overheads.sort_unstable();
        self.overhead = overheads[CALIBRATION_SAMPLES / 2];

        Ok(())
    }

    /// Get calibrated overhead in ticks
    pub fn overhead_ticks(&self) -> u64 {
        self.overhead
    }

    /// Measure operation in CPU ticks (for hot path)
    #[inline]
    pub fn measure_ticks<F, T>(&self, mut operation: F) -> TickMeasurement
    where
        F: FnMut() -> T,
    {
        if !self.rdtsc_available {
            // Fallback to nanosecond measurement
            return self.measure_nanos(operation);
        }

        // Warm up to ensure code is in cache
        let _ = operation();

        // Actual measurement with compiler fences to prevent reordering
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        let start = unsafe { Self::rdtsc() };
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);

        let _ = operation();

        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        let end = unsafe { Self::rdtsc() };
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);

        let raw_ticks = end.saturating_sub(start);
        let adjusted_ticks = raw_ticks.saturating_sub(self.overhead);

        TickMeasurement {
            value: adjusted_ticks,
            is_ticks: true,
        }
    }

    /// Measure operation in nanoseconds (for warm/cold path)
    #[inline]
    pub fn measure_nanos<F, T>(&self, mut operation: F) -> TickMeasurement
    where
        F: FnMut() -> T,
    {
        // Warm up
        let _ = operation();

        // Actual measurement
        let start = Instant::now();
        let _ = operation();
        let elapsed = start.elapsed();

        TickMeasurement {
            value: elapsed.as_nanos() as u64,
            is_ticks: false,
        }
    }

    /// Measure multiple iterations and return all measurements
    pub fn measure_iterations<F, T>(
        &self,
        iterations: usize,
        use_ticks: bool,
        mut operation: F,
    ) -> Vec<u64>
    where
        F: FnMut() -> T,
    {
        let mut measurements = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let measurement = if use_ticks {
                self.measure_ticks(&mut operation)
            } else {
                self.measure_nanos(&mut operation)
            };
            measurements.push(measurement.value);
        }

        measurements
    }

    /// Convert ticks to nanoseconds (approximate)
    pub fn ticks_to_nanos(&self, ticks: u64) -> u64 {
        if self.cpu_freq_mhz == 0.0 {
            return ticks; // Fallback: assume 1 tick ≈ 1 ns
        }

        // ns = ticks / (MHz / 1000)
        (ticks as f64 / (self.cpu_freq_mhz / 1000.0)) as u64
    }

    /// Convert nanoseconds to ticks (approximate)
    pub fn nanos_to_ticks(&self, nanos: u64) -> u64 {
        if self.cpu_freq_mhz == 0.0 {
            return nanos; // Fallback: assume 1 ns ≈ 1 tick
        }

        // ticks = ns * (MHz / 1000)
        (nanos as f64 * (self.cpu_freq_mhz / 1000.0)) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = PrecisionTimer::new();
        assert!(timer.is_ok());
    }

    #[test]
    fn test_tick_measurement() {
        let timer = PrecisionTimer::new().unwrap();

        let measurement = timer.measure_ticks(|| {
            // Very fast operation
            42
        });

        // Should be very small (likely < 100 ticks even on slow systems)
        assert!(
            measurement.value < 1000,
            "Measurement: {}",
            measurement.value
        );
        assert!(measurement.is_ticks);
    }

    #[test]
    fn test_nano_measurement() {
        let timer = PrecisionTimer::new().unwrap();

        let measurement = timer.measure_nanos(|| {
            // Very fast operation
            42
        });

        // Should be small but measurable
        assert!(
            measurement.value < 1_000_000,
            "Measurement: {}ns",
            measurement.value
        );
        assert!(!measurement.is_ticks);
    }

    #[test]
    fn test_overhead_calibration() {
        let timer = PrecisionTimer::new().unwrap();

        // Overhead should be reasonable (< 100 ticks)
        let overhead = timer.overhead_ticks();
        assert!(overhead < 100, "Overhead: {} ticks", overhead);
    }

    #[test]
    fn test_multiple_iterations() {
        let timer = PrecisionTimer::new().unwrap();

        let measurements = timer.measure_iterations(100, true, || 42);

        assert_eq!(measurements.len(), 100);
        // All measurements should be small and non-zero (after overhead adjustment)
        for &measurement in &measurements {
            assert!(measurement < 1000, "Measurement: {}", measurement);
        }
    }
}
