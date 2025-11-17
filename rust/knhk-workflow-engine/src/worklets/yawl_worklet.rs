//! YAWL Worklet System Port with TRIZ Hyper-Advanced Patterns
//!
//! This module ports Java YAWL's WorkletService and RDR (Ripple-Down Rules) system
//! while applying TRIZ principles:
//! - **Principle 15 (Dynamics)**: Runtime worklet selection
//! - **Principle 1 (Segmentation)**: Separate worklet execution from main engine
//! - **Principle 10 (Prior Action)**: Pre-index worklets by exception type
//!
//! # Architecture
//!
//! YAWL worklets provide dynamic workflow adaptation:
//! 1. **Selection**: RDR-based worklet selection when task is enabled
//! 2. **Execution**: Sub-workflow execution replaces original task
//! 3. **Exception Handling**: Worklets handle exceptions and compensation
//!
//! # TRIZ Enhancements
//!
//! - Worklets are pre-indexed by exception type (Principle 10)
//! - Selection is separated from execution (Principle 1)
//! - RDR evaluation adapts based on context (Principle 15)

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use crate::worklets::{Worklet, WorkletId, WorkletRepository};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Ripple-Down Rules (RDR) node
///
/// RDR is a knowledge acquisition technique where rules are organized in a tree.
/// Each node has a condition and two children (true/false branches).
///
/// # TRIZ Principle 24: Intermediary
///
/// RDR provides an intermediate representation for worklet selection rules.
#[derive(Debug, Clone)]
pub struct RdrNode {
    /// Node ID
    pub node_id: u64,
    /// Condition expression
    pub condition: String,
    /// Worklet ID if this node matches (conclusion)
    pub worklet_id: Option<WorkletId>,
    /// True child (condition matches)
    pub true_child: Option<Box<RdrNode>>,
    /// False child (condition doesn't match)
    pub false_child: Option<Box<RdrNode>>,
}

/// RDR Tree for worklet selection
///
/// This implements YAWL's Ripple-Down Rules for selecting worklets based on context.
///
/// # TRIZ Principle 24: Intermediary
///
/// RDR provides an intermediate rule representation that separates selection logic
/// from worklet execution.
pub struct RdrTree {
    /// Root node
    root: Option<RdrNode>,
    /// Task ID this tree is for (None for case-level)
    task_id: Option<String>,
}

impl RdrTree {
    /// Create a new RDR tree
    pub fn new(task_id: Option<String>) -> Self {
        Self {
            root: None,
            task_id,
        }
    }

    /// Evaluate the RDR tree against context
    ///
    /// Traverses the tree based on condition evaluation and returns the selected worklet ID.
    ///
    /// # TRIZ Principle 15: Dynamics
    ///
    /// Evaluation adapts based on runtime context.
    pub fn evaluate(&self, context: &WorkletContext) -> Option<WorkletId> {
        self.evaluate_node(&self.root, context)
    }

    /// Recursively evaluate a node
    fn evaluate_node(&self, node: &Option<RdrNode>, context: &WorkletContext) -> Option<WorkletId> {
        let node = node.as_ref()?;

        // Evaluate condition
        let matches = self.evaluate_condition(&node.condition, context);

        // If condition matches, check true child; otherwise check false child
        let child = if matches {
            &node.true_child
        } else {
            &node.false_child
        };

        // If we have a conclusion (worklet_id) at this node, return it
        if matches && node.worklet_id.is_some() {
            return node.worklet_id;
        }

        // Otherwise, recurse into child
        self.evaluate_node(child, context)
    }

    /// Evaluate a condition expression against context
    ///
    /// Supports simple boolean expressions like:
    /// - "variable == value"
    /// - "variable >= value"
    /// Evaluate RDR condition with full expression support
    ///
    /// Supports:
    /// - Equality: "variable == value"
    /// - Inequality: "variable != value"
    /// - Comparison: "variable > value", "variable < value", "variable >= value", "variable <= value"
    /// - Contains: "variable.contains(value)"
    /// - Boolean logic: "condition1 && condition2", "condition1 || condition2"
    /// - Negation: "!condition"
    fn evaluate_condition(&self, condition: &str, context: &WorkletContext) -> bool {
        // Parse and evaluate condition using recursive descent
        self.evaluate_expression(condition.trim(), context)
    }

