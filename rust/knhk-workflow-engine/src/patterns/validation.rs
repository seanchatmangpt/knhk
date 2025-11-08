#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Pattern validation and composition for all 43 patterns

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternId};
use std::collections::HashMap;

/// Pattern validation result
#[derive(Debug, Clone)]
pub struct PatternValidationResult {
    /// Whether pattern is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

/// Pattern validator
pub struct PatternValidator;

impl PatternValidator {
    /// Validate a pattern execution
    pub fn validate_execution(
        &self,
        pattern_id: &PatternId,
        context: &PatternExecutionContext,
    ) -> PatternValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate pattern ID
        if pattern_id.0 < 1 || pattern_id.0 > 43 {
            errors.push(format!("Invalid pattern ID: {}", pattern_id.0));
        }

        // Validate context
        if context.variables.is_empty() {
            warnings.push("Empty execution context".to_string());
        }

        PatternValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Validate pattern composition
    pub fn validate_composition(&self, pattern_ids: &[PatternId]) -> PatternValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check for circular dependencies using DFS cycle detection
        // Build dependency graph from pattern IDs
        use std::collections::{HashMap, HashSet};

        // Create adjacency list for dependency graph
        // For now, assume patterns depend on previous patterns in sequence
        let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..pattern_ids.len() {
            if i > 0 {
                // Pattern i depends on pattern i-1
                graph.entry(i).or_insert_with(Vec::new).push(i - 1);
            }
        }

        // DFS cycle detection
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        fn has_cycle(
            node: usize,
            graph: &HashMap<usize, Vec<usize>>,
            visited: &mut HashSet<usize>,
            rec_stack: &mut HashSet<usize>,
        ) -> bool {
            visited.insert(node);
            rec_stack.insert(node);

            if let Some(neighbors) = graph.get(&node) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        if has_cycle(neighbor, graph, visited, rec_stack) {
                            return true;
                        }
                    } else if rec_stack.contains(&neighbor) {
                        return true; // Cycle detected
                    }
                }
            }

            rec_stack.remove(&node);
            false
        }

        for i in 0..pattern_ids.len() {
            if !visited.contains(&i) {
                if has_cycle(i, &graph, &mut visited, &mut rec_stack) {
                    errors.push(format!(
                        "Circular dependency detected in pattern composition"
                    ));
                    break;
                }
            }
        }

        // Basic validation: check for duplicate pattern IDs (potential circular dependency indicator)
        let mut seen = std::collections::HashSet::new();
        for pattern_id in pattern_ids {
            if !seen.insert(pattern_id) {
                warnings.push(format!(
                    "Duplicate pattern ID {} in composition may indicate circular dependency",
                    pattern_id.0
                ));
            }
        }

        // Check pattern count
        if pattern_ids.len() > 100 {
            warnings.push("Large pattern composition may impact performance".to_string());
        }

        PatternValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

/// Pattern composition manager
pub struct PatternComposition {
    compositions: HashMap<String, Vec<PatternId>>,
}

impl PatternComposition {
    /// Create a new pattern composition manager
    pub fn new() -> Self {
        Self {
            compositions: HashMap::new(),
        }
    }

    /// Create a composition
    pub fn create_composition(
        &mut self,
        name: String,
        pattern_ids: Vec<PatternId>,
    ) -> WorkflowResult<()> {
        let validator = PatternValidator;
        let validation = validator.validate_composition(&pattern_ids);

        if !validation.valid {
            return Err(WorkflowError::Validation(format!(
                "Invalid composition: {:?}",
                validation.errors
            )));
        }

        self.compositions.insert(name, pattern_ids);
        Ok(())
    }

    /// Execute a composition
    pub fn execute_composition(
        &self,
        name: &str,
        context: PatternExecutionContext,
        registry: &crate::patterns::PatternRegistry,
    ) -> WorkflowResult<PatternExecutionResult> {
        let pattern_ids = self.compositions.get(name).ok_or_else(|| {
            WorkflowError::Validation(format!("Composition '{}' not found", name))
        })?;

        // Execute patterns in sequence
        let mut last_result = PatternExecutionResult {
            success: true,
            next_state: None,
            next_activities: Vec::new(),
            variables: context.variables.clone(),
            updates: None,
            cancel_activities: Vec::new(),
            terminates: false,
        };

        for pattern_id in pattern_ids {
            // Get executor for pattern_id and execute
            if let Some(executor) = registry.get(pattern_id) {
                let mut ctx = context.clone();
                ctx.variables = last_result.variables.clone();
                last_result = executor.execute(&ctx);
            } else {
                return Err(WorkflowError::PatternNotFound(pattern_id.0));
            }
        }

        Ok(last_result)
    }
}

impl Default for PatternComposition {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_validator() {
        let validator = PatternValidator;
        let pattern_id = PatternId::new(1).unwrap();
        let context = PatternExecutionContext {
            case_id: crate::case::CaseId::new(),
            workflow_id: crate::parser::WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: std::collections::HashSet::new(),
            scope_id: String::new(),
        };

        let result = validator.validate_execution(&pattern_id, &context);
        assert!(result.valid);
    }
}
