#![allow(clippy::unwrap_used)] // Test infrastructure - unwrap() acceptable
//! Chicago TDD Test Framework for Workflows
//!
//! Provides a comprehensive framework for writing Chicago TDD tests for workflows following
//! the AAA pattern (Arrange, Act, Assert) with real collaborators.
//!
//! **Architecture**: Uses `chicago-tdd-tools` as generic base, extends with workflow-specific functionality.

use crate::case::{Case, CaseId, CaseState};
use crate::error::{WorkflowError, WorkflowResult};
use crate::executor::WorkflowEngine;
use crate::parser::{JoinType, SplitType, Task, TaskType, WorkflowSpec, WorkflowSpecId};
use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternId, PatternRegistry,
};
use crate::resource::{Capability, Resource, ResourceId, Role};
use crate::state::StateStore;
use crate::worklets::{Worklet, WorkletId, WorkletMetadata};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ============================================================================
// Test Fixture
// ============================================================================

/// Test fixture for workflow testing
pub struct WorkflowTestFixture {
    /// Workflow engine
    pub engine: WorkflowEngine,
    /// Registered workflow specs
    pub specs: HashMap<WorkflowSpecId, WorkflowSpec>,
    /// Created cases
    pub cases: Vec<CaseId>,
    /// Test counter for unique test databases
    test_counter: u64,
}

impl WorkflowTestFixture {
    /// Create a new test fixture with unique database path
    pub fn new() -> WorkflowResult<Self> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let db_path = format!("./test_workflow_db_{}", counter);

        let state_store = StateStore::new(&db_path)?;
        let engine = WorkflowEngine::new(state_store);

        Ok(Self {
            engine,
            specs: HashMap::new(),
            cases: vec![],
            test_counter: counter,
        })
    }

    /// Register a workflow specification
    pub async fn register_workflow(
        &mut self,
        spec: WorkflowSpec,
    ) -> WorkflowResult<WorkflowSpecId> {
        let spec_id = spec.id;
        self.engine.register_workflow(spec.clone()).await?;
        self.specs.insert(spec_id, spec);
        Ok(spec_id)
    }

    /// Create a test case
    pub async fn create_case(
        &mut self,
        spec_id: WorkflowSpecId,
        data: serde_json::Value,
    ) -> WorkflowResult<CaseId> {
        let case_id = self.engine.create_case(spec_id, data).await?;
        self.cases.push(case_id);
        Ok(case_id)
    }

    /// Execute a case and return final state
    pub async fn execute_case(&self, case_id: CaseId) -> WorkflowResult<Case> {
        self.engine.start_case(case_id).await?;
        self.engine.execute_case(case_id).await?;
        self.engine.get_case(case_id).await
    }

    /// Execute a pattern directly
    pub async fn execute_pattern(
        &self,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionResult> {
        self.engine.execute_pattern(pattern_id, context).await
    }

    /// Assert case state
    pub fn assert_case_state(&self, case: &Case, expected_state: CaseState) {
        assert_eq!(
            case.state, expected_state,
            "Expected case state {:?}, but got {:?}",
            expected_state, case.state
        );
    }

    /// Assert case completed successfully
    pub fn assert_case_completed(&self, case: &Case) {
        self.assert_case_state(case, CaseState::Completed);
    }

    /// Assert case failed
    pub fn assert_case_failed(&self, case: &Case) {
        self.assert_case_state(case, CaseState::Failed);
    }

    /// Assert case is running
    pub fn assert_case_running(&self, case: &Case) {
        self.assert_case_state(case, CaseState::Running);
    }

    /// Assert case is cancelled
    pub fn assert_case_cancelled(&self, case: &Case) {
        self.assert_case_state(case, CaseState::Cancelled);
    }

    /// Get case history (for testing)
    pub async fn get_case_history(
        &self,
        case_id: CaseId,
    ) -> WorkflowResult<Vec<crate::state::manager::StateEvent>> {
        Ok(self.engine.state_manager.get_case_history(case_id).await)
    }

    /// Clean up test resources
    pub fn cleanup(&self) -> WorkflowResult<()> {
        // Clean up state store by removing test database
        let db_path = format!("./test_workflow_db_{}", self.test_counter);
        if std::path::Path::new(&db_path).exists() {
            std::fs::remove_dir_all(&db_path).map_err(|e| {
                WorkflowError::StatePersistence(format!("Failed to cleanup test database: {:?}", e))
            })?;
        }
        Ok(())
    }
}

impl Default for WorkflowTestFixture {
    fn default() -> Self {
        Self::new().expect("Failed to create test fixture")
    }
}

