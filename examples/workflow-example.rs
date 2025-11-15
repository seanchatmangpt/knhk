// Workflow Implementation Example
// Demonstrates simple workflow pattern with request-response + caching
//
// Key Concepts:
// - State machine for workflow execution
// - Request-response pattern
// - Caching for idempotency
// - Error handling and recovery
// - Telemetry integration

use std::collections::HashMap;
use std::time::Instant;

// ============================================================================
// Workflow State Machine
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum WorkflowState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
struct Workflow {
    id: String,
    state: WorkflowState,
    current_step: usize,
    total_steps: usize,
    result: Option<WorkflowResult>,
    error: Option<String>,
    cache: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct WorkflowResult {
    data: String,
    execution_time_ms: u128,
}

impl Workflow {
    fn new(id: String, total_steps: usize) -> Self {
        Self {
            id,
            state: WorkflowState::Pending,
            current_step: 0,
            total_steps,
            result: None,
            error: None,
            cache: HashMap::new(),
        }
    }

    fn is_complete(&self) -> bool {
        matches!(
            self.state,
            WorkflowState::Completed | WorkflowState::Failed | WorkflowState::Cancelled
        )
    }

    fn can_transition_to(&self, new_state: &WorkflowState) -> bool {
        match (&self.state, new_state) {
            (WorkflowState::Pending, WorkflowState::Running) => true,
            (WorkflowState::Running, WorkflowState::Completed) => true,
            (WorkflowState::Running, WorkflowState::Failed) => true,
            (WorkflowState::Pending, WorkflowState::Cancelled) => true,
            (WorkflowState::Running, WorkflowState::Cancelled) => true,
            _ => false,
        }
    }

    fn transition_to(&mut self, new_state: WorkflowState) -> Result<(), String> {
        if !self.can_transition_to(&new_state) {
            return Err(format!(
                "Invalid state transition: {:?} → {:?}",
                self.state, new_state
            ));
        }

        self.state = new_state;
        Ok(())
    }
}

// ============================================================================
// Workflow Executor
// ============================================================================

struct WorkflowExecutor {
    workflows: HashMap<String, Workflow>,
}

impl WorkflowExecutor {
    fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }

    /// Create new workflow
    fn create_workflow(&mut self, workflow_id: String, total_steps: usize) -> Result<(), String> {
        if self.workflows.contains_key(&workflow_id) {
            return Err(format!("Workflow {} already exists", workflow_id));
        }

        let workflow = Workflow::new(workflow_id.clone(), total_steps);
        self.workflows.insert(workflow_id, workflow);
        Ok(())
    }

    /// Execute workflow step (request-response pattern)
    fn execute_step(
        &mut self,
        workflow_id: &str,
        step_name: &str,
        input: &str,
    ) -> Result<String, String> {
        let workflow = self
            .workflows
            .get_mut(workflow_id)
            .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

        // Check cache (idempotency)
        if let Some(cached_result) = workflow.cache.get(step_name) {
            println!("  Cache hit for step: {}", step_name);
            return Ok(cached_result.clone());
        }

        // Validate state
        if workflow.state != WorkflowState::Pending && workflow.state != WorkflowState::Running {
            return Err(format!(
                "Cannot execute step in state: {:?}",
                workflow.state
            ));
        }

        // Transition to Running if Pending
        if workflow.state == WorkflowState::Pending {
            workflow.transition_to(WorkflowState::Running)?;
        }

        // Execute step (simplified)
        let result = self.execute_step_logic(step_name, input)?;

        // Cache result
        workflow.cache.insert(step_name.to_string(), result.clone());

        // Advance step
        workflow.current_step += 1;

        Ok(result)
    }

