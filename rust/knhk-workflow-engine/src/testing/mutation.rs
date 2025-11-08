//! Mutation Testing Framework
//!
//! Validates test quality by introducing mutations and checking if tests catch them.
//! High-quality tests should fail when code is mutated.

use crate::error::WorkflowResult;
use crate::parser::{Task, WorkflowSpec};
use crate::testing::chicago_tdd::WorkflowTestFixture;
use std::collections::HashMap;

/// Mutation operator for workflow specifications
#[derive(Debug, Clone)]
pub enum MutationOperator {
    /// Remove a task
    RemoveTask(String),
    /// Add a task
    AddTask(Task),
    /// Change task type
    ChangeTaskType(String, crate::parser::TaskType),
    /// Change split type
    ChangeSplitType(String, crate::parser::SplitType),
    /// Change join type
    ChangeJoinType(String, crate::parser::JoinType),
    /// Remove connection
    RemoveConnection(String, String),
    /// Add connection
    AddConnection(String, String),
}

/// Mutation tester
pub struct MutationTester {
    /// Original workflow specification
    original: WorkflowSpec,
    /// Test fixture
    fixture: WorkflowTestFixture,
    /// Mutations applied
    mutations: Vec<MutationOperator>,
}

impl MutationTester {
    /// Create new mutation tester
    pub fn new(original: WorkflowSpec) -> WorkflowResult<Self> {
        Ok(Self {
            original,
            fixture: WorkflowTestFixture::new()?,
            mutations: vec![],
        })
    }

    /// Apply mutation operator
    pub fn apply_mutation(&mut self, mutation: MutationOperator) -> WorkflowSpec {
        self.mutations.push(mutation.clone());
        self.mutate_spec(&self.original.clone(), mutation)
    }

    fn mutate_spec(&self, spec: &WorkflowSpec, mutation: MutationOperator) -> WorkflowSpec {
        let mut mutated = spec.clone();

        match mutation {
            MutationOperator::RemoveTask(task_id) => {
                mutated.tasks.remove(&task_id);
            }
            MutationOperator::AddTask(mut task) => {
                // Ensure all required fields are set (Task struct fields)
                task.input_conditions = vec![];
                task.output_conditions = vec![];
                task.max_ticks = None;
                task.priority = None;
                task.use_simd = false;
                mutated.tasks.insert(task.id.clone(), task);
            }
            MutationOperator::ChangeTaskType(task_id, new_type) => {
                if let Some(task) = mutated.tasks.get_mut(&task_id) {
                    task.task_type = new_type;
                }
            }
            MutationOperator::ChangeSplitType(task_id, new_split) => {
                if let Some(task) = mutated.tasks.get_mut(&task_id) {
                    task.split_type = new_split;
                }
            }
            MutationOperator::ChangeJoinType(task_id, new_join) => {
                if let Some(task) = mutated.tasks.get_mut(&task_id) {
                    task.join_type = new_join;
                }
            }
            MutationOperator::RemoveConnection(from, to) => {
                if let Some(task) = mutated.tasks.get_mut(&from) {
                    task.outgoing_flows.retain(|flow| flow != &to);
                }
            }
            MutationOperator::AddConnection(from, to) => {
                if let Some(task) = mutated.tasks.get_mut(&from) {
                    if !task.outgoing_flows.contains(&to) {
                        task.outgoing_flows.push(to);
                    }
                }
            }
        }

        mutated
    }

    /// Test if mutation is caught by tests
    pub async fn test_mutation_detection<F>(&mut self, test_fn: F) -> bool
    where
        F: Fn(&WorkflowSpec) -> bool,
    {
        // Test original (should pass)
        if !test_fn(&self.original) {
            return false; // Original test fails - invalid test
        }

        // Apply each mutation and test
        let mutations = self.mutations.clone();
        for mutation in mutations {
            let mutated = self.apply_mutation(mutation);
            if test_fn(&mutated) {
                // Mutation not detected - test quality issue
                return false;
            }
        }

        true
    }
}

/// Mutation score (percentage of mutations caught)
pub struct MutationScore {
    /// Total mutations tested
    total: usize,
    /// Mutations caught by tests
    caught: usize,
    /// Score percentage
    score: f64,
}

impl MutationScore {
    /// Calculate mutation score
    pub fn calculate(caught: usize, total: usize) -> Self {
        let score = if total > 0 {
            (caught as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total,
            caught,
            score,
        }
    }

    /// Get score percentage
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Is score acceptable? (>= 80%)
    pub fn is_acceptable(&self) -> bool {
        self.score >= 80.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{SplitType, TaskType};

    #[test]
    fn test_mutation_tester_creates() {
        let spec = crate::testing::chicago_tdd::WorkflowSpecBuilder::new("Test").build();
        let tester = MutationTester::new(spec);
        assert!(tester.is_ok());
    }

    #[test]
    fn test_mutation_remove_task() {
        let spec = crate::testing::chicago_tdd::WorkflowSpecBuilder::new("Test")
            .add_task(crate::testing::chicago_tdd::TaskBuilder::new("task:1", "Task 1").build())
            .build();
        let mut tester =
            MutationTester::new(spec).expect("MutationTester::new should succeed for valid spec");

        let mutated = tester.apply_mutation(MutationOperator::RemoveTask("task:1".to_string()));
        assert!(!mutated.tasks.contains_key("task:1"));
    }

    #[test]
    fn test_mutation_score_calculation() {
        let score = MutationScore::calculate(8, 10);
        assert_eq!(score.score(), 80.0);
        assert!(score.is_acceptable());
    }
}
