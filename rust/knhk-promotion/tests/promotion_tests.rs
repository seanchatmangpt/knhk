//! Integration tests for promotion pipeline

use knhk_promotion::*;
use knhk_ontology::*;
use knhk_projections::*;
use std::sync::Arc;
use std::time::SystemTime;

fn create_test_snapshot() -> SigmaSnapshot {
    let mut store = TripleStore::new();
    store.add(Triple::new("company1", "rdf:type", "Company"));
    store.add(Triple::new("company1", "sector", "Technology"));
    store.add(Triple::new("company1", "revenue", "1000000"));

    let metadata = SnapshotMetadata {
        created_by: "test".to_string(),
        description: "Test snapshot".to_string(),
        created_at: SystemTime::now(),
        sector: Some("Technology".to_string()),
    };

    let mut snapshot = SigmaSnapshot::new(None, store, metadata)
        .expect("Failed to create snapshot");

    // Add production-ready receipt
    let results = ValidationResults {
        static_checks_passed: true,
        dynamic_checks_passed: true,
        performance_checks_passed: true,
        invariants_q_preserved: true,
        errors: vec![],
        warnings: vec![],
    };

    let receipt = SigmaReceipt::new(
        snapshot.id,
        None,
        "Test snapshot".to_string(),
        results,
        100,
    );

    snapshot.validation_receipt = Some(receipt);
    snapshot
}

#[tokio::test]
async fn test_full_promotion_pipeline() {
    init_hot_path();

    let store = Arc::new(SnapshotStore::new());
    let compiler = Arc::new(ProjectionCompiler::new());
    let pipeline = PromotionPipeline::new(store.clone(), compiler);

    let snapshot = create_test_snapshot();
    let snapshot_id = snapshot.id;

    // Add snapshot to store
    store.add_snapshot(snapshot);

    // Promote
    let result = pipeline.promote_snapshot(snapshot_id).await;

    assert!(result.is_ok(), "Promotion should succeed");

    let result = result.unwrap();
    assert_eq!(result.snapshot_id, snapshot_id);
    assert!(result.total_duration.as_millis() > 0);

    // Verify current snapshot was updated
    assert_eq!(get_current_snapshot(), snapshot_id);
}

#[tokio::test]
async fn test_promotion_telemetry() {
    init_hot_path();

    let store = Arc::new(SnapshotStore::new());
    let compiler = Arc::new(ProjectionCompiler::new());
    let pipeline = PromotionPipeline::new(store.clone(), compiler);

    let snapshot = create_test_snapshot();
    let snapshot_id = snapshot.id;

    store.add_snapshot(snapshot);

    // Promote
    pipeline.promote_snapshot(snapshot_id).await.expect("Failed to promote");

    // Check telemetry
    let telemetry = pipeline.telemetry();
    let stats = telemetry.stats();

    assert_eq!(stats.total_promotions, 1);
    assert_eq!(stats.successful_promotions, 1);
    assert_eq!(stats.failed_promotions, 0);
}

#[tokio::test]
async fn test_promotion_failure_not_production_ready() {
    init_hot_path();

    let store = Arc::new(SnapshotStore::new());
    let compiler = Arc::new(ProjectionCompiler::new());
    let pipeline = PromotionPipeline::new(store.clone(), compiler);

    let mut snapshot = create_test_snapshot();

    // Make receipt indicate NOT production-ready
    if let Some(receipt) = &mut snapshot.validation_receipt {
        let mut bad_results = receipt.validation_results.clone();
        bad_results.invariants_q_preserved = false;

        let bad_receipt = SigmaReceipt::new(
            snapshot.id,
            None,
            "Bad snapshot".to_string(),
            bad_results,
            100,
        );

        snapshot.validation_receipt = Some(bad_receipt);
    }

    let snapshot_id = snapshot.id;
    store.add_snapshot(snapshot);

    // Promotion should fail
    let result = pipeline.promote_snapshot(snapshot_id).await;
    assert!(result.is_err(), "Should reject non-production-ready snapshot");
}

#[tokio::test]
async fn test_multiple_sequential_promotions() {
    init_hot_path();

    let store = Arc::new(SnapshotStore::new());
    let compiler = Arc::new(ProjectionCompiler::new());
    let pipeline = PromotionPipeline::new(store.clone(), compiler);

    // Promote multiple snapshots
    let mut snapshot_ids = Vec::new();

    for i in 0..3 {
        let mut snapshot = create_test_snapshot();

        // Make each snapshot unique
        if let Some(receipt) = &mut snapshot.validation_receipt {
            let new_receipt = SigmaReceipt::new(
                snapshot.id,
                None,
                format!("Snapshot {}", i),
                receipt.validation_results.clone(),
                100 + i as u64,
            );
            snapshot.validation_receipt = Some(new_receipt);
        }

        let id = snapshot.id;
        snapshot_ids.push(id);

        store.add_snapshot(snapshot);

        let result = pipeline.promote_snapshot(id).await;
        assert!(result.is_ok(), "Promotion {} should succeed", i);

        // Verify current snapshot is the latest
        assert_eq!(get_current_snapshot(), id);
    }

    // Verify telemetry tracked all promotions
    let stats = pipeline.telemetry().stats();
    assert_eq!(stats.total_promotions, 3);
    assert_eq!(stats.successful_promotions, 3);
}

#[test]
fn test_promotion_result_fields() {
    let result = PromotionResult {
        snapshot_id: [1; 32],
        promoted_at: SystemTime::now(),
        total_duration: std::time::Duration::from_millis(123),
        new_epoch: 5,
    };

    assert_eq!(result.snapshot_id, [1; 32]);
    assert_eq!(result.total_duration.as_millis(), 123);
    assert_eq!(result.new_epoch, 5);
}
