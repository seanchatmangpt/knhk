// rust/knhk-yawl/src/telemetry/hooks.rs
// Telemetry integration hooks for YAWL workflows
//
// DOCTRINE ALIGNMENT:
// - Covenant 6: Observations Drive Everything
//   Hooks ensure all workflow execution is automatically instrumented
//
// - Covenant 3: MAPE-K feedback loops
//   Hooks integrate with MAPE-K monitoring and analysis

use crate::{PatternType, TaskId, WorkflowId};
use knhk_otel::{Span, SpanContext, Tracer};
use std::sync::Arc;

/// Telemetry hooks for YAWL workflow execution
///
/// This struct provides callback hooks that can be attached to workflow
/// execution to automatically emit telemetry.
///
/// # Usage
/// ```ignore
/// let hooks = TelemetryHooks::new(tracer);
///
/// // Register hooks with workflow engine
/// engine.on_workflow_start(hooks.on_workflow_start());
/// engine.on_task_start(hooks.on_task_start());
/// engine.on_pattern_execute(hooks.on_pattern_execute());
/// ```
pub struct TelemetryHooks {
    tracer: Arc<parking_lot::Mutex<Tracer>>,
}

impl TelemetryHooks {
    /// Create new telemetry hooks
    pub fn new(tracer: Tracer) -> Self {
        Self {
            tracer: Arc::new(parking_lot::Mutex::new(tracer)),
        }
    }

    /// Hook: Workflow started
    ///
    /// Called when a workflow execution begins.
    /// Creates a workflow span and records workflow start event.
    pub fn on_workflow_start(&self, workflow_id: &str, workflow_name: &str) -> SpanContext {
        let span = crate::telemetry::spans::create_workflow_span(workflow_id, workflow_name);
        let context = span.context.clone();

        let mut tracer = self.tracer.lock();
        tracer.spans.push(span);

        // Record workflow started event
        let event = crate::telemetry::events::WorkflowEvent::workflow_started(workflow_id);
        let span_event = knhk_otel::SpanEvent {
            name: "yawl.workflow.started".to_string(),
            timestamp_ms: event.timestamp_ms,
            attributes: event.attributes,
        };

        if let Some(span) = tracer.spans.iter_mut().find(|s| s.context.span_id == context.span_id)
        {
            span.events.push(span_event);
        }

        context
    }

    /// Hook: Workflow completed
    ///
    /// Called when a workflow execution completes successfully.
    /// Ends the workflow span and records completion event.
    pub fn on_workflow_complete(&self, workflow_id: &str, context: SpanContext, duration_ms: u64) {
        let mut tracer = self.tracer.lock();

        // End the workflow span
        tracer.end_span(context.clone(), knhk_otel::SpanStatus::Ok);

        // Record workflow completed event
        let event =
            crate::telemetry::events::WorkflowEvent::workflow_completed(workflow_id, duration_ms);
        let span_event = knhk_otel::SpanEvent {
            name: "yawl.workflow.completed".to_string(),
            timestamp_ms: event.timestamp_ms,
            attributes: event.attributes,
        };

        if let Some(span) = tracer.spans.iter_mut().find(|s| s.context.span_id == context.span_id)
        {
            span.events.push(span_event);
        }
    }

    /// Hook: Workflow failed
    ///
    /// Called when a workflow execution fails.
    /// Ends the workflow span with error status and records failure event.
    pub fn on_workflow_failed(&self, workflow_id: &str, context: SpanContext, error: &str) {
        let mut tracer = self.tracer.lock();

        // End the workflow span with error status
        tracer.end_span(context.clone(), knhk_otel::SpanStatus::Error);

        // Record workflow failed event
        let event = crate::telemetry::events::WorkflowEvent::workflow_failed(workflow_id, error);
        let span_event = knhk_otel::SpanEvent {
            name: "yawl.workflow.failed".to_string(),
            timestamp_ms: event.timestamp_ms,
            attributes: event.attributes,
        };

        if let Some(span) = tracer.spans.iter_mut().find(|s| s.context.span_id == context.span_id)
        {
            span.events.push(span_event);
        }
    }

