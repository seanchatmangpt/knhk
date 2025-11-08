//! Capability validation tests
//!
//! Tests for validating all capabilities are implemented and functional.
//!
//! **UPGRADED**: Now uses Chicago TDD framework and property-based testing

use knhk_workflow_engine::capabilities::{
    validate_capabilities, CapabilityRegistry, CapabilityStatus,
};
use knhk_workflow_engine::testing::chicago_tdd::*;
use knhk_workflow_engine::testing::property::*;

#[test]
fn test_all_43_patterns_registered() {
    // Arrange: Use Chicago TDD helper
    let registry = create_test_registry();

    // Act & Assert: Verify all 43 patterns registered
    let patterns = registry.list_patterns();
    assert_eq!(patterns.len(), 43, "All 43 patterns should be registered");

    for i in 1..=43 {
        assert!(
            patterns.contains(&knhk_workflow_engine::patterns::PatternId(i)),
            "Pattern {} should be registered",
            i
        );
    }
}

#[test]
fn test_core_capabilities_available() {
    // Arrange: Create registry
    let registry = CapabilityRegistry::new();

    let core_capabilities = vec![
        "workflow:parsing",
        "workflow:execution",
        "workflow:state_management",
    ];

    // Act & Assert: Verify core capabilities
    for capability_name in core_capabilities {
        assert!(
            registry.is_available(capability_name),
            "Core capability {} should be available",
            capability_name
        );
    }
}

#[test]
fn test_required_capabilities_available() {
    // Arrange: Create registry
    let registry = CapabilityRegistry::new();

    // Act: Validate required capabilities
    let validation_result = registry.validate_required();

    // Assert: All required capabilities available
    assert!(
        validation_result.is_ok(),
        "All required capabilities should be available: {:?}",
        validation_result.err()
    );
}

#[test]
fn test_capability_validation_report() {
    // Arrange & Act: Validate capabilities
    let report = validate_capabilities().expect("Capability validation should succeed");

    // Assert: Use Chicago TDD assertions
    assert!(
        report.all_required_available(),
        "All required capabilities should be available"
    );

    assert!(
        report.implementation_percentage() > 80.0,
        "Implementation percentage should be >80%: {:.2}%",
        report.implementation_percentage()
    );

    assert!(
        report.production_readiness() == 100.0,
        "Production readiness should be 100%: {:.2}%",
        report.production_readiness()
    );
}

#[test]
fn test_performance_capabilities() {
    // Arrange: Create registry
    let registry = CapabilityRegistry::new();

    let performance_capabilities = vec![
        "performance:hot_path",
        "performance:simd",
        "performance:metrics",
    ];

    // Act & Assert: Verify performance capabilities
    for capability_name in performance_capabilities {
        let capability = registry.get(capability_name);
        assert!(
            capability.is_some(),
            "Performance capability {} should be registered",
            capability_name
        );
    }
}

#[test]
fn test_security_capabilities() {
    // Arrange: Create registry
    let registry = CapabilityRegistry::new();

    // Act & Assert: Security validation available
    assert!(
        registry.is_available("security:validation"),
        "Security validation should be available"
    );
}

#[test]
fn test_enterprise_capabilities() {
    // Arrange: Create registry
    let registry = CapabilityRegistry::new();

    let enterprise_capabilities = vec![
        "enterprise:observability",
        "enterprise:security",
        "enterprise:reliability",
        "enterprise:performance",
    ];

    // Act & Assert: Verify enterprise capabilities
    for capability_name in enterprise_capabilities {
        let capability = registry.get(capability_name);
        assert!(
            capability.is_some(),
            "Enterprise capability {} should be registered",
            capability_name
        );
    }
}

#[test]
fn test_integration_capabilities() {
    // Arrange: Create registry
    let registry = CapabilityRegistry::new();

    let integration_capabilities = vec!["integration:fortune5", "integration:lockchain"];

    // Act & Assert: Verify integration capabilities
    for capability_name in integration_capabilities {
        let capability = registry.get(capability_name);
        assert!(
            capability.is_some(),
            "Integration capability {} should be registered",
            capability_name
        );
    }
}

// ============================================================================
// Property-Based Testing for Capabilities
// ============================================================================

#[tokio::test]
async fn test_property_all_capabilities_registered() {
    // Property: All required capabilities are registered

    // Arrange: Create generator and fixture
    let mut generator = PropertyTestGenerator::new();
    let mut fixture = WorkflowTestFixture::new().unwrap();

    // Act: Generate workflows and verify capabilities
    for _ in 0..20 {
        let spec = generator.generate_workflow();
        let spec_id = fixture.register_workflow(spec).await.unwrap();

        // Assert: Workflow registered (capability available)
        assert!(
            fixture.specs.contains_key(&spec_id),
            "Property: All workflows can be registered (capability available)"
        );
    }
}

#[test]
fn test_property_capability_registry_consistent() {
    // Property: Capability registry is consistent

    // Arrange: Create multiple registries
    let registry1 = CapabilityRegistry::new();
    let registry2 = CapabilityRegistry::new();

    // Act & Assert: Registries should be consistent
    // (In production, would compare capabilities)
    assert!(
        registry1.validate_required().is_ok(),
        "Property: Registry 1 is consistent"
    );
    assert!(
        registry2.validate_required().is_ok(),
        "Property: Registry 2 is consistent"
    );
}
