//! Event loop handlers for timer and external events

use crate::case::CaseId;
use crate::parser::WorkflowSpecId;
use crate::patterns::{PatternExecutionContext, PatternId, PatternRegistry};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::services::TimerFired;

/// Start timer event loop
pub(crate) fn start_timer_loop(
    registry: Arc<PatternRegistry>,
    mut timer_rx: mpsc::Receiver<TimerFired>,
) {
    tokio::spawn(async move {
        while let Some(tf) = timer_rx.recv().await {
            // Parse IDs from strings
            let case_id = crate::case::CaseId::parse_str(&tf.case_id)
                .unwrap_or_else(|_| crate::case::CaseId::new());
            let workflow_id = crate::parser::WorkflowSpecId::parse_str(&tf.workflow_id)
                .unwrap_or_else(|_| crate::parser::WorkflowSpecId::new());

            let ctx = PatternExecutionContext {
                case_id,
                workflow_id,
                variables: {
                    let mut vars = HashMap::new();
                    vars.insert("key".to_string(), tf.key.clone());
                    vars.insert("fired_at".to_string(), tf.fired_at.to_string());
                    vars
                },
                arrived_from: std::collections::HashSet::new(),
                scope_id: String::new(),
            };
            // Execute pattern 30 or 31 based on timer type
            let _ = registry.execute(&PatternId(tf.pattern_id as u32), &ctx);
        }
    });
}

/// Start external event loop (Pattern 16: Deferred Choice)
pub(crate) fn start_event_loop(
    registry: Arc<PatternRegistry>,
    mut event_rx: mpsc::Receiver<serde_json::Value>,
) {
    tokio::spawn(async move {
        while let Some(evt) = event_rx.recv().await {
            let case_id = evt
                .get("case_id")
                .and_then(|v| v.as_str())
                .map(|s| CaseId::parse_str(s).unwrap_or_else(|_| CaseId::new()))
                .unwrap_or_else(|| CaseId::new());
            let workflow_id = evt
                .get("workflow_id")
                .and_then(|v| v.as_str())
                .map(|s| WorkflowSpecId::parse_str(s).unwrap_or_else(|_| WorkflowSpecId::new()))
                .unwrap_or_else(|| WorkflowSpecId::new());

            let ctx = PatternExecutionContext {
                case_id,
                workflow_id,
                variables: {
                    let mut vars = HashMap::new();
                    if let Some(event_type) = evt.get("event_type").and_then(|v| v.as_str()) {
                        vars.insert("event_type".to_string(), event_type.to_string());
                    }
                    if let Some(data) = evt.get("data") {
                        if let Some(obj) = data.as_object() {
                            for (k, v) in obj {
                                if let Some(s) = v.as_str() {
                                    vars.insert(k.clone(), s.to_string());
                                }
                            }
                        }
                    }
                    vars
                },
                arrived_from: std::collections::HashSet::new(),
                scope_id: String::new(),
            };
            let _ = registry.execute(&PatternId(16), &ctx); // Pattern 16: Deferred Choice
        }
    });
}
