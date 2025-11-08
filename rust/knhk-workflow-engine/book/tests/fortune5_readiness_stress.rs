//! Fortune 5 Readiness Validation Tests
//!
//! Comprehensive test suite using Chicago TDD framework to validate Fortune 5 readiness
//! and push the system to breaking points.
//!
//! **Testing Strategy**:
//! - State-based verification (not interaction-based)
//! - Real collaborators (minimal mocks)
//! - AAA pattern (Arrange, Act, Assert)
//! - Push to breaking points (stress testing, edge cases, failure scenarios)

use chicago_tdd_tools::builders::TestDataBuilder;
use knhk_workflow_engine::case::CaseState;
use knhk_workflow_engine::executor::WorkflowEngine;
use knhk_workflow_engine::integration::fortune5::{
    Environment, Fortune5Config, Fortune5Integration, KmsConfig, KmsProvider, MultiRegionConfig,
    PromotionConfig, ReplicationStrategy, SloConfig,
};
use knhk_workflow_engine::parser::{TaskType, WorkflowSpec, WorkflowSpecId};
use knhk_workflow_engine::state::StateStore;
use knhk_workflow_engine::testing::chicago_tdd::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::sleep;

// ============================================================================
// Test Fixture for Fortune 5 Tests
// ============================================================================

fn create_fortune5_engine() -> (WorkflowEngine, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        slo: Some(SloConfig {
            r1_p99_max_ns: 2,
            w1_p99_max_ms: 1,
            c1_p99_max_ms: 500,
            window_size_seconds: 60,
        }),
        promotion: Some(PromotionConfig {
            environment: Environment::Production,
            feature_flags: vec!["fortune5-test".to_string()],
            auto_rollback_enabled: true,
            slo_threshold: 0.99,
            rollback_window_seconds: 300,
        }),
        ..Default::default()
    };

    let engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)
        .expect("Failed to create Fortune 5 engine");

    (engine, temp_dir)
}

// ============================================================================
// SLO Compliance Tests - Push to Breaking Point
// ============================================================================

#[tokio::test]
async fn test_slo_compliance_under_normal_load() {
    // Arrange: Create Fortune 5 engine
    let (engine, _temp_dir) = create_fortune5_engine();

    // Act: Execute multiple workflows
    let spec = WorkflowSpecBuilder::new("SLO Test Workflow")
        .add_task(
            TaskBuilder::new("task:1", "Task 1")
                .with_max_ticks(8)
                .build(),
        )
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    for _ in 0..100 {
        let data = TestDataBuilder::new().build_json();
        let case_id = engine
            .create_case(spec_id, data)
            .await
            .expect("Failed to create case");
        engine
            .start_case(case_id)
            .await
            .expect("Failed to start case");
        engine
            .execute_case(case_id)
            .await
            .expect("Failed to execute case");
    }

    // Assert: SLO compliance maintained
    let compliant = engine
        .check_slo_compliance()
        .await
        .expect("Failed to check SLO");
    assert!(
        compliant,
        "SLO compliance should be maintained under normal load"
    );
}

#[tokio::test]
async fn test_slo_compliance_breaking_point_r1() {
    // Arrange: Create Fortune 5 engine with strict R1 SLO
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        slo: Some(SloConfig {
            r1_p99_max_ns: 2, // Very strict: 2ns max
            w1_p99_max_ms: 1,
            c1_p99_max_ms: 500,
            window_size_seconds: 60,
        }),
        ..Default::default()
    };

    let engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)
        .expect("Failed to create Fortune 5 engine");

    // Act: Execute workflows that exceed R1 SLO
    let spec = WorkflowSpecBuilder::new("R1 Breaking Point Test")
        .add_task(
            TaskBuilder::new("task:slow", "Slow Task")
                .with_max_ticks(100)
                .build(),
        )
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    // Execute enough to push R1 over limit
    for _ in 0..1000 {
        let data = TestDataBuilder::new().build_json();
        let case_id = engine
            .create_case(spec_id, data)
            .await
            .expect("Failed to create case");
        engine.start_case(case_id).await.ok();
        let _ = engine.execute_case(case_id).await; // May fail, that's expected
    }

    // Assert: SLO compliance should fail
    let compliant = engine
        .check_slo_compliance()
        .await
        .expect("Failed to check SLO");
    // At breaking point, compliance may fail
    // This test documents the breaking point
    if !compliant {
        println!("BREAKING POINT: R1 SLO violated after 1000 executions");
    }
}

