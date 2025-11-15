// Workflow Engine Template
// Ready-to-use workflow engine with state management
//
// Features:
// - Workflow registration and execution
// - State persistence
// - Step-by-step execution
// - Error handling and recovery
// - Telemetry integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Workflow Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowSpec {
    id: String,
    name: String,
    version: String,
    steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowStep {
    name: String,
    step_type: StepType,
    inputs: HashMap<String, String>,
    outputs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum StepType {
    Query,
    Transform,
    Condition,
    Parallel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowInstance {
    workflow_id: String,
    instance_id: String,
    state: WorkflowState,
    current_step: usize,
    variables: HashMap<String, String>,
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum WorkflowState {
    Pending,
    Running,
    Completed,
    Failed,
    Suspended,
}

// ============================================================================
// Workflow Engine
// ============================================================================

struct WorkflowEngine {
    /// Registered workflow specifications
    workflows: Arc<RwLock<HashMap<String, WorkflowSpec>>>,
    /// Running workflow instances
    instances: Arc<RwLock<HashMap<String, WorkflowInstance>>>,
}

impl WorkflowEngine {
    fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register workflow specification
    async fn register_workflow(&self, spec: WorkflowSpec) -> Result<(), String> {
        let mut workflows = self.workflows.write().await;

        if workflows.contains_key(&spec.id) {
            return Err(format!("Workflow {} already registered", spec.id));
        }

        workflows.insert(spec.id.clone(), spec);
        Ok(())
    }

    /// Create workflow instance
    async fn create_instance(&self, workflow_id: &str) -> Result<String, String> {
        let workflows = self.workflows.read().await;
        let _spec = workflows
            .get(workflow_id)
            .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

        // Generate instance ID
        let instance_id = format!("{}_{}", workflow_id, uuid::Uuid::new_v4());

        // Create instance
        let instance = WorkflowInstance {
            workflow_id: workflow_id.to_string(),
            instance_id: instance_id.clone(),
            state: WorkflowState::Pending,
            current_step: 0,
            variables: HashMap::new(),
            error: None,
        };

        let mut instances = self.instances.write().await;
        instances.insert(instance_id.clone(), instance);

        Ok(instance_id)
    }

    /// Execute workflow step
    async fn execute_step(&self, instance_id: &str) -> Result<bool, String> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(instance_id)
            .ok_or_else(|| format!("Instance {} not found", instance_id))?;

        // Check state
        if instance.state != WorkflowState::Pending
            && instance.state != WorkflowState::Running
        {
            return Err(format!("Cannot execute step in state: {:?}", instance.state));
        }

        // Transition to Running
        if instance.state == WorkflowState::Pending {
            instance.state = WorkflowState::Running;
        }

        // Get workflow spec
        let workflows = self.workflows.read().await;
        let spec = workflows
            .get(&instance.workflow_id)
            .ok_or_else(|| format!("Workflow {} not found", instance.workflow_id))?;

        // Check if workflow completed
        if instance.current_step >= spec.steps.len() {
            instance.state = WorkflowState::Completed;
            return Ok(true); // Workflow completed
        }

        // Get current step
        let step = &spec.steps[instance.current_step];

        // Execute step (simplified)
        match self.execute_step_logic(step).await {
            Ok(outputs) => {
                // Store outputs in variables
                for (key, value) in outputs {
                    instance.variables.insert(key, value);
                }

                // Advance to next step
                instance.current_step += 1;

                // Check if workflow completed
                if instance.current_step >= spec.steps.len() {
                    instance.state = WorkflowState::Completed;
                    Ok(true) // Workflow completed
                } else {
                    Ok(false) // More steps to execute
                }
            }
            Err(e) => {
                instance.state = WorkflowState::Failed;
                instance.error = Some(e.clone());
                Err(e)
            }
        }
    }

    /// Execute workflow to completion
    async fn execute_workflow(&self, instance_id: &str) -> Result<(), String> {
        loop {
            let completed = self.execute_step(instance_id).await?;
            if completed {
                break;
            }

            // Small delay between steps (in production, this could be async await)
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }

    /// Get instance status
    async fn get_status(&self, instance_id: &str) -> Result<WorkflowInstance, String> {
        let instances = self.instances.read().await;
        instances
            .get(instance_id)
            .cloned()
            .ok_or_else(|| format!("Instance {} not found", instance_id))
    }

    // Private: Execute step logic (simplified)
    async fn execute_step_logic(
        &self,
        step: &WorkflowStep,
    ) -> Result<HashMap<String, String>, String> {
        // Simulate step execution
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        println!("  Executing step: {} ({}:?})", step.name, step.step_type);

        // Return outputs
        let mut outputs = HashMap::new();
        outputs.insert("result".to_string(), "success".to_string());
        Ok(outputs)
    }
}

// ============================================================================
// Main: Workflow Engine Example
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Workflow Engine Template ===\n");

    // Create engine
    let engine = WorkflowEngine::new();

    // Example 1: Register workflow
    println!("--- Example 1: Register Workflow ---");
    let spec = WorkflowSpec {
        id: "user-registration".to_string(),
        name: "User Registration Workflow".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![
            WorkflowStep {
                name: "validate-email".to_string(),
                step_type: StepType::Query,
                inputs: HashMap::from([("email".to_string(), "${input.email}".to_string())]),
                outputs: HashMap::new(),
            },
            WorkflowStep {
                name: "create-user".to_string(),
                step_type: StepType::Transform,
                inputs: HashMap::new(),
                outputs: HashMap::from([("user_id".to_string(), "${generated.id}".to_string())]),
            },
            WorkflowStep {
                name: "send-welcome-email".to_string(),
                step_type: StepType::Query,
                inputs: HashMap::from([("user_id".to_string(), "${user_id}".to_string())]),
                outputs: HashMap::new(),
            },
        ],
    };

    engine.register_workflow(spec).await?;
    println!("✅ Workflow registered: user-registration\n");

    // Example 2: Create instance
    println!("--- Example 2: Create Workflow Instance ---");
    let instance_id = engine.create_instance("user-registration").await?;
    println!("✅ Instance created: {}\n", instance_id);

    // Example 3: Execute workflow
    println!("--- Example 3: Execute Workflow ---");
    engine.execute_workflow(&instance_id).await?;
    println!("✅ Workflow executed\n");

    // Example 4: Get status
    println!("--- Example 4: Get Workflow Status ---");
    let status = engine.get_status(&instance_id).await?;
    println!("Status: {:?}", status.state);
    println!("Current step: {}/{}", status.current_step, 3);
    println!("Variables: {:?}", status.variables);
    println!();

    println!("=== Production Enhancements ===");
    println!("- [ ] Persistence (save workflow state to database)");
    println!("- [ ] Telemetry (OTEL spans for each step)");
    println!("- [ ] Retry logic (transient error recovery)");
    println!("- [ ] Compensation (rollback on failure)");
    println!("- [ ] Parallel execution (independent steps)");
    println!("- [ ] Conditional branching (if/else logic)");
    println!("- [ ] Loop support (for each, while)");
    println!("- [ ] Human tasks (wait for approval)");
    println!("- [ ] Event sourcing (audit trail)");
    println!("- [ ] Workflow versioning (multiple versions)");

    Ok(())
}

// ============================================================================
// Integration with KNHK Workflow Engine
// ============================================================================

// For production, use knhk-workflow-engine crate:
//
// use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};
//
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Initialize state store
//     let state_store = StateStore::new("./workflow_db")?;
//
//     // Create engine
//     let engine = WorkflowEngine::new(state_store);
//
//     // Parse workflow from file
//     let mut parser = WorkflowParser::new()?;
//     let spec = parser.parse_file("workflow.ttl")?;
//
//     // Register workflow
//     engine.register_workflow(spec).await?;
//
//     // Execute workflow
//     let instance_id = engine.create_instance("workflow-id").await?;
//     engine.execute_workflow(&instance_id).await?;
//
//     Ok(())
// }

// Dependencies (add to Cargo.toml):
// [dependencies]
// tokio = { version = "1", features = ["full"] }
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
// uuid = { version = "1", features = ["v4"] }
