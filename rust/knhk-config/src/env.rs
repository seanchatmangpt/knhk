// knhk-config/src/env.rs
// Environment variable parsing and override support

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::config::Config;

#[cfg(feature = "std")]
use std::env;

/// Load configuration from environment variables
/// Environment variables override config file values
/// 
/// Format: KNHK_<SECTION>_<KEY> for nested values
/// Examples:
///   KNHK_CONTEXT=production
///   KNHK_CONNECTOR_KAFKA_BOOTSTRAP_SERVERS=localhost:9092
///   KNHK_EPOCH_DEFAULT_TAU=8
#[cfg(feature = "std")]
pub fn load_env_config() -> BTreeMap<String, String> {
    let mut env_config = BTreeMap::new();
    
    for (key, value) in env::vars() {
        if key.starts_with("KNHK_") {
            // Normalize key (remove KNHK_ prefix, convert to lowercase)
            let normalized_key = key.trim_start_matches("KNHK_").to_lowercase();
            env_config.insert(normalized_key, value);
        }
    }
    
    env_config
}

#[cfg(not(feature = "std"))]
pub fn load_env_config() -> BTreeMap<String, String> {
    // In no_std mode, return empty map
    BTreeMap::new()
}

/// Merge environment variables into config
/// Environment variables override config file values
#[cfg(feature = "std")]
pub fn apply_env_overrides(config: &mut Config, env_vars: &BTreeMap<String, String>) {
    // Apply KNHK context override
    if let Some(context) = env_vars.get("context") {
        config.knhk.context = context.clone();
    }
    
    // Apply connector overrides (format: connector_<name>_<key>)
    for (key, value) in env_vars.iter() {
        if key.starts_with("connector_") {
            let parts: Vec<&str> = key.splitn(3, '_').collect();
            if parts.len() >= 3 {
                let connector_name = parts[1].to_string();
                let config_key = parts[2].to_string();
                
                if !config.connectors.contains_key(&connector_name) {
                    // Create default connector config
                    config.connectors.insert(
                        connector_name.clone(),
                        crate::config::ConnectorConfig {
                            r#type: "kafka".to_string(),
                            bootstrap_servers: None,
                            topic: None,
                            schema: None,
                            max_run_len: 8,
                            max_batch_size: 1000,
                        },
                    );
                }
                
                if let Some(connector) = config.connectors.get_mut(&connector_name) {
                    match config_key.as_str() {
                        "bootstrap_servers" => {
                            connector.bootstrap_servers = Some(value.split(',').map(|s| s.trim().to_string()).collect());
                        }
                        "topic" => {
                            connector.topic = Some(value.clone());
                        }
                        "schema" => {
                            connector.schema = Some(value.clone());
                        }
                        "max_run_len" => {
                            if let Ok(val) = value.parse::<u64>() {
                                connector.max_run_len = val;
                            }
                        }
                        "max_batch_size" => {
                            if let Ok(val) = value.parse::<u64>() {
                                connector.max_batch_size = val;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Apply epoch overrides (format: epoch_<name>_<key>)
        if key.starts_with("epoch_") {
            let parts: Vec<&str> = key.splitn(3, '_').collect();
            if parts.len() >= 3 {
                let epoch_name = parts[1].to_string();
                let config_key = parts[2].to_string();
                
                if !config.epochs.contains_key(&epoch_name) {
                    config.epochs.insert(
                        epoch_name.clone(),
                        crate::config::EpochConfig {
                            tau: 8,
                            ordering: "deterministic".to_string(),
                        },
                    );
                }
                
                if let Some(epoch) = config.epochs.get_mut(&epoch_name) {
                    match config_key.as_str() {
                        "tau" => {
                            if let Ok(val) = value.parse::<u64>() {
                                epoch.tau = val;
                            }
                        }
                        "ordering" => {
                            epoch.ordering = value.clone();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "std"))]
pub fn apply_env_overrides(_config: &mut Config, _env_vars: &BTreeMap<String, String>) {
    // In no_std mode, no-op
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_load_env_config() {
        std::env::set_var("KNHK_CONTEXT", "test");
        let env_vars = load_env_config();
        assert!(env_vars.contains_key("context"));
        assert_eq!(env_vars.get("context"), Some(&"test".to_string()));
        std::env::remove_var("KNHK_CONTEXT");
    }
}