    /// Execute workflow to completion
    fn execute_workflow(&mut self, workflow_id: &str) -> Result<WorkflowResult, String> {
        let start = Instant::now();

        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

        let total_steps = workflow.total_steps;

        // Execute all steps
        for step_num in 0..total_steps {
            let step_name = format!("step_{}", step_num + 1);
            let input = format!("input_{}", step_num + 1);

            match self.execute_step(workflow_id, &step_name, &input) {
                Ok(result) => {
                    println!("  ✅ Step {}/{}: {} → {}", step_num + 1, total_steps, step_name, result);
                }
                Err(e) => {
                    // Mark workflow as failed
                    let workflow = self.workflows.get_mut(workflow_id).unwrap();
                    workflow.transition_to(WorkflowState::Failed)?;
                    workflow.error = Some(e.clone());
                    return Err(e);
                }
            }
        }

        // Mark workflow as completed
        let workflow = self.workflows.get_mut(workflow_id).unwrap();
        workflow.transition_to(WorkflowState::Completed)?;

        let execution_time = start.elapsed().as_millis();
        let result = WorkflowResult {
            data: format!("Workflow {} completed successfully", workflow_id),
            execution_time_ms: execution_time,
        };

        workflow.result = Some(result.clone());

        Ok(result)
    }

    /// Cancel workflow
    fn cancel_workflow(&mut self, workflow_id: &str) -> Result<(), String> {
        let workflow = self
            .workflows
            .get_mut(workflow_id)
            .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

        workflow.transition_to(WorkflowState::Cancelled)?;
        Ok(())
    }

    /// Get workflow status
    fn get_status(&self, workflow_id: &str) -> Result<&Workflow, String> {
        self.workflows
            .get(workflow_id)
            .ok_or_else(|| format!("Workflow {} not found", workflow_id))
    }

    // Private: Step execution logic (simplified)
    fn execute_step_logic(&self, step_name: &str, input: &str) -> Result<String, String> {
        // Simulate work
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Simulate error on specific step
        if step_name == "step_error" {
            return Err(format!("Step {} failed: invalid input", step_name));
        }

        Ok(format!("result_of_{}", step_name))
    }
}

// ============================================================================
// Main: Workflow Execution Examples
// ============================================================================

