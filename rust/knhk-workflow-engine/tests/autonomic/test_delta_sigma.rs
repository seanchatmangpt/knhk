//! Integration tests for ΔΣ Guarded Overlay Engine
//!
//! Tests the complete flow from overlay proposal to validation to execution.

use knhk_workflow_engine::autonomic::delta_sigma::{
    CompositionStrategy, DeltaSigma, OverlayChange, OverlayComposition, OverlayScope,
    ProofObligation, ProofPending, Proven, Unproven,
};
use knhk_workflow_engine::autonomic::overlay_validator::{
    ObligationResult, OverlayProof, OverlayValidator, TestResults, ValidationResult,
};
use knhk_workflow_engine::autonomic::{
    AdaptationPlan, Analysis, Analyzer, KnowledgeBase, Monitor, Planner,
};
use knhk_workflow_engine::patterns::{PatternId, PatternRegistry, RegisterAllExt};
use knhk_workflow_engine::WorkflowResult;
use std::sync::Arc;

/// Test complete overlay lifecycle: Unproven → ProofPending → Proven
#[tokio::test]
async fn test_overlay_lifecycle() -> WorkflowResult<()> {
    // Step 1: Create overlay proposal (Unproven state)
    let scope = OverlayScope::new()
        .with_pattern(PatternId::new(12)?)
        .with_guard("max_run_len".to_string());

    let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];

    let unproven = DeltaSigma::new(scope, changes)
        .with_metadata("source".to_string(), "test".to_string())
        .merge_change_scopes();

    // Verify unproven state
    assert_eq!(unproven.scope.patterns.len(), 4); // MI patterns 12-15
    assert!(!unproven.changes.is_empty());

    // Step 2: Generate proof obligations (Unproven → ProofPending)
    let proof_pending = unproven.generate_proof_obligations()?;

    // Verify proof obligations
    let obligations = proof_pending.proof_obligations();
    assert!(!obligations.is_empty());

    // Should have performance obligation
    assert!(obligations
        .iter()
        .any(|o| matches!(o, ProofObligation::ValidatePerformance { .. })));

    // Should have invariant obligation
    assert!(obligations
        .iter()
        .any(|o| matches!(o, ProofObligation::ValidateInvariants { .. })));

    // Step 3: Validate overlay (ProofPending → Proven)
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();
    let registry = Arc::new(registry);

    let kb = Arc::new(KnowledgeBase::new());
    let validator = OverlayValidator::new(registry, kb);

    let result = validator.validate(&proof_pending).await?;

    // Verify validation succeeded
    assert!(result.is_proven());
    assert!(result.proof().is_valid());

    // Step 4: Get proven overlay
    let proven = result.into_proven()?;

    // Verify proven overlay is ready for application
    assert!(!proven.changes().is_empty());
    assert!(proven.is_valid(300_000)); // Valid for 5 minutes

    Ok(())
}

/// Test proof obligation generation
#[test]
fn test_proof_obligations() -> WorkflowResult<()> {
    let scope = OverlayScope::new()
        .with_pattern(PatternId::new(1)?)
        .with_pattern(PatternId::new(2)?)
        .with_guard("max_run_len".to_string());

    let changes = vec![
        OverlayChange::ScaleMultiInstance { delta: 1 },
        OverlayChange::AdjustPerformance { target_ticks: 7 },
    ];

    let unproven = DeltaSigma::new(scope, changes);
    let proof_pending = unproven.generate_proof_obligations()?;

    let obligations = proof_pending.proof_obligations();

    // Should have multiple obligations
    assert!(obligations.len() >= 4);

    // Check obligation types
    let has_invariants = obligations
        .iter()
        .any(|o| matches!(o, ProofObligation::ValidateInvariants { .. }));
    let has_performance = obligations
        .iter()
        .any(|o| matches!(o, ProofObligation::ValidatePerformance { .. }));
    let has_guards = obligations
        .iter()
        .any(|o| matches!(o, ProofObligation::ValidateGuards { .. }));
    let has_slo = obligations
        .iter()
        .any(|o| matches!(o, ProofObligation::ValidateSLO { .. }));
    let has_doctrine = obligations
        .iter()
        .any(|o| matches!(o, ProofObligation::ValidateDoctrine { .. }));

    assert!(has_invariants);
    assert!(has_performance);
    assert!(has_guards);
    assert!(has_slo);
    assert!(has_doctrine);

    Ok(())
}

/// Test overlay validation with pattern registry
#[tokio::test]
async fn test_overlay_validation() -> WorkflowResult<()> {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();
    let registry = Arc::new(registry);

    let kb = Arc::new(KnowledgeBase::new());
    let validator = OverlayValidator::new(registry, kb);

    // Create overlay with valid patterns
    let scope = OverlayScope::new()
        .with_pattern(PatternId::new(1)?)
        .with_pattern(PatternId::new(2)?);

    let changes = vec![OverlayChange::AdjustPerformance { target_ticks: 6 }];

    let unproven = DeltaSigma::new(scope, changes);
    let proof_pending = unproven.generate_proof_obligations()?;

    // Validate
    let result = validator.validate(&proof_pending).await?;

    // Should succeed
    assert!(result.is_proven());

    let proof = result.proof();
    assert!(proof.is_valid());
    assert!(proof.failed_obligations().is_empty());

    Ok(())
}

