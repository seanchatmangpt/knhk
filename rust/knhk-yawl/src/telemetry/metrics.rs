// rust/knhk-yawl/src/telemetry/metrics.rs
// OTEL metrics for YAWL workflows
//
// DOCTRINE ALIGNMENT:
// - Covenant 6: Observations Drive Everything
//   All workflow metrics are observable and measurable
//
// - Covenant 2: Performance Invariants (≤8 ticks for hot path)
//   Metric recording must be lightweight

use crate::PatternType;
use knhk_otel::{Metric, MetricValue, Tracer};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// YAWL workflow metrics
///
/// This struct provides methods to record all YAWL workflow metrics
/// following OpenTelemetry semantic conventions.
pub struct YawlMetrics {
    tracer: Arc<parking_lot::Mutex<Tracer>>,
    active_workflows: Arc<AtomicU64>,
    token_count: Arc<AtomicU64>,
}

impl YawlMetrics {
    /// Create a new YawlMetrics instance
    pub fn new(tracer: Tracer) -> Self {
        Self {
            tracer: Arc::new(parking_lot::Mutex::new(tracer)),
            active_workflows: Arc::new(AtomicU64::new(0)),
            token_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record workflow execution duration
    ///
    /// # Semantic Convention
    /// - Metric name: `yawl.workflow.duration`
    /// - Type: Histogram (milliseconds)
    /// - Attributes:
    ///   - `yawl.workflow.id`: Workflow identifier
    ///   - `yawl.workflow.status`: "success" | "failed" | "cancelled"
    pub fn record_workflow_duration(&self, workflow_id: &str, duration_ms: u64, status: &str) {
        let mut tracer = self.tracer.lock();

        let mut attributes = BTreeMap::new();
        attributes.insert("yawl.workflow.id".to_string(), workflow_id.to_string());
        attributes.insert("yawl.workflow.status".to_string(), status.to_string());

        let metric = Metric {
            name: "yawl.workflow.duration".to_string(),
            value: MetricValue::Histogram(vec![duration_ms]),
            timestamp_ms: knhk_otel::get_timestamp_ms(),
            attributes,
        };

        tracer.record_metric(metric);
    }

    /// Record task execution time
    ///
    /// # Semantic Convention
    /// - Metric name: `yawl.task.execution_time`
    /// - Type: Histogram (milliseconds)
    /// - Attributes:
    ///   - `yawl.task.id`: Task identifier
    ///   - `yawl.task.pattern`: Pattern type
    ///   - `yawl.task.status`: "success" | "failed"
    pub fn record_task_execution_time(
        &self,
        task_id: &str,
        pattern: PatternType,
        duration_ms: u64,
        status: &str,
    ) {
        let mut tracer = self.tracer.lock();

        let mut attributes = BTreeMap::new();
        attributes.insert("yawl.task.id".to_string(), task_id.to_string());
        attributes.insert("yawl.task.pattern".to_string(), pattern.as_str().to_string());
        attributes.insert(
            "yawl.task.pattern_number".to_string(),
            pattern.pattern_number().to_string(),
        );
        attributes.insert("yawl.task.status".to_string(), status.to_string());

        let metric = Metric {
            name: "yawl.task.execution_time".to_string(),
            value: MetricValue::Histogram(vec![duration_ms]),
            timestamp_ms: knhk_otel::get_timestamp_ms(),
            attributes,
        };

        tracer.record_metric(metric);
    }

    /// Increment active workflows gauge
    pub fn increment_active_workflows(&self) {
        self.active_workflows.fetch_add(1, Ordering::Relaxed);
        self.record_active_workflows_gauge();
    }

    /// Decrement active workflows gauge
    pub fn decrement_active_workflows(&self) {
        self.active_workflows.fetch_sub(1, Ordering::Relaxed);
        self.record_active_workflows_gauge();
    }

    /// Record active workflows gauge value
    fn record_active_workflows_gauge(&self) {
        let mut tracer = self.tracer.lock();
        let count = self.active_workflows.load(Ordering::Relaxed);

        let metric = Metric {
            name: "yawl.workflows.active".to_string(),
            value: MetricValue::Gauge(count as f64),
            timestamp_ms: knhk_otel::get_timestamp_ms(),
            attributes: BTreeMap::new(),
        };

        tracer.record_metric(metric);
    }

    /// Increment token count
    pub fn increment_token_count(&self, workflow_id: &str) {
        self.token_count.fetch_add(1, Ordering::Relaxed);
        self.record_token_count_gauge(workflow_id);
    }

    /// Decrement token count
    pub fn decrement_token_count(&self, workflow_id: &str) {
        self.token_count.fetch_sub(1, Ordering::Relaxed);
        self.record_token_count_gauge(workflow_id);
    }

    /// Record token count gauge
    fn record_token_count_gauge(&self, workflow_id: &str) {
        let mut tracer = self.tracer.lock();
        let count = self.token_count.load(Ordering::Relaxed);

        let mut attributes = BTreeMap::new();
        attributes.insert("yawl.workflow.id".to_string(), workflow_id.to_string());

        let metric = Metric {
            name: "yawl.tokens.count".to_string(),
            value: MetricValue::Gauge(count as f64),
            timestamp_ms: knhk_otel::get_timestamp_ms(),
            attributes,
        };

        tracer.record_metric(metric);
    }

    /// Record pattern execution count
    ///
    /// # Semantic Convention
    /// - Metric name: `yawl.pattern.executions`
    /// - Type: Counter
    /// - Attributes:
    ///   - `yawl.pattern.type`: Pattern type name
    ///   - `yawl.pattern.number`: W3C pattern number
    pub fn record_pattern_execution(&self, pattern: PatternType) {
        let mut tracer = self.tracer.lock();

        let mut attributes = BTreeMap::new();
        attributes.insert("yawl.pattern.type".to_string(), pattern.as_str().to_string());
        attributes.insert(
            "yawl.pattern.number".to_string(),
            pattern.pattern_number().to_string(),
        );

        let metric = Metric {
            name: "yawl.pattern.executions".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: knhk_otel::get_timestamp_ms(),
            attributes,
        };

        tracer.record_metric(metric);
    }

    /// Record actor message latency
    ///
    /// # Semantic Convention
    /// - Metric name: `yawl.actor.message_latency`
    /// - Type: Histogram (microseconds)
    /// - Attributes:
    ///   - `yawl.actor.id`: Actor identifier
    ///   - `yawl.message.type`: Message type
    pub fn record_actor_message_latency(
        &self,
        actor_id: &str,
        message_type: &str,
        latency_us: u64,
    ) {
        let mut tracer = self.tracer.lock();

        let mut attributes = BTreeMap::new();
        attributes.insert("yawl.actor.id".to_string(), actor_id.to_string());
        attributes.insert("yawl.message.type".to_string(), message_type.to_string());

        let metric = Metric {
            name: "yawl.actor.message_latency".to_string(),
            value: MetricValue::Histogram(vec![latency_us]),
            timestamp_ms: knhk_otel::get_timestamp_ms(),
            attributes,
        };

        tracer.record_metric(metric);
    }

    /// Record error count
    ///
    /// # Semantic Convention
    /// - Metric name: `yawl.errors`
    /// - Type: Counter
    /// - Attributes:
    ///   - `yawl.error.type`: Error type/pattern
    ///   - `yawl.error.severity`: "warning" | "error" | "critical"
    pub fn record_error(&self, error_type: &str, severity: &str, pattern: Option<PatternType>) {
        let mut tracer = self.tracer.lock();

        let mut attributes = BTreeMap::new();
        attributes.insert("yawl.error.type".to_string(), error_type.to_string());
        attributes.insert("yawl.error.severity".to_string(), severity.to_string());

        if let Some(p) = pattern {
            attributes.insert("yawl.pattern.type".to_string(), p.as_str().to_string());
        }

        let metric = Metric {
            name: "yawl.errors".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: knhk_otel::get_timestamp_ms(),
            attributes,
        };

        tracer.record_metric(metric);
    }

    /// Get the underlying tracer (for testing)
    #[cfg(test)]
    pub fn tracer(&self) -> Arc<parking_lot::Mutex<Tracer>> {
        self.tracer.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_workflow_duration() {
        let tracer = Tracer::new();
        let metrics = YawlMetrics::new(tracer);

        metrics.record_workflow_duration("wf-001", 1500, "success");

        let tracer = metrics.tracer.lock();
        let recorded_metrics = tracer.metrics();
        assert_eq!(recorded_metrics.len(), 1);
        assert_eq!(recorded_metrics[0].name, "yawl.workflow.duration");
    }

    #[test]
    fn test_record_task_execution_time() {
        let tracer = Tracer::new();
        let metrics = YawlMetrics::new(tracer);

        metrics.record_task_execution_time("task-001", PatternType::Sequence, 250, "success");

        let tracer = metrics.tracer.lock();
        let recorded_metrics = tracer.metrics();
        assert_eq!(recorded_metrics.len(), 1);
        assert_eq!(recorded_metrics[0].name, "yawl.task.execution_time");
        assert_eq!(
            recorded_metrics[0].attributes.get("yawl.task.pattern"),
            Some(&"Sequence".to_string())
        );
    }

    #[test]
    fn test_active_workflows_gauge() {
        let tracer = Tracer::new();
        let metrics = YawlMetrics::new(tracer);

        metrics.increment_active_workflows();
        metrics.increment_active_workflows();

        assert_eq!(metrics.active_workflows.load(Ordering::Relaxed), 2);

        metrics.decrement_active_workflows();

        assert_eq!(metrics.active_workflows.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_token_count_gauge() {
        let tracer = Tracer::new();
        let metrics = YawlMetrics::new(tracer);

        metrics.increment_token_count("wf-001");
        metrics.increment_token_count("wf-001");
        metrics.increment_token_count("wf-001");

        assert_eq!(metrics.token_count.load(Ordering::Relaxed), 3);

        metrics.decrement_token_count("wf-001");

        assert_eq!(metrics.token_count.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_record_pattern_execution() {
        let tracer = Tracer::new();
        let metrics = YawlMetrics::new(tracer);

        metrics.record_pattern_execution(PatternType::ParallelSplit);
        metrics.record_pattern_execution(PatternType::Synchronization);

        let tracer = metrics.tracer.lock();
        let recorded_metrics = tracer.metrics();
        assert_eq!(recorded_metrics.len(), 2);
        assert_eq!(recorded_metrics[0].name, "yawl.pattern.executions");
        assert_eq!(
            recorded_metrics[0].attributes.get("yawl.pattern.type"),
            Some(&"ParallelSplit".to_string())
        );
    }

    #[test]
    fn test_record_actor_message_latency() {
        let tracer = Tracer::new();
        let metrics = YawlMetrics::new(tracer);

        metrics.record_actor_message_latency("actor-001", "task.execute", 150);

        let tracer = metrics.tracer.lock();
        let recorded_metrics = tracer.metrics();
        assert_eq!(recorded_metrics.len(), 1);
        assert_eq!(recorded_metrics[0].name, "yawl.actor.message_latency");
    }

    #[test]
    fn test_record_error() {
        let tracer = Tracer::new();
        let metrics = YawlMetrics::new(tracer);

        metrics.record_error("validation_failed", "error", Some(PatternType::ExclusiveChoice));

        let tracer = metrics.tracer.lock();
        let recorded_metrics = tracer.metrics();
        assert_eq!(recorded_metrics.len(), 1);
        assert_eq!(recorded_metrics[0].name, "yawl.errors");
        assert_eq!(
            recorded_metrics[0].attributes.get("yawl.error.type"),
            Some(&"validation_failed".to_string())
        );
    }
}
