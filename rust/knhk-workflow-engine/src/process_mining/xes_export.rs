//! XES (eXtensible Event Stream) export for ProM process mining
//!
//! Implements IEEE XES Standard: http://www.xes-standard.org/
//!
//! **Core XES Attributes (MUST HAVE):**
//! - concept:name - Activity/trace name
//! - time:timestamp - Event timestamp
//! - lifecycle:transition - Event lifecycle (start/complete/cancel)
//!
//! **KNHK Extensions (NICE TO HAVE):**
//! - pattern:id - YAWL pattern identifier
//! - org:resource - Resource assignment

use crate::case::CaseId;
use crate::state::StateEvent;
use chrono::{DateTime, Utc};

/// XES exporter for ProM process mining
pub struct XesExporter;

impl XesExporter {
    /// Export single case execution log to XES format
    ///
    /// Generates IEEE XES 2.0 compliant XML with:
    /// - Single trace (case)
    /// - Multiple events (state changes)
    /// - Standard extensions (Concept, Time, Lifecycle, Organizational)
    pub fn export_case_log(case_id: &CaseId, events: Vec<WorkflowEvent>) -> String {
        let mut xml = String::new();

        // XES 2.0 header with standard extensions
        xml.push_str(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<log xes.version="2.0" xes.features="nested-attributes">
  <extension name="Concept" prefix="concept" uri="http://www.xes-standard.org/concept.xesext"/>
  <extension name="Time" prefix="time" uri="http://www.xes-standard.org/time.xesext"/>
  <extension name="Lifecycle" prefix="lifecycle" uri="http://www.xes-standard.org/lifecycle.xesext"/>
  <extension name="Organizational" prefix="org" uri="http://www.xes-standard.org/org.xesext"/>

  <global scope="trace">
    <string key="concept:name" value="__INVALID__"/>
  </global>

  <global scope="event">
    <string key="concept:name" value="__INVALID__"/>
    <string key="lifecycle:transition" value="complete"/>
    <date key="time:timestamp" value="1970-01-01T00:00:00.000Z"/>
  </global>

  <classifier name="Activity" keys="concept:name"/>
  <classifier name="activity classifier" keys="concept:name lifecycle:transition"/>
"#,
        );

        // Trace (case) header
        xml.push_str(&format!(
            r#"  <trace>
    <string key="concept:name" value="{}"/>
"#,
            Self::escape_xml(&case_id.to_string())
        ));

        // Events
        for event in events {
            xml.push_str("    <event>\n");
            xml.push_str(&format!(
                r#"      <string key="concept:name" value="{}"/>
"#,
                Self::escape_xml(&event.activity_name)
            ));
            xml.push_str(&format!(
                r#"      <string key="lifecycle:transition" value="{}"/>
"#,
                event.lifecycle
            ));
            xml.push_str(&format!(
                r#"      <date key="time:timestamp" value="{}"/>
"#,
                event.timestamp.to_rfc3339()
            ));

            // Optional: Resource assignment
            if let Some(ref resource) = event.resource {
                xml.push_str(&format!(
                    r#"      <string key="org:resource" value="{}"/>
"#,
                    Self::escape_xml(resource)
                ));
            }

            // Optional: Pattern ID (KNHK-specific extension)
            if let Some(pattern_id) = event.pattern_id {
                xml.push_str(&format!(
                    r#"      <int key="pattern:id" value="{}"/>
"#,
                    pattern_id
                ));
            }

            xml.push_str("    </event>\n");
        }

        xml.push_str("  </trace>\n</log>");
        xml
    }

    /// Export multiple cases to XES format
    ///
    /// Generates XES log with multiple traces (cases)
    pub fn export_multiple_cases(cases: Vec<(CaseId, Vec<WorkflowEvent>)>) -> String {
        let mut xml = String::new();

        // XES 2.0 header
        xml.push_str(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<log xes.version="2.0" xes.features="nested-attributes">
  <extension name="Concept" prefix="concept" uri="http://www.xes-standard.org/concept.xesext"/>
  <extension name="Time" prefix="time" uri="http://www.xes-standard.org/time.xesext"/>
  <extension name="Lifecycle" prefix="lifecycle" uri="http://www.xes-standard.org/lifecycle.xesext"/>
  <extension name="Organizational" prefix="org" uri="http://www.xes-standard.org/org.xesext"/>

  <global scope="trace">
    <string key="concept:name" value="__INVALID__"/>
  </global>

  <global scope="event">
    <string key="concept:name" value="__INVALID__"/>
    <string key="lifecycle:transition" value="complete"/>
    <date key="time:timestamp" value="1970-01-01T00:00:00.000Z"/>
  </global>

  <classifier name="Activity" keys="concept:name"/>
  <classifier name="activity classifier" keys="concept:name lifecycle:transition"/>
"#,
        );

        // Export each case as a trace
        for (case_id, events) in cases {
            xml.push_str(&format!(
                r#"  <trace>
    <string key="concept:name" value="{}"/>
"#,
                Self::escape_xml(&case_id.to_string())
            ));

            for event in events {
                xml.push_str("    <event>\n");
                xml.push_str(&format!(
                    r#"      <string key="concept:name" value="{}"/>
"#,
                    Self::escape_xml(&event.activity_name)
                ));
                xml.push_str(&format!(
                    r#"      <string key="lifecycle:transition" value="{}"/>
"#,
                    event.lifecycle
                ));
                xml.push_str(&format!(
                    r#"      <date key="time:timestamp" value="{}"/>
"#,
                    event.timestamp.to_rfc3339()
                ));

                if let Some(ref resource) = event.resource {
                    xml.push_str(&format!(
                        r#"      <string key="org:resource" value="{}"/>
"#,
                        Self::escape_xml(resource)
                    ));
                }

                if let Some(pattern_id) = event.pattern_id {
                    xml.push_str(&format!(
                        r#"      <int key="pattern:id" value="{}"/>
"#,
                        pattern_id
                    ));
                }

                xml.push_str("    </event>\n");
            }

            xml.push_str("  </trace>\n");
        }

        xml.push_str("</log>");
        xml
    }

