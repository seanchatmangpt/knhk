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
use crate::parser::{Condition, JoinType, SplitType, Task, TaskType, WorkflowSpec, WorkflowSpecId};
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
    /// Note: This accesses the internal state_manager which is pub(crate)
    pub async fn get_case_history(
        &self,
        case_id: CaseId,
    ) -> Vec<crate::state::manager::StateEvent> {
        // Access through engine's state_manager field (pub(crate) allows crate access)
        // Since tests are in the same crate, we can access it
        self.engine.state_manager.get_case_history(case_id).await
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

    /// Add a condition to the workflow
    pub fn add_condition(mut self, condition: Condition) -> Self {
        self.spec.conditions.insert(condition.id.clone(), condition);
        self
    }

    /// Automatically set up start and end conditions based on task IDs
    /// This creates condition objects and connects them to the first and last tasks
    ///
    /// # Arguments
    /// * `start_task_id` - ID of the first task in the workflow
    /// * `end_task_id` - ID of the last task in the workflow
    pub fn with_auto_conditions(
        mut self,
        start_task_id: impl Into<String>,
        end_task_id: impl Into<String>,
    ) -> Self {
        let start_id = start_task_id.into();
        let end_id = end_task_id.into();

        // Create start condition
        let start_condition = Condition {
            id: format!("condition:{}", start_id),
            name: "Start".to_string(),
            outgoing_flows: vec![start_id.clone()],
            incoming_flows: vec![],
        };
        self.spec
            .conditions
            .insert(start_condition.id.clone(), start_condition.clone());
        self.spec.start_condition = Some(start_condition.id.clone());

        // Update first task to have input condition
        if let Some(task) = self.spec.tasks.get_mut(&start_id) {
            task.input_conditions.push(start_condition.id.clone());
        }

        // Create end condition
        let end_condition = Condition {
            id: format!("condition:{}", end_id),
            name: "End".to_string(),
            outgoing_flows: vec![],
            incoming_flows: vec![end_id.clone()],
        };
        self.spec
            .conditions
            .insert(end_condition.id.clone(), end_condition.clone());
        self.spec.end_condition = Some(end_condition.id.clone());

        // Update last task to have output condition
        if let Some(task) = self.spec.tasks.get_mut(&end_id) {
            task.output_conditions.push(end_condition.id.clone());
        }

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

    /// Add a flow to the workflow
    pub fn add_flow(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        use crate::parser::Flow;
        let from_str = from.into();
        let to_str = to.into();
        self.spec.flows.push(Flow {
            id: format!("flow:{}:{}", from_str, to_str),
            from: from_str,
            to: to_str,
            predicate: None,
        });
        self
    }

    /// Add a flow with a specific ID
    pub fn add_flow_with_id(
        mut self,
        flow_id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        use crate::parser::Flow;
        self.spec.flows.push(Flow {
            id: flow_id.into(),
            from: from.into(),
            to: to.into(),
            predicate: None,
        });
        self
    }

    /// Add a flow with a predicate
    pub fn add_flow_with_predicate(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        predicate: impl Into<String>,
    ) -> Self {
        use crate::parser::Flow;
        let from_str = from.into();
        let to_str = to.into();
        self.spec.flows.push(Flow {
            id: format!("flow:{}:{}", from_str, to_str),
            from: from_str,
            to: to_str,
            predicate: Some(predicate.into()),
        });
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

    /// Add input condition to task
    pub fn with_input_condition(mut self, condition_id: impl Into<String>) -> Self {
        self.task.input_conditions.push(condition_id.into());
        self
    }

    /// Add output condition to task
    pub fn with_output_condition(mut self, condition_id: impl Into<String>) -> Self {
        self.task.output_conditions.push(condition_id.into());
        self
    }

    /// Build the task
    pub fn build(self) -> Task {
        self.task
    }
}

// ============================================================================
// Condition Builder
// ============================================================================

/// Builder for creating test conditions
pub struct ConditionBuilder {
    condition: Condition,
}

impl ConditionBuilder {
    /// Create a new condition builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            condition: Condition {
                id: id.into(),
                name: name.into(),
                outgoing_flows: vec![],
                incoming_flows: vec![],
            },
        }
    }

    /// Add outgoing flow (to task)
    pub fn add_outgoing_flow(mut self, task_id: impl Into<String>) -> Self {
        self.condition.outgoing_flows.push(task_id.into());
        self
    }

    /// Add incoming flow (from task)
    pub fn add_incoming_flow(mut self, task_id: impl Into<String>) -> Self {
        self.condition.incoming_flows.push(task_id.into());
        self
    }

    /// Build the condition
    pub fn build(self) -> Condition {
        self.condition
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
// Workflow Helper Functions
// ============================================================================

/// Create a simple sequential workflow with proper conditions
/// This helper creates a workflow with start → task → end structure
///
/// # Arguments
/// * `name` - Workflow name
/// * `task_id` - ID of the single task
/// * `task_name` - Name of the single task
pub fn create_simple_sequential_workflow(
    name: impl Into<String>,
    task_id: impl Into<String>,
    task_name: impl Into<String>,
) -> WorkflowSpec {
    let task_id_str = task_id.into();
    let start_condition_id = "condition:start".to_string();
    let end_condition_id = "condition:end".to_string();

    let task = TaskBuilder::new(task_id_str.clone(), task_name)
        .with_input_condition(start_condition_id.clone())
        .with_output_condition(end_condition_id.clone())
        .build();

    let start_condition = ConditionBuilder::new(start_condition_id.clone(), "Start")
        .add_outgoing_flow(task_id_str.clone())
        .build();

    let end_condition = ConditionBuilder::new(end_condition_id.clone(), "End")
        .add_incoming_flow(task_id_str.clone())
        .build();

    WorkflowSpecBuilder::new(name)
        .add_task(task)
        .add_condition(start_condition)
        .add_condition(end_condition)
        .with_start_condition(start_condition_id)
        .with_end_condition(end_condition_id)
        .build()
}

/// Create a sequential workflow using flows (not conditions)
/// This is useful for tests that need Flow objects explicitly
/// Creates: start → task1 → task2 → ... → end
///
/// # Arguments
/// * `name` - Workflow name
/// * `task_ids` - Vector of (task_id, task_name) tuples
pub fn create_sequential_workflow_with_flows(
    name: impl Into<String>,
    task_ids: Vec<(String, String)>,
) -> WorkflowSpec {
    if task_ids.is_empty() {
        panic!("Cannot create workflow with no tasks");
    }

    let mut builder = WorkflowSpecBuilder::new(name);

    // Create tasks
    for (idx, (task_id, task_name)) in task_ids.iter().enumerate() {
        let mut task_builder = TaskBuilder::new(task_id.clone(), task_name.clone());

        // Set up incoming/outgoing flows
        if idx > 0 {
            let prev_task_id = &task_ids[idx - 1].0;
            task_builder =
                task_builder.add_outgoing_flow(format!("{}_to_{}", prev_task_id, task_id));
        }
        if idx < task_ids.len() - 1 {
            let next_task_id = &task_ids[idx + 1].0;
            task_builder =
                task_builder.add_outgoing_flow(format!("{}_to_{}", task_id, next_task_id));
        }

        builder = builder.add_task(task_builder.build());
    }

    // Create flows
    builder = builder.add_flow_with_id("start_to_first", "start", &task_ids[0].0);
    for idx in 0..task_ids.len() - 1 {
        let flow_id = format!("{}_to_{}", task_ids[idx].0, task_ids[idx + 1].0);
        builder = builder.add_flow_with_id(flow_id, &task_ids[idx].0, &task_ids[idx + 1].0);
    }
    builder = builder.add_flow_with_id("last_to_end", &task_ids.last().unwrap().0, "end");

    builder
        .with_start_condition("start")
        .with_end_condition("end")
        .build()
}

/// Create a workflow with multiple tasks in sequence with proper conditions
///
/// # Arguments
/// * `name` - Workflow name
/// * `task_ids` - Vector of (task_id, task_name) tuples
pub fn create_sequential_workflow(
    name: impl Into<String>,
    task_ids: Vec<(String, String)>,
) -> WorkflowSpec {
    if task_ids.is_empty() {
        panic!("Cannot create workflow with no tasks");
    }

    let start_condition_id = "condition:start".to_string();
    let end_condition_id = "condition:end".to_string();

    let mut builder = WorkflowSpecBuilder::new(name);

    // Create start condition
    let start_condition = ConditionBuilder::new(start_condition_id.clone(), "Start")
        .add_outgoing_flow(task_ids[0].0.clone())
        .build();
    builder = builder.add_condition(start_condition);

    // Create tasks with proper connections
    for (idx, (task_id, task_name)) in task_ids.iter().enumerate() {
        let mut task_builder = TaskBuilder::new(task_id.clone(), task_name.clone());

        // First task gets start condition as input
        if idx == 0 {
            task_builder = task_builder.with_input_condition(start_condition_id.clone());
        }

        // Last task gets end condition as output
        if idx == task_ids.len() - 1 {
            task_builder = task_builder.with_output_condition(end_condition_id.clone());
        }

        builder = builder.add_task(task_builder.build());
    }

    // Create end condition
    let end_condition = ConditionBuilder::new(end_condition_id.clone(), "End")
        .add_incoming_flow(task_ids.last().unwrap().0.clone())
        .build();
    builder = builder.add_condition(end_condition);

    builder
        .with_start_condition(start_condition_id)
        .with_end_condition(end_condition_id)
        .build()
}

/// Create a workflow with parallel split pattern (Pattern 2)
/// Creates: start → split → [task1, task2, ...] → join → end
///
/// # Arguments
/// * `name` - Workflow name
/// * `task_ids` - Vector of (task_id, task_name) tuples for parallel tasks
pub fn create_parallel_split_workflow(
    name: impl Into<String>,
    task_ids: Vec<(String, String)>,
) -> WorkflowSpec {
    if task_ids.is_empty() {
        panic!("Cannot create parallel workflow with no tasks");
    }

    let start_condition_id = "condition:start".to_string();
    let split_condition_id = "condition:split".to_string();
    let join_condition_id = "condition:join".to_string();
    let end_condition_id = "condition:end".to_string();

    let mut builder = WorkflowSpecBuilder::new(name);

    // Create start condition
    let start_condition = ConditionBuilder::new(start_condition_id.clone(), "Start")
        .add_outgoing_flow("split_task")
        .build();
    builder = builder.add_condition(start_condition);

    // Create split task
    let split_task = TaskBuilder::new("split_task", "Split")
        .with_type(TaskType::Atomic)
        .with_split_type(SplitType::And)
        .with_input_condition(start_condition_id.clone())
        .with_output_condition(split_condition_id.clone())
        .build();
    builder = builder.add_task(split_task);

    // Create split condition
    let mut split_condition = ConditionBuilder::new(split_condition_id.clone(), "Split");
    for (task_id, _) in &task_ids {
        split_condition = split_condition.add_outgoing_flow(task_id.clone());
    }
    builder = builder.add_condition(split_condition.build());

    // Create parallel tasks
    for (task_id, task_name) in &task_ids {
        let task = TaskBuilder::new(task_id.clone(), task_name.clone())
            .with_type(TaskType::Atomic)
            .with_input_condition(split_condition_id.clone())
            .with_output_condition(join_condition_id.clone())
            .build();
        builder = builder.add_task(task);
    }

    // Create join condition
    let mut join_condition = ConditionBuilder::new(join_condition_id.clone(), "Join");
    for (task_id, _) in &task_ids {
        join_condition = join_condition.add_incoming_flow(task_id.clone());
    }
    join_condition = join_condition.add_outgoing_flow("join_task");
    builder = builder.add_condition(join_condition.build());

    // Create join task
    let join_task = TaskBuilder::new("join_task", "Join")
        .with_type(TaskType::Atomic)
        .with_join_type(JoinType::And)
        .with_input_condition(join_condition_id.clone())
        .with_output_condition(end_condition_id.clone())
        .build();
    builder = builder.add_task(join_task);

    // Create end condition
    let end_condition = ConditionBuilder::new(end_condition_id.clone(), "End")
        .add_incoming_flow("join_task")
        .build();
    builder = builder.add_condition(end_condition);

    builder
        .with_start_condition(start_condition_id)
        .with_end_condition(end_condition_id)
        .build()
}

/// Create a workflow with XOR split pattern (Pattern 4)
/// Creates: start → split → [task1 OR task2 OR ...] → join → end
///
/// # Arguments
/// * `name` - Workflow name
/// * `task_ids` - Vector of (task_id, task_name) tuples for XOR branches
pub fn create_xor_split_workflow(
    name: impl Into<String>,
    task_ids: Vec<(String, String)>,
) -> WorkflowSpec {
    if task_ids.is_empty() {
        panic!("Cannot create XOR workflow with no tasks");
    }

    let start_condition_id = "condition:start".to_string();
    let split_condition_id = "condition:split".to_string();
    let join_condition_id = "condition:join".to_string();
    let end_condition_id = "condition:end".to_string();

    let mut builder = WorkflowSpecBuilder::new(name);

    // Create start condition
    let start_condition = ConditionBuilder::new(start_condition_id.clone(), "Start")
        .add_outgoing_flow("split_task")
        .build();
    builder = builder.add_condition(start_condition);

    // Create split task
    let split_task = TaskBuilder::new("split_task", "Split")
        .with_type(TaskType::Atomic)
        .with_split_type(SplitType::Xor)
        .with_input_condition(start_condition_id.clone())
        .with_output_condition(split_condition_id.clone())
        .build();
    builder = builder.add_task(split_task);

    // Create split condition
    let mut split_condition = ConditionBuilder::new(split_condition_id.clone(), "Split");
    for (task_id, _) in &task_ids {
        split_condition = split_condition.add_outgoing_flow(task_id.clone());
    }
    builder = builder.add_condition(split_condition.build());

    // Create XOR branch tasks
    for (task_id, task_name) in &task_ids {
        let task = TaskBuilder::new(task_id.clone(), task_name.clone())
            .with_type(TaskType::Atomic)
            .with_input_condition(split_condition_id.clone())
            .with_output_condition(join_condition_id.clone())
            .build();
        builder = builder.add_task(task);
    }

    // Create join condition
    let mut join_condition = ConditionBuilder::new(join_condition_id.clone(), "Join");
    for (task_id, _) in &task_ids {
        join_condition = join_condition.add_incoming_flow(task_id.clone());
    }
    join_condition = join_condition.add_outgoing_flow("join_task");
    builder = builder.add_condition(join_condition.build());

    // Create join task
    let join_task = TaskBuilder::new("join_task", "Join")
        .with_type(TaskType::Atomic)
        .with_join_type(JoinType::Xor)
        .with_input_condition(join_condition_id.clone())
        .with_output_condition(end_condition_id.clone())
        .build();
    builder = builder.add_task(join_task);

    // Create end condition
    let end_condition = ConditionBuilder::new(end_condition_id.clone(), "End")
        .add_incoming_flow("join_task")
        .build();
    builder = builder.add_condition(end_condition);

    builder
        .with_start_condition(start_condition_id)
        .with_end_condition(end_condition_id)
        .build()
}

/// Create a workflow with multiple instance pattern
/// Creates: start → mi_task (MultipleInstance) → end
///
/// # Arguments
/// * `name` - Workflow name
/// * `task_id` - ID of the MI task
/// * `task_name` - Name of the MI task
/// * `instance_count` - Optional instance count (None for unbounded)
pub fn create_mi_workflow(
    name: impl Into<String>,
    task_id: impl Into<String>,
    task_name: impl Into<String>,
    instance_count: Option<usize>,
) -> WorkflowSpec {
    let task_id_str = task_id.into();
    let start_condition_id = "condition:start".to_string();
    let end_condition_id = "condition:end".to_string();

    let task = TaskBuilder::new(task_id_str.clone(), task_name)
        .with_type(TaskType::MultipleInstance)
        .with_input_condition(start_condition_id.clone())
        .with_output_condition(end_condition_id.clone())
        .build();

    let start_condition = ConditionBuilder::new(start_condition_id.clone(), "Start")
        .add_outgoing_flow(task_id_str.clone())
        .build();

    let end_condition = ConditionBuilder::new(end_condition_id.clone(), "End")
        .add_incoming_flow(task_id_str.clone())
        .build();

    WorkflowSpecBuilder::new(name)
        .add_task(task)
        .add_condition(start_condition)
        .add_condition(end_condition)
        .with_start_condition(start_condition_id)
        .with_end_condition(end_condition_id)
        .build()
}

/// Create a test engine with a temporary state store
/// This is useful for tests that need isolated state stores
///
/// # Returns
/// A tuple of (WorkflowEngine, TempDir) where TempDir keeps the temp directory alive
///
/// # Example
/// ```rust
/// let (engine, _temp_dir) = create_test_engine_with_temp_store().unwrap();
/// // _temp_dir will be cleaned up when dropped
/// ```
#[cfg(feature = "tempfile")]
pub fn create_test_engine_with_temp_store() -> WorkflowResult<(WorkflowEngine, tempfile::TempDir)> {
    let temp_dir = tempfile::tempdir().map_err(|e| {
        WorkflowError::StatePersistence(format!("Failed to create temp directory: {}", e))
    })?;
    let state_store = StateStore::new(temp_dir.path()).map_err(|e| {
        WorkflowError::StatePersistence(format!("Failed to create state store: {}", e))
    })?;
    let engine = WorkflowEngine::new(state_store);
    Ok((engine, temp_dir))
}

/// Create a workflow with a loop pattern
/// Creates: start → task1 → [loop: task2 → task3] → task4 → end
///
/// # Arguments
/// * `name` - Workflow name
/// * `loop_tasks` - Vector of (task_id, task_name) tuples for loop body
pub fn create_loop_workflow(
    name: impl Into<String>,
    loop_tasks: Vec<(String, String)>,
) -> WorkflowSpec {
    if loop_tasks.is_empty() {
        panic!("Cannot create loop workflow with no loop tasks");
    }

    let start_condition_id = "condition:start".to_string();
    let loop_start_condition_id = "condition:loop_start".to_string();
    let loop_end_condition_id = "condition:loop_end".to_string();
    let end_condition_id = "condition:end".to_string();

    let mut builder = WorkflowSpecBuilder::new(name);

    // Create start condition
    let start_condition = ConditionBuilder::new(start_condition_id.clone(), "Start")
        .add_outgoing_flow("entry_task")
        .build();
    builder = builder.add_condition(start_condition);

    // Create entry task
    let entry_task = TaskBuilder::new("entry_task", "Entry")
        .with_type(TaskType::Atomic)
        .with_input_condition(start_condition_id.clone())
        .with_output_condition(loop_start_condition_id.clone())
        .build();
    builder = builder.add_task(entry_task);

    // Create loop start condition
    let mut loop_start_condition =
        ConditionBuilder::new(loop_start_condition_id.clone(), "Loop Start");
    loop_start_condition = loop_start_condition.add_outgoing_flow(loop_tasks[0].0.clone());
    builder = builder.add_condition(loop_start_condition.build());

    // Create loop tasks
    for (idx, (task_id, task_name)) in loop_tasks.iter().enumerate() {
        let mut task_builder =
            TaskBuilder::new(task_id.clone(), task_name.clone()).with_type(TaskType::Atomic);

        if idx == 0 {
            task_builder = task_builder.with_input_condition(loop_start_condition_id.clone());
        }

        if idx == loop_tasks.len() - 1 {
            // Last task in loop can go back to start or to exit
            task_builder = task_builder.with_output_condition(loop_end_condition_id.clone());
        }

        builder = builder.add_task(task_builder.build());
    }

    // Create loop end condition (can loop back or exit)
    let mut loop_end_condition = ConditionBuilder::new(loop_end_condition_id.clone(), "Loop End");
    loop_end_condition = loop_end_condition
        .add_incoming_flow(loop_tasks.last().unwrap().0.clone())
        .add_outgoing_flow("exit_task");
    builder = builder.add_condition(loop_end_condition.build());

    // Create exit task
    let exit_task = TaskBuilder::new("exit_task", "Exit")
        .with_type(TaskType::Atomic)
        .with_input_condition(loop_end_condition_id.clone())
        .with_output_condition(end_condition_id.clone())
        .build();
    builder = builder.add_task(exit_task);

    // Create end condition
    let end_condition = ConditionBuilder::new(end_condition_id.clone(), "End")
        .add_incoming_flow("exit_task")
        .build();
    builder = builder.add_condition(end_condition);

    builder
        .with_start_condition(start_condition_id)
        .with_end_condition(end_condition_id)
        .build()
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
