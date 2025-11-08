//! Schema management - Load Σ from Oxigraph

use super::store::StateStore;
use oxigraph::model::{Graph, NamedNode, TripleRef};
use std::sync::Arc;

/// Schema loader - Loads Σ from Oxigraph
pub struct SchemaLoader {
    store: Arc<StateStore>,
}

impl SchemaLoader {
    /// Create new schema loader
    pub fn new(store: StateStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// Load schema Σ from Oxigraph
    pub fn load(&self, schema_iri: &str) -> Result<Graph, String> {
        let graph_name =
            NamedNode::new(schema_iri).map_err(|e| format!("Invalid schema IRI: {}", e))?;

        let mut graph = Graph::new();
        let graph_name_enum = oxigraph::model::GraphName::NamedNode(graph_name);
        for quad_result in
            self.store
                .store()
                .quads_for_pattern(None, None, None, Some(graph_name_enum.as_ref()))
        {
            let quad = quad_result
                .map_err(|e| format!("Failed to query Oxigraph store for schema: {}", e))?;
            let triple_ref = TripleRef::new(
                quad.subject.as_ref(),
                quad.predicate.as_ref(),
                quad.object.as_ref(),
            );
            graph.insert(triple_ref);
        }

        Ok(graph)
    }

    /// Check if schema Σ exists
    pub fn exists(&self, schema_iri: &str) -> Result<bool, String> {
        let graph_name =
            NamedNode::new(schema_iri).map_err(|e| format!("Invalid schema IRI: {}", e))?;

        let mut count = 0;
        let graph_name_enum = oxigraph::model::GraphName::NamedNode(graph_name);
        for quad_result in
            self.store
                .store()
                .quads_for_pattern(None, None, None, Some(graph_name_enum.as_ref()))
        {
            quad_result.map_err(|e| format!("Failed to query Oxigraph store: {}", e))?;
            count += 1;
            if count > 0 {
                break;
            }
        }

        Ok(count > 0)
    }
}
