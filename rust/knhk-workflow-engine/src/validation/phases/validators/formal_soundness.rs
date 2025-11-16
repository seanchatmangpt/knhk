//! Formal Soundness Validator
//!
//! Implements formal verification of workflow soundness properties:
//! 1. Option to Complete: Every task can eventually complete
//! 2. Proper Completion: Workflow reaches proper end state
//! 3. No Dead Tasks: No unreachable tasks
//!
//! Based on van der Aalst's workflow verification theory.

use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Instant;

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::PatternId;
use crate::validation::phases::core::{Phase, PhaseContext, PhaseMetadata, PhaseResult, PhaseStatus};
use crate::WorkflowSpec;

/// Formal soundness validation output
#[derive(Debug, Clone)]
pub struct FormalSoundnessData {
    /// Option to complete check passed
    pub option_to_complete: bool,
    /// Proper completion check passed
    pub proper_completion: bool,
    /// No dead tasks check passed
    pub no_dead_tasks: bool,
    /// Reachable tasks
    pub reachable_tasks: HashSet<String>,
    /// Dead tasks (unreachable)
    pub dead_tasks: HashSet<String>,
    /// Completion paths count
    pub completion_paths: usize,
    /// State space size (states explored)
    pub state_space_size: usize,
}

/// Formal soundness phase
pub struct FormalSoundnessPhase<M = ()> {
    _phantom: PhantomData<M>,
}

