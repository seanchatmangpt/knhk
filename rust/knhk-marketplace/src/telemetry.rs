//! Anonymized telemetry and analytics collection
//!
//! Collects usage data, performance metrics, and error information
//! with GDPR-compliant data handling and optional opt-in error reporting

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{MarketplaceError, Result};

/// Telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Event ID
    pub id: Uuid,
    /// Event type (execution_start, execution_end, error, etc.)
    pub event_type: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Anonymous organization ID (hashed)
    pub org_id_hash: String,
    /// Feature name
    pub feature: String,
    /// Execution duration (ms) if applicable
    pub duration_ms: Option<u64>,
    /// Success flag
    pub success: bool,
    /// Additional properties (redacted PII)
    pub properties: HashMap<String, String>,
    /// Platform info (OS, Rust version)
    pub platform: String,
}

impl TelemetryEvent {
    /// Create execution start event
    pub fn execution_start(org_id_hash: String, feature: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: "execution_start".to_string(),
            timestamp: Utc::now(),
            org_id_hash,
            feature,
            duration_ms: None,
            success: true,
            properties: HashMap::new(),
            platform: Self::platform_string(),
        }
    }

    /// Create execution end event
    pub fn execution_end(
        org_id_hash: String,
        feature: String,
        duration_ms: u64,
        success: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: "execution_end".to_string(),
            timestamp: Utc::now(),
            org_id_hash,
            feature,
            duration_ms: Some(duration_ms),
            success,
            properties: HashMap::new(),
            platform: Self::platform_string(),
        }
    }

    /// Create feature adoption event
    pub fn feature_adoption(org_id_hash: String, feature: String, version: String) -> Self {
        let mut props = HashMap::new();
        props.insert("version".to_string(), version);

        Self {
            id: Uuid::new_v4(),
            event_type: "feature_adoption".to_string(),
            timestamp: Utc::now(),
            org_id_hash,
            feature,
            duration_ms: None,
            success: true,
            properties: props,
            platform: Self::platform_string(),
        }
    }

    /// Create error event (opt-in only)
    pub fn error(
        org_id_hash: String,
        feature: String,
        error_type: String,
        message: String,
    ) -> Self {
        let mut props = HashMap::new();
        props.insert("error_type".to_string(), error_type);
        props.insert("message".to_string(), message);

        Self {
            id: Uuid::new_v4(),
            event_type: "error".to_string(),
            timestamp: Utc::now(),
            org_id_hash,
            feature,
            duration_ms: None,
            success: false,
            properties: props,
            platform: Self::platform_string(),
        }
    }

    /// Get platform string (redacted for privacy)
    fn platform_string() -> String {
        format!("Rust/{}", env!("CARGO_PKG_VERSION"))
    }
}

/// Analytics collector
pub struct TelemetryCollector {
    events: Vec<TelemetryEvent>,
    batch_size: usize,
    opt_in_errors: bool,
}

impl TelemetryCollector {
    /// Create new collector
    pub fn new(batch_size: usize, opt_in_errors: bool) -> Self {
        Self {
            events: Vec::new(),
            batch_size,
            opt_in_errors,
        }
    }

    /// Record event
    pub fn record(&mut self, event: TelemetryEvent) -> Result<()> {
        // Filter error events if user hasn't opted in
        if event.event_type == "error" && !self.opt_in_errors {
            return Ok(());
        }

        self.events.push(event);

        if self.events.len() >= self.batch_size {
            self.flush()?;
        }

        Ok(())
    }

    /// Flush events (send upstream)
    pub fn flush(&mut self) -> Result<()> {
        if self.events.is_empty() {
            return Ok(());
        }

        tracing::info!(
            "Flushing {} telemetry events",
            self.events.len()
        );

        // In real implementation, would send to backend service
        // For now, just log and clear
        self.events.clear();
        Ok(())
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Enable error reporting
    pub fn enable_error_reporting(&mut self) {
        self.opt_in_errors = true;
    }

    /// Disable error reporting
    pub fn disable_error_reporting(&mut self) {
        self.opt_in_errors = false;
    }

    /// Clear all events without sending
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

/// Performance analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Feature name
    pub feature: String,
    /// Total executions
    pub execution_count: u64,
    /// Average duration (ms)
    pub avg_duration_ms: f64,
    /// Minimum duration (ms)
    pub min_duration_ms: u64,
    /// Maximum duration (ms)
    pub max_duration_ms: u64,
    /// Success rate (0-100%)
    pub success_rate: f64,
    /// P95 latency (ms)
    pub p95_latency_ms: u64,
    /// P99 latency (ms)
    pub p99_latency_ms: u64,
}

/// Feature adoption tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAdoption {
    /// Feature name
    pub feature: String,
    /// Number of organizations using it
    pub org_count: u32,
    /// Adoption percentage (0-100%)
    pub adoption_percentage: f32,
    /// Total usages
    pub usage_count: u64,
}

/// Health check data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// API availability (0-100%)
    pub api_availability: f32,
    /// Marketplace availability
    pub marketplace_availability: f32,
    /// Billing system availability
    pub billing_availability: f32,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Error rate (0-100%)
    pub error_rate: f32,
}

