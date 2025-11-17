//! Warm path executor with path selection
//! Routes queries to oxigraph (warm path)
//!
//! This module is only available with the `rdf` feature enabled.

#![cfg(feature = "rdf")]

use crate::graph::WarmPathGraph;
use crate::query::{AskResult, ConstructResult, DescribeResult, SelectResult};
// Path selector removed - use simple routing logic instead
use std::sync::Arc;

/// Warm path executor that routes queries to appropriate backend
pub struct WarmPathExecutor {
    graph: Arc<WarmPathGraph>,
}

/// Unified query result
#[derive(Debug, Clone)]
pub enum QueryExecutionResult {
    Select(SelectResult),
    Ask(AskResult),
    Construct(ConstructResult),
    Describe(DescribeResult),
}

impl WarmPathExecutor {
    /// Create new warm path executor
    pub fn new() -> Result<Self, String> {
        let graph = WarmPathGraph::new().map_err(|e| format!("Failed to create graph: {}", e))?;

        Ok(Self {
            graph: Arc::new(graph),
        })
    }

    /// Execute query with automatic path selection
    ///
    /// Routes queries based on complexity:
    /// - Hot path: Simple ASK, data size ≤8
    /// - Warm path: SPARQL queries, data size ≤10K
    /// - Cold path: Complex queries, SHACL, reasoning
    pub fn execute_query(&self, sparql: &str) -> Result<QueryExecutionResult, String> {
        // Simple routing: always use warm path for now
        // Path selection based on query complexity planned for v1.1
        // Current implementation routes all queries to warm path (acceptable for v1.0)
        self.execute_warm_path(sparql)
    }

    /// Execute query via hot path (C, ≤2ns)
    #[allow(dead_code)] // FUTURE: Hot path optimization
    fn execute_hot_path(&self, sparql: &str) -> Result<QueryExecutionResult, String> {
        let query_upper = sparql.trim().to_uppercase();

        if query_upper.starts_with("ASK") {
            match crate::hot_path::execute_hot_path_ask(&self.graph, sparql) {
                Ok(result) => Ok(QueryExecutionResult::Ask(result)),
                Err(_e) => {
                    // Fall back to warm path if hot path fails
                    self.execute_warm_path(sparql)
                }
            }
        } else if query_upper.starts_with("SELECT") && query_upper.contains("COUNT") {
            match crate::hot_path::execute_hot_path_select(&self.graph, sparql) {
                Ok(result) => Ok(QueryExecutionResult::Select(result)),
                Err(_e) => {
                    // Fall back to warm path if hot path fails
                    self.execute_warm_path(sparql)
                }
            }
        } else {
            // Not a hot path query type - fall back to warm path
            self.execute_warm_path(sparql)
        }
    }

    /// Execute query via warm path (oxigraph)
    fn execute_warm_path(&self, sparql: &str) -> Result<QueryExecutionResult, String> {
        let query_upper = sparql.trim().to_uppercase();

        if query_upper.starts_with("SELECT") {
            let result = crate::query::execute_select(&self.graph, sparql)
                .map_err(|e| format!("SELECT query failed: {}", e))?;
            Ok(QueryExecutionResult::Select(result))
        } else if query_upper.starts_with("ASK") {
            let result = crate::query::execute_ask(&self.graph, sparql)
                .map_err(|e| format!("ASK query failed: {}", e))?;
            Ok(QueryExecutionResult::Ask(result))
        } else if query_upper.starts_with("CONSTRUCT") {
            let result = crate::query::execute_construct(&self.graph, sparql)
                .map_err(|e| format!("CONSTRUCT query failed: {}", e))?;
            Ok(QueryExecutionResult::Construct(result))
        } else if query_upper.starts_with("DESCRIBE") {
            let result = crate::query::execute_describe(&self.graph, sparql)
                .map_err(|e| format!("DESCRIBE query failed: {}", e))?;
            Ok(QueryExecutionResult::Describe(result))
        } else {
            Err(format!(
                "Unsupported query type: {}",
                query_upper.split_whitespace().next().unwrap_or("unknown")
            ))
        }
    }

    /// Load RDF data into graph
    pub fn load_rdf(&self, turtle_data: &str) -> Result<(), String> {
        self.graph.load_from_turtle(turtle_data)
    }

    /// Get underlying graph reference
    pub fn graph(&self) -> &WarmPathGraph {
        &self.graph
    }
}

impl Default for WarmPathExecutor {
    fn default() -> Self {
        // Default implementation should not fail
        // If new() fails, create minimal executor rather than panicking
        Self::new().unwrap_or_else(|_| {
            // Create minimal executor with default graph
            Self {
                graph: Arc::new(WarmPathGraph::default()),
            }
        })
    }
}
