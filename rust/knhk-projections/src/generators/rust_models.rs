//! Rust Models Generator (Π_models)
//!
//! Generates type-safe Rust structs and enums from RDF classes in snapshots.

use crate::determinism::blake3_hash;
use crate::Result;
use knhk_ontology::SigmaSnapshot;
use std::collections::{HashMap, HashSet};
use tracing::instrument;

/// Output from Rust models generation
#[derive(Clone, Debug)]
pub struct RustModelsOutput {
    /// Generated Rust code
    pub models_code: String,

    /// Content hash (for determinism verification)
    pub hash: [u8; 32],
}

/// Generates Rust models from ontology snapshots
pub struct RustModelsGenerator;

impl RustModelsGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate Rust models from snapshot
    #[instrument(skip(self, snapshot))]
    pub async fn generate(&self, snapshot: &SigmaSnapshot) -> Result<RustModelsOutput> {
        let mut code = String::new();

        // Header
        code.push_str("// AUTO-GENERATED from Σ snapshot\n");
        code.push_str("// Do NOT edit manually\n");
        code.push_str("//\n");
        code.push_str(&format!("// Snapshot ID: {:?}\n", snapshot.id));
        code.push_str(&format!("// Generated: {:?}\n", snapshot.metadata.created_at));
        code.push('\n');
        code.push_str("use serde::{Deserialize, Serialize};\n");
        code.push_str("use chrono::{DateTime, Utc};\n");
        code.push('\n');

        // Extract classes from RDF triples
        let classes = self.extract_classes(snapshot)?;

        // Generate struct for each class
        for (class_name, properties) in &classes {
            code.push_str(&self.generate_struct(class_name, properties)?);
            code.push('\n');
        }

        // Compute hash
        let hash = blake3_hash(code.as_bytes());

        Ok(RustModelsOutput { models_code: code, hash })
    }

    /// Extract classes and their properties from triples
    fn extract_classes(&self, snapshot: &SigmaSnapshot) -> Result<HashMap<String, Vec<Property>>> {
        let triples = snapshot.all_triples();
        let mut classes: HashMap<String, Vec<Property>> = HashMap::new();
        let mut class_types: HashSet<String> = HashSet::new();

        // First pass: identify classes (subjects with rdf:type)
        for triple in &triples {
            if triple.predicate == "rdf:type" {
                class_types.insert(triple.object.clone());
            }
        }

        // Second pass: extract properties for each subject
        let mut subject_properties: HashMap<String, Vec<Property>> = HashMap::new();

        for triple in &triples {
            if triple.predicate != "rdf:type" {
                let prop = Property {
                    name: triple.predicate.clone(),
                    rust_type: self.infer_rust_type(&triple.object),
                };

                subject_properties
                    .entry(triple.subject.clone())
                    .or_default()
                    .push(prop);
            }
        }

        // Third pass: group by class type
        for triple in &triples {
            if triple.predicate == "rdf:type" {
                let class_name = &triple.object;
                if let Some(props) = subject_properties.get(&triple.subject) {
                    classes
                        .entry(class_name.clone())
                        .or_default()
                        .extend(props.iter().cloned());
                }
            }
        }

        // Deduplicate properties per class
        for props in classes.values_mut() {
            props.sort_by(|a, b| a.name.cmp(&b.name));
            props.dedup_by(|a, b| a.name == b.name);
        }

        // If no explicit classes found, create generic models from unique predicates
        if classes.is_empty() {
            let mut generic_props: HashMap<String, HashSet<Property>> = HashMap::new();

            for triple in &triples {
                let prop = Property {
                    name: triple.predicate.clone(),
                    rust_type: self.infer_rust_type(&triple.object),
                };
                generic_props
                    .entry("GenericModel".to_string())
                    .or_default()
                    .insert(prop);
            }

            for (class_name, props) in generic_props {
                classes.insert(class_name, props.into_iter().collect());
            }
        }

        Ok(classes)
    }

    /// Generate Rust struct for a class
    fn generate_struct(&self, class_name: &str, properties: &[Property]) -> Result<String> {
        let mut code = String::new();

        // Sanitize class name for Rust
        let struct_name = self.sanitize_identifier(class_name);

        code.push_str(&format!("/// {} model\n", struct_name));
        code.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        code.push_str(&format!("pub struct {} {{\n", struct_name));

        for prop in properties {
            let field_name = self.sanitize_identifier(&prop.name);
            code.push_str(&format!("    pub {}: {},\n", field_name, prop.rust_type));
        }

        code.push_str("}\n");

        Ok(code)
    }

    /// Infer Rust type from RDF object value
    fn infer_rust_type(&self, value: &str) -> String {
        // Try to infer type from value
        if value.parse::<i64>().is_ok() {
            "i64".to_string()
        } else if value.parse::<f64>().is_ok() {
            "f64".to_string()
        } else if value == "true" || value == "false" {
            "bool".to_string()
        } else {
            // Default to String
            "String".to_string()
        }
    }

    /// Sanitize identifier to be valid Rust
    fn sanitize_identifier(&self, name: &str) -> String {
        // Replace special characters with underscores
        let sanitized: String = name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect();

        // Ensure it starts with a letter or underscore
        if sanitized.chars().next().is_some_and(|c| c.is_numeric()) {
            format!("_{}", sanitized)
        } else {
            sanitized
        }
    }
}

impl Default for RustModelsGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Property {
    name: String,
    rust_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_ontology::{SnapshotMetadata, Triple, TripleStore};

    #[tokio::test]
    async fn test_generate_rust_models() {
        let mut store = TripleStore::new();
        store.add(Triple::new("company1", "rdf:type", "Company"));
        store.add(Triple::new("company1", "name", "TechCorp"));
        store.add(Triple::new("company1", "sector", "Technology"));
        store.add(Triple::new("company1", "employees", "1000"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default()).unwrap();

        let generator = RustModelsGenerator::new();
        let output = generator.generate(&snapshot).await.unwrap();

        assert!(!output.models_code.is_empty());
        assert!(output.models_code.contains("struct Company"));
        assert!(output.models_code.contains("pub name: String"));
        assert!(output.models_code.contains("pub sector: String"));
        assert!(output.models_code.contains("pub employees: i64"));
    }

    #[test]
    fn test_sanitize_identifier() {
        let gen = RustModelsGenerator::new();

        assert_eq!(gen.sanitize_identifier("valid_name"), "valid_name");
        assert_eq!(gen.sanitize_identifier("with-dashes"), "with_dashes");
        assert_eq!(gen.sanitize_identifier("with.dots"), "with_dots");
        assert_eq!(gen.sanitize_identifier("123invalid"), "_123invalid");
    }

    #[test]
    fn test_infer_rust_type() {
        let gen = RustModelsGenerator::new();

        assert_eq!(gen.infer_rust_type("123"), "i64");
        assert_eq!(gen.infer_rust_type("123.45"), "f64");
        assert_eq!(gen.infer_rust_type("true"), "bool");
        assert_eq!(gen.infer_rust_type("false"), "bool");
        assert_eq!(gen.infer_rust_type("text"), "String");
    }
}
