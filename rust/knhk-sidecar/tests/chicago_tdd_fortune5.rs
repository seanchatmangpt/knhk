// Chicago TDD Tests for Fortune 5 Features
// Fortune 5 Readiness Validation
//
// Principles:
// 1. State-based verification (not interaction-based)
// 2. Real collaborators (no mocks)
// 3. Verify outputs and invariants, not implementation details

use knhk_sidecar::capacity::CapacityManager;
use knhk_sidecar::error::{SidecarError, SidecarResult};
use knhk_sidecar::key_rotation::KeyRotationManager;
use knhk_sidecar::kms::{KmsConfig, KmsProvider};
use knhk_sidecar::multi_region::{
    HoldCriteria, HoldPolicy, LegalHoldManager, Receipt, ReceiptSyncManager, RegionConfig,
};
use knhk_sidecar::promotion::{Environment, PromotionConfig, PromotionGateManager};
use knhk_sidecar::slo_admission::{
    AdmissionStrategy, RuntimeClass, SloAdmissionController, SloConfig,
};
use knhk_sidecar::spiffe::{
    extract_trust_domain, validate_spiffe_id, SpiffeCertManager, SpiffeConfig,
};
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

// Helper: Create temporary directory for SPIFFE tests
fn create_temp_spiffe_dir() -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let socket_path = temp_dir.path().join("api.sock");
    let cert_path = temp_dir.path().join("svid.pem");
    let key_path = temp_dir.path().join("key.pem");

    // Create socket directory
    std::fs::create_dir_all(temp_dir.path()).expect("Failed to create directory");

    // Create dummy certificate files
    std::fs::write(
        &cert_path,
        b"-----BEGIN CERTIFICATE-----\nDUMMY\n-----END CERTIFICATE-----\n",
    )
    .expect("Failed to write cert");
    std::fs::write(
        &key_path,
        b"-----BEGIN PRIVATE KEY-----\nDUMMY\n-----END PRIVATE KEY-----\n",
    )
    .expect("Failed to write key");

    (temp_dir, socket_path.to_string_lossy().to_string())
}

// ============================================================================
// Test Suite: SPIFFE/SPIRE Integration
// ============================================================================

#[tokio::test]
async fn test_spiffe_config_validation() {
    // Arrange: Create SPIFFE config with nonexistent socket
    let config = SpiffeConfig {
        socket_path: "/nonexistent/socket".to_string(),
        trust_domain: "example.com".to_string(),
        spiffe_id: None,
        refresh_interval: Duration::from_secs(3600),
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error for missing socket
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("SPIRE agent socket not found"));
}

#[tokio::test]
async fn test_spiffe_certificate_loading() {
    // Arrange: Create temp directory with SPIRE cert files
    let (temp_dir, socket_path) = create_temp_spiffe_dir();
    let config = SpiffeConfig {
        socket_path: socket_path.clone(),
        trust_domain: "example.com".to_string(),
        spiffe_id: None,
        refresh_interval: Duration::from_secs(3600),
    };

    // Act: Load certificate
    let mut manager = SpiffeCertManager::new(config).expect("Failed to create manager");
    let result = manager.load_certificate().await;

    // Assert: Certificate should be loaded and available
    assert!(result.is_ok(), "Certificate loading should succeed");
    assert!(
        manager.get_certificate().is_ok(),
        "Certificate should be available"
    );
    assert!(
        manager.get_private_key().is_ok(),
        "Private key should be available"
    );

    // Verify SPIFFE ID extraction
    let spiffe_id = manager.get_spiffe_id();
    assert!(
        spiffe_id.starts_with("spiffe://"),
        "SPIFFE ID should have correct format"
    );
    assert!(
        spiffe_id.contains("example.com"),
        "SPIFFE ID should contain trust domain"
    );
}

