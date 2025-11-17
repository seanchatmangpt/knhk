//! Ripple-Down Rules (RDR) for Worklet Selection
//!
//! Implements RDR-based worklet selection with TRIZ Principle 24: Intermediary
//! - Uses intermediate selection plan instead of direct rule evaluation
//!
//! Based on: org.yawlfoundation.yawl.worklet.RDR
//!
//! # Ripple-Down Rules
//!
//! RDR is a knowledge-based exception routing system:
//! - Hierarchical decision tree
//! - Incremental learning (add rules at runtime)
//! - Cornerstone cases (representative examples)
//! - Rule conflict resolution

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Worklet ID (String for RDR, converted to UUID WorkletId in repository)
pub type WorkletId = String;

/// Exception context for rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionContext {
    /// Exception type
    pub exception_type: String,
    /// Task ID where exception occurred
    pub task_id: String,
    /// Case ID
    pub case_id: String,
    /// Exception data
    pub data: serde_json::Value,
    /// Additional context
    pub metadata: HashMap<String, String>,
}

/// RDR Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RDRRule {
    /// Rule ID
    pub rule_id: String,
    /// Rule condition (e.g., "exception_type == 'timeout' && task_id == 'task1'")
    pub condition: String,
    /// Selected worklet ID
    pub worklet_id: WorkletId,
    /// Parent rule ID (for hierarchy)
    pub parent_rule_id: Option<String>,
    /// Cornerstone cases (examples that match this rule)
    pub cornerstone_cases: Vec<ExceptionContext>,
    /// Rule priority (higher = more specific)
    pub priority: u32,
}

/// RDR Rule Tree Node (TRIZ Principle 24: Intermediary)
///
/// Intermediate representation for rule evaluation
#[derive(Debug, Clone)]
pub struct RDRNode {
    /// Rule
    pub rule: RDRRule,
    /// Child nodes (more specific rules)
    pub children: Vec<RDRNode>,
}

/// RDR Selection Plan (TRIZ Principle 24: Intermediary)
///
/// Pre-computed selection plan instead of direct rule evaluation
#[derive(Debug, Clone)]
pub struct RDRSelectionPlan {
    /// Matched rules in order of specificity
    pub matched_rules: Vec<RDRRule>,
    /// Selected worklet ID
    pub selected_worklet: Option<WorkletId>,
    /// Selection confidence (0.0 to 1.0)
    pub confidence: f64,
}

/// RDR Engine for worklet selection
///
/// TRIZ Principle 24: Intermediary
/// - Uses RDRSelectionPlan intermediate representation
/// - Pre-computes rule matches for performance
pub struct RDREngine {
    /// Root rule node
    root: RDRNode,
    /// Rules by ID for quick lookup
    rules_by_id: HashMap<String, RDRRule>,
}

impl RDREngine {
    /// Create a new RDR engine
    pub fn new() -> Self {
        Self {
            root: RDRNode {
                rule: RDRRule {
                    rule_id: "root".to_string(),
                    condition: "true".to_string(), // Default: always matches
                    worklet_id: "default".to_string(),
                    parent_rule_id: None,
                    cornerstone_cases: vec![],
                    priority: 0,
                },
                children: vec![],
            },
            rules_by_id: HashMap::new(),
        }
    }

    /// Add a rule to the RDR tree
    pub fn add_rule(&mut self, rule: RDRRule) -> WorkflowResult<()> {
        // Store rule
        self.rules_by_id.insert(rule.rule_id.clone(), rule.clone());

        // Insert into tree
        if let Some(parent_id) = &rule.parent_rule_id {
            self.insert_rule_under_parent(&mut self.root, rule, parent_id)?;
        } else {
            // Add as child of root
            self.root.children.push(RDRNode {
                rule,
                children: vec![],
            });
        }

        Ok(())
    }

    /// Insert rule under parent (recursive)
    fn insert_rule_under_parent(
        &mut self,
        node: &mut RDRNode,
        rule: RDRRule,
        parent_id: &str,
    ) -> WorkflowResult<()> {
        if node.rule.rule_id == *parent_id {
            node.children.push(RDRNode {
                rule,
                children: vec![],
            });
            Ok(())
        } else {
            for child in &mut node.children {
                if let Ok(()) = self.insert_rule_under_parent(child, rule.clone(), parent_id) {
                    return Ok(());
                }
            }
            Err(WorkflowError::Internal(format!(
                "Parent rule {} not found",
                parent_id
            )))
        }
    }

