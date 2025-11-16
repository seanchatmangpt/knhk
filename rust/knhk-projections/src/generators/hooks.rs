//! Hooks Generator (Π_hooks)
//!
//! Generates KNHK hook configurations for guards and operators.

use crate::determinism::blake3_hash;
use crate::Result;
use knhk_ontology::SigmaSnapshot;
use tracing::instrument;

/// Output from hooks generation
#[derive(Clone, Debug)]
pub struct HooksOutput {
    /// YAML/TOML hook definitions
    pub hooks_config: String,

    /// Operator names
    pub operators: Vec<String>,

    /// Guard implementations
    pub guards: Vec<String>,

    /// Content hash (for determinism verification)
    pub hash: [u8; 32],
}

/// Generates hook configurations from ontology snapshots
pub struct HooksGenerator;

impl HooksGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate hook configurations from snapshot
    #[instrument(skip(self, snapshot))]
    pub async fn generate(&self, snapshot: &SigmaSnapshot) -> Result<HooksOutput> {
        let mut config = String::new();
        let mut operators = Vec::new();
        let mut guards = Vec::new();

        config.push_str("# AUTO-GENERATED KNHK Hooks Configuration\n");
        config.push_str(&format!("# Snapshot ID: {:?}\n", snapshot.id));
        config.push_str(&format!("# Generated: {:?}\n\n", snapshot.metadata.created_at));

        // Extract guards and operators from snapshot metadata
        // For now, generate default validation hooks

        config.push_str("[hooks]\n");
        config.push_str("enabled = true\n\n");

        // Pre-task hooks
        config.push_str("[[hooks.pre_task]]\n");
        config.push_str("name = \"validate-snapshot\"\n");
        config.push_str("operator = \"validate\"\n");
        config.push_str("description = \"Validate ontology snapshot before processing\"\n\n");

        operators.push("validate".to_string());
        guards.push("validate-snapshot".to_string());

        // Post-task hooks
        config.push_str("[[hooks.post_task]]\n");
        config.push_str("name = \"verify-invariants\"\n");
        config.push_str("operator = \"verify-q\"\n");
        config.push_str("description = \"Verify invariants Q are preserved\"\n\n");

        operators.push("verify-q".to_string());
        guards.push("verify-invariants".to_string());

        // Performance monitoring hooks
        config.push_str("[[hooks.post_task]]\n");
        config.push_str("name = \"check-performance\"\n");
        config.push_str("operator = \"perf-check\"\n");
        config.push_str("description = \"Ensure hot path operations ≤8 ticks\"\n");
        config.push_str("max_ticks = 8\n\n");

        operators.push("perf-check".to_string());
        guards.push("check-performance".to_string());

        let hash = blake3_hash(config.as_bytes());

        Ok(HooksOutput {
            hooks_config: config,
            operators,
            guards,
            hash,
        })
    }
}

impl Default for HooksGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_ontology::{SnapshotMetadata, Triple, TripleStore};

    #[tokio::test]
    async fn test_generate_hooks() {
        let mut store = TripleStore::new();
        store.add(Triple::new("data", "property", "value"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default()).unwrap();

        let generator = HooksGenerator::new();
        let output = generator.generate(&snapshot).await.unwrap();

        assert!(!output.hooks_config.is_empty());
        assert!(output.hooks_config.contains("[hooks]"));
        assert!(output.hooks_config.contains("validate"));
        assert!(output.operators.len() > 0);
        assert!(output.guards.len() > 0);
    }
}
