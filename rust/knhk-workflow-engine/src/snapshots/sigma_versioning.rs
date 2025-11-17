//! Σ Snapshot Versioning System
//!
//! Implements snapshot versioning with SHA3 hashing and atomic pointer updates.
//! Provides rollback mechanism and snapshot manifest structure.

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Snapshot ID (SHA-256 hash)
pub type SnapshotId = String;

/// Snapshot structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Snapshot ID (hash of content)
    pub id: SnapshotId,
    /// Parent snapshot ID (for versioning)
    pub parent_id: Option<SnapshotId>,
    /// Snapshot content (workflow state)
    pub content: serde_json::Value,
    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
    /// Creation timestamp
    pub created_at_ms: u64,
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Snapshot version
    pub version: u64,
    /// Snapshot description
    pub description: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

impl Snapshot {
    /// Create new snapshot
    pub fn new(content: serde_json::Value, parent_id: Option<SnapshotId>, version: u64) -> Self {
        let id = Self::compute_hash(&content);
        let created_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            id,
            parent_id,
            content,
            metadata: SnapshotMetadata {
                version,
                description: None,
                tags: Vec::new(),
                attributes: HashMap::new(),
            },
            created_at_ms,
        }
    }

    /// Compute SHA-256 hash of content
    fn compute_hash(content: &serde_json::Value) -> String {
        let json_str = serde_json::to_string(content).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json_str.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify snapshot integrity
    pub fn verify_integrity(&self) -> bool {
        let computed_hash = Self::compute_hash(&self.content);
        computed_hash == self.id
    }
}

/// Snapshot manifest (current state pointer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotManifest {
    /// Current snapshot ID
    pub current_snapshot_id: SnapshotId,
    /// Snapshot history (chronological)
    pub history: Vec<SnapshotId>,
    /// Manifest version
    pub version: u64,
}

/// Snapshot versioning system
pub struct SnapshotVersioning {
    /// Snapshot store
    snapshots: Arc<RwLock<HashMap<SnapshotId, Snapshot>>>,
    /// Current snapshot manifest
    manifest: Arc<RwLock<SnapshotManifest>>,
}

impl SnapshotVersioning {
    /// Create new snapshot versioning system
    pub fn new() -> Self {
        // Create initial empty snapshot
        let initial_content = serde_json::json!({});
        let initial_snapshot = Snapshot::new(initial_content, None, 0);
        let initial_id = initial_snapshot.id.clone();

        let mut snapshots = HashMap::new();
        snapshots.insert(initial_id.clone(), initial_snapshot);

        let manifest = SnapshotManifest {
            current_snapshot_id: initial_id.clone(),
            history: vec![initial_id],
            version: 0,
        };

        Self {
            snapshots: Arc::new(RwLock::new(snapshots)),
            manifest: Arc::new(RwLock::new(manifest)),
        }
    }

    /// Create new snapshot from current state
    pub async fn create_snapshot(&self, content: serde_json::Value) -> WorkflowResult<SnapshotId> {
        let manifest = self.manifest.read().await;
        let parent_id = manifest.current_snapshot_id.clone();
        let version = manifest.version + 1;
        drop(manifest);

        let snapshot = Snapshot::new(content, Some(parent_id), version);

        if !snapshot.verify_integrity() {
            return Err(WorkflowError::SnapshotError(
                "Snapshot integrity check failed".to_string(),
            ));
        }

        let snapshot_id = snapshot.id.clone();

        // Store snapshot
        {
            let mut snapshots = self.snapshots.write().await;
            snapshots.insert(snapshot_id.clone(), snapshot);
        }

        // Update manifest atomically
        {
            let mut manifest = self.manifest.write().await;
            manifest.current_snapshot_id = snapshot_id.clone();
            manifest.history.push(snapshot_id.clone());
            manifest.version = version;
        }

        Ok(snapshot_id)
    }

