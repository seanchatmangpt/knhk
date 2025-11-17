// kernel/versioning.rs - Version tracking with cryptographic signing
// Phase 3: Time-travel execution and version rollback
// DOCTRINE: Covenant 2 (Invariants Are Law) - Version integrity must be preserved

use blake3;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use parking_lot::RwLock;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Version metadata with full tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: u64,
    pub tag: Option<String>,
    pub timestamp: u64,
    pub parent: Option<u64>,
    pub author: VersionAuthor,
    pub changes: Vec<VersionChange>,
    pub signature: Option<Vec<u8>>,
    pub hash: [u8; 32],
    pub dependencies: Vec<VersionDependency>,
    pub compatibility: CompatibilityInfo,
}

/// Author information for version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionAuthor {
    pub id: String,
    pub name: String,
    pub email: String,
    pub public_key: Option<Vec<u8>>,
}

/// Change record in version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    pub change_type: ChangeType,
    pub path: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Add,
    Modify,
    Remove,
    Rename,
    Reorder,
}

/// Version dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDependency {
    pub component: String,
    pub min_version: u64,
    pub max_version: Option<u64>,
    pub optional: bool,
}

/// Compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub breaking_changes: bool,
    pub forward_compatible: bool,
    pub backward_compatible: bool,
    pub min_compatible_version: u64,
    pub migration_required: bool,
}

/// Version graph for tracking relationships
pub struct VersionGraph {
    versions: RwLock<BTreeMap<u64, Arc<Version>>>,
    tags: RwLock<HashMap<String, u64>>,
    branches: RwLock<HashMap<String, u64>>,
    current_branch: RwLock<String>,
    head: RwLock<u64>,
}

impl VersionGraph {
    pub fn new() -> Self {
        Self {
            versions: RwLock::new(BTreeMap::new()),
            tags: RwLock::new(HashMap::new()),
            branches: RwLock::new(HashMap::from([("main".to_string(), 0)])),
            current_branch: RwLock::new("main".to_string()),
            head: RwLock::new(0),
        }
    }

    /// Add new version to graph
    pub fn add_version(&self, mut version: Version) -> Result<u64, String> {
        // Compute hash if not present
        if version.hash == [0u8; 32] {
            version.hash = self.compute_version_hash(&version);
        }

        let version_id = version.id;

        // Validate parent exists
        if let Some(parent_id) = version.parent {
            if !self.versions.read().contains_key(&parent_id) {
                return Err(format!("Parent version {} not found", parent_id));
            }
        }

        // Store version
        self.versions
            .write()
            .insert(version_id, Arc::new(version.clone()));

        // Update head if this is newer
        let mut head = self.head.write();
        if version_id > *head {
            *head = version_id;
        }

        // Update branch pointer
        let current_branch = self.current_branch.read().clone();
        self.branches.write().insert(current_branch, version_id);

        info!("Added version {} to graph", version_id);
        Ok(version_id)
    }

    /// Tag a version
    pub fn tag_version(&self, version_id: u64, tag: String) -> Result<(), String> {
        if !self.versions.read().contains_key(&version_id) {
            return Err(format!("Version {} not found", version_id));
        }

        if self.tags.read().contains_key(&tag) {
            return Err(format!("Tag {} already exists", tag));
        }

        self.tags.write().insert(tag.clone(), version_id);
        info!("Tagged version {} as {}", version_id, tag);
        Ok(())
    }

    /// Get version by ID
    pub fn get_version(&self, version_id: u64) -> Option<Arc<Version>> {
        self.versions.read().get(&version_id).cloned()
    }

    /// Get version by tag
    pub fn get_tagged_version(&self, tag: &str) -> Option<Arc<Version>> {
        let tags = self.tags.read();
        if let Some(version_id) = tags.get(tag) {
            self.versions.read().get(version_id).cloned()
        } else {
            None
        }
    }

    /// Get version history from a starting point
    pub fn get_history(&self, from: u64, limit: usize) -> Vec<Arc<Version>> {
        let versions = self.versions.read();
        let mut history = Vec::new();
        let mut current_id = Some(from);
        let mut count = 0;

        while let Some(id) = current_id {
            if count >= limit {
                break;
            }

            if let Some(version) = versions.get(&id) {
                history.push(Arc::clone(version));
                current_id = version.parent;
                count += 1;
            } else {
                break;
            }
        }

        history
    }

