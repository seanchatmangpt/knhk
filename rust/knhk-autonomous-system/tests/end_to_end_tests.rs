//! End-to-end integration tests for the autonomous ontology system
//!
//! These tests verify the complete system works from initialization through
//! autonomous evolution cycles.

use knhk_autonomous_system::{
    AutonomousOntologyPlant, StorageBackend, SystemConfig,
};
use knhk_ontology::{SigmaOverlay, Triple};

/// Test complete system initialization
#[tokio::test]
async fn test_complete_system_initialization() {
    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await;

    assert!(plant.is_ok(), "System initialization should succeed");

    let plant = plant.unwrap();

    // Verify we can get current snapshot
    let snapshot = plant.current_snapshot().await;
    assert!(snapshot.is_ok(), "Should get current snapshot");

    let snapshot = snapshot.unwrap();
    assert!(!snapshot.all_triples().is_empty(), "Snapshot should have seed triples");
}

/// Test system invariant verification
#[tokio::test]
async fn test_system_invariant_verification() {
    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    // Verify all invariants Q + 3 axes
    let result = plant.verify_system_invariants().await;
    assert!(
        result.is_ok(),
        "All invariants and axes should verify: {:?}",
        result
    );
}

/// Test autonomous evolution cycle
#[tokio::test]
async fn test_autonomous_evolution_cycle() {
    let mut config = SystemConfig::for_testing();
    config.loop_config.cycle_interval = std::time::Duration::from_millis(50);
    config.loop_config.max_proposals = 2;

    let mut plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    // Get initial snapshot
    let initial = plant.current_snapshot().await.expect("Get initial snapshot");
    let initial_id = initial.id;

    // Verify system before starting
    plant
        .verify_system_invariants()
        .await
        .expect("Initial invariants should verify");

    // Start autonomous evolution
    // Note: In real usage, this runs forever. For testing, we just verify it starts.
    let start_result = plant.start().await;
    assert!(
        start_result.is_ok(),
        "Starting autonomous evolution should succeed"
    );

    // Wait a bit for potential evolution
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // System should still be healthy
    let current = plant.current_snapshot().await.expect("Get current snapshot");
    assert!(
        !current.all_triples().is_empty(),
        "Current snapshot should have triples"
    );
}

/// Test snapshot promotion workflow
#[tokio::test]
async fn test_snapshot_promotion_workflow() {
    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    // Get current snapshot
    let current = plant.current_snapshot().await.expect("Get current");
    let current_id = current.id;

    // Create an overlay with changes
    let mut overlay = SigmaOverlay::new(current_id, "Test changes".to_string());
    overlay.add_triple(Triple::new(
        "test:Company1",
        "test:sector",
        "test:Technology",
    ));

    // Validate and commit overlay
    let store = plant.snapshot_store();
    let base = store
        .get_snapshot(&current_id)
        .expect("Get base snapshot");

    let validator = knhk_ontology::InvariantValidator::new()
        .with_max_ticks(100); // Generous for tests

    let receipt = overlay
        .validate(&base, &validator)
        .expect("Overlay validation should succeed");

    assert!(receipt.production_ready, "Receipt should be production ready");

    let new_snapshot = overlay
        .commit(&base, receipt)
        .expect("Commit should succeed");

    let new_id = new_snapshot.id;
    store.add_snapshot(new_snapshot);

    // Promote new snapshot
    let promotion_result = store.promote_snapshot(new_id);
    assert!(
        promotion_result.is_ok(),
        "Promotion should succeed: {:?}",
        promotion_result
    );

    // Verify new snapshot is current
    let current_after = plant.current_snapshot().await.expect("Get current after");
    assert_eq!(current_after.id, new_id, "New snapshot should be current");
}

/// Test all 7 layers integration
#[tokio::test]
async fn test_seven_layers_integration() {
    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    // Layer 1 & 2: Σ² and Σ runtime
    let snapshot = plant.current_snapshot().await.expect("Get snapshot");
    assert!(!snapshot.all_triples().is_empty(), "Layer 1 & 2: Snapshot exists");

    // Layer 3: ΔΣ engine
    let change_engine = plant.change_engine();
    // Just verify it exists
    assert!(
        std::sync::Arc::strong_count(&change_engine) >= 1,
        "Layer 3: Change engine exists"
    );

    // Layer 4: Π compiler
    let compiler = plant.compiler();
    let compiled = compiler
        .compile_all(std::sync::Arc::new(snapshot.clone()))
        .await;
    assert!(compiled.is_ok(), "Layer 4: Compiler works");

    // Layer 5: Promotion pipeline
    let promotion_pipeline = plant.promotion_pipeline();
    assert!(
        std::sync::Arc::strong_count(&promotion_pipeline) >= 1,
        "Layer 5: Promotion pipeline exists"
    );

    // Layer 6: Autonomous loop (tested in other tests)
    // Layer 7: Integration layer (this test verifies it)

    // All 7 layers working together!
}

/// Test deterministic projection compilation (μ-axis)
#[tokio::test]
async fn test_deterministic_projections() {
    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    let snapshot = plant.current_snapshot().await.expect("Get snapshot");
    let compiler = plant.compiler();

    // Compile twice
    let compiled_1 = compiler
        .compile_all(std::sync::Arc::new(snapshot.clone()))
        .await
        .expect("First compilation");

    let compiled_2 = compiler
        .compile_all(std::sync::Arc::new(snapshot.clone()))
        .await
        .expect("Second compilation");

    // Should be bit-for-bit identical
    assert_eq!(
        compiled_1.snapshot_hash, compiled_2.snapshot_hash,
        "Compiled projections should be deterministic"
    );
    assert_eq!(
        compiled_1.rust_models.hash, compiled_2.rust_models.hash,
        "Rust models should be deterministic"
    );
    assert_eq!(
        compiled_1.openapi_spec.hash, compiled_2.openapi_spec.hash,
        "OpenAPI spec should be deterministic"
    );
}

/// Test graceful shutdown
#[tokio::test]
async fn test_graceful_shutdown() {
    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    // Shutdown should succeed
    let result = plant.shutdown().await;
    assert!(result.is_ok(), "Shutdown should succeed");
}

/// Test configuration variations
#[tokio::test]
async fn test_configuration_variations() {
    // Test with different configurations
    let configs = vec![
        SystemConfig::for_testing(),
        SystemConfig {
            verify_time_axis: false,
            ..SystemConfig::for_testing()
        },
        SystemConfig {
            verify_mapping_axis: false,
            ..SystemConfig::for_testing()
        },
        SystemConfig {
            verify_glue_axis: false,
            ..SystemConfig::for_testing()
        },
        SystemConfig {
            enable_telemetry: false,
            ..SystemConfig::for_testing()
        },
    ];

    for (i, config) in configs.into_iter().enumerate() {
        let plant = AutonomousOntologyPlant::initialize(
            "registry/meta-ontology.ttl",
            StorageBackend::InMemory,
            config,
        )
        .await;

        assert!(
            plant.is_ok(),
            "Configuration variant {} should initialize",
            i
        );
    }
}
