// rust/knhk-aot/src/template.rs
// CONSTRUCT template analysis for AOT optimization

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct TemplateTriple {
    pub subject: TripleComponent,
    pub predicate: TripleComponent,
    pub object: TripleComponent,
}

#[derive(Debug, Clone)]
pub enum TripleComponent {
    Constant(u64),      // Hashed IRI or literal
    Variable(String),  // Variable name (e.g., "?x", "?name")
}

#[derive(Debug, Clone)]
pub struct ConstructTemplate {
    pub ground_triples: Vec<(u64, u64, u64)>,  // (s, p, o) constants - no variables
    pub template_triples: Vec<TemplateTriple>, // Variable triples
    pub variable_bindings: BTreeMap<String, usize>, // Variable name -> binding index
}

impl ConstructTemplate {
    pub fn new() -> Self {
        Self {
            ground_triples: Vec::new(),
            template_triples: Vec::new(),
            variable_bindings: BTreeMap::new(),
        }
    }

    /// Analyze CONSTRUCT template from SPARQL query
    /// Extracts constants vs variables, identifies ground triples
    pub fn analyze(template_str: &str, where_clause: &str) -> Result<Self, String> {
        let mut template = Self::new();

        // Parse template triples
        // Note: Full SPARQL parser integration planned for v1.0
        let triples = Self::parse_template_triples(template_str)?;

        for triple in triples {
            let is_ground = matches!(&triple.subject, TripleComponent::Constant(_)) &&
                           matches!(&triple.predicate, TripleComponent::Constant(_)) &&
                           matches!(&triple.object, TripleComponent::Constant(_));

            if is_ground {
                // Extract constants
                if let (TripleComponent::Constant(s), 
                       TripleComponent::Constant(p), 
                       TripleComponent::Constant(o)) = 
                    (triple.subject, triple.predicate, triple.object) {
                    template.ground_triples.push((s, p, o));
                }
            } else {
                template.template_triples.push(triple);
            }
        }

        // Analyze WHERE clause for variable bindings
        template.variable_bindings = Self::extract_variable_bindings(where_clause)?;

        Ok(template)
    }

    fn parse_template_triples(_template_str: &str) -> Result<Vec<TemplateTriple>, String> {
        // Note: Full SPARQL parser integration planned for v1.0
        // For now, return empty vector (basic structure only)
        Ok(Vec::new())
    }

    fn extract_variable_bindings(_where_clause: &str) -> Result<BTreeMap<String, usize>, String> {
        // Extract variable bindings from WHERE clause
        // For now, return empty map
        Ok(BTreeMap::new())
    }

    /// Check if template has only constants (ground triples)
    pub fn is_all_ground(&self) -> bool {
        self.template_triples.is_empty()
    }

    /// Get number of variable triples
    pub fn variable_triple_count(&self) -> usize {
        self.template_triples.len()
    }

    /// Get number of ground triples
    pub fn ground_triple_count(&self) -> usize {
        self.ground_triples.len()
    }
}

impl Default for ConstructTemplate {
    fn default() -> Self {
        Self::new()
    }
}

