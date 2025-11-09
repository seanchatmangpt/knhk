//! RDF/Turtle Parser for All 43 Workflow Patterns
//!
//! Provides comprehensive RDF parsing for workflow definitions supporting all 43 Van der Aalst patterns.
//! Supports YAWL-compatible RDF/Turtle format with KNHK extensions.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::{Task, TaskType, WorkflowSpec, WorkflowSpecId};
use crate::patterns::PatternId;
use oxigraph::io::RdfFormat;
use oxigraph::model::{NamedNode, Quad, Term};
use oxigraph::sparql::SparqlEvaluator;
use oxigraph::store::Store;
use std::collections::HashMap;

/// RDF namespaces
pub const YAWL_NS: &str = "http://www.yawlfoundation.org/xsd/yawl_20";
pub const KNHK_PATTERN_NS: &str = "https://knhk.org/ns/workflow/pattern#";
pub const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";
pub const OWL_NS: &str = "http://www.w3.org/2002/07/owl#";

/// RDF parser for workflow specifications
pub struct RdfWorkflowParser {
    store: Store,
}

impl RdfWorkflowParser {
    /// Create new RDF parser
    pub fn new() -> WorkflowResult<Self> {
        let store = Store::new()
            .map_err(|e| WorkflowError::Parse(format!("Failed to create RDF store: {:?}", e)))?;
        Ok(Self { store })
    }

    /// Parse workflow from RDF/Turtle string
    pub fn parse_turtle(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // Load RDF into store
        self.store
            .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| WorkflowError::Parse(format!("Failed to parse Turtle: {:?}", e)))?;

