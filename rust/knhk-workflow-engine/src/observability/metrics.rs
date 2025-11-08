//! Metrics collection for workflow engine

use crate::case::CaseId;
use knhk_otel::{Metric, MetricValue};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Workflow metrics collector
pub struct MetricsCollector {
    /// Metrics prefix
    prefix: String,
    /// OTEL tracer for metrics (if available)
    tracer: Option<Arc<RwLock<Option<knhk_otel::Tracer>>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            tracer: None,
        }
    }

    /// Create a new metrics collector with OTEL integration
    pub fn with_otel(prefix: String, tracer: Arc<RwLock<Option<knhk_otel::Tracer>>>) -> Self {
        Self {
            prefix,
            tracer: Some(tracer),
        }
    }

    /// Record workflow registration
    pub fn record_workflow_registration(&self, success: bool) {
        self.record_counter_metric(
            "workflow.registration",
            if success { 1 } else { 0 },
            vec![("success".to_string(), success.to_string())],
        );
    }

    /// Record case creation
    pub fn record_case_creation(&self, success: bool) {
        self.record_counter_metric(
            "case.creation",
            if success { 1 } else { 0 },
            vec![("success".to_string(), success.to_string())],
        );
    }

    /// Record case execution
    pub fn record_case_execution(&self, case_id: &CaseId, duration_ms: u64, success: bool) {
        self.record_histogram_metric(
            "case.execution.duration",
            vec![duration_ms],
            vec![
                ("case_id".to_string(), case_id.to_string()),
                ("success".to_string(), success.to_string()),
            ],
        );
    }

    /// Record pattern execution
    pub fn record_pattern_execution(&self, pattern_id: u32, duration_ns: u64, success: bool) {
        self.record_histogram_metric(
            "pattern.execution.duration",
            vec![duration_ns],
            vec![
                ("pattern_id".to_string(), pattern_id.to_string()),
                ("success".to_string(), success.to_string()),
            ],
        );
    }

    /// Record active cases
    pub fn record_active_cases(&self, count: usize) {
        self.record_gauge_metric("cases.active", count as f64, vec![]);
    }

    /// Record circuit breaker state
    pub fn record_circuit_breaker_state(&self, name: &str, state: &str) {
        self.record_gauge_metric(
            "circuit_breaker.state",
            if state == "open" { 1.0 } else { 0.0 },
            vec![
                ("name".to_string(), name.to_string()),
                ("state".to_string(), state.to_string()),
            ],
        );
    }

    /// Record rate limit hits
    pub fn record_rate_limit_hit(&self, limiter_name: &str) {
        self.record_counter_metric(
            "rate_limit.hits",
            1,
            vec![("limiter".to_string(), limiter_name.to_string())],
        );
    }

    /// Record timeout
    pub fn record_timeout(&self, operation: &str) {
        self.record_counter_metric(
            "timeout.count",
            1,
            vec![("operation".to_string(), operation.to_string())],
        );
    }

    /// Record counter metric
    fn record_counter_metric(&self, name: &str, value: u64, attributes: Vec<(String, String)>) {
        if let Some(ref tracer_arc) = self.tracer {
            let metric = Metric {
                name: format!("{}{}", self.prefix, name),
                value: MetricValue::Counter(value),
                timestamp_ms: Self::get_timestamp_ms(),
                attributes: attributes.into_iter().collect(),
            };

            // Record metric asynchronously (non-blocking)
            let tracer_clone = tracer_arc.clone();
            tokio::spawn(async move {
                let mut guard = tracer_clone.write().await;
                if let Some(ref mut tracer) = *guard {
                    if let Some(ref mut tracer) = *guard {
                        tracer.record_metric(metric);
                    }
                }
            });
        }
    }

    /// Record gauge metric
    fn record_gauge_metric(&self, name: &str, value: f64, attributes: Vec<(String, String)>) {
        if let Some(ref tracer_arc) = self.tracer {
            let metric = Metric {
                name: format!("{}{}", self.prefix, name),
                value: MetricValue::Gauge(value),
                timestamp_ms: Self::get_timestamp_ms(),
                attributes: attributes.into_iter().collect(),
            };

            // Record metric asynchronously (non-blocking)
            let tracer_clone = tracer_arc.clone();
            tokio::spawn(async move {
                let mut guard = tracer_clone.write().await;
                if let Some(ref mut tracer) = *guard {
                    if let Some(ref mut tracer) = *guard {
                        tracer.record_metric(metric);
                    }
                }
            });
        }
    }

    /// Record histogram metric
    fn record_histogram_metric(
        &self,
        name: &str,
        values: Vec<u64>,
        attributes: Vec<(String, String)>,
    ) {
        if let Some(ref tracer_arc) = self.tracer {
            let metric = Metric {
                name: format!("{}{}", self.prefix, name),
                value: MetricValue::Histogram(values),
                timestamp_ms: Self::get_timestamp_ms(),
                attributes: attributes.into_iter().collect(),
            };

            // Record metric asynchronously (non-blocking)
            let tracer_clone = tracer_arc.clone();
            tokio::spawn(async move {
                let mut guard = tracer_clone.write().await;
                if let Some(ref mut tracer) = *guard {
                    if let Some(ref mut tracer) = *guard {
                        tracer.record_metric(metric);
                    }
                }
            });
        }
    }

    /// Get current timestamp in milliseconds
    fn get_timestamp_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new("workflow_engine_".to_string())
    }
}

/// Workflow metrics wrapper
pub struct WorkflowMetrics {
    collector: Arc<MetricsCollector>,
}

impl WorkflowMetrics {
    /// Create new workflow metrics
    pub fn new(prefix: String) -> Self {
        Self {
            collector: Arc::new(MetricsCollector::new(prefix)),
        }
    }

    /// Get metrics collector
    pub fn collector(&self) -> &MetricsCollector {
        &self.collector
    }
}

impl Default for WorkflowMetrics {
    fn default() -> Self {
        Self::new("workflow_engine_".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new("test_".to_string());
        collector.record_workflow_registration(true);
        collector.record_case_creation(true);
    }
}