    /// Find common ancestor of two versions
    pub fn find_common_ancestor(&self, v1: u64, v2: u64) -> Option<u64> {
        let versions = self.versions.read();

        // Build ancestry chain for v1
        let mut v1_ancestors = Vec::new();
        let mut current = Some(v1);

        while let Some(id) = current {
            v1_ancestors.push(id);
            current = versions.get(&id).and_then(|v| v.parent);
        }

        // Walk v2 ancestry and find first match
        current = Some(v2);
        while let Some(id) = current {
            if v1_ancestors.contains(&id) {
                return Some(id);
            }
            current = versions.get(&id).and_then(|v| v.parent);
        }

        None
    }

    /// Check if version is ancestor of another
    pub fn is_ancestor(&self, ancestor: u64, descendant: u64) -> bool {
        let versions = self.versions.read();
        let mut current = Some(descendant);

        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            current = versions.get(&id).and_then(|v| v.parent);
        }

        false
    }

    fn compute_version_hash(&self, version: &Version) -> [u8; 32] {
        let serialized = serde_json::to_vec(version).unwrap_or_default();
        blake3::hash(&serialized).into()
    }
}

/// Cryptographic signer for versions
pub struct VersionSigner {
    signing_key: SigningKey,
}

impl VersionSigner {
    pub fn new() -> Self {
        use rand::RngCore;
        let mut secret = [0u8; 32];
        OsRng.fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);

