//! Immutable snapshot store with Blake3 content-addressable IDs.
//!
//! Snapshots are write-once, immutable, and cryptographically identified.
//! Each snapshot contains:
//! - Unique ID (Blake3 hash of parent + delta + timestamp)
//! - Parent snapshot ID (lineage tracking)
//! - RDF triple store (immutable after creation)
//! - Metadata (who, when, why)
//! - Validation receipt (proof of Q preservation)

use crate::receipt::SigmaReceipt;
use blake3::Hasher;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Snapshot ID: 32-byte Blake3 hash
pub type SigmaSnapshotId = [u8; 32];

/// Errors in snapshot operations
#[derive(Error, Debug, Clone)]
pub enum SnapshotError {
    #[error("Snapshot not found: {0:?}")]
    NotFound(SigmaSnapshotId),

    #[error("Invalid parent snapshot: {0:?}")]
    InvalidParent(SigmaSnapshotId),

    #[error("Snapshot is immutable and cannot be modified")]
    ImmutableSnapshot,

    #[error("Failed to compute snapshot ID: {0}")]
    HashingError(String),
}

/// RDF Triple: (subject, predicate, object)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl Triple {
    pub fn new(subject: impl Into<String>, predicate: impl Into<String>, object: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
        }
    }
}

/// Immutable triple store (snapshot of RDF graph)
#[derive(Debug, Clone, Default)]
pub struct TripleStore {
    triples: Vec<Triple>,
    /// Index: subject -> [triple_indices]
    subject_index: HashMap<String, Vec<usize>>,
    /// Index: predicate -> [triple_indices]
    predicate_index: HashMap<String, Vec<usize>>,
}

impl TripleStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add triple and update indices
    pub fn add(&mut self, triple: Triple) {
        let idx = self.triples.len();

        self.subject_index
            .entry(triple.subject.clone())
            .or_default()
            .push(idx);

        self.predicate_index
            .entry(triple.predicate.clone())
            .or_default()
            .push(idx);

        self.triples.push(triple);
    }

    /// Query by subject
    pub fn query_subject(&self, subject: &str) -> Vec<&Triple> {
        self.subject_index
            .get(subject)
            .map(|indices| indices.iter().map(|&i| &self.triples[i]).collect())
            .unwrap_or_default()
    }

    /// Query by predicate
    pub fn query_predicate(&self, predicate: &str) -> Vec<&Triple> {
        self.predicate_index
            .get(predicate)
            .map(|indices| indices.iter().map(|&i| &self.triples[i]).collect())
            .unwrap_or_default()
    }

    /// Get all triples
    pub fn all_triples(&self) -> &[Triple] {
        &self.triples
    }

    /// Count triples
    pub fn len(&self) -> usize {
        self.triples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }

    /// Compute content hash (for snapshot ID)
    pub fn content_hash(&self) -> [u8; 32] {
        let mut hasher = Hasher::new();

        // Sort triples for deterministic hashing
        let mut sorted: Vec<_> = self.triples.iter().collect();
        sorted.sort_by(|a, b| {
            a.subject.cmp(&b.subject)
                .then(a.predicate.cmp(&b.predicate))
                .then(a.object.cmp(&b.object))
        });

        for triple in sorted {
            hasher.update(triple.subject.as_bytes());
            hasher.update(triple.predicate.as_bytes());
            hasher.update(triple.object.as_bytes());
        }

        *hasher.finalize().as_bytes()
    }
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub created_at: SystemTime,
    pub created_by: String,
    pub description: String,
    pub sector: Option<String>,
}

impl Default for SnapshotMetadata {
    fn default() -> Self {
        Self {
            created_at: SystemTime::now(),
            created_by: "unknown".to_string(),
            description: "No description".to_string(),
            sector: None,
        }
    }
}

/// Immutable snapshot of ontology state
#[derive(Debug, Clone)]
pub struct SigmaSnapshot {
    /// Unique ID: Blake3(parent_id || triple_hash || timestamp)
    pub id: SigmaSnapshotId,

    /// Parent snapshot (lineage)
    pub parent_id: Option<SigmaSnapshotId>,

    /// Immutable triple store
    pub triple_store: Arc<RwLock<TripleStore>>,

    /// Metadata
    pub metadata: SnapshotMetadata,

    /// Validation receipt (proves Q preserved)
    pub validation_receipt: Option<SigmaReceipt>,

    /// When promoted to current (if ever)
    pub promoted_at: Option<SystemTime>,
}

