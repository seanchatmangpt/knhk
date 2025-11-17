// kernel/descriptor_manager.rs - Descriptor versioning and hot-swap mechanism
// Phase 3: Lock-free descriptor updates with version history
// DOCTRINE: Rule 4 (All changes are descriptor changes)

use blake3;
use crossbeam::epoch::{self, Atomic, Guard, Owned, Shared};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Descriptor version with cryptographic hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptorVersion {
    pub version: u64,
    pub timestamp: u64,
    pub hash: [u8; 32],
    pub parent_version: Option<u64>,
    pub author: String,
    pub message: String,
    pub tags: Vec<String>,
}

/// Core descriptor content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptorContent {
    pub id: String,
    pub schema_version: String,
    pub rules: Vec<Rule>,
    pub patterns: Vec<Pattern>,
    pub constraints: Constraints,
    pub metadata: HashMap<String, String>,
}

/// Rule definition within descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub condition: String,
    pub action: String,
    pub priority: u8,
}

/// Pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub template: String,
    pub parameters: HashMap<String, String>,
}

/// Constraints for descriptor execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub max_execution_time_us: u64,
    pub max_memory_bytes: usize,
    pub required_capabilities: Vec<String>,
    pub forbidden_operations: Vec<String>,
}

/// Atomic descriptor holder for lock-free access
pub struct AtomicDescriptor {
    current: Atomic<Descriptor>,
    version: AtomicU64,
    readers: AtomicUsize,
}

impl AtomicDescriptor {
    pub fn new(descriptor: Descriptor) -> Self {
        let version = descriptor.version.version;
        Self {
            current: Atomic::new(descriptor),
            version: AtomicU64::new(version),
            readers: AtomicUsize::new(0),
        }
    }

    /// Get current descriptor with epoch-based memory management
    pub fn load<'g>(&self, guard: &'g Guard) -> Shared<'g, Descriptor> {
        self.readers.fetch_add(1, Ordering::Acquire);
        let descriptor = self.current.load(Ordering::Acquire, guard);
        self.readers.fetch_sub(1, Ordering::Release);
        descriptor
    }

    /// Atomically swap descriptor (lock-free)
    pub fn swap(&self, new_descriptor: Descriptor, guard: &Guard) -> Result<u64, String> {
        let new_version = new_descriptor.version.version;
        let old_version = self.version.load(Ordering::Acquire);

        if new_version <= old_version {
            return Err(format!(
                "Version {} not newer than current {}",
                new_version, old_version
            ));
        }

        let owned = Owned::new(new_descriptor);
        let _old = self.current.swap(owned, Ordering::AcqRel, guard);

        // Update version counter
        self.version.store(new_version, Ordering::Release);

        // The old descriptor will be deallocated when safe via epoch-based reclamation
        unsafe {
            guard.defer_destroy(_old);
        }

        Ok(new_version)
    }

    /// Get current version without loading descriptor
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    /// Get number of active readers
    pub fn reader_count(&self) -> usize {
        self.readers.load(Ordering::Acquire)
    }
}

/// Full descriptor with version and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descriptor {
    pub version: DescriptorVersion,
    pub content: DescriptorContent,
    #[serde(skip)]
    pub compiled: Option<CompiledDescriptor>,
}

/// Compiled descriptor for fast execution
#[derive(Debug, Clone)]
pub struct CompiledDescriptor {
    pub bytecode: Vec<u8>,
    pub jump_table: Vec<usize>,
    pub constant_pool: Vec<Vec<u8>>,
    pub metadata: HashMap<String, usize>,
}

/// Version history tracker
pub struct VersionHistory {
    versions: RwLock<HashMap<u64, DescriptorVersion>>,
    timeline: RwLock<Vec<u64>>,
    max_history: usize,
}

