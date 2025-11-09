//! Turtle/YAWL workflow parser
//!
//! Provides parsing of workflow specifications from RDF/Turtle format.

mod extractor;
mod types;

pub use extractor::*;
pub use types::*;

use crate::error::{WorkflowError, WorkflowResult};
use crate::validation::DeadlockDetector;
use oxigraph::io::RdfFormat;
use oxigraph::store::Store;
use std::io::Read;

/// Turtle/YAWL parser
pub struct WorkflowParser {
    store: Store,
    /// Deadlock detector for validation
    deadlock_detector: DeadlockDetector,
}

impl WorkflowParser {
    /// Create a new parser
    pub fn new() -> WorkflowResult<Self> {
        let store = Store::new()
            .map_err(|e| WorkflowError::Parse(format!("Failed to create RDF store: {:?}", e)))?;
        Ok(Self {
            store,
            deadlock_detector: DeadlockDetector,
        })
    }

    /// Parse workflow from Turtle string with deadlock validation
    pub fn parse_turtle(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // Parse Turtle into RDF store using oxigraph's built-in parser
        self.store
            .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
            .map_err(|e| WorkflowError::Parse(format!("Failed to load Turtle: {:?}", e)))?;

        // Extract workflow specification
        let mut spec = extractor::extract_workflow_spec(&self.store)?;

        // Store source turtle for runtime RDF queries
        spec.source_turtle = Some(turtle.to_string());

        // Validate for deadlocks
        self.deadlock_detector.validate(&spec)?;

        Ok(spec)
    }

    /// Parse workflow from JSON-LD string with deadlock validation
    pub fn parse_jsonld(&mut self, jsonld: &str) -> WorkflowResult<WorkflowSpec> {
        // Parse JSON-LD into RDF store using oxigraph's built-in parser
        // Use default JSON-LD profile (expanded)
        self.store
            .load_from_reader(
                RdfFormat::JsonLd {
                    profile: oxigraph::io::JsonLdProfile::Expanded.into(),
                },
                jsonld.as_bytes(),
            )
            .map_err(|e| WorkflowError::Parse(format!("Failed to load JSON-LD: {:?}", e)))?;

        // Extract workflow specification
        let spec = extractor::extract_workflow_spec(&self.store)?;

        // Validate for deadlocks
        self.deadlock_detector.validate(&spec)?;

        Ok(spec)
    }

    /// Parse workflow from JSON-LD file
    pub fn parse_jsonld_file(&mut self, path: &std::path::Path) -> WorkflowResult<WorkflowSpec> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| WorkflowError::Parse(format!("Failed to open file: {}", e)))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| WorkflowError::Parse(format!("Failed to read file: {}", e)))?;
        self.parse_jsonld(&contents)
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

    /// Export current RDF store as Turtle string
    pub fn export_turtle(&self) -> WorkflowResult<String> {
        use oxigraph::io::RdfSerializer;
        use oxigraph::model::GraphNameRef;

        let mut buffer = Vec::new();
        self.store
            .dump_graph_to_writer(
                GraphNameRef::DefaultGraph,
                RdfSerializer::from_format(RdfFormat::Turtle),
                &mut buffer,
            )
            .map_err(|e| {
                WorkflowError::Internal(format!("Failed to export RDF store as Turtle: {:?}", e))
            })?;

        String::from_utf8(buffer).map_err(|e| {
            WorkflowError::Internal(format!("Failed to convert Turtle to UTF-8: {}", e))
        })
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