    /// Get snapshot by ID
    pub async fn get_snapshot(&self, snapshot_id: &SnapshotId) -> Option<Snapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.get(snapshot_id).cloned()
    }

    /// Get current snapshot
    pub async fn get_current_snapshot(&self) -> Option<Snapshot> {
        let manifest = self.manifest.read().await;
        let current_id = manifest.current_snapshot_id.clone();
        drop(manifest);

        self.get_snapshot(&current_id).await
    }

    /// Rollback to previous snapshot
    pub async fn rollback(&self) -> WorkflowResult<SnapshotId> {
        let mut manifest = self.manifest.write().await;

        if manifest.history.len() <= 1 {
            return Err(WorkflowError::SnapshotError(
                "Cannot rollback: no previous snapshot".to_string(),
            ));
        }

        // Remove current from history
        manifest.history.pop();

        // Get previous snapshot
        let previous_id = manifest
            .history
            .last()
            .ok_or_else(|| WorkflowError::SnapshotError("Invalid history state".to_string()))?
            .clone();

        // Update manifest atomically
        manifest.current_snapshot_id = previous_id.clone();
        manifest.version = manifest.version.saturating_sub(1);

        Ok(previous_id)
    }

    /// Rollback to specific snapshot
    pub async fn rollback_to(&self, snapshot_id: &SnapshotId) -> WorkflowResult<()> {
        // Verify snapshot exists
        {
            let snapshots = self.snapshots.read().await;
            if !snapshots.contains_key(snapshot_id) {
                return Err(WorkflowError::SnapshotError(format!(
                    "Snapshot {} not found",
                    snapshot_id
                )));
            }
        }

        // Update manifest atomically
        let mut manifest = self.manifest.write().await;

        // Find snapshot in history
        if let Some(pos) = manifest.history.iter().position(|id| id == snapshot_id) {
            // Truncate history to this point
            manifest.history.truncate(pos + 1);
            manifest.current_snapshot_id = snapshot_id.clone();
            manifest.version = pos as u64;
            Ok(())
        } else {
            Err(WorkflowError::SnapshotError(format!(
                "Snapshot {} not in history",
                snapshot_id
            )))
        }
    }

    /// Get current snapshot ID
    pub async fn current_id(&self) -> SnapshotId {
        let manifest = self.manifest.read().await;
        manifest.current_snapshot_id.clone()
    }

    /// Create shadow snapshot for testing
    pub async fn create_shadow_snapshot(
        &self,
        content: serde_json::Value,
    ) -> WorkflowResult<SnapshotId> {
        // Create snapshot without updating manifest (shadow)
        let snapshot = Snapshot::new(content, None, 0);
        let snapshot_id = snapshot.id.clone();

        let mut snapshots = self.snapshots.write().await;
        snapshots.insert(snapshot_id.clone(), snapshot);

        Ok(snapshot_id)
    }

    /// Promote shadow snapshot to current
    pub async fn promote_shadow(&self, shadow_id: &SnapshotId) -> WorkflowResult<()> {
        let mut manifest = self.manifest.write().await;
        manifest.current_snapshot_id = shadow_id.clone();
        manifest.history.push(shadow_id.clone());
        manifest.version += 1;
        Ok(())
    }

    /// Get snapshot history
    pub async fn get_history(&self) -> Vec<SnapshotId> {
        let manifest = self.manifest.read().await;
        manifest.history.clone()
    }

    /// Get manifest
    pub async fn get_manifest(&self) -> SnapshotManifest {
        let manifest = self.manifest.read().await;
        manifest.clone()
    }

    /// Get snapshot count
    pub async fn count_snapshots(&self) -> usize {
        let snapshots = self.snapshots.read().await;
        snapshots.len()
    }
}

impl Default for SnapshotVersioning {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_snapshot_creation() {
        let versioning = SnapshotVersioning::new();

        let content = serde_json::json!({"state": "v1"});
        let snapshot_id = versioning
            .create_snapshot(content.clone())
            .await
            .expect("Snapshot creation failed");

        let snapshot = versioning
            .get_snapshot(&snapshot_id)
            .await
            .expect("Snapshot not found");
        assert_eq!(snapshot.content, content);
        assert!(snapshot.verify_integrity());
    }

    #[tokio::test]
    async fn test_snapshot_history() {
        let versioning = SnapshotVersioning::new();

        // Create multiple snapshots
        let id1 = versioning
            .create_snapshot(serde_json::json!({"v": 1}))
            .await
            .expect("Failed");
        let id2 = versioning
            .create_snapshot(serde_json::json!({"v": 2}))
            .await
            .expect("Failed");
        let id3 = versioning
            .create_snapshot(serde_json::json!({"v": 3}))
            .await
            .expect("Failed");

        let history = versioning.get_history().await;
        assert_eq!(history.len(), 4); // Initial + 3 new

        let current = versioning.get_current_snapshot().await.expect("No current");
        assert_eq!(current.id, id3);
    }

    #[tokio::test]
    async fn test_rollback() {
        let versioning = SnapshotVersioning::new();

        let id1 = versioning
            .create_snapshot(serde_json::json!({"v": 1}))
            .await
            .expect("Failed");
        let id2 = versioning
            .create_snapshot(serde_json::json!({"v": 2}))
            .await
            .expect("Failed");
        let _id3 = versioning
            .create_snapshot(serde_json::json!({"v": 3}))
            .await
            .expect("Failed");

        // Rollback once
        let rolled_back_id = versioning.rollback().await.expect("Rollback failed");
        assert_eq!(rolled_back_id, id2);

        let current = versioning.get_current_snapshot().await.expect("No current");
        assert_eq!(current.content, serde_json::json!({"v": 2}));
    }

    #[tokio::test]
    async fn test_rollback_to_specific() {
        let versioning = SnapshotVersioning::new();

        let id1 = versioning
            .create_snapshot(serde_json::json!({"v": 1}))
            .await
            .expect("Failed");
        let _id2 = versioning
            .create_snapshot(serde_json::json!({"v": 2}))
            .await
            .expect("Failed");
        let _id3 = versioning
            .create_snapshot(serde_json::json!({"v": 3}))
            .await
            .expect("Failed");

        // Rollback to v1
        versioning.rollback_to(&id1).await.expect("Rollback failed");

        let current = versioning.get_current_snapshot().await.expect("No current");
        assert_eq!(current.content, serde_json::json!({"v": 1}));

        let history = versioning.get_history().await;
        assert_eq!(history.len(), 2); // Initial + v1 only
    }

    #[tokio::test]
    async fn test_snapshot_integrity() {
        let content = serde_json::json!({"test": "data"});
        let snapshot = Snapshot::new(content, None, 1);

        assert!(snapshot.verify_integrity());

        // Tampered snapshot
        let mut tampered = snapshot.clone();
        tampered.content = serde_json::json!({"tampered": true});
        assert!(!tampered.verify_integrity());
    }

    #[tokio::test]
    async fn test_snapshot_parent_chain() {
        let versioning = SnapshotVersioning::new();

        let id1 = versioning
            .create_snapshot(serde_json::json!({"v": 1}))
            .await
            .expect("Failed");
        let id2 = versioning
            .create_snapshot(serde_json::json!({"v": 2}))
            .await
            .expect("Failed");

        let snapshot2 = versioning.get_snapshot(&id2).await.expect("Not found");
        assert_eq!(snapshot2.parent_id, Some(id1));
    }
}
