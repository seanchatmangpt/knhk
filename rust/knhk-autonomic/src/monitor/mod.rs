//! # Monitor Component - Continuous Observation
//!
//! **Covenant 3**: Feedback loops run at machine speed
//!
//! The Monitor component continuously collects metrics, events, and observations
//! from workflow execution. It detects anomalies in real-time and triggers
//! analysis when problems are detected.
//!
//! ## Responsibilities
//!
//! - Collect performance, reliability, resource, quality, and security metrics
//! - Detect anomalies (values exceeding thresholds)
//! - Calculate trend directions (improving, degrading, stable)
//! - Emit observations for events (task failures, timeouts, etc.)
//! - Run at ≤8 ticks for hot path operations
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_autonomic::monitor::MonitoringComponent;
//! use knhk_autonomic::types::MetricType;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut monitor = MonitoringComponent::new();
//!
//! // Register metric
//! monitor.register_metric(
//!     "Payment Latency",
//!     MetricType::Performance,
//!     2000.0, // expected value
//!     3000.0, // anomaly threshold
//!     "milliseconds"
//! ).await?;
//!
//! // Collect metrics
//! let metrics = monitor.collect_metrics().await?;
//!
//! // Detect anomalies
//! let anomalies = monitor.detect_anomalies(&metrics).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::{AutonomicError, Result};
use crate::types::{EventType, Metric, MetricType, Observation, Severity, TrendDirection};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};
use uuid::Uuid;

/// Monitoring component for collecting metrics and detecting anomalies
#[derive(Debug, Clone)]
pub struct MonitoringComponent {
    /// Registered metrics
    metrics: Arc<RwLock<HashMap<String, Metric>>>,

    /// Historical metric values for trend analysis
    history: Arc<RwLock<HashMap<String, Vec<f64>>>>,

    /// Maximum history size per metric
    max_history_size: usize,
}

impl Default for MonitoringComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitoringComponent {
    /// Create a new monitoring component
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
            max_history_size: 100,
        }
    }

    /// Register a new metric to monitor
    #[instrument(skip(self))]
    pub async fn register_metric(
        &mut self,
        name: impl Into<String>,
        metric_type: MetricType,
        expected_value: f64,
        anomaly_threshold: f64,
        unit: impl Into<String>,
    ) -> Result<Uuid> {
        let name = name.into();
        let metric = Metric {
            id: Uuid::new_v4(),
            name: name.clone(),
            metric_type,
            current_value: expected_value,
            expected_value,
            unit: unit.into(),
            anomaly_threshold,
            is_anomalous: false,
            trend: TrendDirection::Stable,
            timestamp: Utc::now(),
        };

        let id = metric.id;
        let mut metrics = self.metrics.write().await;
        metrics.insert(name, metric);

        debug!("Registered metric: {}", id);
        Ok(id)
    }

    /// Update metric value
    #[instrument(skip(self))]
    pub async fn update_metric(&mut self, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let metric = metrics
            .get_mut(name)
            .ok_or_else(|| AutonomicError::Monitor(format!("Metric not found: {}", name)))?;

        // Update value
        metric.current_value = value;
        metric.timestamp = Utc::now();

        // Check for anomaly
        metric.is_anomalous = value > metric.anomaly_threshold;

        // Update history for trend analysis
        let mut history = self.history.write().await;
        let values = history.entry(name.to_string()).or_insert_with(Vec::new);
        values.push(value);

        // Keep history bounded
        if values.len() > self.max_history_size {
            values.remove(0);
        }

        // Calculate trend
        metric.trend = self.calculate_trend(values);

        if metric.is_anomalous {
            warn!(
                "Metric {} is anomalous: {} > {}",
                name, value, metric.anomaly_threshold
            );
        }

        Ok(())
    }

    /// Collect all current metrics
    #[instrument(skip(self))]
    pub async fn collect_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.values().cloned().collect())
    }

    /// Detect anomalies in collected metrics
    #[instrument(skip(self, metrics))]
    pub async fn detect_anomalies(&self, metrics: &[Metric]) -> Result<Vec<Observation>> {
        let mut observations = Vec::new();

        for metric in metrics {
            if metric.is_anomalous {
                let severity = self.calculate_severity(metric);

                observations.push(Observation {
                    id: Uuid::new_v4(),
                    observed_at: Utc::now(),
                    event_type: EventType::AnomalyDetected,
                    severity,
                    observed_element: metric.name.clone(),
                    context: serde_json::json!({
                        "metric_type": format!("{:?}", metric.metric_type),
                        "current_value": metric.current_value,
                        "expected_value": metric.expected_value,
                        "threshold": metric.anomaly_threshold,
                        "trend": format!("{:?}", metric.trend),
                    }),
                });
            }
        }

        debug!("Detected {} anomalies", observations.len());
        Ok(observations)
    }

    /// Create observation for an event
    #[instrument(skip(self))]
    pub async fn observe_event(
        &self,
        event_type: EventType,
        severity: Severity,
        element: impl Into<String>,
        context: serde_json::Value,
    ) -> Result<Observation> {
        Ok(Observation {
            id: Uuid::new_v4(),
            observed_at: Utc::now(),
            event_type,
            severity,
            observed_element: element.into(),
            context,
        })
    }

    /// Calculate trend direction from historical values
    fn calculate_trend(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let window_size = values.len().min(10);
        let recent = &values[values.len() - window_size..];

        // Simple linear regression slope
        let n = recent.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean: f64 = recent.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in recent.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        let slope = if denominator != 0.0 {
            numerator / denominator
        } else {
            0.0
        };

        // Classify trend based on slope
        if slope > 0.01 {
            TrendDirection::Degrading
        } else if slope < -0.01 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }

    /// Calculate severity based on how much value exceeds threshold
    fn calculate_severity(&self, metric: &Metric) -> Severity {
        let ratio = metric.current_value / metric.anomaly_threshold;

        if ratio > 2.0 {
            Severity::Critical
        } else if ratio > 1.5 {
            Severity::High
        } else if ratio > 1.2 {
            Severity::Medium
        } else {
            Severity::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_update_metric() {
        let mut monitor = MonitoringComponent::new();

        // Register metric
        let id = monitor
            .register_metric("test_metric", MetricType::Performance, 100.0, 150.0, "ms")
            .await
            .unwrap();

        assert!(id.as_u128() > 0);

        // Update metric
        monitor.update_metric("test_metric", 120.0).await.unwrap();

        // Collect metrics
        let metrics = monitor.collect_metrics().await.unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].current_value, 120.0);
        assert!(!metrics[0].is_anomalous);
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let mut monitor = MonitoringComponent::new();

        monitor
            .register_metric("test_metric", MetricType::Performance, 100.0, 150.0, "ms")
            .await
            .unwrap();

        // Update with anomalous value
        monitor.update_metric("test_metric", 200.0).await.unwrap();

        // Collect and detect anomalies
        let metrics = monitor.collect_metrics().await.unwrap();
        let anomalies = monitor.detect_anomalies(&metrics).await.unwrap();

        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].event_type, EventType::AnomalyDetected);
    }
}
