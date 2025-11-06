// rust/knhk-validation/src/policy.rs
// Policy Advisor Pattern (inspired by Weaver)
// Pluggable advisors that provide advice/validation using Rego policies

#![cfg(feature = "policy-engine")]

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[cfg(feature = "policy-engine")]
use regorus::{Engine, Value};

#[cfg(feature = "policy-engine")]
use crate::error::PolicyError;

/// Policy advisor trait - provides validation advice
pub trait PolicyAdvisor: Send + Sync {
    /// Evaluate policy against input data
    fn evaluate(&self, input: &PolicyInput) -> Result<PolicyResult, PolicyError>;
    
    /// Get advisor name
    fn name(&self) -> &str;
    
    /// Get policy source
    fn policy_source(&self) -> &str;
}

/// Policy input data structure
#[derive(Debug, Clone)]
pub struct PolicyInput {
    pub data: BTreeMap<String, Value>,
}

impl PolicyInput {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }
    
    pub fn with_field(mut self, key: &str, value: Value) -> Self {
        self.data.insert(key.to_string(), value);
        self
    }
    
    pub fn with_run_len(mut self, len: usize) -> Self {
        self.data.insert("run_len".to_string(), Value::Number(len as f64));
        self
    }
    
    pub fn with_ticks(mut self, ticks: u32) -> Self {
        self.data.insert("ticks".to_string(), Value::Number(ticks as f64));
        self
    }
    
    pub fn with_runtime_class(mut self, class: &str) -> Self {
        self.data.insert("runtime_class".to_string(), Value::String(class.to_string()));
        self
    }
    
    pub fn with_latency_ns(mut self, latency: u64) -> Self {
        self.data.insert("latency_ns".to_string(), Value::Number(latency as f64));
        self
    }
    
    pub fn with_receipt(mut self, receipt_id: &str, receipt_hash: &[u8], ticks: u32, timestamp_ms: u64) -> Self {
        self.data.insert("receipt_id".to_string(), Value::String(receipt_id.to_string()));
        self.data.insert("receipt_hash".to_string(), Value::Array(
            receipt_hash.iter().map(|b| Value::Number(*b as f64)).collect()
        ));
        self.data.insert("ticks".to_string(), Value::Number(ticks as f64));
        self.data.insert("timestamp_ms".to_string(), Value::Number(timestamp_ms as f64));
        self
    }
}

impl Default for PolicyInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Policy evaluation result
#[derive(Debug, Clone)]
pub struct PolicyResult {
    pub valid: bool,
    pub violations: Vec<String>,
    pub advice: Vec<String>,
}

impl PolicyResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            violations: Vec::new(),
            advice: Vec::new(),
        }
    }
    
    pub fn with_violation(mut self, msg: String) -> Self {
        self.valid = false;
        self.violations.push(msg);
        self
    }
    
    pub fn with_advice(mut self, msg: String) -> Self {
        self.advice.push(msg);
        self
    }
}

/// Guard constraint advisor - validates max_run_len ≤ 8
pub struct GuardConstraintAdvisor {
    engine: Engine,
}

