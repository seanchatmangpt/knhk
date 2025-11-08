//! Turtle/YAWL workflow parser
//!
//! Provides parsing of workflow specifications from RDF/Turtle format.

mod extractor;
mod types;

pub use extractor::*;
pub use types::*;

use crate::error::{WorkflowError, WorkflowResult};
use oxigraph::io::RdfFormat;
use oxigraph::store::Store;
use std::io::Read;

/// Turtle/YAWL parser
pub struct WorkflowParser {
    store: Store,
}

impl WorkflowParser {
    /// Create a new parser
    pub fn new() -> WorkflowResult<Self> {
        let store = Store::new()
            .map_err(|e| WorkflowError::Parse(format!("Failed to create RDF store: {:?}", e)))?;
        Ok(Self { store })
    }

    /// Parse workflow from Turtle string
    pub fn parse_turtle(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // Parse Turtle into RDF store using oxigraph's built-in parser
        self.store
            .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| WorkflowError::Parse(format!("Failed to load Turtle: {:?}", e)))?;

        // Extract workflow specification
        extractor::extract_workflow_spec(&self.store)
    }

    /// Parse workflow from file
    pub fn parse_file(&mut self, path: &std::path::Path) -> WorkflowResult<WorkflowSpec> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| WorkflowError::Parse(format!("Failed to open file: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| WorkflowError::Parse(format!("Failed to read file: {}", e)))?;
        self.parse_turtle(&contents)
    }

    /// Load YAWL ontology
    pub fn load_yawl_ontology(&mut self, ontology_path: &std::path::Path) -> WorkflowResult<()> {
        let mut file = std::fs::File::open(ontology_path)
            .map_err(|e| WorkflowError::Parse(format!("Failed to open ontology: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| WorkflowError::Parse(format!("Failed to read ontology: {}", e)))?;

        // Parse Turtle and load into store
        self.store
            .load_from_reader(RdfFormat::Turtle, contents.as_bytes())
            .map_err(|e| WorkflowError::Parse(format!("Failed to load Turtle: {:?}", e)))?;

        Ok(())
    }
}

impl Default for WorkflowParser {
    fn default() -> Self {
        // Default implementation should not fail
        // If new() fails, we'll panic as this is a programming error
        // FUTURE: Consider making Default return Result or use a static parser
        Self::new().unwrap_or_else(|e| panic!("Failed to create workflow parser: {:?}", e))
    }
}
