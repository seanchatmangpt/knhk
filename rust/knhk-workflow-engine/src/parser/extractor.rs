//! RDF extraction utilities
//!
//! Provides methods for extracting workflow components from RDF stores.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::types::{
    Condition, JoinType, SplitType, Task, TaskType, WorkflowSpec, WorkflowSpecId,
};
use oxigraph::store::Store;
use std::collections::HashMap;

/// Extract workflow specification from RDF store
pub fn extract_workflow_spec(store: &Store) -> WorkflowResult<WorkflowSpec> {
    // YAWL namespace prefixes
    let yawl_ns = "http://bitflow.ai/ontology/yawl/v2#";
    let rdfs = "http://www.w3.org/2000/01/rdf-schema#";
    let _rdf_type_iri = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

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
    let query_results = store
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
    let mut tasks = extract_tasks(store, &yawl_ns, spec_iri.as_deref())?;

    // Extract conditions
    let mut conditions = extract_conditions(store, &yawl_ns, spec_iri.as_deref())?;

    // Extract flows
    extract_flows(
        store,
        &yawl_ns,
        spec_iri.as_deref(),
        &mut tasks,
        &mut conditions,
    )?;

    // Find start and end conditions
    let start_condition = find_start_condition(store, &yawl_ns, spec_iri.as_deref())?;
    let end_condition = find_end_condition(store, &yawl_ns, spec_iri.as_deref())?;

    Ok(WorkflowSpec {
        id: spec_id,
        name: spec_name,
        tasks,
        conditions,
        start_condition,
        end_condition,
    })
}

/// Extract tasks from RDF store
pub fn extract_tasks(
    store: &Store,
    yawl_ns: &str,
    spec_iri: Option<&str>,
) -> WorkflowResult<HashMap<String, Task>> {
    let mut tasks = HashMap::new();

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
    let query_results = store
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
                .and_then(|n| {
                    if let oxigraph::model::Term::Literal(lit) = n {
                        Some(lit.value().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Unnamed Task".to_string());

            let task_type = solution
                .get("type")
                .and_then(|t| {
                    if let oxigraph::model::Term::NamedNode(node) = t {
                        let type_str = node.as_str();
                        if type_str.contains("Atomic") {
                            Some(TaskType::Atomic)
                        } else if type_str.contains("Composite") {
                            Some(TaskType::Composite)
                        } else if type_str.contains("MultipleInstance") {
                            Some(TaskType::MultipleInstance)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(TaskType::Atomic);

            let split_type = solution
                .get("split")
                .and_then(|s| {
                    if let oxigraph::model::Term::Literal(lit) = s {
                        match lit.value() {
                            "AND" => Some(SplitType::And),
                            "XOR" => Some(SplitType::Xor),
                            "OR" => Some(SplitType::Or),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(SplitType::And);

            let join_type = solution
                .get("join")
                .and_then(|j| {
                    if let oxigraph::model::Term::Literal(lit) = j {
                        match lit.value() {
                            "AND" => Some(JoinType::And),
                            "XOR" => Some(JoinType::Xor),
                            "OR" => Some(JoinType::Or),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(JoinType::And);

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
pub fn extract_conditions(
    store: &Store,
    yawl_ns: &str,
    spec_iri: Option<&str>,
) -> WorkflowResult<HashMap<String, Condition>> {
    let mut conditions = HashMap::new();

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
    let query_results = store
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
pub fn extract_flows(
    store: &Store,
    yawl_ns: &str,
    _spec_iri: Option<&str>,
    tasks: &mut HashMap<String, Task>,
    conditions: &mut HashMap<String, Condition>,
) -> WorkflowResult<()> {
    let query = format!(
        "PREFIX yawl: <{}>\n\
         PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         SELECT ?from ?to WHERE {{\n\
           ?from yawl:hasOutgoingFlow ?to .\n\
         }}",
        yawl_ns
    );

    #[allow(deprecated)]
    let query_results = store
        .query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Failed to query flows: {:?}", e)))?;

    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            let solution = solution.map_err(|e| {
                WorkflowError::Parse(format!("Failed to process flow solution: {:?}", e))
            })?;

            if let (Some(from_term), Some(to_term)) = (solution.get("from"), solution.get("to")) {
                let from_id = from_term.to_string();
                let to_id = to_term.to_string();

                // Add to task or condition based on what exists
                if let Some(task) = tasks.get_mut(&from_id) {
                    task.outgoing_flows.push(to_id.clone());
                }
                if let Some(condition) = conditions.get_mut(&from_id) {
                    condition.outgoing_flows.push(to_id.clone());
                }

                if let Some(task) = tasks.get_mut(&to_id) {
                    task.incoming_flows.push(from_id.clone());
                }
                if let Some(condition) = conditions.get_mut(&to_id) {
                    condition.incoming_flows.push(from_id.clone());
                }
            }
        }
    }

    Ok(())
}

/// Find start condition
pub fn find_start_condition(
    store: &Store,
    yawl_ns: &str,
    spec_iri: Option<&str>,
) -> WorkflowResult<Option<String>> {
    let query = if let Some(spec) = spec_iri {
        format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             SELECT ?condition WHERE {{\n\
               <{}> yawl:hasStartCondition ?condition .\n\
             }} LIMIT 1",
            yawl_ns, spec
        )
    } else {
        format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             SELECT ?condition WHERE {{\n\
               ?condition rdf:type yawl:StartCondition .\n\
             }} LIMIT 1",
            yawl_ns
        )
    };

    #[allow(deprecated)]
    let query_results = store
        .query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Failed to query start condition: {:?}", e)))?;

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
pub fn find_end_condition(
    store: &Store,
    yawl_ns: &str,
    spec_iri: Option<&str>,
) -> WorkflowResult<Option<String>> {
    let query = if let Some(spec) = spec_iri {
        format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             SELECT ?condition WHERE {{\n\
               <{}> yawl:hasEndCondition ?condition .\n\
             }} LIMIT 1",
            yawl_ns, spec
        )
    } else {
        format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             SELECT ?condition WHERE {{\n\
               ?condition rdf:type yawl:EndCondition .\n\
             }} LIMIT 1",
            yawl_ns
        )
    };

    #[allow(deprecated)]
    let query_results = store
        .query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Failed to query end condition: {:?}", e)))?;

    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            let solution = solution.map_err(|e| {
                WorkflowError::Parse(format!("Failed to process end condition solution: {:?}", e))
            })?;

            if let Some(condition_term) = solution.get("condition") {
                return Ok(Some(condition_term.to_string()));
            }
        }
    }

    Ok(None)
}
