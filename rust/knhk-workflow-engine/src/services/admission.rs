//! Admission gate for case validation
//!
//! Validates cases before execution starts:
//! - Data validation
//! - Policy enforcement
//! - Resource availability checks

use crate::error::{WorkflowError, WorkflowResult};
use serde_json::Value;

/// Admission gate for case validation
pub struct AdmissionGate {
    /// Validation policies
    policies: Vec<Box<dyn Fn(&Value) -> Result<(), String> + Send + Sync>>,
}

impl AdmissionGate {
    /// Create a new admission gate
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    /// Add a validation policy
    pub fn add_policy<F>(&mut self, policy: F)
    where
        F: Fn(&Value) -> Result<(), String> + Send + Sync + 'static,
    {
        self.policies.push(Box::new(policy));
    }

    /// Admit a case (validate before execution)
    pub fn admit(&self, case_data: &Value) -> WorkflowResult<()> {
        for policy in &self.policies {
            policy(case_data).map_err(|e| {
                WorkflowError::Validation(format!("Admission policy failed: {}", e))
            })?;
        }
        Ok(())
    }
}

impl Default for AdmissionGate {
    fn default() -> Self {
        Self::new()
    }
}

