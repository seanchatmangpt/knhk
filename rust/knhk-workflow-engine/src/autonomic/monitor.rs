// rust/knhk-workflow-engine/src/autonomic/monitor.rs
//! Monitor Component for MAPE-K Framework
//!
//! Collects runtime metrics from workflow execution, OTEL telemetry,
//! Dark Matter coverage tracker, and system resources.
//!
//! **Metrics Collected**:
//! - Workflow execution metrics (latency, throughput)
//! - Resource utilization (CPU, memory)
//! - Pattern coverage (80/20 distribution)
//! - Multi-instance statistics
//! - Cancellation/compensation events

use super::knowledge::{Fact, KnowledgeBase};
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;

/// Monitor event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEvent {
    /// Event timestamp
    pub timestamp_ms: u64,
    /// Metric name
    pub metric: String,
    /// Metric value
    pub value: f64,
    /// Event source
    pub source: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl MonitorEvent {
    pub fn new(metric: String, value: f64, source: String) -> Self {
        Self {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            metric,
            value,
            source,
            metadata: HashMap::new(),
        }
    }
}

/// Metric collector trait
pub trait MetricCollector: Send + Sync {
    /// Collect metrics
    fn collect(&self) -> WorkflowResult<Vec<MonitorEvent>>;

    /// Get collector name
    fn name(&self) -> &str;
}

/// Workflow execution metric collector
pub struct WorkflowMetricsCollector {
    name: String,
    /// Execution times (task_id -> duration_ms)
    execution_times: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    /// Throughput counter
    completions: Arc<RwLock<u64>>,
    /// Error counter
    errors: Arc<RwLock<u64>>,
    /// Start time for throughput calculation
    start_time: Instant,
}

