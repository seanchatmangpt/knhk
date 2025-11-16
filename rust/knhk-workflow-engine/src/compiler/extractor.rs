//! Pattern Extractor via SPARQL
//!
//! Extracts all workflow patterns, guards, variables, and constraints
//! from RDF store using SPARQL queries.

use crate::error::{WorkflowError, WorkflowResult};
use oxigraph::{
    model::{Term, Literal},
    sparql::{QueryResults, QuerySolution},
    store::Store,
};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, instrument, warn};

/// Extracted pattern
#[derive(Debug, Clone)]
pub struct ExtractedPattern {
    /// Pattern ID (1-43)
    pub pattern_id: u8,
    /// Pattern IRI
    pub iri: String,
    /// Pattern type
    pub pattern_type: PatternType,
    /// Guards
    pub guards: Vec<Guard>,
    /// Variables
    pub variables: Vec<Variable>,
    /// Timeout (milliseconds)
    pub timeout_ms: Option<u64>,
    /// Tick budget (max ticks)
    pub tick_budget: u8,
    /// Constraints
    pub constraints: Vec<Constraint>,
    /// Data flows
    pub data_flows: Vec<DataFlow>,
    /// Event handlers
    pub event_handlers: Vec<EventHandler>,
}

/// Pattern type
#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    Sequence,
    ParallelSplit,
    Synchronization,
    ExclusiveChoice,
    SimpleMerge,
    MultiChoice,
    SynchronizingMerge,
    MultiMerge,
    Discriminator,
    ArbitraryLoop,
    StructuredLoop,
    ImplicitTermination,
    MultipleInstance,
    CriticalSection,
    InterleavedRouting,
    Milestone,
    Cancel,
    CompensateTask,
    Custom(String),
}

/// Guard predicate
#[derive(Debug, Clone)]
pub struct Guard {
    /// Guard ID
    pub id: String,
    /// Boolean expression
    pub expression: String,
    /// Variables referenced
    pub variables: Vec<String>,
    /// Guard type
    pub guard_type: GuardType,
}

/// Guard type
#[derive(Debug, Clone)]
pub enum GuardType {
    PreCondition,
    PostCondition,
    Invariant,
    ExceptionHandler,
}

/// Workflow variable
#[derive(Debug, Clone)]
pub struct Variable {
    /// Variable name
    pub name: String,
    /// Data type
    pub data_type: DataType,
    /// Initial value
    pub initial_value: Option<String>,
    /// Constraints
    pub constraints: Vec<String>,
}

/// Data type
#[derive(Debug, Clone)]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Duration,
    Object(String),
}

/// Constraint
#[derive(Debug, Clone)]
pub struct Constraint {
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Expression
    pub expression: String,
    /// Severity
    pub severity: ConstraintSeverity,
}

/// Constraint type
#[derive(Debug, Clone)]
pub enum ConstraintType {
    Temporal,
    Resource,
    Data,
    Control,
}

/// Constraint severity
#[derive(Debug, Clone)]
pub enum ConstraintSeverity {
    Must,
    Should,
    May,
}

/// Data flow
#[derive(Debug, Clone)]
pub struct DataFlow {
    /// Source variable
    pub source: String,
    /// Target variable
    pub target: String,
    /// Transformation
    pub transformation: Option<String>,
}

/// Event handler
#[derive(Debug, Clone)]
pub struct EventHandler {
    /// Event type
    pub event_type: String,
    /// Handler expression
    pub handler: String,
    /// Priority
    pub priority: u8,
}

/// Pattern extractor
pub struct PatternExtractor {
    parallel: bool,
    namespaces: HashMap<String, String>,
}