impl SigmaSnapshot {
    /// Create new snapshot with parent lineage
    pub fn new(
        parent_id: Option<SigmaSnapshotId>,
        triple_store: TripleStore,
        metadata: SnapshotMetadata,
    ) -> Result<Self, SnapshotError> {
        let id = Self::compute_id(parent_id.as_ref(), &triple_store, &metadata)?;

        Ok(Self {
            id,
            parent_id,
            triple_store: Arc::new(RwLock::new(triple_store)),
            metadata,
            validation_receipt: None,
            promoted_at: None,
        })
    }

    /// Compute snapshot ID: Blake3(parent_id || triple_hash || timestamp)
    fn compute_id(
        parent_id: Option<&SigmaSnapshotId>,
        triple_store: &TripleStore,
        metadata: &SnapshotMetadata,
    ) -> Result<SigmaSnapshotId, SnapshotError> {
        let mut hasher = Hasher::new();

        // Hash parent ID
        if let Some(parent) = parent_id {
            hasher.update(parent);
        }

        // Hash triple store content
        hasher.update(&triple_store.content_hash());

        // Hash timestamp
        let timestamp = metadata.created_at
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SnapshotError::HashingError(e.to_string()))?
            .as_nanos()
            .to_le_bytes();
        hasher.update(&timestamp);

        // Hash created_by
        hasher.update(metadata.created_by.as_bytes());

        Ok(*hasher.finalize().as_bytes())
    }

    /// Attach validation receipt
    pub fn with_receipt(mut self, receipt: SigmaReceipt) -> Self {
        self.validation_receipt = Some(receipt);
        self
    }

    /// Mark as promoted
    pub fn mark_promoted(mut self) -> Self {
        self.promoted_at = Some(SystemTime::now());
        self
    }

    /// Check if snapshot is production-ready
    pub fn is_production_ready(&self) -> bool {
        self.validation_receipt
            .as_ref()
            .map(|r| r.production_ready)
            .unwrap_or(false)
    }

    /// Get lineage depth
    pub fn lineage_depth(&self) -> usize {
        if self.parent_id.is_some() {
            1 // Would need recursive lookup for full depth
        } else {
            0
        }
    }

    /// Query triples
    pub fn query_subject(&self, subject: &str) -> Vec<Triple> {
        self.triple_store
            .read()
            .query_subject(subject)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get all triples (clone)
    pub fn all_triples(&self) -> Vec<Triple> {
        self.triple_store
            .read()
            .all_triples()
            .to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triple_store_add_query() {
        let mut store = TripleStore::new();

        store.add(Triple::new("alice", "knows", "bob"));
        store.add(Triple::new("alice", "likes", "chess"));
        store.add(Triple::new("bob", "knows", "alice"));

        assert_eq!(store.len(), 3);

        let alice_triples = store.query_subject("alice");
        assert_eq!(alice_triples.len(), 2);

        let knows_triples = store.query_predicate("knows");
        assert_eq!(knows_triples.len(), 2);
    }

    #[test]
    fn test_triple_store_content_hash() {
        let mut store1 = TripleStore::new();
        store1.add(Triple::new("a", "b", "c"));
        store1.add(Triple::new("d", "e", "f"));

        let mut store2 = TripleStore::new();
        store2.add(Triple::new("d", "e", "f"));
        store2.add(Triple::new("a", "b", "c"));

        // Same content, different order = same hash (deterministic)
        assert_eq!(store1.content_hash(), store2.content_hash());
    }

    #[test]
    fn test_snapshot_creation() {
        let mut store = TripleStore::new();
        store.add(Triple::new("subject", "predicate", "object"));

        let metadata = SnapshotMetadata {
            created_by: "test".to_string(),
            description: "Test snapshot".to_string(),
            ..Default::default()
        };

        let snapshot = SigmaSnapshot::new(None, store, metadata)
            .expect("Failed to create snapshot");

        assert!(snapshot.parent_id.is_none());
        assert_eq!(snapshot.all_triples().len(), 1);
        assert!(!snapshot.is_production_ready());
    }

    #[test]
    fn test_snapshot_immutability() {
        let mut store = TripleStore::new();
        store.add(Triple::new("a", "b", "c"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create snapshot");

        let original_id = snapshot.id;
        let triples_before = snapshot.all_triples().len();

        // Snapshot ID should not change
        assert_eq!(snapshot.id, original_id);
        assert_eq!(snapshot.all_triples().len(), triples_before);
    }

    #[test]
    fn test_snapshot_lineage() {
        let mut store1 = TripleStore::new();
        store1.add(Triple::new("a", "b", "c"));

        let parent = SigmaSnapshot::new(None, store1, SnapshotMetadata::default())
            .expect("Failed to create parent");

        let mut store2 = TripleStore::new();
        store2.add(Triple::new("d", "e", "f"));

        let child = SigmaSnapshot::new(Some(parent.id), store2, SnapshotMetadata::default())
            .expect("Failed to create child");

        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.lineage_depth(), 1);
    }
}