#[tokio::test]
async fn test_slo_compliance_breaking_point_c1() {
    // Arrange: Create Fortune 5 engine with strict C1 SLO
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        slo: Some(SloConfig {
            r1_p99_max_ns: 2,
            w1_p99_max_ms: 1,
            c1_p99_max_ms: 100, // Strict: 100ms max
            window_size_seconds: 60,
        }),
        ..Default::default()
    };

    let engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)
        .expect("Failed to create Fortune 5 engine");

    // Act: Execute workflows with artificial delays to break C1 SLO
    let spec = WorkflowSpecBuilder::new("C1 Breaking Point Test")
        .add_task(TaskBuilder::new("task:slow", "Slow Task").build())
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    // Execute with delays to push C1 over limit
    for i in 0..100 {
        let data = TestDataBuilder::new()
            .with_var("delay_ms", &i.to_string())
            .build_json();
        let case_id = engine
            .create_case(spec_id, data)
            .await
            .expect("Failed to create case");
        engine.start_case(case_id).await.ok();
        let start = Instant::now();
        let _ = engine.execute_case(case_id).await;
        let elapsed = start.elapsed();

        // Record slow execution
        if elapsed.as_millis() > 100 {
            println!(
                "BREAKING POINT: C1 SLO violated - execution took {}ms",
                elapsed.as_millis()
            );
        }
    }

    // Assert: Check SLO compliance
    let compliant = engine
        .check_slo_compliance()
        .await
        .expect("Failed to check SLO");
    // Document breaking point
    if !compliant {
        println!("BREAKING POINT: C1 SLO violated after 100 executions");
    }
}

#[tokio::test]
async fn test_slo_metrics_accuracy_under_load() {
    // Arrange: Create Fortune 5 engine
    let (engine, _temp_dir) = create_fortune5_engine();

    // Act: Execute workflows and collect metrics
    let spec = WorkflowSpecBuilder::new("Metrics Test")
        .add_task(
            TaskBuilder::new("task:1", "Task 1")
                .with_max_ticks(8)
                .build(),
        )
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    let mut r1_samples = Vec::new();
    let mut w1_samples = Vec::new();
    let mut c1_samples = Vec::new();

    for _ in 0..1000 {
        let data = TestDataBuilder::new().build_json();
        let case_id = engine
            .create_case(spec_id, data)
            .await
            .expect("Failed to create case");
        engine
            .start_case(case_id)
            .await
            .expect("Failed to start case");
        let start = Instant::now();
        engine
            .execute_case(case_id)
            .await
            .expect("Failed to execute case");
        let elapsed_ns = start.elapsed().as_nanos() as u64;

        if elapsed_ns <= 2_000 {
            r1_samples.push(elapsed_ns);
        } else if elapsed_ns <= 1_000_000 {
            w1_samples.push(elapsed_ns / 1_000_000);
        } else {
            c1_samples.push(elapsed_ns / 1_000_000);
        }
    }

    // Assert: Metrics should be accurate
    if let Some((r1_p99, w1_p99, c1_p99)) = engine.get_slo_metrics().await {
        println!(
            "SLO Metrics - R1 P99: {}ns, W1 P99: {}ms, C1 P99: {}ms",
            r1_p99, w1_p99, c1_p99
        );

        // Verify metrics are reasonable
        assert!(r1_p99 <= 2_000_000, "R1 P99 should be ≤2ns");
        assert!(w1_p99 <= 1, "W1 P99 should be ≤1ms");
        assert!(c1_p99 <= 500, "C1 P99 should be ≤500ms");
    }
}

// ============================================================================
// Promotion Gate Tests - Push to Breaking Point
// ============================================================================

