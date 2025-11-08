// legacy.rs: Legacy facade for YAWL-native behaviors
// Analyzes workflow specs for hot-path compatibility and promotion

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use crate::reflex::ReflexBridge;
use serde_json::Value;

/// Legacy facade: analyzes YAWL-native behaviors for hot-path compatibility
pub struct LegacyFacade {
    /// Reflex bridge for promotion analysis
    reflex_bridge: ReflexBridge,
}

impl LegacyFacade {
    /// Create a new legacy facade
    pub fn new() -> Self {
        Self {
            reflex_bridge: ReflexBridge::new(),
        }
    }

    /// Check if workflow spec has promotable segments
    ///
    /// Analyzes YAWL-native behaviors to determine if workflow can be promoted to hot path:
    /// - Checks tick budget compliance (≤8 ticks)
    /// - Verifies SLO compliance
    /// - Validates pattern safety for hot path
    pub fn promoteable(&self, spec: &WorkflowSpec) -> bool {
        // Use reflex bridge to analyze promotion opportunities
        let analysis = self.reflex_bridge.analyze_promotion_opportunities(spec);

        // Workflow is promotable if it has at least one hot path candidate
        // and meets promotion criteria
        if analysis.hot_path_candidates == 0 {
            return false;
        }

        // Check that hot path candidates meet tick budget
        for task in spec.tasks.values() {
            if let Some(max_ticks) = task.max_ticks {
                if max_ticks <= 8 {
                    // Check task is safe for hot path
                    if self.is_task_promotable(task) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if individual task is promotable
    fn is_task_promotable(&self, task: &crate::parser::Task) -> bool {
        // Atomic tasks with low tick budget are promotable
        if task.task_type != crate::parser::TaskType::Atomic {
            return false;
        }

        // Check tick budget
        if let Some(max_ticks) = task.max_ticks {
            if max_ticks > 8 {
                return false;
            }
        } else {
            return false; // No tick budget specified
        }

        // Check for blocking capabilities
        if task.required_capabilities.iter().any(|cap| {
            cap.contains("io") || cap.contains("network") || cap.contains("database")
        }) {
            return false;
        }

        true
    }

    /// Analyze workflow spec for promotion compatibility
    pub fn analyze_promotion(&self, spec: &WorkflowSpec) -> PromotionCompatibility {
        let analysis = self.reflex_bridge.analyze_promotion_opportunities(spec);
        let mut promotable_tasks = 0;
        let mut non_promotable_tasks = 0;

        for task in spec.tasks.values() {
            if self.is_task_promotable(task) {
                promotable_tasks += 1;
            } else {
                non_promotable_tasks += 1;
            }
        }

        PromotionCompatibility {
            promotable_tasks,
            non_promotable_tasks,
            hot_path_candidates: analysis.hot_path_candidates,
            warm_path_candidates: analysis.warm_path_candidates,
            cold_path_candidates: analysis.cold_path_candidates,
            overall_promotable: promotable_tasks > 0,
        }
    }
}

impl Default for LegacyFacade {
    fn default() -> Self {
        Self::new()
    }
}

/// Promotion compatibility analysis
#[derive(Debug, Clone)]
pub struct PromotionCompatibility {
    pub promotable_tasks: usize,
    pub non_promotable_tasks: usize,
    pub hot_path_candidates: usize,
    pub warm_path_candidates: usize,
    pub cold_path_candidates: usize,
    pub overall_promotable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_facade_creation() {
        let facade = LegacyFacade::new();
        assert!(facade.reflex_bridge.max_tick_budget <= 8);
    }

    #[test]
    fn test_promoteable_check() {
        let facade = LegacyFacade::new();
        
        let mut spec = crate::parser::WorkflowSpec {
            id: crate::parser::WorkflowSpecId::new(),
            name: "Test Workflow".to_string(),
            tasks: std::collections::HashMap::new(),
            conditions: std::collections::HashMap::new(),
            start_condition: None,
            end_condition: None,
        };

        // Add promotable task
        let task = crate::parser::Task {
            id: "task1".to_string(),
            name: "Promotable Task".to_string(),
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
            exception_worklet: None,
        };

        spec.tasks.insert("task1".to_string(), task);

        assert!(facade.promoteable(&spec));
    }

    #[test]
    fn test_non_promoteable_check() {
        let facade = LegacyFacade::new();
        
        let mut spec = crate::parser::WorkflowSpec {
            id: crate::parser::WorkflowSpecId::new(),
            name: "Test Workflow".to_string(),
            tasks: std::collections::HashMap::new(),
            conditions: std::collections::HashMap::new(),
            start_condition: None,
            end_condition: None,
        };

        // Add non-promotable task (composite)
        let task = crate::parser::Task {
            id: "task1".to_string(),
            name: "Non-Promotable Task".to_string(),
            task_type: crate::parser::TaskType::Composite, // Not promotable
            split_type: crate::parser::SplitType::And,
            join_type: crate::parser::JoinType::And,
            max_ticks: Some(4),
            priority: None,
            use_simd: false,
            input_conditions: Vec::new(),
            output_conditions: Vec::new(),
            outgoing_flows: Vec::new(),
            incoming_flows: Vec::new(),
            allocation_policy: None,
            required_roles: Vec::new(),
            required_capabilities: Vec::new(),
            exception_worklet: None,
        };

        spec.tasks.insert("task1".to_string(), task);

        assert!(!facade.promoteable(&spec));
    }
}

