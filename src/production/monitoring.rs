// KNHK Monitoring Layer - SLA Tracking and Alerting
// Phase 5: 99.99% uptime monitoring with comprehensive health checks
// Tracks SLAs, performance, and triggers alerts for Fortune 500 deployments

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::mpsc;
use tokio::time::interval;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug, instrument};
use dashmap::DashMap;
use super::platform::SystemHealth;

const SLA_TARGET: f64 = 99.99; // 99.99% uptime = 52.6 minutes/year downtime
const LATENCY_SLA_P50_MS: u64 = 100;
const LATENCY_SLA_P99_MS: u64 = 1000;
const THROUGHPUT_SLA_RPS: f64 = 1000.0;
const HEALTH_CHECK_WINDOW: Duration = Duration::from_secs(60);
const ALERT_COOLDOWN: Duration = Duration::from_secs(300); // 5 minutes
const METRIC_RETENTION_HOURS: u64 = 168; // 7 days

/// SLA compliance tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLATracker {
    pub uptime_percentage: f64,
    pub total_time_ms: u64,
    pub downtime_ms: u64,
    pub violations: Vec<SLAViolation>,
    pub start_time: SystemTime,
    pub last_check: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAViolation {
    pub timestamp: SystemTime,
    pub violation_type: ViolationType,
    pub severity: Severity,
    pub duration_ms: Option<u64>,
    pub details: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    Downtime,
    LatencyP50,
    LatencyP99,
    Throughput,
    ErrorRate,
    ResourceExhaustion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enable_alerts: bool,
    pub alert_channels: Vec<AlertChannel>,
    pub thresholds: AlertThresholds,
    pub cooldown_period: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    Webhook { url: String },
    Email { addresses: Vec<String> },
    PagerDuty { service_key: String },
    Slack { webhook_url: String },
    Console,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub downtime_seconds: u64,
    pub latency_p50_ms: u64,
    pub latency_p99_ms: u64,
    pub error_rate_percent: f64,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            downtime_seconds: 60,
            latency_p50_ms: LATENCY_SLA_P50_MS * 2,
            latency_p99_ms: LATENCY_SLA_P99_MS * 2,
            error_rate_percent: 1.0,
            cpu_percent: 90.0,
            memory_percent: 90.0,
            disk_percent: 95.0,
        }
    }
}

/// Performance metrics tracking
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub latency_samples: VecDeque<LatencySample>,
    pub throughput_samples: VecDeque<ThroughputSample>,
    pub error_samples: VecDeque<ErrorSample>,
    pub resource_samples: VecDeque<ResourceSample>,
}

#[derive(Debug, Clone)]
pub struct LatencySample {
    pub timestamp: Instant,
    pub p50_ms: u64,
    pub p90_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub p999_ms: u64,
    pub max_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ThroughputSample {
    pub timestamp: Instant,
    pub requests_per_second: f64,
    pub bytes_per_second: f64,
    pub active_connections: usize,
}

#[derive(Debug, Clone)]
pub struct ErrorSample {
    pub timestamp: Instant,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub error_rate: f64,
    pub errors_by_type: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct ResourceSample {
    pub timestamp: Instant,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
    pub network_mbps: f64,
}

/// Main monitoring layer
pub struct MonitoringLayer {
    // SLA tracking
    sla_tracker: Arc<RwLock<SLATracker>>,

    // Performance metrics
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,

    // Health status
    health_status: Arc<RwLock<SystemHealth>>,
    is_healthy: Arc<AtomicBool>,
    last_healthy: Arc<RwLock<Option<Instant>>>,

    // Alert management
    alert_config: AlertConfig,
    active_alerts: Arc<DashMap<String, Alert>>,
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    alert_tx: mpsc::UnboundedSender<Alert>,
    alert_rx: Option<mpsc::UnboundedReceiver<Alert>>,

    // Anomaly detection
    anomaly_detector: Arc<AnomalyDetector>,

    // Metrics aggregation
    metric_aggregator: Arc<MetricAggregator>,