/// Test overlay validation failure
#[tokio::test]
async fn test_overlay_validation_failure() -> WorkflowResult<()> {
    let registry = Arc::new(PatternRegistry::new()); // Empty registry
    let kb = Arc::new(KnowledgeBase::new());
    let validator = OverlayValidator::new(registry, kb);

    // Create overlay with unregistered pattern
    let scope = OverlayScope::new().with_pattern(PatternId::new(1)?);

    let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];

    let unproven = DeltaSigma::new(scope, changes);
    let proof_pending = unproven.generate_proof_obligations()?;

    // Validate
    let result = validator.validate(&proof_pending).await?;

    // Should fail (pattern not registered)
    assert!(!result.is_proven());

    let proof = result.proof();
    assert!(!proof.is_valid());
    assert!(!proof.failed_obligations().is_empty());

    Ok(())
}

/// Test overlay composition (parallel)
#[tokio::test]
async fn test_overlay_composition_parallel() -> WorkflowResult<()> {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();
    let registry = Arc::new(registry);

    let kb = Arc::new(KnowledgeBase::new());
    let validator = OverlayValidator::new(registry, kb);

    // Create two non-overlapping overlays
    let scope1 = OverlayScope::new().with_pattern(PatternId::new(1)?);
    let changes1 = vec![OverlayChange::AdjustPerformance { target_ticks: 6 }];
    let unproven1 = DeltaSigma::new(scope1, changes1);
    let proof_pending1 = unproven1.generate_proof_obligations()?;
    let result1 = validator.validate(&proof_pending1).await?;
    let proven1 = result1.into_proven()?;

    let scope2 = OverlayScope::new().with_pattern(PatternId::new(2)?);
    let changes2 = vec![OverlayChange::ScaleMultiInstance { delta: 1 }];
    let unproven2 = DeltaSigma::new(scope2, changes2);
    let proof_pending2 = unproven2.generate_proof_obligations()?;
    let result2 = validator.validate(&proof_pending2).await?;
    let proven2 = result2.into_proven()?;

    // Compose in parallel
    let composition = OverlayComposition::new(CompositionStrategy::Parallel)
        .add(proven1)
        .add(proven2);

    // Should validate successfully (no overlapping scopes)
    assert!(composition.validate().is_ok());

    Ok(())
}

/// Test overlay composition conflict detection
#[tokio::test]
async fn test_overlay_composition_conflict() -> WorkflowResult<()> {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();
    let registry = Arc::new(registry);

    let kb = Arc::new(KnowledgeBase::new());
    let validator = OverlayValidator::new(registry, kb);

    // Create two overlays with overlapping scope
    let scope1 = OverlayScope::new().with_pattern(PatternId::new(1)?);
    let changes1 = vec![OverlayChange::AdjustPerformance { target_ticks: 6 }];
    let unproven1 = DeltaSigma::new(scope1, changes1);
    let proof_pending1 = unproven1.generate_proof_obligations()?;
    let result1 = validator.validate(&proof_pending1).await?;
    let proven1 = result1.into_proven()?;

    let scope2 = OverlayScope::new().with_pattern(PatternId::new(1)?); // Same pattern!
    let changes2 = vec![OverlayChange::AdjustPerformance { target_ticks: 7 }];
    let unproven2 = DeltaSigma::new(scope2, changes2);
    let proof_pending2 = unproven2.generate_proof_obligations()?;
    let result2 = validator.validate(&proof_pending2).await?;
    let proven2 = result2.into_proven()?;

    // Compose in parallel (should fail due to overlap)
    let composition = OverlayComposition::new(CompositionStrategy::Parallel)
        .add(proven1)
        .add(proven2);

    // Should fail validation (overlapping scopes)
    assert!(composition.validate().is_err());

    Ok(())
}

/// Test integration with MAPE-K cycle
#[tokio::test]
async fn test_mapek_integration() -> WorkflowResult<()> {
    // Setup MAPE-K components
    let kb = Arc::new(KnowledgeBase::new());
    let monitor = Monitor::new(kb.clone());
    let analyzer = Analyzer::new(kb.clone());
    let planner = Planner::new(kb.clone());

    // Setup overlay validator
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();
    let registry = Arc::new(registry);
    let validator = OverlayValidator::new(registry, kb.clone());

    // Simulate MAPE-K cycle
    // 1. Monitor (collect metrics)
    // ... metrics collection ...

    // 2. Analyze (detect anomalies)
    let analysis = Analysis::new();
    // ... analysis logic ...

    // 3. Plan (generate adaptation - traditional approach)
    let plan_opt = planner.plan(&analysis).await?;

    // 4. Plan (generate overlay proposal - new approach)
    let scope = OverlayScope::new()
        .with_pattern(PatternId::new(12)?)
        .with_pattern(PatternId::new(13)?);

    let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];

    let overlay_unproven = DeltaSigma::new(scope, changes)
        .with_metadata("source".to_string(), "mapek_planner".to_string())
        .merge_change_scopes();

    // 5. Validate overlay
    let overlay_proof_pending = overlay_unproven.generate_proof_obligations()?;
    let validation_result = validator.validate(&overlay_proof_pending).await?;

    // 6. Execute (only if proven)
    if validation_result.is_proven() {
        let proven_overlay = validation_result.into_proven()?;
        // Apply proven overlay to system
        // ... execution logic ...
        assert!(!proven_overlay.changes().is_empty());
    }

    Ok(())
}

