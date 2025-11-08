//! Capability validation tests
//!
//! Tests for validating all capabilities are implemented and functional.

use knhk_workflow_engine::capabilities::{
    validate_capabilities, CapabilityRegistry, CapabilityStatus,
};

#[test]
fn test_all_43_patterns_registered() {
    let registry = CapabilityRegistry::new();

    for i in 1..=43 {
        let pattern_name = format!("pattern:{}", i);
        let capability = registry.get(&pattern_name);
        assert!(capability.is_some(), "Pattern {} should be registered", i);
        assert!(
            capability.unwrap().status == CapabilityStatus::Implemented,
            "Pattern {} should be implemented",
            i
        );
    }
}

#[test]
fn test_core_capabilities_available() {
    let registry = CapabilityRegistry::new();

    let core_capabilities = vec![
        "workflow:parsing",
        "workflow:execution",
        "workflow:state_management",
    ];

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
    let registry = CapabilityRegistry::new();

    let validation_result = registry.validate_required();
    assert!(
        validation_result.is_ok(),
        "All required capabilities should be available: {:?}",
        validation_result.err()
    );
}

#[test]
fn test_capability_validation_report() {
    let report = validate_capabilities().expect("Capability validation should succeed");

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
    let registry = CapabilityRegistry::new();

    let performance_capabilities = vec![
        "performance:hot_path",
        "performance:simd",
        "performance:metrics",
    ];

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
    let registry = CapabilityRegistry::new();

    assert!(
        registry.is_available("security:validation"),
        "Security validation should be available"
    );
}

#[test]
fn test_enterprise_capabilities() {
    let registry = CapabilityRegistry::new();

    let enterprise_capabilities = vec![
        "enterprise:observability",
        "enterprise:security",
        "enterprise:reliability",
        "enterprise:performance",
    ];

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
    let registry = CapabilityRegistry::new();

    let integration_capabilities = vec!["integration:fortune5", "integration:lockchain"];

    for capability_name in integration_capabilities {
        let capability = registry.get(capability_name);
        assert!(
            capability.is_some(),
            "Integration capability {} should be registered",
            capability_name
        );
    }
}
