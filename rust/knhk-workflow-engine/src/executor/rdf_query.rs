//! Runtime RDF Query API
//!
//! Provides SPARQL query interface to inspect workflow state, patterns, and metadata at runtime.

use crate::case::CaseId;
use crate::parser::WorkflowSpecId;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;
use std::collections::HashMap;

use super::engine::WorkflowEngine;

/// Execute SPARQL query on store and return results
fn execute_sparql_query(
    store: &Store,
    sparql: &str,
) -> Result<Vec<HashMap<String, String>>, String> {
    // Use SparqlEvaluator (oxigraph 0.5 best practices)
    let query_result = oxigraph::sparql::SparqlEvaluator::new()
        .parse_query(sparql)
        .map_err(|e| format!("Failed to parse SPARQL query: {:?}", e))?
        .on_store(store)
        .execute()
        .map_err(|e| format!("Invalid SPARQL query: {:?}", e))?;

    match query_result {
        QueryResults::Solutions(solutions) => {
            let mut results = Vec::new();

            for solution in solutions {
                let solution =
                    solution.map_err(|e| format!("Failed to read query solution: {:?}", e))?;

                let mut binding = HashMap::new();
                for (var, term) in solution.iter() {
                    // Convert term to string and clean up literal quotes and type annotations
                    let term_string = term.to_string();
                    let cleaned = if term_string.starts_with('"') {
                        // Strip quotes and type annotations from literals
                        // Format: "value"^^<type> or "value"@lang or just "value"
                        let end_quote_pos = term_string[1..].find('"').map(|p| p + 1);
                        if let Some(pos) = end_quote_pos {
                            term_string[1..pos].to_string()
                        } else {
                            term_string
                        }
                    } else {
                        term_string
                    };
                    binding.insert(var.as_str().to_string(), cleaned);
                }
                results.push(binding);
            }

            Ok(results)
        }
        QueryResults::Boolean(b) => {
            // For ASK queries, return single result with "result" binding
            let mut binding = HashMap::new();
            binding.insert("result".to_string(), b.to_string());
            Ok(vec![binding])
        }
        QueryResults::Graph(_) => {
            // CONSTRUCT queries not supported for now
            Err("CONSTRUCT queries not supported, use SELECT instead".to_string())
        }
    }
}

impl WorkflowEngine {
    /// Query workflow specification RDF store
    ///
    /// Executes SPARQL SELECT query against the workflow specification RDF store.
    /// Returns variable bindings as Vec<HashMap<var_name, value>>.
    pub async fn query_rdf(
        &self,
        _spec_id: &WorkflowSpecId,
        sparql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let store = self.spec_rdf_store.read().await;
        execute_sparql_query(&store, sparql_query)
    }

    /// Query case runtime state RDF store
    ///
    /// Executes SPARQL SELECT query against case-specific RDF store containing
    /// runtime variables and state.
    pub async fn query_case_rdf(
        &self,
        case_id: &CaseId,
        sparql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let case_stores = self.case_rdf_stores.read().await;
        let store = case_stores
            .get(case_id)
            .ok_or_else(|| format!("Case {:?} not found", case_id))?;
        execute_sparql_query(store, sparql_query)
    }

    /// Query pattern metadata RDF store
    ///
    /// Executes SPARQL SELECT query against pattern metadata store containing
    /// all 43 Van der Aalst workflow patterns with their metadata, dependencies,
    /// and categorization.
    pub async fn query_pattern_metadata(
        &self,
        sparql_query: &str,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        let store = self.pattern_metadata_store.read().await;
        execute_sparql_query(&store, sparql_query)
    }

    /// Load workflow specification into RDF store (called during registration)
    pub(crate) async fn load_spec_rdf(&self, turtle: &str) -> Result<(), String> {
        let store = self.spec_rdf_store.write().await;
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| format!("Failed to load spec into RDF store: {:?}", e))
    }

    /// Create case RDF store
    pub(crate) async fn create_case_rdf_store(
        &self,
        case_id: CaseId,
        _data: &serde_json::Value,
    ) -> Result<(), String> {
        let store =
            Store::new().map_err(|e| format!("Failed to create case RDF store: {:?}", e))?;

        let mut case_stores = self.case_rdf_stores.write().await;
        case_stores.insert(case_id, store);
        Ok(())
    }

    /// Load pattern metadata into RDF store
    pub(crate) async fn load_pattern_metadata_rdf(&self) -> Result<(), String> {
        use crate::patterns::{get_all_pattern_metadata, serialize_metadata_to_rdf};

        let metadata = get_all_pattern_metadata();
        let store = self.pattern_metadata_store.write().await;

        for pattern_meta in metadata {
            let turtle = serialize_metadata_to_rdf(&pattern_meta)
                .map_err(|e| format!("Failed to serialize pattern metadata: {:?}", e))?;

            store
                .load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle.as_bytes())
                .map_err(|e| format!("Failed to load pattern metadata: {:?}", e))?;
        }

        Ok(())
    }
}
