// Cycle counter utilities for tick metering
// Platform-specific cycle counting (RDTSC on x86, CNTVCT on ARM)

/// Read CPU cycle counter (platform-specific)
///
/// Returns raw CPU cycle count from hardware counter.
/// On x86-64: uses RDTSC instruction
/// On ARM64: uses CNTVCT_EL0 register
/// Fallback: returns 0 (will trigger validation failure)
#[inline(always)]
pub fn read_cycles() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        unsafe { core::arch::x86_64::_rdtsc() }
    }
    #[cfg(target_arch = "aarch64")]
    {
        let val: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) val);
        }
        val
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        // Fallback: return 0 (will trigger validation failure)
        0
    }
}

/// Read CPU cycle counter with serialization (more precise, higher overhead)
///
/// Uses serializing instructions to prevent out-of-order execution from skewing measurements.
/// On x86-64: uses RDTSCP + LFENCE
/// On ARM64: uses DSB + CNTVCT_EL0 + DSB
#[inline(always)]
pub fn read_cycles_precise() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        unsafe {
            let mut aux = 0u32;
            let cycles = core::arch::x86_64::__rdtscp(&mut aux);
            core::arch::x86_64::_mm_lfence(); // Load fence
            cycles
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        unsafe {
            core::arch::asm!("dsb sy", options(nostack, nomem));
            let val: u64;
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) val);
            core::arch::asm!("dsb sy", options(nostack, nomem));
            val
        }
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        // Fallback: return 0 (will trigger validation failure)
        0
    }
}

/// Convert CPU cycles to KNHK ticks
///
/// 1 tick = 1 nanosecond @ 1GHz reference clock
/// Adjust CYCLES_PER_TICK based on actual CPU frequency
/// Example: 4GHz CPU → 4 cycles per tick, 2GHz CPU → 2 cycles per tick
///
/// Default: assumes 4GHz CPU (4 cycles per tick)
const CYCLES_PER_TICK: u64 = 4;

/// Convert cycles to ticks
#[inline(always)]
pub fn cycles_to_ticks(cycles: u64) -> u32 {
    (cycles / CYCLES_PER_TICK) as u32
}

/// Tick measurement context for μ execution
///
/// Records cycle counts at μ entry and exit to compute actual ticks.
pub struct TickMeasurement {
    start_cycles: u64,
    end_cycles: Option<u64>,
}

impl TickMeasurement {
    /// Start tick measurement at μ entry
    #[inline(always)]
    pub fn start() -> Self {
        Self {
            start_cycles: read_cycles(),
            end_cycles: None,
        }
    }

    /// Start tick measurement with precise serialization
    #[inline(always)]
    pub fn start_precise() -> Self {
        Self {
            start_cycles: read_cycles_precise(),
            end_cycles: None,
        }
    }

    /// Stop tick measurement at receipt finalize
    #[inline(always)]
    pub fn stop(&mut self) {
        self.end_cycles = Some(read_cycles());
    }

    /// Stop tick measurement with precise serialization
    #[inline(always)]
    pub fn stop_precise(&mut self) {
        self.end_cycles = Some(read_cycles_precise());
    }

    /// Get elapsed ticks (must call stop() first)
    ///
    /// Returns None if stop() hasn't been called.
    pub fn elapsed_ticks(&self) -> Option<u32> {
        self.end_cycles.map(|end| {
            let cycles = end.saturating_sub(self.start_cycles);
            cycles_to_ticks(cycles)
        })
    }

    /// Get elapsed cycles (must call stop() first)
    pub fn elapsed_cycles(&self) -> Option<u64> {
        self.end_cycles
            .map(|end| end.saturating_sub(self.start_cycles))
    }

    /// Check if measurement is complete (stop() has been called)
    pub fn is_complete(&self) -> bool {
        self.end_cycles.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_measurement() {
        let mut measurement = TickMeasurement::start();
        // Simulate some work
        let _ = read_cycles();
        measurement.stop();

        let ticks = measurement.elapsed_ticks();
        assert!(ticks.is_some(), "Measurement should be complete");
        assert!(
            measurement.is_complete(),
            "Measurement should be marked complete"
        );
    }

    #[test]
    fn test_cycles_to_ticks() {
        // 4 cycles = 1 tick at 4GHz
        assert_eq!(cycles_to_ticks(4), 1);
        assert_eq!(cycles_to_ticks(8), 2);
        assert_eq!(cycles_to_ticks(32), 8);
    }
}