    /// Hook: Task started
    ///
    /// Called when a task execution begins.
    /// Creates a task span and records task start event.
    pub fn on_task_start(
        &self,
        task_id: &str,
        task_name: &str,
        pattern: PatternType,
        parent: SpanContext,
    ) -> SpanContext {
        let span = crate::telemetry::spans::create_task_span(
            task_id,
            task_name,
            pattern,
            Some(parent),
        );
        let context = span.context.clone();

        let mut tracer = self.tracer.lock();
        tracer.spans.push(span);

        // Record task started event
        let event =
            crate::telemetry::events::WorkflowEvent::task_started(task_id, pattern.as_str());
        let span_event = knhk_otel::SpanEvent {
            name: "yawl.task.started".to_string(),
            timestamp_ms: event.timestamp_ms,
            attributes: event.attributes,
        };

        if let Some(span) = tracer.spans.iter_mut().find(|s| s.context.span_id == context.span_id)
        {
            span.events.push(span_event);
        }

        context
    }

    /// Hook: Task completed
    ///
    /// Called when a task execution completes successfully.
    /// Ends the task span and records completion event.
    pub fn on_task_complete(
        &self,
        task_id: &str,
        context: SpanContext,
        duration_ms: u64,
        output: Option<String>,
    ) {
        let mut tracer = self.tracer.lock();

        // End the task span
        tracer.end_span(context.clone(), knhk_otel::SpanStatus::Ok);

        // Record task completed event
        let event =
            crate::telemetry::events::WorkflowEvent::task_completed(task_id, duration_ms, output);
        let span_event = knhk_otel::SpanEvent {
            name: "yawl.task.completed".to_string(),
            timestamp_ms: event.timestamp_ms,
            attributes: event.attributes,
        };

        if let Some(span) = tracer.spans.iter_mut().find(|s| s.context.span_id == context.span_id)
        {
            span.events.push(span_event);
        }
    }

    /// Hook: Pattern execution
    ///
    /// Called when a YAWL pattern is executed.
    /// Creates a pattern span.
    pub fn on_pattern_execute(&self, pattern: PatternType, parent: SpanContext) -> SpanContext {
        let span = crate::telemetry::spans::create_pattern_span(pattern, Some(parent));
        let context = span.context.clone();

        let mut tracer = self.tracer.lock();
        tracer.spans.push(span);

        context
    }

    /// Hook: Transition fired
    ///
    /// Called when a workflow transition fires.
    /// Records a transition event.
    pub fn on_transition_fired(
        &self,
        from: TaskId,
        to: TaskId,
        condition: Option<String>,
        parent: SpanContext,
    ) {
        let mut tracer = self.tracer.lock();

        // Record transition fired event
        let event =
            crate::telemetry::events::WorkflowEvent::transition_fired(from, to, condition);
        let span_event = knhk_otel::SpanEvent {
            name: "yawl.transition.fired".to_string(),
            timestamp_ms: event.timestamp_ms,
            attributes: event.attributes,
        };

        if let Some(span) = tracer.spans.iter_mut().find(|s| s.context.span_id == parent.span_id) {
            span.events.push(span_event);
        }
    }

    /// Export all telemetry to OTLP endpoint
    #[cfg(feature = "std")]
    pub fn export(&self) -> Result<(), String> {
        let mut tracer = self.tracer.lock();
        tracer.export()
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
    fn test_workflow_start_hook() {
        let tracer = Tracer::new();
        let hooks = TelemetryHooks::new(tracer);

        let context = hooks.on_workflow_start("wf-001", "Test Workflow");

        let tracer = hooks.tracer.lock();
        let spans = tracer.spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "yawl.workflow.execute");
        assert_eq!(
            spans[0].attributes.get("yawl.workflow.id"),
            Some(&"wf-001".to_string())
        );
    }

