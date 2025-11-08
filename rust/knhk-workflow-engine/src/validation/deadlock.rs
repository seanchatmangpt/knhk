//! Deadlock Detection
//!
//! Implements YAWL-style deadlock detection for workflow specifications.
//! Uses Petri net analysis to detect potential deadlocks at design-time.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use std::collections::{HashMap, HashSet, VecDeque};

/// Deadlock detection result
#[derive(Debug, Clone)]
pub struct DeadlockDetectionResult {
    /// Whether deadlock was detected
    pub has_deadlock: bool,
    /// Deadlock locations (task/condition IDs)
    pub deadlock_locations: Vec<String>,
    /// Deadlock cycles (if any)
    pub cycles: Vec<Vec<String>>,
    /// Warnings
    pub warnings: Vec<String>,
}

/// Petri net node (task or condition)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PetriNetNode {
    Task(String),
    Condition(String),
}

/// Petri net analyzer
pub struct DeadlockDetector;

impl DeadlockDetector {
    /// Detect deadlocks in workflow specification
    pub fn detect_deadlocks(&self, spec: &WorkflowSpec) -> DeadlockDetectionResult {
        let mut has_deadlock = false;
        let mut deadlock_locations = Vec::new();
        let mut cycles = Vec::new();
        let mut warnings = Vec::new();

        // Build Petri net graph
        let graph = self.build_petri_net(spec);

        // Detect cycles (potential deadlocks)
        let detected_cycles = self.detect_cycles(&graph);

        if !detected_cycles.is_empty() {
            has_deadlock = true;
            for cycle in &detected_cycles {
                cycles.push(cycle.iter().map(|n| self.node_to_string(n)).collect());
                deadlock_locations.extend(cycle.iter().map(|n| self.node_to_string(n)));
            }
        }

        // Check for unreachable tasks
        let unreachable = self.find_unreachable_tasks(spec, &graph);
        if !unreachable.is_empty() {
            warnings.push(format!("Unreachable tasks detected: {:?}", unreachable));
        }

        // Check for tasks without outgoing flows (potential dead ends)
        let dead_ends = self.find_dead_ends(spec);
        if !dead_ends.is_empty() {
            warnings.push(format!("Dead-end tasks detected: {:?}", dead_ends));
        }

        DeadlockDetectionResult {
            has_deadlock,
            deadlock_locations,
            cycles,
            warnings,
        }
    }

    /// Build Petri net graph from workflow specification
    fn build_petri_net(&self, spec: &WorkflowSpec) -> HashMap<PetriNetNode, Vec<PetriNetNode>> {
        let mut graph: HashMap<PetriNetNode, Vec<PetriNetNode>> = HashMap::new();

        // Add edges from conditions to tasks
        for (condition_id, condition) in &spec.conditions {
            let condition_node = PetriNetNode::Condition(condition_id.clone());
            for task_id in &condition.outgoing_flows {
                let task_node = PetriNetNode::Task(task_id.to_string());
                graph
                    .entry(condition_node.clone())
                    .or_insert_with(Vec::new)
                    .push(task_node.clone());
            }
        }

        // Add edges from tasks to conditions
        for (task_id, task) in &spec.tasks {
            let task_node = PetriNetNode::Task(task_id.clone());
            for condition_id in &task.outgoing_flows {
                let condition_node = PetriNetNode::Condition(condition_id.clone());
                graph
                    .entry(task_node.clone())
                    .or_insert_with(Vec::new)
                    .push(condition_node.clone());
            }
        }

        graph
    }

