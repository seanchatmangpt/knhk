//! SPARQL query execution functions for warm path
//! Wraps oxigraph query execution with KNHK-compatible result formats

use crate::graph::WarmPathGraph;
use oxigraph::sparql::QueryResults;
use oxigraph::model::Triple;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// Query execution error
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Query parse failed: {0}")]
    ParseError(String),
    #[error("Query execution failed: {0}")]
    ExecutionError(String),
    #[error("Unsupported query type: {0}")]
    UnsupportedQueryType(String),
}

/// SELECT query result
#[derive(Debug, Clone)]
pub struct SelectResult {
    pub bindings: Vec<BTreeMap<String, String>>,
    pub variables: Vec<String>,
}

/// ASK query result
#[derive(Debug, Clone)]
pub struct AskResult {
    pub result: bool,
}

/// CONSTRUCT query result
#[derive(Debug, Clone)]
pub struct ConstructResult {
    pub triples: Vec<String>,
}

/// DESCRIBE query result
#[derive(Debug, Clone)]
pub struct DescribeResult {
    pub triples: Vec<String>,
}

/// Execute SPARQL SELECT query
pub fn execute_select(graph: &WarmPathGraph, sparql: &str) -> Result<SelectResult, QueryError> {
    let results = graph.query(sparql)
        .map_err(|e| QueryError::ExecutionError(e))?;

    match results {
        QueryResults::Solutions(solutions) => {
            let mut bindings = Vec::new();
            let mut variables = Vec::new();

            for solution in solutions {
                let solution = solution
                    .map_err(|e| QueryError::ExecutionError(format!("Solution error: {}", e)))?;
                
                let mut binding = BTreeMap::new();
                for (var, term) in solution.iter() {
                    let var_str = var.as_str().to_string();
                    if !variables.contains(&var_str) {
                        variables.push(var_str.clone());
                    }
                    binding.insert(var_str, term.to_string());
                }
                bindings.push(binding);
            }

            Ok(SelectResult {
                bindings,
                variables,
            })
        }
        _ => Err(QueryError::UnsupportedQueryType("Expected SELECT query".to_string())),
    }
}

/// Execute SPARQL ASK query
pub fn execute_ask(graph: &WarmPathGraph, sparql: &str) -> Result<AskResult, QueryError> {
    let results = graph.query(sparql)
        .map_err(|e| QueryError::ExecutionError(e))?;

    match results {
        QueryResults::Boolean(b) => Ok(AskResult { result: b }),
        _ => Err(QueryError::UnsupportedQueryType("Expected ASK query".to_string())),
    }
}

/// Execute SPARQL CONSTRUCT query
pub fn execute_construct(graph: &WarmPathGraph, sparql: &str) -> Result<ConstructResult, QueryError> {
    let results = graph.query(sparql)
        .map_err(|e| QueryError::ExecutionError(e))?;

    match results {
        QueryResults::Graph(triples_iter) => {
            let triples: Result<Vec<String>, String> = triples_iter
                .map(|t| t.map(|triple| triple.to_string()).map_err(|e| e.to_string()))
                .collect();
            
            Ok(ConstructResult {
                triples: triples.map_err(|e| QueryError::ExecutionError(e))?,
            })
        }
        _ => Err(QueryError::UnsupportedQueryType("Expected CONSTRUCT query".to_string())),
    }
}

/// Execute SPARQL DESCRIBE query
pub fn execute_describe(graph: &WarmPathGraph, sparql: &str) -> Result<DescribeResult, QueryError> {
    let results = graph.query(sparql)
        .map_err(|e| QueryError::ExecutionError(e))?;

    match results {
        QueryResults::Graph(triples_iter) => {
            let triples: Result<Vec<String>, String> = triples_iter
                .map(|t| t.map(|triple| triple.to_string()).map_err(|e| e.to_string()))
                .collect();
            
            Ok(DescribeResult {
                triples: triples.map_err(|e| QueryError::ExecutionError(e))?,
            })
        }
        _ => Err(QueryError::UnsupportedQueryType("Expected DESCRIBE query".to_string())),
    }
}

/// Convert SelectResult to JSON
pub fn select_to_json(result: &SelectResult) -> JsonValue {
    let mut arr = Vec::new();
    for binding in &result.bindings {
        let mut obj = serde_json::Map::new();
        for (k, v) in binding {
            obj.insert(k.clone(), JsonValue::String(v.clone()));
        }
        arr.push(JsonValue::Object(obj));
    }
    JsonValue::Array(arr)
}

/// Convert AskResult to JSON
pub fn ask_to_json(result: &AskResult) -> JsonValue {
    JsonValue::Bool(result.result)
}

/// Convert ConstructResult to JSON
pub fn construct_to_json(result: &ConstructResult) -> JsonValue {
    JsonValue::Array(
        result.triples
            .iter()
            .map(|t| JsonValue::String(t.clone()))
            .collect()
    )
}

