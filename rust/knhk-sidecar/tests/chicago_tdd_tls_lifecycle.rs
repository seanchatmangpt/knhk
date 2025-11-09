// Chicago TDD Tests for TLS Full Lifecycle
// Tests TLS configuration, validation, and certificate management
//
// Principles:
// 1. State-based verification (not interaction-based)
// 2. Real collaborators (no mocks)
// 3. Verify outputs and invariants, not implementation details
// 4. AAA pattern (Arrange, Act, Assert)

use knhk_sidecar::error::SidecarResult;
use knhk_sidecar::tls::{create_tls_client_config, create_tls_server_config, TlsConfig};
use std::fs;
use tempfile::TempDir;

// Helper: Create valid PEM certificate and key files
fn create_test_cert_files(temp_dir: &TempDir) -> (String, String) {
    // Create minimal valid PEM certificate
    let cert_pem = b"-----BEGIN CERTIFICATE-----
MIIBkTCB+wIJAK7wJ1K5 example
-----END CERTIFICATE-----";

    // Create minimal valid PEM private key
    let key_pem = b"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC example
-----END PRIVATE KEY-----";

    let cert_path = temp_dir.path().join("cert.pem");
    let key_path = temp_dir.path().join("key.pem");

    fs::write(&cert_path, cert_pem).expect("Failed to write cert");
    fs::write(&key_path, key_pem).expect("Failed to write key");

    (
        cert_path.to_string_lossy().to_string(),
        key_path.to_string_lossy().to_string(),
    )
}

// Helper: Create CA certificate file
fn create_test_ca_file(temp_dir: &TempDir) -> String {
    let ca_pem = b"-----BEGIN CERTIFICATE-----
MIIBkTCB+wIJAK7wJ1K5 ca-example
-----END CERTIFICATE-----";

    let ca_path = temp_dir.path().join("ca.pem");
    fs::write(&ca_path, ca_pem).expect("Failed to write CA cert");
    ca_path.to_string_lossy().to_string()
}

// ============================================================================
// Test Suite: TLS Configuration Lifecycle
// ============================================================================

#[test]
fn test_tls_config_default_disabled() {
    // Arrange: Create default TLS config
    let config = TlsConfig::default();

    // Act: Validate default config
    let result = config.validate();

    // Assert: Default config should be valid (disabled)
    assert!(
        result.is_ok(),
        "Default TLS config should be valid when disabled"
    );
    assert!(!config.enabled, "TLS should be disabled by default");
}

#[test]
fn test_tls_config_enable_tls() {
    // Arrange: Create temp directory with cert files
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);

    // Act: Enable TLS
    let config = TlsConfig::new().with_tls(cert_path.clone(), key_path.clone());

    // Assert: TLS should be enabled with correct paths
    assert!(config.enabled, "TLS should be enabled");
    assert_eq!(
        config.cert_file,
        Some(cert_path),
        "Certificate path should be set"
    );
    assert_eq!(config.key_file, Some(key_path), "Key path should be set");
    assert!(!config.mtls_enabled, "mTLS should not be enabled");
}

#[test]
fn test_tls_config_enable_mtls() {
    // Arrange: Create temp directory with cert files
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let ca_path = create_test_ca_file(&temp_dir);

    // Act: Enable mTLS
    let config = TlsConfig::new().with_mtls(cert_path.clone(), key_path.clone(), ca_path.clone());

    // Assert: mTLS should be enabled with all paths
    assert!(config.enabled, "TLS should be enabled");
    assert!(config.mtls_enabled, "mTLS should be enabled");
    assert_eq!(
        config.cert_file,
        Some(cert_path),
        "Certificate path should be set"
    );
    assert_eq!(config.key_file, Some(key_path), "Key path should be set");
    assert_eq!(config.ca_file, Some(ca_path), "CA path should be set");
}

