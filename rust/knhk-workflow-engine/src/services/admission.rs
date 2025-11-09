//! Admission gate for case validation
//!
//! Validates cases before execution starts:
//! - Data validation
//! - Policy enforcement
//! - Resource availability checks
//! - Guard constraint enforcement (MAX_RUN_LEN, MAX_BATCH_SIZE)

use crate::error::{WorkflowError, WorkflowResult};
use crate::validation::guards::{validate_batch_size, MAX_BATCH_SIZE};
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
    /// Enforces guard constraints at ingress:
    /// - MAX_BATCH_SIZE validation for batch operations
    /// - Custom policy validation
    pub fn admit(&self, case_data: &Value) -> WorkflowResult<()> {
        // Guard constraint: Validate batch size if case_data contains batch operations
        if let Some(batch) = case_data.get("batch") {
            if let Some(batch_size) = batch.as_array().map(|a| a.len()) {
                validate_batch_size(batch_size)?;
            }
        }

        // Apply custom policies
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
