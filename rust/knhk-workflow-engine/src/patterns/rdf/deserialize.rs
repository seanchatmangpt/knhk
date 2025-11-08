//! RDF deserialization functions

use super::metadata::{get_all_pattern_metadata, PatternMetadata};
use super::{WORKFLOW_PATTERN_NS, YAWL_NS};
use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternId};
use oxigraph::model::Term;
use oxigraph::store::Store;
use std::collections::HashMap;

/// Deserialize pattern metadata from RDF store
pub fn deserialize_metadata_from_rdf(
    store: &Store,
    pattern_id: PatternId,
) -> WorkflowResult<PatternMetadata> {
    let pattern_ns = WORKFLOW_PATTERN_NS;
    let pattern_iri = format!("{}pattern:{}", pattern_ns, pattern_id.0);

    let query = format!(
        "PREFIX pattern: <{}>\n\
         PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
         SELECT ?name ?description ?category ?complexity ?dep WHERE {{\n\
           <{}> rdf:type pattern:WorkflowPattern ;\n\
                pattern:patternId {} ;\n\
                rdfs:label ?name ;\n\
                rdfs:description ?description ;\n\
                pattern:category ?category ;\n\
                pattern:complexity ?complexity .\n\
           OPTIONAL {{ <{}> pattern:dependsOn ?dep }} .\n\
         }}",
        pattern_ns, pattern_iri, pattern_id.0, pattern_iri
    );

    #[allow(deprecated)]
    let query_results = store
        .query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Failed to query pattern metadata: {}", e)))?;

    if let oxigraph::sparql::QueryResults::Solutions(mut solutions) = query_results {
        if let Some(solution) = solutions.next() {
            let solution = solution.map_err(|e| {
                WorkflowError::Parse(format!("Failed to process solution: {:?}", e))
            })?;

            let name = solution
                .get("name")
                .and_then(|t| {
                    if let Term::Literal(lit) = t {
                        Some(lit.value().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| format!("Pattern {}", pattern_id.0));

            let description = solution
                .get("description")
                .and_then(|t| {
                    if let Term::Literal(lit) = t {
                        Some(lit.value().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let category = solution
                .get("category")
                .and_then(|t| {
                    if let Term::Literal(lit) = t {
                        Some(lit.value().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let complexity = solution
                .get("complexity")
                .and_then(|t| {
                    if let Term::Literal(lit) = t {
                        Some(lit.value().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Medium".to_string());

            // Collect dependencies
            let mut dependencies = Vec::new();
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    WorkflowError::Parse(format!("Failed to process dependency solution: {:?}", e))
                })?;
                if let Some(dep_term) = solution.get("dep") {
                    if let Term::NamedNode(node) = dep_term {
                        let dep_str = node.as_str();
                        if let Some(dep_id_str) =
                            dep_str.strip_prefix(&format!("{}pattern:", pattern_ns))
                        {
                            if let Ok(dep_id) = dep_id_str.parse::<u32>() {
                                dependencies.push(dep_id);
                            }
                        }
                    }
                }
            }

            Ok(PatternMetadata {
                pattern_id: pattern_id.0,
                name,
                description,
                category,
                complexity,
                dependencies,
            })
        } else {
            // Fallback: return metadata from get_all_pattern_metadata
            get_all_pattern_metadata()
                .into_iter()
                .find(|m| m.pattern_id == pattern_id.0)
                .ok_or_else(|| WorkflowError::PatternNotFound(pattern_id.0))
        }
    } else {
        Err(WorkflowError::Parse(
            "No solutions found for pattern metadata query".to_string(),
        ))
    }
}

/// Load pattern metadata from RDF store for all patterns
pub fn load_all_metadata_from_rdf(store: &Store) -> WorkflowResult<Vec<PatternMetadata>> {
    let mut metadata = Vec::new();

    for pattern_id in 1..=43 {
        let pattern_id = PatternId::new(pattern_id)?;
        match deserialize_metadata_from_rdf(store, pattern_id) {
            Ok(meta) => metadata.push(meta),
            Err(_) => {
                // Fallback to hardcoded metadata if not found in store
                if let Some(meta) = get_all_pattern_metadata()
                    .into_iter()
                    .find(|m| m.pattern_id == pattern_id.0)
                {
                    metadata.push(meta);
                }
            }
        }
    }

    Ok(metadata)
}

/// Deserialize pattern execution context from RDF/Turtle
pub fn deserialize_context_from_rdf(
    turtle: &str,
) -> WorkflowResult<(PatternId, PatternExecutionContext)> {
    use oxigraph::io::RdfFormat;

    let store = Store::new()
        .map_err(|e| WorkflowError::Parse(format!("Failed to create RDF store: {:?}", e)))?;

    store
        .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
        .map_err(|e| WorkflowError::Parse(format!("Failed to parse Turtle: {:?}", e)))?;

    let pattern_ns = WORKFLOW_PATTERN_NS;
    let yawl_ns = YAWL_NS;

    let query = format!(
        "PREFIX pattern: <{}>\n\
         PREFIX yawl: <{}>\n\
         PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         SELECT ?patternId ?caseId ?workflowId ?varKey ?varValue WHERE {{\n\
           ?execution rdf:type pattern:PatternExecution ;\n\
                      pattern:executesPattern ?pattern ;\n\
                      yawl:hasCase ?case ;\n\
                      yawl:hasWorkflowSpec ?workflow .\n\
           ?pattern pattern:patternId ?patternId .\n\
           OPTIONAL {{ ?execution pattern:hasVariables [ pattern:variable ?varKey ?varValue ] }} .\n\
         }}",
        pattern_ns, yawl_ns
    );

    #[allow(deprecated)]
    let query_results = store
        .query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Failed to query execution context: {}", e)))?;

    if let oxigraph::sparql::QueryResults::Solutions(mut solutions) = query_results {
        if let Some(solution) = solutions.next() {
            let solution = solution.map_err(|e| {
                WorkflowError::Parse(format!("Failed to process solution: {:?}", e))
            })?;

            let pattern_id = solution
                .get("patternId")
                .and_then(|t| {
                    if let Term::Literal(lit) = t {
                        lit.value().parse::<u32>().ok()
                    } else {
                        None
                    }
                })
                .ok_or_else(|| WorkflowError::Parse("Pattern ID missing".to_string()))?;

            let pattern_id = PatternId::new(pattern_id)?;

            // Extract case ID and workflow ID from IRIs
            let case_id = solution
                .get("caseId")
                .and_then(|t| {
                    if let Term::NamedNode(node) = t {
                        let iri = node.as_str();
                        if let Some(id_str) = iri.strip_prefix(&format!("{}case:", yawl_ns)) {
                            crate::case::CaseId::parse_str(id_str).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| crate::case::CaseId::new());

            let workflow_id = solution
                .get("workflowId")
                .and_then(|t| {
                    if let Term::NamedNode(node) = t {
                        let iri = node.as_str();
                        if let Some(id_str) = iri.strip_prefix(&format!("{}workflow:", yawl_ns)) {
                            crate::parser::WorkflowSpecId::parse_str(id_str).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| crate::parser::WorkflowSpecId::new());

            let mut variables = HashMap::new();

            // Collect variables from all solutions
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    WorkflowError::Parse(format!("Failed to process variable solution: {:?}", e))
                })?;

                if let (Some(key_term), Some(value_term)) =
                    (solution.get("varKey"), solution.get("varValue"))
                {
                    let key = if let Term::Literal(lit) = key_term {
                        lit.value().to_string()
                    } else {
                        continue;
                    };

                    let value = if let Term::Literal(lit) = value_term {
                        lit.value().to_string()
                    } else {
                        continue;
                    };

                    variables.insert(key, value);
                }
            }

            let context = PatternExecutionContext {
                case_id,
                workflow_id,
                variables,
                arrived_from: std::collections::HashSet::new(),
                scope_id: String::new(),
            };

            Ok((pattern_id, context))
        } else {
            Err(WorkflowError::Parse(
                "No execution context found in RDF".to_string(),
            ))
        }
    } else {
        Err(WorkflowError::Parse(
            "No solutions found for execution context query".to_string(),
        ))
    }
}

/// Deserialize pattern execution result from RDF/Turtle
pub fn deserialize_result_from_rdf(turtle: &str) -> WorkflowResult<PatternExecutionResult> {
    use oxigraph::io::RdfFormat;

    let store = Store::new()
        .map_err(|e| WorkflowError::Parse(format!("Failed to create RDF store: {:?}", e)))?;

    store
        .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
        .map_err(|e| WorkflowError::Parse(format!("Failed to parse Turtle: {:?}", e)))?;

    let pattern_ns = WORKFLOW_PATTERN_NS;

    let query = format!(
        "PREFIX pattern: <{}>\n\
         PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         SELECT ?success ?nextState ?varKey ?varValue WHERE {{\n\
           ?result rdf:type pattern:PatternExecutionResult ;\n\
                   pattern:success ?success .\n\
           OPTIONAL {{ ?result pattern:nextState ?nextState }} .\n\
           OPTIONAL {{ ?result pattern:hasOutputVariables [ pattern:variable ?varKey ?varValue ] }} .\n\
         }}",
        pattern_ns
    );

    #[allow(deprecated)]
    let query_results = store
        .query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Failed to query execution result: {}", e)))?;

    if let oxigraph::sparql::QueryResults::Solutions(mut solutions) = query_results {
        if let Some(solution) = solutions.next() {
            let solution = solution.map_err(|e| {
                WorkflowError::Parse(format!("Failed to process solution: {:?}", e))
            })?;

            let success = solution
                .get("success")
                .and_then(|t| {
                    if let Term::Literal(lit) = t {
                        lit.value().parse::<bool>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(false);

            let next_state = solution.get("nextState").and_then(|t| {
                if let Term::Literal(lit) = t {
                    Some(lit.value().to_string())
                } else {
                    None
                }
            });

            let mut variables = HashMap::new();

            // Collect variables from all solutions
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    WorkflowError::Parse(format!("Failed to process variable solution: {:?}", e))
                })?;

                if let (Some(key_term), Some(value_term)) =
                    (solution.get("varKey"), solution.get("varValue"))
                {
                    let key = if let Term::Literal(lit) = key_term {
                        lit.value().to_string()
                    } else {
                        continue;
                    };

                    let value = if let Term::Literal(lit) = value_term {
                        lit.value().to_string()
                    } else {
                        continue;
                    };

                    variables.insert(key, value);
                }
            }

            Ok(PatternExecutionResult {
                success,
                next_state,
                next_activities: Vec::new(),
                variables,
                updates: None,
                cancel_activities: Vec::new(),
                terminates: false,
            })
        } else {
            Err(WorkflowError::Parse(
                "No execution result found in RDF".to_string(),
            ))
        }
    } else {
        Err(WorkflowError::Parse(
            "No solutions found for execution result query".to_string(),
        ))
    }
}
