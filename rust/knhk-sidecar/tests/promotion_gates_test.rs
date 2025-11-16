// Integration tests for Phase 4: Promotion Gates
// Tests deterministic canary routing, SLO validation, automatic rollback, and promotion logic

use knhk_sidecar::promotion::{
    CanaryHealth, Environment, PromotionConfig, PromotionGateManager, RoutingDecision,
};
use knhk_sidecar::slo_admission::{AdmissionMetrics, SloAdmissionController, SloConfig};
use std::time::Duration;

/// Create test promotion config for canary environment
fn create_canary_config(traffic_percent: f64) -> PromotionConfig {
    PromotionConfig {
        environment: Environment::Canary { traffic_percent },
        feature_flags: vec!["new_feature".to_string(), "beta_api".to_string()],
        auto_rollback_enabled: true,
        slo_threshold: 0.95,
        rollback_window_seconds: 300,
    }
}

/// Create test promotion config for staging environment
fn create_staging_config() -> PromotionConfig {
    PromotionConfig {
        environment: Environment::Staging,
        feature_flags: vec!["new_feature".to_string(), "beta_api".to_string()],
        auto_rollback_enabled: true,
        slo_threshold: 0.95,
        rollback_window_seconds: 300,
    }
}

/// Create test promotion config for production environment
fn create_production_config() -> PromotionConfig {
    PromotionConfig {
        environment: Environment::Production,
        feature_flags: Vec::new(),
        auto_rollback_enabled: true,
        slo_threshold: 0.95,
        rollback_window_seconds: 300,
    }
}

/// Create test SLO admission controller
fn create_test_slo_controller() -> SloAdmissionController {
    let config = SloConfig::default();
    SloAdmissionController::new(config).expect("Failed to create SLO controller")
}

// ============================================================================
// DETERMINISTIC CANARY ROUTING TESTS
// ============================================================================

#[test]
fn test_deterministic_canary_routing_same_request_always_routes_same() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    let request_id = "user-12345-request";

    // Act: Route same request ID multiple times
    let decision1 = manager.route_request(request_id);
    let decision2 = manager.route_request(request_id);
    let decision3 = manager.route_request(request_id);

    // Assert: All decisions should be identical (deterministic)
    assert_eq!(
        decision1.is_canary, decision2.is_canary,
        "Same request_id should always route to same version"
    );
    assert_eq!(
        decision2.is_canary, decision3.is_canary,
        "Deterministic routing must be consistent"
    );
    assert_eq!(
        decision1.reason, decision2.reason,
        "Routing reason should be identical"
    );
}

#[test]
fn test_canary_traffic_split_respects_percentage() {
    // Arrange
    let config = create_canary_config(20.0); // 20% traffic to canary
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Route many different request IDs
    let mut canary_count = 0;
    let mut production_count = 0;
    let sample_size = 1000;

    for i in 0..sample_size {
        let request_id = format!("request-{}", i);
        let decision = manager.route_request(&request_id);
        if decision.is_canary {
            canary_count += 1;
        } else {
            production_count += 1;
        }
    }

    // Assert: Traffic should be split approximately at the threshold
    let canary_percentage = (canary_count as f64 / sample_size as f64) * 100.0;
    let expected_percentage = 20.0;
    let tolerance = 5.0; // Allow 5% tolerance

    assert!(
        (canary_percentage - expected_percentage).abs() < tolerance,
        "Canary traffic {} should be close to expected {}% (tolerance {}%)",
        canary_percentage,
        expected_percentage,
        tolerance
    );
}

#[test]
fn test_canary_routing_zero_percent_routes_all_to_production() {
    // Arrange
    let config = create_canary_config(0.0); // 0% traffic to canary
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Route multiple requests
    let mut all_production = true;
    for i in 0..100 {
        let request_id = format!("request-{}", i);
        let decision = manager.route_request(&request_id);
        if decision.is_canary {
            all_production = false;
            break;
        }
    }

    // Assert: All requests should route to production
    assert!(
        all_production,
        "0% canary traffic should route all requests to production"
    );
}

#[test]
fn test_canary_routing_100_percent_routes_all_to_canary() {
    // Arrange
    let config = create_canary_config(100.0); // 100% traffic to canary
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Route multiple requests
    let mut all_canary = true;
    for i in 0..100 {
        let request_id = format!("request-{}", i);
        let decision = manager.route_request(&request_id);
        if !decision.is_canary {
            all_canary = false;
            break;
        }
    }

    // Assert: All requests should route to canary
    assert!(
        all_canary,
        "100% canary traffic should route all requests to canary"
    );
}

#[test]
fn test_canary_request_includes_feature_flags() {
    // Arrange
    let config = create_canary_config(100.0); // Force all to canary
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act
    let decision = manager.route_request("test-request");

    // Assert: Canary request should include feature flags
    assert!(
        decision.is_canary,
        "Request should route to canary at 100%"
    );
    assert!(
        !decision.enabled_features.is_empty(),
        "Canary request should have enabled features"
    );
    assert!(
        decision.enabled_features.contains(&"new_feature".to_string()),
        "Feature 'new_feature' should be enabled for canary"
    );
}