    /// Recursive expression evaluator with operator precedence
    fn evaluate_expression(&self, expr: &str, context: &WorkletContext) -> bool {
        // Handle parentheses
        if let Some((start, end)) = self.find_matching_parens(expr) {
            let inner = &expr[start + 1..end];
            let result = self.evaluate_expression(inner, context);
            if start > 0 && end < expr.len() - 1 {
                // Continue evaluation with result
                let new_expr = format!("{}{}{}", &expr[..start], result, &expr[end + 1..]);
                return self.evaluate_expression(&new_expr, context);
            }
            return result;
        }

        // Handle logical OR (lowest precedence)
        if let Some(pos) = expr.find("||") {
            let left = self.evaluate_expression(&expr[..pos], context);
            let right = self.evaluate_expression(&expr[pos + 2..], context);
            return left || right;
        }

        // Handle logical AND
        if let Some(pos) = expr.find("&&") {
            let left = self.evaluate_expression(&expr[..pos], context);
            let right = self.evaluate_expression(&expr[pos + 2..], context);
            return left && right;
        }

        // Handle negation
        if expr.starts_with('!') {
            return !self.evaluate_expression(&expr[1..], context);
        }

        // Handle comparison operators
        for op in ["==", "!=", ">=", "<=", ">", "<"] {
            if let Some(pos) = expr.find(op) {
                let var = expr[..pos].trim();
                let value_str = expr[pos + op.len()..].trim().trim_matches('"');
                
                if let Some(var_value) = context.data.get(var) {
                    return match op {
                        "==" => self.compare_values(var_value, value_str, |a, b| a == b),
                        "!=" => self.compare_values(var_value, value_str, |a, b| a != b),
                        ">" => self.compare_values(var_value, value_str, |a, b| a > b),
                        "<" => self.compare_values(var_value, value_str, |a, b| a < b),
                        ">=" => self.compare_values(var_value, value_str, |a, b| a >= b),
                        "<=" => self.compare_values(var_value, value_str, |a, b| a <= b),
                        _ => false,
                    };
                }
                return false;
            }
        }

        // Handle contains
        if let Some(pos) = expr.find(".contains(") {
            let var = expr[..pos].trim();
            if let Some(start) = expr[pos + 10..].find('(') {
                if let Some(end) = expr[pos + 10 + start + 1..].find(')') {
                    let value = expr[pos + 10 + start + 1..pos + 10 + start + 1 + end].trim_matches('"');
                    if let Some(var_value) = context.data.get(var) {
                        if let Some(s) = var_value.as_str() {
                            return s.contains(value);
                        }
                    }
                }
            }
        }

        // Boolean literal
        if expr == "true" {
            return true;
        }
        if expr == "false" {
            return false;
        }

        // Variable lookup (truthy if exists and not empty/false)
        if let Some(value) = context.data.get(expr) {
            if let Some(b) = value.as_bool() {
                return b;
            }
            if let Some(s) = value.as_str() {
                return !s.is_empty();
            }
            return true;
        }

        false
    }