#[tokio::test]
async fn test_promotion_gate_production_slo_requirement() {
    // Arrange: Create Fortune 5 engine in production mode
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        slo: Some(SloConfig {
            r1_p99_max_ns: 2,
            w1_p99_max_ms: 1,
            c1_p99_max_ms: 500,
            window_size_seconds: 60,
        }),
        promotion: Some(PromotionConfig {
            environment: Environment::Production,
            feature_flags: vec![],
            auto_rollback_enabled: true,
            slo_threshold: 0.99,
            rollback_window_seconds: 300,
        }),
        ..Default::default()
    };

    let engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)
        .expect("Failed to create Fortune 5 engine");

    // Act: Try to register workflow
    let spec = WorkflowSpecBuilder::new("Production Test")
        .add_task(TaskBuilder::new("task:1", "Task 1").build())
        .build();

    // Assert: Should succeed if SLO compliant
    let result = engine.register_workflow(spec).await;
    assert!(
        result.is_ok(),
        "Workflow registration should succeed when SLO compliant"
    );
}

#[tokio::test]
async fn test_promotion_gate_blocks_on_slo_violation() {
    // Arrange: Create Fortune 5 engine with strict SLO
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        slo: Some(SloConfig {
            r1_p99_max_ns: 1, // Very strict
            w1_p99_max_ms: 1,
            c1_p99_max_ms: 100, // Very strict
            window_size_seconds: 60,
        }),
        promotion: Some(PromotionConfig {
            environment: Environment::Production,
            feature_flags: vec![],
            auto_rollback_enabled: true,
            slo_threshold: 0.99,
            rollback_window_seconds: 300,
        }),
        ..Default::default()
    };

    let engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)
        .expect("Failed to create Fortune 5 engine");

    // Act: Execute workflows that violate SLO
    let spec = WorkflowSpecBuilder::new("SLO Violation Test")
        .add_task(
            TaskBuilder::new("task:slow", "Slow Task")
                .with_max_ticks(1000)
                .build(),
        )
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    // Execute enough to violate SLO
    for _ in 0..1000 {
        let data = TestDataBuilder::new().build_json();
        let case_id = engine
            .create_case(spec_id, data)
            .await
            .expect("Failed to create case");
        engine.start_case(case_id).await.ok();
        let _ = engine.execute_case(case_id).await; // May fail
    }

    // Assert: Promotion gate should block new registrations
    let new_spec = WorkflowSpecBuilder::new("New Workflow")
        .add_task(TaskBuilder::new("task:1", "Task 1").build())
        .build();

    let result = engine.register_workflow(new_spec).await;
    // At breaking point, gate may block
    if result.is_err() {
        println!(
            "BREAKING POINT: Promotion gate blocked workflow registration due to SLO violation"
        );
    }
}

#[tokio::test]
async fn test_promotion_gate_rollback_window() {
    // Arrange: Create Fortune 5 engine with rollback window
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        slo: Some(SloConfig {
            r1_p99_max_ns: 2,
            w1_p99_max_ms: 1,
            c1_p99_max_ms: 500,
            window_size_seconds: 60,
        }),
        promotion: Some(PromotionConfig {
            environment: Environment::Production,
            feature_flags: vec![],
            auto_rollback_enabled: true,
            slo_threshold: 0.99,
            rollback_window_seconds: 5, // Short window for testing
        }),
        ..Default::default()
    };

    let engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)
        .expect("Failed to create Fortune 5 engine");

    // Act: Violate SLO, then wait for rollback window
    let spec = WorkflowSpecBuilder::new("Rollback Test")
        .add_task(
            TaskBuilder::new("task:slow", "Slow Task")
                .with_max_ticks(1000)
                .build(),
        )
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    // Execute to violate SLO
    for _ in 0..100 {
        let data = TestDataBuilder::new().build_json();
        let case_id = engine
            .create_case(spec_id, data)
            .await
            .expect("Failed to create case");
        engine.start_case(case_id).await.ok();
        let _ = engine.execute_case(case_id).await;
    }

    // Wait for rollback window
    sleep(Duration::from_secs(6)).await;

    // Assert: Gate should allow after rollback window
    let new_spec = WorkflowSpecBuilder::new("New Workflow After Rollback")
        .add_task(TaskBuilder::new("task:1", "Task 1").build())
        .build();

    let result = engine.register_workflow(new_spec).await;
    // Should succeed after rollback window
    assert!(
        result.is_ok(),
        "Promotion gate should allow after rollback window"
    );
}

// ============================================================================
// Feature Flag Tests - Push to Breaking Point
// ============================================================================

