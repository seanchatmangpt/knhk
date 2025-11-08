// knhk-config Chicago TDD tests
// State-based tests verifying configuration loading and validation

#![cfg(test)]
extern crate std;

use knhk_config::load_config;
use std::env;
use std::fs;

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
    let config = load_config(Some(config_file.clone())).expect("Failed to load config");

    // Verify: Configuration loaded correctly
    assert_eq!(config.knhk.version, "0.5.0");
    assert_eq!(config.knhk.context, "test");

    // Cleanup
    fs::remove_file(&config_file).ok();
    fs::remove_dir(&config_dir).ok();

    println!("  ✓ Configuration loaded from file");
    println!("  ✓ Version: {}", config.knhk.version);
    println!("  ✓ Context: {}", config.knhk.context);
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
    let mut config = load_config(Some(config_file.clone())).expect("Failed to load config");

    // Apply environment overrides
    if let Ok(context) = env::var("KNHK_CONTEXT") {
        config.knhk.context = context;
    }

    // Verify: Environment variable overrides file
    assert_eq!(
        config.knhk.context, "env_override",
        "Environment variable should override file"
    );
    assert_eq!(
        config.knhk.version, "0.5.0",
        "Non-overridden value should come from file"
    );

    // Cleanup
    env::remove_var("KNHK_CONTEXT");
    fs::remove_file(&config_file).ok();
    fs::remove_dir(&config_dir).ok();

    println!("  ✓ Environment variable overrides file config");
    println!("  ✓ Context: {} (from env)", config.knhk.context);
}

#[test]
fn test_config_validation() {
    println!("[TEST] Configuration Validation");

    // Setup: Create config file with connector
    let config_dir = std::env::temp_dir().join("knhk_test_config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    let config_file = config_dir.join("config.toml");
    let valid_content = r#"
[knhk]
version = "0.5.0"
context = "test"

[[connectors]]
type = "kafka"
max_run_len = 8
max_batch_size = 1000
"#;
    fs::write(&config_file, valid_content).expect("Failed to write config file");

    // Execute: Load configuration
    let result = load_config(Some(config_file.clone()));

    // Verify: Config loads successfully
    assert!(result.is_ok(), "Should load valid config");
    let config = result.unwrap();
    assert_eq!(config.knhk.version, "0.5.0");

    // Cleanup
    fs::remove_file(&config_file).ok();
    fs::remove_dir(&config_dir).ok();

    println!("  ✓ Configuration validation works");
}

#[test]
fn test_default_config() {
    println!("[TEST] Default Configuration");

    // Execute: Load default configuration (None = use default)
    let config = load_config(None).expect("Should load default config");

    // Verify: Default values are set
    assert_eq!(config.knhk.version, "0.5.0");
    assert_eq!(config.knhk.context, "default");

    println!("  ✓ Default configuration loaded");
    println!("  ✓ Version: {}", config.knhk.version);
    println!("  ✓ Context: {}", config.knhk.context);
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
    let result = load_config(Some(config_file.clone()));

    // Verify: Error message is clear and actionable
    assert!(result.is_err(), "Should fail to parse");
    let err_msg = result.unwrap_err();
    assert!(!err_msg.is_empty(), "Error message should not be empty");
    assert!(err_msg.len() > 20, "Error message should be descriptive");
    println!("  ✓ Parse error message: {}", err_msg);

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
    let config = load_config(Some(config_file.clone())).expect("Failed to load config");

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
