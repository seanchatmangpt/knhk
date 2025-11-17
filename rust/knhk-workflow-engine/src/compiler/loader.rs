//! Turtle RDF Loader
//!
//! Loads and parses Turtle files into RDF triple stores.
//! Validates against ontology and prepares for extraction.

use crate::error::{WorkflowError, WorkflowResult};
use oxigraph::{
    io::{RdfFormat, RdfParser},
    model::{GraphName, NamedNode, NamedOrBlankNode, Quad, Term},
    store::Store,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, info, instrument, warn};

/// Ontology namespaces
#[derive(Debug, Clone)]
pub struct Namespaces {
    /// YAWL namespace
    pub yawl: String,
    /// FIBO namespace
    pub fibo: String,
    /// SHACL namespace
    pub shacl: String,
    /// OWL-Time namespace
    pub time: String,
    /// RDF namespace
    pub rdf: String,
    /// RDFS namespace
    pub rdfs: String,
}

impl Default for Namespaces {
    fn default() -> Self {
        Self {
            yawl: "http://bitflow.ai/ontology/yawl/v2#".to_string(),
            fibo: "https://spec.edmcouncil.org/fibo/ontology/".to_string(),
            shacl: "http://www.w3.org/ns/shacl#".to_string(),
            time: "http://www.w3.org/2006/time#".to_string(),
            rdf: "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
            rdfs: "http://www.w3.org/2000/01/rdf-schema#".to_string(),
        }
    }
}

/// Turtle file loader
pub struct TurtleLoader {
    namespaces: Namespaces,
    validate_ontology: bool,
    max_file_size: usize,
}

impl TurtleLoader {
    /// Create new loader
    pub fn new() -> Self {
        Self {
            namespaces: Namespaces::default(),
            validate_ontology: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
        }
    }

    /// Load Turtle file into RDF store
    #[instrument(skip(self))]
    pub async fn load_turtle(&self, path: &Path) -> WorkflowResult<Store> {
        // Check file exists and size
        let metadata = fs::metadata(path)
            .map_err(|e| WorkflowError::Io(format!("Cannot read file metadata: {}", e)))?;

        if metadata.len() as usize > self.max_file_size {
            return Err(WorkflowError::Parse(format!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                self.max_file_size
            )));
        }

