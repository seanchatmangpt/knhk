//! Configuration for the Autonomous Evolution Loop

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the autonomous evolution loop
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomousLoopConfig {
    /// How often to run evolution cycles
    ///
    /// Default: 60 seconds
    #[serde(with = "humantime_serde")]
    pub cycle_interval: Duration,

    /// Minimum patterns required before proposing changes
    ///
    /// This prevents premature evolution with insufficient data.
    /// Default: 5
    pub min_patterns_for_proposal: usize,

    /// Automatically promote snapshots if production-ready
    ///
    /// When true, snapshots that pass validation with production_ready=true
    /// are automatically promoted to production.
    /// Default: true
    pub auto_promote: bool,

    /// Automatically rollback on SLO violation
    ///
    /// When true, the system automatically rolls back to the previous snapshot
    /// if SLO violations are detected after promotion.
    /// Default: true
    pub auto_rollback_on_slo_violation: bool,

    /// Maximum changes per cycle (safety limit)
    ///
    /// This prevents runaway evolution by limiting the number of changes
    /// that can be applied in a single cycle.
    /// Default: 3
    pub max_changes_per_cycle: usize,

    /// Email address for alerts (optional)
    ///
    /// If provided, critical events (promotions, rollbacks, errors) will
    /// trigger email alerts.
    pub alert_email: Option<String>,

    /// Pause evolution if error rate exceeds threshold (optional)
    ///
    /// Error rate is calculated as (failed_cycles / total_cycles) * 100.
    /// If this threshold is exceeded, the loop pauses until manual intervention.
    /// Example: Some(5.0) pauses at 5% error rate
    pub pause_on_error_rate: Option<f64>,

    /// Maximum number of retry attempts for transient failures
    ///
    /// Default: 3
    pub max_retries: u32,

    /// Backoff duration between retries
    ///
    /// Default: 5 seconds
    #[serde(with = "humantime_serde")]
    pub retry_backoff: Duration,
}

impl Default for AutonomousLoopConfig {
    fn default() -> Self {
        Self {
            cycle_interval: Duration::from_secs(60),
            min_patterns_for_proposal: 5,
            auto_promote: true,
            auto_rollback_on_slo_violation: true,
            max_changes_per_cycle: 3,
            alert_email: None,
            pause_on_error_rate: Some(10.0), // Pause at 10% error rate
            max_retries: 3,
            retry_backoff: Duration::from_secs(5),
        }
    }
}

impl AutonomousLoopConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set cycle interval
    pub fn with_cycle_interval(mut self, interval: Duration) -> Self {
        self.cycle_interval = interval;
        self
    }

    /// Set minimum patterns for proposal
    pub fn with_min_patterns(mut self, min: usize) -> Self {
        self.min_patterns_for_proposal = min;
        self
    }

    /// Enable/disable auto-promotion
    pub fn with_auto_promote(mut self, enabled: bool) -> Self {
        self.auto_promote = enabled;
        self
    }

    /// Set alert email
    pub fn with_alert_email(mut self, email: String) -> Self {
        self.alert_email = Some(email);
        self
    }

    /// Set error rate threshold for pausing
    pub fn with_error_threshold(mut self, threshold: f64) -> Self {
        self.pause_on_error_rate = Some(threshold);
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> crate::Result<()> {
        if self.cycle_interval < Duration::from_secs(1) {
            return Err(crate::AutonomousLoopError::ConfigurationError(
                "cycle_interval must be at least 1 second".to_string(),
            ));
        }

        if self.min_patterns_for_proposal == 0 {
            return Err(crate::AutonomousLoopError::ConfigurationError(
                "min_patterns_for_proposal must be greater than 0".to_string(),
            ));
        }

        if self.max_changes_per_cycle == 0 {
            return Err(crate::AutonomousLoopError::ConfigurationError(
                "max_changes_per_cycle must be greater than 0".to_string(),
            ));
        }

        if let Some(threshold) = self.pause_on_error_rate {
            if threshold <= 0.0 || threshold > 100.0 {
                return Err(crate::AutonomousLoopError::ConfigurationError(
                    "pause_on_error_rate must be between 0 and 100".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AutonomousLoopConfig::default();
        assert_eq!(config.cycle_interval, Duration::from_secs(60));
        assert_eq!(config.min_patterns_for_proposal, 5);
        assert!(config.auto_promote);
        assert!(config.auto_rollback_on_slo_violation);
        assert_eq!(config.max_changes_per_cycle, 3);
    }

    #[test]
    fn test_builder_pattern() {
        let config = AutonomousLoopConfig::new()
            .with_cycle_interval(Duration::from_secs(30))
            .with_min_patterns(10)
            .with_auto_promote(false)
            .with_alert_email("admin@example.com".to_string());

        assert_eq!(config.cycle_interval, Duration::from_secs(30));
        assert_eq!(config.min_patterns_for_proposal, 10);
        assert!(!config.auto_promote);
        assert_eq!(
            config.alert_email,
            Some("admin@example.com".to_string())
        );
    }

    #[test]
    fn test_validation_success() {
        let config = AutonomousLoopConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_interval() {
        let config = AutonomousLoopConfig::default()
            .with_cycle_interval(Duration::from_millis(500));
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_min_patterns() {
        let config = AutonomousLoopConfig {
            min_patterns_for_proposal: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_error_threshold() {
        let config = AutonomousLoopConfig::default().with_error_threshold(150.0);
        assert!(config.validate().is_err());
    }
}
