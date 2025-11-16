//! Integration tests for type-level state machine

use knhk_promotion::*;
use knhk_ontology::*;
use knhk_projections::*;
use std::sync::Arc;
use std::time::SystemTime;

fn create_production_ready_receipt(snapshot_id: SigmaSnapshotId) -> SigmaReceipt {
    let results = ValidationResults {
        static_checks_passed: true,
        dynamic_checks_passed: true,
        performance_checks_passed: true,
        invariants_q_preserved: true,
        errors: vec![],
        warnings: vec![],
    };

    SigmaReceipt::new(
        snapshot_id,
        None,
        "Test snapshot".to_string(),
        results,
        100,
    )
}

fn create_test_artifacts(snapshot_id: SigmaSnapshotId) -> CompiledProjections {
    CompiledProjections {
        snapshot_id,
        snapshot_hash: [0; 32],
        rust_models: knhk_projections::generators::RustModelsOutput {
            models_code: "// test".to_string(),
            hash: [0; 32],
        },
        openapi_spec: knhk_projections::generators::OpenApiOutput {
            openapi_spec: "{}".to_string(),
            hash: [0; 32],
        },
        hooks_config: knhk_projections::generators::HooksOutput {
            hooks_config: "{}".to_string(),
            hash: [0; 32],
        },
        markdown_docs: knhk_projections::generators::MarkdownOutput {
            markdown: "# Test".to_string(),
            hash: [0; 32],
        },
        otel_schema: knhk_projections::generators::OtelOutput {
            otel_schema: "{}".to_string(),
            hash: [0; 32],
        },
        compiled_at: SystemTime::now(),
    }
}

#[test]
fn test_type_level_state_transitions() {
    init_hot_path();

    let snapshot_id = [1u8; 32];
    let receipt = Arc::new(create_production_ready_receipt(snapshot_id));
    let artifacts = Arc::new(create_test_artifacts(snapshot_id));

    // Create guard in Preparing state
    let guard = PromotionGuard::new(snapshot_id, receipt, artifacts);
    assert!(guard.is_ok(), "Should create guard in Preparing state");

    // Note: This demonstrates type safety - you CANNOT do this:
    // let promoted = guard.promote(); // ← Compile error!
    // The promote() method only exists on PromotionGuard<Ready>
}

#[tokio::test]
async fn test_full_state_machine_workflow() {
    init_hot_path();

    let snapshot_id = [2u8; 32];
    let receipt = Arc::new(create_production_ready_receipt(snapshot_id));
    let artifacts = Arc::new(create_test_artifacts(snapshot_id));

    // State 1: Preparing
    let guard = PromotionGuard::new(snapshot_id, receipt, artifacts)
        .expect("Failed to create guard");

    // State 2: Ready
    let guard = guard.ready().await.expect("Failed to transition to Ready");

    // Verify we can access data in Ready state
    assert_eq!(guard.snapshot_id(), snapshot_id);
    assert!(guard.receipt().production_ready);

    // State 3: Promoted
    let promoted = guard.promote().expect("Failed to promote");

    // Verify promotion succeeded
    assert!(promoted.verify_promoted().is_ok());
    assert_eq!(get_current_snapshot(), snapshot_id);
}

#[test]
fn test_guard_rejects_invalid_receipt() {
    init_hot_path();

    let snapshot_id = [3u8; 32];

    // Create invalid receipt (invariants not preserved)
    let results = ValidationResults {
        static_checks_passed: true,
        dynamic_checks_passed: true,
        performance_checks_passed: true,
        invariants_q_preserved: false, // INVALID!
        errors: vec![],
        warnings: vec![],
    };

    let receipt = Arc::new(SigmaReceipt::new(
        snapshot_id,
        None,
        "Invalid snapshot".to_string(),
        results,
        100,
    ));

    let artifacts = Arc::new(create_test_artifacts(snapshot_id));

    // Should reject invalid receipt
    let guard = PromotionGuard::new(snapshot_id, receipt, artifacts);
    assert!(guard.is_err(), "Should reject invalid receipt");

    if let Err(e) = guard {
        assert!(matches!(e, PromotionError::InvariantsViolated(_)));
    }
}

#[tokio::test]
async fn test_concurrent_promotions() {
    init_hot_path();

    // Promote multiple snapshots sequentially
    for i in 1..=5 {
        let snapshot_id = [i as u8; 32];
        let receipt = Arc::new(create_production_ready_receipt(snapshot_id));
        let artifacts = Arc::new(create_test_artifacts(snapshot_id));

        let guard = PromotionGuard::new(snapshot_id, receipt, artifacts)
            .expect("Failed to create guard");

        let guard = guard.ready().await.expect("Failed to transition to Ready");

        guard.promote().expect("Failed to promote");

        assert_eq!(get_current_snapshot(), snapshot_id);
    }
}

#[test]
fn test_promotion_guard_clone() {
    init_hot_path();

    let snapshot_id = [4u8; 32];
    let receipt = Arc::new(create_production_ready_receipt(snapshot_id));
    let artifacts = Arc::new(create_test_artifacts(snapshot_id));

    let guard1 = PromotionGuard::new(snapshot_id, receipt, artifacts)
        .expect("Failed to create guard");

    // Guards should be cloneable
    let guard2 = guard1.clone();

    // Both guards should refer to the same snapshot
    drop(guard1);
    drop(guard2);
}