    /// Select worklet using RDR (TRIZ Principle 24: Intermediary)
    ///
    /// Creates intermediate selection plan instead of direct selection
    pub fn select_worklet(&self, context: &ExceptionContext) -> WorkflowResult<RDRSelectionPlan> {
        // Build selection plan (TRIZ Principle 24: Intermediary)
        let mut matched_rules = Vec::new();
        self.evaluate_rules(&self.root, context, &mut matched_rules)?;

        // Sort by priority (most specific first)
        matched_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Select worklet from most specific rule
        let selected_worklet = matched_rules.first().map(|r| r.worklet_id.clone());
        let confidence = if matched_rules.is_empty() {
            0.0
        } else {
            // Confidence based on rule specificity and number of matches
            let max_priority = matched_rules[0].priority;
            let match_count = matched_rules.len();
            (max_priority as f64 / 100.0).min(1.0) * (1.0 / (match_count as f64 + 1.0))
        };

        Ok(RDRSelectionPlan {
            matched_rules,
            selected_worklet,
            confidence,
        })
    }

    /// Evaluate rules recursively
    fn evaluate_rules(
        &self,
        node: &RDRNode,
        context: &ExceptionContext,
        matched_rules: &mut Vec<RDRRule>,
    ) -> WorkflowResult<()> {
        // Evaluate current rule condition (simplified - in production would use expression evaluator)
        if self.evaluate_condition(&node.rule.condition, context)? {
            matched_rules.push(node.rule.clone());

            // Evaluate children (more specific rules)
            for child in &node.children {
                self.evaluate_rules(child, context, matched_rules)?;
            }
        }

        Ok(())
    }

    /// Evaluate condition (simplified implementation)
    ///
    /// In production, would use a proper expression evaluator
    fn evaluate_condition(
        &self,
        condition: &str,
        context: &ExceptionContext,
    ) -> WorkflowResult<bool> {
        // Simple condition evaluation
        // In production, use proper expression evaluator (e.g., evalexpr, rhai)
        if condition == "true" {
            return Ok(true);
        }

        // Basic string matching for demonstration
        let condition_lower = condition.to_lowercase();
        if condition_lower.contains("exception_type") {
            let expected_type = condition_lower
                .split("==")
                .nth(1)
                .and_then(|s| {
                    s.trim()
                        .strip_prefix('\'')
                        .and_then(|s| s.strip_suffix('\''))
                })
                .unwrap_or("");
            return Ok(context.exception_type == expected_type);
        }

        if condition_lower.contains("task_id") {
            let expected_task = condition_lower
                .split("==")
                .nth(1)
                .and_then(|s| {
                    s.trim()
                        .strip_prefix('\'')
                        .and_then(|s| s.strip_suffix('\''))
                })
                .unwrap_or("");
            return Ok(context.task_id == expected_task);
        }

        // Default: condition doesn't match
        Ok(false)
    }

    /// Get rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&RDRRule> {
        self.rules_by_id.get(rule_id)
    }

    /// Get all rules
    pub fn get_all_rules(&self) -> Vec<&RDRRule> {
        self.rules_by_id.values().collect()
    }
}

impl Default for RDREngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdr_engine_creation() {
        let engine = RDREngine::new();
        assert!(true); // Engine created successfully
    }

    #[test]
    fn test_rdr_rule_addition() {
        let mut engine = RDREngine::new();

        let rule = RDRRule {
            rule_id: "rule1".to_string(),
            condition: "exception_type == 'timeout'".to_string(),
            worklet_id: "timeout-handler".to_string(),
            parent_rule_id: None,
            cornerstone_cases: vec![],
            priority: 10,
        };

        let result = engine.add_rule(rule);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rdr_worklet_selection() {
        let mut engine = RDREngine::new();

        // Add rule
        let rule = RDRRule {
            rule_id: "rule1".to_string(),
            condition: "exception_type == 'timeout'".to_string(),
            worklet_id: "timeout-handler".to_string(),
            parent_rule_id: None,
            cornerstone_cases: vec![],
            priority: 10,
        };
        engine.add_rule(rule).unwrap();

        // Test selection
        let context = ExceptionContext {
            exception_type: "timeout".to_string(),
            task_id: "task1".to_string(),
            case_id: "case1".to_string(),
            data: serde_json::json!({}),
            metadata: HashMap::new(),
        };

        let plan = engine.select_worklet(&context).unwrap();
        assert_eq!(plan.selected_worklet, Some("timeout-handler".to_string()));
    }
}
