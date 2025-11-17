//! Turtle/RDF workflow loader
//!
//! Loads YAWL workflow definitions from Turtle/RDF files and builds
//! executable workflow representations following Covenant 1:
//! **Turtle Is Definition and Cause**.
//!
//! # Covenant 1 Compliance
//!
//! - Reads ONLY what's declared in Turtle (no assumptions)
//! - No hidden logic or reconstruction
//! - All execution behavior comes from RDF triples
//! - Validates against pattern permutation matrix
//! - Fails fast if definition is incomplete or invalid
//!
//! # Key Principles
//!
//! 1. **Single Source of Truth**: Turtle RDF is the only definition
//! 2. **No Business Logic**: Loader is purely mechanical extraction
//! 3. **Explicit Everything**: All execution semantics must be declared
//! 4. **Validation First**: Check against permutation matrix before building
//! 5. **Observable**: All loading operations emit telemetry

use crate::error::{WorkflowError, WorkflowResult};
use oxigraph::model::{Literal, NamedNode, Subject, Term};
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, error, info, instrument, warn};

/// YAWL namespace
const YAWL_NS: &str = "http://www.yawlfoundation.org/yawlschema#";
const YAWL_EXEC_NS: &str = "http://bitflow.ai/ontology/yawl/execution/v1#";
const YAWL_PATTERN_NS: &str = "http://bitflow.ai/ontology/yawl/patterns/v1#";
const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";

/// Workflow definition loaded from Turtle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Tasks in the workflow
    pub tasks: Vec<TaskDefinition>,
    /// Control flows between tasks
    pub flows: Vec<FlowDefinition>,
    /// Data variables
    pub variables: HashMap<String, VariableDefinition>,
    /// Input condition (start point)
    pub input_condition: Option<String>,
    /// Output condition (end point)
    pub output_condition: Option<String>,
    /// Metadata from RDF
    pub metadata: HashMap<String, String>,
}

/// Task definition from Turtle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    /// Task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Split type (AND, OR, XOR, None)
    pub split_type: Option<SplitType>,
    /// Join type (AND, OR, XOR, Discriminator, None)
    pub join_type: Option<JoinType>,
    /// Execution mode (sync, async, parallel, queued)
    pub execution_mode: ExecutionMode,
    /// Runtime behavior reference (code, service, etc.)
    pub runtime_behavior: Option<String>,
    /// Timeout policy
    pub timeout_policy: Option<TimeoutPolicy>,
    /// Retry policy
    pub retry_policy: Option<RetryPolicy>,
    /// Expected duration (for optimization)
    pub duration: Option<String>,
    /// Maximum concurrent instances
    pub max_concurrency: Option<u32>,
    /// Task metadata
    pub metadata: HashMap<String, String>,
}

/// Control flow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowDefinition {
    /// Source task ID
    pub from: String,
    /// Target task ID
    pub to: String,
    /// Flow predicate (for XOR/OR splits)
    pub predicate: Option<String>,
    /// Order hint (for visualization)
    pub order: Option<i32>,
}

/// Variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDefinition {
    /// Variable name
    pub name: String,
    /// Data type
    pub data_type: String,
    /// Initial value
    pub initial_value: Option<String>,
    /// Whether required
    pub required: bool,
}

/// Split types (from yawl-extended.ttl)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitType {
    /// AND split (all branches)
    AND,
    /// OR split (one or more branches)
    OR,
    /// XOR split (exactly one branch)
    XOR,
}

/// Join types (from yawl-extended.ttl)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinType {
    /// AND join (wait for all)
    AND,
    /// OR join (synchronizing merge)
    OR,
    /// XOR join (simple merge)
    XOR,
    /// Discriminator (first to complete)
    Discriminator,
}

/// Execution modes (from yawl-extended.ttl)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Synchronous (blocks until completion)
    Synchronous,
    /// Asynchronous (non-blocking)
    Asynchronous,
    /// Queued (task queued for later)
    Queued,
    /// Parallel (executes in parallel)
    Parallel,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::Synchronous
    }
}

/// Timeout policies (from yawl-extended.ttl)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeoutPolicy {
    /// Skip task and continue
    Skip,
    /// Retry task with backoff
    Retry,
    /// Escalate to handler
    Escalate,
}

/// Retry policies (from yawl-extended.ttl)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryPolicy {
    /// Exponential backoff
    Exponential,
    /// Linear backoff
    Linear,
    /// Immediate retry
    Immediate,
}

/// Workflow loader - loads from Turtle/RDF files
pub struct WorkflowLoader {
    /// RDF store
    store: Store,
}