#[test]
fn test_spiffe_id_validation() {
    // Arrange & Act & Assert: Test SPIFFE ID format validation
    assert!(validate_spiffe_id("spiffe://example.com/sidecar"));
    assert!(validate_spiffe_id("spiffe://trust.domain/path/to/service"));
    assert!(!validate_spiffe_id("invalid"));
    assert!(!validate_spiffe_id("spiffe://"));
    assert!(!validate_spiffe_id("http://example.com"));
}

#[test]
fn test_extract_trust_domain() {
    // Arrange & Act & Assert: Test trust domain extraction
    assert_eq!(
        extract_trust_domain("spiffe://example.com/sidecar"),
        Some("example.com".to_string())
    );
    assert_eq!(
        extract_trust_domain("spiffe://trust.domain/path"),
        Some("trust.domain".to_string())
    );
    assert_eq!(extract_trust_domain("invalid"), None);
}

#[tokio::test]
async fn test_spiffe_certificate_refresh() {
    // Arrange: Create config with short refresh interval
    let (temp_dir, socket_path) = create_temp_spiffe_dir();
    let config = SpiffeConfig {
        socket_path: socket_path.clone(),
        trust_domain: "example.com".to_string(),
        spiffe_id: None,
        refresh_interval: Duration::from_secs(1), // Short interval for testing
    };

    let mut manager = SpiffeCertManager::new(config).expect("Failed to create manager");
    manager
        .load_certificate()
        .await
        .expect("Initial load should succeed");

    // Act: Check if refresh is needed immediately (should be false)
    let needs_refresh_immediately = manager.needs_refresh();

    // Wait for refresh interval
    tokio::time::sleep(Duration::from_secs(2)).await;
    let needs_refresh_after_wait = manager.needs_refresh();

    // Assert: Should not need refresh immediately, but after interval
    assert!(
        !needs_refresh_immediately,
        "Should not need refresh immediately after load"
    );
    assert!(
        needs_refresh_after_wait,
        "Should need refresh after interval expires"
    );
}

// ============================================================================
// Test Suite: KMS Integration
// ============================================================================

#[test]
fn test_kms_config_validation_aws() {
    // Arrange: Create AWS KMS config
    let config = KmsConfig::aws("us-east-1".to_string(), "key-123".to_string());

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should pass validation, auto-rotation enabled
    assert!(
        result.is_ok(),
        "Valid AWS KMS config should pass validation"
    );
    assert!(
        config.auto_rotation_enabled,
        "Auto rotation should be enabled"
    );
    assert_eq!(
        config.rotation_interval.as_secs(),
        86400,
        "Rotation interval should be 24h"
    );
}

#[test]
fn test_kms_config_validation_azure() {
    // Arrange: Create Azure Key Vault config
    let config = KmsConfig::azure("https://vault.azure.net".to_string(), "my-key".to_string());

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should pass validation
    assert!(
        result.is_ok(),
        "Valid Azure KMS config should pass validation"
    );
}

#[test]
fn test_kms_config_validation_vault() {
    // Arrange: Create HashiCorp Vault config
    let config = KmsConfig::vault(
        "http://localhost:8200".to_string(),
        "transit".to_string(),
        "my-key".to_string(),
    );

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should pass validation
    assert!(
        result.is_ok(),
        "Valid Vault KMS config should pass validation"
    );
}

#[test]
fn test_kms_config_validation_empty_region() {
    // Arrange: Create AWS config with empty region
    let config = KmsConfig::aws("".to_string(), "key-123".to_string());

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error
    assert!(result.is_err(), "Empty region should fail validation");
}

#[test]
fn test_kms_config_rotation_interval_validation() {
    // Arrange: Create config with rotation interval > 24h
    let mut config = KmsConfig::aws("us-east-1".to_string(), "key-123".to_string());
    config.rotation_interval = Duration::from_secs(86401); // > 24h

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error (Fortune 5 requirement)
    assert!(
        result.is_err(),
        "Rotation interval > 24h should fail validation"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("exceeds Fortune 5 requirement"));
}

// ============================================================================
// Test Suite: Key Rotation
// ============================================================================

