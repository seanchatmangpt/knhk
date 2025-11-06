// knhk-validation: Policy Engine
// Rego-based policy engine for guard constraints, performance validation, and receipt validation
// Inspired by Weaver's policy engine architecture

#![cfg(feature = "policy-engine")]

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Policy violation level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationLevel {
    /// Information - useful context without action needed
    Information,
    /// Improvement - suggested change that would improve things
    Improvement,
    /// Violation - something that breaks compliance rules
    Violation,
}

/// Policy violation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyViolation {
    /// Guard constraint violation (max_run_len > 8)
    GuardConstraintViolation {
        /// Violation ID
        id: String,
        /// Category
        category: String,
        /// Actual run length
        actual_run_len: u64,
        /// Maximum allowed run length
        max_run_len: u64,
        /// Context message
        message: String,
    },
    /// Performance budget violation (ticks > 8)
    PerformanceBudgetViolation {
        /// Violation ID
        id: String,
        /// Category
        category: String,
        /// Actual ticks
        actual_ticks: u32,
        /// Maximum allowed ticks
        max_ticks: u32,
        /// Context message
        message: String,
    },
    /// Receipt validation violation
    ReceiptValidationViolation {
        /// Violation ID
        id: String,
        /// Category
        category: String,
        /// Receipt ID
        receipt_id: String,
        /// Context message
        message: String,
    },
}

impl PolicyViolation {
    /// Get violation level
    pub fn level(&self) -> ViolationLevel {
        match self {
            PolicyViolation::GuardConstraintViolation { .. }
            | PolicyViolation::PerformanceBudgetViolation { .. }
            | PolicyViolation::ReceiptValidationViolation { .. } => ViolationLevel::Violation,
        }
    }

    /// Get violation ID
    pub fn id(&self) -> &str {
        match self {
            PolicyViolation::GuardConstraintViolation { id, .. }
            | PolicyViolation::PerformanceBudgetViolation { id, .. }
            | PolicyViolation::ReceiptValidationViolation { id, .. } => id,
        }
    }

    /// Get violation message
    pub fn message(&self) -> &str {
        match self {
            PolicyViolation::GuardConstraintViolation { message, .. }
            | PolicyViolation::PerformanceBudgetViolation { message, .. }
            | PolicyViolation::ReceiptValidationViolation { message, .. } => message,
        }
    }
}

/// Policy engine for validating guard constraints, performance budgets, and receipts
/// Inspired by Weaver's policy engine architecture
/// 
/// Supports both built-in policies and custom Rego policies (when rego feature enabled)
pub struct PolicyEngine {
    /// Built-in policies enabled
    builtin_policies: Vec<BuiltinPolicy>,
    /// Custom Rego policies (when rego feature enabled)
    #[cfg(feature = "rego")]
    rego_policies: Vec<RegoPolicy>,
}

/// Rego policy definition (when rego feature enabled)
#[cfg(feature = "rego")]
#[derive(Debug, Clone)]
pub struct RegoPolicy {
    /// Policy name
    pub name: String,
    /// Rego policy code
    pub code: String,
    /// Policy description
    pub description: String,
}

/// Built-in policy types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinPolicy {
    /// Guard constraint policy (max_run_len ≤ 8)
    GuardConstraint,
    /// Performance budget policy (ticks ≤ 8)
    PerformanceBudget,
    /// Receipt validation policy
    ReceiptValidation,
}

impl PolicyEngine {
    /// Create new policy engine with default built-in policies
    pub fn new() -> Self {
        Self {
            builtin_policies: vec![
                BuiltinPolicy::GuardConstraint,
                BuiltinPolicy::PerformanceBudget,
                BuiltinPolicy::ReceiptValidation,
            ],
            #[cfg(feature = "rego")]
            rego_policies: Vec::new(),
        }
    }

    /// Create policy engine with custom built-in policies
    pub fn with_policies(policies: Vec<BuiltinPolicy>) -> Self {
        Self {
            builtin_policies: policies,
            #[cfg(feature = "rego")]
            rego_policies: Vec::new(),
        }
    }

