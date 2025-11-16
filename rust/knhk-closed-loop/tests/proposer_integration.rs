// Integration tests for LLM Proposer System
// Tests end-to-end proposal generation, validation, and learning

use knhk_closed_loop::proposer::*;
use knhk_closed_loop::prompt_engine::PromptEngine;
use knhk_closed_loop::validator_llm::{ValidationPipeline, ProposalValidator};
use knhk_closed_loop::learning::LearningSystem;
use knhk_closed_loop::observation::{DetectedPattern, PatternAction};
use knhk_closed_loop::doctrine::DoctrineRule;
use knhk_closed_loop::invariants::HardInvariants;
use knhk_closed_loop::receipt::ReceiptStore;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use ed25519_dalek::SigningKey;

/// Mock LLM client for testing
pub struct MockLLMClient {
    response: String,
}

impl MockLLMClient {
    pub fn new(response: String) -> Self {
        MockLLMClient { response }
    }

    pub fn with_valid_finance_response() -> Self {
        MockLLMClient::new(r#"{
  "reasoning": "Finance sector requires specialized account types for regulatory compliance. Adding RetirementAccount as subclass of Account satisfies approval chain requirements and maintains audit trails. Adds only 1 tick overhead.",
  "confidence": 0.85,
  "estimated_ticks": 7,
  "delta_sigma": {
    "added_classes": [{
      "uri": "finance:RetirementAccount",
      "label": "Retirement Account",
      "subclass_of": "finance:Account",
      "properties_required": ["account_id", "tax_status"],
      "properties_optional": []
    }],
    "added_properties": [{
      "uri": "finance:tax_status",
      "label": "Tax Treatment Status",
      "domain": "finance:RetirementAccount",
      "range": "xsd:string",
      "required": true,
      "cardinality": "One"
    }],
    "removed_classes": [],
    "removed_properties": [],
    "modified_shapes": []
  },
  "doctrines_satisfied": ["FIN-001", "FIN-002"],
  "invariants_satisfied": ["Q1", "Q2", "Q3", "Q4", "Q5"],
  "rollback_plan": "Remove RetirementAccount class and tax_status property. No data loss as class is new."
}"#.to_string())
    }

    pub fn with_q3_violation_response() -> Self {
        MockLLMClient::new(r#"{
  "reasoning": "Add comprehensive transaction validation with 10-step verification",
  "confidence": 0.7,
  "estimated_ticks": 12,
  "delta_sigma": {
    "added_classes": [],
    "added_properties": [],
    "removed_classes": [],
    "removed_properties": [],
    "modified_shapes": []
  },
  "doctrines_satisfied": [],
  "invariants_satisfied": [],
  "rollback_plan": "N/A"
}"#.to_string())
    }
}

#[async_trait]
impl LLMClient for MockLLMClient {
    async fn generate(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.response.clone())
    }
}

fn create_test_pattern(sector: Sector) -> DetectedPattern {
    DetectedPattern {
        id: format!("test-pattern-{}", uuid::Uuid::new_v4()),
        name: "New account types observed".to_string(),
        description: "Observed 15 retirement account variations (401k, IRA, Roth IRA)".to_string(),
        sector,
        confidence: 0.85,
        observations: vec![],
        timestamp: chrono::Utc::now(),
        recommended_action: PatternAction::ProposeChange {
            description: "Add RetirementAccount class".to_string(),
        },
    }
}

fn create_test_guards(sector: Sector) -> GuardProfile {
    match sector {
        Sector::Finance => GuardProfile {
            id: "FINANCE_CORE_GUARD".to_string(),
            name: "Finance Core Guard".to_string(),
            protected_classes: vec!["Account".to_string(), "Transaction".to_string()],
            protected_properties: vec!["account_id".to_string(), "transaction_id".to_string()],
            max_run_len: 8,
            performance_tier: PerformanceTier::HotPath,
        },
        _ => GuardProfile {
            id: "GENERIC_GUARD".to_string(),
            name: "Generic Guard".to_string(),
            protected_classes: vec![],
            protected_properties: vec![],
            max_run_len: 8,
            performance_tier: PerformanceTier::WarmPath,
        },
    }
}

fn create_signing_key() -> SigningKey {
    let mut seed = [0u8; 32];
    seed[0] = 42;
    SigningKey::from_bytes(&seed)
}