impl VersionHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            versions: RwLock::new(HashMap::new()),
            timeline: RwLock::new(Vec::new()),
            max_history,
        }
    }

    pub fn add_version(&self, version: DescriptorVersion) {
        let version_id = version.version;

        let mut versions = self.versions.write();
        let mut timeline = self.timeline.write();

        versions.insert(version_id, version);
        timeline.push(version_id);

        // Prune old versions if needed
        if timeline.len() > self.max_history {
            if let Some(oldest) = timeline.first() {
                let oldest_id = *oldest;
                versions.remove(&oldest_id);
                timeline.remove(0);
            }
        }
    }

    pub fn get_version(&self, version_id: u64) -> Option<DescriptorVersion> {
        self.versions.read().get(&version_id).cloned()
    }

    pub fn get_timeline(&self) -> Vec<u64> {
        self.timeline.read().clone()
    }

    pub fn find_by_tag(&self, tag: &str) -> Option<DescriptorVersion> {
        self.versions
            .read()
            .values()
            .find(|v| v.tags.contains(&tag.to_string()))
            .cloned()
    }
}

/// Main descriptor manager
pub struct DescriptorManager {
    current_descriptor: Arc<AtomicDescriptor>,
    history: Arc<VersionHistory>,
    pending_updates: Arc<RwLock<Vec<PendingUpdate>>>,
    rollback_stack: Arc<RwLock<Vec<u64>>>,
    transition_log: Arc<RwLock<Vec<StateTransition>>>,
    compatibility_checker: Arc<CompatibilityChecker>,
}

/// Pending descriptor update
#[derive(Debug, Clone)]
struct PendingUpdate {
    descriptor: Descriptor,
    scheduled_at: u64,
    validation_status: ValidationStatus,
}

#[derive(Debug, Clone)]
enum ValidationStatus {
    Pending,
    Passed,
    Failed(String),
}

/// State transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_version: u64,
    pub to_version: u64,
    pub timestamp: u64,
    pub transition_type: TransitionType,
    pub duration_us: u64,
    pub reader_impact_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionType {
    HotSwap,
    Rollback,
    Emergency,
    Scheduled,
}

impl DescriptorManager {
    pub fn new(initial_descriptor: Descriptor) -> Self {
        let version = initial_descriptor.version.clone();

        let atomic_desc = Arc::new(AtomicDescriptor::new(initial_descriptor));
        let history = Arc::new(VersionHistory::new(100));

        history.add_version(version);

        Self {
            current_descriptor: atomic_desc,
            history,
            pending_updates: Arc::new(RwLock::new(Vec::new())),
            rollback_stack: Arc::new(RwLock::new(Vec::new())),
            transition_log: Arc::new(RwLock::new(Vec::new())),
            compatibility_checker: Arc::new(CompatibilityChecker::new()),
        }
    }

    /// Hot-swap descriptor with atomic update
    pub fn hot_swap(&self, new_descriptor: Descriptor) -> Result<u64, String> {
        let start = std::time::Instant::now();

        // Validate new descriptor
        if let Err(e) = self.validate_descriptor(&new_descriptor) {
            return Err(format!("Validation failed: {}", e));
        }

        // Check compatibility
        let guard = &epoch::pin();
        let current = self.current_descriptor.load(guard);

        unsafe {
            let current_ref = current.as_ref().unwrap();
            if !self
                .compatibility_checker
                .check_compatibility(&current_ref.content, &new_descriptor.content)
            {
                return Err("Incompatible descriptor change".to_string());
            }
        }

        // Record current version for rollback
        let current_version = self.current_descriptor.version();
        self.rollback_stack.write().push(current_version);

        // Perform atomic swap
        let new_version = self
            .current_descriptor
            .swap(new_descriptor.clone(), guard)?;

        // Add to history
        self.history.add_version(new_descriptor.version.clone());

        // Log transition
        let duration_us = start.elapsed().as_micros() as u64;
        let reader_impact_us = self.measure_reader_impact();

        self.transition_log.write().push(StateTransition {
            from_version: current_version,
            to_version: new_version,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transition_type: TransitionType::HotSwap,
            duration_us,
            reader_impact_us,
        });

        info!(
            "Hot-swapped descriptor from v{} to v{} in {}us",
            current_version, new_version, duration_us
        );

        Ok(new_version)
    }

