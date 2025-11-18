// rust/knhk-yawl/src/telemetry/events.rs
// Structured workflow events for YAWL
//
// DOCTRINE ALIGNMENT:
// - Covenant 6: Observations Drive Everything
//   All workflow state changes are captured as structured events

use crate::{TaskId, WorkflowId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Workflow event type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowEventType {
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowCancelled,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TokenCreated,
    TokenConsumed,
    TransitionFired,
    ErrorOccurred,
}

/// Workflow event
///
/// This struct represents a structured event in the workflow execution.
/// Events are recorded as span events and can be queried/analyzed later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    /// Event type
    pub event_type: WorkflowEventType,

    /// Event timestamp (milliseconds since Unix epoch)
    pub timestamp_ms: u64,

    /// Event attributes (contextual information)
    pub attributes: BTreeMap<String, String>,
}

impl WorkflowEvent {
    /// Create a new workflow event
    pub fn new(event_type: WorkflowEventType) -> Self {
        Self {
            event_type,
            timestamp_ms: knhk_otel::get_timestamp_ms(),
            attributes: BTreeMap::new(),
        }
    }

    /// Add an attribute to the event
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Create a workflow started event
    pub fn workflow_started(workflow_id: &str) -> Self {
        Self::new(WorkflowEventType::WorkflowStarted)
            .with_attribute("yawl.workflow.id", workflow_id)
    }

    /// Create a workflow completed event
    pub fn workflow_completed(workflow_id: &str, duration_ms: u64) -> Self {
        Self::new(WorkflowEventType::WorkflowCompleted)
            .with_attribute("yawl.workflow.id", workflow_id)
            .with_attribute("yawl.workflow.duration_ms", duration_ms.to_string())
    }

    /// Create a workflow failed event
    pub fn workflow_failed(workflow_id: &str, error: &str) -> Self {
        Self::new(WorkflowEventType::WorkflowFailed)
            .with_attribute("yawl.workflow.id", workflow_id)
            .with_attribute("yawl.error.message", error)
    }

    /// Create a task started event
    pub fn task_started(task_id: &str, pattern_type: &str) -> Self {
        Self::new(WorkflowEventType::TaskStarted)
            .with_attribute("yawl.task.id", task_id)
            .with_attribute("yawl.task.pattern", pattern_type)
    }

    /// Create a task completed event
    pub fn task_completed(task_id: &str, duration_ms: u64, output: Option<String>) -> Self {
        let mut event = Self::new(WorkflowEventType::TaskCompleted)
            .with_attribute("yawl.task.id", task_id)
            .with_attribute("yawl.task.duration_ms", duration_ms.to_string());

        if let Some(output_data) = output {
            event = event.with_attribute("yawl.task.output", output_data);
        }

        event
    }

    /// Create a task failed event
    pub fn task_failed(task_id: &str, error: &str) -> Self {
        Self::new(WorkflowEventType::TaskFailed)
            .with_attribute("yawl.task.id", task_id)
            .with_attribute("yawl.error.message", error)
    }

    /// Create a token created event
    pub fn token_created(token_id: &str, source_task: &str) -> Self {
        Self::new(WorkflowEventType::TokenCreated)
            .with_attribute("yawl.token.id", token_id)
            .with_attribute("yawl.token.source_task", source_task)
    }

    /// Create a token consumed event
    pub fn token_consumed(token_id: &str, consuming_task: &str) -> Self {
        Self::new(WorkflowEventType::TokenConsumed)
            .with_attribute("yawl.token.id", token_id)
            .with_attribute("yawl.token.consuming_task", consuming_task)
    }

    /// Create a transition fired event
    pub fn transition_fired(from: TaskId, to: TaskId, condition: Option<String>) -> Self {
        let mut event = Self::new(WorkflowEventType::TransitionFired)
            .with_attribute("yawl.transition.from", from.0)
            .with_attribute("yawl.transition.to", to.0);

        if let Some(cond) = condition {
            event = event.with_attribute("yawl.transition.condition", cond);
        }

        event
    }

    /// Create an error occurred event
    pub fn error_occurred(error_type: &str, message: &str, context: Option<String>) -> Self {
        let mut event = Self::new(WorkflowEventType::ErrorOccurred)
            .with_attribute("yawl.error.type", error_type)
            .with_attribute("yawl.error.message", message);

        if let Some(ctx) = context {
            event = event.with_attribute("yawl.error.context", ctx);
        }

        event
    }
}

