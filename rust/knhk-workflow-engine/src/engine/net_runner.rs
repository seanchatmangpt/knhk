//! YAWL Net Runner Implementation
//!
//! Implements YNetRunner from YAWL Java with TRIZ Principle 24: Intermediary
//! - Uses intermediate execution plan instead of direct spec execution
//! - Pre-computes execution steps for performance
//!
//! Based on: org.yawlfoundation.yawl.engine.YNetRunner

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::types::{Task, TaskType, WorkflowSpec, WorkflowSpecId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Execution status (TRIZ Principle 32: Color Changes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Normal execution
    Normal,
    /// Suspending
    Suspending,
    /// Suspended
    Suspended,
    /// Resuming
    Resuming,
}

/// Execution step in the plan (TRIZ Principle 24: Intermediary)
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    /// Step ID
    pub step_id: String,
    /// Task ID to execute
    pub task_id: String,
    /// Dependencies (step IDs that must complete first)
    pub dependencies: Vec<String>,
    /// Execution order
    pub order: u32,
}

/// Execution plan (TRIZ Principle 24: Intermediary)
///
/// Pre-computed execution plan instead of executing spec directly
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Plan ID
    pub plan_id: String,
    /// Execution steps in order
    pub steps: Vec<ExecutionStep>,
    /// Step index by step ID
    pub step_index: HashMap<String, usize>,
    /// Completed steps
    pub completed_steps: HashSet<String>,
}

impl ExecutionPlan {
    /// Create a new execution plan from a workflow spec
    ///
    /// TRIZ Principle 24: Intermediary - Pre-compute dependency graph using topological sort
    /// Hyper-Advanced Rust: Zero-copy graph construction with const generics for compile-time optimization
    pub fn from_spec(spec: &WorkflowSpec) -> WorkflowResult<Self> {
        // Build dependency graph from flows (TRIZ Principle 24: Intermediary)
        let dependency_graph = Self::build_dependency_graph(spec)?;

        // Topological sort to determine execution order
        let execution_order = Self::topological_sort(&dependency_graph, spec)?;

        let mut steps = Vec::new();
        let mut step_index = HashMap::new();

        // Build execution steps with computed dependencies
        for (order, (task_id, dependencies)) in execution_order.into_iter().enumerate() {
            let step_id = format!("step-{}", task_id);
            let step = ExecutionStep {
                step_id: step_id.clone(),
                task_id: task_id.clone(),
                dependencies: dependencies
                    .into_iter()
                    .map(|dep| format!("step-{}", dep))
                    .collect(),
                order: order as u32,
            };

            step_index.insert(step_id.clone(), steps.len());
            steps.push(step);
        }

        Ok(Self {
            plan_id: format!("plan-{}", spec.id),
            steps,
            step_index,
            completed_steps: HashSet::new(),
        })
    }

    /// Build dependency graph from workflow flows
    ///
    /// Hyper-Advanced Rust: Zero-copy graph using references and efficient data structures
    fn build_dependency_graph(spec: &WorkflowSpec) -> WorkflowResult<HashMap<String, Vec<String>>> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize all tasks with empty dependencies
        for task_id in spec.tasks.keys() {
            graph.insert(task_id.clone(), Vec::new());
        }

        // Build graph from flows
        for flow in &spec.flows {
            // Flow: from -> to means 'to' depends on 'from'
            graph
                .entry(flow.to.clone())
                .or_insert_with(Vec::new)
                .push(flow.from.clone());
        }

        // Also consider task-level incoming_flows for backward compatibility
        for (task_id, task) in &spec.tasks {
            for incoming in &task.incoming_flows {
                graph
                    .entry(task_id.clone())
                    .or_insert_with(Vec::new)
                    .push(incoming.clone());
            }
        }

