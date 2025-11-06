// rust/knhk-validation/src/advisor_example.rs
// Example usage of the Advisor pattern

#[cfg(feature = "advisor")]
pub mod examples {
    use super::super::advisor::*;
    use super::super::policy_engine::PolicyViolation;
    use alloc::collections::BTreeMap;
    
    /// Example: Using default advisor chain
    pub fn example_default_advisors() {
        let chain = AdvisorChain::with_default_advisors();
        
        let context = AdvisorContext {
            operation_id: "test_op_001".to_string(),
            runtime_class: "R1".to_string(),
            metadata: BTreeMap::new(),
        };
        
        // Check guard constraint
        let input = AdvisorInput {
            context: context.clone(),
            data: AdvisorData::GuardConstraint {
                run_len: 10, // Violation: exceeds max_run_len of 8
                max_run_len: 8,
            },
        };
        
        let violations = chain.advise(&input);
        assert!(!violations.is_empty());
        println!("Found {} violations", violations.len());
    }
    
    /// Example: Custom advisor chain
    pub fn example_custom_advisors() {
        let mut chain = AdvisorChain::new();
        
        // Add advisors in priority order
        chain.add_advisor(Box::new(GuardConstraintAdvisor::default()));
        chain.add_advisor(Box::new(PerformanceBudgetAdvisor::with_max_ticks(16))); // Custom max
        
        let context = AdvisorContext {
            operation_id: "custom_op".to_string(),
            runtime_class: "W1".to_string(),
            metadata: BTreeMap::new(),
        };
        
        // Check performance budget
        let input = AdvisorInput {
            context,
            data: AdvisorData::PerformanceBudget {
                ticks: 12,
                max_ticks: 16, // Within custom limit
            },
        };
        
        let violations = chain.advise(&input);
        println!("Found {} violations", violations.len());
    }
}

