// knhk-config: Configuration management for KNHK
// TOML configuration file support with environment variable overrides
// Production-ready implementation with proper error handling and validation

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub knhk: KnhkConfig,
    #[serde(default)]
    pub connectors: BTreeMap<String, ConnectorConfig>,
    #[serde(default)]
    pub epochs: BTreeMap<String, EpochConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnhkConfig {
    pub version: String,
    pub context: String,
    #[serde(default = "default_max_run_len")]
    pub max_run_len: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    #[serde(rename = "type")]
    pub connector_type: String,
    pub bootstrap_servers: Option<Vec<String>>,
    pub topic: Option<String>,
    pub schema: Option<String>,
    #[serde(default = "default_max_run_len")]
    pub max_run_len: usize,
    pub max_batch_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochConfig {
    #[serde(default = "default_tau")]
    pub tau: u32,
    pub ordering: Option<String>,
}

fn default_max_run_len() -> usize {
    8
}

fn default_tau() -> u32 {
    8
}

#[derive(Debug, Clone)]
pub enum ConfigError {
    ParseError(String),
    IoError(String),
    ValidationFailed(String),
}

impl ConfigError {
    pub fn message(&self) -> &str {
        match self {
            ConfigError::ParseError(msg) => msg,
            ConfigError::IoError(msg) => msg,
            ConfigError::ValidationFailed(msg) => msg,
        }
    }
}

/// Load configuration from file with environment variable overrides
pub fn load_config<P: AsRef<Path>>(config_path: P) -> Result<Config, ConfigError> {
    #[cfg(feature = "std")]
    {
        use std::fs;
        
        // Read config file
        let content = fs::read_to_string(&config_path)
            .map_err(|e| ConfigError::IoError(format!("Failed to read config file: {}", e)))?;
        
        // Parse TOML
        let mut config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("TOML parse error: {}", e)))?;
        
        // Apply environment variable overrides
        apply_env_overrides(&mut config);
        
        // Validate configuration
        validate_config(&config)?;
        
        Ok(config)
    }
    
    #[cfg(not(feature = "std"))]
    {
        Err(ConfigError::IoError("std feature required for file loading".to_string()))
    }
}

/// Load default configuration
pub fn load_default_config() -> Config {
    Config {
        knhk: KnhkConfig {
            version: "0.5.0".to_string(),
            context: "default".to_string(),
            max_run_len: 8,
        },
        connectors: BTreeMap::new(),
        epochs: BTreeMap::new(),
    }
}

/// Apply environment variable overrides
fn apply_env_overrides(config: &mut Config) {
    #[cfg(feature = "std")]
    {
        // Override context
        if let Ok(context) = env::var("KNHK_CONTEXT") {
            config.knhk.context = context;
        }
        
        // Override version
        if let Ok(version) = env::var("KNHK_VERSION") {
            config.knhk.version = version;
        }
        
        // Override max_run_len
        if let Ok(max_run_len) = env::var("KNHK_MAX_RUN_LEN") {
            if let Ok(len) = max_run_len.parse::<usize>() {
                config.knhk.max_run_len = len;
            }
        }
        
        // Override connector configs
        for (name, connector) in &mut config.connectors {
            let prefix = format!("KNHK_CONNECTOR_{}", name.to_uppercase().replace("-", "_"));
            
            if let Ok(connector_type) = env::var(&format!("{}_TYPE", prefix)) {
                connector.connector_type = connector_type;
            }
            
            if let Ok(max_run_len) = env::var(&format!("{}_MAX_RUN_LEN", prefix)) {
                if let Ok(len) = max_run_len.parse::<usize>() {
                    connector.max_run_len = len;
                }
            }
        }
        
        // Override epoch configs
        for (name, epoch) in &mut config.epochs {
            let prefix = format!("KNHK_EPOCH_{}", name.to_uppercase().replace("-", "_"));
            
            if let Ok(tau) = env::var(&format!("{}_TAU", prefix)) {
                if let Ok(t) = tau.parse::<u32>() {
                    epoch.tau = t;
                }
            }
        }
    }
}

/// Validate configuration
fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // Validate max_run_len ≤ 8 (guard constraint)
    if config.knhk.max_run_len > 8 {
        return Err(ConfigError::ValidationFailed(
            format!("max_run_len {} exceeds guard constraint of 8", config.knhk.max_run_len)
        ));
    }
    
    // Validate connector max_run_len ≤ 8
    for (name, connector) in &config.connectors {
        if connector.max_run_len > 8 {
            return Err(ConfigError::ValidationFailed(
                format!("Connector {} max_run_len {} exceeds guard constraint of 8", name, connector.max_run_len)
            ));
        }
    }
    
    // Validate epoch tau ≤ 8 (guard constraint)
    for (name, epoch) in &config.epochs {
        if epoch.tau > 8 {
            return Err(ConfigError::ValidationFailed(
                format!("Epoch {} tau {} exceeds guard constraint of 8", name, epoch.tau)
            ));
        }
    }
    
    Ok(())
}

/// Get default config file path
#[cfg(feature = "std")]
pub fn get_default_config_path() -> std::path::PathBuf {
    use std::path::PathBuf;
    
    // Try $HOME/.knhk/config.toml on Unix
    #[cfg(unix)]
    {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(".knhk").join("config.toml");
        }
    }
    
    // Try %APPDATA%/knhk/config.toml on Windows
    #[cfg(windows)]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            return PathBuf::from(appdata).join("knhk").join("config.toml");
        }
    }
    
    // Fallback: current directory
    PathBuf::from("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = load_default_config();
        assert_eq!(config.knhk.version, "0.5.0");
        assert_eq!(config.knhk.context, "default");
        assert_eq!(config.knhk.max_run_len, 8);
    }
}

