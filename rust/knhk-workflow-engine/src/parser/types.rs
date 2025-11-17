//! Workflow specification types
//!
//! Core types for workflow specifications including tasks, conditions, and specifications.

use uuid::Uuid;

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::PatternId;

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

/// Join type (AND, XOR, OR, Discriminator)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JoinType {
    /// AND-join: wait for all branches
    And,
    /// XOR-join: wait for one branch
    Xor,
    /// OR-join: wait for all active branches
    Or,
    /// Discriminator join: wait for first N branches (Pattern 9)
    Discriminator {
        /// Quorum: number of branches needed
        quorum: usize,
    },
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

/// Task parameter (input or output)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (e.g., "string", "boolean", "decimal")
    pub param_type: String,
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
    /// Input parameters (from yawl:hasInputParameter)
    pub input_parameters: Vec<TaskParameter>,
    /// Output parameters (from yawl:hasOutputParameter)
    pub output_parameters: Vec<TaskParameter>,
    /// Resource allocation policy
    pub allocation_policy: Option<crate::resource::AllocationPolicy>,
    /// Required roles for task execution
    pub required_roles: Vec<String>,
    /// Required capabilities for task execution
    pub required_capabilities: Vec<String>,
    /// Worklet ID for exception handling (optional)
    pub exception_worklet: Option<crate::worklets::WorkletId>,
    /// Pre-compiled pattern ID (TRIZ Principle 10: Prior Action)
    ///
    /// Pattern identification is computed at registration time to avoid
    /// runtime overhead. This enables ≤8 tick hot path execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_id: Option<PatternId>,
}

/// Flow with optional predicate
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Flow {
    /// Flow identifier (IRI)
    pub id: String,
    /// Source node (task or condition ID)
    pub from: String,
    /// Target node (task or condition ID)
    pub to: String,
    /// Optional predicate (e.g., "approved == true")
    pub predicate: Option<String>,
}

/// Workflow condition
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
    /// Specification ID
    pub id: WorkflowSpecId,
    /// Specification name
    pub name: String,
    /// Tasks in the workflow
    pub tasks: std::collections::HashMap<String, Task>,
    /// Conditions in the workflow
    pub conditions: std::collections::HashMap<String, Condition>,
    /// Flows with predicates
    pub flows: Vec<Flow>,
    /// Start condition ID
    pub start_condition: Option<String>,
    /// End condition ID
    pub end_condition: Option<String>,
    /// Source RDF/Turtle (for runtime RDF queries)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_turtle: Option<String>,
}

