//! OpenTelemetry integration
//!
//! Covenant 6: Observations drive everything (O ⊨ Discovery)
//!
//! This module provides OpenTelemetry hooks for full observability of YAWL workflows.

pub mod spans;

use tracing::{info, warn, error, debug};

/// Initialize telemetry for YAWL
///
/// Sets up OpenTelemetry instrumentation.
pub fn init() {
    info!("Initializing YAWL telemetry");
    // In production, would set up OTel exporters, resource detection, etc.
}

/// Shutdown telemetry
pub fn shutdown() {
    info!("Shutting down YAWL telemetry");
    // In production, would flush and shutdown exporters
}

/// Record a workflow event
#[tracing::instrument]
pub fn record_workflow_event(
    workflow_id: &str,
    instance_id: &str,
    event_type: &str,
    metadata: Option<&std::collections::HashMap<String, String>>,
) {
    info!(
        workflow_id = %workflow_id,
        instance_id = %instance_id,
        event_type = %event_type,
        "Workflow event"
    );

    if let Some(meta) = metadata {
        debug!("Event metadata: {:?}", meta);
    }
}

/// Record a task event
#[tracing::instrument]
pub fn record_task_event(
    task_id: &str,
    event_type: &str,
    duration_ticks: Option<u64>,
) {
    info!(
        task_id = %task_id,
        event_type = %event_type,
        "Task event"
    );

    if let Some(ticks) = duration_ticks {
        // Covenant 5: Validate Chatman constant
        if ticks > 8 {
            warn!(
                task_id = %task_id,
                duration_ticks = %ticks,
                "Task execution exceeded Chatman constant (8 ticks)"
            );
        } else {
            debug!("Task completed in {} ticks", ticks);
        }
    }
}

/// Record a pattern execution
#[tracing::instrument]
pub fn record_pattern_execution(
    pattern_type: &str,
    duration_ticks: u64,
    success: bool,
) {
    if success {
        info!(
            pattern_type = %pattern_type,
            duration_ticks = %duration_ticks,
            "Pattern executed successfully"
        );

        // Covenant 5: Check Chatman constant
        if duration_ticks > 8 {
            warn!(
                pattern_type = %pattern_type,
                duration_ticks = %duration_ticks,
                "Pattern execution exceeded Chatman constant"
            );
        }
    } else {
        error!(
            pattern_type = %pattern_type,
            duration_ticks = %duration_ticks,
            "Pattern execution failed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_init() {
        init();
        shutdown();
    }

    #[test]
    fn test_workflow_event_recording() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        record_workflow_event("wf1", "inst1", "started", Some(&metadata));
    }

    #[test]
    fn test_task_event_recording() {
        record_task_event("task1", "started", None);
        record_task_event("task1", "completed", Some(5));
    }

    #[test]
    fn test_pattern_execution_recording() {
        record_pattern_execution("Sequence", 3, true);
        record_pattern_execution("ParallelSplit", 10, false);
    }
}
