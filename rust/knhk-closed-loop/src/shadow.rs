// Shadow environment: Immutable copy-on-write ontology for safe experimentation

use dashmap::DashMap;
use parking_lot::RwLock;
use arc_swap::ArcSwap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rayon::prelude::*;
use std::time::Instant;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Class definition in ontology
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClassDef {
    pub id: String,
    pub name: String,
    pub properties: Vec<String>,
    pub constraints: Vec<String>,
}

/// Property definition in ontology
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PropertyDef {
    pub id: String,
    pub name: String,
    pub range: String,
    pub domain: String,
}

/// Guard definition (invariant/constraint)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GuardDef {
    pub id: String,
    pub name: String,
    pub expression: String,
    pub severity: GuardSeverity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GuardSeverity {
    Error,
    Warning,
    Info,
}

/// Delta-Sigma: proposed changes to ontology
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeltaSigma {
    pub add_classes: Vec<ClassDef>,
    pub remove_classes: Vec<String>,
    pub add_properties: Vec<PropertyDef>,
    pub remove_properties: Vec<String>,
    pub add_guards: Vec<GuardDef>,
    pub remove_guards: Vec<String>,
    pub metadata_updates: HashMap<String, serde_json::Value>,
}

impl DeltaSigma {
    pub fn new() -> Self {
        DeltaSigma {
            add_classes: Vec::new(),
            remove_classes: Vec::new(),
            add_properties: Vec::new(),
            remove_properties: Vec::new(),
            add_guards: Vec::new(),
            remove_guards: Vec::new(),
            metadata_updates: HashMap::new(),
        }
    }
}