#[test]
fn test_key_rotation_interval_validation() {
    // Arrange & Act & Assert: Test rotation interval validation
    // Valid: 24 hours
    assert!(KeyRotationManager::new(Duration::from_secs(86400)).is_ok());

    // Valid: Less than 24 hours
    assert!(KeyRotationManager::new(Duration::from_secs(3600)).is_ok());

    // Invalid: More than 24 hours
    let result = KeyRotationManager::new(Duration::from_secs(86401));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("exceeds Fortune 5 requirement"));
}

#[test]
fn test_key_rotation_needs_rotation() {
    // Arrange: Create rotation manager
    let manager =
        KeyRotationManager::new(Duration::from_secs(3600)).expect("Should create manager");

    // Act: Check if rotation is needed
    let needs_rotation = manager.needs_rotation();

    // Assert: Should need rotation if never checked
    assert!(needs_rotation, "Should need rotation if never checked");
}

#[test]
fn test_key_rotation_enable_disable() {
    // Arrange: Create rotation manager
    let mut manager =
        KeyRotationManager::new(Duration::from_secs(3600)).expect("Should create manager");

    // Act: Disable and enable rotation
    manager.set_enabled(false);
    let needs_rotation_when_disabled = manager.needs_rotation();

    manager.set_enabled(true);
    let needs_rotation_when_enabled = manager.needs_rotation();

    // Assert: State should change correctly
    assert!(
        !needs_rotation_when_disabled,
        "Should not need rotation when disabled"
    );
    assert!(
        needs_rotation_when_enabled,
        "Should need rotation when enabled"
    );
}

// ============================================================================
// Test Suite: Multi-Region Support
// ============================================================================

