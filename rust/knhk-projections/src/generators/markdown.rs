//! Markdown Generator (Π_docs)
//!
//! Generates user-facing documentation from ontology snapshots.

use crate::determinism::blake3_hash;
use crate::Result;
use knhk_ontology::SigmaSnapshot;
use std::collections::{HashMap, HashSet};
use tracing::instrument;

/// Output from markdown generation
#[derive(Clone, Debug)]
pub struct MarkdownOutput {
    /// Generated markdown documentation
    pub markdown: String,

    /// Section names
    pub sections: Vec<String>,

    /// Content hash (for determinism verification)
    pub hash: [u8; 32],
}

/// Generates markdown documentation from ontology snapshots
pub struct MarkdownGenerator;

impl MarkdownGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate markdown documentation from snapshot
    #[instrument(skip(self, snapshot))]
    pub async fn generate(&self, snapshot: &SigmaSnapshot) -> Result<MarkdownOutput> {
        let mut doc = String::new();
        let mut sections = Vec::new();

        // Title and overview
        doc.push_str("# KNHK Ontology Documentation\n\n");
        doc.push_str("*Auto-generated from ontology snapshot*\n\n");
        doc.push_str(&format!("**Snapshot ID**: `{:?}`\n\n", snapshot.id));
        doc.push_str(&format!("**Created**: {:?}\n\n", snapshot.metadata.created_at));
        doc.push_str(&format!("**Created By**: {}\n\n", snapshot.metadata.created_by));
        doc.push_str(&format!("**Description**: {}\n\n", snapshot.metadata.description));
        doc.push_str("---\n\n");

        sections.push("Overview".to_string());

        // Extract and document classes
        let classes = self.extract_classes_with_properties(snapshot)?;

        if !classes.is_empty() {
            doc.push_str("## Classes\n\n");
            sections.push("Classes".to_string());

            for (class_name, properties) in &classes {
                doc.push_str(&format!("### {}\n\n", class_name));

                if !properties.is_empty() {
                    doc.push_str("**Properties**:\n\n");
                    for prop in properties {
                        doc.push_str(&format!("- `{}`: {}\n", prop.0, prop.1));
                    }
                    doc.push('\n');
                }
            }
        }

        // Statistics section
        doc.push_str("## Statistics\n\n");
        sections.push("Statistics".to_string());

        let triple_count = snapshot.all_triples().len();
        doc.push_str(&format!("- **Total Triples**: {}\n", triple_count));
        doc.push_str(&format!("- **Total Classes**: {}\n", classes.len()));
        doc.push('\n');

        // Validation section
        if let Some(ref receipt) = snapshot.validation_receipt {
            doc.push_str("## Validation\n\n");
            sections.push("Validation".to_string());

            doc.push_str(&format!(
                "- **Production Ready**: {}\n",
                receipt.production_ready
            ));
            doc.push_str(&format!(
                "- **Validation Duration**: {} ms\n",
                receipt.validation_duration_ms
            ));
            doc.push('\n');
        }

        let hash = blake3_hash(doc.as_bytes());

        Ok(MarkdownOutput {
            markdown: doc,
            sections,
            hash,
        })
    }

    /// Extract classes and their properties from snapshot
    fn extract_classes_with_properties(
        &self,
        snapshot: &SigmaSnapshot,
    ) -> Result<HashMap<String, Vec<(String, String)>>> {
        let triples = snapshot.all_triples();
        let mut classes: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut subjects_by_type: HashMap<String, HashSet<String>> = HashMap::new();

        // First pass: map subjects to their types
        for triple in &triples {
            if triple.predicate == "rdf:type" {
                subjects_by_type
                    .entry(triple.object.clone())
                    .or_default()
                    .insert(triple.subject.clone());
            }
        }

        // Second pass: collect properties for each class
        for (class_name, subjects) in &subjects_by_type {
            let mut properties = Vec::new();

            for subject in subjects {
                for triple in &triples {
                    if &triple.subject == subject && triple.predicate != "rdf:type" {
                        properties.push((
                            triple.predicate.clone(),
                            triple.object.clone(),
                        ));
                    }
                }
            }

            // Deduplicate and sort properties
            properties.sort();
            properties.dedup_by(|a, b| a.0 == b.0);

            classes.insert(class_name.clone(), properties);
        }

        Ok(classes)
    }
}

impl Default for MarkdownGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_ontology::{SnapshotMetadata, Triple, TripleStore};

    #[tokio::test]
    async fn test_generate_markdown() {
        let mut store = TripleStore::new();
        store.add(Triple::new("company1", "rdf:type", "Company"));
        store.add(Triple::new("company1", "name", "TechCorp"));
        store.add(Triple::new("company1", "sector", "Technology"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default()).unwrap();

        let generator = MarkdownGenerator::new();
        let output = generator.generate(&snapshot).await.unwrap();

        assert!(!output.markdown.is_empty());
        assert!(output.markdown.contains("# KNHK Ontology Documentation"));
        assert!(output.markdown.contains("## Classes"));
        assert!(output.markdown.contains("### Company"));
        assert!(output.sections.contains(&"Overview".to_string()));
        assert!(output.sections.contains(&"Classes".to_string()));
    }
}
