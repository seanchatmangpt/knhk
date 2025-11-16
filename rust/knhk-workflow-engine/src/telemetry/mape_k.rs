// rust/knhk-workflow-engine/src/telemetry/mape_k.rs
// MAPE-K autonomic feedback loop telemetry integration
// Covenant 6: Observations Drive Everything

//! # MAPE-K Autonomic Feedback
//!
//! Telemetry emission for the MAPE-K autonomic knowledge feedback loop.
//! This enables self-healing, self-optimizing, and self-learning workflows.
//!
//! ## MAPE-K Components
//!
//! 1. **Monitor**: Observe system state and detect anomalies
//! 2. **Analyze**: Diagnose root causes using pattern matching
//! 3. **Plan**: Decide on remediation actions using policies
//! 4. **Execute**: Perform actions and observe effects
//! 5. **Knowledge**: Learn from experience and update patterns
//!
//! ## Integration with Workflow Telemetry
//!
//! Workflow execution telemetry (from `emit.rs`) feeds into the Monitor component.
//! MAPE-K telemetry (this module) tracks the feedback loop's decisions and actions.

use super::TelemetryContext;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// MAPE-K component identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapekComponent {
    Monitor,
    Analyze,
    Plan,
    Execute,
    Knowledge,
}

impl MapekComponent {
    pub fn as_str(&self) -> &'static str {
        match self {
            MapekComponent::Monitor => "Monitor",
            MapekComponent::Analyze => "Analyze",
            MapekComponent::Plan => "Plan",
            MapekComponent::Execute => "Execute",
            MapekComponent::Knowledge => "Knowledge",
        }
    }
}

/// MAPE-K cycle tracker
pub struct MapekCycle {
    /// Unique cycle ID
    pub cycle_id: String,
    /// What triggered this cycle
    pub triggered_by: String,
    /// Telemetry context
    pub context: TelemetryContext,
    /// Start time (milliseconds)
    pub start_time_ms: u64,
}

impl MapekCycle {
    /// Create a new MAPE-K cycle
    pub fn new(triggered_by: String, context: TelemetryContext) -> Self {
        let cycle_id = format!("mapek-{}", uuid::Uuid::new_v4());
        let start_time_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            cycle_id,
            triggered_by,
            context,
            start_time_ms,
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        now_ms - self.start_time_ms
    }
}

/// Emit MAPE-K Monitor telemetry
///
/// Called when monitoring detects an observation worth analyzing.
/// Maps to span: `knhk.mapek.monitor`
pub fn emit_monitor_observation(
    cycle: &MapekCycle,
    observation_type: &str,
    metric_name: &str,
    metric_value: f64,
    threshold_breached: bool,
    threshold_value: Option<f64>,
    severity: &str,
    anomaly_detected: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = HashMap::new();
    attributes.insert("knhk.mapek.component".to_string(), "Monitor".to_string());
    attributes.insert("knhk.mapek.observation_type".to_string(), observation_type.to_string());
    attributes.insert("knhk.mapek.metric_name".to_string(), metric_name.to_string());
    attributes.insert("knhk.mapek.metric_value".to_string(), metric_value.to_string());
    attributes.insert("knhk.mapek.threshold_breached".to_string(), threshold_breached.to_string());
    attributes.insert("knhk.mapek.severity".to_string(), severity.to_string());
    attributes.insert("knhk.mapek.anomaly_detected".to_string(), anomaly_detected.to_string());

    if let Some(threshold) = threshold_value {
        attributes.insert("knhk.mapek.threshold_value".to_string(), threshold.to_string());
    }

    emit_mapek_span(cycle, "knhk.mapek.monitor", attributes)?;

    // Record anomaly metric if detected
    if anomaly_detected {
        let mut metric_attrs = HashMap::new();
        metric_attrs.insert("knhk.mapek.observation_type".to_string(), observation_type.to_string());
        metric_attrs.insert("knhk.mapek.severity".to_string(), severity.to_string());
        emit_counter("knhk.mapek.anomaly_count", 1, metric_attrs)?;
    }

    Ok(())
}

