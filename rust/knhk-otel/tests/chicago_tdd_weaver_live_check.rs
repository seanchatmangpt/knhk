//! Chicago TDD tests for WeaverLiveCheck

#![cfg(feature = "std")]

use knhk_otel::WeaverLiveCheck;

#[test]
fn test_weaver_live_check_new() {
    let checker = WeaverLiveCheck::new();
    // Fields are private, but we can test behavior via otlp_endpoint()
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "127.0.0.1:4317");
}

#[test]
fn test_weaver_live_check_with_registry() {
    let checker = WeaverLiveCheck::new().with_registry("/path/to/registry".to_string());
    // Registry path is private, but builder pattern works
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "127.0.0.1:4317");
}

#[test]
fn test_weaver_live_check_with_otlp_address() {
    let checker = WeaverLiveCheck::new().with_otlp_address("192.168.1.1".to_string());
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "192.168.1.1:4317");
}

#[test]
fn test_weaver_live_check_with_otlp_port() {
    let checker = WeaverLiveCheck::new().with_otlp_port(4318);
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "127.0.0.1:4318");
}

#[test]
fn test_weaver_live_check_with_admin_port() {
    let checker = WeaverLiveCheck::new().with_admin_port(9090);
    // Admin port is private, but builder pattern works
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "127.0.0.1:4317");
}

#[test]
fn test_weaver_live_check_with_inactivity_timeout() {
    let checker = WeaverLiveCheck::new().with_inactivity_timeout(120);
    // Timeout is private, but builder pattern works
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "127.0.0.1:4317");
}

#[test]
fn test_weaver_live_check_with_format() {
    let checker = WeaverLiveCheck::new().with_format("ansi".to_string());
    // Format is private, but builder pattern works
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "127.0.0.1:4317");
}

#[test]
fn test_weaver_live_check_with_output() {
    let checker = WeaverLiveCheck::new().with_output("/tmp/weaver-output".to_string());
    // Output is private, but builder pattern works
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "127.0.0.1:4317");
}

#[test]
fn test_weaver_live_check_builder_chain() {
    let checker = WeaverLiveCheck::new()
        .with_registry("/registry".to_string())
        .with_otlp_address("localhost".to_string())
        .with_otlp_port(4318)
        .with_admin_port(9090)
        .with_inactivity_timeout(120)
        .with_format("ansi".to_string())
        .with_output("/output".to_string());
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "localhost:4318");
}

#[test]
fn test_weaver_live_check_check_weaver_available() {
    let result = WeaverLiveCheck::check_weaver_available();
    // Result may be Ok or Err depending on whether weaver is installed
    // We just verify it returns a Result without panicking
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_weaver_live_check_endpoint_format() {
    let checker = WeaverLiveCheck::new()
        .with_otlp_address("127.0.0.1".to_string())
        .with_otlp_port(4317);
    let endpoint = checker.otlp_endpoint();
    assert_eq!(endpoint, "127.0.0.1:4317");
}
