//! TTL/Turtle serializer
//!
//! Serializes intermediate YAWL representation to TTL/Turtle RDF format.

use crate::error::{LegacyError, LegacyResult};
use crate::parser::{YawlCondition, YawlFlow, YawlTask, YawlWorkflow};

/// Turtle serializer for YAWL workflows
pub struct TurtleSerializer {
    yawl_namespace: String,
    rdfs_namespace: String,
}

impl TurtleSerializer {
    /// Create new Turtle serializer
    pub fn new() -> Self {
        Self {
            yawl_namespace: "http://bitflow.ai/ontology/yawl/v2#".to_string(),
            rdfs_namespace: "http://www.w3.org/2000/01/rdf-schema#".to_string(),
        }
    }

    /// Serialize workflow to TTL string
    pub fn serialize(&self, workflow: &YawlWorkflow) -> LegacyResult<String> {
        let mut ttl = String::new();

        // Add prefixes
        ttl.push_str(&format!(
            "@prefix yawl: <{}> .\n",
            self.yawl_namespace
        ));
        ttl.push_str(&format!(
            "@prefix rdfs: <{}> .\n",
            self.rdfs_namespace
        ));
        ttl.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        ttl.push_str("\n");

        // Serialize specification
        ttl.push_str(&format!("<{}> a yawl:Specification ;\n", workflow.uri));
        ttl.push_str(&format!(
            "    rdfs:label \"{}\" ;\n",
            self.escape_string(&workflow.name)
        ));

        // Add tasks
        if !workflow.tasks.is_empty() {
            for (i, task) in workflow.tasks.iter().enumerate() {
                let sep = if i < workflow.tasks.len() - 1 || !workflow.conditions.is_empty() {
                    " ;"
                } else {
                    " ."
                };
                ttl.push_str(&format!(
                    "    yawl:hasTask <{}#{}>{}\n",
                    workflow.uri, task.id, sep
                ));
            }
        }

        // Add conditions
        if !workflow.conditions.is_empty() {
            for (i, condition) in workflow.conditions.iter().enumerate() {
                let sep = if i < workflow.conditions.len() - 1 {
                    " ;"
                } else {
                    " ."
                };

                if condition.is_start {
                    ttl.push_str(&format!(
                        "    yawl:hasStartCondition <{}#{}>{}\n",
                        workflow.uri, condition.id, sep
                    ));
                } else if condition.is_end {
                    ttl.push_str(&format!(
                        "    yawl:hasEndCondition <{}#{}>{}\n",
                        workflow.uri, condition.id, sep
                    ));
                } else {
                    ttl.push_str(&format!(
                        "    yawl:hasCondition <{}#{}>{}\n",
                        workflow.uri, condition.id, sep
                    ));
                }
            }
        } else {
            ttl.push_str(" .\n");
        }

        ttl.push_str("\n");

        // Serialize tasks
        for task in &workflow.tasks {
            ttl.push_str(&self.serialize_task(&workflow.uri, task)?);
            ttl.push_str("\n");
        }

        // Serialize conditions
        for condition in &workflow.conditions {
            ttl.push_str(&self.serialize_condition(&workflow.uri, condition)?);
            ttl.push_str("\n");
        }

        // Serialize flows
        for flow in &workflow.flows {
            ttl.push_str(&self.serialize_flow(&workflow.uri, flow)?);
            ttl.push_str("\n");
        }

        Ok(ttl)
    }

    fn serialize_task(&self, base_uri: &str, task: &YawlTask) -> LegacyResult<String> {
        let mut ttl = String::new();

        ttl.push_str(&format!("<{}#{}> a yawl:Task ;\n", base_uri, task.id));
        ttl.push_str(&format!(
            "    rdfs:label \"{}\" ",
            self.escape_string(&task.name)
        ));

        // Add join type
        if let Some(ref join) = task.join_type {
            ttl.push_str(";\n");
            ttl.push_str(&format!("    yawl:joinType yawl:{} ", join.to_uppercase()));
        }

        // Add split type
        if let Some(ref split) = task.split_type {
            ttl.push_str(";\n");
            ttl.push_str(&format!(
                "    yawl:splitType yawl:{} ",
                split.to_uppercase()
            ));
        }

        // Add custom attributes
        for (key, value) in &task.attributes {
            ttl.push_str(";\n");
            ttl.push_str(&format!(
                "    yawl:{} \"{}\" ",
                self.sanitize_predicate(key),
                self.escape_string(value)
            ));
        }

        ttl.push_str(".\n");

        Ok(ttl)
    }

    fn serialize_condition(&self, base_uri: &str, condition: &YawlCondition) -> LegacyResult<String> {
        let mut ttl = String::new();

        ttl.push_str(&format!(
            "<{}#{}> a yawl:Condition ",
            base_uri, condition.id
        ));

        if let Some(ref name) = condition.name {
            ttl.push_str(";\n");
            ttl.push_str(&format!(
                "    rdfs:label \"{}\" ",
                self.escape_string(name)
            ));
        }

        if condition.is_start {
            ttl.push_str(";\n");
            ttl.push_str("    yawl:isStartCondition true ");
        }

        if condition.is_end {
            ttl.push_str(";\n");
            ttl.push_str("    yawl:isEndCondition true ");
        }

        ttl.push_str(".\n");

        Ok(ttl)
    }

    fn serialize_flow(&self, base_uri: &str, flow: &YawlFlow) -> LegacyResult<String> {
        let mut ttl = String::new();

        ttl.push_str(&format!("<{}#{}> a yawl:Flow ;\n", base_uri, flow.id));
        ttl.push_str(&format!(
            "    yawl:source <{}#{}> ;\n",
            base_uri, flow.source
        ));
        ttl.push_str(&format!(
            "    yawl:target <{}#{}> ",
            base_uri, flow.target
        ));

        if let Some(ref predicate) = flow.predicate {
            ttl.push_str(";\n");
            ttl.push_str(&format!(
                "    yawl:predicate \"{}\" ",
                self.escape_string(predicate)
            ));
        }

        ttl.push_str(".\n");

        Ok(ttl)
    }

    fn escape_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }

    fn sanitize_predicate(&self, s: &str) -> String {
        s.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect()
    }
}

impl Default for TurtleSerializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{YawlTask, YawlWorkflow};
    use std::collections::HashMap;

    #[test]
    fn test_serialize_basic_workflow() {
        let workflow = YawlWorkflow {
            uri: "http://example.org/test".to_string(),
            name: "Test Workflow".to_string(),
            tasks: vec![YawlTask {
                id: "task1".to_string(),
                name: "Task One".to_string(),
                join_type: None,
                split_type: Some("AND".to_string()),
                attributes: HashMap::new(),
            }],
            conditions: vec![],
            flows: vec![],
        };

        let serializer = TurtleSerializer::new();
        let result = serializer.serialize(&workflow);

        assert!(result.is_ok(), "Serialization should succeed");
        let ttl = result.unwrap();

        assert!(ttl.contains("yawl:Specification"));
        assert!(ttl.contains("Test Workflow"));
        assert!(ttl.contains("yawl:Task"));
        assert!(ttl.contains("yawl:splitType yawl:AND"));
    }

    #[test]
    fn test_escape_string() {
        let serializer = TurtleSerializer::new();

        assert_eq!(
            serializer.escape_string("test\"quote"),
            "test\\\"quote"
        );
        assert_eq!(
            serializer.escape_string("test\nline"),
            "test\\nline"
        );
    }
}