// ============================================================================
// Pattern Test Helpers
// ============================================================================

/// Create a test pattern registry with all 43 patterns registered
pub fn create_test_registry() -> PatternRegistry {
    let mut registry = PatternRegistry::new();
    crate::patterns::register_all_patterns(&mut registry);
    registry
}

/// Create a test execution context
pub fn create_test_context() -> PatternExecutionContext {
    PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: HashMap::new(),
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    }
}

/// Create a test execution context with variables
pub fn create_test_context_with_vars(vars: HashMap<String, String>) -> PatternExecutionContext {
    PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id: WorkflowSpecId::new(),
        variables: vars,
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    }
}

/// Create a test execution context with specific workflow ID
pub fn create_test_context_for_workflow(workflow_id: WorkflowSpecId) -> PatternExecutionContext {
    PatternExecutionContext {
        case_id: CaseId::new(),
        workflow_id,
        variables: HashMap::new(),
        arrived_from: std::collections::HashSet::new(),
        scope_id: String::new(),
    }
}

/// Assert pattern execution succeeded
pub fn assert_pattern_success(result: &PatternExecutionResult) {
    assert!(
        result.success,
        "Pattern execution should succeed, but got failure"
    );
}

/// Assert pattern execution failed
pub fn assert_pattern_failure(result: &PatternExecutionResult) {
    assert!(
        !result.success,
        "Pattern execution should fail, but got success"
    );
}

/// Assert pattern has next state
pub fn assert_pattern_has_next_state(result: &PatternExecutionResult) {
    assert!(
        result.next_state.is_some(),
        "Pattern execution should set next state"
    );
}

/// Assert pattern result contains variable
pub fn assert_pattern_has_variable(result: &PatternExecutionResult, key: &str) {
    assert!(
        result.variables.contains_key(key),
        "Pattern result should contain variable '{}'",
        key
    );
}

/// Assert pattern result variable equals expected value
pub fn assert_pattern_variable_equals(result: &PatternExecutionResult, key: &str, expected: &str) {
    assert_pattern_has_variable(result, key);
    let actual = result.variables.get(key).unwrap();
    // Handle JSON string values (with quotes) vs plain strings
    let actual_trimmed = actual.trim_matches('"');
    assert_eq!(
        actual_trimmed, expected,
        "Variable '{}' should equal '{}' (got '{}')",
        key, expected, actual
    );
}

// ============================================================================
// Workflow Builders
// ============================================================================

/// Builder for creating test workflow specifications
pub struct WorkflowSpecBuilder {
    spec: WorkflowSpec,
}

impl WorkflowSpecBuilder {
    /// Create a new workflow spec builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            spec: WorkflowSpec {
                id: WorkflowSpecId::new(),
                name: name.into(),
                tasks: HashMap::new(),
                conditions: HashMap::new(),
                flows: Vec::new(),
                start_condition: None,
                end_condition: None,
                source_turtle: None,
            },
        }
    }

    /// Add a task to the workflow
    pub fn add_task(mut self, task: Task) -> Self {
        self.spec.tasks.insert(task.id.clone(), task);
        self
    }

    /// Set start condition
    pub fn with_start_condition(mut self, condition_id: impl Into<String>) -> Self {
        self.spec.start_condition = Some(condition_id.into());
        self
    }

    /// Set end condition
    pub fn with_end_condition(mut self, condition_id: impl Into<String>) -> Self {
        self.spec.end_condition = Some(condition_id.into());
        self
    }

    /// Build the workflow specification
    pub fn build(self) -> WorkflowSpec {
        self.spec
    }
}

/// Builder for creating test tasks
pub struct TaskBuilder {
    task: Task,
}

impl TaskBuilder {
    /// Create a new task builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            task: Task {
                input_parameters: Vec::new(),
                output_parameters: Vec::new(),
                id: id.into(),
                name: name.into(),
                task_type: TaskType::Atomic,
                split_type: SplitType::And,
                join_type: JoinType::And,
                max_ticks: None,
                priority: None,
                use_simd: false,
                input_conditions: vec![],
                output_conditions: vec![],
                outgoing_flows: vec![],
                incoming_flows: vec![],
                allocation_policy: None,
                required_roles: vec![],
                required_capabilities: vec![],
                exception_worklet: None,
            },
        }
    }

    /// Set task type
    pub fn with_type(mut self, task_type: TaskType) -> Self {
        self.task.task_type = task_type;
        self
    }

    /// Set split type
    pub fn with_split_type(mut self, split_type: SplitType) -> Self {
        self.task.split_type = split_type;
        self
    }

    /// Set join type
    pub fn with_join_type(mut self, join_type: JoinType) -> Self {
        self.task.join_type = join_type;
        self
    }

    /// Set max ticks
    pub fn with_max_ticks(mut self, max_ticks: u32) -> Self {
        self.task.max_ticks = Some(max_ticks);
        self
    }

    /// Add outgoing flow
    pub fn add_outgoing_flow(mut self, task_id: impl Into<String>) -> Self {
        self.task.outgoing_flows.push(task_id.into());
        self
    }

    /// Build the task
    pub fn build(self) -> Task {
        self.task
    }
}