/// Emit MAPE-K Analyze telemetry
///
/// Called when analysis identifies a root cause.
/// Maps to span: `knhk.mapek.analyze`
pub fn emit_analyze_diagnosis(
    cycle: &MapekCycle,
    pattern_matched: &str,
    root_cause: &str,
    confidence: f64,
    sparql_query: Option<&str>,
    analysis_duration_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = HashMap::new();
    attributes.insert("knhk.mapek.component".to_string(), "Analyze".to_string());
    attributes.insert("knhk.mapek.pattern_matched".to_string(), pattern_matched.to_string());
    attributes.insert("knhk.mapek.root_cause".to_string(), root_cause.to_string());
    attributes.insert("knhk.mapek.confidence".to_string(), confidence.to_string());
    attributes.insert("knhk.mapek.analysis_duration_ms".to_string(), analysis_duration_ms.to_string());

    if let Some(query) = sparql_query {
        attributes.insert("knhk.mapek.sparql_query".to_string(), query.to_string());
    }

    emit_mapek_span(cycle, "knhk.mapek.analyze", attributes)
}

/// Emit MAPE-K Plan telemetry
///
/// Called when planning selects actions.
/// Maps to span: `knhk.mapek.plan`
pub fn emit_plan_actions(
    cycle: &MapekCycle,
    policy_applied: &str,
    action_planned: &str,
    action_sequence: Vec<String>,
    risk_level: &str,
    approval_required: bool,
    historical_success_rate: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = HashMap::new();
    attributes.insert("knhk.mapek.component".to_string(), "Plan".to_string());
    attributes.insert("knhk.mapek.policy_applied".to_string(), policy_applied.to_string());
    attributes.insert("knhk.mapek.action_planned".to_string(), action_planned.to_string());
    attributes.insert("knhk.mapek.action_sequence".to_string(), action_sequence.join(","));
    attributes.insert("knhk.mapek.risk_level".to_string(), risk_level.to_string());
    attributes.insert("knhk.mapek.approval_required".to_string(), approval_required.to_string());
    attributes.insert("knhk.mapek.historical_success_rate".to_string(), historical_success_rate.to_string());

    emit_mapek_span(cycle, "knhk.mapek.plan", attributes)
}

/// Emit MAPE-K Execute telemetry
///
/// Called when executing an action.
/// Maps to span: `knhk.mapek.execute`
pub fn emit_execute_action(
    cycle: &MapekCycle,
    action_executed: &str,
    execution_status: &str,
    execution_duration_ms: u64,
    side_effects: &str,
    rollback_required: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = HashMap::new();
    attributes.insert("knhk.mapek.component".to_string(), "Execute".to_string());
    attributes.insert("knhk.mapek.action_executed".to_string(), action_executed.to_string());
    attributes.insert("knhk.mapek.execution_status".to_string(), execution_status.to_string());
    attributes.insert("knhk.mapek.execution_duration_ms".to_string(), execution_duration_ms.to_string());
    attributes.insert("knhk.mapek.side_effects".to_string(), side_effects.to_string());
    attributes.insert("knhk.mapek.rollback_required".to_string(), rollback_required.to_string());

    emit_mapek_span(cycle, "knhk.mapek.execute", attributes)?;

    // Record remediation success/failure metric
    let success = execution_status == "success";
    let mut metric_attrs = HashMap::new();
    metric_attrs.insert("knhk.mapek.action_executed".to_string(), action_executed.to_string());
    let success_rate = if success { 1.0 } else { 0.0 };
    emit_gauge("knhk.mapek.remediation_success_rate", success_rate, metric_attrs)?;

    Ok(())
}

/// Emit MAPE-K Knowledge Update telemetry
///
/// Called when learning from experience.
/// Maps to span: `knhk.mapek.knowledge_update`
pub fn emit_knowledge_update(
    cycle: &MapekCycle,
    pattern_learned: Option<&str>,
    success_recorded: bool,
    knowledge_updated: &str,
    prediction_accuracy: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attributes = HashMap::new();
    attributes.insert("knhk.mapek.component".to_string(), "Knowledge".to_string());
    attributes.insert("knhk.mapek.success_recorded".to_string(), success_recorded.to_string());
    attributes.insert("knhk.mapek.knowledge_updated".to_string(), knowledge_updated.to_string());

    if let Some(pattern) = pattern_learned {
        attributes.insert("knhk.mapek.pattern_learned".to_string(), pattern.to_string());
    }

    if let Some(accuracy) = prediction_accuracy {
        attributes.insert("knhk.mapek.prediction_accuracy".to_string(), accuracy.to_string());
    }

    emit_mapek_span(cycle, "knhk.mapek.knowledge_update", attributes)?;

    // Record knowledge growth metric
    if pattern_learned.is_some() {
        emit_counter("knhk.mapek.knowledge_growth", 1, HashMap::new())?;
    }

    Ok(())
}