impl PatternExtractor {
    /// Create new extractor
    pub fn new(parallel: bool) -> Self {
        let mut namespaces = HashMap::new();
        namespaces.insert("yawl".to_string(), "http://bitflow.ai/ontology/yawl/v2#".to_string());
        namespaces.insert("rdf".to_string(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string());
        namespaces.insert("rdfs".to_string(), "http://www.w3.org/2000/01/rdf-schema#".to_string());
        namespaces.insert("time".to_string(), "http://www.w3.org/2006/time#".to_string());

        Self {
            parallel,
            namespaces,
        }
    }

    /// Extract all patterns from store
    #[instrument(skip(self, store))]
    pub async fn extract_all(&self, store: &Store) -> WorkflowResult<Vec<ExtractedPattern>> {
        info!("Extracting patterns from RDF store");

        // Extract tasks first
        let tasks = self.extract_tasks(store).await?;
        info!("Extracted {} tasks", tasks.len());

        // Extract patterns from tasks
        let patterns = if self.parallel {
            self.extract_patterns_parallel(store, tasks).await?
        } else {
            self.extract_patterns_sequential(store, tasks).await?
        };

        info!("Extracted {} patterns", patterns.len());
        Ok(patterns)
    }

    /// Extract tasks from store
    async fn extract_tasks(&self, store: &Store) -> WorkflowResult<Vec<TaskInfo>> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <{}>\n\
             PREFIX rdfs: <{}>\n\
             SELECT ?task ?name ?type ?split ?join WHERE {{\n\
               ?task rdf:type yawl:Task .\n\
               OPTIONAL {{ ?task rdfs:label ?name }}\n\
               OPTIONAL {{ ?task yawl:taskType ?type }}\n\
               OPTIONAL {{ ?task yawl:splitType ?split }}\n\
               OPTIONAL {{ ?task yawl:joinType ?join }}\n\
             }}",
            self.namespaces["yawl"], self.namespaces["rdf"], self.namespaces["rdfs"]
        );

        let results = store
            .query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Task extraction failed: {}", e)))?;