#[tokio::test]
async fn test_end_to_end_proposal_generation_success() {
    // Setup
    let llm_client = Arc::new(MockLLMClient::with_valid_finance_response());
    let prompt_engine = Arc::new(PromptEngine::new());
    let validator = Arc::new(ValidationPipeline::new());
    let learning_system = Arc::new(RwLock::new(LearningSystem::new()));
    let signing_key = create_signing_key();
    let verifying_key = signing_key.verifying_key();
    let receipt_store = Arc::new(ReceiptStore::new(verifying_key));

    let proposer = OllamaLLMProposer::new(
        llm_client,
        prompt_engine,
        validator,
        learning_system.clone(),
        receipt_store,
        signing_key,
    );

    // Create test inputs
    let pattern = create_test_pattern(Sector::Finance);
    let doctrines = vec![];
    let invariants = HardInvariants::default();
    let guards = create_test_guards(Sector::Finance);

    // Generate proposal
    let proposal = proposer.generate_proposal(&pattern, &doctrines, &invariants, &guards)
        .await
        .expect("Proposal generation should succeed");

    // Assertions
    assert_eq!(proposal.pattern_id, pattern.id);
    assert!(proposal.confidence > 0.7);
    assert!(proposal.estimated_ticks <= 8);
    assert!(!proposal.delta_sigma.added_classes.is_empty());
    assert_eq!(proposal.delta_sigma.added_classes[0].uri, "finance:RetirementAccount");
}

#[tokio::test]
async fn test_proposal_validation_rejects_q3_violation() {
    // Setup
    let llm_client = Arc::new(MockLLMClient::with_q3_violation_response());
    let prompt_engine = Arc::new(PromptEngine::new());
    let validator = Arc::new(ValidationPipeline::new());
    let learning_system = Arc::new(RwLock::new(LearningSystem::new()));
    let signing_key = create_signing_key();
    let verifying_key = signing_key.verifying_key();
    let receipt_store = Arc::new(ReceiptStore::new(verifying_key));

    let proposer = OllamaLLMProposer::new(
        llm_client,
        prompt_engine,
        validator,
        learning_system.clone(),
        receipt_store,
        signing_key,
    );

    // Create test inputs
    let pattern = create_test_pattern(Sector::Finance);
    let doctrines = vec![];
    let invariants = HardInvariants::default();
    let guards = create_test_guards(Sector::Finance);

    // Attempt to generate proposal (should fail constraint check)
    let result = proposer.generate_proposal(&pattern, &doctrines, &invariants, &guards).await;

    // Should fail due to Q3 violation (12 ticks > 8)
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Q3"));
}

#[tokio::test]
async fn test_prompt_engine_includes_all_constraints() {
    let engine = PromptEngine::new();

    let pattern = create_test_pattern(Sector::Finance);
    let request = ProposalRequest {
        pattern,
        current_snapshot_id: "snapshot-123".to_string(),
        doctrines: vec![DoctrineRule {
            id: "FIN-001".to_string(),
            name: "Approval Chain Requirement".to_string(),
            description: "All account modifications require two-step approval".to_string(),
            constraint_type: knhk_closed_loop::doctrine::ConstraintType::ApprovalRequired,
            enforcement_level: knhk_closed_loop::doctrine::EnforcementLevel::Mandatory,
            affected_classes: vec!["Account".to_string()],
            affected_properties: vec![],
            sector: "finance".to_string(),
            version: 1,
            created_at: chrono::Utc::now(),
        }],
        invariants: HardInvariants::default(),
        guard_profile: create_test_guards(Sector::Finance),
        performance_budget: PerformanceBudget::new(8, 5),
    };

    let prompt = engine.build_full_prompt(&request).expect("Prompt build should succeed");

    // Assert all constraint sections are present
    assert!(prompt.contains("Q1: No Retrocausation"));
    assert!(prompt.contains("Q2: Type Soundness"));
    assert!(prompt.contains("Q3: Guard Preservation"));
    assert!(prompt.contains("Q4: SLO Compliance"));
    assert!(prompt.contains("Q5: Performance Bounds"));
    assert!(prompt.contains("FIN-001"));
    assert!(prompt.contains("Protected classes"));
    assert!(prompt.contains("Performance Budget"));
    assert!(prompt.contains("OUTPUT FORMAT"));
    assert!(prompt.contains("EXAMPLES"));
}

#[tokio::test]
async fn test_learning_system_tracks_outcomes() {
    let mut learning = LearningSystem::new();

    // Create a passing proposal
    let pattern = create_test_pattern(Sector::Finance);
    let proposal = Proposal {
        id: "test-prop-1".to_string(),
        pattern_id: pattern.id.clone(),
        llm_prompt: String::new(),
        llm_response: String::new(),
        delta_sigma: SigmaDiff {
            added_classes: vec![ClassDefinition {
                uri: "finance:RetirementAccount".to_string(),
                label: "Retirement Account".to_string(),
                subclass_of: "finance:Account".to_string(),
                properties_required: vec![],
                properties_optional: vec![],
            }],
            ..Default::default()
        },
        reasoning: "Test reasoning".to_string(),
        confidence: 0.85,
        estimated_ticks: 7,
        doctrines_satisfied: vec!["FIN-001".to_string()],
        invariants_satisfied: vec!["Q1".to_string(), "Q2".to_string(), "Q3".to_string()],
        can_rollback: true,
        timestamp: chrono::Utc::now(),
        pattern,
    };

    let mut report = ValidationReport::new(proposal.id.clone());
    report.add_pass("static_check");
    report.add_pass("invariant_Q1");
    report.add_pass("invariant_Q2");
    report.add_pass("invariant_Q3");

    // Record outcome
    learning.record_outcome(proposal, report).expect("Record should succeed");

    // Verify learning system tracked it
    let metrics = learning.metrics();
    assert_eq!(metrics.acceptance_rate, 1.0);

    // Verify few-shot examples created
    let examples = learning.get_few_shot_examples(&Sector::Finance, 5);
    assert_eq!(examples.len(), 1);
}