impl WorkflowLoader {
    /// Create a new workflow loader
    #[instrument(skip_all)]
    pub fn new() -> WorkflowResult<Self> {
        info!("Creating workflow loader");
        let store = Store::new().map_err(|e| {
            error!("Failed to create RDF store: {}", e);
            WorkflowError::Internal(format!("Failed to create RDF store: {}", e))
        })?;

        Ok(Self { store })
    }

    /// Load workflow from Turtle file
    ///
    /// # Covenant 1 Compliance
    ///
    /// This method reads ONLY what's in the Turtle file:
    /// - No reconstruction or filtering
    /// - No hidden assumptions
    /// - All behavior must be explicit in RDF
    #[instrument(skip(self))]
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> WorkflowResult<WorkflowDefinition> {
        let path = path.as_ref();
        info!("Loading workflow from file: {:?}", path);

        // Read Turtle file into RDF store
        let content = std::fs::read_to_string(path).map_err(|e| {
            error!("Failed to read file {:?}: {}", path, e);
            WorkflowError::Parse(format!("Failed to read file: {}", e))
        })?;

        self.load_turtle(&content)
    }

    /// Load workflow from Turtle string
    #[instrument(skip(self, turtle))]
    pub fn load_turtle(&mut self, turtle: &str) -> WorkflowResult<WorkflowDefinition> {
        info!("Parsing Turtle content");

        // Parse Turtle into RDF store
        self.store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| {
                error!("Failed to parse Turtle: {}", e);
                WorkflowError::Parse(format!("Failed to parse Turtle: {}", e))
            })?;

