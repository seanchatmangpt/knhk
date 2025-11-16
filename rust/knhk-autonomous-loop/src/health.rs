//! Health monitoring for the Autonomous Evolution Loop

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Health status of the evolution loop
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LoopHealth {
    /// Loop is running normally
    Running,

    /// Loop is paused (manual or automatic)
    Paused { reason: String },

    /// Loop encountered an error
    Error {
        error: String,
        retry_count: u32,
        last_error_time: Option<SystemTime>,
    },

    /// Loop has been stopped
    Stopped,
}

impl LoopHealth {
    /// Check if loop is healthy (running)
    pub fn is_healthy(&self) -> bool {
        matches!(self, LoopHealth::Running)
    }

    /// Check if loop is paused
    pub fn is_paused(&self) -> bool {
        matches!(self, LoopHealth::Paused { .. })
    }

    /// Check if loop has errors
    pub fn is_error(&self) -> bool {
        matches!(self, LoopHealth::Error { .. })
    }

    /// Check if loop is stopped
    pub fn is_stopped(&self) -> bool {
        matches!(self, LoopHealth::Stopped)
    }

    /// Get health status as string
    pub fn status(&self) -> &str {
        match self {
            LoopHealth::Running => "running",
            LoopHealth::Paused { .. } => "paused",
            LoopHealth::Error { .. } => "error",
            LoopHealth::Stopped => "stopped",
        }
    }

    /// Get detailed status message
    pub fn message(&self) -> String {
        match self {
            LoopHealth::Running => "Loop is running normally".to_string(),
            LoopHealth::Paused { reason } => format!("Loop is paused: {}", reason),
            LoopHealth::Error {
                error,
                retry_count,
                ..
            } => {
                format!("Loop error (retry {}): {}", retry_count, error)
            }
            LoopHealth::Stopped => "Loop has been stopped".to_string(),
        }
    }
}

/// Health statistics for monitoring
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStats {
    /// Total cycles executed
    pub total_cycles: u64,

    /// Successful cycles
    pub successful_cycles: u64,

    /// Failed cycles
    pub failed_cycles: u64,

    /// Partial success cycles
    pub partial_cycles: u64,

    /// No-change cycles
    pub no_change_cycles: u64,

    /// Current error rate (percentage)
    pub error_rate: f64,

    /// Average cycle duration (milliseconds)
    pub avg_cycle_duration_ms: u64,

    /// Last successful cycle time
    pub last_success_time: Option<SystemTime>,

    /// Last error time
    pub last_error_time: Option<SystemTime>,

    /// Uptime since loop started
    pub uptime_seconds: u64,
}

impl Default for HealthStats {
    fn default() -> Self {
        Self {
            total_cycles: 0,
            successful_cycles: 0,
            failed_cycles: 0,
            partial_cycles: 0,
            no_change_cycles: 0,
            error_rate: 0.0,
            avg_cycle_duration_ms: 0,
            last_success_time: None,
            last_error_time: None,
            uptime_seconds: 0,
        }
    }
}

impl HealthStats {
    /// Update stats after a cycle completes
    pub fn record_cycle_result(
        &mut self,
        result: &crate::cycle::CycleResult,
        duration_ms: u64,
    ) {
        self.total_cycles += 1;

        match result {
            crate::cycle::CycleResult::Success { .. } => {
                self.successful_cycles += 1;
                self.last_success_time = Some(SystemTime::now());
            }
            crate::cycle::CycleResult::PartialSuccess { .. } => {
                self.partial_cycles += 1;
            }
            crate::cycle::CycleResult::NoChange { .. } => {
                self.no_change_cycles += 1;
            }
            crate::cycle::CycleResult::Failure { .. } => {
                self.failed_cycles += 1;
                self.last_error_time = Some(SystemTime::now());
            }
        }

        // Update error rate
        if self.total_cycles > 0 {
            self.error_rate =
                (self.failed_cycles as f64 / self.total_cycles as f64) * 100.0;
        }

        // Update average duration
        let total_duration =
            self.avg_cycle_duration_ms * (self.total_cycles - 1) + duration_ms;
        self.avg_cycle_duration_ms = total_duration / self.total_cycles;
    }

