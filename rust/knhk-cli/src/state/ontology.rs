//! Ontology management - Load, save, and merge O using Oxigraph

use super::store::StateStore;
use oxigraph::model::{Graph, NamedNode, Quad, TripleRef};
use std::sync::Arc;

/// Ontology loader - Loads O from Oxigraph
pub struct OntologyLoader {
    store: Arc<StateStore>,
}

impl OntologyLoader {
    /// Create new ontology loader
    pub fn new(store: StateStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// Load ontology O from Oxigraph
    pub fn load(&self) -> Result<Graph, String> {
        let mut graph = Graph::new();

        for quad_result in self.store.store().quads_for_pattern(None, None, None, None) {
            let quad = quad_result.map_err(|e| format!("Failed to query Oxigraph store: {}", e))?;
            // Convert Quad to TripleRef for Graph::insert
            let triple_ref = TripleRef::new(
                quad.subject.as_ref(),
                quad.predicate.as_ref(),
                quad.object.as_ref(),
            );
            graph.insert(triple_ref);
        }

        Ok(graph)
    }

    /// Load ontology O for a specific graph IRI
    pub fn load_for_graph(&self, graph_iri: &str) -> Result<Graph, String> {
        let graph_name =
            NamedNode::new(graph_iri).map_err(|e| format!("Invalid graph IRI: {}", e))?;

        let mut graph = Graph::new();
        let graph_name_enum = oxigraph::model::GraphName::NamedNode(graph_name);
        for quad_result in
            self.store
                .store()
                .quads_for_pattern(None, None, None, Some(graph_name_enum.as_ref()))
        {
            let quad = quad_result.map_err(|e| format!("Failed to query Oxigraph store: {}", e))?;
            let triple_ref = TripleRef::new(
                quad.subject.as_ref(),
                quad.predicate.as_ref(),
                quad.object.as_ref(),
            );
            graph.insert(triple_ref);
        }

        Ok(graph)
    }
}

/// Ontology saver - Saves O to Oxigraph
pub struct OntologySaver {
    store: Arc<StateStore>,
}

impl OntologySaver {
    /// Create new ontology saver
    pub fn new(store: StateStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// Save ontology O to Oxigraph
    pub fn save(&self, graph: &Graph, graph_iri: Option<&str>) -> Result<(), String> {
        let graph_name = graph_iri
            .map(NamedNode::new)
            .transpose()
            .map_err(|e| format!("Invalid graph IRI: {}", e))?;

        for quad in graph.iter() {
            let quad = if let Some(ref name) = graph_name {
                Quad::new(
                    quad.subject,
                    quad.predicate,
                    quad.object,
                    oxigraph::model::GraphName::NamedNode(name.clone()),
                )
            } else {
                Quad::new(
                    quad.subject,
                    quad.predicate,
                    quad.object,
                    oxigraph::model::GraphName::DefaultGraph,
                )
            };

            self.store
                .store()
                .insert(&quad)
                .map_err(|e| format!("Failed to insert quad into Oxigraph store: {}", e))?;
        }

        Ok(())
    }
}

/// Ontology merger - Merges Δ into O in Oxigraph
pub struct OntologyMerger {
    store: Arc<StateStore>,
}

impl OntologyMerger {
    /// Create new ontology merger
    pub fn new(store: StateStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// Merge delta Δ into ontology O
    pub fn merge(&self, delta: &Graph, graph_iri: Option<&str>) -> Result<(), String> {
        let graph_name = graph_iri
            .map(NamedNode::new)
            .transpose()
            .map_err(|e| format!("Invalid graph IRI: {}", e))?;

        for quad in delta.iter() {
            let quad = if let Some(ref name) = graph_name {
                Quad::new(
                    quad.subject,
                    quad.predicate,
                    quad.object,
                    oxigraph::model::GraphName::NamedNode(name.clone()),
                )
            } else {
                Quad::new(
                    quad.subject,
                    quad.predicate,
                    quad.object,
                    oxigraph::model::GraphName::DefaultGraph,
                )
            };

            self.store
                .store()
                .insert(&quad)
                .map_err(|e| format!("Failed to merge quad into Oxigraph store: {}", e))?;
        }

        Ok(())
    }
}