    // Statistics
    total_health_checks: Arc<AtomicU64>,
    total_alerts_triggered: Arc<AtomicU64>,
    total_anomalies_detected: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub timestamp: SystemTime,
    pub severity: Severity,
    pub alert_type: String,
    pub message: String,
    pub details: HashMap<String, String>,
    pub acknowledged: bool,
    pub resolved: bool,
}

/// Anomaly detection using statistical methods
pub struct AnomalyDetector {
    baseline: Arc<RwLock<BaselineMetrics>>,
    sensitivity: f64,
}

#[derive(Debug, Clone, Default)]
struct BaselineMetrics {
    latency_mean: f64,
    latency_stddev: f64,
    throughput_mean: f64,
    throughput_stddev: f64,
    error_rate_mean: f64,
    error_rate_stddev: f64,
    samples_count: u64,
}

/// Metric aggregation for reporting
pub struct MetricAggregator {
    hourly_metrics: Arc<DashMap<String, HourlyMetrics>>,
    daily_metrics: Arc<DashMap<String, DailyMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HourlyMetrics {
    hour: u64,
    uptime_percentage: f64,
    avg_latency_ms: f64,
    max_latency_ms: u64,
    total_requests: u64,
    failed_requests: u64,
    avg_cpu_percent: f64,
    avg_memory_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DailyMetrics {
    date: String,
    uptime_percentage: f64,
    sla_met: bool,
    total_requests: u64,
    failed_requests: u64,
    total_violations: u64,
    total_alerts: u64,
}

impl MonitoringLayer {
    /// Initialize monitoring layer
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing monitoring layer");

        let (alert_tx, alert_rx) = mpsc::unbounded_channel();

        let sla_tracker = SLATracker {
            uptime_percentage: 100.0,
            total_time_ms: 0,
            downtime_ms: 0,
            violations: Vec::new(),
            start_time: SystemTime::now(),
            last_check: SystemTime::now(),
        };

        Ok(Self {
            sla_tracker: Arc::new(RwLock::new(sla_tracker)),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            health_status: Arc::new(RwLock::new(SystemHealth::default())),
            is_healthy: Arc::new(AtomicBool::new(true)),
            last_healthy: Arc::new(RwLock::new(Some(Instant::now()))),
            alert_config: AlertConfig {
                enable_alerts: true,
                alert_channels: vec![AlertChannel::Console],
                thresholds: AlertThresholds::default(),
                cooldown_period: ALERT_COOLDOWN,
            },
            active_alerts: Arc::new(DashMap::new()),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            alert_tx,
            alert_rx: Some(alert_rx),
            anomaly_detector: Arc::new(AnomalyDetector::new()),
            metric_aggregator: Arc::new(MetricAggregator::new()),
            total_health_checks: Arc::new(AtomicU64::new(0)),
            total_alerts_triggered: Arc::new(AtomicU64::new(0)),
            total_anomalies_detected: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Start monitoring services
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting monitoring services");

        // Start alert processor
        self.start_alert_processor();

        // Start SLA calculator
        self.start_sla_calculator();

        // Start metric aggregator
        self.start_metric_aggregator();

        // Start anomaly detector
        self.start_anomaly_detector();

        info!("Monitoring services started");
        Ok(())
    }

    /// Update system health status
    #[instrument(skip(self, health))]
    pub async fn update_health(&self, health: SystemHealth) {
        self.total_health_checks.fetch_add(1, Ordering::Relaxed);

        let was_healthy = self.is_healthy.load(Ordering::Relaxed);
        let is_healthy_now = health.is_healthy();

        // Update health status
        *self.health_status.write().unwrap() = health.clone();
        self.is_healthy.store(is_healthy_now, Ordering::Relaxed);

        // Track state changes
        if was_healthy && !is_healthy_now {
            // System became unhealthy
            self.record_downtime_start().await;

            // Trigger alert
            self.trigger_alert(
                Severity::Critical,
                "System Health Degraded",
                format!("System entered unhealthy state: {:?}", health),
            ).await;

        } else if !was_healthy && is_healthy_now {
            // System recovered
            self.record_downtime_end().await;

            // Clear related alerts
            self.resolve_alerts("health").await;
        }

        if is_healthy_now {
            *self.last_healthy.write().unwrap() = Some(Instant::now());
        }

        // Check resource thresholds
        self.check_resource_thresholds(&health).await;
    }

    /// Record workflow success
    #[instrument(skip(self))]
    pub async fn record_workflow_success(&self, workflow_id: &str, duration: Duration) {
        let latency_ms = duration.as_millis() as u64;

        // Update latency samples
        let mut metrics = self.performance_metrics.write().unwrap();
        if let Some(last_sample) = metrics.latency_samples.back_mut() {
            // Update existing sample if recent
            if last_sample.timestamp.elapsed() < Duration::from_secs(1) {
                last_sample.p50_ms = (last_sample.p50_ms + latency_ms) / 2;
                last_sample.max_ms = last_sample.max_ms.max(latency_ms);
            }
        }

        // Check SLA compliance
        if latency_ms > LATENCY_SLA_P99_MS {
            self.record_sla_violation(
                ViolationType::LatencyP99,
                format!("Workflow {} exceeded P99 latency SLA: {}ms > {}ms",
                    workflow_id, latency_ms, LATENCY_SLA_P99_MS)
            ).await;
        }
    }

    /// Record workflow failure
    #[instrument(skip(self))]
    pub async fn record_workflow_failure(&self, workflow_id: &str, error: &str) {
        // Update error samples
        let mut metrics = self.performance_metrics.write().unwrap();

        let sample = ErrorSample {
            timestamp: Instant::now(),
            total_requests: 1,
            failed_requests: 1,
            error_rate: 100.0,
            errors_by_type: [(error.to_string(), 1)].into_iter().collect(),
        };

        metrics.error_samples.push_back(sample);

        // Keep only recent samples
        while metrics.error_samples.len() > 1000 {
            metrics.error_samples.pop_front();
        }

        // Check error rate threshold
        let error_rate = self.calculate_error_rate();
        if error_rate > self.alert_config.thresholds.error_rate_percent {
            self.trigger_alert(
                Severity::Warning,
                "High Error Rate",
                format!("Error rate {:.2}% exceeds threshold {:.2}%",
                    error_rate, self.alert_config.thresholds.error_rate_percent)
            ).await;
        }
    }

    /// Record workflow timeout
    #[instrument(skip(self))]
    pub async fn record_workflow_timeout(&self, workflow_id: &str) {
        self.record_workflow_failure(workflow_id, "timeout").await;
    }

    /// Start recording downtime
    async fn record_downtime_start(&self) {
        warn!("System downtime started");

        let mut tracker = self.sla_tracker.write().unwrap();
        tracker.last_check = SystemTime::now();

        // Record violation
        tracker.violations.push(SLAViolation {
            timestamp: SystemTime::now(),
            violation_type: ViolationType::Downtime,
            severity: Severity::Critical,
            duration_ms: None,
            details: "System became unhealthy".to_string(),
        });
    }

    /// Stop recording downtime
    async fn record_downtime_end(&self) {
        let downtime_duration = {
            let tracker = self.sla_tracker.read().unwrap();
            SystemTime::now().duration_since(tracker.last_check)
                .unwrap_or_default()
        };

        info!("System downtime ended after {:?}", downtime_duration);

        let mut tracker = self.sla_tracker.write().unwrap();
        let downtime_ms = downtime_duration.as_millis() as u64;
        tracker.downtime_ms += downtime_ms;

        // Update last violation with duration
        if let Some(last) = tracker.violations.last_mut() {
            if last.violation_type == ViolationType::Downtime && last.duration_ms.is_none() {
                last.duration_ms = Some(downtime_ms);
            }
        }

        // Recalculate uptime
        let total_time = SystemTime::now()
            .duration_since(tracker.start_time)
            .unwrap_or_default();

        tracker.total_time_ms = total_time.as_millis() as u64;
        tracker.uptime_percentage = if tracker.total_time_ms > 0 {
            ((tracker.total_time_ms - tracker.downtime_ms) as f64 / tracker.total_time_ms as f64) * 100.0
        } else {
            100.0
        };

        tracker.last_check = SystemTime::now();
    }

    /// Record SLA violation
    async fn record_sla_violation(&self, violation_type: ViolationType, details: String) {
        warn!("SLA violation: {:?} - {}", violation_type, details);

        let mut tracker = self.sla_tracker.write().unwrap();
        tracker.violations.push(SLAViolation {
            timestamp: SystemTime::now(),
            violation_type,
            severity: match violation_type {
                ViolationType::Downtime => Severity::Critical,
                ViolationType::ErrorRate | ViolationType::ResourceExhaustion => Severity::Warning,
                _ => Severity::Info,
            },
            duration_ms: None,
            details,
        });
    }

    /// Check resource thresholds
    async fn check_resource_thresholds(&self, health: &SystemHealth) {
        let thresholds = &self.alert_config.thresholds;

        if health.cpu_usage > thresholds.cpu_percent {
            self.trigger_alert(
                Severity::Warning,
                "High CPU Usage",
                format!("CPU usage {:.1}% exceeds threshold {:.1}%",
                    health.cpu_usage, thresholds.cpu_percent)
            ).await;
        }

        if health.memory_usage > thresholds.memory_percent {
            self.trigger_alert(
                Severity::Warning,
                "High Memory Usage",
                format!("Memory usage {:.1}% exceeds threshold {:.1}%",
                    health.memory_usage, thresholds.memory_percent)
            ).await;
        }

        if health.disk_usage > thresholds.disk_percent {
            self.trigger_alert(
                Severity::Critical,
                "High Disk Usage",
                format!("Disk usage {:.1}% exceeds threshold {:.1}%",
                    health.disk_usage, thresholds.disk_percent)
            ).await;
        }
    }

    /// Trigger an alert
    async fn trigger_alert(&self, severity: Severity, title: &str, message: String) {
        let alert = Alert {
            id: format!("alert-{}", uuid::Uuid::new_v4()),
            timestamp: SystemTime::now(),
            severity,
            alert_type: title.to_string(),
            message,
            details: HashMap::new(),
            acknowledged: false,
            resolved: false,
        };

        // Check if similar alert exists and is in cooldown
        let key = format!("{}:{:?}", title, severity);
        if let Some(existing) = self.active_alerts.get(&key) {
            if existing.timestamp.elapsed().unwrap_or_default() < self.alert_config.cooldown_period {
                return; // Skip duplicate alert in cooldown
            }
        }

        // Store alert
        self.active_alerts.insert(key, alert.clone());
        self.alert_history.write().unwrap().push_back(alert.clone());

        // Send alert
        if self.alert_config.enable_alerts {
            self.alert_tx.send(alert).ok();
        }

        self.total_alerts_triggered.fetch_add(1, Ordering::Relaxed);
    }

    /// Resolve alerts of a specific type
    async fn resolve_alerts(&self, alert_type: &str) {
        let mut resolved = Vec::new();

        for entry in self.active_alerts.iter() {
            if entry.value().alert_type.contains(alert_type) {
                resolved.push(entry.key().clone());
            }
        }

        for key in resolved {
            if let Some((_, mut alert)) = self.active_alerts.remove(&key) {
                alert.resolved = true;
                info!("Resolved alert: {}", alert.message);
            }
        }
    }

    /// Calculate current error rate
    fn calculate_error_rate(&self) -> f64 {
        let metrics = self.performance_metrics.read().unwrap();

        let recent_window = Instant::now() - Duration::from_secs(60);
        let mut total = 0u64;
        let mut failures = 0u64;

        for sample in &metrics.error_samples {
            if sample.timestamp > recent_window {
                total += sample.total_requests;
                failures += sample.failed_requests;
            }
        }

        if total > 0 {
            (failures as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Start alert processor
    fn start_alert_processor(&self) {
        let mut rx = self.alert_rx.take().unwrap();
        let channels = self.alert_config.alert_channels.clone();

        tokio::spawn(async move {
            while let Some(alert) = rx.recv().await {
                for channel in &channels {
                    send_alert(&alert, channel).await;
                }
            }
        });
    }

    /// Start SLA calculator
    fn start_sla_calculator(&self) {
        let tracker = self.sla_tracker.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));

            loop {
                ticker.tick().await;

                // Update SLA metrics
                let mut t = tracker.write().unwrap();
                let total_time = SystemTime::now()
                    .duration_since(t.start_time)
                    .unwrap_or_default();

                t.total_time_ms = total_time.as_millis() as u64;
                t.uptime_percentage = if t.total_time_ms > 0 {
                    ((t.total_time_ms - t.downtime_ms) as f64 / t.total_time_ms as f64) * 100.0
                } else {
                    100.0
                };

                if t.uptime_percentage < SLA_TARGET {
                    warn!("SLA target not met: {:.2}% < {:.2}%", t.uptime_percentage, SLA_TARGET);
                }
            }
        });
    }

    /// Start metric aggregator
    fn start_metric_aggregator(&self) {
        let aggregator = self.metric_aggregator.clone();
        let metrics = self.performance_metrics.clone();
        let tracker = self.sla_tracker.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(3600)); // Hourly

            loop {
                ticker.tick().await;
                aggregator.aggregate_hourly(&metrics, &tracker).await;
            }
        });
    }

