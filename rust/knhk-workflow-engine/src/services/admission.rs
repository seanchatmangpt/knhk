//! Admission gate for case validation
//!
//! Validates cases before execution starts:
//! - Data validation
//! - Policy enforcement
//! - Resource availability checks
//! - Guard constraint enforcement (MAX_RUN_LEN, MAX_BATCH_SIZE)
//!
//! **CRITICAL**: This is the ONLY place where guard constraints are enforced.
//! Execution paths (hot path, executor, state) assume inputs are pre-validated here.

use crate::error::{WorkflowError, WorkflowResult};
use crate::validation::guards::{validate_batch_size, validate_run_len, MAX_BATCH_SIZE, MAX_RUN_LEN};
use serde_json::Value;

/// Admission gate for case validation
pub struct AdmissionGate {
    /// Validation policies
    policies: Vec<Box<dyn Fn(&Value) -> Result<(), String> + Send + Sync>>,
}

impl AdmissionGate {
    /// Create a new admission gate with default guard constraints
    ///
    /// Default policies enforce:
    /// - MAX_RUN_LEN ≤ 8 (Chatman Constant)
    /// - MAX_BATCH_SIZE validation
    pub fn new() -> Self {
        let mut gate = Self {
            policies: Vec::new(),
        };
        
        // Add default MAX_RUN_LEN guard policy
        gate.add_max_run_len_policy();
        
        gate
    }

    /// Add a validation policy
    pub fn add_policy<F>(&mut self, policy: F)
    where
        F: Fn(&Value) -> Result<(), String> + Send + Sync + 'static,
    {
        self.policies.push(Box::new(policy));
    }

    /// Add default MAX_RUN_LEN guard policy
    ///
    /// Enforces MAX_RUN_LEN ≤ 8 constraint on case data.
    /// Validates:
    /// - `triples` array length ≤ 8
    /// - `runs` array length ≤ 8
    /// - Any array that represents a "run" of operations
    fn add_max_run_len_policy(&mut self) {
        self.add_policy(|case_data: &Value| -> Result<(), String> {
            // Check triples array (most common case)
            if let Some(triples) = case_data.get("triples") {
                if let Some(triples_array) = triples.as_array() {
                    let len = triples_array.len();
                    if len > MAX_RUN_LEN {
                        return Err(format!(
                            "Triples array length {} exceeds MAX_RUN_LEN {} (Chatman Constant violation)",
                            len, MAX_RUN_LEN
                        ));
                    }
                }
            }

            // Check runs array
            if let Some(runs) = case_data.get("runs") {
                if let Some(runs_array) = runs.as_array() {
                    let len = runs_array.len();
                    if len > MAX_RUN_LEN {
                        return Err(format!(
                            "Runs array length {} exceeds MAX_RUN_LEN {} (Chatman Constant violation)",
                            len, MAX_RUN_LEN
                        ));
                    }
                }
            }

            // Check for any array that might represent a run
            // Look for arrays with length > 8 that could be operations
            if let Some(obj) = case_data.as_object() {
                for (key, value) in obj {
                    // Skip known non-run arrays
                    if key == "batch" || key == "metadata" {
                        continue;
                    }
                    
                    if let Some(arr) = value.as_array() {
                        let len = arr.len();
                        // If array has >8 items and looks like operations/triples, validate
                        if len > MAX_RUN_LEN {
                            // Check if array items look like triples/operations
                            if len > 0 {
                                let first_item = &arr[0];
                                // If first item is an object (likely triple/operation), validate
                                if first_item.is_object() {
                                    return Err(format!(
                                        "Array '{}' length {} exceeds MAX_RUN_LEN {} (Chatman Constant violation)",
                                        key, len, MAX_RUN_LEN
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            Ok(())
        });
    }

    /// Admit a case (validate before execution)
    /// Enforces guard constraints at ingress:
    /// - MAX_RUN_LEN ≤ 8 (Chatman Constant) - enforced by default policy
    /// - MAX_BATCH_SIZE validation for batch operations
    /// - Custom policy validation
    ///
    /// **CRITICAL**: This is the ONLY place where guard constraints are enforced.
    /// Execution paths assume inputs are pre-validated here.
    pub fn admit(&self, case_data: &Value) -> WorkflowResult<()> {
        // Guard constraint: Validate batch size if case_data contains batch operations
        if let Some(batch) = case_data.get("batch") {
            if let Some(batch_size) = batch.as_array().map(|a| a.len()) {
                validate_batch_size(batch_size)?;
            }
        }

        // Apply custom policies (includes default MAX_RUN_LEN policy)
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
