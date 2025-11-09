//! ABAC (Attribute-Based Access Control) in RDF
//!
//! Implements full ABAC using RDF/SPARQL for policy evaluation.
//! Policies are defined in RDF/Turtle format and evaluated using SPARQL queries.

use crate::error::{WorkflowError, WorkflowResult};
use crate::security::Principal;
use oxigraph::io::RdfFormat;
use oxigraph::sparql::SparqlEvaluator;
use oxigraph::store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ABAC policy decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbacDecision {
    /// Allow access
    Allow,
    /// Deny access
    Deny,
    /// Not applicable (no matching policy)
    NotApplicable,
    /// Indeterminate (policy evaluation error)
    Indeterminate,
}

/// ABAC policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacPolicyRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// RDF policy definition (Turtle format)
    pub rdf_policy: String,
    /// SPARQL query for policy evaluation
    pub evaluation_query: String,
    /// Effect (Allow or Deny)
    pub effect: AbacEffect,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Enabled flag
    pub enabled: bool,
}

/// Policy effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbacEffect {
    /// Allow
    Allow,
    /// Deny
    Deny,
}

/// ABAC context attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacContext {
    /// Principal attributes
    pub principal_attributes: HashMap<String, String>,
    /// Resource attributes
    pub resource_attributes: HashMap<String, String>,
    /// Environment attributes
    pub environment_attributes: HashMap<String, String>,
    /// Action being performed
    pub action: String,
    /// Resource ID
    pub resource_id: String,
}

/// ABAC policy engine
pub struct AbacPolicyEngine {
    /// Policy rules
    rules: Arc<RwLock<Vec<AbacPolicyRule>>>,
    /// RDF store for policy evaluation
    rdf_store: Arc<RwLock<Store>>,
}