        // Extract workflow specification
        self.extract_workflow_spec()
    }

    /// Extract workflow specification from RDF store
    fn extract_workflow_spec(&self) -> WorkflowResult<WorkflowSpec> {
        // Find workflow specification
        let workflow_type = NamedNode::new(&format!("{}WorkflowSpecification", YAWL_NS))
            .map_err(|e| WorkflowError::Parse(format!("Invalid IRI: {}", e)))?;
        let rdf_type = NamedNode::new(&format!("{}type", RDF_NS))
            .map_err(|e| WorkflowError::Parse(format!("Invalid IRI: {}", e)))?;

        let query = format!(
            "SELECT ?spec WHERE {{ ?spec <{}> <{}> }}",
            rdf_type.as_str(),
            workflow_type.as_str()
        );

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        let query_results = SparqlEvaluator::new()
            .parse_query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Failed to parse SPARQL query: {}", e)))?
            .on_store(&self.store)
            .execute()
            .map_err(|e| WorkflowError::Parse(format!("SPARQL query failed: {}", e)))?;

        // Get first workflow spec
        let spec_id = if let Some(binding) = query_results.iter().next() {
            if let Some(spec_term) = binding.get("spec") {
                if let Term::NamedNode(node) = spec_term {
                    WorkflowSpecId::parse_str(node.as_str())
                        .unwrap_or_else(|_| WorkflowSpecId::new())
                } else {
                    WorkflowSpecId::new()
                }
            } else {
                WorkflowSpecId::new()
            }
        } else {
            WorkflowSpecId::new()
        };

        // Extract workflow name
        let name = self
            .extract_string_property(&spec_id.to_string(), &format!("{}label", RDFS_NS))
            .unwrap_or_else(|| "Parsed Workflow".to_string());

        // Extract tasks
        let tasks = self.extract_tasks(&spec_id.to_string())?;

        // Extract conditions
        let conditions = self.extract_conditions(&spec_id.to_string())?;

        // Extract start and end conditions
        let start_condition = self.extract_start_condition(&spec_id.to_string())?;
        let end_condition = self.extract_end_condition(&spec_id.to_string())?;

        Ok(WorkflowSpec {
            id: spec_id,
            name,
            tasks,
            conditions,
            start_condition,
            end_condition,
        })
    }

    /// Extract tasks from RDF
    fn extract_tasks(&self, workflow_id: &str) -> WorkflowResult<HashMap<String, Task>> {
        let mut tasks = HashMap::new();

        // Query for all tasks
        let query = format!(
            "SELECT ?task ?name ?type WHERE {{
                <{}> <{}hasTask> ?task .
                ?task <{}label> ?name .
                ?task <{}type> ?type .
            }}",
            workflow_id,
            YAWL_NS,
            RDFS_NS,
            YAWL_NS
        );

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        let query_results = SparqlEvaluator::new()
            .parse_query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Failed to parse SPARQL query: {}", e)))?
            .on_store(&self.store)
            .execute()
            .map_err(|e| WorkflowError::Parse(format!("SPARQL query failed: {}", e)))?;

        for binding in query_results {
            if let (Some(task_term), Some(name_term), Some(type_term)) = (
                binding.get("task"),
                binding.get("name"),
                binding.get("type"),
            ) {
                if let (Term::NamedNode(task_node), Term::Literal(name_lit), Term::Literal(type_lit)) =
                    (task_term, name_term, type_term)
                {
                    let task_id = task_node.as_str().to_string();
                    let name = name_lit.value().to_string();
                    let task_type = match type_lit.value() {
                        "Atomic" => TaskType::Atomic,
                        "Composite" => TaskType::Composite,
                        "MultipleInstance" => TaskType::MultipleInstance,
                        _ => TaskType::Atomic,
                    };

                    // Extract task properties
                    let split_type = self.extract_split_type(&task_id)?;
                    let join_type = self.extract_join_type(&task_id)?;
                    let max_ticks = self.extract_max_ticks(&task_id)?;
                    let priority = self.extract_priority(&task_id)?;
                    let use_simd = self.extract_simd_support(&task_id)?;
                    let input_conditions = self.extract_input_conditions(&task_id)?;
                    let output_conditions = self.extract_output_conditions(&task_id)?;
                    let outgoing_flows = self.extract_outgoing_flows(&task_id)?;
                    let incoming_flows = self.extract_incoming_flows(&task_id)?;

                    tasks.insert(
                        task_id.clone(),
                        Task {
                            id: task_id,
                            name,
                            task_type,
                            split_type,
                            join_type,
                            max_ticks,
                            priority,
                            use_simd,
                            input_conditions,
                            output_conditions,
                            outgoing_flows,
                            incoming_flows,
                            input_parameters: Vec::new(),
                            output_parameters: Vec::new(),
                            allocation_policy: None,
                            required_roles: Vec::new(),
                            required_capabilities: Vec::new(),
                            exception_worklet: None,
                        },
                    );
                }
            }
        }

        Ok(tasks)
    }

    /// Extract split type
    fn extract_split_type(&self, task_id: &str) -> WorkflowResult<crate::parser::SplitType> {
        let query = format!(
            "SELECT ?type WHERE {{
                <{}> <{}splitType> ?type .
            }}",
            task_id, YAWL_NS
        );

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        if let Ok(results) = SparqlEvaluator::new()
            .parse_query(&query)
            .and_then(|q| q.on_store(&self.store).execute())
        {
            if let Some(binding) = results.iter().next() {
                if let Some(type_term) = binding.get("type") {
                    if let Term::Literal(lit) = type_term {
                        return Ok(match lit.value() {
                            "AND" => crate::parser::SplitType::And,
                            "XOR" => crate::parser::SplitType::Xor,
                            "OR" => crate::parser::SplitType::Or,
                            _ => crate::parser::SplitType::And,
                        });
                    }
                }
            }
        }

        Ok(crate::parser::SplitType::And) // Default
    }

    /// Extract join type
    fn extract_join_type(&self, task_id: &str) -> WorkflowResult<crate::parser::JoinType> {
        let query = format!(
            "SELECT ?type WHERE {{
                <{}> <{}joinType> ?type .
            }}",
            task_id, YAWL_NS
        );

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        if let Ok(results) = SparqlEvaluator::new()
            .parse_query(&query)
            .and_then(|q| q.on_store(&self.store).execute())
        {
            if let Some(binding) = results.iter().next() {
                if let Some(type_term) = binding.get("type") {
                    if let Term::Literal(lit) = type_term {
                        return Ok(match lit.value() {
                            "AND" => crate::parser::JoinType::And,
                            "XOR" => crate::parser::JoinType::Xor,
                            "OR" => crate::parser::JoinType::Or,
                            _ => crate::parser::JoinType::And,
                        });
                    }
                }
            }
        }

        Ok(crate::parser::JoinType::And) // Default
    }

    /// Extract max ticks
    fn extract_max_ticks(&self, task_id: &str) -> WorkflowResult<Option<u32>> {
        let query = format!(
            "SELECT ?ticks WHERE {{
                <{}> <{}maxTicks> ?ticks .
            }}",
            task_id, KNHK_PATTERN_NS
        );

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        if let Ok(results) = SparqlEvaluator::new()
            .parse_query(&query)
            .and_then(|q| q.on_store(&self.store).execute())
        {
            if let Some(binding) = results.iter().next() {
                if let Some(ticks_term) = binding.get("ticks") {
                    if let Term::Literal(lit) = ticks_term {
                        if let Ok(ticks) = lit.value().parse::<u32>() {
                            return Ok(Some(ticks));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Extract priority
    fn extract_priority(&self, task_id: &str) -> WorkflowResult<Option<u32>> {
        let query = format!(
            "SELECT ?priority WHERE {{
                <{}> <{}priority> ?priority .
            }}",
            task_id, KNHK_PATTERN_NS
        );

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        if let Ok(results) = SparqlEvaluator::new()
            .parse_query(&query)
            .and_then(|q| q.on_store(&self.store).execute())
        {
            if let Some(binding) = results.iter().next() {
                if let Some(priority_term) = binding.get("priority") {
                    if let Term::Literal(lit) = priority_term {
                        if let Ok(priority) = lit.value().parse::<u32>() {
                            return Ok(Some(priority));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Extract SIMD support
    fn extract_simd_support(&self, task_id: &str) -> WorkflowResult<bool> {
        let query = format!(
            "SELECT ?simd WHERE {{
                <{}> <{}simdSupport> ?simd .
            }}",
            task_id, KNHK_PATTERN_NS
        );

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        if let Ok(results) = SparqlEvaluator::new()
            .parse_query(&query)
            .and_then(|q| q.on_store(&self.store).execute())
        {
            if let Some(binding) = results.iter().next() {
                if let Some(simd_term) = binding.get("simd") {
                    if let Term::Literal(lit) = simd_term {
                        return Ok(lit.value().parse::<bool>().unwrap_or(false));
                    }
                }
            }
        }

        Ok(false) // Default
    }

    /// Extract input conditions
    fn extract_input_conditions(&self, task_id: &str) -> WorkflowResult<Vec<String>> {
        self.extract_list_property(task_id, &format!("{}hasInputCondition", YAWL_NS))
    }

    /// Extract output conditions
    fn extract_output_conditions(&self, task_id: &str) -> WorkflowResult<Vec<String>> {
        self.extract_list_property(task_id, &format!("{}hasOutputCondition", YAWL_NS))
    }

    /// Extract outgoing flows
    fn extract_outgoing_flows(&self, task_id: &str) -> WorkflowResult<Vec<String>> {
        self.extract_list_property(task_id, &format!("{}hasOutgoingFlow", YAWL_NS))
    }

    /// Extract incoming flows
    fn extract_incoming_flows(&self, task_id: &str) -> WorkflowResult<Vec<String>> {
        self.extract_list_property(task_id, &format!("{}hasIncomingFlow", YAWL_NS))
    }

    /// Extract list property
    fn extract_list_property(&self, subject: &str, predicate: &str) -> WorkflowResult<Vec<String>> {
        let query = format!(
            "SELECT ?value WHERE {{
                <{}> <{}> ?value .
            }}",
            subject, predicate
        );

        let mut values = Vec::new();
        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        if let Ok(results) = SparqlEvaluator::new()
            .parse_query(&query)
            .and_then(|q| q.on_store(&self.store).execute())
        {
            for binding in results {
                if let Some(value_term) = binding.get("value") {
                    if let Term::NamedNode(node) = value_term {
                        values.push(node.as_str().to_string());
                    } else if let Term::Literal(lit) = value_term {
                        values.push(lit.value().to_string());
                    }
                }
            }
        }

        Ok(values)
    }

    /// Extract string property
    fn extract_string_property(&self, subject: &str, predicate: &str) -> Option<String> {
        let query = format!(
            "SELECT ?value WHERE {{
                <{}> <{}> ?value .
            }}",
            subject, predicate
        );

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        if let Ok(results) = SparqlEvaluator::new()
            .parse_query(&query)
            .and_then(|q| q.on_store(&self.store).execute())
        {
            if let Some(binding) = results.iter().next() {
                if let Some(value_term) = binding.get("value") {
                    if let Term::Literal(lit) = value_term {
                        return Some(lit.value().to_string());
                    }
                }
            }
        }

        None
    }

    /// Extract start condition
    fn extract_start_condition(&self, workflow_id: &str) -> WorkflowResult<Option<String>> {
        Ok(self.extract_string_property(
            workflow_id,
            &format!("{}hasStartCondition", YAWL_NS),
        ))
    }

    /// Extract end condition
    fn extract_end_condition(&self, workflow_id: &str) -> WorkflowResult<Option<String>> {
        Ok(self.extract_string_property(
            workflow_id,
            &format!("{}hasEndCondition", YAWL_NS),
        ))
    }

    /// Extract conditions
    fn extract_conditions(&self, workflow_id: &str) -> WorkflowResult<HashMap<String, String>> {
        let mut conditions = HashMap::new();

        let query = format!(
            "SELECT ?condition ?name WHERE {{
                <{}> <{}hasCondition> ?condition .
                ?condition <{}label> ?name .
            }}",
            workflow_id, YAWL_NS, RDFS_NS
        );

        // Use SparqlEvaluator (oxigraph 0.5 best practices)
        if let Ok(results) = SparqlEvaluator::new()
            .parse_query(&query)
            .and_then(|q| q.on_store(&self.store).execute())
        {
            for binding in results {
                if let (Some(cond_term), Some(name_term)) = (binding.get("condition"), binding.get("name")) {
                    if let (Term::NamedNode(cond_node), Term::Literal(name_lit)) = (cond_term, name_term) {
                        conditions.insert(cond_node.as_str().to_string(), name_lit.value().to_string());
                    }
                }
            }
        }

        Ok(conditions)
    }
}

