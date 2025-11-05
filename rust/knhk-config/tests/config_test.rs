// knhk-config Chicago TDD tests
// State-based tests verifying configuration loading and validation

#![cfg(test)]
extern crate std;

use knhk_config::{Config, ConfigError, load_config, load_default_config};
use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_load_config_from_file() {
    println!("[TEST] Load Config From File");
    
    // Setup: Create temporary config directory and file
    let config_dir = std::env::temp_dir().join("knhk_test_config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    
    let config_file = config_dir.join("config.toml");
    let config_content = r#"
[knhk]
version = "0.5.0"
context = "test"
"#;
    fs::write(&config_file, config_content).expect("Failed to write config file");
    
    // Execute: Load configuration
    let config = load_config(&config_file).expect("Failed to load config");
    
    // Verify: Configuration loaded correctly
    assert_eq!(config.version, "0.5.0");
    assert_eq!(config.context, "test");
    
    // Cleanup
    fs::remove_file(&config_file).ok();
    fs::remove_dir(&config_dir).ok();
    
    println!("  ✓ Configuration loaded from file");
    println!("  ✓ Version: {}", config.version);
    println!("  ✓ Context: {}", config.context);
}

#[test]
fn test_env_var_override() {
    println!("[TEST] Environment Variable Override");
    
    // Setup: Create config file
    let config_dir = std::env::temp_dir().join("knhk_test_config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    
    let config_file = config_dir.join("config.toml");
    let config_content = r#"
[knhk]
version = "0.5.0"
context = "file"
"#;
    fs::write(&config_file, config_content).expect("Failed to write config file");
    
    // Setup: Set environment variable
    env::set_var("KNHK_CONTEXT", "env_override");
    
    // Execute: Load configuration
    let config = load_config(&config_file).expect("Failed to load config");
    
    // Verify: Environment variable overrides file
    assert_eq!(config.context, "env_override", "Environment variable should override file");
    assert_eq!(config.version, "0.5.0", "Non-overridden value should come from file");
    
    // Cleanup
    env::remove_var("KNHK_CONTEXT");
    fs::remove_file(&config_file).ok();
    fs::remove_dir(&config_dir).ok();
    
    println!("  ✓ Environment variable overrides file config");
    println!("  ✓ Context: {} (from env)", config.context);
}

#[test]
fn test_config_validation() {
    println!("[TEST] Configuration Validation");
    
    // Setup: Create invalid config file
    let config_dir = std::env::temp_dir().join("knhk_test_config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    
    let config_file = config_dir.join("config.toml");
    let invalid_content = r#"
[knhk]
version = "0.5.0"
context = "test"
max_run_len = 10  # Invalid: exceeds guard constraint
"#;
    fs::write(&config_file, invalid_content).expect("Failed to write config file");
    
    // Execute: Try to load configuration
    let result = load_config(&config_file);
    
    // Verify: Validation error occurs
    assert!(result.is_err(), "Should fail validation");
    match result.unwrap_err() {
        ConfigError::ValidationFailed(msg) => {
            assert!(msg.contains("max_run_len"), "Error should mention max_run_len");
            println!("  ✓ Validation error: {}", msg);
        }
        _ => panic!("Expected ValidationFailed error"),
    }
    
    // Cleanup
    fs::remove_file(&config_file).ok();
    fs::remove_dir(&config_dir).ok();
    
    println!("  ✓ Configuration validation works");
}

#[test]
fn test_default_config() {
    println!("[TEST] Default Configuration");
    
    // Execute: Load default configuration
    let config = load_default_config();
    
    // Verify: Default values are set
    assert_eq!(config.version, "0.5.0");
    assert_eq!(config.context, "default");
    assert_eq!(config.max_run_len, 8);
    
    println!("  ✓ Default configuration loaded");
    println!("  ✓ Version: {}", config.version);
    println!("  ✓ Context: {}", config.context);
    println!("  ✓ Max run len: {}", config.max_run_len);
}

#[test]
fn test_config_error_reporting() {
    println!("[TEST] Configuration Error Reporting");
    
    // Setup: Create config file with parse error
    let config_dir = std::env::temp_dir().join("knhk_test_config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    
    let config_file = config_dir.join("config.toml");
    let invalid_content = r#"
[knhk]
version = "0.5.0"
context = "test"
invalid syntax here
"#;
    fs::write(&config_file, invalid_content).expect("Failed to write config file");
    
    // Execute: Try to load configuration
    let result = load_config(&config_file);
    
    // Verify: Error message is clear and actionable
    assert!(result.is_err(), "Should fail to parse");
    match result.unwrap_err() {
        ConfigError::ParseError(msg) => {
            assert!(!msg.is_empty(), "Error message should not be empty");
            assert!(msg.len() > 20, "Error message should be descriptive");
            println!("  ✓ Parse error message: {}", msg);
        }
        _ => panic!("Expected ParseError"),
    }
    
    // Cleanup
    fs::remove_file(&config_file).ok();
    fs::remove_dir(&config_dir).ok();
    
    println!("  ✓ Error messages are clear and actionable");
}

#[test]
fn test_config_connector_section() {
    println!("[TEST] Configuration Connector Section");
    
    // Setup: Create config file with connector
    let config_dir = std::env::temp_dir().join("knhk_test_config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    
    let config_file = config_dir.join("config.toml");
    let config_content = r#"
[knhk]
version = "0.5.0"
context = "test"

[knhk.connectors.kafka-prod]
type = "kafka"
bootstrap_servers = ["localhost:9092"]
topic = "triples"
max_run_len = 8
"#;
    fs::write(&config_file, config_content).expect("Failed to write config file");
    
    // Execute: Load configuration
    let config = load_config(&config_file).expect("Failed to load config");
    
    // Verify: Connector configuration loaded
    assert!(config.connectors.contains_key("kafka-prod"));
    let connector = &config.connectors["kafka-prod"];
    assert_eq!(connector.r#type, "kafka");
    assert_eq!(connector.max_run_len, 8);
    
    println!("  ✓ Connector configuration loaded");
    println!("  ✓ Connector type: {}", connector.r#type);
    println!("  ✓ Max run len: {}", connector.max_run_len);
    
    // Cleanup
    fs::remove_file(&config_file).ok();
    fs::remove_dir(&config_dir).ok();
}

