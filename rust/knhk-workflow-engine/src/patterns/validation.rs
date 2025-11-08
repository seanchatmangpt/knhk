#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Pattern validation and composition for all 43 patterns

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternExecutor, PatternId,
};
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

        // Check for circular dependencies
        // FUTURE: Implement actual dependency graph validation

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
    pub async fn execute_composition(
        &self,
        name: &str,
        context: PatternExecutionContext,
        executor: &dyn PatternExecutor,
    ) -> WorkflowResult<PatternExecutionResult> {
        let pattern_ids = self.compositions.get(name).ok_or_else(|| {
            WorkflowError::Validation(format!("Composition '{}' not found", name))
        })?;

        // Execute patterns in sequence
        let mut last_result = PatternExecutionResult {
            success: true,
            next_state: None,
            variables: context.variables.clone(),
        };

        for _pattern_id in pattern_ids {
            // FUTURE: Get executor for pattern_id and execute
            // For now, use provided executor
            let mut ctx = context.clone();
            ctx.variables = last_result.variables.clone();
            last_result = executor.execute(&ctx);
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
        };

        let result = validator.validate_execution(&pattern_id, &context);
        assert!(result.valid);
    }
}