    /// Start anomaly detector
    fn start_anomaly_detector(&self) {
        let detector = self.anomaly_detector.clone();
        let metrics = self.performance_metrics.clone();
        let anomalies = self.total_anomalies_detected.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));

            loop {
                ticker.tick().await;

                if detector.detect_anomalies(&metrics).await {
                    anomalies.fetch_add(1, Ordering::Relaxed);
                    warn!("Anomaly detected in system metrics");
                }
            }
        });
    }

    /// Get monitoring statistics
    pub fn get_stats(&self) -> MonitoringStats {
        let tracker = self.sla_tracker.read().unwrap();
        let health = self.health_status.read().unwrap();

        MonitoringStats {
            uptime_percentage: tracker.uptime_percentage,
            total_downtime_ms: tracker.downtime_ms,
            total_violations: tracker.violations.len() as u64,
            is_healthy: self.is_healthy.load(Ordering::Relaxed),
            current_cpu: health.cpu_usage,
            current_memory: health.memory_usage,
            current_disk: health.disk_usage,
            active_alerts: self.active_alerts.len(),
            total_health_checks: self.total_health_checks.load(Ordering::Relaxed),
            total_alerts: self.total_alerts_triggered.load(Ordering::Relaxed),
            total_anomalies: self.total_anomalies_detected.load(Ordering::Relaxed),
        }
    }

    /// Shutdown monitoring
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Shutting down monitoring layer");

        // Final SLA calculation
        let tracker = self.sla_tracker.read().unwrap();
        info!("Final SLA: {:.2}% uptime, {} total violations",
            tracker.uptime_percentage,
            tracker.violations.len()
        );

        info!("Monitoring layer shutdown complete");
        Ok(())
    }
}

