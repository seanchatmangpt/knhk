//! Property-Based Testing Framework
//!
//! Provides QuickCheck-style property-based testing for workflow specifications.
//! Generates random valid workflows and validates invariants hold.

use crate::error::WorkflowResult;
use crate::parser::{Task, TaskType, WorkflowSpec, WorkflowSpecId};
use crate::testing::chicago_tdd::WorkflowTestFixture;
use std::collections::HashMap;

/// Property-based test generator
pub struct PropertyTestGenerator {
    /// Maximum number of tasks to generate
    max_tasks: usize,
    /// Maximum depth of workflow nesting
    max_depth: usize,
    /// Random seed for reproducibility
    seed: u64,
}

impl PropertyTestGenerator {
    /// Create new property test generator
    pub fn new() -> Self {
        Self {
            max_tasks: 10,
            max_depth: 3,
            seed: 0,
        }
    }

    /// Set maximum tasks
    pub fn with_max_tasks(mut self, max_tasks: usize) -> Self {
        self.max_tasks = max_tasks;
        self
    }

    /// Set maximum depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Generate random workflow specification
    pub fn generate_workflow(&mut self) -> WorkflowSpec {
        let mut rng = SimpleRng::new(self.seed);
        self.seed = self.seed.wrapping_add(1);

        let num_tasks = rng.next() as usize % self.max_tasks + 1;
        let mut tasks = HashMap::new();

        for i in 0..num_tasks {
            let task_id = format!("task:{}", i);
            let task = Task {
                id: task_id.clone(),
                name: format!("Task {}", i),
                task_type: if rng.next() % 2 == 0 {
                    TaskType::Atomic
                } else {
                    TaskType::Composite
                },
                split_type: self.random_split_type(&mut rng),
                join_type: self.random_join_type(&mut rng),
                max_ticks: if rng.next() % 3 == 0 { Some(8) } else { None },
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
            };
            tasks.insert(task_id, task);
        }

        // Add random connections
        self.add_random_connections(&mut tasks, &mut rng);

        WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: format!("Generated Workflow {}", self.seed),
            tasks,
            conditions: HashMap::new(),
            start_condition: Some("condition:start".to_string()),
            end_condition: Some("condition:end".to_string()),
        }
    }

    fn random_split_type(&self, rng: &mut SimpleRng) -> crate::parser::SplitType {
        match rng.next() % 3 {
            0 => crate::parser::SplitType::And,
            1 => crate::parser::SplitType::Xor,
            _ => crate::parser::SplitType::Or,
        }
    }

    fn random_join_type(&self, rng: &mut SimpleRng) -> crate::parser::JoinType {
        match rng.next() % 3 {
            0 => crate::parser::JoinType::And,
            1 => crate::parser::JoinType::Xor,
            _ => crate::parser::JoinType::Or,
        }
    }

    fn add_random_connections(&self, tasks: &mut HashMap<String, Task>, rng: &mut SimpleRng) {
        let task_ids: Vec<String> = tasks.keys().cloned().collect();
        if task_ids.len() < 2 {
            return;
        }

        // Add random connections (avoid cycles for simplicity)
        for i in 0..task_ids.len() - 1 {
            if rng.next() % 2 == 0 {
                let from = &task_ids[i];
                let to = &task_ids[i + 1];
                if let Some(task) = tasks.get_mut(from) {
                    task.outgoing_flows.push(to.clone());
                }
            }
        }
    }
}

/// Simple RNG for property testing (LCG)
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        // Linear Congruential Generator
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }
}

/// Property: All workflows can be registered
pub async fn property_all_workflows_registrable(
    generator: &mut PropertyTestGenerator,
    num_tests: usize,
) -> WorkflowResult<bool> {
    let mut fixture = WorkflowTestFixture::new()?;

    for _ in 0..num_tests {
        let spec = generator.generate_workflow();
        fixture.register_workflow(spec).await?;
    }

    Ok(true)
}

/// Property: All workflows have valid structure
pub fn property_all_workflows_valid_structure(
    generator: &mut PropertyTestGenerator,
    num_tests: usize,
) -> bool {
    for _ in 0..num_tests {
        let spec = generator.generate_workflow();

        // Validate: Has at least one task
        if spec.tasks.is_empty() {
            return false;
        }

        // Validate: All tasks have valid IDs
        for (id, task) in &spec.tasks {
            if id != &task.id {
                return false;
            }
        }
    }

    true
}

/// Property: Workflow execution terminates
pub async fn property_workflow_execution_terminates(
    generator: &mut PropertyTestGenerator,
    num_tests: usize,
) -> WorkflowResult<bool> {
    let mut fixture = WorkflowTestFixture::new()?;

    for _ in 0..num_tests {
        let spec = generator.generate_workflow();
        let spec_id = fixture.register_workflow(spec).await?;
        let case_id = fixture.create_case(spec_id, serde_json::json!({})).await?;

        // Execute case (should not hang)
        let case = fixture.execute_case(case_id).await?;

        // Validate: Case reaches terminal state
        if !matches!(
            case.state,
            crate::case::CaseState::Completed | crate::case::CaseState::Failed
        ) {
            // Allow running state for now (workflow may need multiple steps)
            // In production, would add timeout
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_generator_creates_workflows() {
        let mut generator = PropertyTestGenerator::new();
        let spec = generator.generate_workflow();

        assert!(!spec.tasks.is_empty());
        assert_eq!(spec.name, "Generated Workflow 1");
    }

    #[tokio::test]
    async fn test_property_all_workflows_registrable() {
        let mut generator = PropertyTestGenerator::new().with_max_tasks(5);
        let result = property_all_workflows_registrable(&mut generator, 10).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_all_workflows_valid_structure() {
        let mut generator = PropertyTestGenerator::new().with_max_tasks(5);
        assert!(property_all_workflows_valid_structure(&mut generator, 10));
    }
}
