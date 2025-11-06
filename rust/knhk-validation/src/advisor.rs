// rust/knhk-validation/src/advisor.rs
// Advisor Pattern - Inspired by Weaver's advisor architecture
// Pluggable advisors that provide validation advice

use alloc::string::String;
use alloc::vec::Vec;
#[cfg(feature = "policy-engine")]
use crate::policy_engine::{PolicyViolation, ViolationLevel};

/// Advisor trait for pluggable validation advice
/// Inspired by Weaver's Advisor pattern
pub trait Advisor: Send + Sync {
    /// Provide advice/validation on input
    fn advise(&self, input: &AdvisorInput) -> Vec<PolicyViolation>;
    
    /// Get advisor name
    fn name(&self) -> &str;
    
    /// Get advisor priority (lower = higher priority)
    fn priority(&self) -> u32 {
        100
    }
}

/// Input for advisor evaluation
pub struct AdvisorInput {
    /// Context data
    pub context: AdvisorContext,
    /// Data to validate
    pub data: AdvisorData,
}

/// Context for advisor evaluation
pub struct AdvisorContext {
    /// Operation ID
    pub operation_id: String,
    /// Runtime class
    pub runtime_class: String,
    /// Additional context
    pub metadata: alloc::collections::BTreeMap<String, String>,
}

/// Data to validate
pub enum AdvisorData {
    /// Guard constraint data (run length)
    GuardConstraint { run_len: u64, max_run_len: u64 },
    /// Performance budget data (ticks)
    PerformanceBudget { ticks: u32, max_ticks: u32 },
    /// Receipt validation data
    Receipt { receipt_id: String, hash: String },
    /// Schema validation data
    Schema { schema_id: String, data: Vec<u8> },
}

/// Guard constraint advisor
/// Validates max_run_len ≤ 8 (Chatman Constant)
pub struct GuardConstraintAdvisor {
    max_run_len: u64,
}

impl GuardConstraintAdvisor {
    pub fn new() -> Self {
        Self {
            max_run_len: 8, // Chatman Constant
        }
    }
    
    pub fn with_max_run_len(mut self, max_run_len: u64) -> Self {
        self.max_run_len = max_run_len;
        self
    }
}

impl Advisor for GuardConstraintAdvisor {
    fn advise(&self, input: &AdvisorInput) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();
        
        if let AdvisorData::GuardConstraint { run_len, max_run_len } = &input.data {
            if *run_len > self.max_run_len {
                violations.push(PolicyViolation::GuardConstraintViolation {
                    id: format!("guard_constraint_{}", input.context.operation_id),
                    category: "guard_constraint".to_string(),
                    actual_run_len: *run_len,
                    max_run_len: self.max_run_len,
                    message: format!(
                        "Guard constraint violation: run_len {} exceeds max_run_len {}",
                        run_len, self.max_run_len
                    ),
                });
            }
        }
        
        violations
    }
    
    fn name(&self) -> &str {
        "guard_constraint"
    }
    
    fn priority(&self) -> u32 {
        10 // High priority
    }
}

impl Default for GuardConstraintAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance budget advisor
/// Validates hot path operations ≤ 8 ticks
pub struct PerformanceBudgetAdvisor {
    max_ticks: u32,
}

impl PerformanceBudgetAdvisor {
    pub fn new() -> Self {
        Self {
            max_ticks: 8, // Chatman Constant: 2ns = 8 ticks
        }
    }
    
    pub fn with_max_ticks(mut self, max_ticks: u32) -> Self {
        self.max_ticks = max_ticks;
        self
    }
}

impl Advisor for PerformanceBudgetAdvisor {
    fn advise(&self, input: &AdvisorInput) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();
        
        if let AdvisorData::PerformanceBudget { ticks, max_ticks } = &input.data {
            if *ticks > self.max_ticks {
                violations.push(PolicyViolation::PerformanceBudgetViolation {
                    id: format!("performance_budget_{}", input.context.operation_id),
                    category: "performance_budget".to_string(),
                    actual_ticks: *ticks,
                    max_ticks: self.max_ticks,
                    message: format!(
                        "Performance budget violation: ticks {} exceeds max_ticks {}",
                        ticks, self.max_ticks
                    ),
                });
            }
        }
        
        violations
    }
    
    fn name(&self) -> &str {
        "performance_budget"
    }
    
    fn priority(&self) -> u32 {
        20 // High priority
    }
}

impl Default for PerformanceBudgetAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

/// Receipt validation advisor
/// Validates receipt hash integrity
pub struct ReceiptValidationAdvisor;

impl ReceiptValidationAdvisor {
    pub fn new() -> Self {
        Self
    }
}

impl Advisor for ReceiptValidationAdvisor {
    fn advise(&self, input: &AdvisorInput) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();
        
        if let AdvisorData::Receipt { receipt_id, hash } = &input.data {
            // Validate receipt hash format and integrity
            // In production, this would verify hash(A) = hash(μ(O))
            if hash.is_empty() {
                violations.push(PolicyViolation::ReceiptValidationViolation {
                    id: format!("receipt_validation_{}", receipt_id),
                    category: "receipt_validation".to_string(),
                    receipt_id: receipt_id.clone(),
                    message: "Receipt hash is empty".to_string(),
                });
            }
        }
        
        violations
    }
    
    fn name(&self) -> &str {
        "receipt_validation"
    }
    
    fn priority(&self) -> u32 {
        30 // Medium-high priority
    }
}

impl Default for ReceiptValidationAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

/// Advisor chain for running multiple advisors
pub struct AdvisorChain {
    advisors: Vec<Box<dyn Advisor>>,
}

impl AdvisorChain {
    pub fn new() -> Self {
        Self {
            advisors: Vec::new(),
        }
    }
    
    pub fn add_advisor(&mut self, advisor: Box<dyn Advisor>) {
        self.advisors.push(advisor);
        // Sort by priority
        self.advisors.sort_by_key(|a| a.priority());
    }
    
    pub fn advise(&self, input: &AdvisorInput) -> Vec<PolicyViolation> {
        let mut all_violations = Vec::new();
        
        for advisor in &self.advisors {
            let violations = advisor.advise(input);
            all_violations.extend(violations);
        }
        
        all_violations
    }
    
    pub fn with_default_advisors() -> Self {
        let mut chain = Self::new();
        chain.add_advisor(Box::new(GuardConstraintAdvisor::default()));
        chain.add_advisor(Box::new(PerformanceBudgetAdvisor::default()));
        chain.add_advisor(Box::new(ReceiptValidationAdvisor::default()));
        chain
    }
}

impl Default for AdvisorChain {
    fn default() -> Self {
        Self::with_default_advisors()
    }
}

