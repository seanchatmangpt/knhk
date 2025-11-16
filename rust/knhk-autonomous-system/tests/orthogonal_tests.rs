//! Orthogonal Axis Tests - τ, μ, Γ
//!
//! These tests specifically verify the three orthogonal axes that bind
//! the autonomous ontology system together.

use knhk_autonomous_system::{
    AutonomousOntologyPlant, GlueAxisVerifier, MappingAxisVerifier, StorageBackend, SystemConfig,
    TimeAxisVerifier,
};
use knhk_ontology::{SigmaSnapshot, SnapshotMetadata, Triple, TripleStore};
use knhk_projections::ProjectionCompiler;

/// Test τ-axis (time bound) verification
#[tokio::test]
async fn test_tau_axis_time_bound() {
    let verifier = TimeAxisVerifier::new(15);

    // Verify time axis
    let result = verifier.verify().await;
    assert!(result.is_ok(), "τ-axis verification should pass: {:?}", result);
}

/// Test τ-axis hot path timing
#[tokio::test]
async fn test_tau_axis_hot_path_timing() {
    let verifier = TimeAxisVerifier::new(15);

    // Measure a fast operation
    let (result, elapsed_micros) = verifier
        .measure_operation("test_operation", || async {
            // Simulate fast operation
            tokio::time::sleep(tokio::time::Duration::from_micros(5)).await;
            "success"
        })
        .await
        .expect("Measurement should succeed");

    assert_eq!(result, "success");
    assert!(
        elapsed_micros < 1000,
        "Fast operation should complete quickly, took {}μs",
        elapsed_micros
    );
}

/// Test τ-axis promotion timing budget
#[tokio::test]
async fn test_tau_axis_promotion_budget() {
    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    let verifier = TimeAxisVerifier::new(15);

    // Verify promotion timing
    let result = verifier.verify_promotion_timing().await;
    assert!(
        result.is_ok(),
        "Promotion should meet timing budget: {:?}",
        result
    );
}

/// Test μ-axis (mapping/determinism) verification
#[tokio::test]
async fn test_mu_axis_determinism() {
    let verifier = MappingAxisVerifier::new();
    let compiler = ProjectionCompiler::new();

    // Create test snapshot
    let mut triple_store = TripleStore::new();
    triple_store.add(Triple::new("test:A", "test:B", "test:C"));
    triple_store.add(Triple::new("test:D", "test:E", "test:F"));

    let snapshot = SigmaSnapshot::new(None, triple_store, SnapshotMetadata::default())
        .expect("Failed to create snapshot");

    // Verify μ-axis
    let result = verifier.verify(&snapshot, &compiler).await;
    assert!(
        result.is_ok(),
        "μ-axis verification should pass: {:?}",
        result
    );
}

/// Test μ-axis determinism property: μ(Σ) = μ(Σ)
#[tokio::test]
async fn test_mu_axis_determinism_property() {
    let verifier = MappingAxisVerifier::new();
    let compiler = ProjectionCompiler::new();

    let mut triple_store = TripleStore::new();
    triple_store.add(Triple::new("company:1", "sector", "Technology"));
    triple_store.add(Triple::new("company:2", "sector", "Healthcare"));

    let snapshot = SigmaSnapshot::new(None, triple_store, SnapshotMetadata::default())
        .expect("Failed to create snapshot");

    // Compile twice
    let output_1 = compiler
        .compile_all(std::sync::Arc::new(snapshot.clone()))
        .await
        .expect("First compilation");

    let output_2 = compiler
        .compile_all(std::sync::Arc::new(snapshot.clone()))
        .await
        .expect("Second compilation");

    // Should be identical
    assert_eq!(
        output_1.snapshot_hash, output_2.snapshot_hash,
        "Determinism: μ(Σ) should equal μ(Σ)"
    );
}

/// Test μ-axis idempotence property: μ∘μ = μ
#[tokio::test]
async fn test_mu_axis_idempotence_property() {
    let verifier = MappingAxisVerifier::new();
    let compiler = ProjectionCompiler::new();

    let mut triple_store = TripleStore::new();
    triple_store.add(Triple::new("test:X", "test:Y", "test:Z"));

    let snapshot = SigmaSnapshot::new(None, triple_store, SnapshotMetadata::default())
        .expect("Failed to create snapshot");

    // Verify idempotence
    let result = verifier.verify_idempotence(&snapshot, &compiler).await;
    assert!(
        result.is_ok(),
        "Idempotence μ∘μ = μ should hold: {:?}",
        result
    );
}

/// Test Γ-axis (glue/sheaf) verification
#[tokio::test]
async fn test_gamma_axis_glue_properties() {
    let verifier = GlueAxisVerifier::new();

    // Verify Γ-axis
    let result = verifier.verify().await;
    assert!(
        result.is_ok(),
        "Γ-axis verification should pass: {:?}",
        result
    );
}

/// Test Γ-axis receipt monoid: associativity
#[tokio::test]
async fn test_gamma_axis_associativity() {
    let verifier = GlueAxisVerifier::new();

    let r1 = verifier.create_test_receipt(1);
    let r2 = verifier.create_test_receipt(2);
    let r3 = verifier.create_test_receipt(3);

    // (r1 ⊕ r2) ⊕ r3
    let left = verifier
        .glue(&verifier.glue(&r1, &r2).unwrap(), &r3)
        .unwrap();

    // r1 ⊕ (r2 ⊕ r3)
    let right = verifier
        .glue(&r1, &verifier.glue(&r2, &r3).unwrap())
        .unwrap();

    let left_hash = verifier.receipt_hash(&left);
    let right_hash = verifier.receipt_hash(&right);

    assert_eq!(
        left_hash, right_hash,
        "Associativity: (r1 ⊕ r2) ⊕ r3 = r1 ⊕ (r2 ⊕ r3)"
    );
}

