#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Guard functions for input validation and policy enforcement

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use oxigraph::store::Store;
use serde_json::Value;
use std::sync::Arc;

/// Guard function result
#[derive(Debug, Clone)]
pub struct GuardResult {
    /// Whether guard passed
    pub allowed: bool,
    /// Reason for rejection (if not allowed)
    pub reason: Option<String>,
    /// Execution ticks consumed
    pub ticks: u32,
}

/// Guard function trait
pub trait GuardFunction: Send + Sync {
    /// Execute guard function
    fn execute(&self, input: &Value, context: &GuardContext) -> WorkflowResult<GuardResult>;

    /// Get guard name
    fn name(&self) -> &str;

    /// Get maximum execution ticks (must be ≤8 for hot path)
    fn max_ticks(&self) -> u32;
}

/// Guard execution context
#[derive(Debug, Clone)]
pub struct GuardContext {
    /// Workflow specification
    pub workflow_spec: Option<Arc<WorkflowSpec>>,
    /// RDF store for SHACL validation
    pub rdf_store: Option<Arc<Store>>,
    /// Additional context data
    pub metadata: Value,
}

/// Guard validator that executes multiple guards
pub struct GuardValidator {
    guards: Vec<Arc<dyn GuardFunction>>,
}

impl GuardValidator {
    /// Create a new guard validator
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    /// Add a guard function
    pub fn add_guard(&mut self, guard: Arc<dyn GuardFunction>) {
        self.guards.push(guard);
    }

    /// Validate input against all guards
    pub fn validate(&self, input: &Value, context: &GuardContext) -> WorkflowResult<GuardResult> {
        let mut total_ticks = 0;

        for guard in &self.guards {
            // Check tick budget
            if total_ticks + guard.max_ticks() > 8 {
                return Err(WorkflowError::Validation(format!(
                    "Guard execution would exceed 8-tick budget (current: {}, guard {}: {})",
                    total_ticks,
                    guard.name(),
                    guard.max_ticks()
                )));
            }

            let result = guard.execute(input, context)?;
            total_ticks += result.ticks;

            if !result.allowed {
                return Ok(GuardResult {
                    allowed: false,
                    reason: result.reason,
                    ticks: total_ticks,
                });
            }
        }

        Ok(GuardResult {
            allowed: true,
            reason: None,
            ticks: total_ticks,
        })
    }
}

impl Default for GuardValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Max run length guard (enforces max_run_len ≤ 8)
pub struct MaxRunLengthGuard {
    max_run_len: usize,
}

impl MaxRunLengthGuard {
    /// Create a new max run length guard
    pub fn new(max_run_len: usize) -> Self {
        Self { max_run_len }
    }
}

impl GuardFunction for MaxRunLengthGuard {
    fn execute(&self, input: &Value, _context: &GuardContext) -> WorkflowResult<GuardResult> {
        // Check if input is an array
        if let Value::Array(arr) = input {
            let len = arr.len();
            if len > self.max_run_len {
                return Ok(GuardResult {
                    allowed: false,
                    reason: Some(format!(
                        "Array length {} exceeds max_run_len {}",
                        len, self.max_run_len
                    )),
                    ticks: 1,
                });
            }
        }

        Ok(GuardResult {
            allowed: true,
            reason: None,
            ticks: 1,
        })
    }

    fn name(&self) -> &str {
        "MaxRunLengthGuard"
    }

    fn max_ticks(&self) -> u32 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_run_length_guard() {
        let guard = MaxRunLengthGuard::new(8);
        let context = GuardContext {
            workflow_spec: None,
            rdf_store: None,
            metadata: Value::Null,
        };

        // Valid input
        let input = Value::Array(vec![Value::Null; 8]);
        let result = guard.execute(&input, &context).unwrap();
        assert!(result.allowed);

        // Invalid input
        let input = Value::Array(vec![Value::Null; 9]);
        let result = guard.execute(&input, &context).unwrap();
        assert!(!result.allowed);
    }

    #[test]
    fn test_guard_validator() {
        let mut validator = GuardValidator::new();
        let guard = Arc::new(MaxRunLengthGuard::new(8));
        validator.add_guard(guard);

        let context = GuardContext {
            workflow_spec: None,
            rdf_store: None,
            metadata: Value::Null,
        };

        let input = Value::Array(vec![Value::Null; 8]);
        let result = validator.validate(&input, &context).unwrap();
        assert!(result.allowed);
    }
}
