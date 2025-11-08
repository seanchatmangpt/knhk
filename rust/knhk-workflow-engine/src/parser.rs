//! Turtle/YAWL workflow parser

use oxigraph::store::Store;
use oxigraph::store::StoreOptions;
use oxigraph::Model;
use rio_turtle::{TurtleError, TurtleParser};
use std::io::Read;
use uuid::Uuid;

use crate::error::{WorkflowError, WorkflowResult};

/// Unique identifier for a workflow specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct WorkflowSpecId(pub Uuid);

impl WorkflowSpecId {
    /// Generate a new spec ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse from string
    pub fn parse_str(s: &str) -> WorkflowResult<Self> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|e| WorkflowError::Parse(format!("Invalid spec ID: {}", e)))
    }
}

impl Default for WorkflowSpecId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WorkflowSpecId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Split type (AND, XOR, OR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SplitType {
    /// AND-split: all branches execute
    And,
    /// XOR-split: exactly one branch executes
    Xor,
    /// OR-split: one or more branches execute
    Or,
}

/// Join type (AND, XOR, OR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JoinType {
    /// AND-join: wait for all branches
    And,
    /// XOR-join: wait for one branch
    Xor,
    /// OR-join: wait for all active branches
    Or,
}

/// Task type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    /// Atomic task (cannot be decomposed)
    Atomic,
    /// Composite task (contains sub-workflow)
    Composite,
    /// Multiple instance task
    MultipleInstance,
}

/// Workflow task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    /// Task identifier (IRI)
    pub id: String,
    /// Task name/label
    pub name: String,
    /// Task type
    pub task_type: TaskType,
    /// Split type
    pub split_type: SplitType,
    /// Join type
    pub join_type: JoinType,
    /// Maximum execution ticks (≤8 for hot path)
    pub max_ticks: Option<u32>,
    /// Priority (0-255)
    pub priority: Option<u32>,
    /// Use SIMD optimization
    pub use_simd: bool,
    /// Input conditions
    pub input_conditions: Vec<String>,
    /// Output conditions
    pub output_conditions: Vec<String>,
    /// Outgoing flows (task IDs)
    pub outgoing_flows: Vec<String>,
    /// Incoming flows (task IDs)
    pub incoming_flows: Vec<String>,
}

/// Workflow condition (place in Petri net)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Condition {
    /// Condition identifier (IRI)
    pub id: String,
    /// Condition name/label
    pub name: String,
    /// Outgoing flows (task IDs)
    pub outgoing_flows: Vec<String>,
    /// Incoming flows (task IDs)
    pub incoming_flows: Vec<String>,
}

/// Workflow specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowSpec {
    /// Unique specification identifier
    pub id: WorkflowSpecId,
    /// Specification name
    pub name: String,
    /// Tasks in the workflow
    pub tasks: std::collections::HashMap<String, Task>,
    /// Conditions in the workflow
    pub conditions: std::collections::HashMap<String, Condition>,
    /// Start condition ID
    pub start_condition: Option<String>,
    /// End condition ID
    pub end_condition: Option<String>,
}

/// Turtle/YAWL parser
pub struct WorkflowParser {
    store: Store,
}

impl WorkflowParser {
    /// Create a new parser
    pub fn new() -> WorkflowResult<Self> {
        let store = Store::new(StoreOptions::default())
            .map_err(|e| WorkflowError::Parse(format!("Failed to create RDF store: {}", e)))?;
        Ok(Self { store })
    }

    /// Parse workflow from Turtle string
    pub fn parse_turtle(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // Parse Turtle into RDF store
        let parser = TurtleParser::new(turtle.as_bytes(), None);
        let mut quads = Vec::new();
        
        for quad in parser {
            let quad = quad.map_err(|e| WorkflowError::from(e))?;
            quads.push(quad);
        }

        // Load into store
        for quad in quads {
            self.store.insert(&quad).map_err(|e| WorkflowError::from(e))?;
        }

        // Extract workflow specification
        self.extract_workflow_spec()
    }

    /// Parse workflow from file
    pub fn parse_file(&mut self, path: &std::path::Path) -> WorkflowResult<WorkflowSpec> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| WorkflowError::Parse(format!("Failed to open file: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| WorkflowError::Parse(format!("Failed to read file: {}", e)))?;
        self.parse_turtle(&contents)
    }

    /// Extract workflow specification from RDF store
    fn extract_workflow_spec(&self) -> WorkflowResult<WorkflowSpec> {
        // YAWL namespace prefixes
        let yawl_ns = "http://bitflow.ai/ontology/yawl/v2#";
        let rdf_type = oxigraph::model::NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
            .map_err(|e| WorkflowError::Parse(format!("Invalid IRI: {}", e)))?;
        let workflow_spec_type = oxigraph::model::NamedNode::new(&format!("{}WorkflowSpecification", yawl_ns))
            .map_err(|e| WorkflowError::Parse(format!("Invalid IRI: {}", e)))?;

        // Find workflow specifications
        let query = format!(
            "SELECT ?spec WHERE {{ ?spec <{}> <{}> }}",
            rdf_type.as_str(),
            workflow_spec_type.as_str()
        );

        let query_results = self.store.query(&query)
            .map_err(|e| WorkflowError::Parse(format!("SPARQL query failed: {}", e)))?;

        // For now, create a basic workflow spec
        // FUTURE: Full extraction of tasks, conditions, flows from RDF
        let spec_id = WorkflowSpecId::new();
        let spec = WorkflowSpec {
            id: spec_id,
            name: "Parsed Workflow".to_string(),
            tasks: std::collections::HashMap::new(),
            conditions: std::collections::HashMap::new(),
            start_condition: None,
            end_condition: None,
        };

        Ok(spec)
    }

    /// Load YAWL ontology
    pub fn load_yawl_ontology(&mut self, ontology_path: &std::path::Path) -> WorkflowResult<()> {
        let mut file = std::fs::File::open(ontology_path)
            .map_err(|e| WorkflowError::Parse(format!("Failed to open ontology: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| WorkflowError::Parse(format!("Failed to read ontology: {}", e)))?;
        
        let parser = TurtleParser::new(contents.as_bytes(), None);
        for quad in parser {
            let quad = quad.map_err(|e| WorkflowError::from(e))?;
            self.store.insert(&quad).map_err(|e| WorkflowError::from(e))?;
        }

        Ok(())
    }
}

impl Default for WorkflowParser {
    fn default() -> Self {
        // Default implementation should not fail
        // If new() fails, we'll panic as this is a programming error
        // FUTURE: Consider making Default return Result or use a static parser
        Self::new().unwrap_or_else(|e| {
            panic!("Failed to create workflow parser: {:?}", e)
        })
    }
}