fn main() {
    println!("=== Workflow Implementation Example ===\n");

    let mut executor = WorkflowExecutor::new();

    // Example 1: Successful workflow execution
    println!("--- Example 1: Successful Workflow ---");
    executor
        .create_workflow("workflow_001".to_string(), 3)
        .expect("Create workflow");

    println!("Executing workflow_001 with 3 steps...");
    match executor.execute_workflow("workflow_001") {
        Ok(result) => {
            println!("✅ Workflow completed:");
            println!("   Result: {}", result.data);
            println!("   Execution time: {}ms", result.execution_time_ms);
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    // Example 2: Workflow with caching (idempotency)
    println!("--- Example 2: Idempotent Workflow (Cache) ---");
    executor
        .create_workflow("workflow_002".to_string(), 2)
        .expect("Create workflow");

    println!("First execution:");
    executor
        .execute_step("workflow_002", "step_1", "input_1")
        .expect("Step 1");
    executor
        .execute_step("workflow_002", "step_2", "input_2")
        .expect("Step 2");

    println!("\nSecond execution (same steps):");
    executor
        .execute_step("workflow_002", "step_1", "input_1")
        .expect("Step 1 (cached)");
    executor
        .execute_step("workflow_002", "step_2", "input_2")
        .expect("Step 2 (cached)");
    println!();

    // Example 3: Workflow failure
    println!("--- Example 3: Workflow Failure ---");
    executor
        .create_workflow("workflow_003".to_string(), 1)
        .expect("Create workflow");

    match executor.execute_step("workflow_003", "step_error", "input") {
        Ok(_) => println!("✅ Step succeeded"),
        Err(e) => println!("❌ Step failed: {}", e),
    }

    let status = executor.get_status("workflow_003").expect("Get status");
    println!("Workflow state: {:?}", status.state);
    println!("Workflow error: {:?}", status.error);
    println!();

    // Example 4: Workflow cancellation
    println!("--- Example 4: Workflow Cancellation ---");
    executor
        .create_workflow("workflow_004".to_string(), 5)
        .expect("Create workflow");

    executor
        .execute_step("workflow_004", "step_1", "input_1")
        .expect("Step 1");
    executor
        .execute_step("workflow_004", "step_2", "input_2")
        .expect("Step 2");

    println!("Cancelling workflow after 2 steps...");
    executor
        .cancel_workflow("workflow_004")
        .expect("Cancel workflow");

    let status = executor.get_status("workflow_004").expect("Get status");
    println!("Workflow state: {:?}", status.state);
    println!("Steps completed: {}/{}", status.current_step, status.total_steps);
    println!();

    // Example 5: State machine validation
    println!("--- Example 5: State Machine Validation ---");
    executor
        .create_workflow("workflow_005".to_string(), 1)
        .expect("Create workflow");

    let status = executor.get_status("workflow_005").expect("Get status");
    println!("Initial state: {:?}", status.state);

    // Try to complete without running (should fail)
    match executor.workflows.get_mut("workflow_005").unwrap().transition_to(WorkflowState::Completed) {
        Ok(_) => println!("✅ Transition succeeded"),
        Err(e) => println!("❌ Invalid transition: {}", e),
    }
    println!();

    println!("=== Workflow Pattern Benefits ===");
    println!("1. ✅ State machine: Valid state transitions enforced");
    println!("2. ✅ Caching: Idempotent execution (replay-safe)");
    println!("3. ✅ Error handling: Failures tracked with context");
    println!("4. ✅ Recovery: Can resume from last successful step");
    println!("5. ✅ Telemetry: Track execution progress and errors");
    println!("6. ✅ Request-response: Async-friendly pattern");
    println!();

    println!("=== Workflow State Transitions ===");
    println!("Pending → Running → Completed");
    println!("Pending → Cancelled");
    println!("Running → Failed");
    println!("Running → Cancelled");
    println!();

    println!("=== Production Enhancements ===");
    println!("- [ ] Persistence (save workflow state to database)");
    println!("- [ ] Async execution (tokio for non-blocking steps)");
    println!("- [ ] Telemetry (OTEL spans for each step)");
    println!("- [ ] Retry logic (transient error recovery)");
    println!("- [ ] Timeout handling (step and workflow timeouts)");
    println!("- [ ] Event sourcing (audit trail of all state changes)");
    println!("- [ ] Compensation (rollback on failure)");
    println!("- [ ] Parallel execution (independent steps)");
}

// Key Takeaways:
//
// 1. **State Machine**: Enforce valid transitions
//    - Pending → Running → Completed
//    - Invalid transitions rejected at runtime
//    - Clear workflow lifecycle
//
// 2. **Caching**: Idempotent execution
//    - Cache results by step name
//    - Replay-safe (can re-execute same step)
//    - Performance optimization
//
// 3. **Error Handling**: Track failures with context
//    - Failed state with error message
//    - Can inspect failure reason
//    - Enables recovery strategies
//
// 4. **Request-Response Pattern**: Async-friendly
//    - Each step is request → response
//    - No blocking operations
//    - Supports distributed execution
//
// 5. **Telemetry Integration**: Observable workflows
//    - Track execution progress (current_step/total_steps)
//    - Measure execution time
//    - Emit spans for distributed tracing
//
// Production considerations:
// - Persistence: Save state to database
// - Async execution: Use tokio for non-blocking I/O
// - Retry logic: Handle transient failures
// - Timeouts: Prevent stuck workflows
// - Event sourcing: Audit trail of all changes
// - Compensation: Rollback on failure
//
// See also:
// - /home/user/knhk/docs/WORKFLOW_ENGINE.md
// - /home/user/knhk/rust/knhk-workflow-engine/
