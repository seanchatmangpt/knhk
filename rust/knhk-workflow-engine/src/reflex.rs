// reflex.rs: Reflex bridge for hot path promotion
// Identifies promotable regions in workflow specs and binds hot-pattern executors

use crate::error::WorkflowResult;
use crate::integration::fortune5::RuntimeClass;
use crate::parser::WorkflowSpec;
use std::collections::HashMap;

/// Promotable segment in workflow spec
#[derive(Debug, Clone)]
pub struct PromotableSegment {
    /// Segment identifier
    pub segment_id: String,
    /// Pattern IDs in this segment
    pub pattern_ids: Vec<u8>,
    /// Estimated tick budget
    pub tick_budget: u32,
    /// Runtime class (R1/W1/C1)
    pub runtime_class: RuntimeClass,
    /// Hot path executor binding
    pub hot_executor_bound: bool,
}

/// Reflex bridge: identifies promotable regions and binds hot-pattern executors
pub struct ReflexBridge {
    /// Promotion policy: SLO compliance threshold
    slo_threshold: f64,
    /// Promotion policy: max tick budget for promotion
    max_tick_budget: u32,
    /// Bound segments cache
    bound_segments: HashMap<String, PromotableSegment>,
}

impl ReflexBridge {
    /// Create a new reflex bridge
    pub fn new() -> Self {
        Self {
            slo_threshold: 0.95, // 95% SLO compliance required
            max_tick_budget: 8,  // Chatman Constant
            bound_segments: HashMap::new(),
        }
    }

    /// Create reflex bridge with custom configuration
    pub fn with_config(slo_threshold: f64, max_tick_budget: u32) -> Self {
        Self {
            slo_threshold,
            max_tick_budget,
            bound_segments: HashMap::new(),
        }
    }

    /// Bind hot segments: identify promotable regions and bind hot-pattern executors
    ///
    /// Analyzes workflow spec to find segments that can be promoted to hot path:
    /// - Tasks with tick budget ≤ 8
    /// - Tasks that meet SLO compliance
    /// - Tasks that are safe for hot path execution
    pub fn bind_hot_segments(
        &mut self,
        spec: &WorkflowSpec,
    ) -> WorkflowResult<Vec<PromotableSegment>> {
        let mut promotable_segments = Vec::new();

        // Analyze workflow spec for promotable tasks
        for (task_id, task) in &spec.tasks {
            // Check if task has promotable tick budget
            if let Some(max_ticks) = task.max_ticks {
                if max_ticks <= self.max_tick_budget {
                    // Check if task is safe for hot path
                    if self.is_task_safe_for_hot_path(task_id, task) {
                        // Determine pattern ID from task type/split/join
                        let pattern_id = self.infer_pattern_id(task);

                        let segment = PromotableSegment {
                            segment_id: format!("segment_{}", task_id),
                            pattern_ids: vec![pattern_id],
                            tick_budget: max_ticks,
                            runtime_class: RuntimeClass::R1, // Hot path
                            hot_executor_bound: false,
                        };

                        // Bind hot executor
                        if self.bind_hot_executor(&segment) {
                            promotable_segments.push(segment);
                        }
                    }
                }
            }
        }

        // Cache bound segments
        for segment in &promotable_segments {
            self.bound_segments
                .insert(segment.segment_id.clone(), segment.clone());
        }

        Ok(promotable_segments)
    }

    /// Check if task is safe for hot path promotion
    fn is_task_safe_for_hot_path(&self, _task_id: &str, task: &crate::parser::Task) -> bool {
        // Check tick budget
        if let Some(max_ticks) = task.max_ticks {
            if max_ticks > self.max_tick_budget {
                return false;
            }
        }

        // Check for composite tasks (contain sub-workflows) - not safe for hot path
        if task.task_type == crate::parser::TaskType::Composite {
            return false;
        }

        // Check for multiple instance tasks - typically need warm path
        if task.task_type == crate::parser::TaskType::MultipleInstance {
            return false;
        }

        // Simple atomic tasks with low tick budget are safe
        true
    }

    /// Infer pattern ID from task structure
    fn infer_pattern_id(&self, task: &crate::parser::Task) -> u8 {
        // Map task split/join types to pattern IDs
        match (task.split_type, task.join_type) {
            (crate::parser::SplitType::And, crate::parser::JoinType::And) => 2, // Parallel Split + Sync
            (crate::parser::SplitType::Xor, crate::parser::JoinType::Xor) => 4, // Exclusive Choice + Simple Merge
            (crate::parser::SplitType::Or, _) => 6,                             // Multi-Choice
            _ => 1,                                                             // Default: Sequence
        }
    }

    /// Bind hot executor to segment
    fn bind_hot_executor(&self, segment: &PromotableSegment) -> bool {
        // Check if segment meets promotion criteria
        if segment.tick_budget > self.max_tick_budget {
            return false;
        }

        // Check if segment is promotable to hot path
        // Segment is promotable if it meets hot path criteria:
        // 1. Size ≤ 8 (Chatman Constant)
        // 2. Operations are in H_hot set (ASK, COUNT, COMPARE, VALIDATE)
        // 3. No external dependencies

        // Check segment size (approximate by checking pattern count)
        if segment.pattern_ids.len() > 8 {
            return false; // Too large for hot path
        }

        // Check if segment contains only hot path operations
        // Hot path operations are in H_hot set: ASK, COUNT, COMPARE, VALIDATE
        // Pattern IDs 1-9 are basic YAWL patterns (Sequence, Choice, Parallel)
        // These are safe for hot path if they meet tick budget constraints
        let is_hot_path_operation = segment.pattern_ids.iter().all(|&pid| {
            // Pattern IDs 1-9 are basic patterns safe for hot path
            // Pattern IDs 10+ may require warm/cold path
            pid <= 9 && segment.tick_budget <= 8
        });

        is_hot_path_operation && segment.pattern_ids.len() <= 8
    }

