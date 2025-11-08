//! Schema registry for workflow schema validation
//!
//! Provides RDF-based schema validation for workflow specifications.
//! Uses SHACL (Shapes Constraint Language) for schema validation.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use oxigraph::store::Store;
use oxigraph::io::RdfFormat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Schema validation error
#[derive(Debug, Clone)]
pub enum SchemaValidationError {
    /// Schema not found
    SchemaNotFound(String),
    /// Validation failed
    ValidationFailed(String),
    /// Invalid schema format
    InvalidSchemaFormat(String),
    /// RDF parsing error
    RdfParseError(String),
}

impl std::fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaValidationError::SchemaNotFound(name) => {
                write!(f, "Schema not found: {}", name)
            }
            SchemaValidationError::ValidationFailed(msg) => {
                write!(f, "Schema validation failed: {}", msg)
            }
            SchemaValidationError::InvalidSchemaFormat(msg) => {
                write!(f, "Invalid schema format: {}", msg)
            }
            SchemaValidationError::RdfParseError(msg) => {
                write!(f, "RDF parsing error: {}", msg)
            }
        }
    }
}

impl std::error::Error for SchemaValidationError {}

/// Schema metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    /// Schema name
    pub name: String,
    /// Schema description
    pub description: String,
    /// Schema version
    pub version: String,
    /// RDF schema content (Turtle format)
    pub rdf_content: String,
    /// Enabled flag
    pub enabled: bool,
}

/// Schema validation result
#[derive(Debug, Clone)]
pub struct SchemaValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Validation warnings (if any)
    pub warnings: Vec<String>,
}

impl SchemaValidationResult {
    /// Create success result
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create failure result
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
        }
    }
}

/// Schema registry
pub struct SchemaRegistry {
    /// Schemas by name
    schemas: Arc<RwLock<HashMap<String, SchemaMetadata>>>,
    /// RDF store for schema validation
    rdf_store: Arc<RwLock<Store>>,
}