    /// Convert StateEvent to WorkflowEvent for XES export
    pub fn state_event_to_workflow_event(event: StateEvent) -> Option<WorkflowEvent> {
        match event {
            StateEvent::CaseCreated {
                case_id,
                spec_id,
                timestamp,
            } => Some(WorkflowEvent {
                activity_name: format!("case_created_{}", spec_id),
                lifecycle: "start".to_string(),
                timestamp,
                resource: Some("System".to_string()),
                pattern_id: None,
            }),
            StateEvent::CaseStateChanged {
                old_state,
                new_state,
                timestamp,
                case_id: _,
            } => Some(WorkflowEvent {
                activity_name: format!("state_transition_{}_{}", old_state, new_state),
                lifecycle: if new_state.contains("completed") || new_state.contains("finished") {
                    "complete".to_string()
                } else if new_state.contains("cancelled") || new_state.contains("failed") {
                    "cancel".to_string()
                } else {
                    "start".to_string()
                },
                timestamp,
                resource: Some("System".to_string()),
                pattern_id: None,
            }),
            StateEvent::SpecRegistered { .. } => None, // Skip spec registration events
        }
    }

    /// Escape XML special characters
    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

/// Workflow event for XES export
///
/// Represents a single event in workflow execution with XES-compatible attributes
#[derive(Debug, Clone)]
pub struct WorkflowEvent {
    /// Activity name (concept:name in XES)
    pub activity_name: String,
    /// Lifecycle transition: "start", "complete", or "cancel"
    pub lifecycle: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Resource (actor) that executed the event
    pub resource: Option<String>,
    /// KNHK pattern ID (custom extension)
    pub pattern_id: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xes_export_basic_structure() {
        let case_id = CaseId::new();
        let events = vec![
            WorkflowEvent {
                activity_name: "task_a".to_string(),
                lifecycle: "start".to_string(),
                timestamp: Utc::now(),
                resource: Some("user1".to_string()),
                pattern_id: Some(1),
            },
            WorkflowEvent {
                activity_name: "task_a".to_string(),
                lifecycle: "complete".to_string(),
                timestamp: Utc::now(),
                resource: Some("user1".to_string()),
                pattern_id: Some(1),
            },
        ];

        let xes = XesExporter::export_case_log(&case_id, events);

        // Validate XES structure
        assert!(xes.contains("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>"));
        assert!(xes.contains("<log xes.version=\"2.0\""));
        assert!(xes.contains("<trace>"));
        assert!(xes.contains("<event>"));
        assert!(xes.contains("concept:name"));
        assert!(xes.contains("lifecycle:transition"));
        assert!(xes.contains("time:timestamp"));
        assert!(xes.contains("org:resource"));
        assert!(xes.contains("pattern:id"));
    }

    #[test]
    fn test_xml_escaping() {
        let case_id = CaseId::new();
        let events = vec![WorkflowEvent {
            activity_name: "task_with_<special>&\"characters\"".to_string(),
            lifecycle: "complete".to_string(),
            timestamp: Utc::now(),
            resource: Some("user<1>".to_string()),
            pattern_id: None,
        }];

        let xes = XesExporter::export_case_log(&case_id, events);

        // Verify XML escaping
        assert!(xes.contains("&lt;special&gt;"));
        assert!(xes.contains("&amp;"));
        assert!(xes.contains("&quot;"));
    }

    #[test]
    fn test_multiple_cases_export() {
        let case1 = CaseId::new();
        let case2 = CaseId::new();

        let events1 = vec![WorkflowEvent {
            activity_name: "task_a".to_string(),
            lifecycle: "complete".to_string(),
            timestamp: Utc::now(),
            resource: None,
            pattern_id: None,
        }];

        let events2 = vec![WorkflowEvent {
            activity_name: "task_b".to_string(),
            lifecycle: "complete".to_string(),
            timestamp: Utc::now(),
            resource: None,
            pattern_id: None,
        }];

        let xes = XesExporter::export_multiple_cases(vec![(case1, events1), (case2, events2)]);

        // Verify multiple traces
        assert_eq!(xes.matches("<trace>").count(), 2);
        assert!(xes.contains("task_a"));
        assert!(xes.contains("task_b"));
    }
}