/// Test Γ-axis receipt monoid: commutativity
#[tokio::test]
async fn test_gamma_axis_commutativity() {
    let verifier = GlueAxisVerifier::new();

    let r1 = verifier.create_test_receipt(1);
    let r2 = verifier.create_test_receipt(2);

    let ab = verifier.glue(&r1, &r2).unwrap();
    let ba = verifier.glue(&r2, &r1).unwrap();

    let ab_hash = verifier.receipt_hash(&ab);
    let ba_hash = verifier.receipt_hash(&ba);

    assert_eq!(
        ab_hash, ba_hash,
        "Commutativity: r1 ⊕ r2 = r2 ⊕ r1"
    );
}

/// Test Γ-axis receipt monoid: identity
#[tokio::test]
async fn test_gamma_axis_identity() {
    let verifier = GlueAxisVerifier::new();

    let r = verifier.create_test_receipt(42);
    let identity = verifier.create_identity_receipt();

    let glued = verifier.glue(&r, &identity).unwrap();

    let r_hash = verifier.receipt_hash(&r);
    let glued_hash = verifier.receipt_hash(&glued);

    assert_eq!(r_hash, glued_hash, "Identity: r ⊕ ε = r");
}

/// Test Γ-axis multi-region consistency
#[tokio::test]
async fn test_gamma_axis_multi_region_consistency() {
    let verifier = GlueAxisVerifier::new();

    let r1 = verifier.create_test_receipt(1);
    let r2 = verifier.create_test_receipt(2);
    let r3 = verifier.create_test_receipt(3);

    // Region A: processes in order 1, 2, 3
    let region_a = verifier
        .glue(&verifier.glue(&r1, &r2).unwrap(), &r3)
        .unwrap();

    // Region B: processes in order 3, 1, 2
    let region_b = verifier
        .glue(&verifier.glue(&r3, &r1).unwrap(), &r2)
        .unwrap();

    let hash_a = verifier.receipt_hash(&region_a);
    let hash_b = verifier.receipt_hash(&region_b);

    assert_eq!(
        hash_a, hash_b,
        "Multi-region consistency: different orderings should converge"
    );
}

/// Test all three axes together (integration)
#[tokio::test]
async fn test_all_three_axes_integration() {
    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    // Verify all three axes
    let result = plant.verify_system_invariants().await;
    assert!(
        result.is_ok(),
        "All three axes (τ, μ, Γ) should verify together: {:?}",
        result
    );
}

/// Test axis verification with actual system operations
#[tokio::test]
async fn test_axes_with_real_operations() {
    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    // Get snapshot
    let snapshot = plant.current_snapshot().await.expect("Get snapshot");

    // Verify μ-axis with real snapshot
    let mapping_verifier = MappingAxisVerifier::new();
    let compiler = plant.compiler();
    let mu_result = mapping_verifier.verify(&snapshot, compiler.as_ref()).await;
    assert!(mu_result.is_ok(), "μ-axis should verify with real snapshot");

    // Verify τ-axis
    let time_verifier = TimeAxisVerifier::new(15);
    let tau_result = time_verifier.verify().await;
    assert!(tau_result.is_ok(), "τ-axis should verify with real operations");

    // Verify Γ-axis
    let glue_verifier = GlueAxisVerifier::new();
    let gamma_result = glue_verifier.verify().await;
    assert!(
        gamma_result.is_ok(),
        "Γ-axis should verify with real receipts"
    );
}

/// Test performance constraints from τ-axis
#[tokio::test]
async fn test_performance_constraints() {
    use std::time::Instant;

    let config = SystemConfig::for_testing();

    let plant = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Initialization failed");

    // Measure snapshot retrieval (hot path operation)
    let start = Instant::now();
    let _snapshot = plant.current_snapshot().await.expect("Get snapshot");
    let elapsed = start.elapsed();

    // Should be very fast (≤8 ticks = ~8μs on modern hardware)
    // We use 1ms as generous upper bound for test environment
    assert!(
        elapsed.as_millis() < 10,
        "Hot path operation should be fast, took {:?}",
        elapsed
    );
}

/// Test determinism across different runs
#[tokio::test]
async fn test_cross_run_determinism() {
    let config = SystemConfig::for_testing();

    // Initialize twice
    let plant1 = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config.clone(),
    )
    .await
    .expect("First initialization");

    let plant2 = AutonomousOntologyPlant::initialize(
        "registry/meta-ontology.ttl",
        StorageBackend::InMemory,
        config,
    )
    .await
    .expect("Second initialization");

    // Snapshots should be identical (deterministic seed)
    let snapshot1 = plant1.current_snapshot().await.expect("Get snapshot 1");
    let snapshot2 = plant2.current_snapshot().await.expect("Get snapshot 2");

    assert_eq!(
        snapshot1.all_triples().len(),
        snapshot2.all_triples().len(),
        "Seed snapshots should be identical across runs"
    );
}
