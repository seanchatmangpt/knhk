// rust/knhk-etl/src/slo_monitor.rs
// SLO monitoring and p99 latency tracking
// Tracks latency samples and detects SLO violations per runtime class

extern crate alloc;

use crate::runtime_class::RuntimeClass;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// SLO violation error
#[derive(Debug, Clone)]
pub struct SloViolation {
    /// Runtime class that violated SLO
    pub class: RuntimeClass,
    /// Actual p99 latency in nanoseconds
    pub p99_latency_ns: u64,
    /// SLO threshold in nanoseconds
    pub slo_threshold_ns: u64,
    /// Violation percentage (how much over threshold)
    pub violation_percent: f64,
}

impl SloViolation {
    /// Create new SLO violation
    pub fn new(class: RuntimeClass, p99_latency_ns: u64, slo_threshold_ns: u64) -> Self {
        let violation_percent = if slo_threshold_ns > 0 {
            ((p99_latency_ns as f64 - slo_threshold_ns as f64) / slo_threshold_ns as f64) * 100.0
        } else {
            0.0
        };

        Self {
            class,
            p99_latency_ns,
            slo_threshold_ns,
            violation_percent,
        }
    }

    /// Get error message
    pub fn message(&self) -> String {
        format!(
            "SLO violation: {} p99 latency {}ns exceeds threshold {}ns ({}% over)",
            match self.class {
                RuntimeClass::R1 => "R1",
                RuntimeClass::W1 => "W1",
                RuntimeClass::C1 => "C1",
            },
            self.p99_latency_ns,
            self.slo_threshold_ns,
            self.violation_percent
        )
    }
}

/// SLO monitor for tracking latencies and detecting violations
pub struct SloMonitor {
    /// Rolling window of latency samples (in nanoseconds)
    latency_samples: VecDeque<u64>,
    /// Window size (default: 1000 samples)
    window_size: usize,
    /// Runtime class being monitored
    class: RuntimeClass,
    /// SLO threshold in nanoseconds
    slo_threshold_ns: u64,
}

impl SloMonitor {
    /// Create new SLO monitor
    ///
    /// # Arguments
    /// * `class` - Runtime class to monitor
    /// * `window_size` - Number of samples to keep in rolling window (default: 1000)
    pub fn new(class: RuntimeClass, window_size: usize) -> Self {
        let metadata = class.metadata();
        Self {
            latency_samples: VecDeque::with_capacity(window_size),
            window_size,
            class,
            slo_threshold_ns: metadata.slo_p99_ns,
        }
    }

    /// Record a latency sample
    ///
    /// # Arguments
    /// * `latency_ns` - Latency in nanoseconds
    pub fn record_latency(&mut self, latency_ns: u64) {
        // Add new sample
        self.latency_samples.push_back(latency_ns);

        // Remove oldest sample if window is full
        if self.latency_samples.len() > self.window_size {
            self.latency_samples.pop_front();
        }
    }

    /// Calculate p99 latency from current samples
    ///
    /// # Returns
    /// p99 latency in nanoseconds, or 0 if insufficient samples
    pub fn get_p99_latency(&self) -> u64 {
        if self.latency_samples.is_empty() {
            return 0;
        }

        // Need at least 100 samples for meaningful p99
        if self.latency_samples.len() < 100 {
            return 0;
        }

        // Create sorted copy of samples
        let mut sorted: Vec<u64> = self.latency_samples.iter().copied().collect();
        sorted.sort();

        // Calculate p99 index (99th percentile)
        let p99_index = ((sorted.len() as f64) * 0.99) as usize;
        let p99_index = p99_index.min(sorted.len() - 1);

        sorted[p99_index]
    }

    /// Check for SLO violation
    ///
    /// # Returns
    /// * `Ok(())` - No violation
    /// * `Err(SloViolation)` - SLO threshold exceeded
    pub fn check_slo_violation(&self) -> Result<(), SloViolation> {
        let p99_latency_ns = self.get_p99_latency();

        // No violation if p99 is within threshold
        if p99_latency_ns <= self.slo_threshold_ns {
            return Ok(());
        }

        // Violation detected
        Err(SloViolation::new(
            self.class,
            p99_latency_ns,
            self.slo_threshold_ns,
        ))
    }

    /// Get current sample count
    pub fn sample_count(&self) -> usize {
        self.latency_samples.len()
    }

    /// Get runtime class
    pub fn class(&self) -> RuntimeClass {
        self.class
    }

    /// Get SLO threshold
    pub fn slo_threshold_ns(&self) -> u64 {
        self.slo_threshold_ns
    }

    /// Clear all samples (for testing/reset)
    pub fn clear(&mut self) {
        self.latency_samples.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slo_monitor_r1() {
        let mut monitor = SloMonitor::new(RuntimeClass::R1, 1000);
        assert_eq!(monitor.slo_threshold_ns(), 2); // ≤2 ns SLO

        // Record samples within SLO
        for _ in 0..100 {
            monitor.record_latency(1); // 1ns < 2ns SLO
        }

        assert!(monitor.check_slo_violation().is_ok());
    }

    #[test]
    fn test_slo_monitor_r1_violation() {
        let mut monitor = SloMonitor::new(RuntimeClass::R1, 1000);

        // Record samples exceeding SLO
        for _ in 0..100 {
            monitor.record_latency(5); // 5ns > 2ns SLO
        }

        let violation = monitor.check_slo_violation();
        assert!(violation.is_err());

        if let Err(v) = violation {
            assert_eq!(v.class, RuntimeClass::R1);
            assert!(v.p99_latency_ns > v.slo_threshold_ns);
        }
    }

    #[test]
    fn test_slo_monitor_w1() {
        let mut monitor = SloMonitor::new(RuntimeClass::W1, 1000);
        assert_eq!(monitor.slo_threshold_ns(), 1_000_000); // ≤1 ms SLO

        // Record samples within SLO
        for _ in 0..100 {
            monitor.record_latency(500_000); // 500µs < 1ms SLO
        }

        assert!(monitor.check_slo_violation().is_ok());
    }

    #[test]
    fn test_slo_monitor_c1() {
        let mut monitor = SloMonitor::new(RuntimeClass::C1, 1000);
        assert_eq!(monitor.slo_threshold_ns(), 500_000_000); // ≤500 ms SLO

        // Record samples within SLO
        for _ in 0..100 {
            monitor.record_latency(200_000_000); // 200ms < 500ms SLO
        }

        assert!(monitor.check_slo_violation().is_ok());
    }

    #[test]
    fn test_p99_calculation() {
        let mut monitor = SloMonitor::new(RuntimeClass::R1, 1000);

        // Record 100 samples with known distribution
        for i in 0..100 {
            monitor.record_latency(i as u64);
        }

        let p99 = monitor.get_p99_latency();
        // p99 should be around index 99 (99th percentile)
        assert!(p99 >= 90); // Should be high value
    }

    #[test]
    fn test_window_size_limit() {
        let mut monitor = SloMonitor::new(RuntimeClass::R1, 100);

        // Record more samples than window size
        for i in 0..200 {
            monitor.record_latency(i as u64);
        }

        // Should only keep window_size samples
        assert_eq!(monitor.sample_count(), 100);
    }

    #[test]
    fn test_insufficient_samples() {
        let mut monitor = SloMonitor::new(RuntimeClass::R1, 1000);

        // Record fewer than 100 samples
        for _ in 0..50 {
            monitor.record_latency(100);
        }

        // p99 should return 0 (insufficient samples)
        assert_eq!(monitor.get_p99_latency(), 0);
    }
}
