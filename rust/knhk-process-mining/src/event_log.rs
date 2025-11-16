//! Event log extraction from OpenTelemetry spans
//!
//! This module converts OTEL spans into process mining event logs,
//! enabling workflow analysis and process discovery.

use crate::{ProcessMiningError, Result};
use chrono::{DateTime, Utc};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A single event in a process execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    /// Unique case/workflow instance ID
    pub case_id: String,

    /// Activity/step name
    pub activity: String,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Resource/agent that performed the activity
    pub resource: Option<String>,

    /// Event attributes (from span attributes)
    pub attributes: HashMap<String, String>,

    /// Event lifecycle (start, complete, etc.)
    pub lifecycle: EventLifecycle,
}

/// Event lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventLifecycle {
    Start,
    Complete,
    Suspend,
    Resume,
    Abort,
}

/// Complete event log for a process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    /// All events, sorted by timestamp
    pub events: Vec<ProcessEvent>,

    /// Unique case IDs
    pub case_ids: Vec<String>,

    /// Unique activities
    pub activities: Vec<String>,

    /// Metadata
    pub metadata: EventLogMetadata,
}

/// Event log metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogMetadata {
    /// Total number of cases
    pub total_cases: usize,

    /// Total number of events
    pub total_events: usize,

    /// Time range
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,

    /// Average events per case
    pub avg_events_per_case: f64,
}

impl EventLog {
    /// Get all events for a specific case
    pub fn events_for_case(&self, case_id: &str) -> Vec<&ProcessEvent> {
        self.events
            .iter()
            .filter(|e| e.case_id == case_id)
            .collect()
    }

    /// Get all events for a specific activity
    pub fn events_for_activity(&self, activity: &str) -> Vec<&ProcessEvent> {
        self.events
            .iter()
            .filter(|e| e.activity == activity)
            .collect()
    }

    /// Calculate time between two activities in a case
    pub fn time_between(
        &self,
        case_id: &str,
        from_activity: &str,
        to_activity: &str,
    ) -> Option<Duration> {
        let case_events: Vec<_> = self.events_for_case(case_id);

        let from_event = case_events
            .iter()
            .find(|e| e.activity == from_activity && e.lifecycle == EventLifecycle::Complete)?;

        let to_event = case_events
            .iter()
            .find(|e| e.activity == to_activity && e.lifecycle == EventLifecycle::Start)?;

        let duration = to_event
            .timestamp
            .signed_duration_since(from_event.timestamp);
        Some(duration.to_std().ok()?)
    }

    /// Get case duration (start to end)
    pub fn case_duration(&self, case_id: &str) -> Option<Duration> {
        let case_events: Vec<_> = self.events_for_case(case_id);

        if case_events.is_empty() {
            return None;
        }

        let start = case_events.first()?.timestamp;
        let end = case_events.last()?.timestamp;

        let duration = end.signed_duration_since(start);
        Some(duration.to_std().ok()?)
    }

    /// Export to XES format (XML-based standard for process mining)
    pub fn to_xes(&self) -> Result<String> {
        // Simplified XES generation
        let mut xes = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xes.push_str("<log>\n");

        for case_id in &self.case_ids {
            xes.push_str("  <trace>\n");
            xes.push_str(&format!(
                "    <string key=\"concept:name\" value=\"{}\"/>\n",
                case_id
            ));

            for event in self.events_for_case(case_id) {
                xes.push_str("    <event>\n");
                xes.push_str(&format!(
                    "      <string key=\"concept:name\" value=\"{}\"/>\n",
                    event.activity
                ));
                xes.push_str(&format!(
                    "      <date key=\"time:timestamp\" value=\"{}\"/>\n",
                    event.timestamp.to_rfc3339()
                ));
                if let Some(resource) = &event.resource {
                    xes.push_str(&format!(
                        "      <string key=\"org:resource\" value=\"{}\"/>\n",
                        resource
                    ));
                }
                xes.push_str("    </event>\n");
            }

            xes.push_str("  </trace>\n");
        }

        xes.push_str("</log>\n");
        Ok(xes)
    }
}

/// Builder for creating event logs from OTEL spans
#[derive(Debug, Default)]
pub struct EventLogBuilder {
    events: Vec<ProcessEvent>,
}

impl EventLogBuilder {
    /// Create new event log builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a process event
    pub fn add_event(&mut self, event: ProcessEvent) -> &mut Self {
        self.events.push(event);
        self
    }

    /// Add event from OTEL span-like data
    pub fn add_span_event(
        &mut self,
        case_id: String,
        activity: String,
        timestamp: DateTime<Utc>,
        resource: Option<String>,
        attributes: HashMap<String, String>,
    ) -> &mut Self {
        self.events.push(ProcessEvent {
            case_id,
            activity,
            timestamp,
            resource,
            attributes,
            lifecycle: EventLifecycle::Complete,
        });
        self
    }

    /// Build the event log
    pub fn build(mut self) -> Result<EventLog> {
        if self.events.is_empty() {
            return Err(ProcessMiningError::EventLog(
                "No events in event log".to_string(),
            ));
        }

        // Sort by timestamp
        self.events.sort_by_key(|e| e.timestamp);

        // Extract unique case IDs and activities
        let mut case_ids: Vec<String> = self
            .events
            .iter()
            .map(|e| e.case_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        case_ids.sort();

        let mut activities: Vec<String> = self
            .events
            .iter()
            .map(|e| e.activity.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        activities.sort();

        // Calculate metadata
        let total_cases = case_ids.len();
        let total_events = self.events.len();
        let start_time = self.events.first().unwrap().timestamp;
        let end_time = self.events.last().unwrap().timestamp;
        let avg_events_per_case = total_events as f64 / total_cases as f64;

        let metadata = EventLogMetadata {
            total_cases,
            total_events,
            start_time,
            end_time,
            avg_events_per_case,
        };

        Ok(EventLog {
            events: self.events,
            case_ids,
            activities,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_log_builder() {
        let mut builder = EventLogBuilder::new();

        let now = Utc::now();

        builder
            .add_span_event(
                "case_001".to_string(),
                "step_1".to_string(),
                now,
                Some("agent_1".to_string()),
                HashMap::new(),
            )
            .add_span_event(
                "case_001".to_string(),
                "step_2".to_string(),
                now + chrono::Duration::seconds(5),
                Some("agent_2".to_string()),
                HashMap::new(),
            );

        let log = builder.build().unwrap();

        assert_eq!(log.metadata.total_cases, 1);
        assert_eq!(log.metadata.total_events, 2);
        assert_eq!(log.case_ids.len(), 1);
        assert_eq!(log.activities.len(), 2);
    }

    #[test]
    fn test_event_log_queries() {
        let mut builder = EventLogBuilder::new();
        let now = Utc::now();

        builder
            .add_span_event(
                "case_001".to_string(),
                "step_1".to_string(),
                now,
                None,
                HashMap::new(),
            )
            .add_span_event(
                "case_002".to_string(),
                "step_1".to_string(),
                now + chrono::Duration::seconds(1),
                None,
                HashMap::new(),
            );

        let log = builder.build().unwrap();

        let case_001_events = log.events_for_case("case_001");
        assert_eq!(case_001_events.len(), 1);

        let step_1_events = log.events_for_activity("step_1");
        assert_eq!(step_1_events.len(), 2);
    }
}