#[test]
fn test_production_request_no_feature_flags() {
    // Arrange
    let config = create_canary_config(0.0); // Force all to production
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act
    let decision = manager.route_request("test-request");

    // Assert: Production request should have no feature flags
    assert!(
        !decision.is_canary,
        "Request should route to production at 0%"
    );
    assert!(
        decision.enabled_features.is_empty(),
        "Production request should have no enabled features"
    );
}

// ============================================================================
// ENVIRONMENT ROUTING TESTS
// ============================================================================

#[test]
fn test_staging_environment_routes_all_to_staging() {
    // Arrange
    let config = create_staging_config();
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Route multiple requests
    let mut all_staging = true;
    let mut has_features = true;

    for i in 0..10 {
        let request_id = format!("request-{}", i);
        let decision = manager.route_request(&request_id);

        if decision.target_environment != Environment::Staging {
            all_staging = false;
        }
        if decision.enabled_features.is_empty() {
            has_features = false;
        }
    }

    // Assert
    assert!(all_staging, "All requests in staging should route to staging");
    assert!(has_features, "Staging requests should have feature flags");
}

#[test]
fn test_production_environment_routes_all_to_production() {
    // Arrange
    let config = create_production_config();
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Route multiple requests
    let mut all_production = true;

    for i in 0..10 {
        let request_id = format!("request-{}", i);
        let decision = manager.route_request(&request_id);

        if decision.target_environment != Environment::Production {
            all_production = false;
        }
    }

    // Assert
    assert!(
        all_production,
        "All requests in production should route to production"
    );
}

// ============================================================================
// FEATURE FLAG TESTS
// ============================================================================

#[test]
fn test_feature_flag_enable_disable() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Enable a feature
    manager.enable_feature("test_feature".to_string());
    assert!(
        manager.is_feature_enabled("test_feature"),
        "Feature should be enabled after enable_feature"
    );

    // Act: Disable the feature
    manager.disable_feature("test_feature".to_string());

    // Assert
    assert!(
        !manager.is_feature_enabled("test_feature"),
        "Feature should be disabled after disable_feature"
    );
}

#[test]
fn test_feature_flag_default_disabled() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Assert: Features not in config should be disabled
    assert!(
        !manager.is_feature_enabled("nonexistent_feature"),
        "Features not in config should be disabled by default"
    );
}

// ============================================================================
// SLO COMPLIANCE AND ROLLBACK TESTS
// ============================================================================

#[test]
fn test_slo_compliance_check_passes_with_sufficient_admitted() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Check SLO compliance
    let result = manager.check_slo_compliance();

    // Assert: Should pass (no requests yet, so considered compliant)
    assert!(
        result.is_ok(),
        "SLO compliance check should succeed"
    );
    let compliant = result.expect("Should have result");
    assert!(
        compliant,
        "SLO compliance should be true when no requests have been made"
    );
}

#[test]
fn test_rollback_history_recorded() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    let initial_history_count = manager.get_rollback_history().len();

    // Act: Disable a feature (which triggers rollback recording)
    manager.disable_feature("feature_1".to_string());

    // Assert
    let final_history_count = manager.get_rollback_history().len();
    assert!(
        final_history_count > initial_history_count,
        "Rollback history should be recorded when feature is disabled"
    );
}

// ============================================================================
// PROMOTION LOGIC TESTS
// ============================================================================

#[test]
fn test_promotion_from_canary_to_staging() {
    // Arrange
    let canary_config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(canary_config, slo_controller)
        .expect("Failed to create promotion gate manager");

    let target_env = Environment::Staging;

    // Act
    let result = manager.promote(target_env);

    // Assert
    assert!(
        result.is_ok(),
        "Promotion from canary to staging should succeed"
    );
}

#[test]
fn test_promotion_from_staging_to_production() {
    // Arrange
    let staging_config = create_staging_config();
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(staging_config, slo_controller)
        .expect("Failed to create promotion gate manager");

    let target_env = Environment::Production;

    // Act
    let result = manager.promote(target_env);

    // Assert
    assert!(
        result.is_ok(),
        "Promotion from staging to production should succeed"
    );
}

#[test]
fn test_promotion_skips_staging() {
    // Arrange
    let canary_config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(canary_config, slo_controller)
        .expect("Failed to create promotion gate manager");

    let target_env = Environment::Production;

    // Act: Promote directly from canary to production
    let result = manager.promote(target_env);

    // Assert: Should succeed but log a warning
    assert!(
        result.is_ok(),
        "Promotion from canary to production should be allowed (though not recommended)"
    );
}

#[test]
fn test_invalid_promotion_fails() {
    // Arrange
    let production_config = create_production_config();
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(production_config, slo_controller)
        .expect("Failed to create promotion gate manager");

    let target_env = Environment::Staging;

    // Act: Try invalid promotion (production -> staging)
    let result = manager.promote(target_env);

    // Assert
    assert!(
        result.is_err(),
        "Promotion from production to staging should fail"
    );
}

// ============================================================================
// CANARY HEALTH MONITORING TESTS
// ============================================================================

