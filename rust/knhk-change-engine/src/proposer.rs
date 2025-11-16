//! ΔΣ Proposer - Generates ontology change proposals
//!
//! The Proposer generates validated ΔΣ² change proposals based on detected patterns.
//! It uses both deterministic policy rules and optional LLM-based suggestions.
//!
//! **CRITICAL**: LLM output is NEVER applied directly. All proposals are converted
//! to Σ² objects and validated before execution.

use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, span, Level};
use crate::pattern_miner::DetectedPatterns;

/// ΔΣ Proposal types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeltaSigmaProposal {
    /// Add a new class to the ontology
    AddClass {
        /// Class name
        name: String,
        /// Properties of the class
        properties: Vec<PropertyDef>,
        /// Guards for the class
        guards: Vec<GuardRule>,
        /// Sector (domain area)
        sector: String,
    },

    /// Remove a class from the ontology
    RemoveClass {
        /// Class name
        name: String,
        /// Reason for removal
        reason: String,
    },

    /// Add a property to an existing class
    AddProperty {
        /// Domain class (class this property belongs to)
        domain_class: String,
        /// Property name
        property_name: String,
        /// Range (type of values)
        range: String,
        /// Cardinality constraint
        cardinality: Cardinality,
        /// Guards for the property
        guards: Vec<GuardRule>,
    },

    /// Tighten an existing constraint
    TightenConstraint {
        /// Constraint ID
        constraint_id: String,
        /// New constraint expression
        new_expression: String,
    },

    /// Relax an existing constraint
    RelaxConstraint {
        /// Constraint ID
        constraint_id: String,
        /// New constraint expression
        new_expression: String,
    },
}

/// Property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    /// Property name
    pub name: String,
    /// Property type/range
    pub range: String,
    /// Cardinality
    pub cardinality: Cardinality,
    /// Guards
    pub guards: Vec<GuardRule>,
}

/// Cardinality constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cardinality {
    /// Exactly one
    One,
    /// Zero or one
    ZeroOrOne,
    /// Zero or more
    ZeroOrMany,
    /// One or more
    OneOrMany,
    /// Exact count
    Exact(u32),
    /// Range
    Range(u32, u32),
}

/// Guard rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardRule {
    /// Mask PII data
    MaskPII,
    /// Require authentication
    RequireAuth,
    /// Rate limit
    RateLimit(u32),
    /// Custom SPARQL constraint
    SparqlConstraint(String),
}

/// ΔΣ Proposer
#[derive(Debug)]
pub struct DeltaSigmaProposer {
    /// Access to pattern miner
    patterns: Arc<RwLock<DetectedPatterns>>,

    /// Policy rules (deterministic generators)
    policy_rules: Vec<PolicyRule>,

    /// LLM client (optional)
    #[cfg(feature = "llm")]
    llm_client: Option<Arc<LlmClient>>,
}

/// Policy rule - deterministic proposal generator
pub struct PolicyRule {
    /// Rule name
    pub name: String,

    /// Condition function (returns true if rule should trigger)
    pub condition: Box<dyn Fn(&DetectedPatterns) -> bool + Send + Sync>,

    /// Proposal generator function
    pub proposal_fn: Box<dyn Fn(&DetectedPatterns) -> Vec<DeltaSigmaProposal> + Send + Sync>,
}

impl std::fmt::Debug for PolicyRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolicyRule")
            .field("name", &self.name)
            .finish()
    }
}

/// LLM client for generating proposals
#[cfg(feature = "llm")]
#[derive(Debug)]
pub struct LlmClient {
    /// API endpoint
    endpoint: String,
    /// API key
    api_key: String,
}

impl DeltaSigmaProposer {
    /// Create a new proposer with default policy rules
    pub fn new(patterns: Arc<RwLock<DetectedPatterns>>) -> Self {
        let span = span!(Level::INFO, "proposer_init");
        let _enter = span.enter();

        info!("Initializing ΔΣ proposer with default policy rules");

        Self {
            patterns,
            policy_rules: Self::create_default_policy_rules(),
            #[cfg(feature = "llm")]
            llm_client: None,
        }
    }