    /// Detect cycles in Petri net (potential deadlocks)
    fn detect_cycles(
        &self,
        graph: &HashMap<PetriNetNode, Vec<PetriNetNode>>,
    ) -> Vec<Vec<PetriNetNode>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                self.dfs_cycle_detection(
                    node,
                    graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// Depth-first search for cycle detection
    fn dfs_cycle_detection(
        &self,
        node: &PetriNetNode,
        graph: &HashMap<PetriNetNode, Vec<PetriNetNode>>,
        visited: &mut HashSet<PetriNetNode>,
        rec_stack: &mut HashSet<PetriNetNode>,
        path: &mut Vec<PetriNetNode>,
        cycles: &mut Vec<Vec<PetriNetNode>>,
    ) {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());
        path.push(node.clone());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_cycle_detection(neighbor, graph, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Cycle detected - extract cycle from path
                    if let Some(start_idx) = path.iter().position(|n| n == neighbor) {
                        let cycle: Vec<PetriNetNode> = path[start_idx..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
    }

    /// Find unreachable tasks (tasks that cannot be reached from start)
    fn find_unreachable_tasks(
        &self,
        spec: &WorkflowSpec,
        graph: &HashMap<PetriNetNode, Vec<PetriNetNode>>,
    ) -> Vec<String> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from start condition
        if let Some(ref start_condition) = spec.start_condition {
            let start_node = PetriNetNode::Condition(start_condition.clone());
            queue.push_back(start_node.clone());
            reachable.insert(start_node);
        }

        // BFS to find all reachable nodes
        while let Some(node) = queue.pop_front() {
            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    if !reachable.contains(neighbor) {
                        reachable.insert(neighbor.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        // Find unreachable tasks
        spec.tasks
            .keys()
            .filter(|task_id| {
                let task_node = PetriNetNode::Task(task_id.to_string());
                !reachable.contains(&task_node)
            })
            .cloned()
            .collect()
    }

    /// Find dead-end tasks (tasks without outgoing flows)
    fn find_dead_ends(&self, spec: &WorkflowSpec) -> Vec<String> {
        spec.tasks
            .iter()
            .filter(|(_, task)| task.outgoing_flows.is_empty())
            .map(|(task_id, _)| task_id.clone())
            .collect()
    }

    /// Convert Petri net node to string
    fn node_to_string(&self, node: &PetriNetNode) -> String {
        match node {
            PetriNetNode::Task(id) => format!("task:{}", id),
            PetriNetNode::Condition(id) => format!("condition:{}", id),
        }
    }

    /// Validate workflow specification for deadlocks
    pub fn validate(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        let result = self.detect_deadlocks(spec);

        if result.has_deadlock {
            return Err(WorkflowError::Validation(format!(
                "Deadlock detected in workflow: cycles at {:?}",
                result.deadlock_locations
            )));
        }

        // Log warnings but don't fail validation
        if !result.warnings.is_empty() {
            // In production, would log these warnings
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deadlock_detection_no_deadlock() {
        let detector = DeadlockDetector;
        let spec = WorkflowSpec {
            id: crate::parser::WorkflowSpecId::new(),
            name: "Simple Workflow".to_string(),
            tasks: HashMap::new(),
            conditions: HashMap::new(),
            start_condition: None,
            end_condition: None,
        };

        let result = detector.detect_deadlocks(&spec);
        assert!(!result.has_deadlock);
    }

    #[test]
    fn test_deadlock_detection_with_cycle() {
        let detector = DeadlockDetector;
        let mut spec = WorkflowSpec {
            id: crate::parser::WorkflowSpecId::new(),
            name: "Cyclic Workflow".to_string(),
            tasks: HashMap::new(),
            conditions: HashMap::new(),
            start_condition: None,
            end_condition: None,
        };

        // Create a simple cycle: task1 -> condition1 -> task1
        let task1 = Task {
            id: "task1".to_string(),
            name: "Task 1".to_string(),
            task_type: crate::parser::TaskType::Atomic,
            split_type: crate::parser::SplitType::Xor,
            join_type: crate::parser::JoinType::Xor,
            max_ticks: None,
            priority: None,
            use_simd: false,
            input_conditions: vec!["condition1".to_string()],
            output_conditions: vec!["condition1".to_string()],
            outgoing_flows: vec!["condition1".to_string()],
            incoming_flows: vec!["condition1".to_string()],
        };

        let condition1 = crate::parser::Condition {
            id: "condition1".to_string(),
            name: "Condition 1".to_string(),
            outgoing_flows: vec!["task1".to_string()],
            incoming_flows: vec!["task1".to_string()],
        };

        spec.tasks.insert("task1".to_string(), task1);
        spec.conditions.insert("condition1".to_string(), condition1);

        let result = detector.detect_deadlocks(&spec);
        // Should detect cycle
        assert!(result.has_deadlock || !result.cycles.is_empty());
    }
}
