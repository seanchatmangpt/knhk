//! Invariant management - Load Q from Oxigraph

use super::store::StateStore;
use oxigraph::model::{Graph, NamedNode, TripleRef};
use std::sync::Arc;

/// Invariant loader - Loads Q from Oxigraph
pub struct InvariantLoader {
    store: Arc<StateStore>,
}

impl InvariantLoader {
    /// Create new invariant loader
    pub fn new(store: StateStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// Load invariants Q from Oxigraph
    pub fn load(&self, invariant_iri: &str) -> Result<Graph, String> {
        let graph_name =
            NamedNode::new(invariant_iri).map_err(|e| format!("Invalid invariant IRI: {}", e))?;

        let mut graph = Graph::new();
        let graph_name_enum = oxigraph::model::GraphName::NamedNode(graph_name);
        for quad_result in
            self.store
                .store()
                .quads_for_pattern(None, None, None, Some(graph_name_enum.as_ref()))
        {
            let quad = quad_result
                .map_err(|e| format!("Failed to query Oxigraph store for invariants: {}", e))?;
            let triple_ref = TripleRef::new(
                quad.subject.as_ref(),
                quad.predicate.as_ref(),
                quad.object.as_ref(),
            );
            graph.insert(triple_ref);
        }

        Ok(graph)
    }

    /// Check if invariants Q exist
    pub fn exists(&self, invariant_iri: &str) -> Result<bool, String> {
        let graph_name =
            NamedNode::new(invariant_iri).map_err(|e| format!("Invalid invariant IRI: {}", e))?;

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
