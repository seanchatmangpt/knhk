//! Workflow Ontology Snapshot Management (Σ*)
//!
//! Implements versioned ontology snapshots for deterministic workflow execution.
//! Each snapshot represents an immutable state of the workflow ontology (Σ).
//!
//! # Doctrine Compliance
//!
//! - **A = μ(O)**: Execution is deterministic for a given Σ* and O
//! - **μ ∘ μ = μ**: Idempotent execution via snapshot identity
//! - **O ⊨ Σ**: Observations must validate against snapshot ontology
//! - **Σ ⊨ Q**: Snapshots must satisfy all invariants Q

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[cfg(feature = "std")]
use std::time::SystemTime;

/// Workflow ontology snapshot identifier
///
/// Format: Σ_<timestamp>_<seq>
/// Example: Σ_2027-04-01T15:03:12Z_001
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(pub String);

impl SnapshotId {
    /// Create a new snapshot ID with the current timestamp
    #[cfg(feature = "std")]
    pub fn new(sequence: u32) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self(format!("Σ_{:020}_{:03}", timestamp, sequence))
    }

    /// Create a snapshot ID from string (for testing)
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Ontology file reference in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyFile {
    pub path: String,
    pub content_hash: String,
    pub size_bytes: usize,
}

/// Snapshot manifest
///
/// Contains all metadata needed to reconstruct the exact ontology state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub id: SnapshotId,
    pub created_at: u64, // Unix timestamp
    pub ontology_files: Vec<OntologyFile>,
    pub generated_code_hash: String,
    pub generated_config_hash: String,
    pub guards_checked: Vec<String>,
    pub total_hash: String,
}

impl SnapshotManifest {
    /// Create a new snapshot manifest
    pub fn new(id: SnapshotId, ontology_files: Vec<OntologyFile>) -> Self {
        let mut hasher = Sha3_256::new();

        // Hash all ontology files
        for file in &ontology_files {
            hasher.update(file.content_hash.as_bytes());
        }

        let total_hash = format!("sha3-256:{:x}", hasher.finalize());

        Self {
            id,
            created_at: 0, // Will be set on creation
            ontology_files,
            generated_code_hash: String::new(),
            generated_config_hash: String::new(),
            guards_checked: Vec::new(),
            total_hash,
        }
    }

    /// Verify snapshot integrity
    pub fn verify_integrity(&self) -> bool {
        let mut hasher = Sha3_256::new();

        for file in &self.ontology_files {
            hasher.update(file.content_hash.as_bytes());
        }

        let computed = format!("sha3-256:{:x}", hasher.finalize());
        computed == self.total_hash
    }

    /// Check if snapshot satisfies all invariants Q
    pub fn satisfies_invariants(&self, required_guards: &[String]) -> bool {
        required_guards
            .iter()
            .all(|guard| self.guards_checked.contains(guard))
    }
}

/// Snapshot storage and management
pub struct SnapshotStore {
    snapshots: Arc<RwLock<HashMap<SnapshotId, SnapshotManifest>>>,
    current: Arc<RwLock<Option<SnapshotId>>>,
}

impl SnapshotStore {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            current: Arc::new(RwLock::new(None)),
        }
    }

    /// Store a new snapshot
    pub fn store(&self, mut manifest: SnapshotManifest) -> Result<(), String> {
        // Verify integrity before storing
        if !manifest.verify_integrity() {
            return Err("Snapshot integrity check failed".to_string());
        }

        #[cfg(feature = "std")]
        {
            manifest.created_at = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs();
        }

        let id = manifest.id.clone();

        self.snapshots
            .write()
            .map_err(|e| e.to_string())?
            .insert(id, manifest);

        Ok(())
    }

    /// Get a snapshot by ID
    pub fn get(&self, id: &SnapshotId) -> Result<SnapshotManifest, String> {
        self.snapshots
            .read()
            .map_err(|e| e.to_string())?
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Snapshot not found: {}", id.as_str()))
    }

    /// Set the current active snapshot (atomic pointer update)
    pub fn set_current(&self, id: SnapshotId) -> Result<(), String> {
        // Verify snapshot exists
        let _ = self.get(&id)?;

        *self.current.write().map_err(|e| e.to_string())? = Some(id);

        Ok(())
    }

    /// Get the current active snapshot
    pub fn get_current(&self) -> Result<SnapshotManifest, String> {
        let current_id = self
            .current
            .read()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or_else(|| "No current snapshot set".to_string())?;

        self.get(&current_id)
    }

    /// Atomically promote a new snapshot (shadow → production)
    pub fn promote(&self, from: &SnapshotId, to: &SnapshotId) -> Result<(), String> {
        // Verify both snapshots exist
        let _ = self.get(from)?;
        let to_manifest = self.get(to)?;

        // Verify new snapshot satisfies all invariants from old snapshot
        let from_manifest = self.get(from)?;
        if !to_manifest.satisfies_invariants(&from_manifest.guards_checked) {
            return Err("New snapshot does not satisfy required invariants".to_string());
        }

        // Atomic pointer update
        self.set_current(to.clone())?;

        Ok(())
    }

    /// List all snapshots (for MAPE-K knowledge base)
    pub fn list_all(&self) -> Result<Vec<SnapshotId>, String> {
        Ok(self
            .snapshots
            .read()
            .map_err(|e| e.to_string())?
            .keys()
            .cloned()
            .collect())
    }
}

