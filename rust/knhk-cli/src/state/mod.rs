//! State management module - Manages O, Σ, Q using Oxigraph

pub mod invariant;
pub mod ontology;
pub mod schema;
pub mod store;

pub use invariant::InvariantLoader;
pub use ontology::{OntologyLoader, OntologyMerger, OntologySaver};
pub use schema::SchemaLoader;
pub use store::StateStore;

/// State manager - Coordinates all state operations
pub struct StateManager {
    store: StateStore,
}

impl StateManager {
    /// Create new state manager
    pub fn new() -> Result<Self, String> {
        let store = StateStore::new()?;
        Ok(Self { store })
    }

    /// Get ontology loader
    pub fn ontology_loader(&self) -> OntologyLoader {
        OntologyLoader::new(self.store.clone())
    }

    /// Get ontology saver
    pub fn ontology_saver(&self) -> OntologySaver {
        OntologySaver::new(self.store.clone())
    }

    /// Get ontology merger
    pub fn ontology_merger(&self) -> OntologyMerger {
        OntologyMerger::new(self.store.clone())
    }

    /// Get schema loader
    pub fn schema_loader(&self) -> SchemaLoader {
        SchemaLoader::new(self.store.clone())
    }

    /// Get invariant loader
    pub fn invariant_loader(&self) -> InvariantLoader {
        InvariantLoader::new(self.store.clone())
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create state manager")
    }
}
