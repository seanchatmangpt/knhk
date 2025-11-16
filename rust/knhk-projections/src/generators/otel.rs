//! OpenTelemetry Generator (Π_telemetry)
//!
//! Generates OpenTelemetry semantic conventions from ontology snapshots.

use crate::determinism::blake3_hash;
use crate::Result;
use knhk_ontology::SigmaSnapshot;
use std::collections::HashSet;
use tracing::instrument;

/// Output from OTEL generation
#[derive(Clone, Debug)]
pub struct OtelOutput {
    /// YAML/Weaver schema format
    pub otel_schema: String,

    /// Span names
    pub spans: Vec<String>,

    /// Metric names
    pub metrics: Vec<String>,

    /// Content hash (for determinism verification)
    pub hash: [u8; 32],
}

/// Generates OpenTelemetry schemas from ontology snapshots
pub struct OtelGenerator;

impl OtelGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate OTEL schema from snapshot
    #[instrument(skip(self, snapshot))]
    pub async fn generate(&self, snapshot: &SigmaSnapshot) -> Result<OtelOutput> {
        let mut schema = String::new();
        let mut spans = Vec::new();
        let mut metrics = Vec::new();

        schema.push_str("# AUTO-GENERATED OpenTelemetry Schema\n");
        schema.push_str(&format!("# Snapshot ID: {:?}\n", snapshot.id));
        schema.push_str(&format!("# Generated: {:?}\n\n", snapshot.metadata.created_at));

        schema.push_str("schema_url: https://knhk.io/schemas/1.0.0\n\n");

        schema.push_str("resource_spans:\n");

        // Extract classes and generate spans
        let classes = self.extract_classes(snapshot)?;

        for class in &classes {
            let span_name = format!("{}.process", class.to_lowercase());
            spans.push(span_name.clone());

            schema.push_str(&format!("  - name: {}\n", span_name));
            schema.push_str("    span_kind: INTERNAL\n");
            schema.push_str("    attributes:\n");
            schema.push_str(&format!("      - name: {}.id\n", class.to_lowercase()));
            schema.push_str("        type: string\n");
            schema.push_str("        requirement_level: required\n");
            schema.push_str(&format!("        brief: Unique identifier for {}\n", class));
            schema.push('\n');

            // Add metric for operation count
            let metric_name = format!("{}.operations", class.to_lowercase());
            metrics.push(metric_name.clone());
        }

        // Add metrics section
        schema.push_str("metrics:\n");
        for metric in &metrics {
            schema.push_str(&format!("  - name: {}\n", metric));
            schema.push_str("    unit: 1\n");
            schema.push_str("    instrument: counter\n");
            schema.push_str("    description: Count of operations\n\n");
        }

        let hash = blake3_hash(schema.as_bytes());

        Ok(OtelOutput {
            otel_schema: schema,
            spans,
            metrics,
            hash,
        })
    }

    /// Extract unique classes from snapshot
    fn extract_classes(&self, snapshot: &SigmaSnapshot) -> Result<Vec<String>> {
        let triples = snapshot.all_triples();
        let mut classes = HashSet::new();

        for triple in &triples {
            if triple.predicate == "rdf:type" {
                classes.insert(triple.object.clone());
            }
        }

        // If no explicit types, use generic "Resource"
        if classes.is_empty() {
            classes.insert("Resource".to_string());
        }

        let mut sorted: Vec<_> = classes.into_iter().collect();
        sorted.sort();
        Ok(sorted)
    }
}

impl Default for OtelGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_ontology::{SnapshotMetadata, Triple, TripleStore};

    #[tokio::test]
    async fn test_generate_otel_schema() {
        let mut store = TripleStore::new();
        store.add(Triple::new("company1", "rdf:type", "Company"));
        store.add(Triple::new("company1", "name", "TechCorp"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default()).unwrap();

        let generator = OtelGenerator::new();
        let output = generator.generate(&snapshot).await.unwrap();

        assert!(!output.otel_schema.is_empty());
        assert!(output.otel_schema.contains("schema_url"));
        assert!(output.otel_schema.contains("resource_spans"));
        assert!(output.spans.contains(&"company.process".to_string()));
        assert!(output.metrics.contains(&"company.operations".to_string()));
    }
}