#[test]
fn test_region_config_validation() {
    // Arrange: Create valid region config
    let config = RegionConfig {
        region: "us-east-1".to_string(),
        primary_region: Some("us-east-1".to_string()),
        cross_region_sync_enabled: true,
        receipt_sync_endpoints: vec!["https://us-west-2.example.com".to_string()],
        quorum_threshold: 2,
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should pass validation
    assert!(result.is_ok(), "Valid region config should pass validation");
    assert!(config.is_primary(), "Should be primary region");
}

#[test]
fn test_region_config_validation_empty_region() {
    // Arrange: Create config with empty region
    let config = RegionConfig {
        region: "".to_string(),
        primary_region: None,
        cross_region_sync_enabled: false,
        receipt_sync_endpoints: Vec::new(),
        quorum_threshold: 1,
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error
    assert!(result.is_err(), "Empty region should fail validation");
}

#[test]
fn test_region_config_validation_quorum_threshold() {
    // Arrange: Create config with quorum threshold exceeding total regions
    let config = RegionConfig {
        region: "us-east-1".to_string(),
        primary_region: None,
        cross_region_sync_enabled: true,
        receipt_sync_endpoints: vec!["https://us-west-2.example.com".to_string()],
        quorum_threshold: 5, // More than total regions (2)
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error
    assert!(
        result.is_err(),
        "Quorum threshold > total regions should fail"
    );
}

#[test]
fn test_receipt_sync_manager_creation() {
    // Arrange: Create valid region config
    let config = RegionConfig {
        region: "us-east-1".to_string(),
        primary_region: None,
        cross_region_sync_enabled: true,
        receipt_sync_endpoints: vec!["https://us-west-2.example.com".to_string()],
        quorum_threshold: 1,
    };

    // Act: Create receipt sync manager
    let result = ReceiptSyncManager::new(config);

    // Assert: Should succeed
    assert!(result.is_ok(), "Valid config should create manager");
}

#[tokio::test]
async fn test_receipt_sync_disabled() {
    // Arrange: Create manager with sync disabled
    let config = RegionConfig {
        region: "us-east-1".to_string(),
        primary_region: None,
        cross_region_sync_enabled: false,
        receipt_sync_endpoints: Vec::new(),
        quorum_threshold: 1,
    };

    let manager = ReceiptSyncManager::new(config).expect("Should create manager");
    let receipt = Receipt {
        receipt_id: "test-123".to_string(),
        transaction_id: "txn-456".to_string(),
        hash: vec![1, 2, 3],
        ticks: 5,
        span_id: 789,
        committed: true,
    };

    // Act: Sync receipt (should not sync when disabled)
    let result = manager.sync_receipt(&receipt).await;

    // Assert: Should return success with 0 synced regions
    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert_eq!(
        sync_result.synced_regions, 0,
        "Should not sync when disabled"
    );
    assert_eq!(sync_result.total_regions, 0);
}

#[test]
fn test_legal_hold_manager() {
    // Arrange: Create legal hold manager with policy
    let config = RegionConfig {
        region: "us-east-1".to_string(),
        primary_region: None,
        cross_region_sync_enabled: false,
        receipt_sync_endpoints: Vec::new(),
        quorum_threshold: 1,
    };

    let mut manager = LegalHoldManager::new(config);
    let policy = HoldPolicy {
        name: "SOX Compliance".to_string(),
        retention_days: 2555, // 7 years
        match_criteria: HoldCriteria::All,
    };
    manager.add_hold_policy(policy);

    let receipt = Receipt {
        receipt_id: "test-123".to_string(),
        transaction_id: "txn-456".to_string(),
        hash: vec![1, 2, 3],
        ticks: 5,
        span_id: 789,
        committed: true,
    };

    // Act: Check if receipt should be held and apply hold
    let should_hold = manager.should_hold(&receipt);
    let result = manager.apply_hold(&receipt);

    // Assert: Should hold receipt matching policy
    assert!(should_hold, "Should hold receipt matching policy");
    assert!(result.is_ok(), "Apply hold should succeed");
}

// ============================================================================
// Test Suite: SLO Admission Control
// ============================================================================

#[test]
fn test_slo_config_validation() {
    // Arrange: Create default SLO config
    let config = SloConfig::default();

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should pass validation, values match Fortune 5 requirements
    assert!(result.is_ok(), "Default SLO config should be valid");
    assert_eq!(config.r1_p99_max_ns, 2, "R1 should be 2ns");
    assert_eq!(config.w1_p99_max_ms, 1, "W1 should be 1ms");
    assert_eq!(config.c1_p99_max_ms, 500, "C1 should be 500ms");
}

#[test]
fn test_slo_config_validation_r1_exceeds() {
    // Arrange: Create config with R1 > 2ns
    let config = SloConfig {
        r1_p99_max_ns: 3, // Exceeds 2ns
        w1_p99_max_ms: 1,
        c1_p99_max_ms: 500,
        admission_strategy: AdmissionStrategy::Strict,
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error
    assert!(result.is_err(), "R1 > 2ns should fail validation");
}

#[test]
fn test_slo_admission_strict_reject() {
    // Arrange: Create strict admission controller
    let config = SloConfig {
        r1_p99_max_ns: 2,
        w1_p99_max_ms: 1,
        c1_p99_max_ms: 500,
        admission_strategy: AdmissionStrategy::Strict,
    };

    let mut controller = SloAdmissionController::new(config).expect("Should create controller");

    // Act: Request R1 with latency exceeding SLO
    let result = controller.check_admission(
        RuntimeClass::R1,
        Some(Duration::from_nanos(3)), // Exceeds 2ns
    );

    // Assert: Should reject request exceeding SLO
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None, "Should reject request exceeding SLO");
}

#[test]
fn test_slo_admission_strict_accept() {
    // Arrange: Create strict admission controller
    let config = SloConfig {
        r1_p99_max_ns: 2,
        w1_p99_max_ms: 1,
        c1_p99_max_ms: 500,
        admission_strategy: AdmissionStrategy::Strict,
    };

    let mut controller = SloAdmissionController::new(config).expect("Should create controller");

    // Act: Request R1 with latency within SLO
    let result = controller.check_admission(
        RuntimeClass::R1,
        Some(Duration::from_nanos(1)), // Within 2ns
    );

    // Assert: Should admit request within SLO
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Some(RuntimeClass::R1),
        "Should admit request within SLO"
    );
}

#[test]
fn test_slo_admission_degrade() {
    // Arrange: Create degrade admission controller
    let config = SloConfig {
        r1_p99_max_ns: 2,
        w1_p99_max_ms: 1,
        c1_p99_max_ms: 500,
        admission_strategy: AdmissionStrategy::Degrade,
    };

    let mut controller = SloAdmissionController::new(config).expect("Should create controller");

    // Act: Request R1 with latency exceeding R1 SLO but within W1 SLO
    let result = controller.check_admission(
        RuntimeClass::R1,
        Some(Duration::from_micros(500)), // 500μs exceeds R1 (2ns) but within W1 (1ms)
    );

    // Assert: Should degrade to W1
    assert!(result.is_ok());
    let admitted = result.unwrap();
    assert_eq!(admitted, Some(RuntimeClass::W1), "Should degrade to W1");
}

#[test]
fn test_slo_admission_latency_tracking() {
    // Arrange: Create admission controller
    let config = SloConfig::default();
    let mut controller = SloAdmissionController::new(config).expect("Should create controller");

    // Act: Record some latencies
    controller.record_latency(RuntimeClass::R1, Duration::from_nanos(1));
    controller.record_latency(RuntimeClass::R1, Duration::from_nanos(2));
    controller.record_latency(RuntimeClass::R1, Duration::from_nanos(1));

    // Estimate latency (should use historical data)
    let estimated = controller.estimate_latency(RuntimeClass::R1);

    // Assert: Estimated latency should be based on history
    assert!(
        estimated <= Duration::from_nanos(2),
        "Estimated latency should be ≤2ns"
    );
}

#[test]
fn test_slo_admission_metrics() {
    // Arrange: Create admission controller
    let config = SloConfig::default();
    let mut controller = SloAdmissionController::new(config).expect("Should create controller");

    // Act: Admit and reject some requests
    controller
        .check_admission(RuntimeClass::R1, Some(Duration::from_nanos(1)))
        .unwrap();
    controller
        .check_admission(RuntimeClass::R1, Some(Duration::from_nanos(3)))
        .unwrap();
    controller
        .check_admission(RuntimeClass::W1, Some(Duration::from_millis(1)))
        .unwrap();

    let metrics = controller.get_metrics();

    // Assert: Metrics should be updated correctly
    assert_eq!(metrics.r1_admitted, 1, "Should have 1 R1 admission");
    assert_eq!(metrics.r1_rejected, 1, "Should have 1 R1 rejection");
    assert_eq!(metrics.w1_admitted, 1, "Should have 1 W1 admission");
}

// ============================================================================
// Test Suite: Capacity Planning
// ============================================================================

#[test]
fn test_capacity_manager_record_access() {
    // Arrange: Create capacity manager
    let mut manager = CapacityManager::new(0.95);

    // Act: Record cache hits and misses
    manager.record_access("predicate1", true, true); // Hit, L1
    manager.record_access("predicate1", true, false); // Hit, not L1
    manager.record_access("predicate1", false, false); // Miss

    // Assert: Metrics should be tracked correctly
    let heat = manager
        .get_heat("predicate1")
        .expect("Should have heat data");
    assert_eq!(heat.cache_hits, 2);
    assert_eq!(heat.cache_misses, 1);
    assert_eq!(heat.l1_hits, 1);
    assert_eq!(heat.hit_rate(), 2.0 / 3.0, "Hit rate should be 2/3");
}

#[test]
fn test_capacity_manager_hit_rate() {
    // Arrange: Create capacity manager
    let mut manager = CapacityManager::new(0.95);

    // Act: Record mostly hits
    for _ in 0..95 {
        manager.record_access("predicate1", true, false);
    }
    for _ in 0..5 {
        manager.record_access("predicate1", false, false);
    }

    // Assert: Should meet capacity threshold
    assert!(
        manager.meets_capacity("predicate1"),
        "Should meet capacity threshold"
    );
    let heat = manager.get_heat("predicate1").unwrap();
    assert_eq!(heat.hit_rate(), 0.95, "Hit rate should be 95%");
}

#[test]
fn test_capacity_manager_hottest_predicates() {
    // Arrange: Create capacity manager
    let mut manager = CapacityManager::new(0.95);

    // Act: Record accesses for multiple predicates
    for i in 0..100 {
        manager.record_access("predicate1", i < 90, false); // 90% hit rate
        manager.record_access("predicate2", i < 80, false); // 80% hit rate
        manager.record_access("predicate3", i < 70, false); // 70% hit rate
    }

    let hottest = manager.get_hottest_predicates(2);

    // Assert: Should return top 2, sorted by hit rate
    assert_eq!(hottest.len(), 2, "Should return top 2");
    assert_eq!(hottest[0].0, "predicate1", "predicate1 should be hottest");
    assert!(hottest[0].1 > hottest[1].1, "Should be sorted by hit rate");
}

#[test]
fn test_capacity_manager_l1_locality_prediction() {
    // Arrange: Create capacity manager
    let mut manager = CapacityManager::new(0.95);

    // Act: Record L1 hits and misses
    for _ in 0..80 {
        manager.record_access("predicate1", true, true); // L1 hit
    }
    for _ in 0..20 {
        manager.record_access("predicate1", true, false); // Hit but not L1
    }

    // Assert: L1 locality should be 80%
    let l1_locality = manager.predict_l1_locality("predicate1");
    assert_eq!(l1_locality, 0.8, "L1 locality should be 80%");
}

// ============================================================================
// Test Suite: Promotion Gates
// ============================================================================

#[test]
fn test_promotion_config_validation() {
    // Arrange: Create default promotion config
    let config = PromotionConfig::default();

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should pass validation
    assert!(result.is_ok(), "Default promotion config should be valid");
    assert_eq!(config.slo_threshold, 0.95, "SLO threshold should be 0.95");
}

#[test]
fn test_promotion_config_validation_invalid_threshold() {
    // Arrange: Create config with SLO threshold > 1.0
    let config = PromotionConfig {
        environment: Environment::Production,
        feature_flags: Vec::new(),
        auto_rollback_enabled: true,
        slo_threshold: 1.5, // Invalid: > 1.0
        rollback_window_seconds: 300,
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error
    assert!(
        result.is_err(),
        "SLO threshold > 1.0 should fail validation"
    );
}

#[test]
fn test_promotion_config_validation_canary_traffic() {
    // Arrange: Create canary config with traffic > 100%
    let config = PromotionConfig {
        environment: Environment::Canary {
            traffic_percent: 150.0,
        }, // Invalid: > 100%
        feature_flags: Vec::new(),
        auto_rollback_enabled: true,
        slo_threshold: 0.95,
        rollback_window_seconds: 300,
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error
    assert!(
        result.is_err(),
        "Canary traffic > 100% should fail validation"
    );
}

#[test]
fn test_promotion_gate_manager_creation() {
    // Arrange: Create valid promotion and SLO configs
    let promotion_config = PromotionConfig::default();
    let slo_config = SloConfig::default();
    let slo_controller =
        SloAdmissionController::new(slo_config).expect("Should create SLO controller");

    // Act: Create promotion gate manager
    let result = PromotionGateManager::new(promotion_config, slo_controller);

    // Assert: Should succeed
    assert!(result.is_ok(), "Valid config should create manager");
}

#[test]
fn test_promotion_gate_feature_flags() {
    // Arrange: Create manager with feature flags
    let promotion_config = PromotionConfig {
        environment: Environment::Production,
        feature_flags: vec!["feature1".to_string(), "feature2".to_string()],
        auto_rollback_enabled: true,
        slo_threshold: 0.95,
        rollback_window_seconds: 300,
    };
    let slo_config = SloConfig::default();
    let slo_controller =
        SloAdmissionController::new(slo_config).expect("Should create SLO controller");
    let mut manager =
        PromotionGateManager::new(promotion_config, slo_controller).expect("Should create manager");

    // Act & Assert: Test feature flag management
    assert!(
        manager.is_feature_enabled("feature1"),
        "Feature1 should be enabled"
    );
    assert!(
        manager.is_feature_enabled("feature2"),
        "Feature2 should be enabled"
    );
    assert!(
        !manager.is_feature_enabled("feature3"),
        "Feature3 should not be enabled"
    );

    // Disable feature
    manager.disable_feature("feature1".to_string());
    assert!(
        !manager.is_feature_enabled("feature1"),
        "Feature1 should be disabled"
    );
}

#[test]
fn test_promotion_gate_slo_compliance_no_requests() {
    // Arrange: Create manager
    let promotion_config = PromotionConfig::default();
    let slo_config = SloConfig::default();
    let slo_controller =
        SloAdmissionController::new(slo_config).expect("Should create SLO controller");
    let mut manager =
        PromotionGateManager::new(promotion_config, slo_controller).expect("Should create manager");

    // Act: Check compliance with no requests
    let result = manager.check_slo_compliance();

    // Assert: Should return true (no data to evaluate)
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        true,
        "Should be compliant with no requests"
    );
}

#[test]
fn test_promotion_gate_slo_compliance_below_threshold() {
    // Arrange: Create manager
    let promotion_config = PromotionConfig {
        environment: Environment::Production,
        feature_flags: Vec::new(),
        auto_rollback_enabled: true,
        slo_threshold: 0.95,
        rollback_window_seconds: 300,
    };
    let slo_config = SloConfig::default();
    let mut slo_controller =
        SloAdmissionController::new(slo_config).expect("Should create SLO controller");

    // Act: Create requests with low compliance (50% admitted)
    // Admit 5, reject 5 to create 50% compliance
    for _ in 0..5 {
        slo_controller
            .check_admission(RuntimeClass::R1, Some(Duration::from_nanos(1)))
            .unwrap();
    }
    for _ in 0..5 {
        slo_controller
            .check_admission(RuntimeClass::R1, Some(Duration::from_nanos(3)))
            .unwrap();
    }

    // Create manager with controller that has updated metrics
    let mut manager =
        PromotionGateManager::new(promotion_config, slo_controller).expect("Should create manager");

    let result = manager.check_slo_compliance();

    // Assert: Should not be compliant with 50% admission rate
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        false,
        "Should not be compliant with 50% admission rate"
    );
    assert!(
        !manager.get_rollback_history().is_empty(),
        "Should have rollback history"
    );
}

#[test]
fn test_promotion_gate_promote_canary_to_staging() {
    // Arrange: Create manager in canary environment
    let promotion_config = PromotionConfig {
        environment: Environment::Canary {
            traffic_percent: 10.0,
        },
        feature_flags: Vec::new(),
        auto_rollback_enabled: true,
        slo_threshold: 0.95,
        rollback_window_seconds: 300,
    };
    let slo_config = SloConfig::default();
    let slo_controller =
        SloAdmissionController::new(slo_config).expect("Should create SLO controller");
    let mut manager =
        PromotionGateManager::new(promotion_config, slo_controller).expect("Should create manager");

    // Act: Promote from canary to staging
    let result = manager.promote(Environment::Staging);

    // Assert: Should succeed
    assert!(result.is_ok(), "Should promote from canary to staging");
}

#[test]
fn test_promotion_gate_promote_invalid() {
    // Arrange: Create manager in production environment
    let promotion_config = PromotionConfig {
        environment: Environment::Production,
        feature_flags: Vec::new(),
        auto_rollback_enabled: true,
        slo_threshold: 0.95,
        rollback_window_seconds: 300,
    };
    let slo_config = SloConfig::default();
    let slo_controller =
        SloAdmissionController::new(slo_config).expect("Should create SLO controller");
    let mut manager =
        PromotionGateManager::new(promotion_config, slo_controller).expect("Should create manager");

    // Act: Try to promote from production to staging (invalid)
    let result = manager.promote(Environment::Staging);

    // Assert: Should return error
    assert!(
        result.is_err(),
        "Should not allow promotion from production to staging"
    );
}

// ============================================================================
// Test Suite: Integration Tests
// ============================================================================

#[tokio::test]
async fn test_fortune5_integration_spiffe_kms() {
    // Arrange: Create SPIFFE and KMS managers
    let (temp_dir, socket_path) = create_temp_spiffe_dir();
    let spiffe_config = SpiffeConfig {
        socket_path: socket_path.clone(),
        trust_domain: "example.com".to_string(),
        spiffe_id: None,
        refresh_interval: Duration::from_secs(3600),
    };

    let mut spiffe_manager =
        SpiffeCertManager::new(spiffe_config).expect("Should create SPIFFE manager");
    spiffe_manager
        .load_certificate()
        .await
        .expect("Should load certificate");

    // KMS config (will fail without fortune5 feature, but structure is valid)
    let kms_config = KmsConfig::aws("us-east-1".to_string(), "key-123".to_string());

    // Act: Validate both configurations
    let spiffe_valid = spiffe_manager.get_certificate().is_ok();
    let kms_valid = kms_config.validate().is_ok();

    // Assert: Both should be valid
    assert!(spiffe_valid, "SPIFFE should be valid");
    assert!(kms_valid, "KMS config should be valid");
}

#[test]
fn test_fortune5_integration_slo_capacity() {
    // Arrange: Create SLO and capacity managers
    let slo_config = SloConfig::default();
    let mut slo_controller =
        SloAdmissionController::new(slo_config).expect("Should create SLO controller");
    let mut capacity_manager = CapacityManager::new(0.95);

    // Act: Record capacity and check admission
    capacity_manager.record_access("predicate1", true, false);
    let meets_capacity = capacity_manager.meets_capacity("predicate1");

    // Record latency for SLO
    slo_controller.record_latency(RuntimeClass::R1, Duration::from_nanos(1));
    let admission = slo_controller.check_admission(RuntimeClass::R1, None);

    // Assert: Both should work together
    assert!(meets_capacity, "Should meet capacity after hit");
    assert!(admission.is_ok());
    assert_eq!(
        admission.unwrap(),
        Some(RuntimeClass::R1),
        "Should admit based on history"
    );
}

#[test]
fn test_fortune5_integration_multi_region_legal_hold() {
    // Arrange: Create multi-region and legal hold managers
    let region_config = RegionConfig {
        region: "us-east-1".to_string(),
        primary_region: Some("us-east-1".to_string()),
        cross_region_sync_enabled: false,
        receipt_sync_endpoints: Vec::new(),
        quorum_threshold: 1,
    };

    let mut legal_hold_manager = LegalHoldManager::new(region_config.clone());
    let policy = HoldPolicy {
        name: "GDPR Compliance".to_string(),
        retention_days: 2555,
        match_criteria: HoldCriteria::ByTransactionId("txn-123".to_string()),
    };
    legal_hold_manager.add_hold_policy(policy);

    let receipt = Receipt {
        receipt_id: "rcpt-456".to_string(),
        transaction_id: "txn-123".to_string(),
        hash: vec![1, 2, 3],
        ticks: 5,
        span_id: 789,
        committed: true,
    };

    // Act: Check if receipt should be held and apply hold
    let should_hold = legal_hold_manager.should_hold(&receipt);
    let result = legal_hold_manager.apply_hold(&receipt);

    // Assert: Should hold receipt matching transaction ID
    assert!(should_hold, "Should hold receipt matching transaction ID");
    assert!(result.is_ok(), "Apply hold should succeed");
}
