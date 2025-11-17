// LLM-Based Proposer: Constraint-aware ontology change proposal generation
// Implements defense-in-depth constraint enforcement for autonomous evolution

use crate::doctrine::{DoctrineRule, DoctrineStore};
use crate::governance::{Guard, GuardProfile};
use crate::invariants::HardInvariants;
use crate::observation::DetectedPattern;
use crate::receipt::{Receipt, ReceiptStore, ReceiptOperation, ReceiptOutcome};
use crate::learning::{LearningSystem, ProposalOutcome};
use crate::prompt_engine::PromptEngine;
use crate::validator_llm::ProposalValidator;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum ProposerError {
    #[error("LLM generation failed: {0}")]
    LLMGenerationFailed(String),

    #[error("Proposal parsing failed: {0}")]
    ParsingFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Token budget exceeded: {0}")]
    TokenBudgetExceeded(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ProposerError>;

/// Sector classification for domain-specific prompting
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Sector {
    Finance,
    Healthcare,
    Manufacturing,
    Logistics,
    Generic,
}

impl std::fmt::Display for Sector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sector::Finance => write!(f, "finance"),
            Sector::Healthcare => write!(f, "healthcare"),
            Sector::Manufacturing => write!(f, "manufacturing"),
            Sector::Logistics => write!(f, "logistics"),
            Sector::Generic => write!(f, "generic"),
        }
    }
}

/// Request to generate a proposal from a detected pattern
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposalRequest {
    pub pattern: DetectedPattern,
    pub current_snapshot_id: String,
    pub doctrines: Vec<DoctrineRule>,
    pub invariants: HardInvariants,
    pub guard_profile: GuardProfile,
    pub performance_budget: PerformanceBudget,
}

/// Performance budget tracking for hot path operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceBudget {
    pub max_ticks: u32,           // Maximum allowed (default: 8)
    pub consumed_ticks: u32,      // Already used by existing ontology
    pub remaining_ticks: u32,     // Available for new proposal
    pub cost_per_class: f64,      // Estimated tick cost (default: 1.0)
    pub cost_per_property: f64,   // Estimated tick cost (default: 0.5)
    pub cost_per_validation: f64, // Estimated tick cost (default: 3.0)
}

impl PerformanceBudget {
    pub fn new(max_ticks: u32, consumed_ticks: u32) -> Self {
        PerformanceBudget {
            max_ticks,
            consumed_ticks,
            remaining_ticks: max_ticks.saturating_sub(consumed_ticks),
            cost_per_class: 1.0,
            cost_per_property: 0.5,
            cost_per_validation: 3.0,
        }
    }

    pub fn estimate_cost(&self, diff: &SigmaDiff) -> u32 {
        let class_cost = (diff.added_classes.len() as f64 * self.cost_per_class).ceil() as u32;
        let prop_cost = (diff.added_properties.len() as f64 * self.cost_per_property).ceil() as u32;
        let validation_cost = (diff.modified_shapes.len() as f64 * self.cost_per_validation).ceil() as u32;

        class_cost + prop_cost + validation_cost
    }

    pub fn can_afford(&self, diff: &SigmaDiff) -> bool {
        self.estimate_cost(diff) <= self.remaining_ticks
    }
}

impl Default for PerformanceBudget {
    fn default() -> Self {
        PerformanceBudget::new(8, 0)
    }
}

/// Guard profile defining immutable boundaries
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuardProfile {
    pub id: String,
    pub name: String,
    pub protected_classes: Vec<String>,
    pub protected_properties: Vec<String>,
    pub max_run_len: usize,
    pub performance_tier: PerformanceTier,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PerformanceTier {
    HotPath,   // ≤8 ticks (branchless C)
    WarmPath,  // ≤1ms (Rust)
    ColdPath,  // ≤100ms (Python/validation)
}

/// LLM-generated proposal for ontology change
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub pattern_id: String,
    pub pattern: DetectedPattern,        // Original pattern that triggered this proposal
    pub llm_prompt: String,              // Full prompt sent to LLM
    pub llm_response: String,            // Raw LLM output
    pub delta_sigma: SigmaDiff,          // Parsed ontology change
    pub reasoning: String,               // LLM's explanation
    pub confidence: f64,                 // LLM's confidence score (0.0-1.0)
    pub estimated_ticks: u32,            // Predicted execution time
    pub doctrines_satisfied: Vec<String>, // Doctrine IDs claimed
    pub invariants_satisfied: Vec<String>, // Q1-Q5 claimed
    pub can_rollback: bool,              // Is this reversible?
    pub timestamp: DateTime<Utc>,
}