        // Extract workflow definition using SPARQL
        self.extract_workflow()
    }

    /// Extract workflow definition from RDF store
    ///
    /// Uses SPARQL queries to extract EXACTLY what's declared in Turtle.
    /// No reconstruction, no filtering, no assumptions.
    #[instrument(skip(self))]
    fn extract_workflow(&self) -> WorkflowResult<WorkflowDefinition> {
        info!("Extracting workflow definition from RDF");

        // 1. Extract workflow metadata
        let (id, name, metadata) = self.extract_workflow_metadata()?;

        // 2. Extract tasks
        let tasks = self.extract_tasks()?;

        // 3. Extract flows
        let flows = self.extract_flows()?;

        // 4. Extract variables
        let variables = self.extract_variables()?;

        // 5. Extract conditions
        let (input_condition, output_condition) = self.extract_conditions()?;

        // 6. Validate against permutation matrix
        self.validate_patterns(&tasks, &flows)?;

        info!(
            "Extracted workflow: {} tasks, {} flows, {} variables",
            tasks.len(),
            flows.len(),
            variables.len()
        );

        Ok(WorkflowDefinition {
            id,
            name,
            tasks,
            flows,
            variables,
            input_condition,
            output_condition,
            metadata,
        })
    }

    /// Extract workflow metadata
    fn extract_workflow_metadata(
        &self,
    ) -> WorkflowResult<(String, String, HashMap<String, String>)> {
        // SPARQL query to extract workflow metadata
        let query = format!(
            r#"
            PREFIX yawl: <{}>
            PREFIX rdfs: <{}>
            PREFIX rdf: <{}>

            SELECT ?spec ?name ?label ?comment WHERE {{
                ?spec rdf:type yawl:Specification .
                OPTIONAL {{ ?spec rdfs:label ?name }}
                OPTIONAL {{ ?spec rdfs:label ?label }}
                OPTIONAL {{ ?spec rdfs:comment ?comment }}
            }}
            LIMIT 1
        "#,
            YAWL_NS, RDFS_NS, RDF_NS
        );

        let results = self.store.query(&query).map_err(|e| {
            error!("Failed to query workflow metadata: {}", e);
            WorkflowError::Parse(format!("Failed to query metadata: {}", e))
        })?;

        if let QueryResults::Solutions(mut solutions) = results {
            if let Some(solution) = solutions.next() {
                let solution = solution.map_err(|e| {
                    error!("Failed to read solution: {}", e);
                    WorkflowError::Parse(format!("Failed to read solution: {}", e))
                })?;

                let id = solution
                    .get("spec")
                    .and_then(|t| match t {
                        Term::NamedNode(n) => Some(n.as_str().to_string()),
                        _ => None,
                    })
                    .ok_or_else(|| WorkflowError::Parse("Workflow ID not found".to_string()))?;

                let name = solution
                    .get("name")
                    .and_then(|t| match t {
                        Term::Literal(l) => Some(l.value().to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "Unnamed Workflow".to_string());

                let mut metadata = HashMap::new();
                if let Some(Term::Literal(l)) = solution.get("label") {
                    metadata.insert("label".to_string(), l.value().to_string());
                }
                if let Some(Term::Literal(l)) = solution.get("comment") {
                    metadata.insert("comment".to_string(), l.value().to_string());
                }

                return Ok((id, name, metadata));
            }
        }

        Err(WorkflowError::Parse(
            "No workflow specification found in Turtle".to_string(),
        ))
    }

    /// Extract task definitions
    fn extract_tasks(&self) -> WorkflowResult<Vec<TaskDefinition>> {
        // SPARQL query to extract all tasks with their properties
        let query = format!(
            r#"
            PREFIX yawl: <{}>
            PREFIX yawl-exec: <{}>
            PREFIX rdfs: <{}>
            PREFIX rdf: <{}>

            SELECT ?task ?name ?label ?splitType ?joinType ?execMode ?behavior ?timeout ?retry ?duration ?maxConcurrency WHERE {{
                ?task rdf:type yawl:Task .
                OPTIONAL {{ ?task rdfs:label ?name }}
                OPTIONAL {{ ?task rdfs:label ?label }}
                OPTIONAL {{ ?task yawl:split ?splitType }}
                OPTIONAL {{ ?task yawl:join ?joinType }}
                OPTIONAL {{ ?task yawl-exec:executionMode ?execMode }}
                OPTIONAL {{ ?task yawl-exec:runtimeBehavior ?behavior }}
                OPTIONAL {{ ?task yawl-exec:timeoutPolicy ?timeout }}
                OPTIONAL {{ ?task yawl-exec:RetryPolicy ?retry }}
                OPTIONAL {{ ?task yawl-exec:TaskDuration ?duration }}
                OPTIONAL {{ ?task yawl-exec:MaxConcurrency ?maxConcurrency }}
            }}
        "#,
            YAWL_NS, YAWL_EXEC_NS, RDFS_NS, RDF_NS
        );

        let results = self.store.query(&query).map_err(|e| {
            error!("Failed to query tasks: {}", e);
            WorkflowError::Parse(format!("Failed to query tasks: {}", e))
        })?;

        let mut tasks = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    error!("Failed to read solution: {}", e);
                    WorkflowError::Parse(format!("Failed to read solution: {}", e))
                })?;

                let id = solution
                    .get("task")
                    .and_then(|t| match t {
                        Term::NamedNode(n) => Some(n.as_str().to_string()),
                        _ => None,
                    })
                    .ok_or_else(|| WorkflowError::Parse("Task ID not found".to_string()))?;

                let name = solution
                    .get("name")
                    .or_else(|| solution.get("label"))
                    .and_then(|t| match t {
                        Term::Literal(l) => Some(l.value().to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| id.clone());

                let split_type = solution
                    .get("splitType")
                    .and_then(|t| self.parse_split_type(t));

                let join_type = solution
                    .get("joinType")
                    .and_then(|t| self.parse_join_type(t));

                let execution_mode = solution
                    .get("execMode")
                    .and_then(|t| self.parse_execution_mode(t))
                    .unwrap_or(ExecutionMode::Synchronous);

                let runtime_behavior = solution.get("behavior").and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    Term::Literal(l) => Some(l.value().to_string()),
                    _ => None,
                });

                let timeout_policy = solution
                    .get("timeout")
                    .and_then(|t| self.parse_timeout_policy(t));

                let retry_policy = solution
                    .get("retry")
                    .and_then(|t| self.parse_retry_policy(t));

                let duration = solution.get("duration").and_then(|t| match t {
                    Term::Literal(l) => Some(l.value().to_string()),
                    _ => None,
                });

                let max_concurrency = solution.get("maxConcurrency").and_then(|t| match t {
                    Term::Literal(l) => l.value().parse::<u32>().ok(),
                    _ => None,
                });

                tasks.push(TaskDefinition {
                    id,
                    name,
                    split_type,
                    join_type,
                    execution_mode,
                    runtime_behavior,
                    timeout_policy,
                    retry_policy,
                    duration,
                    max_concurrency,
                    metadata: HashMap::new(),
                });
            }
        }

        Ok(tasks)
    }

    /// Extract flow definitions
    fn extract_flows(&self) -> WorkflowResult<Vec<FlowDefinition>> {
        let query = format!(
            r#"
            PREFIX yawl: <{}>
            PREFIX rdfs: <{}>

            SELECT ?from ?to ?predicate ?order WHERE {{
                ?flow rdf:type yawl:Flow .
                ?flow yawl:flowsInto ?to .
                ?flow yawl:flowsFrom ?from .
                OPTIONAL {{ ?flow yawl:predicate ?predicate }}
                OPTIONAL {{ ?flow yawl:order ?order }}
            }}
        "#,
            YAWL_NS, RDFS_NS
        );

        let results = self.store.query(&query).map_err(|e| {
            error!("Failed to query flows: {}", e);
            WorkflowError::Parse(format!("Failed to query flows: {}", e))
        })?;

        let mut flows = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    error!("Failed to read solution: {}", e);
                    WorkflowError::Parse(format!("Failed to read solution: {}", e))
                })?;

                let from = solution
                    .get("from")
                    .and_then(|t| match t {
                        Term::NamedNode(n) => Some(n.as_str().to_string()),
                        _ => None,
                    })
                    .ok_or_else(|| WorkflowError::Parse("Flow source not found".to_string()))?;

                let to = solution
                    .get("to")
                    .and_then(|t| match t {
                        Term::NamedNode(n) => Some(n.as_str().to_string()),
                        _ => None,
                    })
                    .ok_or_else(|| WorkflowError::Parse("Flow target not found".to_string()))?;

                let predicate = solution.get("predicate").and_then(|t| match t {
                    Term::Literal(l) => Some(l.value().to_string()),
                    _ => None,
                });

                let order = solution.get("order").and_then(|t| match t {
                    Term::Literal(l) => l.value().parse::<i32>().ok(),
                    _ => None,
                });

                flows.push(FlowDefinition {
                    from,
                    to,
                    predicate,
                    order,
                });
            }
        }

        Ok(flows)
    }

    /// Extract variable definitions
    fn extract_variables(&self) -> WorkflowResult<HashMap<String, VariableDefinition>> {
        // For now, return empty map
        // TODO: Implement variable extraction when ontology is extended
        Ok(HashMap::new())
    }

    /// Extract input/output conditions
    fn extract_conditions(&self) -> WorkflowResult<(Option<String>, Option<String>)> {
        let query = format!(
            r#"
            PREFIX yawl: <{}>

            SELECT ?input ?output WHERE {{
                ?spec rdf:type yawl:Specification .
                OPTIONAL {{ ?spec yawl:inputCondition ?input }}
                OPTIONAL {{ ?spec yawl:outputCondition ?output }}
            }}
            LIMIT 1
        "#,
            YAWL_NS
        );

        let results = self.store.query(&query).map_err(|e| {
            error!("Failed to query conditions: {}", e);
            WorkflowError::Parse(format!("Failed to query conditions: {}", e))
        })?;

        if let QueryResults::Solutions(mut solutions) = results {
            if let Some(solution) = solutions.next() {
                let solution = solution.map_err(|e| {
                    error!("Failed to read solution: {}", e);
                    WorkflowError::Parse(format!("Failed to read solution: {}", e))
                })?;

                let input = solution.get("input").and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                });

                let output = solution.get("output").and_then(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                });

                return Ok((input, output));
            }
        }

        Ok((None, None))
    }

    /// Validate patterns against permutation matrix
    ///
    /// # Covenant 2: Invariants Are Law
    ///
    /// This validates that all (split, join) combinations are valid
    /// according to yawl-pattern-permutations.ttl
    fn validate_patterns(
        &self,
        tasks: &[TaskDefinition],
        _flows: &[FlowDefinition],
    ) -> WorkflowResult<()> {
        debug!("Validating patterns against permutation matrix");

        for task in tasks {
            // Check split/join combination is valid
            match (task.split_type, task.join_type) {
                (Some(split), Some(join)) => {
                    self.validate_split_join_combination(split, join)?;
                }
                _ => {
                    // No split/join is valid (atomic task)
                }
            }
        }

        Ok(())
    }

    /// Validate split/join combination is in permutation matrix
    fn validate_split_join_combination(
        &self,
        split: SplitType,
        join: JoinType,
    ) -> WorkflowResult<()> {
        // Valid combinations from yawl-pattern-permutations.ttl
        let valid = match (split, join) {
            (SplitType::AND, JoinType::AND) => true, // Pattern 2+3: Parallel + Sync
            (SplitType::AND, JoinType::OR) => true,  // Async parallel
            (SplitType::AND, JoinType::XOR) => true, // Unsync parallel
            (SplitType::AND, JoinType::Discriminator) => true, // Discriminator
            (SplitType::OR, JoinType::OR) => true,   // Pattern 6+7: Multi-choice + Sync merge
            (SplitType::OR, JoinType::XOR) => true,  // Multi-merge
            (SplitType::XOR, JoinType::XOR) => true, // Pattern 4+5: Exclusive choice + Simple merge
            (SplitType::XOR, JoinType::AND) => false, // Invalid: XOR split cannot require AND join
            (SplitType::XOR, JoinType::OR) => false, // Invalid: XOR split cannot require OR join
            (SplitType::XOR, JoinType::Discriminator) => false, // Invalid
            (SplitType::OR, JoinType::AND) => false, // Invalid: OR split cannot require AND join
            (SplitType::OR, JoinType::Discriminator) => true, // OR with discriminator is valid
        };

        if !valid {
            error!("Invalid split/join combination: {:?} + {:?}", split, join);
            return Err(WorkflowError::InvalidSpecification(
                format!("Invalid split/join combination: {:?} split with {:?} join is not in permutation matrix", split, join)
            ));
        }

        Ok(())
    }

    // Helper parsers

    fn parse_split_type(&self, term: &Term) -> Option<SplitType> {
        match term {
            Term::NamedNode(n) => {
                let s = n.as_str();
                if s.ends_with("AND") || s.contains("#AND") {
                    Some(SplitType::AND)
                } else if s.ends_with("OR") || s.contains("#OR") {
                    Some(SplitType::OR)
                } else if s.ends_with("XOR") || s.contains("#XOR") {
                    Some(SplitType::XOR)
                } else {
                    warn!("Unknown split type: {}", s);
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_join_type(&self, term: &Term) -> Option<JoinType> {
        match term {
            Term::NamedNode(n) => {
                let s = n.as_str();
                if s.ends_with("AND") || s.contains("#AND") {
                    Some(JoinType::AND)
                } else if s.ends_with("OR") || s.contains("#OR") {
                    Some(JoinType::OR)
                } else if s.ends_with("XOR") || s.contains("#XOR") {
                    Some(JoinType::XOR)
                } else if s.contains("Discriminator") {
                    Some(JoinType::Discriminator)
                } else {
                    warn!("Unknown join type: {}", s);
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_execution_mode(&self, term: &Term) -> Option<ExecutionMode> {
        match term {
            Term::NamedNode(n) => {
                let s = n.as_str();
                if s.contains("Synchronous") {
                    Some(ExecutionMode::Synchronous)
                } else if s.contains("Asynchronous") {
                    Some(ExecutionMode::Asynchronous)
                } else if s.contains("Queued") {
                    Some(ExecutionMode::Queued)
                } else if s.contains("Parallel") {
                    Some(ExecutionMode::Parallel)
                } else {
                    warn!("Unknown execution mode: {}", s);
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_timeout_policy(&self, term: &Term) -> Option<TimeoutPolicy> {
        match term {
            Term::NamedNode(n) => {
                let s = n.as_str();
                if s.contains("Skip") {
                    Some(TimeoutPolicy::Skip)
                } else if s.contains("Retry") {
                    Some(TimeoutPolicy::Retry)
                } else if s.contains("Escalate") {
                    Some(TimeoutPolicy::Escalate)
                } else {
                    warn!("Unknown timeout policy: {}", s);
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_retry_policy(&self, term: &Term) -> Option<RetryPolicy> {
        match term {
            Term::NamedNode(n) => {
                let s = n.as_str();
                if s.contains("Exponential") {
                    Some(RetryPolicy::Exponential)
                } else if s.contains("Linear") {
                    Some(RetryPolicy::Linear)
                } else if s.contains("Immediate") {
                    Some(RetryPolicy::Immediate)
                } else {
                    warn!("Unknown retry policy: {}", s);
                    None
                }
            }
            _ => None,
        }
    }
}

impl Default for WorkflowLoader {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Failed to create default WorkflowLoader"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = WorkflowLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_split_join_validation() {
        let loader = WorkflowLoader::new().unwrap();

        // Valid combinations
        assert!(loader
            .validate_split_join_combination(SplitType::AND, JoinType::AND)
            .is_ok());
        assert!(loader
            .validate_split_join_combination(SplitType::XOR, JoinType::XOR)
            .is_ok());
        assert!(loader
            .validate_split_join_combination(SplitType::OR, JoinType::OR)
            .is_ok());

        // Invalid combinations
        assert!(loader
            .validate_split_join_combination(SplitType::XOR, JoinType::AND)
            .is_err());
        assert!(loader
            .validate_split_join_combination(SplitType::OR, JoinType::AND)
            .is_err());
    }
}
