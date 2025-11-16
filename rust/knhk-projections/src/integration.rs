//! Integration with SnapshotStore promotion pipeline
//!
//! Extends SnapshotStore to automatically compile projections during promotion.

use crate::compiler::{CompiledProjections, ProjectionCompiler};
use crate::Result;
use knhk_ontology::{PromotionError, SigmaSnapshotId, SnapshotStore};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, instrument};

/// Extended snapshot store with projection compilation
pub struct ProjectionSnapshotStore {
    /// Underlying snapshot store
    store: Arc<SnapshotStore>,

    /// Projection compiler
    compiler: Arc<ProjectionCompiler>,

    /// Compiled artifacts storage (snapshot_id -> compiled projections)
    compiled_artifacts: Arc<RwLock<HashMap<SigmaSnapshotId, CompiledProjections>>>,
}

impl ProjectionSnapshotStore {
    /// Create new projection snapshot store
    pub fn new(store: SnapshotStore) -> Self {
        Self {
            store: Arc::new(store),
            compiler: Arc::new(ProjectionCompiler::new()),
            compiled_artifacts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with custom compiler
    pub fn with_compiler(store: SnapshotStore, compiler: ProjectionCompiler) -> Self {
        Self {
            store: Arc::new(store),
            compiler: Arc::new(compiler),
            compiled_artifacts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get underlying snapshot store
    pub fn store(&self) -> &SnapshotStore {
        &self.store
    }

    /// Promote snapshot with automatic projection compilation
    ///
    /// This performs the complete workflow:
    /// 1. Validate snapshot exists and has receipt
    /// 2. Compile all projections (off-hot-path)
    /// 3. Store compiled artifacts
    /// 4. Atomically promote snapshot (≤10 ticks)
    #[instrument(skip(self), fields(snapshot_id = ?snapshot_id))]
    pub async fn promote_and_compile(
        &self,
        snapshot_id: SigmaSnapshotId,
    ) -> Result<CompiledProjections> {
        info!("Starting promote_and_compile for snapshot {:?}", snapshot_id);

        // 1. Validate snapshot exists and has receipt
        let snapshot = self
            .store
            .get_snapshot(&snapshot_id)
            .ok_or(PromotionError::SnapshotNotFound(snapshot_id))
            .map_err(|e| crate::ProjectionError::GenerationFailed(e.to_string()))?;

        // Check receipt
        if snapshot.validation_receipt.is_none() {
            return Err(crate::ProjectionError::GenerationFailed(
                "Snapshot has no validation receipt".to_string(),
            ));
        }

        if !snapshot.is_production_ready() {
            return Err(crate::ProjectionError::GenerationFailed(
                "Snapshot is not production ready".to_string(),
            ));
        }

        // 2. Compile all projections (happens off-hot-path)
        info!("Compiling projections for snapshot {:?}", snapshot_id);
        let compiled = self.compiler.compile_all(Arc::new(snapshot)).await?;

        // 3. Store compiled artifacts (for later deployment)
        info!("Storing compiled artifacts for snapshot {:?}", snapshot_id);
        self.store_compiled_artifacts(snapshot_id, &compiled);

        // 4. Now promote (fast pointer swap, ≤10 ticks)
        info!("Promoting snapshot {:?}", snapshot_id);
        self.store
            .promote_snapshot(snapshot_id)
            .map_err(|e| crate::ProjectionError::GenerationFailed(e.to_string()))?;

        info!("Successfully promoted and compiled snapshot {:?}", snapshot_id);

        Ok(compiled)
    }

    /// Store compiled artifacts
    fn store_compiled_artifacts(
        &self,
        snapshot_id: SigmaSnapshotId,
        compiled: &CompiledProjections,
    ) {
        self.compiled_artifacts
            .write()
            .insert(snapshot_id, compiled.clone());
    }

    /// Get compiled artifacts for a snapshot
    pub fn get_compiled_artifacts(
        &self,
        snapshot_id: &SigmaSnapshotId,
    ) -> Option<CompiledProjections> {
        self.compiled_artifacts.read().get(snapshot_id).cloned()
    }

    /// Get compiled artifacts for current snapshot
    pub fn current_compiled_artifacts(&self) -> Option<CompiledProjections> {
        let snapshot = self.store.current_snapshot()?;
        self.get_compiled_artifacts(&snapshot.id)
    }

    /// Pre-compile projections without promoting
    ///
    /// Useful for CI/CD pipelines where you want to compile artifacts
    /// before promotion decision is made.
    #[instrument(skip(self), fields(snapshot_id = ?snapshot_id))]
    pub async fn precompile(
        &self,
        snapshot_id: SigmaSnapshotId,
    ) -> Result<CompiledProjections> {
        let snapshot = self
            .store
            .get_snapshot(&snapshot_id)
            .ok_or_else(|| {
                crate::ProjectionError::GenerationFailed(format!(
                    "Snapshot not found: {:?}",
                    snapshot_id
                ))
            })?;

        let compiled = self.compiler.compile_all(Arc::new(snapshot)).await?;
        self.store_compiled_artifacts(snapshot_id, &compiled);

        Ok(compiled)
    }

    /// Clear compiled artifacts cache
    pub fn clear_compiled_artifacts(&self) {
        self.compiled_artifacts.write().clear();
    }

    /// Get compilation statistics
    pub fn compilation_stats(&self) -> (usize, usize) {
        (
            self.compiled_artifacts.read().len(),
            self.compiler.cache_stats().0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_ontology::{
        InvariantValidator, SigmaReceipt, SigmaSnapshot, SnapshotMetadata, Triple, TripleStore,
    };

    fn create_validated_snapshot() -> SigmaSnapshot {
        let mut store = TripleStore::new();
        store.add(Triple::new("company1", "rdf:type", "Company"));
        store.add(Triple::new("company1", "name", "TechCorp"));
        store.add(Triple::new("company1", "sector", "Technology"));

        let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create snapshot");

        // Add validation receipt
        let validator = InvariantValidator::new();
        let results = validator.validate(&snapshot);

        let receipt = SigmaReceipt::new(snapshot.id, None, "Test".to_string(), results, 100);

        snapshot.with_receipt(receipt)
    }

    #[tokio::test]
    async fn test_promote_and_compile() {
        let store = SnapshotStore::new();
        let projection_store = ProjectionSnapshotStore::new(store);

        let snapshot = create_validated_snapshot();
        let snapshot_id = snapshot.id;

        projection_store.store().add_snapshot(snapshot);

        let compiled = projection_store
            .promote_and_compile(snapshot_id)
            .await
            .expect("Promotion failed");

        assert_eq!(compiled.snapshot_id, snapshot_id);
        assert!(compiled.is_complete());

        // Verify stored artifacts
        let retrieved = projection_store
            .get_compiled_artifacts(&snapshot_id)
            .expect("Artifacts not found");
        assert_eq!(retrieved.snapshot_id, snapshot_id);
    }

    #[tokio::test]
    async fn test_precompile() {
        let store = SnapshotStore::new();
        let projection_store = ProjectionSnapshotStore::new(store);

        let snapshot = create_validated_snapshot();
        let snapshot_id = snapshot.id;

        projection_store.store().add_snapshot(snapshot);

        let compiled = projection_store
            .precompile(snapshot_id)
            .await
            .expect("Precompilation failed");

        assert_eq!(compiled.snapshot_id, snapshot_id);

        // Artifacts should be stored
        let retrieved = projection_store
            .get_compiled_artifacts(&snapshot_id)
            .expect("Artifacts not found");
        assert_eq!(retrieved.snapshot_id, snapshot_id);
    }

    #[tokio::test]
    async fn test_current_compiled_artifacts() {
        let store = SnapshotStore::new();
        let projection_store = ProjectionSnapshotStore::new(store);

        let snapshot = create_validated_snapshot();
        let snapshot_id = snapshot.id;

        projection_store.store().add_snapshot(snapshot);

        projection_store
            .promote_and_compile(snapshot_id)
            .await
            .expect("Promotion failed");

        // Give the promotion a moment to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Get current snapshot to debug
        let current_snapshot = projection_store.store().current_snapshot();
        assert!(current_snapshot.is_some(), "Current snapshot should exist after promotion");

        let current = projection_store
            .current_compiled_artifacts()
            .expect("Current artifacts not found");
        assert_eq!(current.snapshot_id, snapshot_id);
    }

    #[tokio::test]
    async fn test_reject_invalid_snapshot() {
        let store = SnapshotStore::new();
        let projection_store = ProjectionSnapshotStore::new(store);

        // Snapshot without receipt
        let mut triple_store = TripleStore::new();
        triple_store.add(Triple::new("test", "data", "value"));

        let snapshot = SigmaSnapshot::new(None, triple_store, SnapshotMetadata::default())
            .expect("Failed to create snapshot");

        let snapshot_id = snapshot.id;
        projection_store.store().add_snapshot(snapshot);

        let result = projection_store.promote_and_compile(snapshot_id).await;
        assert!(result.is_err());
    }
}