    /// Add custom Rego policy (requires rego feature)
    #[cfg(feature = "rego")]
    pub fn add_rego_policy(mut self, policy: RegoPolicy) -> Self {
        self.rego_policies.push(policy);
        self
    }

    /// Load Rego policies from directory (requires rego feature)
    #[cfg(feature = "rego")]
    pub fn load_rego_policies(&mut self, _policy_dir: &str) -> Result<(), String> {
        // TODO: Implement Rego policy loading
        // This would:
        // 1. Scan directory for .rego files
        // 2. Parse and validate Rego policies
        // 3. Add to rego_policies vector
        // For now, this is a placeholder for future implementation
        Ok(())
    }

    /// Evaluate all policies (built-in + Rego) and return violations
    pub fn evaluate_all(&self, context: &PolicyContext) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();

        // Evaluate built-in policies
        for policy in &self.builtin_policies {
            match policy {
                BuiltinPolicy::GuardConstraint => {
                    if let Some(run_len) = context.run_len {
                        if let Err(violation) = self.validate_guard_constraint(run_len) {
                            violations.push(violation);
                        }
                    }
                }
                BuiltinPolicy::PerformanceBudget => {
                    if let Some(ticks) = context.ticks {
                        if let Err(violation) = self.validate_performance_budget(ticks) {
                            violations.push(violation);
                        }
                    }
                }
                BuiltinPolicy::ReceiptValidation => {
                    if let Some((receipt_id, expected_hash, actual_hash)) = &context.receipt {
                        if let Err(violation) = self.validate_receipt(
                            receipt_id,
                            expected_hash,
                            actual_hash,
                        ) {
                            violations.push(violation);
                        }
                    }
                }
            }
        }

        // Evaluate Rego policies (when rego feature enabled)
        #[cfg(feature = "rego")]
        {
            for rego_policy in &self.rego_policies {
                // TODO: Evaluate Rego policy
                // This would use a Rego evaluation engine (e.g., opa-wasm)
            }
        }

        violations
    }
}

/// Policy evaluation context
#[derive(Debug, Clone, Default)]
pub struct PolicyContext {
    /// Run length for guard constraint validation
    pub run_len: Option<u64>,
    /// Ticks for performance budget validation
    pub ticks: Option<u32>,
    /// Receipt validation data (receipt_id, expected_hash, actual_hash)
    pub receipt: Option<(String, Vec<u8>, Vec<u8>)>,
    /// Additional context for custom policies
    pub additional: alloc::collections::BTreeMap<String, String>,
}

impl PolicyEngine {
    /// Validate guard constraint (max_run_len ≤ 8)
    pub fn validate_guard_constraint(&self, run_len: u64) -> Result<(), PolicyViolation> {
        if !self.builtin_policies.contains(&BuiltinPolicy::GuardConstraint) {
            return Ok(());
        }

        const MAX_RUN_LEN: u64 = 8;
        if run_len > MAX_RUN_LEN {
            Err(PolicyViolation::GuardConstraintViolation {
                id: "guard_constraint_violation".to_string(),
                category: "guard_constraint".to_string(),
                actual_run_len: run_len,
                max_run_len: MAX_RUN_LEN,
                message: format!(
                    "Guard constraint violated: run_len {} exceeds maximum {} (Chatman Constant)",
                    run_len, MAX_RUN_LEN
                ),
            })
        } else {
            Ok(())
        }
    }

    /// Validate performance budget (ticks ≤ 8)
    pub fn validate_performance_budget(&self, ticks: u32) -> Result<(), PolicyViolation> {
        if !self.builtin_policies.contains(&BuiltinPolicy::PerformanceBudget) {
            return Ok(());
        }

        const MAX_TICKS: u32 = 8;
        if ticks > MAX_TICKS {
            Err(PolicyViolation::PerformanceBudgetViolation {
                id: "performance_budget_violation".to_string(),
                category: "performance_budget".to_string(),
                actual_ticks: ticks,
                max_ticks: MAX_TICKS,
                message: format!(
                    "Performance budget violated: {} ticks exceeds maximum {} ticks (Chatman Constant)",
                    ticks, MAX_TICKS
                ),
            })
        } else {
            Ok(())
        }
    }