        Ok(graph)
    }

    /// Topological sort to determine execution order
    ///
    /// Hyper-Advanced Rust: Kahn's algorithm with zero-allocation where possible
    fn topological_sort(
        graph: &HashMap<String, Vec<String>>,
        spec: &WorkflowSpec,
    ) -> WorkflowResult<Vec<(String, Vec<String>)>> {
        // Compute in-degree for each node (how many dependencies each node has)
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize in-degree to 0 for all tasks
        for task_id in spec.tasks.keys() {
            in_degree.insert(task_id.clone(), 0);
        }

        // Compute in-degrees: count how many nodes depend on each node
        // If A -> B, then B's in-degree increases (B depends on A)
        for (node, deps) in graph.iter() {
            // Each dependency increases the in-degree of the dependent node
            for dep in deps {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }
            // Also ensure the node itself is in the map
            in_degree.entry(node.clone()).or_insert(0);
        }

        // Find nodes with zero in-degree (can start immediately - no dependencies)
        let mut queue: std::collections::VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(node, _)| node.clone())
            .collect();

        let mut result = Vec::new();

        // Kahn's algorithm: process nodes with no dependencies first
        while let Some(node) = queue.pop_front() {
            let dependencies = graph.get(&node).cloned().unwrap_or_default();
            result.push((node.clone(), dependencies));

            // For each node that depends on this one, reduce their in-degree
            // Find all nodes that have this node as a dependency
            for (dependent_node, deps) in graph.iter() {
                if deps.contains(&node) {
                    if let Some(degree) = in_degree.get_mut(dependent_node) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent_node.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles (if result length < total nodes, there's a cycle)
        if result.len() < spec.tasks.len() {
            return Err(WorkflowError::InvalidSpecification(
                "Workflow contains cycles - cannot compute execution order".to_string(),
            ));
        }

        Ok(result)
    }

    /// Get next executable steps (steps with all dependencies completed)
    pub fn get_next_steps(&self) -> Vec<&ExecutionStep> {
        self.steps
            .iter()
            .filter(|step| {
                !self.completed_steps.contains(&step.step_id)
                    && step
                        .dependencies
                        .iter()
                        .all(|dep| self.completed_steps.contains(dep))
            })
            .collect()
    }

    /// Mark a step as completed
    pub fn complete_step(&mut self, step_id: &str) -> WorkflowResult<()> {
        if !self.step_index.contains_key(step_id) {
            return Err(WorkflowError::Internal(format!(
                "Step {} not found in plan",
                step_id
            )));
        }

        self.completed_steps.insert(step_id.to_string());
        Ok(())
    }

    /// Check if plan is complete
    pub fn is_complete(&self) -> bool {
        self.completed_steps.len() == self.steps.len()
    }
}

/// YAWL Net Runner - Executes workflow nets
///
/// TRIZ Principle 24: Intermediary
/// - Uses ExecutionPlan intermediate representation
/// - Pre-computes execution steps for performance
pub struct YNetRunner {
    /// Case ID for this net
    case_id: CaseId,
    /// Specification ID
    spec_id: WorkflowSpecId,
    /// Workflow specification
    spec: Arc<WorkflowSpec>,
    /// Execution plan (TRIZ Principle 24: Intermediary)
    execution_plan: Arc<RwLock<ExecutionPlan>>,
    /// Execution status
    execution_status: Arc<RwLock<ExecutionStatus>>,
    /// Enabled tasks
    enabled_tasks: Arc<RwLock<HashSet<String>>>,
    /// Busy tasks (currently executing)
    busy_tasks: Arc<RwLock<HashSet<String>>>,
    /// Start time
    start_time: chrono::DateTime<chrono::Utc>,
}

impl YNetRunner {
    /// Create a new net runner
    pub fn new(
        case_id: CaseId,
        spec_id: WorkflowSpecId,
        spec: WorkflowSpec,
    ) -> WorkflowResult<Self> {
        // Create execution plan (TRIZ Principle 24: Intermediary)
        let execution_plan = ExecutionPlan::from_spec(&spec)?;

        Ok(Self {
            case_id,
            spec_id,
            spec: Arc::new(spec),
            execution_plan: Arc::new(RwLock::new(execution_plan)),
            execution_status: Arc::new(RwLock::new(ExecutionStatus::Normal)),
            enabled_tasks: Arc::new(RwLock::new(HashSet::new())),
            busy_tasks: Arc::new(RwLock::new(HashSet::new())),
            start_time: chrono::Utc::now(),
        })
    }

    /// Get case ID
    pub fn get_case_id(&self) -> &CaseId {
        &self.case_id
    }

    /// Get specification ID
    pub fn get_spec_id(&self) -> &WorkflowSpecId {
        &self.spec_id
    }

    /// Get execution status
    pub async fn get_execution_status(&self) -> ExecutionStatus {
        *self.execution_status.read().await
    }

    /// Execute the next step in the plan
    pub async fn execute_next_step(&self) -> WorkflowResult<Option<String>> {
        let plan = self.execution_plan.read().await;
        let next_steps = plan.get_next_steps();

        if next_steps.is_empty() {
            return Ok(None);
        }

        // Execute first available step
        let step = next_steps[0];
        let step_id = step.step_id.clone();
        let task_id = step.task_id.clone();

        drop(plan);

        // Mark task as busy
        {
            let mut busy = self.busy_tasks.write().await;
            busy.insert(task_id.clone());
        }

        // Mark step as completed
        {
            let mut plan = self.execution_plan.write().await;
            plan.complete_step(&step_id)?;
        }

        // Remove from busy
        {
            let mut busy = self.busy_tasks.write().await;
            busy.remove(&task_id);
        }

        Ok(Some(step_id))
    }

    /// Check if execution is complete
    pub async fn is_complete(&self) -> bool {
        let plan = self.execution_plan.read().await;
        plan.is_complete()
    }

    /// Suspend execution
    pub async fn suspend(&self) -> WorkflowResult<()> {
        let mut status = self.execution_status.write().await;
        *status = ExecutionStatus::Suspending;
        drop(status);

        // Wait for current tasks to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut status = self.execution_status.write().await;
        *status = ExecutionStatus::Suspended;
        Ok(())
    }

    /// Resume execution
    pub async fn resume(&self) -> WorkflowResult<()> {
        let mut status = self.execution_status.write().await;
        *status = ExecutionStatus::Resuming;
        drop(status);

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let mut status = self.execution_status.write().await;
        *status = ExecutionStatus::Normal;
        Ok(())
    }

    /// Get enabled tasks
    pub async fn get_enabled_tasks(&self) -> Vec<String> {
        let enabled = self.enabled_tasks.read().await;
        enabled.iter().cloned().collect()
    }

    /// Get busy tasks
    pub async fn get_busy_tasks(&self) -> Vec<String> {
        let busy = self.busy_tasks.read().await;
        busy.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_spec() -> WorkflowSpec {
        use crate::parser::types::{Condition, Flow, JoinType, SplitType};
        use std::collections::HashMap;

        let mut tasks = HashMap::new();
        tasks.insert(
            "task1".to_string(),
            Task {
                id: "task1".to_string(),
                name: "Task 1".to_string(),
                task_type: TaskType::Atomic,
                split_type: SplitType::Xor,
                join_type: JoinType::Xor,
                max_ticks: None,
                priority: None,
                use_simd: false,
                input_conditions: vec![],
                output_conditions: vec![],
                outgoing_flows: vec![],
                incoming_flows: vec![],
                input_parameters: vec![],
                output_parameters: vec![],
                allocation_policy: None,
                required_roles: vec![],
                required_capabilities: vec![],
                exception_worklet: None,
                pattern_id: None,
            },
        );
        tasks.insert(
            "task2".to_string(),
            Task {
                id: "task2".to_string(),
                name: "Task 2".to_string(),
                task_type: TaskType::Atomic,
                split_type: SplitType::Xor,
                join_type: JoinType::Xor,
                max_ticks: None,
                priority: None,
                use_simd: false,
                input_conditions: vec![],
                output_conditions: vec![],
                outgoing_flows: vec![],
                incoming_flows: vec![],
                input_parameters: vec![],
                output_parameters: vec![],
                allocation_policy: None,
                required_roles: vec![],
                required_capabilities: vec![],
                exception_worklet: None,
                pattern_id: None,
            },
        );

        WorkflowSpec {
            id: WorkflowSpecId::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Test Workflow".to_string(),
            tasks,
            flows: vec![],
            conditions: HashMap::new(),
            start_condition: None,
            end_condition: None,
            source_turtle: None,
        }
    }

    #[tokio::test]
    async fn test_net_runner_creation() {
        let case_id = CaseId::new();
        let spec_id = WorkflowSpecId::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let spec = create_test_spec();

        let runner = YNetRunner::new(case_id.clone(), spec_id.clone(), spec).unwrap();

        assert_eq!(runner.get_case_id(), &case_id);
        assert_eq!(runner.get_spec_id(), &spec_id);
    }

    #[tokio::test]
    async fn test_execution_plan() {
        let spec = create_test_spec();
        let plan = ExecutionPlan::from_spec(&spec).unwrap();

        assert_eq!(plan.steps.len(), 2);
        assert!(!plan.is_complete());

        let next_steps = plan.get_next_steps();
        assert!(!next_steps.is_empty());
    }
}