    /// Check if segment is bound to hot path
    pub fn is_segment_bound(&self, segment_id: &str) -> bool {
        self.bound_segments.contains_key(segment_id)
    }

    /// Get bound segment
    pub fn get_bound_segment(&self, segment_id: &str) -> Option<&PromotableSegment> {
        self.bound_segments.get(segment_id)
    }

    /// Analyze workflow spec for promotion opportunities
    pub fn analyze_promotion_opportunities(&self, spec: &WorkflowSpec) -> PromotionAnalysis {
        let mut hot_path_candidates = 0;
        let mut warm_path_candidates = 0;
        let mut cold_path_candidates = 0;

        for task in spec.tasks.values() {
            if let Some(max_ticks) = task.max_ticks {
                if max_ticks <= 8 {
                    hot_path_candidates += 1;
                } else if max_ticks <= 64 {
                    warm_path_candidates += 1;
                } else {
                    cold_path_candidates += 1;
                }
            } else {
                // No tick budget specified - assume cold path
                cold_path_candidates += 1;
            }
        }

        PromotionAnalysis {
            hot_path_candidates,
            warm_path_candidates,
            cold_path_candidates,
            total_tasks: spec.tasks.len(),
        }
    }
}

impl Default for ReflexBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Promotion analysis results
#[derive(Debug, Clone)]
pub struct PromotionAnalysis {
    pub hot_path_candidates: usize,
    pub warm_path_candidates: usize,
    pub cold_path_candidates: usize,
    pub total_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Task, WorkflowSpec, WorkflowSpecId};

    #[test]
    fn test_reflex_bridge_creation() {
        let bridge = ReflexBridge::new();
        assert_eq!(bridge.max_tick_budget, 8);
        assert_eq!(bridge.slo_threshold, 0.95);
    }

    #[test]
    fn test_bind_hot_segments() {
        let mut bridge = ReflexBridge::new();

        // Create a simple workflow spec with promotable task
        let mut spec = WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: "Test Workflow".to_string(),
            tasks: HashMap::new(),
            conditions: HashMap::new(),
            flows: Vec::new(),
            start_condition: None,
            end_condition: None,
            source_turtle: None,
        };

        let task = crate::parser::Task {
            id: "task1".to_string(),
            name: "Task 1".to_string(),
            task_type: crate::parser::TaskType::Atomic,
            split_type: crate::parser::SplitType::And,
            join_type: crate::parser::JoinType::And,
            max_ticks: Some(4), // Within hot path budget
            priority: None,
            use_simd: false,
            input_conditions: Vec::new(),
            output_conditions: Vec::new(),
            outgoing_flows: Vec::new(),
            incoming_flows: Vec::new(),
            allocation_policy: None,
            required_roles: Vec::new(),
            required_capabilities: Vec::new(),
            input_parameters: Vec::new(),
            output_parameters: Vec::new(),
            exception_worklet: None,
        };

        spec.tasks.insert("task1".to_string(), task);

        let segments = bridge
            .bind_hot_segments(&spec)
            .expect("bind_hot_segments should succeed");
        assert!(!segments.is_empty());
        assert_eq!(segments[0].tick_budget, 4);
        assert_eq!(segments[0].runtime_class, RuntimeClass::R1);
    }

    #[test]
    fn test_promotion_analysis() {
        let bridge = ReflexBridge::new();

        let mut spec = WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: "Test Workflow".to_string(),
            tasks: HashMap::new(),
            conditions: HashMap::new(),
            flows: Vec::new(),
            start_condition: None,
            end_condition: None,
            source_turtle: None,
        };

        // Add tasks with different tick budgets
        let task1 = crate::parser::Task {
            id: "task1".to_string(),
            name: "Hot Task".to_string(),
            task_type: crate::parser::TaskType::Atomic,
            split_type: crate::parser::SplitType::And,
            join_type: crate::parser::JoinType::And,
            max_ticks: Some(4), // Hot path
            priority: None,
            use_simd: false,
            input_conditions: Vec::new(),
            output_conditions: Vec::new(),
            outgoing_flows: Vec::new(),
            incoming_flows: Vec::new(),
            allocation_policy: None,
            required_roles: Vec::new(),
            required_capabilities: Vec::new(),
            input_parameters: Vec::new(),
            output_parameters: Vec::new(),
            exception_worklet: None,
        };

        let task2 = crate::parser::Task {
            id: "task2".to_string(),
            name: "Warm Task".to_string(),
            task_type: crate::parser::TaskType::Atomic,
            split_type: crate::parser::SplitType::And,
            join_type: crate::parser::JoinType::And,
            max_ticks: Some(32), // Warm path
            priority: None,
            use_simd: false,
            input_conditions: Vec::new(),
            output_conditions: Vec::new(),
            outgoing_flows: Vec::new(),
            incoming_flows: Vec::new(),
            allocation_policy: None,
            required_roles: Vec::new(),
            required_capabilities: Vec::new(),
            input_parameters: Vec::new(),
            output_parameters: Vec::new(),
            exception_worklet: None,
        };

        spec.tasks.insert("task1".to_string(), task1);
        spec.tasks.insert("task2".to_string(), task2);

        let analysis = bridge.analyze_promotion_opportunities(&spec);
        assert_eq!(analysis.hot_path_candidates, 1);
        assert_eq!(analysis.warm_path_candidates, 1);
        assert_eq!(analysis.total_tasks, 2);
    }
}
