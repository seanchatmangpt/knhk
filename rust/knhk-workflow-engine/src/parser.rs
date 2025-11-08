//! Turtle/YAWL workflow parser

use oxigraph::io::RdfFormat;
use oxigraph::store::Store;
use std::io::Read;
use uuid::Uuid;

use crate::error::{WorkflowError, WorkflowResult};

/// Unique identifier for a workflow specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct WorkflowSpecId(#[serde(with = "uuid::serde::compact")] pub Uuid);

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
        let store = Store::new()
            .map_err(|e| WorkflowError::Parse(format!("Failed to create RDF store: {:?}", e)))?;
        Ok(Self { store })
    }

    /// Parse workflow from Turtle string
    pub fn parse_turtle(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // Parse Turtle into RDF store using oxigraph's built-in parser
        self.store
            .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| WorkflowError::Parse(format!("Failed to load Turtle: {:?}", e)))?;

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
        let rdfs = "http://www.w3.org/2000/01/rdf-schema#";
        let rdf_type_iri = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

        let rdf_type = oxigraph::model::NamedNode::new(rdf_type_iri)
            .map_err(|e| WorkflowError::Parse(format!("Invalid IRI: {}", e)))?;
        let workflow_spec_type =
            oxigraph::model::NamedNode::new(&format!("{}WorkflowSpecification", yawl_ns))
                .map_err(|e| WorkflowError::Parse(format!("Invalid IRI: {}", e)))?;
        let rdfs_label = oxigraph::model::NamedNode::new(&format!("{}label", rdfs))
            .map_err(|e| WorkflowError::Parse(format!("Invalid IRI: {}", e)))?;

        // Find workflow specifications
        let query = format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX rdfs: <{}>\n\
             SELECT ?spec ?name WHERE {{\n\
               ?spec rdf:type yawl:WorkflowSpecification .\n\
               OPTIONAL {{ ?spec rdfs:label ?name }}\n\
             }} LIMIT 1",
            yawl_ns, rdfs
        );

        #[allow(deprecated)]
        let query_results = self
            .store
            .query(&query)
            .map_err(|e| WorkflowError::Parse(format!("SPARQL query failed: {}", e)))?;

        let spec_id = WorkflowSpecId::new();
        let mut spec_name = "Parsed Workflow".to_string();
        let mut spec_iri: Option<String> = None;

        // Extract workflow spec IRI and name
        if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    WorkflowError::Parse(format!("Failed to process query solution: {:?}", e))
                })?;

                if let Some(spec_term) = solution.get("spec") {
                    spec_iri = Some(spec_term.to_string());
                }
                if let Some(name_term) = solution.get("name") {
                    if let oxigraph::model::Term::Literal(lit) = name_term {
                        spec_name = lit.value().to_string();
                    }
                }
            }
        }

        // Extract tasks
        let tasks = self.extract_tasks(&yawl_ns, spec_iri.as_deref())?;

        // Extract conditions
        let conditions = self.extract_conditions(&yawl_ns, spec_iri.as_deref())?;

        // Extract flows
        self.extract_flows(&yawl_ns, spec_iri.as_deref(), &mut tasks, &mut conditions)?;

        // Find start and end conditions
        let start_condition = self.find_start_condition(&yawl_ns, spec_iri.as_deref())?;
        let end_condition = self.find_end_condition(&yawl_ns, spec_iri.as_deref())?;

        let spec = WorkflowSpec {
            id: spec_id,
            name: spec_name,
            tasks,
            conditions,
            start_condition,
            end_condition,
        };

        Ok(spec)
    }

    /// Extract tasks from RDF store
    fn extract_tasks(
        &self,
        yawl_ns: &str,
        spec_iri: Option<&str>,
    ) -> WorkflowResult<std::collections::HashMap<String, Task>> {
        let mut tasks = std::collections::HashMap::new();

        let query = if let Some(spec) = spec_iri {
            format!(
                "PREFIX yawl: <{}>\n\
                 PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
                 PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
                 SELECT ?task ?name ?type ?split ?join ?maxTicks ?priority ?simd WHERE {{\n\
                   <{}> yawl:hasTask ?task .\n\
                   ?task rdf:type ?type .\n\
                   OPTIONAL {{ ?task rdfs:label ?name }}\n\
                   OPTIONAL {{ ?task yawl:splitType ?split }}\n\
                   OPTIONAL {{ ?task yawl:joinType ?join }}\n\
                   OPTIONAL {{ ?task yawl:maxTicks ?maxTicks }}\n\
                   OPTIONAL {{ ?task yawl:priority ?priority }}\n\
                   OPTIONAL {{ ?task yawl:useSimd ?simd }}\n\
                 }}",
                yawl_ns, spec
            )
        } else {
            format!(
                "PREFIX yawl: <{}>\n\
                 PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
                 PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
                 SELECT ?task ?name ?type ?split ?join ?maxTicks ?priority ?simd WHERE {{\n\
                   ?task rdf:type yawl:Task .\n\
                   OPTIONAL {{ ?task rdfs:label ?name }}\n\
                   OPTIONAL {{ ?task yawl:splitType ?split }}\n\
                   OPTIONAL {{ ?task yawl:joinType ?join }}\n\
                   OPTIONAL {{ ?task yawl:maxTicks ?maxTicks }}\n\
                   OPTIONAL {{ ?task yawl:priority ?priority }}\n\
                   OPTIONAL {{ ?task yawl:useSimd ?simd }}\n\
                 }}",
                yawl_ns
            )
        };

        #[allow(deprecated)]
        let query_results = self
            .store
            .query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Failed to query tasks: {:?}", e)))?;

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    WorkflowError::Parse(format!("Failed to process task solution: {:?}", e))
                })?;

                let task_id = solution
                    .get("task")
                    .map(|t| t.to_string())
                    .ok_or_else(|| WorkflowError::Parse("Task IRI missing".into()))?;

                let task_name = solution
                    .get("name")
                    .and_then(|t| {
                        if let oxigraph::model::Term::Literal(lit) = t {
                            Some(lit.value().to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "Unnamed Task".to_string());

                // Determine task type
                let task_type = solution
                    .get("type")
                    .and_then(|t| {
                        let type_str = t.to_string();
                        if type_str.contains("CompositeTask") {
                            Some(TaskType::Composite)
                        } else if type_str.contains("MultipleInstanceTask") {
                            Some(TaskType::MultipleInstance)
                        } else {
                            Some(TaskType::Atomic)
                        }
                    })
                    .unwrap_or(TaskType::Atomic);

                // Parse split type
                let split_type = solution
                    .get("split")
                    .and_then(|s| {
                        let split_str = s.to_string().to_lowercase();
                        if split_str.contains("and") {
                            Some(SplitType::And)
                        } else if split_str.contains("xor") {
                            Some(SplitType::Xor)
                        } else if split_str.contains("or") {
                            Some(SplitType::Or)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(SplitType::Xor);

                // Parse join type
                let join_type = solution
                    .get("join")
                    .and_then(|j| {
                        let join_str = j.to_string().to_lowercase();
                        if join_str.contains("and") {
                            Some(JoinType::And)
                        } else if join_str.contains("xor") {
                            Some(JoinType::Xor)
                        } else if join_str.contains("or") {
                            Some(JoinType::Or)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(JoinType::Xor);

                let max_ticks = solution.get("maxTicks").and_then(|t| {
                    if let oxigraph::model::Term::Literal(lit) = t {
                        lit.value().parse::<u32>().ok()
                    } else {
                        None
                    }
                });

                let priority = solution.get("priority").and_then(|p| {
                    if let oxigraph::model::Term::Literal(lit) = p {
                        lit.value().parse::<u32>().ok()
                    } else {
                        None
                    }
                });

                let use_simd = solution
                    .get("simd")
                    .and_then(|s| {
                        if let oxigraph::model::Term::Literal(lit) = s {
                            lit.value().parse::<bool>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false);

                let task = Task {
                    id: task_id.clone(),
                    name: task_name,
                    task_type,
                    split_type,
                    join_type,
                    max_ticks,
                    priority,
                    use_simd,
                    input_conditions: Vec::new(), // Will be populated by extract_flows
                    output_conditions: Vec::new(),
                    outgoing_flows: Vec::new(),
                    incoming_flows: Vec::new(),
                };

                tasks.insert(task_id, task);
            }
        }

        Ok(tasks)
    }

    /// Extract conditions from RDF store
    fn extract_conditions(
        &self,
        yawl_ns: &str,
        spec_iri: Option<&str>,
    ) -> WorkflowResult<std::collections::HashMap<String, Condition>> {
        let mut conditions = std::collections::HashMap::new();

        let query = if let Some(spec) = spec_iri {
            format!(
                "PREFIX yawl: <{}>\n\
                 PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
                 PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
                 SELECT ?condition ?name WHERE {{\n\
                   <{}> yawl:hasCondition ?condition .\n\
                   OPTIONAL {{ ?condition rdfs:label ?name }}\n\
                 }}",
                yawl_ns, spec
            )
        } else {
            format!(
                "PREFIX yawl: <{}>\n\
                 PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
                 PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
                 SELECT ?condition ?name WHERE {{\n\
                   ?condition rdf:type yawl:Condition .\n\
                   OPTIONAL {{ ?condition rdfs:label ?name }}\n\
                 }}",
                yawl_ns
            )
        };

        #[allow(deprecated)]
        let query_results = self
            .store
            .query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Failed to query conditions: {:?}", e)))?;

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    WorkflowError::Parse(format!("Failed to process condition solution: {:?}", e))
                })?;

                let condition_id = solution
                    .get("condition")
                    .map(|c| c.to_string())
                    .ok_or_else(|| WorkflowError::Parse("Condition IRI missing".into()))?;

                let condition_name = solution
                    .get("name")
                    .and_then(|n| {
                        if let oxigraph::model::Term::Literal(lit) = n {
                            Some(lit.value().to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "Unnamed Condition".to_string());

                let condition = Condition {
                    id: condition_id.clone(),
                    name: condition_name,
                    outgoing_flows: Vec::new(), // Will be populated by extract_flows
                    incoming_flows: Vec::new(),
                };

                conditions.insert(condition_id, condition);
            }
        }

        Ok(conditions)
    }

    /// Extract flows and connect tasks/conditions
    fn extract_flows(
        &self,
        yawl_ns: &str,
        spec_iri: Option<&str>,
        tasks: &mut std::collections::HashMap<String, Task>,
        conditions: &mut std::collections::HashMap<String, Condition>,
    ) -> WorkflowResult<()> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             SELECT ?from ?to ?type WHERE {{\n\
               ?flow yawl:from ?from .\n\
               ?flow yawl:to ?to .\n\
               OPTIONAL {{ ?flow yawl:flowType ?type }}\n\
             }}",
            yawl_ns
        );

        #[allow(deprecated)]
        let query_results = self
            .store
            .query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Failed to query flows: {:?}", e)))?;

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    WorkflowError::Parse(format!("Failed to process flow solution: {:?}", e))
                })?;

                let from_id = solution
                    .get("from")
                    .map(|f| f.to_string())
                    .ok_or_else(|| WorkflowError::Parse("Flow from missing".into()))?;
                let to_id = solution
                    .get("to")
                    .map(|t| t.to_string())
                    .ok_or_else(|| WorkflowError::Parse("Flow to missing".into()))?;

                // Connect task to condition or condition to task
                if let Some(task) = tasks.get_mut(&from_id) {
                    // Task -> Condition
                    task.outgoing_flows.push(to_id.clone());
                    if let Some(condition) = conditions.get_mut(&to_id) {
                        condition.incoming_flows.push(from_id.clone());
                    }
                } else if let Some(condition) = conditions.get_mut(&from_id) {
                    // Condition -> Task
                    condition.outgoing_flows.push(to_id.clone());
                    if let Some(task) = tasks.get_mut(&to_id) {
                        task.incoming_flows.push(from_id.clone());
                        task.input_conditions.push(from_id.clone());
                    }
                }
            }
        }

        Ok(())
    }

    /// Find start condition
    fn find_start_condition(
        &self,
        yawl_ns: &str,
        spec_iri: Option<&str>,
    ) -> WorkflowResult<Option<String>> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             SELECT ?condition WHERE {{\n\
               ?condition rdf:type yawl:StartCondition .\n\
             }} LIMIT 1",
            yawl_ns
        );

        #[allow(deprecated)]
        let query_results = self.store.query(&query).map_err(|e| {
            WorkflowError::Parse(format!("Failed to query start condition: {:?}", e))
        })?;

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    WorkflowError::Parse(format!(
                        "Failed to process start condition solution: {:?}",
                        e
                    ))
                })?;

                if let Some(condition_term) = solution.get("condition") {
                    return Ok(Some(condition_term.to_string()));
                }
            }
        }

        Ok(None)
    }

    /// Find end condition
    fn find_end_condition(
        &self,
        yawl_ns: &str,
        spec_iri: Option<&str>,
    ) -> WorkflowResult<Option<String>> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             SELECT ?condition WHERE {{\n\
               ?condition rdf:type yawl:EndCondition .\n\
             }} LIMIT 1",
            yawl_ns
        );

        #[allow(deprecated)]
        let query_results = self
            .store
            .query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Failed to query end condition: {:?}", e)))?;

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    WorkflowError::Parse(format!(
                        "Failed to process end condition solution: {:?}",
                        e
                    ))
                })?;

                if let Some(condition_term) = solution.get("condition") {
                    return Ok(Some(condition_term.to_string()));
                }
            }
        }

        Ok(None)
    }

    /// Load YAWL ontology
    pub fn load_yawl_ontology(&mut self, ontology_path: &std::path::Path) -> WorkflowResult<()> {
        let mut file = std::fs::File::open(ontology_path)
            .map_err(|e| WorkflowError::Parse(format!("Failed to open ontology: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| WorkflowError::Parse(format!("Failed to read ontology: {}", e)))?;

        // Parse Turtle and load into store
        self.store
            .load_from_reader(RdfFormat::Turtle, contents.as_bytes())
            .map_err(|e| WorkflowError::Parse(format!("Failed to load Turtle: {:?}", e)))?;

        Ok(())
    }
}

impl Default for WorkflowParser {
    fn default() -> Self {
        // Default implementation should not fail
        // If new() fails, we'll panic as this is a programming error
        // FUTURE: Consider making Default return Result or use a static parser
        Self::new().unwrap_or_else(|e| panic!("Failed to create workflow parser: {:?}", e))
    }
}