impl Default for SnapshotStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_id_creation() {
        let id = SnapshotId::from_string("Σ_2027-04-01T15:03:12Z_001".to_string());
        assert_eq!(id.as_str(), "Σ_2027-04-01T15:03:12Z_001");
    }

    #[test]
    fn test_snapshot_manifest_integrity() {
        let files = vec![OntologyFile {
            path: "workflow.ttl".to_string(),
            content_hash: "sha3-256:abc123".to_string(),
            size_bytes: 1024,
        }];

        let id = SnapshotId::from_string("Σ_test_001".to_string());
        let manifest = SnapshotManifest::new(id, files);

        assert!(manifest.verify_integrity());
    }

    #[test]
    fn test_snapshot_store() {
        let store = SnapshotStore::new();

        let files = vec![OntologyFile {
            path: "workflow.ttl".to_string(),
            content_hash: "sha3-256:abc123".to_string(),
            size_bytes: 1024,
        }];

        let id = SnapshotId::from_string("Σ_test_001".to_string());
        let manifest = SnapshotManifest::new(id.clone(), files);

        store.store(manifest.clone()).unwrap();

        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved.id, id);
    }

    #[test]
    fn test_current_snapshot() {
        let store = SnapshotStore::new();

        let files = vec![OntologyFile {
            path: "workflow.ttl".to_string(),
            content_hash: "sha3-256:abc123".to_string(),
            size_bytes: 1024,
        }];

        let id = SnapshotId::from_string("Σ_test_001".to_string());
        let manifest = SnapshotManifest::new(id.clone(), files);

        store.store(manifest).unwrap();
        store.set_current(id.clone()).unwrap();

        let current = store.get_current().unwrap();
        assert_eq!(current.id, id);
    }

    #[test]
    fn test_snapshot_promotion() {
        let store = SnapshotStore::new();

        let files1 = vec![OntologyFile {
            path: "workflow.ttl".to_string(),
            content_hash: "sha3-256:abc123".to_string(),
            size_bytes: 1024,
        }];

        let id1 = SnapshotId::from_string("Σ_test_001".to_string());
        let mut manifest1 = SnapshotManifest::new(id1.clone(), files1);
        manifest1.guards_checked = vec!["Q1".to_string(), "Q2".to_string()];

        let files2 = vec![OntologyFile {
            path: "workflow.ttl".to_string(),
            content_hash: "sha3-256:def456".to_string(),
            size_bytes: 2048,
        }];

        let id2 = SnapshotId::from_string("Σ_test_002".to_string());
        let mut manifest2 = SnapshotManifest::new(id2.clone(), files2);
        manifest2.guards_checked = vec!["Q1".to_string(), "Q2".to_string(), "Q3".to_string()];

        store.store(manifest1).unwrap();
        store.store(manifest2).unwrap();

        store.set_current(id1.clone()).unwrap();
        store.promote(&id1, &id2).unwrap();

        let current = store.get_current().unwrap();
        assert_eq!(current.id, id2);
    }

    #[test]
    fn test_invariant_checking() {
        let files = vec![OntologyFile {
            path: "workflow.ttl".to_string(),
            content_hash: "sha3-256:abc123".to_string(),
            size_bytes: 1024,
        }];

        let id = SnapshotId::from_string("Σ_test_001".to_string());
        let mut manifest = SnapshotManifest::new(id, files);
        manifest.guards_checked = vec!["Q1".to_string(), "Q2".to_string()];

        assert!(manifest.satisfies_invariants(&["Q1".to_string()]));
        assert!(manifest.satisfies_invariants(&["Q1".to_string(), "Q2".to_string()]));
        assert!(!manifest.satisfies_invariants(&["Q1".to_string(), "Q3".to_string()]));
    }
}
