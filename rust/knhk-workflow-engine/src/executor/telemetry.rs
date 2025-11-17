//! OpenTelemetry integration for workflow execution
//!
//! Provides full observability for workflow execution following Covenant 6:
//! **Observations Drive Everything**.
//!
//! # Covenant 6 Compliance
//!
//! - Every workflow/task event is observable
//! - Telemetry conforms to declared schema
//! - All state transitions emit events
//! - Metrics track performance bounds (Covenant 5)
//! - Traces provide full execution context
//!
//! # Telemetry Categories
//!
//! 1. **Workflow Events**: Started, completed, failed, cancelled
//! 2. **Task Events**: Enabled, started, completed, failed
//! 3. **Metrics**: Latency, throughput, resource usage
//! 4. **Traces**: Full execution context with spans

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, span, warn, Level};

/// Workflow telemetry emitter
pub struct WorkflowTelemetry {
    /// Workflow instance ID
    instance_id: String,
}

/// Workflow-level events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    /// Workflow started
    Started {
        workflow_id: String,
        instance_id: String,
    },
    /// Workflow completed successfully
    Completed {
        workflow_id: String,
        instance_id: String,
    },
    /// Workflow failed
    Failed {
        workflow_id: String,
        instance_id: String,
        error: String,
    },
    /// Workflow cancelled
    Cancelled {
        workflow_id: String,
        instance_id: String,
    },
}

/// Task-level events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEvent {
    /// Task enabled (ready to execute)
    Enabled {
        task_id: String,
        instance_id: String,
    },
    /// Task started execution
    Started {
        task_id: String,
        instance_id: String,
    },
    /// Task completed successfully
    Completed {
        task_id: String,
        instance_id: String,
        duration: Option<Duration>,
    },
    /// Task failed
    Failed {
        task_id: String,
        instance_id: String,
        error: Option<String>,
    },
    /// Task cancelled
    Cancelled {
        task_id: String,
        instance_id: String,
    },
}

impl WorkflowTelemetry {
    /// Create new telemetry emitter
    pub fn new(instance_id: String) -> Self {
        Self { instance_id }
    }

    /// Emit workflow event
    ///
    /// # OpenTelemetry Integration
    ///
    /// This should emit:
    /// - Structured log event
    /// - Metric increment (workflow_events_total)
    /// - Trace span event
    pub async fn emit_workflow_event(&self, event: WorkflowEvent) {
        match event {
            WorkflowEvent::Started {
                ref workflow_id,
                ref instance_id,
            } => {
                info!(
                    workflow_id = %workflow_id,
                    instance_id = %instance_id,
                    "Workflow started"
                );
                // Emit OTEL metric
                metrics::counter!("workflow_started_total",
                    "workflow_id" => workflow_id.clone(),
                    "instance_id" => instance_id.clone()
                )
                .increment(1);
            }
            WorkflowEvent::Completed {
                ref workflow_id,
                ref instance_id,
            } => {
                info!(
                    workflow_id = %workflow_id,
                    instance_id = %instance_id,
                    "Workflow completed"
                );
                // Emit OTEL metric
                metrics::counter!("workflow_completed_total",
                    "workflow_id" => workflow_id.clone(),
                    "instance_id" => instance_id.clone()
                )
                .increment(1);
            }
            WorkflowEvent::Failed {
                ref workflow_id,
                ref instance_id,
                ref error,
            } => {
                warn!(
                    workflow_id = %workflow_id,
                    instance_id = %instance_id,
                    error = %error,
                    "Workflow failed"
                );
                // Emit OTEL metric
                metrics::counter!("workflow_failed_total",
                    "workflow_id" => workflow_id.clone(),
                    "instance_id" => instance_id.clone(),
                    "error" => error.clone()
                )
                .increment(1);
            }
            WorkflowEvent::Cancelled {
                ref workflow_id,
                ref instance_id,
            } => {
                info!(
                    workflow_id = %workflow_id,
                    instance_id = %instance_id,
                    "Workflow cancelled"
                );
                // Emit OTEL metric
                metrics::counter!("workflow_cancelled_total",
                    "workflow_id" => workflow_id.clone(),
                    "instance_id" => instance_id.clone()
                )
                .increment(1);
            }
        }
    }