    /// Rollback to previous version
    pub fn rollback(&self) -> Result<u64, String> {
        let rollback_version = self
            .rollback_stack
            .write()
            .pop()
            .ok_or_else(|| "No version to rollback to".to_string())?;

        let version_info = self
            .history
            .get_version(rollback_version)
            .ok_or_else(|| format!("Version {} not found in history", rollback_version))?;

        // Reconstruct descriptor from history
        // In production, this would load from persistent storage
        let descriptor = self.reconstruct_descriptor(version_info)?;

        // Perform hot-swap to rollback version
        let guard = &epoch::pin();
        let result = self.current_descriptor.swap(descriptor, guard)?;

        self.transition_log.write().push(StateTransition {
            from_version: self.current_descriptor.version(),
            to_version: rollback_version,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transition_type: TransitionType::Rollback,
            duration_us: 0,
            reader_impact_us: 0,
        });

        info!("Rolled back to version {}", rollback_version);
        Ok(result)
    }

    /// Schedule descriptor update for later
    pub fn schedule_update(&self, descriptor: Descriptor, at_timestamp: u64) {
        let update = PendingUpdate {
            descriptor,
            scheduled_at: at_timestamp,
            validation_status: ValidationStatus::Pending,
        };

        self.pending_updates.write().push(update);
    }

    /// Process pending updates
    pub fn process_pending_updates(&self) -> Vec<Result<u64, String>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut updates = self.pending_updates.write();
        let mut results = Vec::new();

        updates.retain(|update| {
            if update.scheduled_at <= now {
                match update.validation_status {
                    ValidationStatus::Passed => {
                        results.push(self.hot_swap(update.descriptor.clone()));
                        false // Remove from pending
                    }
                    ValidationStatus::Failed(ref reason) => {
                        results.push(Err(reason.clone()));
                        false // Remove from pending
                    }
                    ValidationStatus::Pending => {
                        // Validate now
                        match self.validate_descriptor(&update.descriptor) {
                            Ok(()) => {
                                results.push(self.hot_swap(update.descriptor.clone()));
                                false
                            }
                            Err(e) => {
                                results.push(Err(e));
                                false
                            }
                        }
                    }
                }
            } else {
                true // Keep in pending
            }
        });

