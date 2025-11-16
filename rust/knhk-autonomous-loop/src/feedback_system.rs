//! Feedback system - observes metrics and decides when to trigger evolution

use crate::{CurrentMetrics, DetectedPatterns, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument};

/// Observes metrics and decides when to trigger ontology evolution
///
/// The feedback system monitors various signals:
/// - Schema mismatches (drift detected)
/// - Guard violations (policy failures)
/// - Performance regressions
/// - New pattern detections
pub struct FeedbackSystem {
    /// Thresholds for triggering changes
    thresholds: FeedbackThresholds,

    /// Current metrics (updated by external systems)
    metrics: Arc<RwLock<CurrentMetrics>>,
}

/// Thresholds that determine when evolution is triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackThresholds {
    /// Trigger if schema mismatches exceed this count
    pub schema_mismatch_count: u32,

    /// Trigger if guard violations exceed this count
    pub guard_violation_count: u32,

    /// Trigger if performance regression detected
    pub performance_regression: bool,

    /// Trigger if new patterns detected
    pub new_patterns_detected: bool,

    /// Trigger if error rate exceeds this threshold
    pub error_rate_threshold: f64,
}

impl Default for FeedbackThresholds {
    fn default() -> Self {
        Self {
            schema_mismatch_count: 10,
            guard_violation_count: 5,
            performance_regression: true,
            new_patterns_detected: true,
            error_rate_threshold: 0.05, // 5% error rate
        }
    }
}

/// Reason why evolution was triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerReason {
    /// No triggers detected
    None,

    /// Schema drift detected
    SchemaDrift,

    /// Guard violations detected
    GuardViolations,

    /// Performance regression detected
    PerformanceRegression,

    /// New patterns detected
    NewPatterns,

    /// Error rate exceeded threshold
    HighErrorRate,

    /// Multiple triggers
    Multiple(Vec<TriggerReason>),
}

impl FeedbackSystem {
    /// Create a new feedback system with default thresholds
    #[instrument]
    pub async fn new() -> Result<Self> {
        Self::with_thresholds(FeedbackThresholds::default()).await
    }

    /// Create a new feedback system with custom thresholds
    #[instrument(skip(thresholds))]
    pub async fn with_thresholds(thresholds: FeedbackThresholds) -> Result<Self> {
        Ok(Self {
            thresholds,
            metrics: Arc::new(RwLock::new(CurrentMetrics::default())),
        })
    }

    /// Check if evolution should be triggered
    #[instrument(skip(self))]
    pub async fn should_trigger_evolution(&self) -> Result<TriggerReason> {
        let metrics = self.metrics.read().await;

        let mut reasons = Vec::new();

        // Check schema drift
        if metrics.schema_mismatches >= self.thresholds.schema_mismatch_count {
            debug!(
                count = metrics.schema_mismatches,
                threshold = self.thresholds.schema_mismatch_count,
                "Schema drift detected"
            );
            reasons.push(TriggerReason::SchemaDrift);
        }

        // Check guard violations
        if metrics.guard_violations >= self.thresholds.guard_violation_count {
            debug!(
                count = metrics.guard_violations,
                threshold = self.thresholds.guard_violation_count,
                "Guard violations detected"
            );
            reasons.push(TriggerReason::GuardViolations);
        }

        // Check performance regression
        if self.thresholds.performance_regression && metrics.performance_regression_detected {
            debug!("Performance regression detected");
            reasons.push(TriggerReason::PerformanceRegression);
        }

        // Check new patterns
        if self.thresholds.new_patterns_detected && !metrics.new_patterns.is_empty() {
            debug!(
                pattern_count = metrics.new_patterns.count(),
                "New patterns detected"
            );
            reasons.push(TriggerReason::NewPatterns);
        }

        // Check error rate
        if metrics.error_rate >= self.thresholds.error_rate_threshold {
            debug!(
                error_rate = metrics.error_rate,
                threshold = self.thresholds.error_rate_threshold,
                "High error rate detected"
            );
            reasons.push(TriggerReason::HighErrorRate);
        }

        // Return result
        if reasons.is_empty() {
            Ok(TriggerReason::None)
        } else if reasons.len() == 1 {
            Ok(reasons.into_iter().next().unwrap())
        } else {
            Ok(TriggerReason::Multiple(reasons))
        }
    }

    /// Update current metrics (called by external systems)
    #[instrument(skip(self, metrics))]
    pub async fn update_metrics(&self, metrics: CurrentMetrics) -> Result<()> {
        let mut current = self.metrics.write().await;
        *current = metrics;
        debug!("Metrics updated");
        Ok(())
    }

    /// Get current metrics (for monitoring)
    pub async fn get_metrics(&self) -> CurrentMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset metrics to zero
    #[instrument(skip(self))]
    pub async fn reset_metrics(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        *metrics = CurrentMetrics::default();
        debug!("Metrics reset");
        Ok(())
    }

    /// Increment schema mismatch counter
    pub async fn record_schema_mismatch(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.schema_mismatches += 1;
        Ok(())
    }

    /// Increment guard violation counter
    pub async fn record_guard_violation(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.guard_violations += 1;
        Ok(())
    }

    /// Record performance regression
    pub async fn record_performance_regression(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.performance_regression_detected = true;
        Ok(())
    }

    /// Add new detected patterns
    pub async fn record_new_patterns(&self, patterns: DetectedPatterns) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.new_patterns = patterns;
        Ok(())
    }

    /// Update error rate
    pub async fn update_error_rate(&self, error_rate: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.error_rate = error_rate;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_triggers() {
        let feedback = FeedbackSystem::new().await.unwrap();
        let reason = feedback.should_trigger_evolution().await.unwrap();
        assert!(matches!(reason, TriggerReason::None));
    }

    #[tokio::test]
    async fn test_schema_drift_trigger() {
        let feedback = FeedbackSystem::new().await.unwrap();

        // Record schema mismatches
        for _ in 0..15 {
            feedback.record_schema_mismatch().await.unwrap();
        }

        let reason = feedback.should_trigger_evolution().await.unwrap();
        assert!(matches!(reason, TriggerReason::SchemaDrift));
    }

    #[tokio::test]
    async fn test_guard_violation_trigger() {
        let feedback = FeedbackSystem::new().await.unwrap();

        // Record guard violations
        for _ in 0..10 {
            feedback.record_guard_violation().await.unwrap();
        }

        let reason = feedback.should_trigger_evolution().await.unwrap();
        assert!(matches!(reason, TriggerReason::GuardViolations));
    }

    #[tokio::test]
    async fn test_multiple_triggers() {
        let feedback = FeedbackSystem::new().await.unwrap();

        // Trigger multiple conditions
        for _ in 0..15 {
            feedback.record_schema_mismatch().await.unwrap();
        }
        feedback.record_performance_regression().await.unwrap();

        let reason = feedback.should_trigger_evolution().await.unwrap();
        assert!(matches!(reason, TriggerReason::Multiple(_)));

        if let TriggerReason::Multiple(reasons) = reason {
            assert_eq!(reasons.len(), 2);
        }
    }

    #[tokio::test]
    async fn test_metrics_reset() {
        let feedback = FeedbackSystem::new().await.unwrap();

        feedback.record_schema_mismatch().await.unwrap();
        feedback.reset_metrics().await.unwrap();

        let metrics = feedback.get_metrics().await;
        assert_eq!(metrics.schema_mismatches, 0);
    }
}