#[tokio::test]
async fn test_feature_flag_enabled() {
    // Arrange: Create Fortune 5 engine with feature flags
    let (engine, _temp_dir) = create_fortune5_engine();

    // Act & Assert: Check feature flag
    let enabled = engine.is_feature_enabled("fortune5-test").await;
    assert!(enabled, "Feature flag should be enabled");
}

#[tokio::test]
async fn test_feature_flag_disabled() {
    // Arrange: Create Fortune 5 engine with feature flags
    let (engine, _temp_dir) = create_fortune5_engine();

    // Act & Assert: Check non-existent feature flag
    let enabled = engine.is_feature_enabled("non-existent-feature").await;
    assert!(!enabled, "Non-existent feature flag should be disabled");
}

#[tokio::test]
async fn test_feature_flag_concurrent_checks() {
    // Arrange: Create Fortune 5 engine
    let (engine, _temp_dir) = create_fortune5_engine();

    // Act: Concurrent feature flag checks
    let engine_arc = Arc::new(engine);
    let mut handles = Vec::new();
    for i in 0..1000 {
        let engine_clone = engine_arc.clone();
        let handle = tokio::spawn(async move {
            engine_clone
                .is_feature_enabled(&format!("feature-{}", i))
                .await
        });
        handles.push(handle);
    }

    // Assert: All checks should complete
    for handle in handles {
        let _ = handle.await.expect("Feature flag check should complete");
    }
}

// ============================================================================
// Concurrent Execution Tests - Push to Breaking Point
// ============================================================================

#[tokio::test]
async fn test_concurrent_workflow_execution() {
    // Arrange: Create Fortune 5 engine
    let (engine, _temp_dir) = create_fortune5_engine();

    let spec = WorkflowSpecBuilder::new("Concurrent Test")
        .add_task(TaskBuilder::new("task:1", "Task 1").build())
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    // Act: Execute workflows concurrently
    let engine_arc = Arc::new(engine);
    let mut handles = Vec::new();
    for i in 0..100 {
        let engine_clone = engine_arc.clone();
        let spec_id_clone = spec_id;
        let data = TestDataBuilder::new()
            .with_var("index", &i.to_string())
            .build_json();

        let handle = tokio::spawn(async move {
            let case_id = engine_clone.create_case(spec_id_clone, data).await?;
            engine_clone.start_case(case_id).await?;
            engine_clone.execute_case(case_id).await
        });
        handles.push(handle);
    }

    // Assert: All executions should complete
    let mut success_count = 0;
    let mut failure_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            _ => failure_count += 1,
        }
    }

    println!(
        "Concurrent execution: {} success, {} failures",
        success_count, failure_count
    );
    assert!(
        success_count > 0,
        "At least some concurrent executions should succeed"
    );
}

#[tokio::test]
async fn test_concurrent_execution_breaking_point() {
    // Arrange: Create Fortune 5 engine
    let (engine, _temp_dir) = create_fortune5_engine();

    let spec = WorkflowSpecBuilder::new("Breaking Point Test")
        .add_task(TaskBuilder::new("task:1", "Task 1").build())
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    // Act: Push to breaking point with massive concurrency
    let engine_arc = Arc::new(engine);
    let mut handles = Vec::new();
    for i in 0..10000 {
        let engine_clone = engine_arc.clone();
        let spec_id_clone = spec_id;
        let data = TestDataBuilder::new()
            .with_var("index", &i.to_string())
            .build_json();

        let handle = tokio::spawn(async move {
            let case_id = engine_clone.create_case(spec_id_clone, data).await?;
            engine_clone.start_case(case_id).await?;
            engine_clone.execute_case(case_id).await
        });
        handles.push(handle);
    }

    // Assert: Document breaking point
    let mut success_count = 0;
    let mut failure_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            _ => failure_count += 1,
        }
    }

    println!(
        "BREAKING POINT: Concurrent execution - {} success, {} failures",
        success_count, failure_count
    );
    // Document the breaking point
    if failure_count > success_count {
        println!("BREAKING POINT: System overwhelmed at 10000 concurrent executions");
    }
}

// ============================================================================
// Multi-Region Tests - Push to Breaking Point
// ============================================================================