impl GuardConstraintAdvisor {
    pub fn new() -> Result<Self, PolicyError> {
        let mut engine = Engine::new();
        
        // Load guard constraints policy
        let policy = include_str!("../policies/guard_constraints.rego");
        engine.add_policy(
            "guard_constraints.rego",
            policy,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        Ok(Self { engine })
    }
}

impl PolicyAdvisor for GuardConstraintAdvisor {
    fn evaluate(&self, input: &PolicyInput) -> Result<PolicyResult, PolicyError> {
        // Convert input to regorus Value
        let input_value = Value::Object(
            input.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        );
        
        // Evaluate policy
        let result = self.engine.eval_query(
            "data.knhk.guard_constraints.violation",
            &input_value,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        let mut policy_result = PolicyResult::new();
        
        // Check for violations
        if let Value::Array(violations) = result {
            for violation in violations {
                if let Value::String(msg) = violation {
                    policy_result = policy_result.with_violation(msg);
                }
            }
        }
        
        // Check if valid
        let valid_result = self.engine.eval_query(
            "data.knhk.guard_constraints.valid",
            &input_value,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        if let Value::Bool(valid) = valid_result {
            if !valid && policy_result.violations.is_empty() {
                policy_result.valid = false;
                policy_result.violations.push("Run length validation failed".to_string());
            }
        }
        
        Ok(policy_result)
    }
    
    fn name(&self) -> &str {
        "GuardConstraintAdvisor"
    }
    
    fn policy_source(&self) -> &str {
        "policies/guard_constraints.rego"
    }
}

/// Performance budget advisor - validates 8-tick budget and SLOs
pub struct PerformanceBudgetAdvisor {
    engine: Engine,
}

impl PerformanceBudgetAdvisor {
    pub fn new() -> Result<Self, PolicyError> {
        let mut engine = Engine::new();
        
        // Load performance budget policy
        let policy = include_str!("../policies/performance_budget.rego");
        engine.add_policy(
            "performance_budget.rego",
            policy,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        Ok(Self { engine })
    }
}

impl PolicyAdvisor for PerformanceBudgetAdvisor {
    fn evaluate(&self, input: &PolicyInput) -> Result<PolicyResult, PolicyError> {
        // Convert input to regorus Value
        let input_value = Value::Object(
            input.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        );
        
        let mut policy_result = PolicyResult::new();
        
        // Check for tick budget violations
        let tick_result = self.engine.eval_query(
            "data.knhk.performance_budget.violation",
            &input_value,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        if let Value::Array(violations) = tick_result {
            for violation in violations {
                if let Value::String(msg) = violation {
                    policy_result = policy_result.with_violation(msg);
                }
            }
        }
        
        // Check for SLO violations
        let slo_result = self.engine.eval_query(
            "data.knhk.performance_budget.slo_violation",
            &input_value,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        if let Value::Array(violations) = slo_result {
            for violation in violations {
                if let Value::String(msg) = violation {
                    policy_result = policy_result.with_violation(msg);
                }
            }
        }
        
        // Check if within budget
        let budget_result = self.engine.eval_query(
            "data.knhk.performance_budget.within_budget",
            &input_value,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        if let Value::Bool(within) = budget_result {
            if !within && policy_result.violations.is_empty() {
                policy_result.valid = false;
                policy_result.violations.push("Performance budget exceeded".to_string());
            }
        }
        
        Ok(policy_result)
    }
    
    fn name(&self) -> &str {
        "PerformanceBudgetAdvisor"
    }
    
    fn policy_source(&self) -> &str {
        "policies/performance_budget.rego"
    }
}

/// Receipt validation advisor - validates receipt structure and hash
pub struct ReceiptValidationAdvisor {
    engine: Engine,
}

impl ReceiptValidationAdvisor {
    pub fn new() -> Result<Self, PolicyError> {
        let mut engine = Engine::new();
        
        // Load receipt validation policy
        let policy = include_str!("../policies/receipt_validation.rego");
        engine.add_policy(
            "receipt_validation.rego",
            policy,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        Ok(Self { engine })
    }
}

impl PolicyAdvisor for ReceiptValidationAdvisor {
    fn evaluate(&self, input: &PolicyInput) -> Result<PolicyResult, PolicyError> {
        // Convert input to regorus Value
        let input_value = Value::Object(
            input.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        );
        
        // Evaluate policy
        let result = self.engine.eval_query(
            "data.knhk.receipt_validation.violation",
            &input_value,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        let mut policy_result = PolicyResult::new();
        
        // Check for violations
        if let Value::Array(violations) = result {
            for violation in violations {
                if let Value::String(msg) = violation {
                    policy_result = policy_result.with_violation(msg);
                }
            }
        }
        
        // Check if valid
        let valid_result = self.engine.eval_query(
            "data.knhk.receipt_validation.valid",
            &input_value,
        ).map_err(|e| PolicyError::RegoEngineError(e.to_string()))?;
        
        if let Value::Bool(valid) = valid_result {
            if !valid && policy_result.violations.is_empty() {
                policy_result.valid = false;
                policy_result.violations.push("Receipt validation failed".to_string());
            }
        }
        
        Ok(policy_result)
    }
    
    fn name(&self) -> &str {
        "ReceiptValidationAdvisor"
    }
    
    fn policy_source(&self) -> &str {
        "policies/receipt_validation.rego"
    }
}

/// Policy advisor chain - evaluates multiple advisors
pub struct PolicyAdvisorChain {
    advisors: Vec<Box<dyn PolicyAdvisor>>,
}

impl PolicyAdvisorChain {
    pub fn new() -> Self {
        Self {
            advisors: Vec::new(),
        }
    }
    
    pub fn with_advisor(mut self, advisor: Box<dyn PolicyAdvisor>) -> Self {
        self.advisors.push(advisor);
        self
    }
    
    pub fn evaluate_all(&self, input: &PolicyInput) -> Vec<Result<PolicyResult, PolicyError>> {
        self.advisors.iter().map(|advisor| advisor.evaluate(input)).collect()
    }
    
    pub fn evaluate_any_violation(&self, input: &PolicyInput) -> Result<PolicyResult, PolicyError> {
        let mut combined_result = PolicyResult::new();
        
        for advisor in &self.advisors {
            match advisor.evaluate(input) {
                Ok(result) => {
                    if !result.valid {
                        combined_result.valid = false;
                        combined_result.violations.extend(result.violations);
                        combined_result.advice.extend(result.advice);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        
        Ok(combined_result)
    }
}

impl Default for PolicyAdvisorChain {
    fn default() -> Self {
        Self::new()
    }
}