/// Emit complete MAPE-K cycle telemetry
///
/// Called when a full cycle completes.
/// Maps to span: `knhk.mapek.cycle`
pub fn emit_mapek_cycle_complete(
    cycle: &MapekCycle,
    outcome: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let duration_ms = cycle.elapsed_ms();

    let mut attributes = HashMap::new();
    attributes.insert("knhk.mapek.component".to_string(), "MAPE-K".to_string());
    attributes.insert("knhk.mapek.cycle_triggered_by".to_string(), cycle.triggered_by.clone());
    attributes.insert("knhk.mapek.cycle_duration_ms".to_string(), duration_ms.to_string());
    attributes.insert("knhk.mapek.cycle_outcome".to_string(), outcome.to_string());

    emit_mapek_span(cycle, "knhk.mapek.cycle", attributes)?;

    // Record cycle latency metric
    let mut metric_attrs = HashMap::new();
    metric_attrs.insert("knhk.mapek.cycle_triggered_by".to_string(), cycle.triggered_by.clone());
    emit_histogram("knhk.mapek.cycle_latency", duration_ms as f64, metric_attrs)?;

    Ok(())
}

// ============================================================================
// Internal helper functions
// ============================================================================

/// Emit MAPE-K span (internal)
fn emit_mapek_span(
    cycle: &MapekCycle,
    span_name: &str,
    attributes: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Reuse emit logic from emit.rs
    let span_json = serde_json::json!({
        "trace_id": cycle.context.trace_id,
        "span_id": uuid::Uuid::new_v4().to_string(),
        "parent_span_id": cycle.context.span_id,
        "name": span_name,
        "kind": "INTERNAL",
        "start_time_unix_nano": get_current_time_nanos(),
        "end_time_unix_nano": get_current_time_nanos(),
        "attributes": attributes,
        "status": { "code": "OK" }
    });

    println!("OTEL_SPAN: {}", span_json);

    Ok(())
}

/// Emit counter metric (internal)
fn emit_counter(
    metric_name: &str,
    value: i64,
    attributes: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let metric_json = serde_json::json!({
        "name": metric_name,
        "type": "counter",
        "value": value,
        "time_unix_nano": get_current_time_nanos(),
        "attributes": attributes,
    });

    println!("OTEL_METRIC: {}", metric_json);

    Ok(())
}

/// Emit gauge metric (internal)
fn emit_gauge(
    metric_name: &str,
    value: f64,
    attributes: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let metric_json = serde_json::json!({
        "name": metric_name,
        "type": "gauge",
        "value": value,
        "time_unix_nano": get_current_time_nanos(),
        "attributes": attributes,
    });

    println!("OTEL_METRIC: {}", metric_json);

    Ok(())
}

/// Emit histogram metric (internal)
fn emit_histogram(
    metric_name: &str,
    value: f64,
    attributes: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let metric_json = serde_json::json!({
        "name": metric_name,
        "type": "histogram",
        "value": value,
        "time_unix_nano": get_current_time_nanos(),
        "attributes": attributes,
    });

    println!("OTEL_METRIC: {}", metric_json);

    Ok(())
}

/// Get current time in nanoseconds
fn get_current_time_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Initialize MAPE-K telemetry subsystem
pub fn init_mapek() -> Result<(), Box<dyn std::error::Error>> {
    println!("MAPEK: Telemetry subsystem initialized");
    Ok(())
}

/// Shutdown MAPE-K telemetry subsystem
pub fn shutdown_mapek() -> Result<(), Box<dyn std::error::Error>> {
    println!("MAPEK: Telemetry subsystem shutdown");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapek_component_as_str() {
        assert_eq!(MapekComponent::Monitor.as_str(), "Monitor");
        assert_eq!(MapekComponent::Analyze.as_str(), "Analyze");
        assert_eq!(MapekComponent::Plan.as_str(), "Plan");
        assert_eq!(MapekComponent::Execute.as_str(), "Execute");
        assert_eq!(MapekComponent::Knowledge.as_str(), "Knowledge");
    }

    #[test]
    fn test_mapek_cycle_creation() {
        let ctx = TelemetryContext::new("trace-123".to_string(), "span-456".to_string());
        let cycle = MapekCycle::new("threshold_breach".to_string(), ctx);

        assert!(cycle.cycle_id.starts_with("mapek-"));
        assert_eq!(cycle.triggered_by, "threshold_breach");
        assert!(cycle.start_time_ms > 0);
    }

    #[test]
    fn test_mapek_cycle_elapsed() {
        let ctx = TelemetryContext::new("trace-123".to_string(), "span-456".to_string());
        let cycle = MapekCycle::new("test".to_string(), ctx);

        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = cycle.elapsed_ms();
        assert!(elapsed >= 10);
    }
}