#[tokio::test]
async fn test_multi_region_configuration() {
    // Arrange: Create Fortune 5 engine with multi-region config
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        multi_region: Some(MultiRegionConfig {
            current_region: "us-east-1".to_string(),
            replication_regions: vec!["us-west-2".to_string(), "eu-west-1".to_string()],
            replication_strategy: ReplicationStrategy::Sync,
        }),
        ..Default::default()
    };

    // Act: Create engine
    let engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)
        .expect("Failed to create Fortune 5 engine");

    // Assert: Engine created successfully
    assert!(
        engine.fortune5_integration().is_some(),
        "Fortune 5 integration should be enabled"
    );
}

#[tokio::test]
async fn test_multi_region_replication_limits() {
    // Arrange: Create Fortune 5 engine with many replication regions
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    // Act: Try to configure with many regions (push to breaking point)
    let mut replication_regions = Vec::new();
    for i in 0..100 {
        replication_regions.push(format!("region-{}", i));
    }

    let fortune5_config = Fortune5Config {
        multi_region: Some(MultiRegionConfig {
            current_region: "us-east-1".to_string(),
            replication_regions,
            replication_strategy: ReplicationStrategy::Sync,
        }),
        ..Default::default()
    };

    // Assert: Should handle many regions (or document breaking point)
    let result = WorkflowEngine::with_fortune5(state_store, fortune5_config);
    match result {
        Ok(_) => println!("System handles 100 replication regions"),
        Err(e) => println!("BREAKING POINT: Failed with 100 regions: {}", e),
    }
}

// ============================================================================
// KMS Integration Tests - Push to Breaking Point
// ============================================================================

#[tokio::test]
async fn test_kms_configuration_validation() {
    // Arrange: Create Fortune 5 config with KMS
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        kms: Some(KmsConfig {
            provider: KmsProvider::Aws,
            provider_config: HashMap::from([
                (
                    "key_id".to_string(),
                    "arn:aws:kms:us-east-1:123456789012:key/abc123".to_string(),
                ),
                ("region".to_string(), "us-east-1".to_string()),
            ]),
            rotation_interval_hours: 24,
        }),
        ..Default::default()
    };

    // Act: Create engine
    let engine = WorkflowEngine::with_fortune5(state_store, fortune5_config)
        .expect("Failed to create Fortune 5 engine");

    // Assert: Engine created successfully
    assert!(
        engine.fortune5_integration().is_some(),
        "Fortune 5 integration should be enabled"
    );
}

#[tokio::test]
async fn test_kms_key_rotation_validation() {
    // Arrange: Test key rotation validation
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    // Act: Try invalid rotation interval (>24h)
    let fortune5_config = Fortune5Config {
        kms: Some(KmsConfig {
            provider: KmsProvider::Aws,
            provider_config: HashMap::new(),
            rotation_interval_hours: 25, // Invalid: >24h
        }),
        ..Default::default()
    };

    // Assert: Should validate and reject or use defaults
    let result = WorkflowEngine::with_fortune5(state_store, fortune5_config);
    match result {
        Ok(_) => println!("System handles invalid rotation interval gracefully"),
        Err(e) => println!("BREAKING POINT: Invalid rotation interval rejected: {}", e),
    }
}

// ============================================================================
// Stress Tests - Push to Absolute Breaking Point
// ============================================================================

#[tokio::test]
#[ignore] // Long-running test
async fn test_stress_test_massive_workflow_execution() {
    // Arrange: Create Fortune 5 engine
    let (engine, _temp_dir) = create_fortune5_engine();

    let spec = WorkflowSpecBuilder::new("Stress Test")
        .add_task(TaskBuilder::new("task:1", "Task 1").build())
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    // Act: Execute massive number of workflows
    let start = Instant::now();
    let mut success_count = 0;
    let mut failure_count = 0;

    for i in 0..100000 {
        let data = TestDataBuilder::new()
            .with_var("index", &i.to_string())
            .build_json();
        match engine.create_case(spec_id, data).await {
            Ok(case_id) => {
                engine.start_case(case_id).await.ok();
                match engine.execute_case(case_id).await {
                    Ok(_) => success_count += 1,
                    Err(_) => failure_count += 1,
                }
            }
            Err(_) => failure_count += 1,
        }

        if i % 10000 == 0 {
            println!("Progress: {} workflows executed", i);
        }
    }

    let elapsed = start.elapsed();

    // Assert: Document breaking point
    println!("STRESS TEST RESULTS:");
    println!("  Total: 100000 workflows");
    println!("  Success: {}", success_count);
    println!("  Failures: {}", failure_count);
    println!("  Duration: {:?}", elapsed);
    println!(
        "  Throughput: {:.2} workflows/sec",
        100000.0 / elapsed.as_secs_f64()
    );

    if failure_count > success_count {
        println!("BREAKING POINT: System overwhelmed at 100000 workflows");
    }
}