impl<M> FormalSoundnessPhase<M> {
    /// Create a new formal soundness phase
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<M> Default for FormalSoundnessPhase<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Send + Sync> Phase<FormalSoundnessData, M> for FormalSoundnessPhase<M> {
    fn metadata() -> PhaseMetadata {
        PhaseMetadata {
            name: "formal_soundness",
            description: "Formal verification of workflow soundness properties",
            version: "1.0.0",
            dependencies: &[],
            parallel: true,
        }
    }

    async fn execute(
        &self,
        ctx: PhaseContext,
    ) -> WorkflowResult<PhaseResult<FormalSoundnessData>> {
        let start = Instant::now();

        // Get workflow spec
        let spec = ctx
            .engine
            .get_spec(&ctx.spec_id)
            .await
            .ok_or_else(|| WorkflowError::SpecNotFound(ctx.spec_id.to_string()))?;

        // Perform soundness checks
        let soundness_data = verify_soundness(&spec).await?;

        // Determine status
        let status = if soundness_data.option_to_complete
            && soundness_data.proper_completion
            && soundness_data.no_dead_tasks
        {
            PhaseStatus::Pass
        } else {
            PhaseStatus::Fail
        };

        let passed = [
            soundness_data.option_to_complete,
            soundness_data.proper_completion,
            soundness_data.no_dead_tasks,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        let failed = 3 - passed;

        let mut result = PhaseResult::new("formal_soundness", status, soundness_data.clone())
            .with_duration(start.elapsed())
            .with_counts(passed, failed, 0);

        // Add metrics
        result.add_metric("reachable_tasks", soundness_data.reachable_tasks.len() as f64);
        result.add_metric("dead_tasks", soundness_data.dead_tasks.len() as f64);
        result.add_metric("completion_paths", soundness_data.completion_paths as f64);
        result.add_metric("state_space_size", soundness_data.state_space_size as f64);

        // Add messages
        if !soundness_data.option_to_complete {
            result.add_message("FAIL: Not all tasks can complete");
        }
        if !soundness_data.proper_completion {
            result.add_message("FAIL: Workflow does not properly complete");
        }
        if !soundness_data.no_dead_tasks {
            result.add_message(format!(
                "FAIL: {} dead tasks found: {:?}",
                soundness_data.dead_tasks.len(),
                soundness_data.dead_tasks
            ));
        }
        if status == PhaseStatus::Pass {
            result.add_message("PASS: Workflow is formally sound");
        }

        Ok(result)
    }
}

/// Verify workflow soundness properties
async fn verify_soundness(spec: &WorkflowSpec) -> WorkflowResult<FormalSoundnessData> {
    // Build task graph from workflow spec
    let task_graph = build_task_graph(spec);

    // Check 1: Find reachable tasks (BFS from start)
    let reachable_tasks = find_reachable_tasks(&task_graph);

    // Check 2: Find dead tasks
    let all_tasks: HashSet<String> = task_graph.keys().cloned().collect();
    let dead_tasks: HashSet<String> = all_tasks
        .difference(&reachable_tasks)
        .cloned()
        .collect();

    // Check 3: Verify all reachable tasks can complete (reach end state)
    let option_to_complete = verify_option_to_complete(&task_graph, &reachable_tasks);

    // Check 4: Count completion paths
    let completion_paths = count_completion_paths(&task_graph);

    // Check 5: Verify proper completion (exactly one end state reachable)
    let proper_completion = verify_proper_completion(&task_graph, &reachable_tasks);

    // State space size (approximate from reachability)
    let state_space_size = reachable_tasks.len();

    Ok(FormalSoundnessData {
        option_to_complete,
        proper_completion,
        no_dead_tasks: dead_tasks.is_empty(),
        reachable_tasks,
        dead_tasks,
        completion_paths,
        state_space_size,
    })
}

/// Build task dependency graph from workflow spec
fn build_task_graph(spec: &WorkflowSpec) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    // Add start node
    graph.insert("__start__".to_string(), Vec::new());

    // Add tasks from workflow patterns
    for task in &spec.tasks {
        let task_id = task.id.to_string();
        let successors = task
            .successors
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        graph.insert(task_id, successors);
    }

    // Add end node
    graph.insert("__end__".to_string(), Vec::new());

    // Connect orphan tasks to start
    if let Some(start_successors) = graph.get_mut("__start__") {
        for task in &spec.tasks {
            // If task has no predecessors, connect from start
            let task_id = task.id.to_string();
            let has_predecessor = spec.tasks.iter().any(|t| {
                t.successors
                    .iter()
                    .any(|s| s.to_string() == task_id)
            });

            if !has_predecessor && task_id != "__start__" && task_id != "__end__" {
                start_successors.push(task_id);
            }
        }
    }

    // Connect tasks with no successors to end
    for task in &spec.tasks {
        let task_id = task.id.to_string();
        if let Some(successors) = graph.get(&task_id) {
            if successors.is_empty() && task_id != "__end__" {
                graph.get_mut(&task_id).unwrap().push("__end__".to_string());
            }
        }
    }

    graph
}

/// Find all reachable tasks via BFS from start
fn find_reachable_tasks(graph: &HashMap<String, Vec<String>>) -> HashSet<String> {
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back("__start__".to_string());
    reachable.insert("__start__".to_string());

    while let Some(task) = queue.pop_front() {
        if let Some(successors) = graph.get(&task) {
            for successor in successors {
                if !reachable.contains(successor) {
                    reachable.insert(successor.clone());
                    queue.push_back(successor.clone());
                }
            }
        }
    }

    reachable
}

/// Verify option to complete: all reachable tasks can reach end state
fn verify_option_to_complete(
    graph: &HashMap<String, Vec<String>>,
    reachable_tasks: &HashSet<String>,
) -> bool {
    // Build reverse graph
    let mut reverse_graph: HashMap<String, Vec<String>> = HashMap::new();
    for (task, successors) in graph {
        for successor in successors {
            reverse_graph
                .entry(successor.clone())
                .or_default()
                .push(task.clone());
        }
    }

    // BFS backward from end
    let mut can_reach_end = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back("__end__".to_string());
    can_reach_end.insert("__end__".to_string());

    while let Some(task) = queue.pop_front() {
        if let Some(predecessors) = reverse_graph.get(&task) {
            for predecessor in predecessors {
                if !can_reach_end.contains(predecessor) {
                    can_reach_end.insert(predecessor.clone());
                    queue.push_back(predecessor.clone());
                }
            }
        }
    }

    // All reachable tasks must be able to reach end
    reachable_tasks.iter().all(|task| can_reach_end.contains(task))
}

/// Count completion paths (simple path count to end)
fn count_completion_paths(graph: &HashMap<String, Vec<String>>) -> usize {
    fn count_paths_recursive(
        current: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
    ) -> usize {
        if current == "__end__" {
            return 1;
        }

        if visited.contains(current) {
            return 0; // Cycle detected
        }

        visited.insert(current.to_string());

        let mut total_paths = 0;
        if let Some(successors) = graph.get(current) {
            for successor in successors {
                total_paths += count_paths_recursive(successor, graph, visited);
            }
        }

        visited.remove(current);
        total_paths
    }

    let mut visited = HashSet::new();
    count_paths_recursive("__start__", graph, &mut visited)
}

/// Verify proper completion: exactly one end state reachable from all paths
fn verify_proper_completion(
    graph: &HashMap<String, Vec<String>>,
    reachable_tasks: &HashSet<String>,
) -> bool {
    // End state must be reachable
    if !reachable_tasks.contains("__end__") {
        return false;
    }

    // All terminal nodes (no successors) should be __end__
    for task in reachable_tasks {
        if let Some(successors) = graph.get(task) {
            if successors.is_empty() && task != "__end__" {
                return false; // Terminal node that's not __end__
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::WorkflowSpecId;
    use crate::state::StateStore;
    use crate::task::{Task, TaskId};
    use crate::WorkflowEngine;

    #[tokio::test]
    async fn test_formal_soundness_simple_workflow() {
        // Create a simple linear workflow: start -> task1 -> task2 -> end
        let spec = WorkflowSpec {
            id: WorkflowSpecId::default(),
            name: "test_workflow".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            tasks: vec![
                Task {
                    id: TaskId::parse_str("task1").unwrap(),
                    name: "Task 1".to_string(),
                    description: None,
                    pattern: PatternId::parse_str("sequence").unwrap(),
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                    successors: vec![TaskId::parse_str("task2").unwrap()],
                    guards: Vec::new(),
                },
                Task {
                    id: TaskId::parse_str("task2").unwrap(),
                    name: "Task 2".to_string(),
                    description: None,
                    pattern: PatternId::parse_str("sequence").unwrap(),
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                    successors: Vec::new(),
                    guards: Vec::new(),
                },
            ],
            metadata: HashMap::new(),
        };

        let soundness_data = verify_soundness(&spec).await.unwrap();

        assert!(soundness_data.option_to_complete);
        assert!(soundness_data.proper_completion);
        assert!(soundness_data.no_dead_tasks);
        assert_eq!(soundness_data.dead_tasks.len(), 0);
    }
}