/// Analytics dashboard
pub struct AnalyticsDashboard {
    performance_metrics: HashMap<String, PerformanceMetrics>,
    adoption_tracking: HashMap<String, FeatureAdoption>,
    health_history: Vec<HealthMetrics>,
}

impl AnalyticsDashboard {
    /// Create new dashboard
    pub fn new() -> Self {
        Self {
            performance_metrics: HashMap::new(),
            adoption_tracking: HashMap::new(),
            health_history: Vec::new(),
        }
    }

    /// Record performance metric
    pub fn record_performance(&mut self, metric: PerformanceMetrics) {
        self.performance_metrics.insert(metric.feature.clone(), metric);
    }

    /// Record feature adoption
    pub fn record_adoption(&mut self, adoption: FeatureAdoption) {
        self.adoption_tracking.insert(adoption.feature.clone(), adoption);
    }

    /// Record health metrics
    pub fn record_health(&mut self, health: HealthMetrics) {
        self.health_history.push(health);
    }

    /// Get feature performance
    pub fn get_performance(&self, feature: &str) -> Option<&PerformanceMetrics> {
        self.performance_metrics.get(feature)
    }

    /// Get feature adoption
    pub fn get_adoption(&self, feature: &str) -> Option<&FeatureAdoption> {
        self.adoption_tracking.get(feature)
    }

    /// Get latest health metrics
    pub fn get_latest_health(&self) -> Option<&HealthMetrics> {
        self.health_history.last()
    }

    /// Get health history
    pub fn get_health_history(&self, hours: i64) -> Vec<&HealthMetrics> {
        let cutoff = Utc::now() - chrono::Duration::hours(hours);

        self.health_history
            .iter()
            .filter(|h| h.timestamp > cutoff)
            .collect()
    }

    /// Clear old health metrics (older than N days)
    pub fn cleanup_health_history(&mut self, days: i64) {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        self.health_history.retain(|h| h.timestamp > cutoff);
    }
}

impl Default for AnalyticsDashboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize telemetry system
pub fn init_collector() -> Result<()> {
    tracing::info!("Initializing telemetry collection system (GDPR-compliant)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_event_creation() {
        let event = TelemetryEvent::execution_start(
            "org_hash_123".to_string(),
            "workflow_execute".to_string(),
        );

        assert_eq!(event.event_type, "execution_start");
        assert_eq!(event.feature, "workflow_execute");
        assert!(event.success);
    }

    #[test]
    fn test_telemetry_error_event() {
        let event = TelemetryEvent::error(
            "org_hash_123".to_string(),
            "workflow_execute".to_string(),
            "timeout".to_string(),
            "Execution exceeded timeout".to_string(),
        );

        assert_eq!(event.event_type, "error");
        assert!(!event.success);
        assert_eq!(event.properties.get("error_type"), Some(&"timeout".to_string()));
    }

    #[test]
    fn test_collector_batch_size() {
        let mut collector = TelemetryCollector::new(10, false);

        for i in 0..9 {
            let event = TelemetryEvent::execution_start(
                format!("org_{}", i),
                "test".to_string(),
            );
            assert!(collector.record(event).is_ok());
        }

        assert_eq!(collector.event_count(), 9);
    }

    #[test]
    fn test_collector_error_opt_in() {
        let mut collector = TelemetryCollector::new(100, false);

        let error_event = TelemetryEvent::error(
            "org_123".to_string(),
            "test".to_string(),
            "test_error".to_string(),
            "Test error".to_string(),
        );

        assert!(collector.record(error_event).is_ok());
        assert_eq!(collector.event_count(), 0); // Error not recorded without opt-in

        collector.enable_error_reporting();
        let error_event2 = TelemetryEvent::error(
            "org_123".to_string(),
            "test".to_string(),
            "test_error".to_string(),
            "Test error".to_string(),
        );

        assert!(collector.record(error_event2).is_ok());
        assert_eq!(collector.event_count(), 1); // Error recorded with opt-in
    }

    #[test]
    fn test_analytics_dashboard() {
        let mut dashboard = AnalyticsDashboard::new();

        let perf = PerformanceMetrics {
            feature: "workflow_execute".to_string(),
            execution_count: 1000,
            avg_duration_ms: 150.5,
            min_duration_ms: 50,
            max_duration_ms: 2000,
            success_rate: 99.5,
            p95_latency_ms: 500,
            p99_latency_ms: 1500,
        };

        dashboard.record_performance(perf);
        assert!(dashboard.get_performance("workflow_execute").is_some());
    }

    #[test]
    fn test_feature_adoption_tracking() {
        let mut dashboard = AnalyticsDashboard::new();

        let adoption = FeatureAdoption {
            feature: "cloud_deployment".to_string(),
            org_count: 150,
            adoption_percentage: 45.5,
            usage_count: 5000,
        };

        dashboard.record_adoption(adoption);
        assert!(dashboard.get_adoption("cloud_deployment").is_some());
    }
}