        let mut tasks = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| WorkflowError::Parse(format!("Solution error: {}", e)))?;

                let task = self.parse_task_info(&solution)?;
                tasks.push(task);
            }
        }

        Ok(tasks)
    }

    /// Parse task info from solution
    fn parse_task_info(&self, solution: &QuerySolution) -> WorkflowResult<TaskInfo> {
        let iri = solution.get("task")
            .map(|t| t.to_string())
            .ok_or_else(|| WorkflowError::Parse("Missing task IRI".to_string()))?;

        let name = solution.get("name")
            .and_then(|t| if let Term::Literal(lit) = t {
                Some(lit.value().to_string())
            } else { None })
            .unwrap_or_else(|| iri.clone());

        let task_type = solution.get("type")
            .and_then(|t| if let Term::Literal(lit) = t {
                Some(lit.value().to_string())
            } else { None });

        let split_type = solution.get("split")
            .and_then(|t| if let Term::Literal(lit) = t {
                Some(lit.value().to_string())
            } else { None });

        let join_type = solution.get("join")
            .and_then(|t| if let Term::Literal(lit) = t {
                Some(lit.value().to_string())
            } else { None });

        Ok(TaskInfo {
            iri,
            name,
            task_type,
            split_type,
            join_type,
        })
    }

    /// Extract patterns sequentially
    async fn extract_patterns_sequential(
        &self,
        store: &Store,
        tasks: Vec<TaskInfo>,
    ) -> WorkflowResult<Vec<ExtractedPattern>> {
        let mut patterns = Vec::new();

        for task in tasks {
            let pattern = self.extract_pattern_for_task(store, &task).await?;
            patterns.push(pattern);
        }

        Ok(patterns)
    }

    /// Extract patterns in parallel
    async fn extract_patterns_parallel(
        &self,
        store: &Store,
        tasks: Vec<TaskInfo>,
    ) -> WorkflowResult<Vec<ExtractedPattern>> {
        // Since we can't use async with rayon directly, we'll collect queries first
        let patterns: Result<Vec<_>, _> = tasks
            .into_par_iter()
            .map(|task| {
                // Extract pattern synchronously (store operations are sync)
                self.extract_pattern_for_task_sync(store, &task)
            })
            .collect();

        patterns
    }

    /// Extract pattern for a single task (sync version for parallel)
    fn extract_pattern_for_task_sync(
        &self,
        store: &Store,
        task: &TaskInfo,
    ) -> WorkflowResult<ExtractedPattern> {
        let pattern_type = self.determine_pattern_type(task);
        let pattern_id = self.map_pattern_to_id(&pattern_type);

        // Extract guards
        let guards = self.extract_guards_sync(store, &task.iri)?;

        // Extract variables
        let variables = self.extract_variables_sync(store, &task.iri)?;

        // Extract constraints
        let constraints = self.extract_constraints_sync(store, &task.iri)?;

        // Extract data flows
        let data_flows = self.extract_data_flows_sync(store, &task.iri)?;

        // Extract event handlers
        let event_handlers = self.extract_event_handlers_sync(store, &task.iri)?;

        // Extract timing
        let timeout_ms = self.extract_timeout_sync(store, &task.iri)?;

        // Calculate tick budget based on pattern complexity
        let tick_budget = self.calculate_tick_budget(&pattern_type, guards.len());

        Ok(ExtractedPattern {
            pattern_id,
            iri: task.iri.clone(),
            pattern_type,
            guards,
            variables,
            timeout_ms,
            tick_budget,
            constraints,
            data_flows,
            event_handlers,
        })
    }

    /// Extract pattern for a single task (async)
    async fn extract_pattern_for_task(
        &self,
        store: &Store,
        task: &TaskInfo,
    ) -> WorkflowResult<ExtractedPattern> {
        self.extract_pattern_for_task_sync(store, task)
    }

    /// Determine pattern type from task info
    fn determine_pattern_type(&self, task: &TaskInfo) -> PatternType {
        match (&task.split_type, &task.join_type) {
            (Some(split), Some(join)) => {
                match (split.as_str(), join.as_str()) {
                    ("AND", "AND") => PatternType::ParallelSplit,
                    ("XOR", "XOR") => PatternType::ExclusiveChoice,
                    ("OR", "OR") => PatternType::MultiChoice,
                    ("AND", "XOR") => PatternType::Discriminator,
                    ("XOR", "AND") => PatternType::Synchronization,
                    ("OR", "AND") => PatternType::SynchronizingMerge,
                    _ => PatternType::Sequence,
                }
            }
            _ => {
                // Check task type for special patterns
                if let Some(ref task_type) = task.task_type {
                    match task_type.as_str() {
                        "MultipleInstance" => PatternType::MultipleInstance,
                        "Loop" => PatternType::ArbitraryLoop,
                        "Cancel" => PatternType::Cancel,
                        "Compensate" => PatternType::CompensateTask,
                        _ => PatternType::Sequence,
                    }
                } else {
                    PatternType::Sequence
                }
            }
        }
    }

    /// Map pattern type to ID (1-43)
    fn map_pattern_to_id(&self, pattern: &PatternType) -> u8 {
        match pattern {
            PatternType::Sequence => 1,
            PatternType::ParallelSplit => 2,
            PatternType::Synchronization => 3,
            PatternType::ExclusiveChoice => 4,
            PatternType::SimpleMerge => 5,
            PatternType::MultiChoice => 6,
            PatternType::SynchronizingMerge => 7,
            PatternType::MultiMerge => 8,
            PatternType::Discriminator => 9,
            PatternType::ArbitraryLoop => 10,
            PatternType::StructuredLoop => 11,
            PatternType::ImplicitTermination => 12,
            PatternType::MultipleInstance => 13,
            PatternType::CriticalSection => 14,
            PatternType::InterleavedRouting => 15,
            PatternType::Milestone => 16,
            PatternType::Cancel => 17,
            PatternType::CompensateTask => 18,
            PatternType::Custom(_) => 43, // Custom patterns
        }
    }

    /// Extract guards for task
    fn extract_guards_sync(&self, store: &Store, task_iri: &str) -> WorkflowResult<Vec<Guard>> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             SELECT ?guard ?expr ?type WHERE {{\n\
               <{}> yawl:hasGuard ?guard .\n\
               ?guard yawl:expression ?expr .\n\
               OPTIONAL {{ ?guard yawl:guardType ?type }}\n\
             }}",
            self.namespaces["yawl"], task_iri
        );

        let results = store.query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Guard query failed: {}", e)))?;

        let mut guards = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| WorkflowError::Parse(format!("Solution error: {}", e)))?;

                let guard = self.parse_guard(&solution)?;
                guards.push(guard);
            }
        }

        Ok(guards)
    }

    /// Parse guard from solution
    fn parse_guard(&self, solution: &QuerySolution) -> WorkflowResult<Guard> {
        let id = solution.get("guard")
            .map(|t| t.to_string())
            .unwrap_or_else(|| "guard".to_string());

        let expression = solution.get("expr")
            .and_then(|t| if let Term::Literal(lit) = t {
                Some(lit.value().to_string())
            } else { None })
            .unwrap_or_else(|| "true".to_string());

        let guard_type = solution.get("type")
            .and_then(|t| if let Term::Literal(lit) = t {
                match lit.value() {
                    "precondition" => Some(GuardType::PreCondition),
                    "postcondition" => Some(GuardType::PostCondition),
                    "invariant" => Some(GuardType::Invariant),
                    "exception" => Some(GuardType::ExceptionHandler),
                    _ => None,
                }
            } else { None })
            .unwrap_or(GuardType::PreCondition);

        // Extract variables from expression
        let variables = self.extract_variables_from_expression(&expression);

        Ok(Guard {
            id,
            expression,
            variables,
            guard_type,
        })
    }

    /// Extract variables from boolean expression
    fn extract_variables_from_expression(&self, expr: &str) -> Vec<String> {
        // Simple regex-based extraction
        let mut vars = HashSet::new();

        // Match variable patterns like $var, :var, or simple identifiers
        let patterns = vec![
            r"\$([a-zA-Z_]\w*)",
            r":([a-zA-Z_]\w*)",
            r"\b([a-zA-Z_]\w*)\b",
        ];

        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.captures_iter(expr) {
                    if let Some(var) = cap.get(1) {
                        vars.insert(var.as_str().to_string());
                    }
                }
            }
        }

        vars.into_iter().collect()
    }

    /// Extract variables for task
    fn extract_variables_sync(&self, store: &Store, task_iri: &str) -> WorkflowResult<Vec<Variable>> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             SELECT ?var ?name ?type ?init WHERE {{\n\
               <{}> yawl:hasVariable ?var .\n\
               ?var yawl:variableName ?name .\n\
               OPTIONAL {{ ?var yawl:dataType ?type }}\n\
               OPTIONAL {{ ?var yawl:initialValue ?init }}\n\
             }}",
            self.namespaces["yawl"], task_iri
        );

        let results = store.query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Variable query failed: {}", e)))?;

        let mut variables = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| WorkflowError::Parse(format!("Solution error: {}", e)))?;

                let variable = self.parse_variable(&solution)?;
                variables.push(variable);
            }
        }

        Ok(variables)
    }

    /// Parse variable from solution
    fn parse_variable(&self, solution: &QuerySolution) -> WorkflowResult<Variable> {
        let name = solution.get("name")
            .and_then(|t| if let Term::Literal(lit) = t {
                Some(lit.value().to_string())
            } else { None })
            .unwrap_or_else(|| "var".to_string());

        let data_type = solution.get("type")
            .and_then(|t| if let Term::Literal(lit) = t {
                match lit.value() {
                    "string" => Some(DataType::String),
                    "integer" => Some(DataType::Integer),
                    "float" => Some(DataType::Float),
                    "boolean" => Some(DataType::Boolean),
                    "datetime" => Some(DataType::DateTime),
                    "duration" => Some(DataType::Duration),
                    other => Some(DataType::Object(other.to_string())),
                }
            } else { None })
            .unwrap_or(DataType::String);

        let initial_value = solution.get("init")
            .and_then(|t| if let Term::Literal(lit) = t {
                Some(lit.value().to_string())
            } else { None });

        Ok(Variable {
            name,
            data_type,
            initial_value,
            constraints: Vec::new(),
        })
    }

    /// Extract constraints
    fn extract_constraints_sync(&self, store: &Store, task_iri: &str) -> WorkflowResult<Vec<Constraint>> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             SELECT ?constraint ?type ?expr ?severity WHERE {{\n\
               <{}> yawl:hasConstraint ?constraint .\n\
               ?constraint yawl:constraintType ?type .\n\
               ?constraint yawl:expression ?expr .\n\
               OPTIONAL {{ ?constraint yawl:severity ?severity }}\n\
             }}",
            self.namespaces["yawl"], task_iri
        );

        let results = store.query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Constraint query failed: {}", e)))?;

        let mut constraints = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| WorkflowError::Parse(format!("Solution error: {}", e)))?;

                let constraint = self.parse_constraint(&solution)?;
                constraints.push(constraint);
            }
        }

        Ok(constraints)
    }

    /// Parse constraint
    fn parse_constraint(&self, solution: &QuerySolution) -> WorkflowResult<Constraint> {
        let constraint_type = solution.get("type")
            .and_then(|t| if let Term::Literal(lit) = t {
                match lit.value() {
                    "temporal" => Some(ConstraintType::Temporal),
                    "resource" => Some(ConstraintType::Resource),
                    "data" => Some(ConstraintType::Data),
                    "control" => Some(ConstraintType::Control),
                    _ => None,
                }
            } else { None })
            .unwrap_or(ConstraintType::Data);

        let expression = solution.get("expr")
            .and_then(|t| if let Term::Literal(lit) = t {
                Some(lit.value().to_string())
            } else { None })
            .unwrap_or_default();

        let severity = solution.get("severity")
            .and_then(|t| if let Term::Literal(lit) = t {
                match lit.value() {
                    "must" => Some(ConstraintSeverity::Must),
                    "should" => Some(ConstraintSeverity::Should),
                    "may" => Some(ConstraintSeverity::May),
                    _ => None,
                }
            } else { None })
            .unwrap_or(ConstraintSeverity::Must);

        Ok(Constraint {
            constraint_type,
            expression,
            severity,
        })
    }

    /// Extract data flows
    fn extract_data_flows_sync(&self, store: &Store, task_iri: &str) -> WorkflowResult<Vec<DataFlow>> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             SELECT ?flow ?source ?target ?transform WHERE {{\n\
               <{}> yawl:hasDataFlow ?flow .\n\
               ?flow yawl:fromVariable ?source .\n\
               ?flow yawl:toVariable ?target .\n\
               OPTIONAL {{ ?flow yawl:transformation ?transform }}\n\
             }}",
            self.namespaces["yawl"], task_iri
        );

        let results = store.query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Data flow query failed: {}", e)))?;

        let mut flows = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| WorkflowError::Parse(format!("Solution error: {}", e)))?;

                let flow = DataFlow {
                    source: solution.get("source")
                        .map(|t| t.to_string())
                        .unwrap_or_default(),
                    target: solution.get("target")
                        .map(|t| t.to_string())
                        .unwrap_or_default(),
                    transformation: solution.get("transform")
                        .and_then(|t| if let Term::Literal(lit) = t {
                            Some(lit.value().to_string())
                        } else { None }),
                };
                flows.push(flow);
            }
        }

        Ok(flows)
    }

    /// Extract event handlers
    fn extract_event_handlers_sync(&self, store: &Store, task_iri: &str) -> WorkflowResult<Vec<EventHandler>> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             SELECT ?handler ?event ?expr ?priority WHERE {{\n\
               <{}> yawl:hasEventHandler ?handler .\n\
               ?handler yawl:eventType ?event .\n\
               ?handler yawl:handlerExpression ?expr .\n\
               OPTIONAL {{ ?handler yawl:priority ?priority }}\n\
             }}",
            self.namespaces["yawl"], task_iri
        );

        let results = store.query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Event handler query failed: {}", e)))?;

        let mut handlers = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution
                    .map_err(|e| WorkflowError::Parse(format!("Solution error: {}", e)))?;

                let handler = EventHandler {
                    event_type: solution.get("event")
                        .and_then(|t| if let Term::Literal(lit) = t {
                            Some(lit.value().to_string())
                        } else { None })
                        .unwrap_or_default(),
                    handler: solution.get("expr")
                        .and_then(|t| if let Term::Literal(lit) = t {
                            Some(lit.value().to_string())
                        } else { None })
                        .unwrap_or_default(),
                    priority: solution.get("priority")
                        .and_then(|t| if let Term::Literal(lit) = t {
                            lit.value().parse().ok()
                        } else { None })
                        .unwrap_or(0),
                };
                handlers.push(handler);
            }
        }

        Ok(handlers)
    }

    /// Extract timeout
    fn extract_timeout_sync(&self, store: &Store, task_iri: &str) -> WorkflowResult<Option<u64>> {
        let query = format!(
            "PREFIX yawl: <{}>\n\
             PREFIX time: <{}>\n\
             SELECT ?timeout WHERE {{\n\
               <{}> yawl:hasTimeout ?timeout .\n\
             }}",
            self.namespaces["yawl"], self.namespaces["time"], task_iri
        );

        let results = store.query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Timeout query failed: {}", e)))?;

        if let QueryResults::Solutions(mut solutions) = results {
            if let Some(Ok(solution)) = solutions.next() {
                if let Some(Term::Literal(lit)) = solution.get("timeout") {
                    return Ok(lit.value().parse().ok());
                }
            }
        }

        Ok(None)
    }

    /// Calculate tick budget based on pattern complexity
    fn calculate_tick_budget(&self, pattern: &PatternType, guard_count: usize) -> u8 {
        let base_ticks = match pattern {
            PatternType::Sequence => 2,
            PatternType::ParallelSplit => 3,
            PatternType::Synchronization => 3,
            PatternType::ExclusiveChoice => 2,
            PatternType::SimpleMerge => 2,
            PatternType::MultiChoice => 4,
            PatternType::SynchronizingMerge => 4,
            PatternType::MultiMerge => 5,
            PatternType::Discriminator => 3,
            PatternType::ArbitraryLoop => 5,
            PatternType::StructuredLoop => 4,
            PatternType::ImplicitTermination => 2,
            PatternType::MultipleInstance => 6,
            PatternType::CriticalSection => 4,
            PatternType::InterleavedRouting => 5,
            PatternType::Milestone => 3,
            PatternType::Cancel => 2,
            PatternType::CompensateTask => 3,
            PatternType::Custom(_) => 8,
        };

        // Add ticks for guards
        let guard_ticks = (guard_count as u8).min(2);

        // Ensure within Chatman constant (8 ticks)
        (base_ticks + guard_ticks).min(8)
    }
}

/// Task information
#[derive(Debug, Clone)]
struct TaskInfo {
    iri: String,
    name: String,
    task_type: Option<String>,
    split_type: Option<String>,
    join_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxigraph::io::{RdfFormat, RdfParser};

    #[tokio::test]
    async fn test_extractor_creation() {
        let extractor = PatternExtractor::new(true);
        assert!(extractor.parallel);
    }

    #[tokio::test]
    async fn test_pattern_type_mapping() {
        let extractor = PatternExtractor::new(false);

        assert_eq!(extractor.map_pattern_to_id(&PatternType::Sequence), 1);
        assert_eq!(extractor.map_pattern_to_id(&PatternType::ParallelSplit), 2);
        assert_eq!(extractor.map_pattern_to_id(&PatternType::MultipleInstance), 13);
    }

    #[tokio::test]
    async fn test_tick_budget_calculation() {
        let extractor = PatternExtractor::new(false);

        let budget = extractor.calculate_tick_budget(&PatternType::Sequence, 0);
        assert!(budget <= 8, "Must respect Chatman constant");

        let budget = extractor.calculate_tick_budget(&PatternType::MultipleInstance, 5);
        assert_eq!(budget, 8, "Should cap at 8 ticks");
    }
}