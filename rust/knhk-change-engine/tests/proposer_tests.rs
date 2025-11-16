//! Integration tests for ΔΣ Proposer
//!
//! These tests verify the proposal generation logic including:
//! - Policy rule triggering
//! - Proposal generation based on patterns
//! - Custom policy rules

use std::sync::Arc;
use parking_lot::RwLock;
use knhk_change_engine::{
    DeltaSigmaProposer,
    PolicyRule,
    DeltaSigmaProposal,
    DetectedPatterns,
    pattern_miner::{SchemaMismatch, Triple, GuardViolation, PerfRegression},
};

#[tokio::test]
async fn test_propose_with_no_patterns() {
    let patterns = Arc::new(RwLock::new(DetectedPatterns::default()));
    let proposer = DeltaSigmaProposer::new(patterns);

    let proposals = proposer.propose_delta_sigma().await.unwrap();

    // Should generate no proposals with no patterns
    assert_eq!(proposals.len(), 0);
}

#[tokio::test]
async fn test_propose_add_class_for_schema_mismatches() {
    let mut detected = DetectedPatterns::default();

    // Add 12 schema mismatches with same predicate (threshold is 10)
    for i in 0..12 {
        detected.schema_mismatches.push(SchemaMismatch {
            triple: Triple {
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

    // Should propose adding a class
    assert!(proposals.len() > 0);

    let add_class_proposals: Vec<_> = proposals.iter()
        .filter_map(|p| match p {
            DeltaSigmaProposal::AddClass { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();

    assert!(add_class_proposals.len() > 0);
}

#[tokio::test]
async fn test_propose_pii_masking_for_pii_patterns() {
    let mut detected = DetectedPatterns::default();

    // Add schema mismatch with PII reason
    detected.schema_mismatches.push(SchemaMismatch {
        triple: Triple {
            subject: "user1".to_string(),
            predicate: "ssn".to_string(),
            object: "123-45-6789".to_string(),
        },
        reason: "PII detected".to_string(),
        frequency: 5,
    });

    let patterns = Arc::new(RwLock::new(detected));
    let proposer = DeltaSigmaProposer::new(patterns);

    let proposals = proposer.propose_delta_sigma().await.unwrap();

    // Should propose adding property with PII masking guard
    assert!(proposals.len() > 0);

    let has_pii_guard = proposals.iter().any(|p| match p {
        DeltaSigmaProposal::AddProperty { guards, .. } => {
            guards.iter().any(|g| matches!(g, knhk_change_engine::proposer::GuardRule::MaskPII))
        }
        _ => false,
    });

    assert!(has_pii_guard);
}

#[tokio::test]
async fn test_propose_tighten_constraint_for_guard_violations() {
    let mut detected = DetectedPatterns::default();

    // Add guard violations
    detected.guard_violations.push(GuardViolation {
        guard_name: "rate_limit".to_string(),
        near_miss_count: 5,
        affected_subjects: vec!["api1".to_string(), "api2".to_string()],
    });

    let patterns = Arc::new(RwLock::new(detected));
    let proposer = DeltaSigmaProposer::new(patterns);

    let proposals = proposer.propose_delta_sigma().await.unwrap();

    // Should propose tightening constraint
    assert!(proposals.len() > 0);

    let has_tighten = proposals.iter().any(|p| matches!(p, DeltaSigmaProposal::TightenConstraint { .. }));
    assert!(has_tighten);
}

#[tokio::test]
async fn test_propose_relax_constraint_for_performance_regressions() {
    let mut detected = DetectedPatterns::default();

    // Add performance regression
    detected.performance_regressions.push(PerfRegression {
        operator: "slow_query".to_string(),
        observed_latency_ticks: 20,
        expected_latency_ticks: 8,
        regression_factor: 2.5,
    });

    let patterns = Arc::new(RwLock::new(detected));
    let proposer = DeltaSigmaProposer::new(patterns);

    let proposals = proposer.propose_delta_sigma().await.unwrap();

    // Should propose relaxing constraint for performance
    assert!(proposals.len() > 0);

    let has_relax = proposals.iter().any(|p| matches!(p, DeltaSigmaProposal::RelaxConstraint { .. }));
    assert!(has_relax);
}

#[tokio::test]
async fn test_custom_policy_rule() {
    let patterns = Arc::new(RwLock::new(DetectedPatterns::default()));
    let mut proposer = DeltaSigmaProposer::new(patterns.clone());

    // Add custom policy rule
    proposer.add_policy_rule(PolicyRule {
        name: "custom_rule".to_string(),
        condition: Box::new(|_| true), // Always trigger
        proposal_fn: Box::new(|_| {
            vec![DeltaSigmaProposal::AddClass {
                name: "CustomClass".to_string(),
                properties: vec![],
                guards: vec![],
                sector: "custom".to_string(),
            }]
        }),
    });

    let proposals = proposer.propose_delta_sigma().await.unwrap();

    // Should include proposal from custom rule
    assert!(proposals.len() > 0);

    let has_custom = proposals.iter().any(|p| match p {
        DeltaSigmaProposal::AddClass { name, .. } => name == "CustomClass",
        _ => false,
    });

    assert!(has_custom);
}

#[tokio::test]
async fn test_multiple_policy_rules_trigger() {
    let mut detected = DetectedPatterns::default();

    // Add patterns that trigger multiple rules
    // 1. Schema mismatches (>10)
    for i in 0..12 {
        detected.schema_mismatches.push(SchemaMismatch {
            triple: Triple {
                subject: format!("s{}", i),
                predicate: "prop".to_string(),
                object: format!("o{}", i),
            },
            reason: "test".to_string(),
            frequency: 1,
        });
    }

    // 2. Guard violations (>3)
    detected.guard_violations.push(GuardViolation {
        guard_name: "test_guard".to_string(),
        near_miss_count: 5,
        affected_subjects: vec!["s1".to_string()],
    });

    let patterns = Arc::new(RwLock::new(detected));
    let proposer = DeltaSigmaProposer::new(patterns);

    let proposals = proposer.propose_delta_sigma().await.unwrap();

    // Should have proposals from multiple rules
    assert!(proposals.len() > 1);
}

#[tokio::test]
async fn test_proposal_serialization() {
    use knhk_change_engine::proposer::{PropertyDef, Cardinality};

    let proposal = DeltaSigmaProposal::AddClass {
        name: "TestClass".to_string(),
        properties: vec![
            PropertyDef {
                name: "prop1".to_string(),
                range: "xsd:string".to_string(),
                cardinality: Cardinality::One,
                guards: vec![],
            }
        ],
        guards: vec![],
        sector: "test".to_string(),
    };

    // Should serialize and deserialize correctly
    let json = serde_json::to_string(&proposal).unwrap();
    let deserialized: DeltaSigmaProposal = serde_json::from_str(&json).unwrap();

    match deserialized {
        DeltaSigmaProposal::AddClass { name, .. } => assert_eq!(name, "TestClass"),
        _ => panic!("Wrong proposal type"),
    }
}
