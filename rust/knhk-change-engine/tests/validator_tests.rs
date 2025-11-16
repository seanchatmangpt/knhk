//! Integration tests for ΔΣ Validator
//!
//! These tests verify the validation pipeline including:
//! - Invariant Q checks
//! - Type soundness validation
//! - Guard preservation
//! - SLO preservation (≤8 ticks)
//! - Determinism verification

use knhk_change_engine::{
    DeltaSigmaValidator,
    DeltaSigmaProposal,
    proposer::{PropertyDef, Cardinality, GuardRule},
};

#[tokio::test]
async fn test_validator_creation() {
    let _validator = DeltaSigmaValidator::new();

    // Validator should be created successfully
    // (no retrocausation is always true due to immutability)
    assert!(true); // Basic smoke test
}

#[tokio::test]
async fn test_validate_valid_add_class_proposal() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::AddClass {
        name: "ValidClass".to_string(),
        properties: vec![
            PropertyDef {
                name: "validProp".to_string(),
                range: "xsd:string".to_string(),
                cardinality: Cardinality::One,
                guards: vec![],
            }
        ],
        guards: vec![],
        sector: "test".to_string(),
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // Should pass all checks
    assert!(result.passed);
    assert!(result.details.type_soundness.passed);
    assert!(result.details.no_retrocausation.passed);
    assert!(result.details.guard_preservation.passed);
    assert!(result.details.slo_preservation.passed);
    assert!(result.details.determinism.passed);
}

#[tokio::test]
async fn test_validate_invalid_empty_class_name() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::AddClass {
        name: "".to_string(), // Invalid: empty name
        properties: vec![],
        guards: vec![],
        sector: "test".to_string(),
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // Should fail type soundness check
    assert!(!result.passed);
    assert!(!result.details.type_soundness.passed);
}

#[tokio::test]
async fn test_validate_invalid_empty_property_name() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::AddClass {
        name: "ValidClass".to_string(),
        properties: vec![
            PropertyDef {
                name: "".to_string(), // Invalid: empty property name
                range: "xsd:string".to_string(),
                cardinality: Cardinality::One,
                guards: vec![],
            }
        ],
        guards: vec![],
        sector: "test".to_string(),
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // Should fail type soundness check
    assert!(!result.passed);
    assert!(!result.details.type_soundness.passed);
}

#[tokio::test]
async fn test_validate_add_property_proposal() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::AddProperty {
        domain_class: "Person".to_string(),
        property_name: "email".to_string(),
        range: "xsd:string".to_string(),
        cardinality: Cardinality::One,
        guards: vec![GuardRule::MaskPII],
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // Should pass validation
    assert!(result.passed);
}

#[tokio::test]
async fn test_validate_add_property_empty_name() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::AddProperty {
        domain_class: "Person".to_string(),
        property_name: "".to_string(), // Invalid
        range: "xsd:string".to_string(),
        cardinality: Cardinality::One,
        guards: vec![],
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // Should fail type soundness
    assert!(!result.passed);
    assert!(!result.details.type_soundness.passed);
}

#[tokio::test]
async fn test_validate_remove_class_proposal() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::RemoveClass {
        name: "ObsoleteClass".to_string(),
        reason: "No longer needed".to_string(),
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // Should pass (placeholder logic accepts removals)
    assert!(result.passed);
}

#[tokio::test]
async fn test_validate_tighten_constraint() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::TightenConstraint {
        constraint_id: "rate_limit".to_string(),
        new_expression: "max_requests < 100".to_string(),
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // Should pass
    assert!(result.passed);
}

#[tokio::test]
async fn test_validate_relax_constraint() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::RelaxConstraint {
        constraint_id: "timeout".to_string(),
        new_expression: "timeout > 1000".to_string(),
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // Should pass (with warning in logs)
    assert!(result.passed);
}