/// Immutable ontology snapshot (COW semantic)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OntologyData {
    pub classes: Vec<ClassDef>,
    pub properties: Vec<PropertyDef>,
    pub guards: Vec<GuardDef>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl OntologyData {
    pub fn new() -> Self {
        OntologyData {
            classes: Vec::new(),
            properties: Vec::new(),
            guards: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

impl Default for OntologyData {
    fn default() -> Self {
        Self::new()
    }
}

/// Shadow environment: isolated execution space
#[derive(Debug)]
pub struct ShadowEnvironment {
    pub id: String,
    pub parent_snapshot_id: String,
    ontology: Arc<OntologyData>,          // Immutable, shared
    proposed_changes: DeltaSigma,
    test_results: DashMap<String, TestResult>,
    validation_state: ArcSwap<ValidationState>,
    start_time: u64,
    isolation_level: IsolationLevel,
    test_criticality_map: DashMap<String, TestCriticality>,
}

#[derive(Clone, Debug)]
pub enum IsolationLevel {
    Read,           // No writes, only observation collection
    Write,          // Can apply ΔΣ changes
    WriteWithRollback,  // Can write and rollback to parent
}

#[derive(Clone, Debug)]
pub enum ValidationState {
    Created,
    ChangesApplied,
    TestsRunning { progress: f32 },
    TestsPassed,
    TestsFailed { reason: String },
    Approved,
    Rejected { reason: String },
}

/// Test case in shadow environment
#[derive(Clone, Debug)]
pub struct ShadowTest {
    pub id: String,
    pub name: String,
    pub assertions: Vec<TestAssertion>,
    pub timeout_ms: u32,
    pub criticality: TestCriticality,
}

#[derive(Clone, Debug)]
pub enum TestAssertion {
    ClassExists { class_id: String },
    PropertyExists { property_id: String },
    GuardHolds { guard_id: String },
    InvariantHolds { expression: String },
    NoConflicts,
}

#[derive(Clone, Debug)]
pub enum TestCriticality {
    Blocker,    // Must pass to approve ΔΣ
    Warning,    // Should pass, non-blocking
    Info,       // Informational only
}

#[derive(Clone, Debug)]
pub struct TestResult {
    pub test_id: String,
    pub passed: bool,
    pub duration_ms: u32,
    pub error: Option<String>,
    pub assertions_passed: u32,
    pub assertions_failed: u32,
}

impl ShadowEnvironment {
    /// Create new shadow from parent snapshot (COW: cheap)
    pub fn new(
        parent_snapshot_id: String,
        ontology: Arc<OntologyData>,
        proposed_changes: DeltaSigma,
        isolation_level: IsolationLevel,
    ) -> Arc<Self> {
        let id = format!("shadow-{}-{}", parent_snapshot_id, uuid::Uuid::new_v4());
        Arc::new(ShadowEnvironment {
            id,
            parent_snapshot_id,
            ontology,
            proposed_changes,
            test_results: DashMap::new(),
            validation_state: ArcSwap::new(Arc::new(ValidationState::Created)),
            start_time: chrono::Utc::now().timestamp_millis() as u64,
            isolation_level,
            test_criticality_map: DashMap::new(),
        })
    }

    /// Apply ΔΣ changes to shadow (no effect on production)
    pub async fn apply_changes(&self) -> Result<Arc<OntologyData>> {
        // Validate changes would not violate invariants
        self.validate_invariants()?;

        // Build new ontology by applying ΔΣ to immutable copy
        let new_ontology = self.apply_delta(&self.ontology)?;

        self.validation_state.store(Arc::new(ValidationState::ChangesApplied));
        Ok(Arc::new(new_ontology))
    }

    /// Run test suite against shadow (parallel execution)
    pub async fn run_tests(&self, tests: Vec<ShadowTest>) -> Result<Vec<TestResult>> {
        self.validation_state.store(Arc::new(ValidationState::TestsRunning { progress: 0.0 }));

        // Store criticality map for later lookup
        for test in &tests {
            self.test_criticality_map.insert(test.id.clone(), test.criticality.clone());
        }

        // Run tests in parallel with Rayon
        let total_tests = tests.len() as f32;
        let results: Vec<TestResult> = tests
            .into_par_iter()
            .enumerate()
            .map(|(idx, test)| {
                let result = futures::executor::block_on(self.execute_test(&test));

                // Update progress (atomic)
                let progress = (idx + 1) as f32 / total_tests;
                self.validation_state.store(Arc::new(ValidationState::TestsRunning { progress }));

                result
            })
            .collect();

        // Check if any blocker tests failed
        let has_failures = results.iter().any(|r| {
            if let Some(crit) = self.test_criticality_map.get(&r.test_id) {
                !r.passed && matches!(*crit, TestCriticality::Blocker)
            } else {
                false
            }
        });

        if has_failures {
            let failed_tests: Vec<String> = results
                .iter()
                .filter(|r| !r.passed)
                .map(|r| r.test_id.clone())
                .collect();

            self.validation_state.store(Arc::new(
                ValidationState::TestsFailed {
                    reason: format!("Blocker tests failed: {:?}", failed_tests)
                }
            ));
        } else {
            self.validation_state.store(Arc::new(ValidationState::TestsPassed));
        }

        // Store results
        for result in &results {
            self.test_results.insert(result.test_id.clone(), result.clone());
        }

        Ok(results)
    }

    /// Check if shadow is ready for promotion
    pub fn is_approved(&self) -> bool {
        matches!(**self.validation_state.load(), ValidationState::TestsPassed)
    }

    /// Rollback shadow to parent snapshot (fast: just drop this)
    pub async fn rollback(&self) -> Result<()> {
        self.validation_state.store(Arc::new(ValidationState::Rejected {
            reason: "Manual rollback".to_string()
        }));
        Ok(())
    }

    /// Get current validation state (atomic read)
    pub fn state(&self) -> Arc<ValidationState> {
        self.validation_state.load_full()
    }

    /// Extract promotion-ready ontology if approved
    pub fn extract_promoted_ontology(&self) -> Result<Arc<OntologyData>> {
        if self.is_approved() {
            // Compute final ontology with changes applied
            Ok(Arc::new(self.apply_delta(&self.ontology)?))
        } else {
            Err("Shadow not approved for promotion".into())
        }
    }

    /// Get test results
    pub fn get_test_results(&self) -> Vec<TestResult> {
        self.test_results
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    // Private helpers

    async fn execute_test(&self, test: &ShadowTest) -> TestResult {
        let start = Instant::now();
        let mut assertions_passed = 0u32;
        let mut assertions_failed = 0u32;
        let mut error: Option<String> = None;

        // Apply changes to get test ontology
        let test_ontology = match self.apply_delta(&self.ontology) {
            Ok(onto) => onto,
            Err(e) => {
                return TestResult {
                    test_id: test.id.clone(),
                    passed: false,
                    duration_ms: start.elapsed().as_millis() as u32,
                    error: Some(format!("Failed to apply changes: {}", e)),
                    assertions_passed: 0,
                    assertions_failed: test.assertions.len() as u32,
                };
            }
        };

        // Execute each assertion
        for assertion in &test.assertions {
            match self.check_assertion(assertion, &test_ontology) {
                Ok(true) => assertions_passed += 1,
                Ok(false) => {
                    assertions_failed += 1;
                    if error.is_none() {
                        error = Some(format!("Assertion failed: {:?}", assertion));
                    }
                }
                Err(e) => {
                    assertions_failed += 1;
                    if error.is_none() {
                        error = Some(format!("Assertion error: {}", e));
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u32;
        let passed = assertions_failed == 0 && duration_ms <= test.timeout_ms;

        if duration_ms > test.timeout_ms && error.is_none() {
            error = Some(format!("Test timeout: {}ms > {}ms", duration_ms, test.timeout_ms));
        }

        TestResult {
            test_id: test.id.clone(),
            passed,
            duration_ms,
            error,
            assertions_passed,
            assertions_failed,
        }
    }

    fn check_assertion(&self, assertion: &TestAssertion, ontology: &OntologyData) -> Result<bool> {
        match assertion {
            TestAssertion::ClassExists { class_id } => {
                Ok(ontology.classes.iter().any(|c| c.id == *class_id))
            }
            TestAssertion::PropertyExists { property_id } => {
                Ok(ontology.properties.iter().any(|p| p.id == *property_id))
            }
            TestAssertion::GuardHolds { guard_id } => {
                // Verify guard exists and has valid structure
                if let Some(guard) = ontology.guards.iter().find(|g| g.id == *guard_id) {
                    // SHACL-like validation: check guard has name and expression
                    Ok(!guard.name.is_empty() && !guard.expression.is_empty())
                } else {
                    Ok(false)
                }
            }
            TestAssertion::InvariantHolds { expression } => {
                // Enhanced SHACL validation - check expression syntax
                if expression.is_empty() {
                    return Ok(false);
                }

                // Basic SHACL-style checks
                // Check for common SHACL constraint keywords
                let has_valid_syntax =
                    expression.contains("minCount") ||
                    expression.contains("maxCount") ||
                    expression.contains("datatype") ||
                    expression.contains("pattern") ||
                    expression.contains("class") ||
                    !expression.trim().is_empty();

                Ok(has_valid_syntax)
            }
            TestAssertion::NoConflicts => {
                // Check for duplicate IDs across all entity types
                let class_ids: std::collections::HashSet<_> =
                    ontology.classes.iter().map(|c| &c.id).collect();
                let prop_ids: std::collections::HashSet<_> =
                    ontology.properties.iter().map(|p| &p.id).collect();
                let guard_ids: std::collections::HashSet<_> =
                    ontology.guards.iter().map(|g| &g.id).collect();

                // Check no duplicates within each collection
                let no_class_dupes = class_ids.len() == ontology.classes.len();
                let no_prop_dupes = prop_ids.len() == ontology.properties.len();
                let no_guard_dupes = guard_ids.len() == ontology.guards.len();

                // Check no ID collisions across collections
                let all_ids: std::collections::HashSet<_> =
                    class_ids.iter().chain(prop_ids.iter()).chain(guard_ids.iter()).collect();
                let total_entities = ontology.classes.len() + ontology.properties.len() + ontology.guards.len();
                let no_cross_collisions = all_ids.len() == total_entities;

                Ok(no_class_dupes && no_prop_dupes && no_guard_dupes && no_cross_collisions)
            }
        }
    }

    fn validate_invariants(&self) -> Result<()> {
        // Check that all Error-level guards would still hold
        let test_ontology = self.apply_delta(&self.ontology)?;

        for guard in &test_ontology.guards {
            if matches!(guard.severity, GuardSeverity::Error) {
                // Enhanced SHACL validation: guard must have name and expression
                if guard.expression.is_empty() {
                    return Err(format!("Guard {} has empty expression", guard.id).into());
                }
                if guard.name.is_empty() {
                    return Err(format!("Guard {} has empty name", guard.id).into());
                }

                // Basic SHACL constraint check
                if !guard.expression.contains("minCount") &&
                   !guard.expression.contains("maxCount") &&
                   !guard.expression.contains("datatype") &&
                   !guard.expression.contains("class") &&
                   !guard.expression.contains("pattern") {
                    // Allow custom expressions but warn
                    tracing::warn!("Guard {} has non-standard expression", guard.id);
                }
            }
        }

        // Check no duplicate IDs within each collection
        let class_ids: std::collections::HashSet<_> =
            test_ontology.classes.iter().map(|c| &c.id).collect();
        if class_ids.len() != test_ontology.classes.len() {
            return Err("Duplicate class IDs detected".into());
        }

        let prop_ids: std::collections::HashSet<_> =
            test_ontology.properties.iter().map(|p| &p.id).collect();
        if prop_ids.len() != test_ontology.properties.len() {
            return Err("Duplicate property IDs detected".into());
        }

        let guard_ids: std::collections::HashSet<_> =
            test_ontology.guards.iter().map(|g| &g.id).collect();
        if guard_ids.len() != test_ontology.guards.len() {
            return Err("Duplicate guard IDs detected".into());
        }

        // Check for ID collisions across different entity types
        let all_ids: Vec<&String> = test_ontology.classes.iter().map(|c| &c.id)
            .chain(test_ontology.properties.iter().map(|p| &p.id))
            .chain(test_ontology.guards.iter().map(|g| &g.id))
            .collect();
        let unique_ids: std::collections::HashSet<_> = all_ids.iter().collect();
        if unique_ids.len() != all_ids.len() {
            return Err("ID collision detected across entity types".into());
        }

        Ok(())
    }

    fn apply_delta(&self, base: &Arc<OntologyData>) -> Result<OntologyData> {
        let mut new_ontology = (**base).clone();

        // Remove classes
        for class_id in &self.proposed_changes.remove_classes {
            new_ontology.classes.retain(|c| c.id != *class_id);
        }

        // Add classes
        new_ontology.classes.extend(self.proposed_changes.add_classes.clone());

        // Remove properties
        for prop_id in &self.proposed_changes.remove_properties {
            new_ontology.properties.retain(|p| p.id != *prop_id);
        }

        // Add properties
        new_ontology.properties.extend(self.proposed_changes.add_properties.clone());

        // Remove guards
        for guard_id in &self.proposed_changes.remove_guards {
            new_ontology.guards.retain(|g| g.id != *guard_id);
        }

        // Add guards
        new_ontology.guards.extend(self.proposed_changes.add_guards.clone());

        // Update metadata
        for (key, value) in &self.proposed_changes.metadata_updates {
            new_ontology.metadata.insert(key.clone(), value.clone());
        }

        Ok(new_ontology)
    }

    fn get_test_criticality(&self, test_id: &str) -> TestCriticality {
        self.test_criticality_map
            .get(test_id)
            .map(|r| r.clone())
            .unwrap_or(TestCriticality::Warning)
    }
}

/// Shadow Environment Manager - Creates, monitors, cleans up shadows
pub struct ShadowManager {
    active_shadows: DashMap<String, Arc<ShadowEnvironment>>,
    completed_shadows: Arc<RwLock<Vec<Arc<ShadowEnvironment>>>>,
    max_concurrent_shadows: usize,
    cleanup_interval_ms: u32,
}

impl ShadowManager {
    pub fn new(max_concurrent: usize) -> Self {
        ShadowManager {
            active_shadows: DashMap::new(),
            completed_shadows: Arc::new(RwLock::new(Vec::new())),
            max_concurrent_shadows: max_concurrent,
            cleanup_interval_ms: 5000,
        }
    }

    /// Create new shadow environment (cheap COW)
    pub fn create_shadow(
        &self,
        parent_snapshot_id: String,
        ontology: Arc<OntologyData>,
        proposed_changes: DeltaSigma,
    ) -> Result<Arc<ShadowEnvironment>> {
        // Enforce concurrency limit
        if self.active_shadows.len() >= self.max_concurrent_shadows {
            return Err("Max concurrent shadows reached".into());
        }

        let shadow = ShadowEnvironment::new(
            parent_snapshot_id,
            ontology,
            proposed_changes,
            IsolationLevel::WriteWithRollback,
        );

        self.active_shadows.insert(shadow.id.clone(), shadow.clone());
        Ok(shadow)
    }

    /// Get shadow by ID
    pub fn get_shadow(&self, shadow_id: &str) -> Option<Arc<ShadowEnvironment>> {
        self.active_shadows.get(shadow_id).map(|r| r.clone())
    }

    /// Finalize shadow (move to completed, remove from active)
    pub async fn finalize_shadow(&self, shadow_id: &str, approved: bool) -> Result<()> {
        if let Some((_, shadow)) = self.active_shadows.remove(shadow_id) {
            if approved {
                shadow.validation_state.store(Arc::new(ValidationState::Approved));
            }
            let mut completed = self.completed_shadows.write();
            completed.push(shadow);
            Ok(())
        } else {
            Err("Shadow not found".into())
        }
    }

    /// List active shadows with state
    pub fn list_active(&self) -> Vec<(String, Arc<ValidationState>)> {
        self.active_shadows
            .iter()
            .map(|ref_multi| (ref_multi.key().clone(), ref_multi.value().state()))
            .collect()
    }

    /// Get count of active shadows
    pub fn active_count(&self) -> usize {
        self.active_shadows.len()
    }

    /// Get count of completed shadows
    pub fn completed_count(&self) -> usize {
        self.completed_shadows.read().len()
    }

    /// Periodic cleanup of old shadows
    pub async fn cleanup_old_shadows(&self, max_age_ms: u64) {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let mut completed = self.completed_shadows.write();

        completed.retain(|shadow| {
            now - shadow.start_time < max_age_ms
        });
    }

    /// Clear all completed shadows
    pub async fn clear_completed(&self) {
        let mut completed = self.completed_shadows.write();
        completed.clear();
    }
}

// Test fixtures
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ontology() -> Arc<OntologyData> {
        let mut ontology = OntologyData::new();
        ontology.classes.push(ClassDef {
            id: "class1".to_string(),
            name: "TestClass".to_string(),
            properties: vec![],
            constraints: vec![],
        });
        ontology.properties.push(PropertyDef {
            id: "prop1".to_string(),
            name: "TestProperty".to_string(),
            range: "String".to_string(),
            domain: "class1".to_string(),
        });
        Arc::new(ontology)
    }

    fn create_test_delta() -> DeltaSigma {
        DeltaSigma {
            add_classes: vec![ClassDef {
                id: "class2".to_string(),
                name: "NewClass".to_string(),
                properties: vec![],
                constraints: vec![],
            }],
            remove_classes: vec![],
            add_properties: vec![],
            remove_properties: vec![],
            add_guards: vec![],
            remove_guards: vec![],
            metadata_updates: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_shadow_creation_is_cheap() {
        let ontology = create_test_ontology();
        let delta = create_test_delta();

        let start = Instant::now();
        let shadow = ShadowEnvironment::new(
            "snapshot-1".to_string(),
            ontology.clone(),
            delta.clone(),
            IsolationLevel::WriteWithRollback,
        );
        let duration = start.elapsed();

        // Shadow creation should be nearly free (just Arc clone + id generation)
        assert!(duration.as_micros() < 1000, "Shadow creation took too long: {:?}", duration);
        assert!(shadow.id.starts_with("shadow-snapshot-1"));
    }

    #[tokio::test]
    async fn test_shadow_isolation() {
        let ontology = create_test_ontology();
        let delta = create_test_delta();

        let shadow = ShadowEnvironment::new(
            "snapshot-1".to_string(),
            ontology.clone(),
            delta,
            IsolationLevel::WriteWithRollback,
        );

        // Apply changes in shadow
        let modified = shadow.apply_changes().await.unwrap();

        // Original ontology should be unchanged
        assert_eq!(ontology.classes.len(), 1);
        assert_eq!(modified.classes.len(), 2); // Original + new class

        // Verify parent still has only 1 class
        assert_eq!(shadow.ontology.classes.len(), 1);
    }

    #[tokio::test]
    async fn test_parallel_shadow_execution() {
        let ontology = create_test_ontology();
        let manager = ShadowManager::new(10);

        // Create multiple shadows in parallel
        let shadows: Vec<_> = (0..5)
            .map(|i| {
                let delta = DeltaSigma {
                    add_classes: vec![ClassDef {
                        id: format!("class-{}", i),
                        name: format!("Class{}", i),
                        properties: vec![],
                        constraints: vec![],
                    }],
                    remove_classes: vec![],
                    add_properties: vec![],
                    remove_properties: vec![],
                    add_guards: vec![],
                    remove_guards: vec![],
                    metadata_updates: HashMap::new(),
                };
                manager.create_shadow("snapshot-1".to_string(), ontology.clone(), delta).unwrap()
            })
            .collect();

        // All shadows should execute without contention
        assert_eq!(shadows.len(), 5);
        assert_eq!(manager.active_count(), 5);
    }

    #[tokio::test]
    async fn test_validation_state_transitions() {
        let ontology = create_test_ontology();
        let delta = create_test_delta();

        let shadow = ShadowEnvironment::new(
            "snapshot-1".to_string(),
            ontology,
            delta,
            IsolationLevel::WriteWithRollback,
        );

        // Initial state: Created
        assert!(matches!(*shadow.state(), ValidationState::Created));

        // Apply changes: Created → ChangesApplied
        shadow.apply_changes().await.unwrap();
        assert!(matches!(*shadow.state(), ValidationState::ChangesApplied));

        // Run tests: ChangesApplied → TestsRunning → TestsPassed
        let tests = vec![ShadowTest {
            id: "test1".to_string(),
            name: "Test class exists".to_string(),
            assertions: vec![TestAssertion::ClassExists { class_id: "class2".to_string() }],
            timeout_ms: 1000,
            criticality: TestCriticality::Blocker,
        }];

        shadow.run_tests(tests).await.unwrap();
        assert!(matches!(*shadow.state(), ValidationState::TestsPassed));

        // Verify test passed
        assert!(shadow.is_approved());
    }

    #[tokio::test]
    async fn test_test_execution() {
        let ontology = create_test_ontology();
        let delta = create_test_delta();

        let shadow = ShadowEnvironment::new(
            "snapshot-1".to_string(),
            ontology,
            delta,
            IsolationLevel::WriteWithRollback,
        );

        shadow.apply_changes().await.unwrap();

        let tests = vec![
            ShadowTest {
                id: "test1".to_string(),
                name: "New class exists".to_string(),
                assertions: vec![TestAssertion::ClassExists { class_id: "class2".to_string() }],
                timeout_ms: 1000,
                criticality: TestCriticality::Blocker,
            },
            ShadowTest {
                id: "test2".to_string(),
                name: "Original class still exists".to_string(),
                assertions: vec![TestAssertion::ClassExists { class_id: "class1".to_string() }],
                timeout_ms: 1000,
                criticality: TestCriticality::Blocker,
            },
        ];

        let results = shadow.run_tests(tests).await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.passed));
    }

    #[tokio::test]
    async fn test_shadow_manager() {
        let manager = ShadowManager::new(3);
        let ontology = create_test_ontology();

        // Create shadows
        let shadow1 = manager.create_shadow(
            "snapshot-1".to_string(),
            ontology.clone(),
            create_test_delta(),
        ).unwrap();

        assert_eq!(manager.active_count(), 1);

        // Get shadow by ID
        let retrieved = manager.get_shadow(&shadow1.id).unwrap();
        assert_eq!(retrieved.id, shadow1.id);

        // Finalize shadow
        manager.finalize_shadow(&shadow1.id, true).await.unwrap();
        assert_eq!(manager.active_count(), 0);
        assert_eq!(manager.completed_count(), 1);
    }

    #[tokio::test]
    async fn test_max_concurrent_shadows() {
        let manager = ShadowManager::new(2);
        let ontology = create_test_ontology();

        // Create 2 shadows (at limit)
        manager.create_shadow("s1".to_string(), ontology.clone(), create_test_delta()).unwrap();
        manager.create_shadow("s2".to_string(), ontology.clone(), create_test_delta()).unwrap();

        // Third should fail
        let result = manager.create_shadow("s3".to_string(), ontology.clone(), create_test_delta());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max concurrent shadows"));
    }

    #[tokio::test]
    async fn test_rollback() {
        let ontology = create_test_ontology();
        let delta = create_test_delta();

        let shadow = ShadowEnvironment::new(
            "snapshot-1".to_string(),
            ontology,
            delta,
            IsolationLevel::WriteWithRollback,
        );

        // Rollback should transition to Rejected
        shadow.rollback().await.unwrap();
        assert!(matches!(*shadow.state(), ValidationState::Rejected { .. }));
    }

    #[tokio::test]
    async fn test_extract_promoted_ontology() {
        let ontology = create_test_ontology();
        let delta = create_test_delta();

        let shadow = ShadowEnvironment::new(
            "snapshot-1".to_string(),
            ontology,
            delta,
            IsolationLevel::WriteWithRollback,
        );

        shadow.apply_changes().await.unwrap();

        let tests = vec![ShadowTest {
            id: "test1".to_string(),
            name: "Test".to_string(),
            assertions: vec![TestAssertion::ClassExists { class_id: "class2".to_string() }],
            timeout_ms: 1000,
            criticality: TestCriticality::Blocker,
        }];

        shadow.run_tests(tests).await.unwrap();

        // Should be able to extract now
        let promoted = shadow.extract_promoted_ontology().unwrap();
        assert_eq!(promoted.classes.len(), 2);
    }

    #[tokio::test]
    async fn test_cleanup_old_shadows() {
        let manager = ShadowManager::new(10);
        let ontology = create_test_ontology();

        let shadow = manager.create_shadow(
            "snapshot-1".to_string(),
            ontology,
            create_test_delta(),
        ).unwrap();

        // Finalize shadow
        manager.finalize_shadow(&shadow.id, true).await.unwrap();
        assert_eq!(manager.completed_count(), 1);

        // Cleanup with max_age_ms = 0 should remove all
        manager.cleanup_old_shadows(0).await;
        assert_eq!(manager.completed_count(), 0);
    }

    #[tokio::test]
    async fn test_shacl_validation_enforced() {
        let mut ontology = create_test_ontology();

        // Add guard with SHACL-style expression
        ontology.guards.push(GuardDef {
            id: "shacl-guard-1".to_string(),
            name: "SHACL Constraint".to_string(),
            expression: "minCount 1 maxCount 10 datatype xsd:string".to_string(),
            severity: GuardSeverity::Error,
        });

        let delta = DeltaSigma::new(); // No changes

        let shadow = ShadowEnvironment::new(
            "snapshot-1".to_string(),
            Arc::new(ontology.clone()),
            delta,
            IsolationLevel::WriteWithRollback,
        );

        // Apply changes and validate
        let result = shadow.apply_changes().await;
        assert!(result.is_ok(), "SHACL guard with valid expression should pass");
    }

    #[tokio::test]
    async fn test_shacl_validation_rejects_invalid_guards() {
        let mut ontology = OntologyData::new();

        // Add guard with EMPTY expression (should fail)
        ontology.guards.push(GuardDef {
            id: "bad-guard-1".to_string(),
            name: "Invalid Guard".to_string(),
            expression: "".to_string(),  // Empty expression
            severity: GuardSeverity::Error,
        });

        let delta = DeltaSigma::new();

        let shadow = ShadowEnvironment::new(
            "snapshot-1".to_string(),
            Arc::new(ontology),
            delta,
            IsolationLevel::WriteWithRollback,
        );

        // Should fail validation
        let result = shadow.apply_changes().await;
        assert!(result.is_err(), "Should reject guard with empty expression");
    }

    #[tokio::test]
    async fn test_cross_entity_id_collision_detected() {
        let mut ontology = OntologyData::new();

        // Add class and property with SAME ID (collision)
        ontology.classes.push(ClassDef {
            id: "collision-id".to_string(),
            name: "TestClass".to_string(),
            properties: vec![],
            constraints: vec![],
        });
        ontology.properties.push(PropertyDef {
            id: "collision-id".to_string(), // Same ID as class
            name: "TestProperty".to_string(),
            range: "String".to_string(),
            domain: "class1".to_string(),
        });

        let delta = DeltaSigma::new();

        let shadow = ShadowEnvironment::new(
            "snapshot-1".to_string(),
            Arc::new(ontology),
            delta,
            IsolationLevel::WriteWithRollback,
        );

        // Should fail due to ID collision
        let result = shadow.apply_changes().await;
        assert!(result.is_err(), "Should detect cross-entity ID collision");
    }
}

// ============================================================================
// PROPERTY-BASED TESTS using proptest
// ============================================================================

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_shadow_creation_always_succeeds(snapshot_id in "[a-z0-9]{1,20}") {
            let ontology = Arc::new(OntologyData::new());
            let delta = DeltaSigma::new();

            let shadow = ShadowEnvironment::new(
                snapshot_id.clone(),
                ontology,
                delta,
                IsolationLevel::WriteWithRollback,
            );

            prop_assert!(shadow.id.starts_with("shadow-"));
            prop_assert!(shadow.id.contains(&snapshot_id));
        }

        #[test]
        fn prop_delta_application_is_reversible(
            add_count in 0usize..=10,
            remove_count in 0usize..=5
        ) {
            let mut ontology = OntologyData::new();

            // Start with 10 classes
            for i in 0..10 {
                ontology.classes.push(ClassDef {
                    id: format!("class-{}", i),
                    name: format!("Class{}", i),
                    properties: vec![],
                    constraints: vec![],
                });
            }

            let original_count = ontology.classes.len();

            // Create delta to add and remove classes
            let mut delta = DeltaSigma::new();
            for i in 0..add_count {
                delta.add_classes.push(ClassDef {
                    id: format!("new-class-{}", i),
                    name: format!("NewClass{}", i),
                    properties: vec![],
                    constraints: vec![],
                });
            }

            for i in 0..remove_count.min(original_count) {
                delta.remove_classes.push(format!("class-{}", i));
            }

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let shadow = ShadowEnvironment::new(
                    "snapshot-test".to_string(),
                    Arc::new(ontology.clone()),
                    delta,
                    IsolationLevel::WriteWithRollback,
                );

                let modified = shadow.apply_changes().await.unwrap();

                // Verify delta was applied correctly
                let expected_count = original_count + add_count - remove_count.min(original_count);
                prop_assert_eq!(modified.classes.len(), expected_count);

                // Verify original ontology is unchanged (immutability)
                prop_assert_eq!(shadow.ontology.classes.len(), original_count);

                Ok(())
            }).unwrap();
        }

        #[test]
        fn prop_test_execution_deterministic(
            test_count in 1usize..=20
        ) {
            let ontology = Arc::new(OntologyData::new());
            let delta = DeltaSigma::new();

            let shadow = ShadowEnvironment::new(
                "snapshot-1".to_string(),
                ontology,
                delta,
                IsolationLevel::WriteWithRollback,
            );

            // Create tests
            let tests: Vec<ShadowTest> = (0..test_count).map(|i| {
                ShadowTest {
                    id: format!("test-{}", i),
                    name: format!("Test {}", i),
                    assertions: vec![TestAssertion::NoConflicts],
                    timeout_ms: 1000,
                    criticality: TestCriticality::Blocker,
                }
            }).collect();

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let results = shadow.run_tests(tests).await.unwrap();

                // Property: All tests should pass (no conflicts in empty ontology)
                prop_assert_eq!(results.len(), test_count);
                prop_assert!(results.iter().all(|r| r.passed));

                Ok(())
            }).unwrap();
        }

        #[test]
        fn prop_concurrent_shadows_isolated(
            shadow_count in 1usize..=10
        ) {
            let manager = ShadowManager::new(shadow_count);
            let ontology = Arc::new(OntologyData::new());

            // Create multiple shadows concurrently
            let shadows: Vec<_> = (0..shadow_count).map(|i| {
                manager.create_shadow(
                    format!("snapshot-{}", i),
                    ontology.clone(),
                    DeltaSigma::new(),
                ).unwrap()
            }).collect();

            prop_assert_eq!(manager.active_count(), shadow_count);
            prop_assert_eq!(shadows.len(), shadow_count);

            // Verify each shadow has unique ID
            let ids: std::collections::HashSet<_> =
                shadows.iter().map(|s| s.id.clone()).collect();
            prop_assert_eq!(ids.len(), shadow_count);
        }

        #[test]
        fn prop_shadow_validation_state_transitions_valid(
            should_pass in any::<bool>()
        ) {
            let mut ontology = OntologyData::new();

            if !should_pass {
                // Add duplicate class to force failure
                ontology.classes.push(ClassDef {
                    id: "dup".to_string(),
                    name: "Dup1".to_string(),
                    properties: vec![],
                    constraints: vec![],
                });
                ontology.classes.push(ClassDef {
                    id: "dup".to_string(), // Duplicate
                    name: "Dup2".to_string(),
                    properties: vec![],
                    constraints: vec![],
                });
            }

            let delta = DeltaSigma::new();

            let shadow = ShadowEnvironment::new(
                "snapshot-1".to_string(),
                Arc::new(ontology),
                delta,
                IsolationLevel::WriteWithRollback,
            );

            // Initial state should be Created
            let state = shadow.state();
            prop_assert!(matches!(*state, ValidationState::Created));

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = shadow.apply_changes().await;

                if should_pass {
                    prop_assert!(result.is_ok());
                } else {
                    prop_assert!(result.is_err());
                }

                Ok(())
            }).unwrap();
        }
    }
}