#[test]
fn test_canary_health_not_applicable_in_staging() {
    // Arrange
    let config = create_staging_config();
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act
    let health = manager.monitor_canary_health();

    // Assert
    assert_eq!(
        health.health_score, 1.0,
        "Health score should be 1.0 when not in canary mode"
    );
    assert!(
        health.recommendation.contains("N/A"),
        "Recommendation should indicate canary mode not applicable"
    );
}

#[test]
fn test_canary_health_in_canary_environment() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Record some request outcomes to establish metrics
    for i in 0..100 {
        let request_id = format!("request-{}", i);
        let is_canary = manager.route_request(&request_id).is_canary;

        // Record successful request with small latency
        let latency = if is_canary {
            Duration::from_millis(5)
        } else {
            Duration::from_millis(3)
        };
        manager.record_request_outcome(&request_id, true, latency);
    }

    // Act: Check health
    let health = manager.monitor_canary_health();

    // Assert
    assert!(
        health.health_score >= 0.0 && health.health_score <= 1.0,
        "Health score should be between 0.0 and 1.0"
    );
    assert!(
        health.canary_requests > 0,
        "Canary requests should be recorded"
    );
    assert!(
        health.production_requests > 0,
        "Production requests should be recorded"
    );
}

// ============================================================================
// ERROR HANDLING AND VALIDATION TESTS
// ============================================================================

#[test]
fn test_invalid_traffic_percentage_rejected() {
    // Arrange
    let mut config = create_canary_config(50.0);
    config.environment = Environment::Canary {
        traffic_percent: 150.0, // Invalid: > 100
    };
    let slo_controller = create_test_slo_controller();

    // Act
    let result = PromotionGateManager::new(config, slo_controller);

    // Assert
    assert!(
        result.is_err(),
        "Invalid traffic percentage should be rejected during validation"
    );
}

#[test]
fn test_invalid_slo_threshold_rejected() {
    // Arrange
    let mut config = create_canary_config(50.0);
    config.slo_threshold = 1.5; // Invalid: > 1.0
    let slo_controller = create_test_slo_controller();

    // Act
    let result = PromotionGateManager::new(config, slo_controller);

    // Assert
    assert!(
        result.is_err(),
        "Invalid SLO threshold should be rejected during validation"
    );
}

#[test]
fn test_valid_configuration_accepted() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();

    // Act
    let result = PromotionGateManager::new(config, slo_controller);

    // Assert
    assert!(
        result.is_ok(),
        "Valid configuration should be accepted"
    );
}

// ============================================================================
// ENVIRONMENT NAME HELPER TESTS
// ============================================================================

#[test]
fn test_environment_name_helper() {
    // Act & Assert
    assert_eq!(
        Environment::Canary {
            traffic_percent: 50.0
        }
        .name(),
        "canary"
    );
    assert_eq!(Environment::Staging.name(), "staging");
    assert_eq!(Environment::Production.name(), "production");
}

// ============================================================================
// REQUEST OUTCOME RECORDING TESTS
// ============================================================================

#[test]
fn test_request_outcome_recording_canary() {
    // Arrange
    let config = create_canary_config(100.0); // Force to canary
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Record successful request
    let request_id = "test-request-1";
    manager.record_request_outcome(request_id, true, Duration::from_millis(5));

    // Act: Record failed request
    let request_id = "test-request-2";
    manager.record_request_outcome(request_id, false, Duration::from_millis(10));

    // Assert: Rollback history should capture the failures
    // (We can't directly access canary_metrics, but we can verify no errors occurred)
}

#[test]
fn test_routing_decision_contains_reason() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act
    let decision = manager.route_request("test-request");

    // Assert
    assert!(
        !decision.reason.is_empty(),
        "Routing decision should include a reason"
    );
    assert!(
        decision.reason.contains("%"),
        "Reason should contain percentage information"
    );
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_empty_request_id_routing() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Route request with empty ID
    let decision = manager.route_request("");

    // Assert: Should still produce a routing decision (deterministic hash of empty string)
    assert!(
        !decision.reason.is_empty(),
        "Even empty request_id should produce valid routing decision"
    );
}

#[test]
fn test_very_long_request_id_routing() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Route request with very long ID
    let long_id = "x".repeat(10000);
    let decision = manager.route_request(&long_id);

    // Assert: Should still produce a routing decision
    assert!(
        !decision.reason.is_empty(),
        "Very long request_id should produce valid routing decision"
    );
}

#[test]
fn test_special_characters_in_request_id() {
    // Arrange
    let config = create_canary_config(50.0);
    let slo_controller = create_test_slo_controller();
    let mut manager = PromotionGateManager::new(config, slo_controller)
        .expect("Failed to create promotion gate manager");

    // Act: Route requests with special characters
    let special_ids = vec![
        "user@example.com/request-123",
        "trace-id:abc-def-ghi",
        "request-with-emoji-🚀",
        "request with spaces",
    ];

    for request_id in special_ids {
        let decision = manager.route_request(request_id);
        assert!(
            !decision.reason.is_empty(),
            "Request with special characters should produce valid routing decision: {}",
            request_id
        );
    }
}