/// Ontology change specification (ΔΣ)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SigmaDiff {
    pub added_classes: Vec<ClassDefinition>,
    pub removed_classes: Vec<String>,  // URIs
    pub added_properties: Vec<PropertyDefinition>,
    pub removed_properties: Vec<String>,  // URIs
    pub modified_shapes: Vec<ShapeDefinition>,  // SHACL updates
}

impl std::fmt::Display for SigmaDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ΔΣ: +{}c -{}c +{}p -{}p ~{}s",
            self.added_classes.len(),
            self.removed_classes.len(),
            self.added_properties.len(),
            self.removed_properties.len(),
            self.modified_shapes.len()
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassDefinition {
    pub uri: String,
    pub label: String,
    pub subclass_of: String,
    pub properties_required: Vec<String>,
    pub properties_optional: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub uri: String,
    pub label: String,
    pub domain: String,  // Class URI
    pub range: String,   // Datatype or Class URI
    pub required: bool,
    pub cardinality: Cardinality,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cardinality {
    One,           // Exactly 1
    ZeroOrOne,     // Optional
    ZeroOrMore,    // List
    OneOrMore,     // Non-empty list
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShapeDefinition {
    pub uri: String,
    pub added_constraints: Vec<String>,
    pub removed_constraints: Vec<String>,
}

/// Validation report for a proposal
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub proposal_id: String,
    pub passed: bool,
    pub stages: Vec<ValidationStage>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationStage {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

impl ValidationReport {
    pub fn new(proposal_id: String) -> Self {
        ValidationReport {
            proposal_id,
            passed: true,
            stages: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn add_pass(&mut self, stage: &str) {
        self.stages.push(ValidationStage {
            name: stage.to_string(),
            passed: true,
            message: None,
        });
    }

    pub fn add_fail(&mut self, stage: &str, message: String) {
        self.stages.push(ValidationStage {
            name: stage.to_string(),
            passed: false,
            message: Some(message),
        });
        self.passed = false;
    }
}

/// Core LLM proposer trait
#[async_trait]
pub trait LLMProposer: Send + Sync {
    /// Generate a proposal from an observed pattern
    async fn generate_proposal(
        &self,
        pattern: &DetectedPattern,
        doctrines: &[DoctrineRule],
        invariants: &HardInvariants,
        guards: &GuardProfile,
    ) -> Result<Proposal>;

    /// Validate a proposal against constraints
    async fn validate_proposal(
        &self,
        proposal: &Proposal,
    ) -> Result<ValidationReport>;

    /// Record proposal outcome for learning
    async fn record_outcome(
        &self,
        proposal: Proposal,
        report: ValidationReport,
    ) -> Result<()>;

    /// Get few-shot examples for a sector
    fn get_examples(&self, sector: &Sector, count: usize) -> Vec<FewShotExample>;
}

/// Few-shot example from learning corpus
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FewShotExample {
    pub pattern: String,
    pub proposal: SigmaDiff,
    pub reasoning: String,
    pub confidence: f64,
    pub validation_result: ValidationReport,
}

/// Main LLM proposer implementation
pub struct OllamaLLMProposer {
    llm_client: Arc<dyn LLMClient>,
    prompt_engine: Arc<PromptEngine>,
    validator: Arc<dyn ProposalValidator>,
    learning_system: Arc<RwLock<LearningSystem>>,
    receipt_store: Arc<ReceiptStore>,
    signing_key: SigningKey,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    cost_controller: Arc<RwLock<CostController>>,
}

impl OllamaLLMProposer {
    pub fn new(
        llm_client: Arc<dyn LLMClient>,
        prompt_engine: Arc<PromptEngine>,
        validator: Arc<dyn ProposalValidator>,
        learning_system: Arc<RwLock<LearningSystem>>,
        receipt_store: Arc<ReceiptStore>,
        signing_key: SigningKey,
    ) -> Self {
        OllamaLLMProposer {
            llm_client,
            prompt_engine,
            validator,
            learning_system,
            receipt_store,
            signing_key,
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new(10))), // 10 proposals/hour
            cost_controller: Arc::new(RwLock::new(CostController::new(10000, 100000))), // 10K tokens/proposal, 100K total
        }
    }

    #[tracing::instrument(skip(self, pattern), fields(pattern_id = %pattern.id))]
    async fn generate_proposal_internal(
        &self,
        pattern: &DetectedPattern,
        doctrines: &[DoctrineRule],
        invariants: &HardInvariants,
        guards: &GuardProfile,
    ) -> Result<Proposal> {
        // Build request
        let request = ProposalRequest {
            pattern: pattern.clone(),
            // Get current snapshot ID from pattern metadata or use timestamp-based ID
            current_snapshot_id: format!(
                "snapshot-{}",
                pattern.timestamp.timestamp_millis()
            ),
            doctrines: doctrines.to_vec(),
            invariants: invariants.clone(),
            guard_profile: guards.clone(),
            // Calculate performance budget from current guard profile
            // max_run_len is the limit, assume current consumption is half of limit
            performance_budget: PerformanceBudget::new(
                guards.max_run_len as u32,
                (guards.max_run_len as u32) / 2 // Conservative estimate: assume 50% already used
            ),
        };

        // Generate constraint-aware prompt
        let prompt = self.prompt_engine.build_full_prompt(&request)?;

        // Check token budget
        self.cost_controller.write().await
            .check_token_budget(estimate_token_count(&prompt))
            .map_err(|e| ProposerError::TokenBudgetExceeded(e.to_string()))?;

        // Call LLM with timeout
        let llm_response = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.llm_client.generate(&prompt)
        )
        .await
        .map_err(|_| ProposerError::Timeout("LLM generation timed out".to_string()))?
        .map_err(|e| ProposerError::LLMGenerationFailed(e.to_string()))?;

        // Record token usage
        self.cost_controller.write().await
            .record_usage(estimate_token_count(&llm_response));

        // Parse LLM response
        let proposal = self.parse_proposal_response(&llm_response, pattern)?;

        // Post-hoc validation
        self.validate_all_constraints(&proposal, &request).await?;

        Ok(proposal)
    }

    fn parse_proposal_response(
        &self,
        response: &str,
        pattern: &DetectedPattern,
    ) -> Result<Proposal> {
        // Parse JSON response
        let parsed: serde_json::Value = serde_json::from_str(response)
            .map_err(|e| ProposerError::ParsingFailed(e.to_string()))?;

        // Extract fields
        let reasoning = parsed["reasoning"].as_str()
            .ok_or_else(|| ProposerError::ParsingFailed("Missing 'reasoning' field".to_string()))?
            .to_string();

        let confidence = parsed["confidence"].as_f64()
            .ok_or_else(|| ProposerError::ParsingFailed("Missing 'confidence' field".to_string()))?;

        let estimated_ticks = parsed["estimated_ticks"].as_u64()
            .ok_or_else(|| ProposerError::ParsingFailed("Missing 'estimated_ticks' field".to_string()))? as u32;

        // Parse delta_sigma
        let delta_sigma: SigmaDiff = serde_json::from_value(parsed["delta_sigma"].clone())
            .map_err(|e| ProposerError::ParsingFailed(format!("Invalid delta_sigma: {}", e)))?;

        let doctrines_satisfied = parsed["doctrines_satisfied"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default();

        let invariants_satisfied = parsed["invariants_satisfied"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default();

        Ok(Proposal {
            id: format!("prop-{}", uuid::Uuid::new_v4()),
            pattern_id: pattern.id.clone(),
            pattern: pattern.clone(),
            llm_prompt: parsed.get("original_prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            llm_response: response.to_string(),
            delta_sigma: delta_sigma.clone(),
            reasoning,
            confidence,
            estimated_ticks,
            doctrines_satisfied,
            invariants_satisfied,
            // Analyze rollback capability based on the delta
            can_rollback: Self::analyze_rollback_capability(&delta_sigma),
            timestamp: Utc::now(),
        })
    }

    /// Analyze if a delta can be rolled back
    fn analyze_rollback_capability(delta: &SigmaDiff) -> bool {
        // A change is reversible if:
        // 1. It only adds elements (no removals) - always reversible
        // 2. It modifies shapes but doesn't remove constraints - reversible
        // 3. It removes elements - may not be reversible if data exists

        // If we're only adding things, always safe to rollback (just remove what we added)
        if delta.removed_classes.is_empty() && delta.removed_properties.is_empty() {
            return true;
        }

        // If we're removing classes or properties, rollback is risky
        // because we might have lost data or references
        if !delta.removed_classes.is_empty() || !delta.removed_properties.is_empty() {
            tracing::warn!(
                "Rollback analysis: proposal removes {} classes and {} properties - may not be reversible",
                delta.removed_classes.len(),
                delta.removed_properties.len()
            );
            return false;
        }

        // Shape modifications are usually reversible
        for shape_mod in &delta.modified_shapes {
            if !shape_mod.removed_constraints.is_empty() {
                // Removing constraints might allow invalid data - risky to rollback
                tracing::debug!(
                    "Rollback analysis: shape {} removes constraints - may affect data validity",
                    shape_mod.uri
                );
            }
        }

        // Default to conservative (true = can rollback)
        true
    }

    async fn validate_all_constraints(
        &self,
        proposal: &Proposal,
        request: &ProposalRequest,
    ) -> Result<()> {
        // Validate performance budget
        if !request.performance_budget.can_afford(&proposal.delta_sigma) {
            return Err(ProposerError::ConstraintViolation(format!(
                "Performance budget exceeded: estimated {} ticks, remaining {}",
                request.performance_budget.estimate_cost(&proposal.delta_sigma),
                request.performance_budget.remaining_ticks
            )));
        }

        // Validate estimated ticks within limit
        if proposal.estimated_ticks > request.performance_budget.max_ticks {
            return Err(ProposerError::ConstraintViolation(format!(
                "Q3 violation: estimated {} ticks > max {}",
                proposal.estimated_ticks,
                request.performance_budget.max_ticks
            )));
        }

        // Validate no protected elements removed
        for removed_class in &proposal.delta_sigma.removed_classes {
            if request.guard_profile.protected_classes.contains(removed_class) {
                return Err(ProposerError::ConstraintViolation(format!(
                    "Guard violation: cannot remove protected class '{}'",
                    removed_class
                )));
            }
        }

        for removed_prop in &proposal.delta_sigma.removed_properties {
            if request.guard_profile.protected_properties.contains(removed_prop) {
                return Err(ProposerError::ConstraintViolation(format!(
                    "Guard violation: cannot remove protected property '{}'",
                    removed_prop
                )));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl LLMProposer for OllamaLLMProposer {
    async fn generate_proposal(
        &self,
        pattern: &DetectedPattern,
        doctrines: &[DoctrineRule],
        invariants: &HardInvariants,
        guards: &GuardProfile,
    ) -> Result<Proposal> {
        // Check rate limit
        self.rate_limiter.write().await
            .check_rate_limit()
            .map_err(|e| ProposerError::RateLimitExceeded(e.to_string()))?;

        // Generate proposal
        self.generate_proposal_internal(pattern, doctrines, invariants, guards).await
    }

    async fn validate_proposal(
        &self,
        proposal: &Proposal,
    ) -> Result<ValidationReport> {
        self.validator.validate_all(proposal).await
            .map_err(|e| ProposerError::ValidationFailed(e.to_string()))
    }

    async fn record_outcome(
        &self,
        proposal: Proposal,
        report: ValidationReport,
    ) -> Result<()> {
        let mut learning = self.learning_system.write().await;
        learning.record_outcome(proposal, report)
            .map_err(|e| ProposerError::Internal(e.to_string()))
    }

    fn get_examples(&self, sector: &Sector, count: usize) -> Vec<FewShotExample> {
        // Handle async lock appropriately using blocking_lock() for sync context
        // This is safe because we're in a sync trait method
        match self.learning_system.try_read() {
            Some(learning) => {
                // Successfully acquired read lock, get examples
                learning.get_successful_examples(sector, count)
                    .unwrap_or_else(|_| Vec::new())
            }
            None => {
                // Lock contention, return empty to avoid blocking
                tracing::warn!(
                    "Failed to acquire learning system lock for examples, returning empty"
                );
                Vec::new()
            }
        }
    }
}

/// LLM client trait for API abstraction
#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn generate(&self, prompt: &str) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

/// Rate limiter for proposal generation
pub struct RateLimiter {
    max_proposals_per_hour: usize,
    recent_proposals: std::collections::VecDeque<DateTime<Utc>>,
}

impl RateLimiter {
    pub fn new(max_proposals_per_hour: usize) -> Self {
        RateLimiter {
            max_proposals_per_hour,
            recent_proposals: std::collections::VecDeque::new(),
        }
    }

    pub fn check_rate_limit(&mut self) -> std::result::Result<(), String> {
        // Remove proposals older than 1 hour
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        self.recent_proposals.retain(|ts| *ts > cutoff);

        if self.recent_proposals.len() >= self.max_proposals_per_hour {
            return Err(format!(
                "Rate limit exceeded: {} proposals in last hour (max: {})",
                self.recent_proposals.len(),
                self.max_proposals_per_hour
            ));
        }

        self.recent_proposals.push_back(Utc::now());
        Ok(())
    }
}

/// Cost controller for token budget management
pub struct CostController {
    max_tokens_per_proposal: usize,
    total_tokens_used: usize,
    budget_limit_tokens: usize,
}

impl CostController {
    pub fn new(max_tokens_per_proposal: usize, budget_limit_tokens: usize) -> Self {
        CostController {
            max_tokens_per_proposal,
            total_tokens_used: 0,
            budget_limit_tokens,
        }
    }

    pub fn check_token_budget(&self, prompt_tokens: usize) -> std::result::Result<(), String> {
        if prompt_tokens > self.max_tokens_per_proposal {
            return Err(format!(
                "Prompt too large: {} tokens (max: {})",
                prompt_tokens,
                self.max_tokens_per_proposal
            ));
        }

        if self.total_tokens_used + prompt_tokens > self.budget_limit_tokens {
            return Err(format!(
                "Token budget exceeded: {} + {} > {}",
                self.total_tokens_used,
                prompt_tokens,
                self.budget_limit_tokens
            ));
        }

        Ok(())
    }

    pub fn record_usage(&mut self, tokens: usize) {
        self.total_tokens_used += tokens;
    }
}

/// Estimate token count (rough approximation: 1 token ≈ 4 characters)
fn estimate_token_count(text: &str) -> usize {
    text.len() / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_budget_estimation() {
        let budget = PerformanceBudget::new(8, 5);

        assert_eq!(budget.remaining_ticks, 3);

        let diff = SigmaDiff {
            added_classes: vec![ClassDefinition {
                uri: "test:NewClass".to_string(),
                label: "New Class".to_string(),
                subclass_of: "test:Base".to_string(),
                properties_required: vec![],
                properties_optional: vec![],
            }],
            added_properties: vec![PropertyDefinition {
                uri: "test:newProp".to_string(),
                label: "New Property".to_string(),
                domain: "test:NewClass".to_string(),
                range: "xsd:string".to_string(),
                required: true,
                cardinality: Cardinality::One,
            }],
            ..Default::default()
        };

        let cost = budget.estimate_cost(&diff);
        assert_eq!(cost, 2); // 1 class (1.0) + 1 property (0.5) = 1.5 → ceil = 2
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3);

        assert!(limiter.check_rate_limit().is_ok());
        assert!(limiter.check_rate_limit().is_ok());
        assert!(limiter.check_rate_limit().is_ok());
        assert!(limiter.check_rate_limit().is_err()); // 4th should fail
    }

    #[test]
    fn test_cost_controller() {
        let mut controller = CostController::new(1000, 5000);

        assert!(controller.check_token_budget(500).is_ok());
        controller.record_usage(500);

        assert!(controller.check_token_budget(4500).is_ok());
        controller.record_usage(4500);

        assert!(controller.check_token_budget(100).is_err()); // Would exceed 5000
    }
}