    /// Emit task event
    ///
    /// # OpenTelemetry Integration
    ///
    /// This should emit:
    /// - Structured log event
    /// - Metric increment (task_events_total)
    /// - Trace span event
    /// - Latency histogram (for completed tasks)
    pub async fn emit_task_event(&self, event: TaskEvent) {
        match event {
            TaskEvent::Enabled {
                ref task_id,
                ref instance_id,
            } => {
                debug!(
                    task_id = %task_id,
                    instance_id = %instance_id,
                    "Task enabled"
                );
                // Emit OTEL metric
                metrics::counter!("task_enabled_total",
                    "task_id" => task_id.clone(),
                    "instance_id" => instance_id.clone()
                )
                .increment(1);
            }
            TaskEvent::Started {
                ref task_id,
                ref instance_id,
            } => {
                info!(
                    task_id = %task_id,
                    instance_id = %instance_id,
                    "Task started"
                );
                // Emit OTEL metric
                metrics::counter!("task_started_total",
                    "task_id" => task_id.clone(),
                    "instance_id" => instance_id.clone()
                )
                .increment(1);
            }
            TaskEvent::Completed {
                ref task_id,
                ref instance_id,
                duration,
            } => {
                info!(
                    task_id = %task_id,
                    instance_id = %instance_id,
                    duration_us = duration.map(|d| d.as_micros()),
                    "Task completed"
                );
                // Emit OTEL metrics
                metrics::counter!("task_completed_total",
                    "task_id" => task_id.clone(),
                    "instance_id" => instance_id.clone()
                )
                .increment(1);

                if let Some(d) = duration {
                    metrics::histogram!("task_duration_seconds",
                        "task_id" => task_id.clone(),
                        "instance_id" => instance_id.clone()
                    )
                    .record(d.as_secs_f64());

                    // Covenant 5: Check against Chatman constant (8 ticks)
                    if d.as_nanos() > 8 {
                        warn!(
                            "Task {} exceeded 8 tick limit: {} ns",
                            task_id,
                            d.as_nanos()
                        );
                        metrics::counter!("task_chatman_violations_total",
                            "task_id" => task_id.clone()
                        )
                        .increment(1);
                    }
                }
            }
            TaskEvent::Failed {
                ref task_id,
                ref instance_id,
                ref error,
            } => {
                warn!(
                    task_id = %task_id,
                    instance_id = %instance_id,
                    error = ?error,
                    "Task failed"
                );
                // Emit OTEL metric
                metrics::counter!("task_failed_total",
                    "task_id" => task_id.clone(),
                    "instance_id" => instance_id.clone(),
                    "error" => error.as_ref().unwrap_or(&"unknown".to_string()).clone()
                )
                .increment(1);
            }
            TaskEvent::Cancelled {
                ref task_id,
                ref instance_id,
            } => {
                info!(
                    task_id = %task_id,
                    instance_id = %instance_id,
                    "Task cancelled"
                );
                // Emit OTEL metric
                metrics::counter!("task_cancelled_total",
                    "task_id" => task_id.clone(),
                    "instance_id" => instance_id.clone()
                )
                .increment(1);
            }
        }
    }

    /// Create trace span for workflow execution
    ///
    /// # Returns
    ///
    /// A tracing span that should be entered for the duration of workflow execution.
    pub fn create_workflow_span(&self, workflow_id: &str) -> tracing::Span {
        span!(
            Level::INFO,
            "workflow_execution",
            workflow_id = %workflow_id,
            instance_id = %self.instance_id,
        )
    }

    /// Create trace span for task execution
    ///
    /// # Returns
    ///
    /// A tracing span that should be entered for the duration of task execution.
    pub fn create_task_span(&self, task_id: &str) -> tracing::Span {
        span!(
            Level::INFO,
            "task_execution",
            task_id = %task_id,
            instance_id = %self.instance_id,
        )
    }