impl AnomalyDetector {
    fn new() -> Self {
        Self {
            baseline: Arc::new(RwLock::new(BaselineMetrics::default())),
            sensitivity: 2.5, // Standard deviations
        }
    }

    async fn detect_anomalies(&self, metrics: &Arc<RwLock<PerformanceMetrics>>) -> bool {
        // Simple statistical anomaly detection
        let metrics = metrics.read().unwrap();
        let baseline = self.baseline.read().unwrap();

        if baseline.samples_count < 100 {
            return false; // Not enough data
        }

        // Check for anomalies in recent samples
        if let Some(latest) = metrics.latency_samples.back() {
            let latency = latest.p99_ms as f64;
            if (latency - baseline.latency_mean).abs() > baseline.latency_stddev * self.sensitivity {
                return true;
            }
        }

        false
    }
}

impl MetricAggregator {
    fn new() -> Self {
        Self {
            hourly_metrics: Arc::new(DashMap::new()),
            daily_metrics: Arc::new(DashMap::new()),
        }
    }

    async fn aggregate_hourly(
        &self,
        metrics: &Arc<RwLock<PerformanceMetrics>>,
        tracker: &Arc<RwLock<SLATracker>>,
    ) {
        // Aggregate metrics for the past hour
        // Implementation would calculate averages, max values, etc.
    }
}