        results
    }

    /// Get current descriptor for reading
    pub fn get_current(&self) -> Arc<AtomicDescriptor> {
        Arc::clone(&self.current_descriptor)
    }

    /// Get version history
    pub fn get_history(&self) -> Arc<VersionHistory> {
        Arc::clone(&self.history)
    }

    /// Get transition log
    pub fn get_transitions(&self) -> Vec<StateTransition> {
        self.transition_log.read().clone()
    }

    /// Validate descriptor
    fn validate_descriptor(&self, descriptor: &Descriptor) -> Result<(), String> {
        // Check version
        if descriptor.version.version == 0 {
            return Err("Invalid version number".to_string());
        }

        // Verify hash
        let computed_hash = self.compute_hash(&descriptor.content);
        if computed_hash != descriptor.version.hash {
            return Err("Hash mismatch".to_string());
        }

        // Validate content structure
        if descriptor.content.rules.is_empty() {
            return Err("Descriptor must have at least one rule".to_string());
        }

        // Check constraints
        if descriptor.content.constraints.max_execution_time_us == 0 {
            return Err("Invalid execution time constraint".to_string());
        }

        Ok(())
    }

    fn compute_hash(&self, content: &DescriptorContent) -> [u8; 32] {
        let serialized = serde_json::to_vec(content).unwrap_or_default();
        blake3::hash(&serialized).into()
    }

    fn reconstruct_descriptor(&self, version: DescriptorVersion) -> Result<Descriptor, String> {
        // In production, this would load from persistent storage
        // For now, create a placeholder
        Ok(Descriptor {
            version,
            content: DescriptorContent {
                id: "reconstructed".to_string(),
                schema_version: "1.0.0".to_string(),
                rules: vec![],
                patterns: vec![],
                constraints: Constraints {
                    max_execution_time_us: 1000,
                    max_memory_bytes: 1024 * 1024,
                    required_capabilities: vec![],
                    forbidden_operations: vec![],
                },
                metadata: HashMap::new(),
            },
            compiled: None,
        })
    }

    fn measure_reader_impact(&self) -> u64 {
        // Measure impact on readers during swap
        let start = std::time::Instant::now();
        let guard = &epoch::pin();

        for _ in 0..100 {
            let _ = self.current_descriptor.load(guard);
        }

        start.elapsed().as_micros() as u64 / 100
    }

    /// Emergency rollback to known good state
    pub fn emergency_rollback(&self, version: u64) -> Result<u64, String> {
        warn!("Emergency rollback initiated to version {}", version);

        let version_info = self
            .history
            .get_version(version)
            .ok_or_else(|| format!("Version {} not found", version))?;

        let descriptor = self.reconstruct_descriptor(version_info)?;

        let guard = &epoch::pin();
        let result = self.current_descriptor.swap(descriptor, guard)?;

        self.transition_log.write().push(StateTransition {
            from_version: self.current_descriptor.version(),
            to_version: version,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transition_type: TransitionType::Emergency,
            duration_us: 0,
            reader_impact_us: 0,
        });

        Ok(result)
    }

    /// Atomic state transition with validation
    pub fn atomic_transition<F>(&self, transition_fn: F) -> Result<u64, String>
    where
        F: FnOnce(&DescriptorContent) -> Result<DescriptorContent, String>,
    {
        let guard = &epoch::pin();
        let current = self.current_descriptor.load(guard);

        let new_content = unsafe {
            let current_ref = current.as_ref().unwrap();
            transition_fn(&current_ref.content)?
        };

        let new_version = DescriptorVersion {
            version: self.current_descriptor.version() + 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            hash: self.compute_hash(&new_content),
            parent_version: Some(self.current_descriptor.version()),
            author: "system".to_string(),
            message: "Atomic transition".to_string(),
            tags: vec![],
        };

        let new_descriptor = Descriptor {
            version: new_version.clone(),
            content: new_content,
            compiled: None,
        };

        self.hot_swap(new_descriptor)
    }
}

/// Compatibility checker for descriptor changes
struct CompatibilityChecker {
    rules: Vec<CompatibilityRule>,
}

impl CompatibilityChecker {
    fn new() -> Self {
        Self {
            rules: vec![
                CompatibilityRule::SchemaVersion,
                CompatibilityRule::RequiredFields,
                CompatibilityRule::ConstraintRelaxation,
            ],
        }
    }

    fn check_compatibility(&self, old: &DescriptorContent, new: &DescriptorContent) -> bool {
        for rule in &self.rules {
            if !self.check_rule(rule, old, new) {
                return false;
            }
        }
        true
    }

    fn check_rule(
        &self,
        rule: &CompatibilityRule,
        old: &DescriptorContent,
        new: &DescriptorContent,
    ) -> bool {
        match rule {
            CompatibilityRule::SchemaVersion => {
                // Major version must match
                let old_major = old.schema_version.split('.').next().unwrap_or("0");
                let new_major = new.schema_version.split('.').next().unwrap_or("0");
                old_major == new_major
            }
            CompatibilityRule::RequiredFields => {
                // New descriptor must have all required fields from old
                old.metadata.keys().all(|k| new.metadata.contains_key(k))
            }
            CompatibilityRule::ConstraintRelaxation => {
                // Constraints can only be relaxed, not tightened
                new.constraints.max_execution_time_us >= old.constraints.max_execution_time_us
                    && new.constraints.max_memory_bytes >= old.constraints.max_memory_bytes
            }
        }
    }
}