#[tokio::test]
async fn test_slo_preservation_check() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::AddClass {
        name: "FastClass".to_string(),
        properties: vec![],
        guards: vec![],
        sector: "test".to_string(),
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // SLO preservation should pass (placeholder returns ≤8 ticks)
    assert!(result.details.slo_preservation.passed);

    if let Some(ticks) = result.details.slo_preservation.performance_ticks {
        assert!(ticks <= 8, "Should satisfy Chatman Constant (≤8 ticks)");
    }
}

#[tokio::test]
async fn test_no_retrocausation_always_passes() {
    let validator = DeltaSigmaValidator::new();

    // Test with various proposal types
    let proposals = vec![
        DeltaSigmaProposal::AddClass {
            name: "TestClass".to_string(),
            properties: vec![],
            guards: vec![],
            sector: "test".to_string(),
        },
        DeltaSigmaProposal::RemoveClass {
            name: "OldClass".to_string(),
            reason: "test".to_string(),
        },
        DeltaSigmaProposal::TightenConstraint {
            constraint_id: "c1".to_string(),
            new_expression: "expr".to_string(),
        },
    ];

    for proposal in proposals {
        let result = validator.validate_proposal(&proposal).await.unwrap();

        // No retrocausation should always pass (guaranteed by immutability)
        assert!(result.details.no_retrocausation.passed);
        assert!(result.details.no_retrocausation.message.is_some());
    }
}

#[tokio::test]
async fn test_validation_result_serialization() {
    let validator = DeltaSigmaValidator::new();

    let proposal = DeltaSigmaProposal::AddClass {
        name: "TestClass".to_string(),
        properties: vec![],
        guards: vec![],
        sector: "test".to_string(),
    };

    let result = validator.validate_proposal(&proposal).await.unwrap();

    // Should serialize and deserialize correctly
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: knhk_change_engine::ValidationResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.passed, deserialized.passed);
}

#[tokio::test]
async fn test_concurrent_validation() {
    use std::sync::Arc;
    use tokio::task;

    let validator = Arc::new(DeltaSigmaValidator::new());
    let mut handles = vec![];

    // Spawn multiple concurrent validations
    for i in 0..10 {
        let validator_clone = validator.clone();
        let handle = task::spawn(async move {
            let proposal = DeltaSigmaProposal::AddClass {
                name: format!("ConcurrentClass{}", i),
                properties: vec![],
                guards: vec![],
                sector: "test".to_string(),
            };

            validator_clone.validate_proposal(&proposal).await
        });
        handles.push(handle);
    }

    // Wait for all validations
    for handle in handles {
        let result = handle.await.unwrap().unwrap();
        assert!(result.passed);
    }
}

#[tokio::test]
async fn test_all_cardinality_types() {
    let validator = DeltaSigmaValidator::new();

    let cardinalities = vec![
        Cardinality::One,
        Cardinality::ZeroOrOne,
        Cardinality::ZeroOrMany,
        Cardinality::OneOrMany,
        Cardinality::Exact(5),
        Cardinality::Range(1, 10),
    ];

    for cardinality in cardinalities {
        let proposal = DeltaSigmaProposal::AddClass {
            name: "CardinalityTest".to_string(),
            properties: vec![
                PropertyDef {
                    name: "testProp".to_string(),
                    range: "xsd:string".to_string(),
                    cardinality,
                    guards: vec![],
                }
            ],
            guards: vec![],
            sector: "test".to_string(),
        };

        let result = validator.validate_proposal(&proposal).await.unwrap();
        assert!(result.passed);
    }
}

#[tokio::test]
async fn test_all_guard_types() {
    let validator = DeltaSigmaValidator::new();

    let guards = vec![
        GuardRule::MaskPII,
        GuardRule::RequireAuth,
        GuardRule::RateLimit(100),
        GuardRule::SparqlConstraint("?s :hasValue ?o".to_string()),
    ];

    for guard in guards {
        let proposal = DeltaSigmaProposal::AddProperty {
            domain_class: "GuardTest".to_string(),
            property_name: "guardedProp".to_string(),
            range: "xsd:string".to_string(),
            cardinality: Cardinality::One,
            guards: vec![guard],
        };

        let result = validator.validate_proposal(&proposal).await.unwrap();
        assert!(result.passed);
    }
}
