//! Workflow definition parser for WASM

use crate::{WasmError, WasmResult};
use crate::runtime::{WorkflowSpec, Task};
use serde_json::Value as JsonValue;

/// Workflow definition parser
pub struct WorkflowParser;

impl WorkflowParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse workflow from Turtle/RDF format
    #[cfg(feature = "rdf")]
    pub fn parse_turtle(&self, _turtle: &str) -> WasmResult<WorkflowSpec> {
        // For now, return a simple error
        // Full RDF parsing would require significant size overhead
        Err(WasmError::ParseError(
            "RDF parsing not yet implemented in WASM. Use JSON format.".to_string()
        ))
    }

    /// Parse workflow from Turtle/RDF format (fallback when RDF feature disabled)
    #[cfg(not(feature = "rdf"))]
    pub fn parse_turtle(&self, _turtle: &str) -> WasmResult<WorkflowSpec> {
        Err(WasmError::ParseError(
            "RDF parsing disabled. Use JSON format or enable 'rdf' feature.".to_string()
        ))
    }

    /// Parse workflow from JSON format
    pub fn parse_json(&self, json: &str) -> WasmResult<WorkflowSpec> {
        let value: JsonValue = serde_json::from_str(json)
            .map_err(|e| WasmError::ParseError(e.to_string()))?;

        let obj = value.as_object()
            .ok_or_else(|| WasmError::InvalidSpec("Expected JSON object".to_string()))?;

        let id = obj.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WasmError::InvalidSpec("Missing 'id' field".to_string()))?
            .to_string();

        let pattern = obj.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WasmError::InvalidSpec("Missing 'pattern' field".to_string()))?
            .to_string();

        let tasks = obj.get("tasks")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WasmError::InvalidSpec("Missing 'tasks' field".to_string()))?
            .iter()
            .map(|t| self.parse_task(t))
            .collect::<WasmResult<Vec<_>>>()?;

        let loop_condition = obj.get("loopCondition").cloned();

        Ok(WorkflowSpec {
            id,
            pattern,
            tasks,
            loop_condition,
        })
    }

    fn parse_task(&self, value: &JsonValue) -> WasmResult<Task> {
        let obj = value.as_object()
            .ok_or_else(|| WasmError::InvalidSpec("Task must be an object".to_string()))?;

        let id = obj.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WasmError::InvalidSpec("Task missing 'id' field".to_string()))?
            .to_string();

        let task_type = obj.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("compute")
            .to_string();

        let config = obj.get("config").cloned();
        let condition = obj.get("condition").cloned();

        Ok(Task {
            id,
            task_type,
            config,
            condition,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_workflow() {
        let json = r#"{
            "id": "test-workflow",
            "pattern": "Sequence",
            "tasks": [
                {"id": "task1", "type": "compute"},
                {"id": "task2", "type": "validate"}
            ]
        }"#;

        let parser = WorkflowParser::new();
        let spec = parser.parse_json(json).unwrap();

        assert_eq!(spec.id, "test-workflow");
        assert_eq!(spec.pattern, "Sequence");
        assert_eq!(spec.tasks.len(), 2);
    }
}