#[derive(Debug)]
enum CompatibilityRule {
    SchemaVersion,
    RequiredFields,
    ConstraintRelaxation,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_descriptor(version: u64) -> Descriptor {
        let content = DescriptorContent {
            id: format!("test-{}", version),
            schema_version: "1.0.0".to_string(),
            rules: vec![Rule {
                id: "rule1".to_string(),
                condition: "true".to_string(),
                action: "allow".to_string(),
                priority: 10,
            }],
            patterns: vec![],
            constraints: Constraints {
                max_execution_time_us: 1000,
                max_memory_bytes: 1024 * 1024,
                required_capabilities: vec![],
                forbidden_operations: vec![],
            },
            metadata: HashMap::new(),
        };

        let hash = blake3::hash(serde_json::to_string(&content).unwrap().as_bytes()).into();

        Descriptor {
            version: DescriptorVersion {
                version,
                timestamp: 0,
                hash,
                parent_version: if version > 1 { Some(version - 1) } else { None },
                author: "test".to_string(),
                message: format!("Test version {}", version),
                tags: vec![],
            },
            content,
            compiled: None,
        }
    }

    #[test]
    fn test_descriptor_manager_creation() {
        let descriptor = create_test_descriptor(1);
        let manager = DescriptorManager::new(descriptor);

        assert_eq!(manager.current_descriptor.version(), 1);
    }

    #[test]
    fn test_hot_swap() {
        let descriptor1 = create_test_descriptor(1);
        let manager = DescriptorManager::new(descriptor1);

        let descriptor2 = create_test_descriptor(2);
        let result = manager.hot_swap(descriptor2);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(manager.current_descriptor.version(), 2);
    }

    #[test]
    fn test_rollback() {
        let descriptor1 = create_test_descriptor(1);
        let manager = DescriptorManager::new(descriptor1);

        let descriptor2 = create_test_descriptor(2);
        manager.hot_swap(descriptor2).unwrap();

        let rollback_result = manager.rollback();
        assert!(rollback_result.is_ok());
    }

    #[test]
    fn test_version_history() {
        let descriptor = create_test_descriptor(1);
        let manager = DescriptorManager::new(descriptor);

        for i in 2..=5 {
            let desc = create_test_descriptor(i);
            manager.hot_swap(desc).unwrap();
        }

        let timeline = manager.history.get_timeline();
        assert_eq!(timeline.len(), 5);
    }

    #[test]
    fn test_atomic_descriptor_swap() {
        let desc1 = create_test_descriptor(1);
        let atomic = AtomicDescriptor::new(desc1);

        let guard = &epoch::pin();
        let current = atomic.load(guard);
        assert!(unsafe { current.as_ref().unwrap().version.version == 1 });

        let desc2 = create_test_descriptor(2);
        let result = atomic.swap(desc2, guard);
        assert!(result.is_ok());
        assert_eq!(atomic.version(), 2);
    }

    #[test]
    fn test_compatibility_checker() {
        let checker = CompatibilityChecker::new();

        let old = DescriptorContent {
            id: "test".to_string(),
            schema_version: "1.0.0".to_string(),
            rules: vec![],
            patterns: vec![],
            constraints: Constraints {
                max_execution_time_us: 1000,
                max_memory_bytes: 1024,
                required_capabilities: vec![],
                forbidden_operations: vec![],
            },
            metadata: HashMap::new(),
        };

        let compatible = DescriptorContent {
            id: "test".to_string(),
            schema_version: "1.1.0".to_string(), // Minor version change OK
            rules: vec![],
            patterns: vec![],
            constraints: Constraints {
                max_execution_time_us: 2000, // Relaxed constraint OK
                max_memory_bytes: 2048,      // Relaxed constraint OK
                required_capabilities: vec![],
                forbidden_operations: vec![],
            },
            metadata: HashMap::new(),
        };

        assert!(checker.check_compatibility(&old, &compatible));

        let incompatible = DescriptorContent {
            id: "test".to_string(),
            schema_version: "2.0.0".to_string(), // Major version change NOT OK
            rules: vec![],
            patterns: vec![],
            constraints: Constraints {
                max_execution_time_us: 500, // Tightened constraint NOT OK
                max_memory_bytes: 512,
                required_capabilities: vec![],
                forbidden_operations: vec![],
            },
            metadata: HashMap::new(),
        };

        assert!(!checker.check_compatibility(&old, &incompatible));
    }
}