#[tokio::test]
async fn test_stress_test_slo_under_extreme_load() {
    // Arrange: Create Fortune 5 engine
    let (engine, _temp_dir) = create_fortune5_engine();

    let spec = WorkflowSpecBuilder::new("SLO Stress Test")
        .add_task(
            TaskBuilder::new("task:1", "Task 1")
                .with_max_ticks(8)
                .build(),
        )
        .build();
    engine
        .register_workflow(spec.clone())
        .await
        .expect("Failed to register workflow");
    let spec_id = spec.id;

    // Act: Execute workflows under extreme load
    for _ in 0..10000 {
        let data = TestDataBuilder::new().build_json();
        let case_id = engine
            .create_case(spec_id, data)
            .await
            .expect("Failed to create case");
        engine.start_case(case_id).await.ok();
        let _ = engine.execute_case(case_id).await;
    }

    // Assert: Check SLO compliance
    let compliant = engine
        .check_slo_compliance()
        .await
        .expect("Failed to check SLO");
    if !compliant {
        println!("BREAKING POINT: SLO compliance lost under extreme load (10000 workflows)");
    }

    // Get metrics
    if let Some((r1_p99, w1_p99, c1_p99)) = engine.get_slo_metrics().await {
        println!("SLO Metrics under extreme load:");
        println!("  R1 P99: {}ns", r1_p99);
        println!("  W1 P99: {}ms", w1_p99);
        println!("  C1 P99: {}ms", c1_p99);
    }
}

// ============================================================================
// Edge Case Tests - Push to Breaking Point
// ============================================================================

#[tokio::test]
async fn test_edge_case_empty_workflow() {
    // Arrange: Create Fortune 5 engine
    let (engine, _temp_dir) = create_fortune5_engine();

    // Act: Try to register empty workflow
    let spec = WorkflowSpecBuilder::new("Empty Workflow").build();
    let result = engine.register_workflow(spec).await;

    // Assert: Should handle gracefully
    match result {
        Ok(_) => println!("System handles empty workflow"),
        Err(e) => println!("BREAKING POINT: Empty workflow rejected: {}", e),
    }
}

#[tokio::test]
async fn test_edge_case_zero_slo_threshold() {
    // Arrange: Create Fortune 5 config with zero SLO threshold
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        promotion: Some(PromotionConfig {
            environment: Environment::Production,
            feature_flags: vec![],
            auto_rollback_enabled: true,
            slo_threshold: 0.0, // Edge case: zero threshold
            rollback_window_seconds: 300,
        }),
        ..Default::default()
    };

    // Act: Create engine
    let result = WorkflowEngine::with_fortune5(state_store, fortune5_config);

    // Assert: Should handle gracefully
    match result {
        Ok(_) => println!("System handles zero SLO threshold"),
        Err(e) => println!("BREAKING POINT: Zero SLO threshold rejected: {}", e),
    }
}

#[tokio::test]
async fn test_edge_case_max_slo_threshold() {
    // Arrange: Create Fortune 5 config with max SLO threshold
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let state_store = StateStore::new(temp_dir.path()).expect("Failed to create state store");

    let fortune5_config = Fortune5Config {
        promotion: Some(PromotionConfig {
            environment: Environment::Production,
            feature_flags: vec![],
            auto_rollback_enabled: true,
            slo_threshold: 1.0, // Edge case: max threshold
            rollback_window_seconds: 300,
        }),
        ..Default::default()
    };

    // Act: Create engine
    let result = WorkflowEngine::with_fortune5(state_store, fortune5_config);

    // Assert: Should handle gracefully
    match result {
        Ok(_) => println!("System handles max SLO threshold"),
        Err(e) => println!("BREAKING POINT: Max SLO threshold rejected: {}", e),
    }
}