// ============================================================================
// Resource Test Helpers
// ============================================================================

/// Create a test resource
pub fn create_test_resource(
    name: impl Into<String>,
    roles: Vec<Role>,
    capabilities: Vec<Capability>,
) -> Resource {
    Resource {
        id: ResourceId::new(),
        name: name.into(),
        roles,
        capabilities,
        workload: 0,
        queue_length: 0,
        available: true,
    }
}

/// Create a test role
pub fn create_test_role(id: impl Into<String>, name: impl Into<String>) -> Role {
    Role {
        id: id.into(),
        name: name.into(),
        capabilities: vec![],
    }
}

/// Create a test capability
pub fn create_test_capability(
    id: impl Into<String>,
    name: impl Into<String>,
    level: u8,
) -> Capability {
    Capability {
        id: id.into(),
        name: name.into(),
        level,
    }
}

// ============================================================================
// Worklet Test Helpers
// ============================================================================

/// Create a test worklet
pub fn create_test_worklet(name: impl Into<String>, exception_types: Vec<String>) -> Worklet {
    Worklet {
        metadata: WorkletMetadata {
            id: WorkletId::new(),
            name: name.into(),
            description: "Test worklet".to_string(),
            version: "1.0.0".to_string(),
            exception_types,
            required_context: vec![],
            pattern_ids: vec![],
            tags: vec!["test".to_string()],
        },
        workflow_spec: WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: "Test Worklet Workflow".to_string(),
            tasks: HashMap::new(),
            conditions: HashMap::new(),
            flows: Vec::new(),
            start_condition: None,
            end_condition: None,
            source_turtle: None,
        },
        rules: vec![],
    }
}

// ============================================================================
// Performance Test Helpers
// ============================================================================

/// Measure execution time and verify tick budget
pub struct PerformanceTestHelper {
    start_time: Instant,
    max_ticks: u32,
}

impl PerformanceTestHelper {
    /// Create a new performance test helper
    pub fn new(max_ticks: u32) -> Self {
        Self {
            start_time: Instant::now(),
            max_ticks,
        }
    }

    /// Verify execution completed within tick budget
    pub fn verify_tick_budget(&self) {
        let duration = self.start_time.elapsed();
        let ticks = (duration.as_nanos() / 2) as u32; // 2ns per tick

        assert!(
            ticks <= self.max_ticks,
            "Execution took {} ticks, exceeds budget of {} ticks",
            ticks,
            self.max_ticks
        );
    }

    /// Get elapsed ticks
    pub fn elapsed_ticks(&self) -> u32 {
        (self.start_time.elapsed().as_nanos() / 2) as u32
    }
}

// ============================================================================
// Integration Test Helpers
// ============================================================================

/// Helper for testing workflow execution end-to-end
pub struct IntegrationTestHelper {
    fixture: WorkflowTestFixture,
}

impl IntegrationTestHelper {
    /// Create a new integration test helper
    pub fn new() -> WorkflowResult<Self> {
        Ok(Self {
            fixture: WorkflowTestFixture::new()?,
        })
    }

    /// Execute complete workflow: register → create case → execute
    pub async fn execute_complete_workflow(
        &mut self,
        spec: WorkflowSpec,
        data: serde_json::Value,
    ) -> WorkflowResult<Case> {
        let spec_id = self.fixture.register_workflow(spec).await?;
        let case_id = self.fixture.create_case(spec_id, data).await?;
        self.fixture.execute_case(case_id).await
    }

    /// Get the underlying fixture
    pub fn fixture(&self) -> &WorkflowTestFixture {
        &self.fixture
    }

    /// Get mutable access to fixture
    pub fn fixture_mut(&mut self) -> &mut WorkflowTestFixture {
        &mut self.fixture
    }
}

// ============================================================================
// Property-Based Testing
// ============================================================================

/// Property-based test generator for workflows
pub struct WorkflowPropertyTester {
    /// Test fixture
    fixture: WorkflowTestFixture,
    /// Number of test cases to generate
    num_cases: usize,
}