    /// Validate receipt
    pub fn validate_receipt(
        &self,
        receipt_id: &str,
        receipt_hash: &[u8],
        expected_hash: &[u8],
    ) -> Result<(), PolicyViolation> {
        if !self.builtin_policies.contains(&BuiltinPolicy::ReceiptValidation) {
            return Ok(());
        }

        if receipt_hash != expected_hash {
            Err(PolicyViolation::ReceiptValidationViolation {
                id: "receipt_validation_violation".to_string(),
                category: "receipt_validation".to_string(),
                receipt_id: receipt_id.to_string(),
                message: format!(
                    "Receipt validation failed: hash mismatch for receipt {}",
                    receipt_id
                ),
            })
        } else {
            Ok(())
        }
    }

    /// Check all policies and return violations
    pub fn check_all(
        &self,
        run_len: Option<u64>,
        ticks: Option<u32>,
        receipt: Option<(&str, &[u8], &[u8])>,
    ) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();

        if let Some(len) = run_len {
            if let Err(v) = self.validate_guard_constraint(len) {
                violations.push(v);
            }
        }

        if let Some(t) = ticks {
            if let Err(v) = self.validate_performance_budget(t) {
                violations.push(v);
            }
        }

        if let Some((id, hash, expected)) = receipt {
            if let Err(v) = self.validate_receipt(id, hash, expected) {
                violations.push(v);
            }
        }

        violations
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_constraint_valid() {
        let engine = PolicyEngine::new();
        assert!(engine.validate_guard_constraint(8).is_ok());
        assert!(engine.validate_guard_constraint(7).is_ok());
        assert!(engine.validate_guard_constraint(0).is_ok());
    }

    #[test]
    fn test_guard_constraint_violation() {
        let engine = PolicyEngine::new();
        let result = engine.validate_guard_constraint(9);
        assert!(result.is_err());
        if let Err(PolicyViolation::GuardConstraintViolation {
            actual_run_len,
            max_run_len,
            ..
        }) = result
        {
            assert_eq!(actual_run_len, 9);
            assert_eq!(max_run_len, 8);
        } else {
            panic!("Expected GuardConstraintViolation");
        }
    }

    #[test]
    fn test_performance_budget_valid() {
        let engine = PolicyEngine::new();
        assert!(engine.validate_performance_budget(8).is_ok());
        assert!(engine.validate_performance_budget(7).is_ok());
        assert!(engine.validate_performance_budget(0).is_ok());
    }

    #[test]
    fn test_performance_budget_violation() {
        let engine = PolicyEngine::new();
        let result = engine.validate_performance_budget(9);
        assert!(result.is_err());
        if let Err(PolicyViolation::PerformanceBudgetViolation {
            actual_ticks,
            max_ticks,
            ..
        }) = result
        {
            assert_eq!(actual_ticks, 9);
            assert_eq!(max_ticks, 8);
        } else {
            panic!("Expected PerformanceBudgetViolation");
        }
    }

    #[test]
    fn test_receipt_validation_valid() {
        let engine = PolicyEngine::new();
        let hash = b"test_hash";
        assert!(engine.validate_receipt("receipt-1", hash, hash).is_ok());
    }

    #[test]
    fn test_receipt_validation_violation() {
        let engine = PolicyEngine::new();
        let hash1 = b"test_hash_1";
        let hash2 = b"test_hash_2";
        let result = engine.validate_receipt("receipt-1", hash1, hash2);
        assert!(result.is_err());
        if let Err(PolicyViolation::ReceiptValidationViolation { receipt_id, .. }) = result {
            assert_eq!(receipt_id, "receipt-1");
        } else {
            panic!("Expected ReceiptValidationViolation");
        }
    }

    #[test]
    fn test_check_all() {
        let engine = PolicyEngine::new();
        let violations = engine.check_all(Some(9), Some(10), None);
        assert_eq!(violations.len(), 2);
    }
}