    /// Record metric for state transition latency
    ///
    /// # Covenant 5: Chatman Constant
    ///
    /// This records latency and checks against the 8-tick bound.
    pub fn record_transition_latency(&self, operation: &str, duration: Duration) {
        debug!(
            operation = %operation,
            duration_ns = duration.as_nanos(),
            "State transition latency"
        );

        // Emit OTEL histogram
        metrics::histogram!("workflow_transition_duration_seconds",
            "operation" => operation.to_string()
        )
        .record(duration.as_secs_f64());

        // Covenant 5: Check against Chatman constant
        const CHATMAN_CONSTANT_NS: u128 = 8;
        if duration.as_nanos() > CHATMAN_CONSTANT_NS {
            warn!(
                operation = %operation,
                duration_ns = duration.as_nanos(),
                "State transition exceeded 8-tick limit"
            );
            metrics::counter!("workflow_chatman_violations_total",
                "operation" => operation.to_string()
            )
            .increment(1);
        }
    }

    /// Record metric for workflow throughput
    pub fn record_throughput(&self, tasks_completed: usize, elapsed: Duration) {
        let throughput = tasks_completed as f64 / elapsed.as_secs_f64();
        debug!(
            tasks_completed = tasks_completed,
            elapsed_s = elapsed.as_secs_f64(),
            throughput = throughput,
            "Workflow throughput"
        );

        // Emit OTEL gauge
        metrics::gauge!("workflow_throughput_tasks_per_second").set(throughput);
    }

    /// Record resource usage metrics
    pub fn record_resource_usage(&self, memory_bytes: usize, cpu_percent: f64) {
        debug!(
            memory_bytes = memory_bytes,
            cpu_percent = cpu_percent,
            "Resource usage"
        );

        // Emit OTEL gauges
        metrics::gauge!("workflow_memory_bytes").set(memory_bytes as f64);
        metrics::gauge!("workflow_cpu_percent").set(cpu_percent);
    }
}

/// Helper to generate UUID v4
mod uuid {
    use rand::Rng;

    pub fn uuid4() -> String {
        let mut rng = rand::thread_rng();
        let bytes: [u8; 16] = rng.gen();

        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-4{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_events() {
        let telemetry = WorkflowTelemetry::new("test-instance".to_string());

        telemetry
            .emit_workflow_event(WorkflowEvent::Started {
                workflow_id: "test-workflow".to_string(),
                instance_id: "test-instance".to_string(),
            })
            .await;

        telemetry
            .emit_workflow_event(WorkflowEvent::Completed {
                workflow_id: "test-workflow".to_string(),
                instance_id: "test-instance".to_string(),
            })
            .await;
    }

    #[tokio::test]
    async fn test_task_events() {
        let telemetry = WorkflowTelemetry::new("test-instance".to_string());

        telemetry
            .emit_task_event(TaskEvent::Enabled {
                task_id: "task1".to_string(),
                instance_id: "test-instance".to_string(),
            })
            .await;

        telemetry
            .emit_task_event(TaskEvent::Started {
                task_id: "task1".to_string(),
                instance_id: "test-instance".to_string(),
            })
            .await;

        telemetry
            .emit_task_event(TaskEvent::Completed {
                task_id: "task1".to_string(),
                instance_id: "test-instance".to_string(),
                duration: Some(Duration::from_micros(5)),
            })
            .await;
    }

    #[test]
    fn test_transition_latency_within_bound() {
        let telemetry = WorkflowTelemetry::new("test-instance".to_string());
        // Should not warn for duration within 8 ticks
        telemetry.record_transition_latency("test_op", Duration::from_nanos(7));
    }

    #[test]
    fn test_transition_latency_exceeds_bound() {
        let telemetry = WorkflowTelemetry::new("test-instance".to_string());
        // Should warn for duration exceeding 8 ticks
        telemetry.record_transition_latency("slow_op", Duration::from_nanos(100));
    }
}