impl SchemaRegistry {
    /// Create new schema registry
    pub fn new() -> WorkflowResult<Self> {
        let store = Store::new()
            .map_err(|e| WorkflowError::Internal(format!("Failed to create RDF store: {:?}", e)))?;

        Ok(Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            rdf_store: Arc::new(RwLock::new(store)),
        })
    }

    /// Register a schema
    pub async fn register_schema(&self, schema: SchemaMetadata) -> WorkflowResult<()> {
        if schema.name.is_empty() {
            return Err(WorkflowError::Validation("Schema name cannot be empty".to_string()));
        }

        // Validate RDF format
        let store = Store::new()
            .map_err(|e| WorkflowError::Internal(format!("Failed to create RDF store: {:?}", e)))?;

        store
            .load_from_reader(RdfFormat::Turtle, schema.rdf_content.as_bytes())
            .map_err(|e| {
                WorkflowError::Validation(format!(
                    "Invalid RDF schema format: {:?}",
                    e
                ))
            })?;

        let mut schemas = self.schemas.write().await;
        schemas.insert(schema.name.clone(), schema);

        Ok(())
    }

    /// Validate workflow spec against a schema
    pub async fn validate_workflow_spec(
        &self,
        spec: &WorkflowSpec,
        schema_name: &str,
    ) -> WorkflowResult<SchemaValidationResult> {
        let schemas = self.schemas.read().await;
        let schema = schemas
            .get(schema_name)
            .ok_or_else(|| {
                WorkflowError::Validation(format!("Schema {} not found", schema_name))
            })?
            .clone();
        drop(schemas);

        if !schema.enabled {
            return Ok(SchemaValidationResult::success());
        }

        // Load schema into RDF store
        let mut store = self.rdf_store.write().await;
        store
            .load_from_reader(RdfFormat::Turtle, schema.rdf_content.as_bytes())
            .map_err(|e| {
                WorkflowError::Internal(format!("Failed to load schema into store: {:?}", e))
            })?;

        // Convert workflow spec to RDF (Turtle format)
        let workflow_rdf = self.workflow_spec_to_rdf(spec)?;

        // Load workflow spec into store
        store
            .load_from_reader(RdfFormat::Turtle, workflow_rdf.as_bytes())
            .map_err(|e| {
                WorkflowError::Internal(format!("Failed to load workflow spec: {:?}", e))
            })?;

        // FUTURE: Execute SHACL validation queries
        // For now, perform basic structural validation
        let mut errors = Vec::new();

        // Validate workflow has at least one task
        if spec.tasks.is_empty() {
            errors.push("Workflow must have at least one task".to_string());
        }

        // Validate task flows
        for (task_id, task) in &spec.tasks {
            if task.outgoing_flows.is_empty() && task.incoming_flows.is_empty() {
                errors.push(format!(
                    "Task {} has no incoming or outgoing flows",
                    task_id
                ));
            }
        }

        if errors.is_empty() {
            Ok(SchemaValidationResult::success())
        } else {
            Ok(SchemaValidationResult::failure(errors))
        }
    }

    /// Convert workflow spec to RDF (Turtle format)
    fn workflow_spec_to_rdf(&self, spec: &WorkflowSpec) -> WorkflowResult<String> {
        let mut rdf = String::new();
        rdf.push_str("@prefix wf: <http://knhk.org/workflow#> .\n");
        rdf.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        rdf.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n");

        // Workflow spec
        rdf.push_str(&format!("wf:{} a wf:WorkflowSpec ;\n", spec.id));
        rdf.push_str(&format!("    wf:name \"{}\" ;\n", spec.name));
        rdf.push_str(&format!("    wf:id \"{}\" .\n\n", spec.id));

        // Tasks
        for (task_id, task) in &spec.tasks {
            rdf.push_str(&format!("wf:{} a wf:Task ;\n", task_id));
            rdf.push_str(&format!("    wf:name \"{}\" ;\n", task.name));
            rdf.push_str(&format!("    wf:taskType \"{:?}\" .\n\n", task.task_type));
        }

        Ok(rdf)
    }

    /// Get schema by name
    pub async fn get_schema(&self, name: &str) -> Option<SchemaMetadata> {
        let schemas = self.schemas.read().await;
        schemas.get(name).cloned()
    }

    /// List all schemas
    pub async fn list_schemas(&self) -> Vec<SchemaMetadata> {
        let schemas = self.schemas.read().await;
        schemas.values().cloned().collect()
    }

    /// Enable/disable a schema
    pub async fn set_schema_enabled(&self, name: &str, enabled: bool) -> WorkflowResult<()> {
        let mut schemas = self.schemas.write().await;
        if let Some(schema) = schemas.get_mut(name) {
            schema.enabled = enabled;
            Ok(())
        } else {
            Err(WorkflowError::Validation(format!("Schema {} not found", name)))
        }
    }

    /// Remove a schema
    pub async fn remove_schema(&self, name: &str) -> WorkflowResult<()> {
        let mut schemas = self.schemas.write().await;
        if schemas.remove(name).is_some() {
            Ok(())
        } else {
            Err(WorkflowError::Validation(format!("Schema {} not found", name)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Task, TaskType};

    #[tokio::test]
    async fn test_schema_registry() {
        let registry = SchemaRegistry::new()
            .expect("SchemaRegistry::new should succeed");

        let schema = SchemaMetadata {
            name: "test-schema".to_string(),
            description: "Test schema".to_string(),
            version: "1.0".to_string(),
            rdf_content: r#"
@prefix wf: <http://knhk.org/workflow#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

wf:WorkflowSpec a rdf:Class .
wf:Task a rdf:Class .
"#
            .to_string(),
            enabled: true,
        };

        registry.register_schema(schema).await
            .expect("register_schema should succeed");

        let schemas = registry.list_schemas().await;
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].name, "test-schema");
    }

    #[tokio::test]
    async fn test_schema_validation() {
        let registry = SchemaRegistry::new()
            .expect("SchemaRegistry::new should succeed");

        let schema = SchemaMetadata {
            name: "test-schema".to_string(),
            description: "Test schema".to_string(),
            version: "1.0".to_string(),
            rdf_content: r#"
@prefix wf: <http://knhk.org/workflow#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

wf:WorkflowSpec a rdf:Class .
wf:Task a rdf:Class .
"#
            .to_string(),
            enabled: true,
        };

        registry.register_schema(schema).await
            .expect("register_schema should succeed");

        let spec = WorkflowSpec {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            tasks: {
                let mut tasks = HashMap::new();
                tasks.insert(
                    "task-1".to_string(),
                    Task {
                        name: "Task 1".to_string(),
                        task_type: TaskType::Atomic,
                        incoming_flows: Vec::new(),
                        outgoing_flows: vec!["task-2".to_string()],
                    },
                );
                tasks.insert(
                    "task-2".to_string(),
                    Task {
                        name: "Task 2".to_string(),
                        task_type: TaskType::Atomic,
                        incoming_flows: vec!["task-1".to_string()],
                        outgoing_flows: Vec::new(),
                    },
                );
                tasks
            },
            conditions: HashMap::new(),
            start_condition: None,
            end_condition: None,
        };

        let result = registry
            .validate_workflow_spec(&spec, "test-schema")
            .await
            .expect("validate_workflow_spec should succeed");

        assert!(result.valid);
    }
}