#[tokio::test]
async fn test_rate_limiting() {
    let llm_client = Arc::new(MockLLMClient::with_valid_finance_response());
    let prompt_engine = Arc::new(PromptEngine::new());
    let validator = Arc::new(ValidationPipeline::new());
    let learning_system = Arc::new(RwLock::new(LearningSystem::new()));
    let signing_key = create_signing_key();
    let verifying_key = signing_key.verifying_key();
    let receipt_store = Arc::new(ReceiptStore::new(verifying_key));

    let proposer = OllamaLLMProposer::new(
        llm_client,
        prompt_engine,
        validator,
        learning_system,
        receipt_store,
        signing_key,
    );

    let pattern = create_test_pattern(Sector::Finance);
    let doctrines = vec![];
    let invariants = HardInvariants::default();
    let guards = create_test_guards(Sector::Finance);

    // Generate 10 proposals (at rate limit)
    for _ in 0..10 {
        proposer.generate_proposal(&pattern, &doctrines, &invariants, &guards)
            .await
            .expect("Should succeed within rate limit");
    }

    // 11th should fail
    let result = proposer.generate_proposal(&pattern, &doctrines, &invariants, &guards).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Rate limit"));
}

#[tokio::test]
async fn test_sector_specific_prompts() {
    let engine = PromptEngine::new();

    // Test Finance sector
    let finance_pattern = create_test_pattern(Sector::Finance);
    let finance_request = ProposalRequest {
        pattern: finance_pattern,
        current_snapshot_id: "snapshot-1".to_string(),
        doctrines: vec![],
        invariants: HardInvariants::default(),
        guard_profile: create_test_guards(Sector::Finance),
        performance_budget: PerformanceBudget::default(),
    };

    let finance_prompt = engine.build_full_prompt(&finance_request).unwrap();
    assert!(finance_prompt.contains("Finance"));
    assert!(finance_prompt.contains("Regulatory compliance"));

    // Test Healthcare sector
    let healthcare_pattern = create_test_pattern(Sector::Healthcare);
    let healthcare_request = ProposalRequest {
        pattern: healthcare_pattern,
        current_snapshot_id: "snapshot-2".to_string(),
        doctrines: vec![],
        invariants: HardInvariants::default(),
        guard_profile: create_test_guards(Sector::Healthcare),
        performance_budget: PerformanceBudget::default(),
    };

    let healthcare_prompt = engine.build_full_prompt(&healthcare_request).unwrap();
    assert!(healthcare_prompt.contains("Healthcare"));
    assert!(healthcare_prompt.contains("HIPAA"));
}

#[tokio::test]
async fn test_confidence_score_range() {
    // Create proposal with various confidence values
    let pattern = create_test_pattern(Sector::Finance);

    for conf in [0.0, 0.5, 0.85, 1.0] {
        let proposal = Proposal {
            id: format!("test-prop-{}", conf),
            pattern_id: pattern.id.clone(),
            llm_prompt: String::new(),
            llm_response: String::new(),
            delta_sigma: SigmaDiff::default(),
            reasoning: "Test".to_string(),
            confidence: conf,
            estimated_ticks: 6,
            doctrines_satisfied: vec![],
            invariants_satisfied: vec![],
            can_rollback: true,
            timestamp: chrono::Utc::now(),
            pattern: pattern.clone(),
        };

        // Confidence should be in valid range
        assert!(proposal.confidence >= 0.0);
        assert!(proposal.confidence <= 1.0);
    }
}

#[tokio::test]
async fn test_performance_budget_estimation() {
    let budget = PerformanceBudget::new(8, 5);

    let diff = SigmaDiff {
        added_classes: vec![
            ClassDefinition {
                uri: "test:Class1".to_string(),
                label: "Class 1".to_string(),
                subclass_of: "owl:Thing".to_string(),
                properties_required: vec![],
                properties_optional: vec![],
            },
            ClassDefinition {
                uri: "test:Class2".to_string(),
                label: "Class 2".to_string(),
                subclass_of: "owl:Thing".to_string(),
                properties_required: vec![],
                properties_optional: vec![],
            },
        ],
        added_properties: vec![PropertyDefinition {
            uri: "test:prop1".to_string(),
            label: "Property 1".to_string(),
            domain: "test:Class1".to_string(),
            range: "xsd:string".to_string(),
            required: true,
            cardinality: Cardinality::One,
        }],
        ..Default::default()
    };

    let cost = budget.estimate_cost(&diff);

    // 2 classes * 1.0 + 1 property * 0.5 = 2.5 → ceil = 3
    assert_eq!(cost, 3);
    assert!(budget.can_afford(&diff)); // 3 ≤ 3 remaining
}
