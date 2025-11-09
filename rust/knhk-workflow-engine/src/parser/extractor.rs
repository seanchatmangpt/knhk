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

    // Find workflow specifications (try both Specification and WorkflowSpecification)
    let query = format!(
        "PREFIX yawl: <{}>\n\
         PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         PREFIX rdfs: <{}>\n\
         SELECT ?spec ?name WHERE {{\n\
           {{ ?spec rdf:type yawl:Specification . }}\n\
           UNION\n\
           {{ ?spec rdf:type yawl:WorkflowSpecification . }}\n\
           OPTIONAL {{ ?spec rdfs:label ?name }}\n\
           OPTIONAL {{ ?spec yawl:specName ?name }}\n\
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
            if let Some(oxigraph::model::Term::Literal(lit)) = solution.get("name") {
                spec_name = lit.value().to_string();
            }
        }
    }

    // Extract tasks
    let mut tasks = extract_tasks(store, yawl_ns, spec_iri.as_deref())?;

    // Extract conditions
    let mut conditions = extract_conditions(store, yawl_ns, spec_iri.as_deref())?;

    // Extract flows
    let flows = extract_flows(
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
        flows,
        start_condition,
        end_condition,
        source_turtle: None,
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
             SELECT ?task ?name ?type ?splitType ?joinType ?maxTicks ?priority ?simd WHERE {{\n\
               {} yawl:hasTask ?task .\n\
               ?task rdf:type ?type .\n\
               OPTIONAL {{ ?task rdfs:label ?name }}\n\
               OPTIONAL {{ ?task yawl:taskName ?name }}\n\
               OPTIONAL {{ ?task yawl:hasSplit ?split . ?split rdf:type ?splitType }}\n\
               OPTIONAL {{ ?task yawl:hasJoin ?join . ?join rdf:type ?joinType }}\n\
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
             SELECT ?task ?name ?splitType ?joinType WHERE {{\n\
               ?task rdf:type yawl:Task .\n\
               OPTIONAL {{ ?task rdfs:label ?name }}\n\
               OPTIONAL {{ ?task yawl:taskName ?name }}\n\
               OPTIONAL {{ ?task yawl:hasSplit ?split . ?split rdf:type ?splitType }}\n\
               OPTIONAL {{ ?task yawl:hasJoin ?join . ?join rdf:type ?joinType }}\n\
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
                .get("splitType")
                .or_else(|| solution.get("split"))
                .and_then(|s| {
                    if let oxigraph::model::Term::NamedNode(node) = s {
                        let type_str = node.as_str();
                        if type_str.contains("ControlTypeAnd") || type_str.contains("And") {
                            Some(SplitType::And)
                        } else if type_str.contains("ControlTypeXor")
                            || type_str.contains("Xor")
                            || type_str.contains("XOR")
                        {
                            Some(SplitType::Xor)
                        } else if type_str.contains("ControlTypeOr")
                            || type_str.contains("Or")
                            || type_str.contains("OR")
                        {
                            Some(SplitType::Or)
                        } else {
                            None
                        }
                    } else if let oxigraph::model::Term::Literal(lit) = s {
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
                .get("joinType")
                .or_else(|| solution.get("join"))
                .and_then(|j| {
                    if let oxigraph::model::Term::NamedNode(node) = j {
                        let type_str = node.as_str();
                        if type_str.contains("ControlTypeAnd") || type_str.contains("And") {
                            Some(JoinType::And)
                        } else if type_str.contains("ControlTypeXor")
                            || type_str.contains("Xor")
                            || type_str.contains("XOR")
                        {
                            Some(JoinType::Xor)
                        } else if type_str.contains("ControlTypeOr")
                            || type_str.contains("Or")
                            || type_str.contains("OR")
                        {
                            Some(JoinType::Or)
                        } else {
                            None
                        }
                    } else if let oxigraph::model::Term::Literal(lit) = j {
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

            // Extract input and output parameters
            let input_parameters = extract_task_parameters(store, &yawl_ns, &task_id, true)?;
            let output_parameters = extract_task_parameters(store, &yawl_ns, &task_id, false)?;

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
                input_parameters,
                output_parameters,
                allocation_policy: None,
                required_roles: Vec::new(),
                required_capabilities: Vec::new(),
                exception_worklet: None,
            };

            tasks.insert(task_id, task);
        }
    }

    Ok(tasks)
}

/// Extract task parameters (input or output) from RDF store
fn extract_task_parameters(
    store: &Store,
    yawl_ns: &str,
    task_id: &str,
    is_input: bool,
) -> WorkflowResult<Vec<crate::parser::types::TaskParameter>> {
    let param_type = if is_input {
        "hasInputParameter"
    } else {
        "hasOutputParameter"
    };

    // Ensure task_id is properly formatted as an IRI (trim any whitespace)
    // Remove any existing angle brackets from task_id if present
    let task_id_clean = task_id.trim().trim_start_matches('<').trim_end_matches('>');

    // Construct full IRI for the parameter property
    let param_property_iri = format!("{}{}", yawl_ns, param_type);

    let query = format!(
        "PREFIX yawl: <{}>\n\
         PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         SELECT ?paramName ?paramType WHERE {{\n\
           <{}> <{}> ?param .\n\
           ?param yawl:paramName ?paramName .\n\
           OPTIONAL {{ ?param yawl:paramType ?paramType }}\n\
         }}",
        yawl_ns, task_id_clean, param_property_iri
    );

    #[allow(deprecated)]
    let query_results = store
        .query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Failed to query task parameters: {:?}", e)))?;

    let mut parameters = Vec::new();

    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            let solution = solution.map_err(|e| {
                WorkflowError::Parse(format!("Failed to process parameter solution: {:?}", e))
            })?;

            if let Some(param_name_term) = solution.get("paramName") {
                let param_name = if let oxigraph::model::Term::Literal(lit) = param_name_term {
                    lit.value().to_string()
                } else {
                    param_name_term.to_string()
                };

                let param_type = solution
                    .get("paramType")
                    .and_then(|t| {
                        if let oxigraph::model::Term::Literal(lit) = t {
                            Some(lit.value().to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "string".to_string());

                parameters.push(crate::parser::types::TaskParameter {
                    name: param_name,
                    param_type,
                });
            }
        }
    }

    Ok(parameters)
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
               {{ {} yawl:hasCondition ?condition . }}\n\
               UNION\n\
               {{ {} yawl:hasInputCondition ?condition . }}\n\
               UNION\n\
               {{ {} yawl:hasOutputCondition ?condition . }}\n\
               OPTIONAL {{ ?condition rdfs:label ?name }}\n\
               OPTIONAL {{ ?condition yawl:conditionName ?name }}\n\
             }}",
            yawl_ns, spec, spec, spec
        )
    } else {
        format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
             SELECT ?condition ?name WHERE {{\n\
               {{ ?condition rdf:type yawl:Condition . }}\n\
               UNION\n\
               {{ ?condition rdf:type yawl:InputCondition . }}\n\
               UNION\n\
               {{ ?condition rdf:type yawl:OutputCondition . }}\n\
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
) -> WorkflowResult<Vec<crate::parser::types::Flow>> {
    // Query for flows using yawl:flowsFrom, yawl:flowsInto, and optional yawl:predicate
    let query = format!(
        "PREFIX yawl: <{}>\n\
         PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         SELECT ?flow ?from ?to ?predicate WHERE {{\n\
           ?flow rdf:type yawl:Flow .\n\
           ?flow yawl:flowsFrom ?from .\n\
           ?flow yawl:flowsInto ?to .\n\
           OPTIONAL {{ ?flow yawl:predicate ?predicate . }}\n\
         }}",
        yawl_ns
    );

    #[allow(deprecated)]
    let query_results = store
        .query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Failed to query flows: {:?}", e)))?;

    let mut flows = Vec::new();

    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            let solution = solution.map_err(|e| {
                WorkflowError::Parse(format!("Failed to process flow solution: {:?}", e))
            })?;

            if let (Some(flow_term), Some(from_term), Some(to_term)) = (
                solution.get("flow"),
                solution.get("from"),
                solution.get("to"),
            ) {
                let flow_id = flow_term.to_string();
                let from_id = from_term.to_string();
                let to_id = to_term.to_string();

                // Extract predicate if present
                let predicate = solution.get("predicate").map(|p| {
                    let pred_str = p.to_string();
                    // Remove quotes if present
                    pred_str.trim_matches('"').to_string()
                });

                // Create Flow object
                flows.push(crate::parser::types::Flow {
                    id: flow_id,
                    from: from_id.clone(),
                    to: to_id.clone(),
                    predicate,
                });

                // Add to task or condition based on what exists (for backward compatibility)
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

    Ok(flows)
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
               {{ {} yawl:hasStartCondition ?condition . }}\n\
               UNION\n\
               {{ {} yawl:hasInputCondition ?condition . }}\n\
             }} LIMIT 1",
            yawl_ns, spec, spec
        )
    } else {
        format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             SELECT ?condition WHERE {{\n\
               {{ ?condition rdf:type yawl:StartCondition . }}\n\
               UNION\n\
               {{ ?condition rdf:type yawl:InputCondition . }}\n\
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
               {{ {} yawl:hasEndCondition ?condition . }}\n\
               UNION\n\
               {{ {} yawl:hasOutputCondition ?condition . }}\n\
             }} LIMIT 1",
            yawl_ns, spec, spec
        )
    } else {
        format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
             SELECT ?condition WHERE {{\n\
               {{ ?condition rdf:type yawl:EndCondition . }}\n\
               UNION\n\
               {{ ?condition rdf:type yawl:OutputCondition . }}\n\
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