/// Helper struct for transition fired events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionFired {
    pub from: TaskId,
    pub to: TaskId,
    pub condition: Option<String>,
    pub timestamp_ms: u64,
}

impl TransitionFired {
    pub fn new(from: TaskId, to: TaskId, condition: Option<String>) -> Self {
        Self {
            from,
            to,
            condition,
            timestamp_ms: knhk_otel::get_timestamp_ms(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_started_event() {
        let event = WorkflowEvent::workflow_started("wf-001");

        assert_eq!(event.event_type, WorkflowEventType::WorkflowStarted);
        assert_eq!(
            event.attributes.get("yawl.workflow.id"),
            Some(&"wf-001".to_string())
        );
    }

    #[test]
    fn test_workflow_completed_event() {
        let event = WorkflowEvent::workflow_completed("wf-001", 1500);

        assert_eq!(event.event_type, WorkflowEventType::WorkflowCompleted);
        assert_eq!(
            event.attributes.get("yawl.workflow.id"),
            Some(&"wf-001".to_string())
        );
        assert_eq!(
            event.attributes.get("yawl.workflow.duration_ms"),
            Some(&"1500".to_string())
        );
    }

    #[test]
    fn test_task_started_event() {
        let event = WorkflowEvent::task_started("task-001", "Sequence");

        assert_eq!(event.event_type, WorkflowEventType::TaskStarted);
        assert_eq!(event.attributes.get("yawl.task.id"), Some(&"task-001".to_string()));
        assert_eq!(
            event.attributes.get("yawl.task.pattern"),
            Some(&"Sequence".to_string())
        );
    }

    #[test]
    fn test_task_completed_event() {
        let output = Some("{\"result\": \"success\"}".to_string());
        let event = WorkflowEvent::task_completed("task-001", 250, output);

        assert_eq!(event.event_type, WorkflowEventType::TaskCompleted);
        assert_eq!(event.attributes.get("yawl.task.id"), Some(&"task-001".to_string()));
        assert_eq!(
            event.attributes.get("yawl.task.duration_ms"),
            Some(&"250".to_string())
        );
        assert_eq!(
            event.attributes.get("yawl.task.output"),
            Some(&"{\"result\": \"success\"}".to_string())
        );
    }

    #[test]
    fn test_token_created_event() {
        let event = WorkflowEvent::token_created("token-001", "task-001");

        assert_eq!(event.event_type, WorkflowEventType::TokenCreated);
        assert_eq!(
            event.attributes.get("yawl.token.id"),
            Some(&"token-001".to_string())
        );
        assert_eq!(
            event.attributes.get("yawl.token.source_task"),
            Some(&"task-001".to_string())
        );
    }

    #[test]
    fn test_transition_fired_event() {
        let from = TaskId("task-001".to_string());
        let to = TaskId("task-002".to_string());
        let condition = Some("amount > 1000".to_string());

        let event = WorkflowEvent::transition_fired(from, to, condition);

        assert_eq!(event.event_type, WorkflowEventType::TransitionFired);
        assert_eq!(
            event.attributes.get("yawl.transition.from"),
            Some(&"task-001".to_string())
        );
        assert_eq!(
            event.attributes.get("yawl.transition.to"),
            Some(&"task-002".to_string())
        );
        assert_eq!(
            event.attributes.get("yawl.transition.condition"),
            Some(&"amount > 1000".to_string())
        );
    }

    #[test]
    fn test_error_occurred_event() {
        let context = Some("task-001".to_string());
        let event = WorkflowEvent::error_occurred("validation_failed", "Invalid input", context);

        assert_eq!(event.event_type, WorkflowEventType::ErrorOccurred);
        assert_eq!(
            event.attributes.get("yawl.error.type"),
            Some(&"validation_failed".to_string())
        );
        assert_eq!(
            event.attributes.get("yawl.error.message"),
            Some(&"Invalid input".to_string())
        );
        assert_eq!(
            event.attributes.get("yawl.error.context"),
            Some(&"task-001".to_string())
        );
    }

    #[test]
    fn test_event_builder_pattern() {
        let event = WorkflowEvent::new(WorkflowEventType::TaskStarted)
            .with_attribute("yawl.task.id", "task-001")
            .with_attribute("yawl.task.pattern", "ParallelSplit")
            .with_attribute("custom.attribute", "custom value");

        assert_eq!(event.event_type, WorkflowEventType::TaskStarted);
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(
            event.attributes.get("custom.attribute"),
            Some(&"custom value".to_string())
        );
    }
}