impl SystemHealth {
    fn is_healthy(&self) -> bool {
        self.cpu_usage < 90.0 && self.memory_usage < 90.0 && self.disk_usage < 95.0
    }
}

impl Default for SystemHealth {
    fn default() -> Self {
        SystemHealth {
            state: super::platform::PlatformState::Running,
            active_workflows: 0,
            memory_usage: 0.0,
            cpu_usage: 0.0,
            disk_usage: 0.0,
        }
    }
}

async fn send_alert(alert: &Alert, channel: &AlertChannel) {
    match channel {
        AlertChannel::Console => {
            match alert.severity {
                Severity::Emergency | Severity::Critical => {
                    error!("[ALERT] {} - {}", alert.alert_type, alert.message);
                }
                Severity::Warning => {
                    warn!("[ALERT] {} - {}", alert.alert_type, alert.message);
                }
                Severity::Info => {
                    info!("[ALERT] {} - {}", alert.alert_type, alert.message);
                }
            }
        }
        AlertChannel::Webhook { url } => {
            // Send HTTP webhook
        }
        AlertChannel::Email { addresses } => {
            // Send email alerts
        }
        AlertChannel::PagerDuty { service_key } => {
            // Trigger PagerDuty incident
        }
        AlertChannel::Slack { webhook_url } => {
            // Send Slack notification
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    pub uptime_percentage: f64,
    pub total_downtime_ms: u64,
    pub total_violations: u64,
    pub is_healthy: bool,
    pub current_cpu: f64,
    pub current_memory: f64,
    pub current_disk: f64,
    pub active_alerts: usize,
    pub total_health_checks: u64,
    pub total_alerts: u64,
    pub total_anomalies: u64,
}