    /// Check if error rate exceeds threshold
    pub fn exceeds_error_threshold(&self, threshold: Option<f64>) -> bool {
        if let Some(threshold) = threshold {
            self.error_rate > threshold && self.total_cycles >= 10
        } else {
            false
        }
    }

    /// Get success rate (percentage)
    pub fn success_rate(&self) -> f64 {
        if self.total_cycles == 0 {
            0.0
        } else {
            (self.successful_cycles as f64 / self.total_cycles as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cycle::CycleResult;

    #[test]
    fn test_health_status_checks() {
        assert!(LoopHealth::Running.is_healthy());
        assert!(!LoopHealth::Running.is_paused());
        assert!(!LoopHealth::Running.is_error());
        assert!(!LoopHealth::Running.is_stopped());

        let paused = LoopHealth::Paused {
            reason: "manual".to_string(),
        };
        assert!(!paused.is_healthy());
        assert!(paused.is_paused());

        let error = LoopHealth::Error {
            error: "test".to_string(),
            retry_count: 1,
            last_error_time: None,
        };
        assert!(error.is_error());

        assert!(LoopHealth::Stopped.is_stopped());
    }

    #[test]
    fn test_health_stats_default() {
        let stats = HealthStats::default();
        assert_eq!(stats.total_cycles, 0);
        assert_eq!(stats.error_rate, 0.0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_health_stats_success() {
        let mut stats = HealthStats::default();

        let result = CycleResult::Success {
            new_snapshot_id: [0u8; 32],
            duration_ms: 1000,
        };

        stats.record_cycle_result(&result, 1000);

        assert_eq!(stats.total_cycles, 1);
        assert_eq!(stats.successful_cycles, 1);
        assert_eq!(stats.error_rate, 0.0);
        assert_eq!(stats.success_rate(), 100.0);
        assert!(stats.last_success_time.is_some());
    }

    #[test]
    fn test_health_stats_failure() {
        let mut stats = HealthStats::default();

        let result = CycleResult::Failure {
            error: "test error".to_string(),
            rollback_performed: false,
        };

        stats.record_cycle_result(&result, 500);

        assert_eq!(stats.total_cycles, 1);
        assert_eq!(stats.failed_cycles, 1);
        assert_eq!(stats.error_rate, 100.0);
        assert!(stats.last_error_time.is_some());
    }

    #[test]
    fn test_error_threshold() {
        let mut stats = HealthStats::default();

        // Need at least 10 cycles for threshold check
        for _ in 0..8 {
            stats.record_cycle_result(
                &CycleResult::Success {
                    new_snapshot_id: [0u8; 32],
                    duration_ms: 1000,
                },
                1000,
            );
        }

        for _ in 0..2 {
            stats.record_cycle_result(
                &CycleResult::Failure {
                    error: "test".to_string(),
                    rollback_performed: false,
                },
                500,
            );
        }

        assert_eq!(stats.total_cycles, 10);
        assert_eq!(stats.error_rate, 20.0);
        assert!(stats.exceeds_error_threshold(Some(15.0)));
        assert!(!stats.exceeds_error_threshold(Some(25.0)));
        assert!(!stats.exceeds_error_threshold(None));
    }

    #[test]
    fn test_avg_duration() {
        let mut stats = HealthStats::default();

        stats.record_cycle_result(
            &CycleResult::Success {
                new_snapshot_id: [0u8; 32],
                duration_ms: 1000,
            },
            1000,
        );

        stats.record_cycle_result(
            &CycleResult::Success {
                new_snapshot_id: [0u8; 32],
                duration_ms: 2000,
            },
            2000,
        );

        assert_eq!(stats.avg_cycle_duration_ms, 1500);
    }
}