        info!("Loading Turtle file: {:?} ({} bytes)", path, metadata.len());

        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| WorkflowError::Io(format!("Cannot read file: {}", e)))?;

        // Parse Turtle into store
        let store = self.parse_turtle(&content).await?;

        // Validate ontology if enabled
        if self.validate_ontology {
            self.validate_ontology_structure(&store).await?;
        }

        Ok(store)
    }

    /// Load multiple Turtle files into a single store
    pub async fn load_multiple(&self, paths: &[&Path]) -> WorkflowResult<Store> {
        let mut combined_store = Store::new()
            .map_err(|e| WorkflowError::Internal(format!("Cannot create store: {}", e)))?;

        for path in paths {
            let store = self.load_turtle(path).await?;
            self.merge_stores(&mut combined_store, &store)?;
        }

        Ok(combined_store)
    }

    /// Parse Turtle string into store
    async fn parse_turtle(&self, content: &str) -> WorkflowResult<Store> {
        let store = Store::new()
            .map_err(|e| WorkflowError::Internal(format!("Cannot create store: {}", e)))?;

        // Parse Turtle content
        let parser = RdfParser::from_format(RdfFormat::Turtle)
            .with_base_iri(&self.namespaces.yawl)
            .map_err(|e| WorkflowError::Parse(format!("Invalid base IRI: {}", e)))?;

        let quads = parser
            .parse_from_read(content.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| WorkflowError::Parse(format!("Failed to parse Turtle: {}", e)))?;

        // Add quads to store
        for quad in quads {
            store
                .insert(&quad)
                .map_err(|e| WorkflowError::Internal(format!("Failed to insert quad: {}", e)))?;
        }

        debug!("Parsed {} quads", store.len());
        Ok(store)
    }

    /// Validate ontology structure
    async fn validate_ontology_structure(&self, store: &Store) -> WorkflowResult<()> {
        // Check for required namespaces
        let required_prefixes = vec![
            ("yawl", &self.namespaces.yawl),
            ("rdf", &self.namespaces.rdf),
            ("rdfs", &self.namespaces.rdfs),
        ];

        for (prefix, namespace) in required_prefixes {
            if !self.has_namespace(store, namespace) {
                warn!("Missing expected namespace: {} ({})", prefix, namespace);
            }
        }

        // Check for workflow specification
        if !self.has_workflow_spec(store)? {
            return Err(WorkflowError::Validation(
                "No workflow specification found in Turtle file".to_string(),
            ));
        }

        // Check for basic YAWL elements
        self.validate_yawl_elements(store)?;

        Ok(())
    }

    /// Check if store contains namespace
    fn has_namespace(&self, store: &Store, namespace: &str) -> bool {
        let namespace_iri = NamedNode::new(namespace).ok();

        if let Some(iri) = namespace_iri {
            // Check if any triple uses this namespace
            for quad in store.iter() {
                if let Ok(quad) = quad {
                    // Check subject
                    if let NamedOrBlankNode::NamedNode(node) = &quad.subject {
                        if node.as_str().starts_with(namespace) {
                            return true;
                        }
                    }
                    // Check predicate
                    if quad.predicate.as_str().starts_with(namespace) {
                        return true;
                    }
                    // Check object
                    if let Term::NamedNode(node) = &quad.object {
                        if node.as_str().starts_with(namespace) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if store has workflow specification
    fn has_workflow_spec(&self, store: &Store) -> WorkflowResult<bool> {
        use oxigraph::sparql::QueryResults;

        let query = format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <{}>\n\
             ASK {{\n\
               {{ ?spec rdf:type yawl:Specification }}\n\
               UNION\n\
               {{ ?spec rdf:type yawl:WorkflowSpecification }}\n\
             }}",
            self.namespaces.yawl, self.namespaces.rdf
        );

        let results = store
            .query(&query)
            .map_err(|e| WorkflowError::Parse(format!("SPARQL query failed: {}", e)))?;

        match results {
            QueryResults::Boolean(has_spec) => Ok(has_spec),
            _ => Ok(false),
        }
    }

    /// Validate YAWL elements
    fn validate_yawl_elements(&self, store: &Store) -> WorkflowResult<()> {
        // Check for tasks
        let task_count = self.count_elements(store, "Task")?;
        if task_count == 0 {
            return Err(WorkflowError::Validation(
                "No tasks found in workflow".to_string(),
            ));
        }
        debug!("Found {} tasks", task_count);

        // Check for conditions
        let condition_count = self.count_elements(store, "Condition")?;
        debug!("Found {} conditions", condition_count);

        // Check for flows
        let flow_count = self.count_elements(store, "Flow")?;
        if flow_count == 0 {
            warn!("No explicit flows found (may be implicit)");
        } else {
            debug!("Found {} flows", flow_count);
        }

        Ok(())
    }

    /// Count elements of a given type
    fn count_elements(&self, store: &Store, element_type: &str) -> WorkflowResult<usize> {
        use oxigraph::sparql::QueryResults;

        let query = format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdf: <{}>\n\
             SELECT (COUNT(?elem) as ?count) WHERE {{\n\
               ?elem rdf:type yawl:{}\n\
             }}",
            self.namespaces.yawl, self.namespaces.rdf, element_type
        );

        let results = store
            .query(&query)
            .map_err(|e| WorkflowError::Parse(format!("SPARQL query failed: {}", e)))?;

        if let QueryResults::Solutions(mut solutions) = results {
            if let Some(Ok(solution)) = solutions.next() {
                if let Some(Term::Literal(lit)) = solution.get("count") {
                    return lit
                        .value()
                        .parse::<usize>()
                        .map_err(|_| WorkflowError::Parse("Invalid count".to_string()));
                }
            }
        }

        Ok(0)
    }

    /// Merge stores
    fn merge_stores(&self, target: &mut Store, source: &Store) -> WorkflowResult<()> {
        for quad in source.iter() {
            let quad =
                quad.map_err(|e| WorkflowError::Internal(format!("Failed to iterate: {}", e)))?;
            target
                .insert(&quad)
                .map_err(|e| WorkflowError::Internal(format!("Failed to merge: {}", e)))?;
        }
        Ok(())
    }

    /// Compute hash of store content
    pub fn compute_source_hash(&self, store: &Store) -> WorkflowResult<[u8; 32]> {
        let mut hasher = Sha256::new();

        // Sort quads for deterministic hashing
        let mut quads: Vec<Quad> = Vec::new();
        for quad in store.iter() {
            let quad =
                quad.map_err(|e| WorkflowError::Internal(format!("Failed to iterate: {}", e)))?;
            quads.push(quad);
        }

        // Sort by string representation for determinism
        quads.sort_by_key(|q| {
            format!(
                "{} {} {} {}",
                q.subject, q.predicate, q.object, q.graph_name
            )
        });

        // Hash sorted quads
        for quad in quads {
            hasher.update(quad.subject.to_string().as_bytes());
            hasher.update(quad.predicate.to_string().as_bytes());
            hasher.update(quad.object.to_string().as_bytes());
            hasher.update(quad.graph_name.to_string().as_bytes());
        }

        Ok(hasher.finalize().into())
    }

    /// Load pattern matrix for validation
    pub async fn load_pattern_matrix(&self, path: &Path) -> WorkflowResult<Store> {
        info!("Loading pattern matrix from: {:?}", path);
        self.load_turtle(path).await
    }

    /// Extract workflow metadata
    pub fn extract_metadata(&self, store: &Store) -> WorkflowResult<WorkflowMetadata> {
        use oxigraph::sparql::QueryResults;

        let query = format!(
            "PREFIX yawl: <{}>\n\
             PREFIX rdfs: <{}>\n\
             PREFIX rdf: <{}>\n\
             SELECT ?spec ?name ?description ?version WHERE {{\n\
               {{ ?spec rdf:type yawl:Specification }}\n\
               UNION\n\
               {{ ?spec rdf:type yawl:WorkflowSpecification }}\n\
               OPTIONAL {{ ?spec rdfs:label ?name }}\n\
               OPTIONAL {{ ?spec rdfs:comment ?description }}\n\
               OPTIONAL {{ ?spec yawl:version ?version }}\n\
             }} LIMIT 1",
            self.namespaces.yawl, self.namespaces.rdfs, self.namespaces.rdf
        );

        let results = store
            .query(&query)
            .map_err(|e| WorkflowError::Parse(format!("Metadata query failed: {}", e)))?;

        let mut metadata = WorkflowMetadata::default();

        if let QueryResults::Solutions(mut solutions) = results {
            if let Some(Ok(solution)) = solutions.next() {
                if let Some(term) = solution.get("spec") {
                    metadata.spec_iri = term.to_string();
                }
                if let Some(Term::Literal(lit)) = solution.get("name") {
                    metadata.name = lit.value().to_string();
                }
                if let Some(Term::Literal(lit)) = solution.get("description") {
                    metadata.description = Some(lit.value().to_string());
                }
                if let Some(Term::Literal(lit)) = solution.get("version") {
                    metadata.version = Some(lit.value().to_string());
                }
            }
        }

        Ok(metadata)
    }
}

/// Workflow metadata
#[derive(Debug, Clone, Default)]
pub struct WorkflowMetadata {
    /// Specification IRI
    pub spec_iri: String,
    /// Workflow name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version
    pub version: Option<String>,
}

impl Default for TurtleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_loader_creation() {
        let loader = TurtleLoader::new();
        assert!(loader.validate_ontology);
        assert_eq!(loader.max_file_size, 100 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_parse_simple_turtle() {
        let loader = TurtleLoader::new();

        let turtle = r#"
            @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

            :spec1 rdf:type yawl:Specification .
            :task1 rdf:type yawl:Task .
        "#;

        let store = loader.parse_turtle(turtle).await.unwrap();
        assert!(store.len() > 0);
    }

    #[tokio::test]
    async fn test_load_from_file() {
        let loader = TurtleLoader::new();

        // Create temp file with Turtle content
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> ."
        )
        .unwrap();
        writeln!(
            file,
            "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> ."
        )
        .unwrap();
        writeln!(file, ":spec1 rdf:type yawl:Specification .").unwrap();
        writeln!(file, ":task1 rdf:type yawl:Task .").unwrap();
        file.flush().unwrap();

        let store = loader.load_turtle(file.path()).await.unwrap();
        assert!(store.len() > 0);
    }

    #[tokio::test]
    async fn test_hash_determinism() {
        let loader = TurtleLoader::new();

        let turtle = r#"
            @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
            :task1 rdf:type yawl:Task .
            :task2 rdf:type yawl:Task .
        "#;

        let store1 = loader.parse_turtle(turtle).await.unwrap();
        let hash1 = loader.compute_source_hash(&store1).unwrap();

        let store2 = loader.parse_turtle(turtle).await.unwrap();
        let hash2 = loader.compute_source_hash(&store2).unwrap();

        assert_eq!(hash1, hash2, "Hashing should be deterministic");
    }
}