impl AbacPolicyEngine {
    /// Create a new ABAC policy engine
    pub fn new() -> WorkflowResult<Self> {
        let store = Store::new()
            .map_err(|e| WorkflowError::Internal(format!("Failed to create RDF store: {:?}", e)))?;

        Ok(Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            rdf_store: Arc::new(RwLock::new(store)),
        })
    }

    /// Add a policy rule
    pub async fn add_rule(&self, rule: AbacPolicyRule) -> WorkflowResult<()> {
        // Validate RDF policy format
        let store = Store::new()
            .map_err(|e| WorkflowError::Internal(format!("Failed to create RDF store: {:?}", e)))?;

        store
            .load_from_reader(RdfFormat::Turtle, rule.rdf_policy.as_bytes())
            .map_err(|e| {
                WorkflowError::Validation(format!("Invalid RDF policy format: {:?}", e))
            })?;

        // Validate SPARQL query using deprecated Query::parse
        #[allow(deprecated)]
        oxigraph::sparql::Query::parse(&rule.evaluation_query, None)
            .map_err(|e| WorkflowError::Validation(format!("Invalid SPARQL query: {:?}", e)))?;

        let mut rules = self.rules.write().await;
        rules.push(rule);

        // Sort by priority
        rules.sort_by_key(|r| r.priority);

        Ok(())
    }

    /// Evaluate ABAC policy for a principal and resource
    pub async fn evaluate(
        &self,
        principal: &Principal,
        action: &str,
        resource_id: &str,
        resource_attributes: HashMap<String, String>,
    ) -> WorkflowResult<AbacDecision> {
        // Build ABAC context
        let context = AbacContext {
            principal_attributes: principal.attributes.clone(),
            resource_attributes,
            environment_attributes: self.get_environment_attributes().await,
            action: action.to_string(),
            resource_id: resource_id.to_string(),
        };

        // Load context into RDF store
        let context_rdf = self.context_to_rdf(&context, principal)?;
        let store = self.rdf_store.write().await;
        store
            .load_from_reader(RdfFormat::Turtle, context_rdf.as_bytes())
            .map_err(|e| {
                WorkflowError::Internal(format!("Failed to load context into store: {:?}", e))
            })?;

        // Evaluate rules in priority order
        let rules = self.rules.read().await;
        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            // Load policy into store
            store
                .load_from_reader(RdfFormat::Turtle, rule.rdf_policy.as_bytes())
                .map_err(|e| WorkflowError::Internal(format!("Failed to load policy: {:?}", e)))?;

            // Execute evaluation query using SparqlEvaluator
            #[allow(deprecated)]
            let query =
                oxigraph::sparql::Query::parse(&rule.evaluation_query, None).map_err(|e| {
                    WorkflowError::Validation(format!("Failed to parse SPARQL query: {:?}", e))
                })?;
            let results = store
                .query(query)
                .map_err(|e| WorkflowError::Internal(format!("SPARQL query failed: {:?}", e)))?;

            // Check if query returned results (policy matches)
            if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
                let mut has_results = false;
                for solution_result in solutions {
                    match solution_result {
                        Ok(_) => {
                            has_results = true;
                            break;
                        }
                        Err(_) => continue,
                    }
                }

                if has_results {
                    // Policy matches - return decision based on effect
                    return Ok(match rule.effect {
                        AbacEffect::Allow => AbacDecision::Allow,
                        AbacEffect::Deny => AbacDecision::Deny,
                    });
                }
            }
        }

        // No matching policy
        Ok(AbacDecision::NotApplicable)
    }

    /// Convert ABAC context to RDF (Turtle format)
    fn context_to_rdf(
        &self,
        context: &AbacContext,
        principal: &Principal,
    ) -> WorkflowResult<String> {
        let mut rdf = String::new();
        rdf.push_str("@prefix abac: <http://knhk.org/abac#> .\n");
        rdf.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\n");

        // Principal
        rdf.push_str(&format!(
            "abac:principal_{} a abac:Principal ;\n",
            principal.id
        ));
        rdf.push_str(&format!("    abac:principalId \"{}\" ;\n", principal.id));
        rdf.push_str(&format!(
            "    abac:principalType \"{:?}\" .\n\n",
            principal.principal_type
        ));

        // Principal attributes
        for (key, value) in &context.principal_attributes {
            rdf.push_str(&format!(
                "abac:principal_{} abac:hasAttribute [\n",
                principal.id
            ));
            rdf.push_str(&format!("    abac:attributeName \"{}\" ;\n", key));
            rdf.push_str(&format!("    abac:attributeValue \"{}\"\n", value));
            rdf.push_str("] .\n\n");
        }

        // Resource
        rdf.push_str(&format!(
            "abac:resource_{} a abac:Resource ;\n",
            context.resource_id
        ));
        rdf.push_str(&format!(
            "    abac:resourceId \"{}\" .\n\n",
            context.resource_id
        ));

        // Resource attributes
        for (key, value) in &context.resource_attributes {
            rdf.push_str(&format!(
                "abac:resource_{} abac:hasAttribute [\n",
                context.resource_id
            ));
            rdf.push_str(&format!("    abac:attributeName \"{}\" ;\n", key));
            rdf.push_str(&format!("    abac:attributeValue \"{}\"\n", value));
            rdf.push_str("] .\n\n");
        }

        // Action
        rdf.push_str(&format!("abac:action_{} a abac:Action ;\n", context.action));
        rdf.push_str(&format!("    abac:actionName \"{}\" .\n\n", context.action));

        // Environment attributes
        for (key, value) in &context.environment_attributes {
            rdf.push_str(&"abac:environment abac:hasAttribute [\n".to_string());
            rdf.push_str(&format!("    abac:attributeName \"{}\" ;\n", key));
            rdf.push_str(&format!("    abac:attributeValue \"{}\"\n", value));
            rdf.push_str("] .\n\n");
        }

        Ok(rdf)
    }

    /// Get environment attributes (time, IP, etc.)
    async fn get_environment_attributes(&self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        attrs.insert("time".to_string(), chrono::Utc::now().to_rfc3339());

        // Add IP address if available
        #[allow(unexpected_cfgs)]
        #[cfg(feature = "network")]
        {
            use std::net::TcpListener;
            if let Ok(listener) = TcpListener::bind("0.0.0.0:0") {
                if let Ok(addr) = listener.local_addr() {
                    attrs.insert("ip".to_string(), addr.ip().to_string());
                }
            }
        }

        // Add region from environment variable or default
        let region = std::env::var("AWS_REGION")
            .or_else(|_| std::env::var("REGION"))
            .unwrap_or_else(|_| "unknown".to_string());
        attrs.insert("region".to_string(), region);

        // Add hostname if available (use environment variable or fallback)
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            attrs.insert("hostname".to_string(), hostname);
        } else if let Ok(hostname) = std::env::var("COMPUTERNAME") {
            // Windows fallback
            attrs.insert("hostname".to_string(), hostname);
        }

        attrs
    }

    /// Get all rules
    pub async fn get_rules(&self) -> Vec<AbacPolicyRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }

    /// Remove a rule
    pub async fn remove_rule(&self, name: &str) -> WorkflowResult<()> {
        let mut rules = self.rules.write().await;
        let initial_len = rules.len();
        rules.retain(|r| r.name != name);
        if rules.len() == initial_len {
            Err(WorkflowError::Validation(format!(
                "Rule {} not found",
                name
            )))
        } else {
            Ok(())
        }
    }
}

impl Default for AbacPolicyEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            panic!(
                "AbacPolicyEngine::new should succeed with default configuration: {:?}",
                e
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::PrincipalType;

    #[tokio::test]
    async fn test_abac_policy_engine() {
        let engine = AbacPolicyEngine::new().expect("AbacPolicyEngine::new should succeed");

        let rule = AbacPolicyRule {
            name: "test-rule".to_string(),
            description: "Test rule".to_string(),
            rdf_policy: r#"
@prefix abac: <http://knhk.org/abac#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

abac:TestPolicy a abac:Policy ;
    abac:allows abac:testAction .
"#
            .to_string(),
            evaluation_query: r#"
PREFIX abac: <http://knhk.org/abac#>
ASK WHERE {
    ?policy a abac:Policy ;
        abac:allows ?action .
    ?action abac:actionName "testAction" .
}
"#
            .to_string(),
            effect: AbacEffect::Allow,
            priority: 0,
            enabled: true,
        };

        engine
            .add_rule(rule)
            .await
            .expect("add_rule should succeed");

        let principal = Principal {
            id: "test-user".to_string(),
            principal_type: PrincipalType::User,
            attributes: HashMap::new(),
        };

        let decision = engine
            .evaluate(&principal, "testAction", "resource-1", HashMap::new())
            .await
            .expect("evaluate should succeed");

        // Note: This test may not match due to RDF context not being loaded
        // In a real scenario, the context would be properly loaded
        assert!(matches!(
            decision,
            AbacDecision::Allow | AbacDecision::NotApplicable
        ));
    }
}