    /// Create default policy rules
    fn create_default_policy_rules() -> Vec<PolicyRule> {
        vec![
            // Rule 1: High frequency schema mismatches → Add class
            PolicyRule {
                name: "high_schema_mismatches".to_string(),
                condition: Box::new(|patterns| {
                    patterns.schema_mismatches.len() > 10
                }),
                proposal_fn: Box::new(|patterns| {
                    let mut proposals = Vec::new();

                    // Group mismatches by predicate
                    let mut predicate_groups: std::collections::HashMap<String, Vec<&crate::pattern_miner::SchemaMismatch>>
                        = std::collections::HashMap::new();

                    for mismatch in &patterns.schema_mismatches {
                        predicate_groups
                            .entry(mismatch.triple.predicate.clone())
                            .or_default()
                            .push(mismatch);
                    }

                    // Propose new classes for high-frequency predicates
                    for (predicate, mismatches) in predicate_groups {
                        if mismatches.len() >= 5 {
                            proposals.push(DeltaSigmaProposal::AddClass {
                                name: format!("{}Class", predicate),
                                properties: vec![
                                    PropertyDef {
                                        name: predicate.clone(),
                                        range: "xsd:string".to_string(),
                                        cardinality: Cardinality::One,
                                        guards: vec![],
                                    }
                                ],
                                guards: vec![],
                                sector: "auto-detected".to_string(),
                            });
                        }
                    }

                    proposals
                }),
            },

            // Rule 2: PII patterns → Add masking guards
            PolicyRule {
                name: "pii_patterns".to_string(),
                condition: Box::new(|patterns| {
                    patterns.schema_mismatches.iter().any(|m| m.reason.contains("PII"))
                }),
                proposal_fn: Box::new(|patterns| {
                    patterns.schema_mismatches.iter()
                        .filter(|m| m.reason.contains("PII"))
                        .map(|m| DeltaSigmaProposal::AddProperty {
                            domain_class: "Entity".to_string(),
                            property_name: m.triple.predicate.clone(),
                            range: "xsd:string".to_string(),
                            cardinality: Cardinality::One,
                            guards: vec![GuardRule::MaskPII],
                        })
                        .collect()
                }),
            },

            // Rule 3: Guard violations → Tighten constraints
            PolicyRule {
                name: "guard_violations".to_string(),
                condition: Box::new(|patterns| {
                    patterns.guard_violations.iter().any(|v| v.near_miss_count > 3)
                }),
                proposal_fn: Box::new(|patterns| {
                    patterns.guard_violations.iter()
                        .filter(|v| v.near_miss_count > 3)
                        .map(|v| DeltaSigmaProposal::TightenConstraint {
                            constraint_id: v.guard_name.clone(),
                            new_expression: format!("stricter_{}", v.guard_name),
                        })
                        .collect()
                }),
            },

            // Rule 4: Performance regressions → Relax non-critical constraints
            PolicyRule {
                name: "performance_regressions".to_string(),
                condition: Box::new(|patterns| {
                    patterns.performance_regressions.iter().any(|r| r.regression_factor > 2.0)
                }),
                proposal_fn: Box::new(|patterns| {
                    patterns.performance_regressions.iter()
                        .filter(|r| r.regression_factor > 2.0)
                        .map(|r| DeltaSigmaProposal::RelaxConstraint {
                            constraint_id: format!("perf_{}", r.operator),
                            new_expression: "relaxed_for_performance".to_string(),
                        })
                        .collect()
                }),
            },
        ]
    }

    /// Propose ΔΣ changes based on detected patterns
    pub async fn propose_delta_sigma(&self) -> crate::Result<Vec<DeltaSigmaProposal>> {
        let span = span!(Level::INFO, "propose_delta_sigma");
        let _enter = span.enter();

        let patterns = self.patterns.read().clone();
        let mut proposals = Vec::new();

        // 1. Apply policy rules (deterministic)
        for rule in &self.policy_rules {
            if (rule.condition)(&patterns) {
                debug!(rule_name = %rule.name, "Policy rule triggered");
                let rule_proposals = (rule.proposal_fn)(&patterns);
                info!(rule_name = %rule.name, count = rule_proposals.len(), "Generated proposals from policy rule");
                proposals.extend(rule_proposals);
            }
        }

        // 2. Use LLM to propose higher-level changes (optional)
        #[cfg(feature = "llm")]
        if let Some(ref llm) = self.llm_client {
            if patterns.schema_mismatches.len() > 10 {
                let llm_proposals = self.llm_propose_classes(&patterns, llm).await?;
                info!(count = llm_proposals.len(), "Generated proposals from LLM");
                proposals.extend(llm_proposals);
            }
        }

        info!(total_proposals = proposals.len(), "ΔΣ proposal generation complete");

        Ok(proposals)
    }

    /// Use LLM to propose new classes (feature-gated)
    #[cfg(feature = "llm")]
    async fn llm_propose_classes(
        &self,
        patterns: &DetectedPatterns,
        _llm: &LlmClient,
    ) -> crate::Result<Vec<DeltaSigmaProposal>> {
        // Placeholder for LLM integration
        // In production, this would:
        // 1. Format patterns as prompt context
        // 2. Call LLM API (Claude, GPT, etc.)
        // 3. Parse response into DeltaSigmaProposal objects
        // 4. Return for validation (NEVER applied directly)

        debug!(mismatches = patterns.schema_mismatches.len(), "LLM proposal generation (placeholder)");
        Ok(vec![])
    }

    /// Add a custom policy rule
    pub fn add_policy_rule(&mut self, rule: PolicyRule) {
        self.policy_rules.push(rule);
    }

    /// Set LLM client (feature-gated)
    #[cfg(feature = "llm")]
    pub fn set_llm_client(&mut self, client: LlmClient) {
        self.llm_client = Some(Arc::new(client));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern_miner::SchemaMismatch;

    #[test]
    fn test_proposer_creation() {
        let patterns = Arc::new(RwLock::new(DetectedPatterns::default()));
        let proposer = DeltaSigmaProposer::new(patterns);
        assert_eq!(proposer.policy_rules.len(), 4);
    }

    #[tokio::test]
    async fn test_propose_with_no_patterns() {
        let patterns = Arc::new(RwLock::new(DetectedPatterns::default()));
        let proposer = DeltaSigmaProposer::new(patterns);

        let proposals = proposer.propose_delta_sigma().await.unwrap();
        assert_eq!(proposals.len(), 0);
    }

    #[tokio::test]
    async fn test_propose_with_schema_mismatches() {
        let mut detected = DetectedPatterns::default();

        // Add 12 schema mismatches (threshold is 10)
        for i in 0..12 {
            detected.schema_mismatches.push(SchemaMismatch {
                triple: crate::pattern_miner::Triple {
                    subject: format!("s{}", i),
                    predicate: "hasName".to_string(),
                    object: format!("o{}", i),
                },
                reason: "type mismatch".to_string(),
                frequency: 1,
            });
        }

        let patterns = Arc::new(RwLock::new(detected));
        let proposer = DeltaSigmaProposer::new(patterns);

        let proposals = proposer.propose_delta_sigma().await.unwrap();
        assert!(proposals.len() > 0);
    }
}