impl WorkflowPropertyTester {
    /// Create a new property tester
    pub fn new(num_cases: usize) -> WorkflowResult<Self> {
        Ok(Self {
            fixture: WorkflowTestFixture::new()?,
            num_cases,
        })
    }

    /// Test workflow property: All cases eventually complete or fail
    pub async fn test_completion_property(
        &mut self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<bool> {
        for _ in 0..self.num_cases {
            let case_id = self
                .fixture
                .create_case(spec_id, serde_json::json!({}))
                .await?;
            let case = self.fixture.execute_case(case_id).await?;

            // Property: Case must be in Completed or Failed state
            if case.state != CaseState::Completed && case.state != CaseState::Failed {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Test workflow property: No deadlocks
    pub async fn test_deadlock_property(
        &mut self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<bool> {
        // Test for deadlocks by checking workflow spec structure
        // Deadlock detection: check for circular dependencies in task flows
        let spec = self.fixture.engine.get_workflow(spec_id).await?;

        // Check for circular dependencies in outgoing_flows
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for task_id in spec.tasks.keys() {
            if self.has_cycle(&spec, task_id, &mut visited, &mut rec_stack) {
                return Ok(false); // Deadlock detected
            }
        }

        Ok(true) // No deadlock detected
    }

    fn has_cycle(
        &self,
        spec: &crate::parser::WorkflowSpec,
        task_id: &str,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        if rec_stack.contains(task_id) {
            return true; // Cycle detected
        }
        if visited.contains(task_id) {
            return false; // Already processed, no cycle
        }

        visited.insert(task_id.to_string());
        rec_stack.insert(task_id.to_string());

        if let Some(task) = spec.tasks.get(task_id) {
            for next_task_id in &task.outgoing_flows {
                if self.has_cycle(spec, next_task_id, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(task_id);
        false
    }

    /// Test workflow property: Tick budget compliance
    pub async fn test_tick_budget_property(
        &mut self,
        spec_id: WorkflowSpecId,
    ) -> WorkflowResult<bool> {
        // Verify all tasks complete in ≤8 ticks by checking task max_ticks constraints
        let spec = self.fixture.engine.get_workflow(spec_id).await?;

        // Check all tasks have max_ticks ≤ 8 (Chatman Constant)
        for task in spec.tasks.values() {
            if let Some(max_ticks) = task.max_ticks {
                if max_ticks > 8 {
                    return Ok(false); // Task exceeds tick budget
                }
            }
        }

        Ok(true) // All tasks comply with tick budget
    }
}

// ============================================================================
// Test Macros
// ============================================================================

/// Macro for Chicago TDD workflow tests
#[macro_export]
macro_rules! chicago_tdd_workflow_test {
    ($name:ident, $test_fn:expr) => {
        #[tokio::test]
        async fn $name() {
            // Arrange: Set up test fixture
            let mut fixture = $crate::testing::chicago_tdd::WorkflowTestFixture::new()
                .expect("Failed to create test fixture");

            // Execute test function
            $test_fn(&mut fixture).await.expect("Test failed");

            // Cleanup
            fixture.cleanup().expect("Failed to cleanup");
        }
    };
}

/// Macro for pattern tests
#[macro_export]
macro_rules! chicago_tdd_pattern_test {
    ($name:ident, $pattern_id:expr, $test_fn:expr) => {
        #[test]
        fn $name() {
            // Arrange: Create registry and context
            let registry = $crate::testing::chicago_tdd::create_test_registry();
            let ctx = $crate::testing::chicago_tdd::create_test_context();

            // Act: Execute pattern
            let result = registry
                .execute(&$crate::patterns::PatternId($pattern_id), &ctx)
                .expect(&format!("Pattern {} should be registered", $pattern_id));

            // Assert: Verify result
            $test_fn(&result);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::builders::TestDataBuilder;

    #[tokio::test]
    async fn test_fixture_creation() {
        let fixture = WorkflowTestFixture::new().unwrap();
        assert!(fixture.specs.is_empty());
        assert!(fixture.cases.is_empty());
    }

    #[test]
    fn test_create_test_registry() {
        let registry = create_test_registry();
        assert!(registry.has_pattern(&PatternId(1)));
    }

    #[test]
    fn test_create_test_context() {
        let ctx = create_test_context();
        assert_eq!(ctx.variables.len(), 0);
    }

    #[test]
    fn test_test_data_builder() {
        let data = TestDataBuilder::new()
            .with_order_data("ORD-001", "100.00")
            .with_customer_data("CUST-001")
            .build_json();

        assert_eq!(data["order_id"], "ORD-001");
        assert_eq!(data["total_amount"], "100.00");
    }
}
