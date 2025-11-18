//! XML YAWL parser
//!
//! Parses legacy XML YAWL workflow definitions into intermediate representation.

use crate::error::{LegacyError, LegacyResult};
use roxmltree::{Document, Node};
use std::collections::HashMap;

/// Intermediate representation of a YAWL workflow
#[derive(Debug, Clone)]
pub struct YawlWorkflow {
    /// Workflow URI/identifier
    pub uri: String,
    /// Workflow name
    pub name: String,
    /// Tasks in the workflow
    pub tasks: Vec<YawlTask>,
    /// Conditions (places) in the workflow
    pub conditions: Vec<YawlCondition>,
    /// Flows (arcs) connecting elements
    pub flows: Vec<YawlFlow>,
}

/// YAWL task representation
#[derive(Debug, Clone)]
pub struct YawlTask {
    /// Task identifier
    pub id: String,
    /// Task name/label
    pub name: String,
    /// Join type (AND, OR, XOR)
    pub join_type: Option<String>,
    /// Split type (AND, OR, XOR)
    pub split_type: Option<String>,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

/// YAWL condition (place)
#[derive(Debug, Clone)]
pub struct YawlCondition {
    /// Condition identifier
    pub id: String,
    /// Condition name/label
    pub name: Option<String>,
    /// Whether this is the start condition
    pub is_start: bool,
    /// Whether this is the end condition
    pub is_end: bool,
}

/// YAWL flow (arc)
#[derive(Debug, Clone)]
pub struct YawlFlow {
    /// Flow identifier
    pub id: String,
    /// Source element ID
    pub source: String,
    /// Target element ID
    pub target: String,
    /// Flow predicate/condition
    pub predicate: Option<String>,
}

/// XML parser for YAWL workflows
pub struct XmlParser;

impl XmlParser {
    /// Create new XML parser
    pub fn new() -> Self {
        Self
    }

    /// Parse XML YAWL string into intermediate representation
    pub fn parse(&self, xml: &str) -> LegacyResult<YawlWorkflow> {
        // Parse XML document
        let doc = Document::parse(xml)
            .map_err(|e| LegacyError::XmlParse(format!("Failed to parse XML: {}", e)))?;

        // Find specification element
        let spec = doc
            .root_element()
            .descendants()
            .find(|n| n.tag_name().name() == "specification")
            .ok_or_else(|| LegacyError::MissingElement("specification".to_string()))?;

        // Extract URI
        let uri = spec
            .attribute("uri")
            .ok_or_else(|| LegacyError::MissingElement("uri attribute".to_string()))?
            .to_string();

        // Extract name
        let name = spec
            .descendants()
            .find(|n| n.tag_name().name() == "name")
            .and_then(|n| n.text())
            .unwrap_or("Unnamed Workflow")
            .to_string();

        // Extract tasks
        let tasks = self.extract_tasks(&spec)?;

        // Extract conditions
        let conditions = self.extract_conditions(&spec)?;

        // Extract flows
        let flows = self.extract_flows(&spec)?;

        Ok(YawlWorkflow {
            uri,
            name,
            tasks,
            conditions,
            flows,
        })
    }

    fn extract_tasks(&self, spec: &Node) -> LegacyResult<Vec<YawlTask>> {
        let mut tasks = Vec::new();

        for task_node in spec.descendants().filter(|n| n.tag_name().name() == "task") {
            let id = task_node
                .attribute("id")
                .ok_or_else(|| LegacyError::MissingElement("task id".to_string()))?
                .to_string();

            let name = task_node
                .descendants()
                .find(|n| n.tag_name().name() == "name")
                .and_then(|n| n.text())
                .unwrap_or(&id)
                .to_string();

            // Extract join type
            let join_type = task_node
                .descendants()
                .find(|n| n.tag_name().name() == "join")
                .and_then(|n| n.attribute("code"))
                .map(|s| s.to_string());

            // Extract split type
            let split_type = task_node
                .descendants()
                .find(|n| n.tag_name().name() == "split")
                .and_then(|n| n.attribute("code"))
                .map(|s| s.to_string());

            // Extract custom attributes
            let mut attributes = HashMap::new();
            for attr in task_node.attributes() {
                if attr.name() != "id" {
                    attributes.insert(attr.name().to_string(), attr.value().to_string());
                }
            }

            tasks.push(YawlTask {
                id,
                name,
                join_type,
                split_type,
                attributes,
            });
        }

        Ok(tasks)
    }

    fn extract_conditions(&self, spec: &Node) -> LegacyResult<Vec<YawlCondition>> {
        let mut conditions = Vec::new();

        for cond_node in spec
            .descendants()
            .filter(|n| n.tag_name().name() == "condition")
        {
            let id = cond_node
                .attribute("id")
                .ok_or_else(|| LegacyError::MissingElement("condition id".to_string()))?
                .to_string();

            let name = cond_node
                .descendants()
                .find(|n| n.tag_name().name() == "name")
                .and_then(|n| n.text())
                .map(|s| s.to_string());

            // Check if start/end condition
            let is_start = cond_node.attribute("isStartCondition") == Some("true");
            let is_end = cond_node.attribute("isEndCondition") == Some("true");

            conditions.push(YawlCondition {
                id,
                name,
                is_start,
                is_end,
            });
        }

        Ok(conditions)
    }

    fn extract_flows(&self, spec: &Node) -> LegacyResult<Vec<YawlFlow>> {
        let mut flows = Vec::new();

        for flow_node in spec.descendants().filter(|n| n.tag_name().name() == "flow") {
            let id = flow_node
                .attribute("id")
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("flow_{}", flows.len()));

            let source = flow_node
                .attribute("source")
                .ok_or_else(|| LegacyError::MissingElement("flow source".to_string()))?
                .to_string();

            let target = flow_node
                .attribute("target")
                .ok_or_else(|| LegacyError::MissingElement("flow target".to_string()))?
                .to_string();

            let predicate = flow_node
                .descendants()
                .find(|n| n.tag_name().name() == "predicate")
                .and_then(|n| n.text())
                .map(|s| s.to_string());

            flows.push(YawlFlow {
                id,
                source,
                target,
                predicate,
            });
        }

        Ok(flows)
    }
}

impl Default for XmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_workflow() {
        let xml = r#"<?xml version="1.0"?>
<specification uri="http://example.org/test">
  <name>Test Workflow</name>
  <task id="task1">
    <name>Task One</name>
    <split code="AND"/>
  </task>
  <condition id="c1" isStartCondition="true">
    <name>Start</name>
  </condition>
</specification>"#;

        let parser = XmlParser::new();
        let result = parser.parse(xml);

        assert!(result.is_ok(), "Parse should succeed");
        let workflow = result.unwrap();

        assert_eq!(workflow.uri, "http://example.org/test");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.tasks.len(), 1);
        assert_eq!(workflow.tasks[0].id, "task1");
        assert_eq!(workflow.tasks[0].split_type, Some("AND".to_string()));
        assert_eq!(workflow.conditions.len(), 1);
        assert!(workflow.conditions[0].is_start);
    }
}