/// Property test: All valid overlays should pass validation
#[tokio::test]
async fn property_valid_overlays_pass_validation() -> WorkflowResult<()> {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();
    let registry = Arc::new(registry);

    let kb = Arc::new(KnowledgeBase::new());
    let validator = OverlayValidator::new(registry, kb);

    // Test various valid overlay configurations
    let test_cases = vec![
        (
            OverlayScope::new().with_pattern(PatternId::new(1)?),
            vec![OverlayChange::AdjustPerformance { target_ticks: 5 }],
        ),
        (
            OverlayScope::new().with_pattern(PatternId::new(12)?),
            vec![OverlayChange::ScaleMultiInstance { delta: 1 }],
        ),
        (
            OverlayScope::new()
                .with_pattern(PatternId::new(1)?)
                .with_pattern(PatternId::new(2)?),
            vec![OverlayChange::AdjustResources {
                resource: "cpu".to_string(),
                multiplier: 1.1,
            }],
        ),
    ];

    for (scope, changes) in test_cases {
        let unproven = DeltaSigma::new(scope, changes);
        let proof_pending = unproven.generate_proof_obligations()?;
        let result = validator.validate(&proof_pending).await?;

        assert!(
            result.is_proven(),
            "Valid overlay should pass validation"
        );
    }

    Ok(())
}

/// Property test: Overlays with invalid patterns should fail
#[tokio::test]
async fn property_invalid_patterns_fail_validation() -> WorkflowResult<()> {
    let registry = Arc::new(PatternRegistry::new()); // Empty registry
    let kb = Arc::new(KnowledgeBase::new());
    let validator = OverlayValidator::new(registry, kb);

    // Create overlay with pattern that's not registered
    let scope = OverlayScope::new().with_pattern(PatternId::new(1)?);
    let changes = vec![OverlayChange::ScaleMultiInstance { delta: 1 }];

    let unproven = DeltaSigma::new(scope, changes);
    let proof_pending = unproven.generate_proof_obligations()?;
    let result = validator.validate(&proof_pending).await?;

    assert!(
        !result.is_proven(),
        "Overlay with unregistered pattern should fail"
    );

    Ok(())
}

/// Test proof caching
#[tokio::test]
async fn test_proof_caching() -> WorkflowResult<()> {
    let mut registry = PatternRegistry::new();
    registry.register_all_patterns();
    let registry = Arc::new(registry);

    let kb = Arc::new(KnowledgeBase::new());
    let validator = OverlayValidator::new(registry, kb);

    // Create overlay
    let scope = OverlayScope::new().with_pattern(PatternId::new(1)?);
    let changes = vec![OverlayChange::AdjustPerformance { target_ticks: 6 }];
    let unproven = DeltaSigma::new(scope, changes);
    let overlay_id = unproven.id;

    // First validation
    let proof_pending = unproven.generate_proof_obligations()?;
    let result1 = validator.validate(&proof_pending).await?;
    assert!(result1.is_proven());

    // Check cache
    let cached_proof = validator.get_cached_proof(&overlay_id).await;
    assert!(cached_proof.is_some());
    assert!(cached_proof.unwrap().is_valid());

    Ok(())
}

/// Test validation effort estimation
#[test]
fn test_validation_effort() -> WorkflowResult<()> {
    // Low effort: few patterns, few changes
    let scope1 = OverlayScope::new().with_pattern(PatternId::new(1)?);
    let changes1 = vec![OverlayChange::AdjustPerformance { target_ticks: 6 }];
    let unproven1 = DeltaSigma::new(scope1, changes1);
    let proof_pending1 = unproven1.generate_proof_obligations()?;
    let effort1 = proof_pending1.validation_effort();

    // Medium/High effort: many patterns, many changes
    let scope2 = OverlayScope::new()
        .with_pattern(PatternId::new(1)?)
        .with_pattern(PatternId::new(2)?)
        .with_pattern(PatternId::new(3)?)
        .with_pattern(PatternId::new(4)?);
    let changes2 = vec![
        OverlayChange::AdjustPerformance { target_ticks: 6 },
        OverlayChange::ScaleMultiInstance { delta: 2 },
        OverlayChange::AdjustResources {
            resource: "cpu".to_string(),
            multiplier: 1.2,
        },
    ];
    let unproven2 = DeltaSigma::new(scope2, changes2);
    let proof_pending2 = unproven2.generate_proof_obligations()?;
    let effort2 = proof_pending2.validation_effort();

    // More complex overlay should have higher or equal effort
    assert!(
        effort2 as u8 >= effort1 as u8,
        "More complex overlay should require more effort"
    );

    Ok(())
}