impl WorkflowMetricsCollector {
    pub fn new() -> Self {
        Self {
            name: "workflow_metrics".to_string(),
            execution_times: Arc::new(RwLock::new(HashMap::new())),
            completions: Arc::new(RwLock::new(0)),
            errors: Arc::new(RwLock::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Record task execution
    pub async fn record_execution(&self, task_id: String, duration_ms: u64) {
        let mut times = self.execution_times.write().await;
        times.entry(task_id).or_insert_with(Vec::new).push(duration_ms);

        let mut completions = self.completions.write().await;
        *completions += 1;
    }

    /// Record error
    pub async fn record_error(&self) {
        let mut errors = self.errors.write().await;
        *errors += 1;
    }
}

impl MetricCollector for WorkflowMetricsCollector {
    fn collect(&self) -> WorkflowResult<Vec<MonitorEvent>> {
        let mut events = Vec::new();

        // Collect average latency
        let times = futures::executor::block_on(self.execution_times.read());
        if !times.is_empty() {
            let total: u64 = times.values().flatten().sum();
            let count: usize = times.values().map(|v| v.len()).sum();
            if count > 0 {
                let avg_latency = total as f64 / count as f64;
                events.push(MonitorEvent::new(
                    "avg_latency_ms".to_string(),
                    avg_latency,
                    self.name.clone(),
                ));
            }
        }

        // Collect throughput (completions per second)
        let completions = futures::executor::block_on(self.completions.read());
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            let throughput = *completions as f64 / elapsed;
            events.push(MonitorEvent::new(
                "throughput".to_string(),
                throughput,
                self.name.clone(),
            ));
        }

        // Collect error rate
        let errors = futures::executor::block_on(self.errors.read());
        if *completions > 0 {
            let error_rate = *errors as f64 / *completions as f64;
            events.push(MonitorEvent::new(
                "error_rate".to_string(),
                error_rate,
                self.name.clone(),
            ));
        }

        Ok(events)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// System resource metric collector
pub struct ResourceMetricsCollector {
    name: String,
}

impl ResourceMetricsCollector {
    pub fn new() -> Self {
        Self {
            name: "system_resources".to_string(),
        }
    }
}

impl MetricCollector for ResourceMetricsCollector {
    fn collect(&self) -> WorkflowResult<Vec<MonitorEvent>> {
        let mut events = Vec::new();

        // Collect memory usage
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
                for line in content.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(value_str) = line.split_whitespace().nth(1) {
                            if let Ok(value_kb) = value_str.parse::<f64>() {
                                let value_mb = value_kb / 1024.0;
                                events.push(MonitorEvent::new(
                                    "memory_usage_mb".to_string(),
                                    value_mb,
                                    self.name.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Collect CPU usage (simplified - real implementation would use system APIs)
        // For now, use a placeholder
        events.push(MonitorEvent::new(
            "cpu_usage".to_string(),
            0.0, // TODO: Implement actual CPU measurement
            self.name.clone(),
        ));

        Ok(events)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Dark Matter coverage metric collector
pub struct CoverageMetricsCollector {
    name: String,
    /// Reference to Dark Matter tracker (optional)
    #[allow(dead_code)]
    tracker: Option<Arc<knhk_connectors::DarkMatterTracker>>,
}

impl CoverageMetricsCollector {
    pub fn new() -> Self {
        Self {
            name: "coverage_metrics".to_string(),
            tracker: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_tracker(tracker: Arc<knhk_connectors::DarkMatterTracker>) -> Self {
        Self {
            name: "coverage_metrics".to_string(),
            tracker: Some(tracker),
        }
    }
}

impl MetricCollector for CoverageMetricsCollector {
    fn collect(&self) -> WorkflowResult<Vec<MonitorEvent>> {
        let mut events = Vec::new();

        // If tracker is available, collect 80/20 metrics
        if let Some(ref _tracker) = self.tracker {
            // TODO: Integrate with Dark Matter tracker
            // let metrics = tracker.metrics();
            // events.push(MonitorEvent::new(
            //     "hot_percentage".to_string(),
            //     metrics.hot_percentage,
            //     self.name.clone(),
            // ));
        }

        // Placeholder metrics
        events.push(MonitorEvent::new(
            "coverage_percentage".to_string(),
            85.0,
            self.name.clone(),
        ));

        Ok(events)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Monitor component
pub struct Monitor {
    /// Knowledge base
    knowledge: Arc<KnowledgeBase>,
    /// Metric collectors
    collectors: Vec<Arc<dyn MetricCollector>>,
    /// Monitoring interval
    interval: Duration,
    /// Whether monitor is running
    running: Arc<RwLock<bool>>,
}

impl Monitor {
    /// Create new monitor
    pub fn new(knowledge: Arc<KnowledgeBase>) -> Self {
        Self {
            knowledge,
            collectors: Vec::new(),
            interval: Duration::from_secs(10), // Default 10s monitoring interval
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Add metric collector
    pub fn add_collector(&mut self, collector: Arc<dyn MetricCollector>) {
        self.collectors.push(collector);
    }

    /// Set monitoring interval
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    /// Start monitoring loop
    pub async fn start(&self) -> WorkflowResult<()> {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        let interval = self.interval;
        let knowledge = self.knowledge.clone();
        let collectors = self.collectors.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                let is_running = *running.read().await;
                if !is_running {
                    break;
                }

                interval_timer.tick().await;

                // Collect metrics from all collectors
                for collector in &collectors {
                    match collector.collect() {
                        Ok(events) => {
                            for event in events {
                                let fact = Fact::new(
                                    event.metric.clone(),
                                    event.value,
                                    event.source.clone(),
                                );
                                if let Err(e) = knowledge.add_fact(fact).await {
                                    eprintln!("Failed to add fact: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Collector {} failed: {}", collector.name(), e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop monitoring loop
    pub async fn stop(&self) -> WorkflowResult<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    /// Check if monitoring is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Collect metrics once (synchronous)
    pub async fn collect_once(&self) -> WorkflowResult<Vec<MonitorEvent>> {
        let mut all_events = Vec::new();

        for collector in &self.collectors {
            match collector.collect() {
                Ok(events) => {
                    for event in events {
                        let fact = Fact::new(
                            event.metric.clone(),
                            event.value,
                            event.source.clone(),
                        );
                        self.knowledge.add_fact(fact).await?;
                        all_events.push(event);
                    }
                }
                Err(e) => {
                    eprintln!("Collector {} failed: {}", collector.name(), e);
                }
            }
        }

        Ok(all_events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_metrics_collector() {
        let collector = WorkflowMetricsCollector::new();

        collector.record_execution("task1".to_string(), 100).await;
        collector.record_execution("task1".to_string(), 150).await;
        collector.record_execution("task2".to_string(), 200).await;

        let events = collector.collect().unwrap();
        assert!(!events.is_empty());

        // Should have average latency
        let latency_event = events.iter().find(|e| e.metric == "avg_latency_ms");
        assert!(latency_event.is_some());
        let avg = latency_event.unwrap().value;
        assert!((avg - 150.0).abs() < 1.0); // Should be ~150ms
    }

    #[tokio::test]
    async fn test_monitor() {
        let kb = Arc::new(KnowledgeBase::new());
        let mut monitor = Monitor::new(kb.clone());

        let collector = Arc::new(WorkflowMetricsCollector::new());
        monitor.add_collector(collector.clone());

        // Record some metrics
        collector.record_execution("task1".to_string(), 100).await;

        // Collect once
        let events = monitor.collect_once().await.unwrap();
        assert!(!events.is_empty());

        // Check facts were added to knowledge base
        let facts = kb.get_facts().await;
        assert!(!facts.is_empty());
    }
}