    #[test]
    fn test_workflow_complete_hook() {
        let tracer = Tracer::new();
        let hooks = TelemetryHooks::new(tracer);

        let context = hooks.on_workflow_start("wf-001", "Test Workflow");
        hooks.on_workflow_complete("wf-001", context, 1500);

        let tracer = hooks.tracer.lock();
        let spans = tracer.spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].status, knhk_otel::SpanStatus::Ok);
        assert!(spans[0].end_time_ms.is_some());
    }

    #[test]
    fn test_task_start_hook() {
        let tracer = Tracer::new();
        let hooks = TelemetryHooks::new(tracer);

        let workflow_context = hooks.on_workflow_start("wf-001", "Test Workflow");
        let task_context = hooks.on_task_start(
            "task-001",
            "Validate Order",
            PatternType::Sequence,
            workflow_context,
        );

        let tracer = hooks.tracer.lock();
        let spans = tracer.spans();
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[1].name, "yawl.task.execute");
        assert_eq!(spans[1].context.parent_span_id, Some(workflow_context.span_id));
    }

    #[test]
    fn test_task_complete_hook() {
        let tracer = Tracer::new();
        let hooks = TelemetryHooks::new(tracer);

        let workflow_context = hooks.on_workflow_start("wf-001", "Test Workflow");
        let task_context = hooks.on_task_start(
            "task-001",
            "Validate Order",
            PatternType::Sequence,
            workflow_context,
        );

        let output = Some("{\"valid\": true}".to_string());
        hooks.on_task_complete("task-001", task_context, 250, output);

        let tracer = hooks.tracer.lock();
        let spans = tracer.spans();
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[1].status, knhk_otel::SpanStatus::Ok);
        assert!(spans[1].end_time_ms.is_some());
    }

    #[test]
    fn test_pattern_execute_hook() {
        let tracer = Tracer::new();
        let hooks = TelemetryHooks::new(tracer);

        let workflow_context = hooks.on_workflow_start("wf-001", "Test Workflow");
        let pattern_context =
            hooks.on_pattern_execute(PatternType::ParallelSplit, workflow_context);

        let tracer = hooks.tracer.lock();
        let spans = tracer.spans();
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[1].name, "yawl.pattern.execute");
        assert_eq!(
            spans[1].attributes.get("yawl.pattern.type"),
            Some(&"ParallelSplit".to_string())
        );
    }

    #[test]
    fn test_transition_fired_hook() {
        let tracer = Tracer::new();
        let hooks = TelemetryHooks::new(tracer);

        let workflow_context = hooks.on_workflow_start("wf-001", "Test Workflow");

        let from = TaskId("task-001".to_string());
        let to = TaskId("task-002".to_string());
        let condition = Some("amount > 1000".to_string());

        hooks.on_transition_fired(from, to, condition, workflow_context);

        let tracer = hooks.tracer.lock();
        let spans = tracer.spans();
        assert_eq!(spans.len(), 1);
        assert!(!spans[0].events.is_empty());
        assert_eq!(spans[0].events[0].name, "yawl.workflow.started");
        assert_eq!(spans[0].events[1].name, "yawl.transition.fired");
    }

    #[test]
    fn test_complete_workflow_execution() {
        let tracer = Tracer::new();
        let hooks = TelemetryHooks::new(tracer);

        // Start workflow
        let wf_ctx = hooks.on_workflow_start("wf-001", "Order Processing");

        // Start task 1
        let task1_ctx =
            hooks.on_task_start("task-001", "Validate Order", PatternType::Sequence, wf_ctx);

        // Complete task 1
        hooks.on_task_complete("task-001", task1_ctx, 100, None);

        // Transition
        let from = TaskId("task-001".to_string());
        let to = TaskId("task-002".to_string());
        hooks.on_transition_fired(from, to, None, wf_ctx);

        // Start task 2
        let task2_ctx =
            hooks.on_task_start("task-002", "Process Payment", PatternType::Sequence, wf_ctx);

        // Complete task 2
        hooks.on_task_complete("task-002", task2_ctx, 200, None);

        // Complete workflow
        hooks.on_workflow_complete("wf-001", wf_ctx, 300);

        let tracer = hooks.tracer.lock();
        let spans = tracer.spans();
        assert_eq!(spans.len(), 3); // workflow + 2 tasks
        assert_eq!(spans[0].status, knhk_otel::SpanStatus::Ok);
        assert_eq!(spans[1].status, knhk_otel::SpanStatus::Ok);
        assert_eq!(spans[2].status, knhk_otel::SpanStatus::Ok);
    }
}
