#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Policy engine for RDF-based policy enforcement

use crate::error::{WorkflowError, WorkflowResult};
use oxigraph::store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// RDF policy definition (Turtle)
    pub rdf_policy: String,
    /// Enabled flag
    pub enabled: bool,
}

/// Policy decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    /// Allow
    Allow,
    /// Deny
    Deny,
    /// Not applicable
    NotApplicable,
}

/// Policy engine
pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
    rdf_store: Arc<Store>,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> WorkflowResult<Self> {
        let store = Store::new()
            .map_err(|e| WorkflowError::Internal(format!("Failed to create RDF store: {:?}", e)))?;

        Ok(Self {
            rules: Vec::new(),
            rdf_store: Arc::new(store),
        })
    }

    /// Add a policy rule
    pub fn add_rule(&mut self, rule: PolicyRule) -> WorkflowResult<()> {
        // FUTURE: Load RDF policy into store when oxigraph API is available
        // For now, policies are evaluated via rule.enabled flag
        self.rules.push(rule);
        Ok(())
    }

    /// Evaluate policy for a resource
    pub fn evaluate(
        &self,
        resource: &str,
        action: &str,
        context: &serde_json::Value,
    ) -> WorkflowResult<PolicyDecision> {
        // FUTURE: Implement RDF-based policy evaluation using SPARQL
        // For now, return Allow for all enabled rules
        for rule in &self.rules {
            if rule.enabled {
                // Check if rule applies
                // This would use SPARQL queries against the RDF store
                return Ok(PolicyDecision::Allow);
            }
        }

        Ok(PolicyDecision::NotApplicable)
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[PolicyRule] {
        &self.rules
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_engine() {
        let mut engine = PolicyEngine::new().unwrap();
        let rule = PolicyRule {
            name: "test-rule".to_string(),
            description: "Test rule".to_string(),
            rdf_policy: "".to_string(),
            enabled: true,
        };
        engine.add_rule(rule).unwrap();

        let decision = engine
            .evaluate("resource-1", "read", &serde_json::json!({}))
            .unwrap();
        assert_eq!(decision, PolicyDecision::Allow);
    }
}