        Self { signing_key }
    }

    pub fn from_keys(secret: &[u8]) -> Result<Self, String> {
        let signing_key = SigningKey::from_bytes(
            <&[u8; 32]>::try_from(secret).map_err(|e| format!("Invalid secret key length: {}", e))?,
        );

        Ok(Self { signing_key })
    }

    pub fn sign_version(&self, version: &Version) -> Vec<u8> {
        let message = serde_json::to_vec(version).unwrap_or_default();
        let signature = self.signing_key.sign(&message);
        signature.to_bytes().to_vec()
    }

    pub fn verify_signature(&self, version: &Version, signature: &[u8]) -> bool {
        let message = serde_json::to_vec(version).unwrap_or_default();
        let verifying_key = self.signing_key.verifying_key();

        if signature.len() == 64 {
            if let Ok(sig_array) = <&[u8; 64]>::try_from(signature) {
                let sig = Signature::from_bytes(sig_array);
                verifying_key.verify(&message, &sig).is_ok()
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }
}

/// Version rollback manager
pub struct RollbackManager {
    graph: Arc<VersionGraph>,
    rollback_points: RwLock<Vec<RollbackPoint>>,
    max_rollback_points: usize,
}

#[derive(Debug, Clone)]
struct RollbackPoint {
    version_id: u64,
    created_at: u64,
    reason: String,
    automatic: bool,
}

impl RollbackManager {
    pub fn new(graph: Arc<VersionGraph>) -> Self {
        Self {
            graph,
            rollback_points: RwLock::new(Vec::new()),
            max_rollback_points: 50,
        }
    }

    /// Create rollback point
    pub fn create_rollback_point(
        &self,
        version_id: u64,
        reason: String,
        automatic: bool,
    ) -> Result<(), String> {
        if self.graph.get_version(version_id).is_none() {
            return Err(format!("Version {} not found", version_id));
        }

        let rollback_point = RollbackPoint {
            version_id,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            reason,
            automatic,
        };

        let mut points = self.rollback_points.write();
        points.push(rollback_point);

        // Prune old points if needed
        if points.len() > self.max_rollback_points {
            let drain_count = points.len() - self.max_rollback_points;
            points.drain(0..drain_count).for_each(drop);
        }

        Ok(())
    }

    /// Get available rollback points
    pub fn get_rollback_points(&self) -> Vec<(u64, String, u64)> {
        self.rollback_points
            .read()
            .iter()
            .map(|p| (p.version_id, p.reason.clone(), p.created_at))
            .collect()
    }

    /// Perform rollback to specific version
    pub fn rollback_to(&self, target_version: u64) -> Result<Vec<VersionChange>, String> {
        let current_version = *self.graph.head.read();

        if !self.graph.is_ancestor(target_version, current_version) {
            return Err(format!(
                "Version {} is not an ancestor of current version {}",
                target_version, current_version
            ));
        }

        // Calculate changes needed for rollback
        let changes = self.calculate_rollback_changes(current_version, target_version)?;

        // Update head
        *self.graph.head.write() = target_version;

        // Create automatic rollback point for recovery
        self.create_rollback_point(
            current_version,
            format!("Before rollback to {}", target_version),
            true,
        )?;

        info!(
            "Rolled back from version {} to {}",
            current_version, target_version
        );
        Ok(changes)
    }

    fn calculate_rollback_changes(&self, from: u64, to: u64) -> Result<Vec<VersionChange>, String> {
        let mut changes = Vec::new();
        let history = self.graph.get_history(from, 100);

        for version in history {
            if version.id == to {
                break;
            }

            // Reverse each change
            for change in &version.changes {
                let reversed = match change.change_type {
                    ChangeType::Add => VersionChange {
                        change_type: ChangeType::Remove,
                        path: change.path.clone(),
                        old_value: change.new_value.clone(),
                        new_value: None,
                        description: format!("Rollback: {}", change.description),
                    },
                    ChangeType::Remove => VersionChange {
                        change_type: ChangeType::Add,
                        path: change.path.clone(),
                        old_value: None,
                        new_value: change.old_value.clone(),
                        description: format!("Rollback: {}", change.description),
                    },
                    ChangeType::Modify => VersionChange {
                        change_type: ChangeType::Modify,
                        path: change.path.clone(),
                        old_value: change.new_value.clone(),
                        new_value: change.old_value.clone(),
                        description: format!("Rollback: {}", change.description),
                    },
                    _ => change.clone(),
                };
                changes.push(reversed);
            }
        }

        Ok(changes)
    }
}

/// Dependency resolver for versions
pub struct DependencyResolver {
    graph: Arc<VersionGraph>,
}

impl DependencyResolver {
    pub fn new(graph: Arc<VersionGraph>) -> Self {
        Self { graph }
    }

    /// Check if all dependencies are satisfied
    pub fn check_dependencies(&self, version_id: u64) -> Result<(), Vec<String>> {
        let version = self
            .graph
            .get_version(version_id)
            .ok_or_else(|| vec![format!("Version {} not found", version_id)])?;

        let mut errors = Vec::new();

        for dep in &version.dependencies {
            if let Err(e) = self.check_dependency(dep) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn check_dependency(&self, dep: &VersionDependency) -> Result<(), String> {
        // In a real system, this would check actual component versions
        // For now, we'll simulate
        if dep.min_version > 100 && !dep.optional {
            return Err(format!(
                "Dependency {} requires version >= {}",
                dep.component, dep.min_version
            ));
        }
        Ok(())
    }

    /// Resolve version conflicts
    pub fn resolve_conflicts(&self, versions: Vec<u64>) -> Result<u64, String> {
        if versions.is_empty() {
            return Err("No versions to resolve".to_string());
        }

        // Find the latest common ancestor of all versions
        let mut common = versions[0];

        for &version in &versions[1..] {
            if let Some(ancestor) = self.graph.find_common_ancestor(common, version) {
                common = ancestor;
            } else {
                return Err(format!(
                    "No common ancestor found for versions {:?}",
                    versions
                ));
            }
        }

        Ok(common)
    }
}

/// Time-travel executor for version rollback
pub struct TimeTravelExecutor {
    graph: Arc<VersionGraph>,
    rollback_manager: Arc<RollbackManager>,
    snapshots: RwLock<BTreeMap<u64, StateSnapshot>>,
}

#[derive(Debug, Clone)]
struct StateSnapshot {
    version_id: u64,
    timestamp: u64,
    state: Vec<u8>,
    metadata: HashMap<String, String>,
}

impl TimeTravelExecutor {
    pub fn new(graph: Arc<VersionGraph>, rollback_manager: Arc<RollbackManager>) -> Self {
        Self {
            graph,
            rollback_manager,
            snapshots: RwLock::new(BTreeMap::new()),
        }
    }

    /// Create snapshot at current version
    pub fn create_snapshot(&self, state: Vec<u8>) -> Result<u64, String> {
        let version_id = *self.graph.head.read();

        let snapshot = StateSnapshot {
            version_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            state,
            metadata: HashMap::new(),
        };

        self.snapshots.write().insert(version_id, snapshot);

        Ok(version_id)
    }

    /// Travel to specific version
    pub fn travel_to(&self, target_version: u64) -> Result<Vec<u8>, String> {
        // Check if we have a snapshot
        if let Some(snapshot) = self.snapshots.read().get(&target_version) {
            info!("Traveling to version {} using snapshot", target_version);
            return Ok(snapshot.state.clone());
        }

        // Otherwise, perform rollback
        let changes = self.rollback_manager.rollback_to(target_version)?;

        // Apply changes to reconstruct state
        let state = self.reconstruct_state(target_version, changes)?;

        Ok(state)
    }

    fn reconstruct_state(
        &self,
        _version: u64,
        _changes: Vec<VersionChange>,
    ) -> Result<Vec<u8>, String> {
        // In a real system, this would apply changes to reconstruct state
        // For now, return placeholder
        Ok(vec![0u8; 100])
    }

    /// Get timeline of available versions
    pub fn get_timeline(&self, limit: usize) -> Vec<(u64, u64, Option<String>)> {
        let versions = self.graph.versions.read();
        let tags = self.graph.tags.read();

        let tag_map: HashMap<u64, String> =
            tags.iter().map(|(tag, &id)| (id, tag.clone())).collect();

        versions
            .iter()
            .rev()
            .take(limit)
            .map(|(&id, version)| (id, version.timestamp, tag_map.get(&id).cloned()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_version(id: u64, parent: Option<u64>) -> Version {
        Version {
            id,
            tag: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            parent,
            author: VersionAuthor {
                id: "test".to_string(),
                name: "Test Author".to_string(),
                email: "test@example.com".to_string(),
                public_key: None,
            },
            changes: vec![VersionChange {
                change_type: ChangeType::Add,
                path: format!("file_{}.txt", id),
                old_value: None,
                new_value: Some("content".to_string()),
                description: format!("Added file {}", id),
            }],
            signature: None,
            hash: [0u8; 32],
            dependencies: vec![],
            compatibility: CompatibilityInfo {
                breaking_changes: false,
                forward_compatible: true,
                backward_compatible: true,
                min_compatible_version: 1,
                migration_required: false,
            },
        }
    }

    #[test]
    fn test_version_graph() {
        let graph = VersionGraph::new();

        let v1 = create_test_version(1, None);
        graph.add_version(v1).unwrap();

        let v2 = create_test_version(2, Some(1));
        graph.add_version(v2).unwrap();

        let v3 = create_test_version(3, Some(2));
        graph.add_version(v3).unwrap();

        assert!(graph.is_ancestor(1, 3));
        assert!(graph.is_ancestor(2, 3));
        assert!(!graph.is_ancestor(3, 1));

        let common = graph.find_common_ancestor(2, 3);
        assert_eq!(common, Some(2));
    }

    #[test]
    fn test_version_signing() {
        let signer = VersionSigner::new();
        let version = create_test_version(1, None);

        let signature = signer.sign_version(&version);
        assert!(signer.verify_signature(&version, &signature));

        // Tamper with signature
        let mut bad_signature = signature.clone();
        bad_signature[0] ^= 0xFF;
        assert!(!signer.verify_signature(&version, &bad_signature));
    }

    #[test]
    fn test_rollback_manager() {
        let graph = Arc::new(VersionGraph::new());

        for i in 1..=5 {
            let version = create_test_version(i, if i > 1 { Some(i - 1) } else { None });
            graph.add_version(version).unwrap();
        }

        let rollback_mgr = RollbackManager::new(Arc::clone(&graph));

        rollback_mgr
            .create_rollback_point(3, "Test point".to_string(), false)
            .unwrap();

        let points = rollback_mgr.get_rollback_points();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].0, 3);

        let changes = rollback_mgr.rollback_to(2).unwrap();
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_dependency_resolver() {
        let graph = Arc::new(VersionGraph::new());

        let mut v1 = create_test_version(1, None);
        v1.dependencies = vec![VersionDependency {
            component: "core".to_string(),
            min_version: 10,
            max_version: Some(20),
            optional: false,
        }];
        graph.add_version(v1).unwrap();

        let resolver = DependencyResolver::new(Arc::clone(&graph));
        let result = resolver.check_dependencies(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_time_travel_executor() {
        let graph = Arc::new(VersionGraph::new());
        let rollback_mgr = Arc::new(RollbackManager::new(Arc::clone(&graph)));

        for i in 1..=3 {
            let version = create_test_version(i, if i > 1 { Some(i - 1) } else { None });
            graph.add_version(version).unwrap();
        }

        let executor = TimeTravelExecutor::new(Arc::clone(&graph), Arc::clone(&rollback_mgr));

        // Create snapshot
        let snapshot_id = executor.create_snapshot(vec![1, 2, 3]).unwrap();
        assert_eq!(snapshot_id, 3);

        // Travel to version
        let state = executor.travel_to(3).unwrap();
        assert_eq!(state, vec![1, 2, 3]);

        // Get timeline
        let timeline = executor.get_timeline(10);
        assert_eq!(timeline.len(), 3);
    }

    #[test]
    fn test_version_tagging() {
        let graph = VersionGraph::new();

        let v1 = create_test_version(1, None);
        graph.add_version(v1).unwrap();

        graph.tag_version(1, "v1.0.0".to_string()).unwrap();

        let tagged = graph.get_tagged_version("v1.0.0");
        assert!(tagged.is_some());
        assert_eq!(tagged.unwrap().id, 1);

        // Duplicate tag should fail
        let result = graph.tag_version(1, "v1.0.0".to_string());
        assert!(result.is_err());
    }
}