    /// Find matching parentheses in expression
    fn find_matching_parens(&self, expr: &str) -> Option<(usize, usize)> {
        let mut depth = 0;
        let mut start = None;
        for (i, ch) in expr.char_indices() {
            match ch {
                '(' => {
                    if depth == 0 {
                        start = Some(i);
                    }
                    depth += 1;
                }
                ')' => {
                    depth -= 1;
                    if depth == 0 && start.is_some() {
                        return Some((start.unwrap(), i));
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Compare values with type coercion
    fn compare_values<F>(&self, var_value: &serde_json::Value, value_str: &str, cmp: F) -> bool
    where
        F: FnOnce(&str, &str) -> bool,
    {
        let var_str = match var_value {
            serde_json::Value::String(s) => s.as_str(),
            serde_json::Value::Number(n) => n.to_string().as_str(),
            serde_json::Value::Bool(b) => if *b { "true" } else { "false" },
            _ => return false,
        };
        cmp(var_str, value_str)
    }

    /// Set root node
    pub fn set_root(&mut self, root: RdrNode) {
        self.root = Some(root);
    }
}

/// Worklet selection context
#[derive(Debug, Clone)]
pub struct WorkletContext {
    /// Task ID
    pub task_id: String,
    /// Case data
    pub data: serde_json::Value,
    /// Exception type (if exception handling)
    pub exception_type: Option<String>,
    /// Case ID
    pub case_id: String,
}

/// YAWL Worklet Service
///
/// Manages worklet selection, execution, and exception handling.
///
/// # TRIZ Principle 1: Segmentation
///
/// Worklet selection is separated from execution, allowing independent optimization.
///
/// # TRIZ Principle 10: Prior Action
///
/// Worklets are pre-indexed by exception type for fast lookup.
pub struct YawlWorkletService {
    /// Worklet repository
    repository: Arc<WorkletRepository>,
    /// RDR trees by task ID (None for case-level)
    rdr_trees: Arc<RwLock<HashMap<Option<String>, RdrTree>>>,
    /// Exception type to worklet mapping (pre-indexed - TRIZ Principle 10)
    exception_index: Arc<RwLock<HashMap<String, Vec<WorkletId>>>>,
}

impl YawlWorkletService {
    /// Create a new YAWL worklet service
    pub fn new(repository: Arc<WorkletRepository>) -> Self {
        Self {
            repository,
            rdr_trees: Arc::new(RwLock::new(HashMap::new())),
            exception_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Select a worklet for a task using RDR
    ///
    /// This implements YAWL's worklet selection process:
    /// 1. Get RDR tree for task (or case-level if no task-specific tree)
    /// 2. Evaluate tree against context
    /// 3. Return selected worklet ID
    ///
    /// # TRIZ Principle 15: Dynamics
    ///
    /// Selection adapts based on runtime context.
    pub async fn select_worklet(
        &self,
        task_id: Option<&str>,
        context: &WorkletContext,
    ) -> WorkflowResult<Option<WorkletId>> {
        let trees = self.rdr_trees.read().await;

        // Try task-specific tree first, then case-level
        let tree_key = task_id.map(|s| s.to_string());
        let tree = trees.get(&tree_key).or_else(|| trees.get(&None));

        if let Some(tree) = tree {
            let selected = tree.evaluate(context);
            debug!(
                "YawlWorkletService: Selected worklet {:?} for task {:?}",
                selected, task_id
            );
            return Ok(selected);
        }

        // No RDR tree, try exception-based selection
        if let Some(ref exception_type) = context.exception_type {
            let index = self.exception_index.read().await;
            if let Some(worklets) = index.get(exception_type) {
                if !worklets.is_empty() {
                    // Return first matching worklet (could be enhanced with priority)
                    return Ok(Some(worklets[0]));
                }
            }
        }

        Ok(None)
    }

    /// Register a worklet with RDR tree
    ///
    /// This adds a worklet to the repository and associates it with an RDR tree.
    pub async fn register_worklet(
        &self,
        worklet: Worklet,
        rdr_tree: Option<RdrTree>,
    ) -> WorkflowResult<WorkletId> {
        let worklet_id = worklet.metadata.id;

        // Add to repository
        self.repository.add_worklet(worklet).await?;

        // Add RDR tree if provided
        if let Some(mut tree) = rdr_tree {
            let task_id = tree.task_id.clone();
            let mut trees = self.rdr_trees.write().await;
            trees.insert(task_id, tree);
        }

        // Index by exception type (TRIZ Principle 10: Prior Action)
        let mut index = self.exception_index.write().await;
        for exception_type in &worklet.metadata.exception_types {
            index
                .entry(exception_type.clone())
                .or_insert_with(Vec::new)
                .push(worklet_id);
        }

        info!("YawlWorkletService: Registered worklet {}", worklet_id);
        Ok(worklet_id)
    }

    /// Execute a worklet as a sub-workflow
    ///
    /// This replaces the original task with worklet execution.
    ///
    /// # TRIZ Principle 1: Segmentation
    ///
    /// Worklet execution is separated from main engine execution.
    pub async fn execute_worklet(
        &self,
        worklet_id: WorkletId,
        context: &WorkletContext,
    ) -> WorkflowResult<WorkflowSpec> {
        // Get worklet from repository
        let worklet = self
            .repository
            .get_worklet(worklet_id)
            .await
            .ok_or_else(|| WorkflowError::WorkletNotFound(format!("Worklet {} not found", worklet_id)))?;

        // Return worklet's workflow spec for execution
        // Note: Execution is handled via dependency injection in WorkletExecutor::execute_worklet
        // This breaks circular dependency using TRIZ Principle 24: Intermediary
        Ok(worklet.workflow_spec)
    }

    /// Get RDR tree for a task
    pub async fn get_rdr_tree(&self, task_id: Option<&str>) -> Option<RdrTree> {
        let trees = self.rdr_trees.read().await;
        let key = task_id.map(|s| s.to_string());
        trees.get(&key).cloned()
    }

    /// Add RDR tree for a task
    pub async fn add_rdr_tree(&self, task_id: Option<String>, tree: RdrTree) {
        let mut trees = self.rdr_trees.write().await;
        trees.insert(task_id, tree);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Condition, Task, TaskType, WorkflowSpec, WorkflowSpecId};
    use crate::worklets::{WorkletMetadata, WorkletRule};

    fn create_test_worklet() -> Worklet {
        let spec = WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: "Test Worklet".to_string(),
            tasks: vec![],
            flows: vec![],
            conditions: vec![],
        };

        Worklet {
            metadata: WorkletMetadata {
                id: WorkletId::new(),
                name: "Test Worklet".to_string(),
                description: "Test".to_string(),
                version: "1.0".to_string(),
                exception_types: vec!["Timeout".to_string()],
                required_context: vec![],
                pattern_ids: vec![],
                tags: vec![],
            },
            workflow_spec: spec,
            rules: vec![],
        }
    }

    #[tokio::test]
    async fn test_worklet_selection() {
        let repository = Arc::new(WorkletRepository::new());
        let service = YawlWorkletService::new(repository);

        let worklet = create_test_worklet();
        let worklet_id = worklet.metadata.id;

        // Register worklet
        service.register_worklet(worklet, None).await.unwrap();

        // Create context with exception
        let context = WorkletContext {
            task_id: "task1".to_string(),
            data: serde_json::json!({}),
            exception_type: Some("Timeout".to_string()),
            case_id: "case1".to_string(),
        };

        // Select worklet by exception type
        let selected = service.select_worklet(None, &context).await.unwrap();
        assert_eq!(selected, Some(worklet_id));
    }

    #[tokio::test]
    async fn test_rdr_evaluation() {
        // Create RDR tree
        let mut tree = RdrTree::new(None);
        let root = RdrNode {
            node_id: 1,
            condition: "exception_type == \"Timeout\"".to_string(),
            worklet_id: Some(WorkletId::new()),
            true_child: None,
            false_child: None,
        };
        tree.set_root(root);

        let context = WorkletContext {
            task_id: "task1".to_string(),
            data: serde_json::json!({"exception_type": "Timeout"}),
            exception_type: Some("Timeout".to_string()),
            case_id: "case1".to_string(),
        };

        let selected = tree.evaluate(&context);
        assert!(selected.is_some());
    }
}

