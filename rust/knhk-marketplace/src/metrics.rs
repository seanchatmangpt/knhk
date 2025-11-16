// Usage Metering and Analytics
// Track customer usage for billing integration and resource optimization

use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Invalid metric: {0}")]
    InvalidMetric(String),

    #[error("Metric not found")]
    NotFound,

    #[error("Metering failed: {0}")]
    MeteringFailed(String),
}

/// Usage metric type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MetricType {
    APICall,
    WorkflowExecution,
    DataProcessed, // bytes
    ComputeTime,   // milliseconds
    StorageUsed,   // bytes
    GpuHours,
    NetworkBandwidth, // bytes
}

/// Usage metrics for a customer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub customer_id: String,
    pub tenant_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub metrics: std::collections::HashMap<String, MetricValue>,
}

/// Individual metric value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricValue {
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub recorded_at: DateTime<Utc>,
    pub count: u64,
}

impl UsageMetrics {
    /// Create new usage metrics
    pub fn new(customer_id: String, tenant_id: String) -> Self {
        let now = Utc::now();
        Self {
            customer_id,
            tenant_id,
            period_start: now,
            period_end: now,
            metrics: std::collections::HashMap::new(),
        }
    }

    /// Record a metric
    pub fn record(&mut self, name: String, value: f64, metric_type: MetricType, unit: String) {
        self.metrics.insert(
            name,
            MetricValue {
                metric_type,
                value,
                unit,
                recorded_at: Utc::now(),
                count: 1,
            },
        );
    }

    /// Get metric value
    pub fn get_metric(&self, name: &str) -> Option<&MetricValue> {
        self.metrics.get(name)
    }

    /// Get total billable units
    pub fn get_total_billable(&self) -> f64 {
        // Phase 10 implementation stub
        // TODO: Implement billable calculation
        // Different metrics have different billing weights
        // Example: 1 API call = 0.001 units, 1GB data = 1 unit

        let mut total = 0.0;

        for (_, metric) in &self.metrics {
            total += metric.value;
        }

        total
    }
}

/// Metrics collector for recording customer usage
#[derive(Debug)]
pub struct MetricsCollector {
    metrics: std::collections::HashMap<String, UsageMetrics>,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: std::collections::HashMap::new(),
        }
    }

    /// Get or create metrics for customer
    pub fn get_or_create(&mut self, customer_id: String, tenant_id: String) -> &mut UsageMetrics {
        self.metrics.entry(customer_id).or_insert_with(|| UsageMetrics::new(customer_id, tenant_id))
    }

    /// Record API call
    pub fn record_api_call(&mut self, customer_id: String, tenant_id: String) -> Result<(), MetricsError> {
        let metrics = self.get_or_create(customer_id, tenant_id);
        metrics.record(
            "api_calls".to_string(),
            1.0,
            MetricType::APICall,
            "count".to_string(),
        );

        tracing::trace!("Metrics: recorded API call");

        Ok(())
    }

    /// Record workflow execution
    pub fn record_workflow(&mut self, customer_id: String, tenant_id: String, duration_ms: u64) -> Result<(), MetricsError> {
        let metrics = self.get_or_create(customer_id, tenant_id);
        metrics.record(
            "workflows".to_string(),
            1.0,
            MetricType::WorkflowExecution,
            "count".to_string(),
        );
        metrics.record(
            "compute_time".to_string(),
            duration_ms as f64,
            MetricType::ComputeTime,
            "ms".to_string(),
        );

        tracing::trace!("Metrics: recorded workflow ({}ms)", duration_ms);

        Ok(())
    }

    /// Record data processing
    pub fn record_data_processed(&mut self, customer_id: String, tenant_id: String, bytes: u64) -> Result<(), MetricsError> {
        let metrics = self.get_or_create(customer_id.clone(), tenant_id);
        metrics.record(
            "data_processed".to_string(),
            bytes as f64,
            MetricType::DataProcessed,
            "bytes".to_string(),
        );

        tracing::trace!("Metrics: recorded {} bytes processed", bytes);

        Ok(())
    }

    /// Get customer metrics
    pub fn get_metrics(&self, customer_id: &str) -> Result<UsageMetrics, MetricsError> {
        self.metrics.get(customer_id)
            .cloned()
            .ok_or(MetricsError::NotFound)
    }

    /// Get all metrics
    pub fn list_all(&self) -> Vec<UsageMetrics> {
        self.metrics.values().cloned().collect()
    }

    /// Get total billable for customer
    pub fn get_customer_billable(&self, customer_id: &str) -> Result<f64, MetricsError> {
        let metrics = self.get_metrics(customer_id)?;
        Ok(metrics.get_total_billable())
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Metering engine for real-time usage tracking
#[derive(Debug)]
pub struct MeteringEngine {
    collector: MetricsCollector,
    /// Check interval in seconds
    pub check_interval_secs: u64,
    /// Enable rate limiting
    pub rate_limiting_enabled: bool,
}

impl MeteringEngine {
    /// Create new metering engine
    pub fn new() -> Self {
        Self {
            collector: MetricsCollector::new(),
            check_interval_secs: 60,
            rate_limiting_enabled: true,
        }
    }

    /// Check if customer has exceeded quota
    pub fn check_quota(&self, customer_id: &str, quota: f64) -> Result<bool, MetricsError> {
        // Phase 10 implementation stub
        // TODO: Implement quota checking
        // Step 1: Get customer metrics
        // Step 2: Calculate current usage
        // Step 3: Compare to quota

        let billable = self.collector.get_customer_billable(customer_id)?;
        Ok(billable > quota)
    }

    /// Get metering status
    pub fn get_status(&self) -> MeteringStatus {
        MeteringStatus {
            customers_tracked: self.collector.metrics.len(),
            total_api_calls: 0,
            total_workflows: 0,
            total_data_processed_gb: 0.0,
        }
    }
}

impl Default for MeteringEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Metering status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeteringStatus {
    pub customers_tracked: usize,
    pub total_api_calls: u64,
    pub total_workflows: u64,
    pub total_data_processed_gb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_metrics_creation() {
        let metrics = UsageMetrics::new("cust1".to_string(), "tenant1".to_string());
        assert_eq!(metrics.customer_id, "cust1");
        assert_eq!(metrics.metrics.len(), 0);
    }

    #[test]
    fn test_usage_metrics_record() {
        let mut metrics = UsageMetrics::new("cust1".to_string(), "tenant1".to_string());
        metrics.record(
            "api_calls".to_string(),
            1.0,
            MetricType::APICall,
            "count".to_string(),
        );

        assert!(metrics.get_metric("api_calls").is_some());
    }

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.metrics.len(), 0);
    }

    #[test]
    fn test_metrics_collector_record_api_call() {
        let mut collector = MetricsCollector::new();
        let result = collector.record_api_call("cust1".to_string(), "tenant1".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_metrics_collector_record_workflow() {
        let mut collector = MetricsCollector::new();
        let result = collector.record_workflow("cust1".to_string(), "tenant1".to_string(), 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_metrics_collector_get_metrics() {
        let mut collector = MetricsCollector::new();
        collector.record_api_call("cust1".to_string(), "tenant1".to_string()).unwrap();

        let result = collector.get_metrics("cust1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_metering_engine_creation() {
        let engine = MeteringEngine::new();
        assert_eq!(engine.check_interval_secs, 60);
        assert!(engine.rate_limiting_enabled);
    }

    #[test]
    fn test_metering_engine_status() {
        let engine = MeteringEngine::new();
        let status = engine.get_status();
        assert_eq!(status.customers_tracked, 0);
    }
}
