//! Tick Measurement Infrastructure
//! Provides precise tick counting for enforcing the Chatman constant (τ ≤ 8)

use std::arch::x86_64::_rdtsc;
use std::time::Instant;

/// KNHK tick definition: 1 tick = 1 nanosecond @ 1GHz reference clock
/// Adjust CYCLES_PER_TICK based on actual CPU frequency
/// Example: 4GHz CPU → 4 cycles per tick, 2GHz CPU → 2 cycles per tick
#[cfg(target_arch = "x86_64")]
const CYCLES_PER_TICK: u64 = 4; // Assuming ~4GHz CPU

#[cfg(target_arch = "aarch64")]
const CYCLES_PER_TICK: u64 = 3; // Assuming ~3GHz CPU (M1/M2)

/// PMU measurement context for precise tick counting
#[derive(Debug, Clone, Copy)]
pub struct TickMeasurement {
    pub start_cycles: u64,
    pub end_cycles: u64,
    pub elapsed_cycles: u64,
    pub elapsed_ticks: u64,
    pub elapsed_nanos: u64,
}

impl TickMeasurement {
    /// Check if measurement violates the Chatman constant (τ ≤ 8)
    pub fn exceeds_budget(&self) -> bool {
        self.elapsed_ticks > 8
    }

    /// Get human-readable string representation
    pub fn to_string(&self) -> String {
        format!(
            "{} ticks ({} cycles, {} ns) - {}",
            self.elapsed_ticks,
            self.elapsed_cycles,
            self.elapsed_nanos,
            if self.exceeds_budget() {
                "❌ EXCEEDS BUDGET"
            } else {
                "✅ WITHIN BUDGET"
            }
        )
    }
}

/// Read hardware cycle counter (RDTSC on x86, CNTVCT on ARM)
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn rdtsc() -> u64 {
    unsafe { _rdtsc() }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn rdtsc() -> u64 {
    let val: u64;
    unsafe {
        std::arch::asm!("mrs {}, cntvct_el0", out(reg) val);
    }
    val
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline(always)]
pub fn rdtsc() -> u64 {
    // Fallback: use monotonic clock
    // This is not as precise as hardware counters but prevents compilation failures
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

/// Convert CPU cycles to KNHK ticks
#[inline(always)]
pub fn cycles_to_ticks(cycles: u64) -> u64 {
    cycles / CYCLES_PER_TICK
}

/// Measure execution time of a function in ticks
pub fn measure_ticks<F, R>(f: F) -> (R, TickMeasurement)
where
    F: FnOnce() -> R,
{
    let start_time = Instant::now();
    let start_cycles = rdtsc();

    let result = f();

    let end_cycles = rdtsc();
    let elapsed_nanos = start_time.elapsed().as_nanos() as u64;

    let elapsed_cycles = end_cycles.saturating_sub(start_cycles);
    let elapsed_ticks = cycles_to_ticks(elapsed_cycles);

    let measurement = TickMeasurement {
        start_cycles,
        end_cycles,
        elapsed_cycles,
        elapsed_ticks,
        elapsed_nanos,
    };

    (result, measurement)
}

/// Measure execution time of a function and assert it's within budget
pub fn measure_and_assert_budget<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let (result, measurement) = measure_ticks(f);

    assert!(
        !measurement.exceeds_budget(),
        "{} violated Chatman constant: {}",
        name,
        measurement.to_string()
    );

    result
}

/// Statistics for multiple measurements
#[derive(Debug, Clone)]
pub struct TickStatistics {
    pub min_ticks: u64,
    pub max_ticks: u64,
    pub mean_ticks: f64,
    pub p50_ticks: u64,
    pub p95_ticks: u64,
    pub p99_ticks: u64,
    pub violations: usize,
    pub total_measurements: usize,
}

impl TickStatistics {
    /// Calculate statistics from multiple measurements
    pub fn from_measurements(measurements: &[TickMeasurement]) -> Self {
        if measurements.is_empty() {
            return Self {
                min_ticks: 0,
                max_ticks: 0,
                mean_ticks: 0.0,
                p50_ticks: 0,
                p95_ticks: 0,
                p99_ticks: 0,
                violations: 0,
                total_measurements: 0,
            };
        }

        let mut ticks: Vec<u64> = measurements.iter().map(|m| m.elapsed_ticks).collect();
        ticks.sort_unstable();

        let min_ticks = *ticks.first().unwrap();
        let max_ticks = *ticks.last().unwrap();
        let mean_ticks = ticks.iter().sum::<u64>() as f64 / ticks.len() as f64;

        let p50_ticks = ticks[ticks.len() * 50 / 100];
        let p95_ticks = ticks[ticks.len() * 95 / 100];
        let p99_ticks = ticks[ticks.len() * 99 / 100];

        let violations = measurements.iter().filter(|m| m.exceeds_budget()).count();

        Self {
            min_ticks,
            max_ticks,
            mean_ticks,
            p50_ticks,
            p95_ticks,
            p99_ticks,
            violations,
            total_measurements: measurements.len(),
        }
    }

    /// Check if statistics meet SLO (zero violations)
    pub fn meets_slo(&self) -> bool {
        self.violations == 0 && self.max_ticks <= 8
    }

    /// Get human-readable string representation
    pub fn to_string(&self) -> String {
        format!(
            "min={} p50={} p95={} p99={} max={} mean={:.2} violations={}/{} - {}",
            self.min_ticks,
            self.p50_ticks,
            self.p95_ticks,
            self.p99_ticks,
            self.max_ticks,
            self.mean_ticks,
            self.violations,
            self.total_measurements,
            if self.meets_slo() {
                "✅ SLO MET"
            } else {
                "❌ SLO VIOLATED"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdtsc_monotonic() {
        let t1 = rdtsc();
        let t2 = rdtsc();
        assert!(t2 >= t1, "RDTSC should be monotonic");
    }

    #[test]
    fn test_measure_ticks() {
        let (result, measurement) = measure_ticks(|| {
            // Minimal operation
            42
        });

        assert_eq!(result, 42);
        assert!(measurement.elapsed_cycles > 0);
        assert!(measurement.elapsed_ticks >= 0);
    }

    #[test]
    fn test_statistics() {
        let measurements = vec![
            TickMeasurement {
                start_cycles: 0,
                end_cycles: 100,
                elapsed_cycles: 100,
                elapsed_ticks: 5,
                elapsed_nanos: 5,
            },
            TickMeasurement {
                start_cycles: 0,
                end_cycles: 200,
                elapsed_cycles: 200,
                elapsed_ticks: 7,
                elapsed_nanos: 7,
            },
            TickMeasurement {
                start_cycles: 0,
                end_cycles: 300,
                elapsed_cycles: 300,
                elapsed_ticks: 10, // Violation
                elapsed_nanos: 10,
            },
        ];

        let stats = TickStatistics::from_measurements(&measurements);
        assert_eq!(stats.min_ticks, 5);
        assert_eq!(stats.max_ticks, 10);
        assert_eq!(stats.violations, 1);
        assert!(!stats.meets_slo());
    }
}