impl WorkflowSpec {
    /// Serialize workflow to Turtle format with YAWL ontology annotations
    ///
    /// Converts the WorkflowSpec to RDF/Turtle format using the YAWL ontology.
    /// This is the inverse operation of `WorkflowParser::parse_turtle()`.
    ///
    /// # Returns
    /// A Turtle string representing the workflow with full YAWL semantics.
    ///
    /// # Example
    /// ```ignore
    /// let spec = WorkflowSpec { /* ... */ };
    /// let turtle = spec.to_turtle()?;
    /// // Can now save to file, transmit, or re-parse
    /// ```
    pub fn to_turtle(&self) -> WorkflowResult<String> {
        use std::fmt::Write;
        let mut output = String::new();

        // Write Turtle prefixes
        writeln!(
            &mut output,
            "@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> ."
        )?;
        writeln!(
            &mut output,
            "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> ."
        )?;
        writeln!(
            &mut output,
            "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> ."
        )?;
        writeln!(
            &mut output,
            "@prefix workflow: <http://knhk.org/ns/workflow#> ."
        )?;
        writeln!(&mut output)?;

        // Generate base IRI from workflow name (sanitize for IRI)
        let base_iri = self.name.to_lowercase().replace(' ', "-");
        let workflow_iri = format!("<http://knhk.org/workflows/{}>", base_iri);

        // Write workflow specification
        writeln!(&mut output, "{}", workflow_iri)?;
        writeln!(&mut output, "    a yawl:WorkflowSpecification ;")?;
        writeln!(&mut output, "    rdfs:label \"{}\" ;", self.name)?;

        // Write task references
        for task_id in self.tasks.keys() {
            writeln!(&mut output, "    yawl:hasTask <{}> ;", task_id)?;
        }

        // Write condition references
        for condition_id in self.conditions.keys() {
            writeln!(&mut output, "    yawl:hasCondition <{}> ;", condition_id)?;
        }

        // Remove trailing semicolon and add period
        output = output.trim_end().trim_end_matches(';').to_string();
        writeln!(&mut output, " .")?;
        writeln!(&mut output)?;

        // Write conditions
        if let Some(start_id) = &self.start_condition {
            if let Some(condition) = self.conditions.get(start_id) {
                writeln!(&mut output, "<{}>", condition.id)?;
                writeln!(&mut output, "    a yawl:StartCondition ;")?;
                writeln!(&mut output, "    rdfs:label \"{}\" .", condition.name)?;
                writeln!(&mut output)?;
            }
        }

        if let Some(end_id) = &self.end_condition {
            if let Some(condition) = self.conditions.get(end_id) {
                writeln!(&mut output, "<{}>", condition.id)?;
                writeln!(&mut output, "    a yawl:EndCondition ;")?;
                writeln!(&mut output, "    rdfs:label \"{}\" .", condition.name)?;
                writeln!(&mut output)?;
            }
        }

        // Write other conditions (not start/end)
        for (id, condition) in &self.conditions {
            if Some(id) != self.start_condition.as_ref() && Some(id) != self.end_condition.as_ref()
            {
                writeln!(&mut output, "<{}>", condition.id)?;
                writeln!(&mut output, "    a yawl:Condition ;")?;
                writeln!(&mut output, "    rdfs:label \"{}\" .", condition.name)?;
                writeln!(&mut output)?;
            }
        }

        // Write tasks
        for task in self.tasks.values() {
            writeln!(&mut output, "<{}>", task.id)?;

            // Task type
            let task_type_str = match task.task_type {
                TaskType::Atomic => "yawl:AtomicTask",
                TaskType::Composite => "yawl:CompositeTask",
                TaskType::MultipleInstance => "yawl:MultipleInstanceTask",
            };
            writeln!(&mut output, "    a {} ;", task_type_str)?;
            writeln!(&mut output, "    rdfs:label \"{}\" ;", task.name)?;

            // Split type
            let split_type_str = match task.split_type {
                SplitType::And => "yawl:And",
                SplitType::Xor => "yawl:Xor",
                SplitType::Or => "yawl:Or",
            };
            writeln!(&mut output, "    yawl:splitType {} ;", split_type_str)?;

            // Join type
            let join_type_str = match task.join_type {
                JoinType::And => "yawl:And",
                JoinType::Xor => "yawl:Xor",
                JoinType::Or => "yawl:Or",
                JoinType::Discriminator { .. } => "yawl:Discriminator",
            };
            writeln!(&mut output, "    yawl:joinType {} ;", join_type_str)?;

            // Performance annotations (hot path optimizations)
            if let Some(max_ticks) = task.max_ticks {
                writeln!(&mut output, "    yawl:maxTicks {} ;", max_ticks)?;
            }
            if let Some(priority) = task.priority {
                writeln!(&mut output, "    yawl:priority {} ;", priority)?;
            }
            if task.use_simd {
                writeln!(&mut output, "    yawl:useSimd true ;")?;
            }

            // Remove trailing semicolon and add period
            output = output.trim_end().trim_end_matches(';').to_string();
            writeln!(&mut output, " .")?;
            writeln!(&mut output)?;
        }

        // Write flows
        for flow in &self.flows {
            writeln!(&mut output, "<{}>", flow.id)?;
            writeln!(&mut output, "    a yawl:Flow ;")?;
            writeln!(&mut output, "    yawl:from <{}> ;", flow.from)?;
            writeln!(&mut output, "    yawl:to <{}> ;", flow.to)?;

            if let Some(ref predicate) = flow.predicate {
                writeln!(&mut output, "    yawl:predicate \"{}\" ;", predicate)?;
            }

            // Remove trailing semicolon and add period
            output = output.trim_end().trim_end_matches(';').to_string();
            writeln!(&mut output, " .")?;
            writeln!(&mut output)?;
        }

        Ok(output)
    }
}