#[test]
fn test_tls_config_validation_missing_cert_file() {
    // Arrange: Create config with TLS enabled but missing cert file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let key_path = temp_dir.path().join("key.pem");
    fs::write(&key_path, b"dummy key").expect("Failed to write key");

    let config = TlsConfig {
        enabled: true,
        cert_file: Some("/nonexistent/cert.pem".to_string()),
        key_file: Some(key_path.to_string_lossy().to_string()),
        ca_file: None,
        mtls_enabled: false,
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error for missing cert file
    assert!(
        result.is_err(),
        "Validation should fail for missing cert file"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Certificate file not found"));
}

#[test]
fn test_tls_config_validation_missing_key_file() {
    // Arrange: Create config with TLS enabled but missing key file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cert_path = temp_dir.path().join("cert.pem");
    fs::write(&cert_path, b"dummy cert").expect("Failed to write cert");

    let config = TlsConfig {
        enabled: true,
        cert_file: Some(cert_path.to_string_lossy().to_string()),
        key_file: Some("/nonexistent/key.pem".to_string()),
        ca_file: None,
        mtls_enabled: false,
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error for missing key file
    assert!(
        result.is_err(),
        "Validation should fail for missing key file"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Key file not found"));
}

#[test]
fn test_tls_config_validation_mtls_missing_ca() {
    // Arrange: Create mTLS config with missing CA file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);

    let config = TlsConfig {
        enabled: true,
        cert_file: Some(cert_path),
        key_file: Some(key_path),
        ca_file: Some("/nonexistent/ca.pem".to_string()),
        mtls_enabled: true,
    };

    // Act: Validate configuration
    let result = config.validate();

    // Assert: Should return error for missing CA file
    assert!(
        result.is_err(),
        "Validation should fail for missing CA file"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("CA certificate file not found"));
}

#[test]
fn test_tls_config_load_certificate() {
    // Arrange: Create config with valid cert file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let config = TlsConfig::new().with_tls(cert_path.clone(), key_path);

    // Act: Load certificate
    let result = config.load_cert();

    // Assert: Certificate should be loaded successfully
    assert!(result.is_ok(), "Certificate loading should succeed");
    let cert_data = result.unwrap();
    assert!(
        !cert_data.is_empty(),
        "Certificate data should not be empty"
    );
    let cert_str = String::from_utf8_lossy(&cert_data);
    assert!(
        cert_str.contains("BEGIN CERTIFICATE"),
        "Certificate should contain PEM header"
    );
}

#[test]
fn test_tls_config_load_key() {
    // Arrange: Create config with valid key file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let config = TlsConfig::new().with_tls(cert_path, key_path.clone());

    // Act: Load private key
    let result = config.load_key();

    // Assert: Key should be loaded successfully
    assert!(result.is_ok(), "Key loading should succeed");
    let key_data = result.unwrap();
    assert!(!key_data.is_empty(), "Key data should not be empty");
    let key_str = String::from_utf8_lossy(&key_data);
    assert!(
        key_str.contains("BEGIN PRIVATE KEY"),
        "Key should contain PEM header"
    );
}

#[test]
fn test_tls_config_load_ca_certificate() {
    // Arrange: Create mTLS config with valid CA file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let ca_path = create_test_ca_file(&temp_dir);
    let config = TlsConfig::new().with_mtls(cert_path, key_path, ca_path.clone());

    // Act: Load CA certificate
    let result = config.load_ca();

    // Assert: CA certificate should be loaded successfully
    assert!(result.is_ok(), "CA certificate loading should succeed");
    let ca_data = result.unwrap();
    assert!(
        !ca_data.is_empty(),
        "CA certificate data should not be empty"
    );
    let ca_str = String::from_utf8_lossy(&ca_data);
    assert!(
        ca_str.contains("BEGIN CERTIFICATE"),
        "CA certificate should contain PEM header"
    );
}

#[test]
fn test_tls_config_load_certificate_not_configured() {
    // Arrange: Create config without cert file
    let config = TlsConfig::default();

    // Act: Try to load certificate
    let result = config.load_cert();

    // Assert: Should return error
    assert!(result.is_err(), "Loading cert without config should fail");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Certificate file not configured"));
}

// ============================================================================
// Test Suite: TLS Server Configuration Lifecycle
// ============================================================================

#[test]
fn test_create_tls_server_config_basic_tls() {
    // Arrange: Create valid TLS config
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let config = TlsConfig::new().with_tls(cert_path, key_path);

    // Act: Create server TLS config
    let result = create_tls_server_config(&config);

    // Assert: Server config should be created successfully
    assert!(result.is_ok(), "Server TLS config creation should succeed");
    let _server_config = result.unwrap();
    // Note: We can't easily verify internal state of tonic::transport::ServerTlsConfig
    // but successful creation indicates the lifecycle works
}

#[test]
fn test_create_tls_server_config_mtls() {
    // Arrange: Create valid mTLS config
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let ca_path = create_test_ca_file(&temp_dir);
    let config = TlsConfig::new().with_mtls(cert_path, key_path, ca_path);

    // Act: Create server TLS config with mTLS
    let result = create_tls_server_config(&config);

    // Assert: Server config with mTLS should be created successfully
    assert!(result.is_ok(), "Server mTLS config creation should succeed");
    let _server_config = result.unwrap();
}

#[test]
fn test_create_tls_server_config_missing_file() {
    // Arrange: Create config with missing cert file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let key_path = temp_dir.path().join("key.pem");
    fs::write(
        &key_path,
        b"-----BEGIN PRIVATE KEY-----\nDUMMY\n-----END PRIVATE KEY-----\n",
    )
    .expect("Failed to write key");

    let config = TlsConfig {
        enabled: true,
        cert_file: Some("/nonexistent/cert.pem".to_string()),
        key_file: Some(key_path.to_string_lossy().to_string()),
        ca_file: None,
        mtls_enabled: false,
    };

    // Act: Try to create server config
    let result = create_tls_server_config(&config);

    // Assert: Should fail validation due to missing cert file
    assert!(
        result.is_err(),
        "Server config creation should fail with missing cert file"
    );
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// ============================================================================
// Test Suite: TLS Client Configuration Lifecycle
// ============================================================================

#[test]
fn test_create_tls_client_config_basic_tls() {
    // Arrange: Create valid TLS config
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let config = TlsConfig::new().with_tls(cert_path, key_path);

    // Act: Create client TLS config
    let result = create_tls_client_config(&config);

    // Assert: Client config should be created successfully
    assert!(result.is_ok(), "Client TLS config creation should succeed");
    let _client_config = result.unwrap();
}

#[test]
fn test_create_tls_client_config_mtls() {
    // Arrange: Create valid mTLS config
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let ca_path = create_test_ca_file(&temp_dir);
    let config = TlsConfig::new().with_mtls(cert_path, key_path, ca_path);

    // Act: Create client TLS config with mTLS
    let result = create_tls_client_config(&config);

    // Assert: Client config with mTLS should be created successfully
    assert!(result.is_ok(), "Client mTLS config creation should succeed");
    let _client_config = result.unwrap();
}

#[test]
fn test_create_tls_client_config_validation_failure() {
    // Arrange: Create config with missing files
    let config = TlsConfig {
        enabled: true,
        cert_file: Some("/nonexistent/cert.pem".to_string()),
        key_file: Some("/nonexistent/key.pem".to_string()),
        ca_file: None,
        mtls_enabled: false,
    };

    // Act: Try to create client config
    let result = create_tls_client_config(&config);

    // Assert: Should fail validation
    assert!(
        result.is_err(),
        "Client config creation should fail validation"
    );
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// ============================================================================
// Test Suite: TLS Full Lifecycle Integration
// ============================================================================

#[test]
fn test_tls_full_lifecycle_server_and_client() {
    // Arrange: Create valid TLS config for both server and client
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let config = TlsConfig::new().with_tls(cert_path.clone(), key_path.clone());

    // Act: Create both server and client configs
    let server_result = create_tls_server_config(&config);
    let client_result = create_tls_client_config(&config);

    // Assert: Both configs should be created successfully
    assert!(server_result.is_ok(), "Server config should be created");
    assert!(client_result.is_ok(), "Client config should be created");

    // Verify config state
    assert!(config.enabled, "TLS should be enabled");
    assert_eq!(config.cert_file, Some(cert_path), "Cert path should match");
    assert_eq!(config.key_file, Some(key_path), "Key path should match");
}

#[test]
fn test_tls_full_lifecycle_mtls_server_and_client() {
    // Arrange: Create valid mTLS config for both server and client
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let ca_path = create_test_ca_file(&temp_dir);
    let config = TlsConfig::new().with_mtls(cert_path.clone(), key_path.clone(), ca_path.clone());

    // Act: Create both server and client configs with mTLS
    let server_result = create_tls_server_config(&config);
    let client_result = create_tls_client_config(&config);

    // Assert: Both configs should be created successfully
    assert!(
        server_result.is_ok(),
        "Server mTLS config should be created"
    );
    assert!(
        client_result.is_ok(),
        "Client mTLS config should be created"
    );

    // Verify mTLS state
    assert!(config.enabled, "TLS should be enabled");
    assert!(config.mtls_enabled, "mTLS should be enabled");
    assert_eq!(config.ca_file, Some(ca_path), "CA path should match");
}

#[test]
fn test_tls_config_clone_preserves_state() {
    // Arrange: Create TLS config
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);
    let ca_path = create_test_ca_file(&temp_dir);
    let config = TlsConfig::new().with_mtls(cert_path.clone(), key_path.clone(), ca_path.clone());

    // Act: Clone config
    let cloned = config.clone();

    // Assert: Cloned config should preserve all state
    assert_eq!(cloned.enabled, config.enabled);
    assert_eq!(cloned.mtls_enabled, config.mtls_enabled);
    assert_eq!(cloned.cert_file, config.cert_file);
    assert_eq!(cloned.key_file, config.key_file);
    assert_eq!(cloned.ca_file, config.ca_file);
}

#[test]
fn test_tls_config_builder_pattern_chaining() {
    // Arrange: Create config using builder pattern
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let (cert_path, key_path) = create_test_cert_files(&temp_dir);

    // Act: Chain builder methods
    let config = TlsConfig::new().with_tls(cert_path.clone(), key_path.clone());

    // Assert: Builder pattern should work correctly
    assert!(config.enabled, "TLS should be enabled after with_tls");
    assert_eq!(config.cert_file, Some(cert_path), "Cert should be set");
    assert_eq!(config.key_file, Some(key_path), "Key should be set");
}
