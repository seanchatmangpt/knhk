// knhk-config v0.5.0 — TOML configuration system
// Configuration loading hierarchy: env > file > defaults
// Supports connectors, epochs, hooks, routes, and general settings

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub knhk: KnhkConfig,
    #[serde(default)]
    pub connectors: HashMap<String, ConnectorConfig>,
    #[serde(default)]
    pub epochs: HashMap<String, EpochConfig>,
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub routes: HashMap<String, RouteConfig>,
}

/// KNHK general configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnhkConfig {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_context")]
    pub context: String,
}

fn default_version() -> String {
    "0.5.0".to_string()
}

fn default_context() -> String {
    "default".to_string()
}

impl Default for KnhkConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            context: default_context(),
        }
    }
}

/// Connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub r#type: String,
    #[serde(default)]
    pub bootstrap_servers: Vec<String>,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub schema: String,
    #[serde(default = "default_max_run_len")]
    pub max_run_len: u64,
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: u64,
}

fn default_max_run_len() -> u64 {
    8
}

fn default_max_batch_size() -> u64 {
    1000
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        Self {
            r#type: "kafka".to_string(),
            bootstrap_servers: vec!["localhost:9092".to_string()],
            topic: "triples".to_string(),
            schema: "urn:knhk:schema:enterprise".to_string(),
            max_run_len: default_max_run_len(),
            max_batch_size: default_max_batch_size(),
        }
    }
}

/// Epoch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochConfig {
    #[serde(default = "default_tau")]
    pub tau: u64,
    #[serde(default = "default_ordering")]
    pub ordering: String,
}

fn default_tau() -> u64 {
    8
}

fn default_ordering() -> String {
    "deterministic".to_string()
}

impl Default for EpochConfig {
    fn default() -> Self {
        Self {
            tau: default_tau(),
            ordering: default_ordering(),
        }
    }
}

/// Hooks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(default = "default_max_hooks")]
    pub max_count: u64,
}

fn default_max_hooks() -> u64 {
    100
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            max_count: default_max_hooks(),
        }
    }
}

/// Route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub kind: String,
    pub target: String,
    #[serde(default = "default_encode")]
    pub encode: String,
}

fn default_encode() -> String {
    "json-ld".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            knhk: KnhkConfig::default(),
            connectors: HashMap::new(),
            epochs: HashMap::new(),
            hooks: HooksConfig::default(),
            routes: HashMap::new(),
        }
    }
}

/// Configuration loader with environment variable override support
pub struct ConfigLoader;

impl ConfigLoader {
    /// Get configuration directory path
    pub fn config_dir() -> Result<PathBuf, String> {
        #[cfg(target_os = "windows")]
        {
            let appdata = env::var("APPDATA")
                .map_err(|_| "APPDATA not set")?;
            Ok(PathBuf::from(appdata).join("knhk"))
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let home = env::var("HOME")
                .map_err(|_| "HOME not set")?;
            Ok(PathBuf::from(home).join(".knhk"))
        }
    }

    /// Get configuration file path
    pub fn config_file() -> Result<PathBuf, String> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load configuration with hierarchy: env > file > defaults
    pub fn load() -> Result<Config, ConfigError> {
        let mut config = Self::load_from_file().unwrap_or_else(|_| Config::default());
        
        // Apply environment variable overrides
        Self::apply_env_overrides(&mut config);
        
        // Validate configuration
        Self::validate(&config)?;
        
        Ok(config)
    }

    /// Load configuration from file
    fn load_from_file() -> Result<Config, ConfigError> {
        let config_file = Self::config_file()?;
        
        if !config_file.exists() {
            return Err(ConfigError::FileNotFound(config_file));
        }
        
        let content = fs::read_to_string(&config_file)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(config: &mut Config) {
        // KNHK_CONTEXT
        if let Ok(context) = env::var("KNHK_CONTEXT") {
            config.knhk.context = context;
        }

        // KNHK_CONNECTOR_* pattern
        for (key, value) in env::vars() {
            if key.starts_with("KNHK_CONNECTOR_") {
                // Parse connector name and field from key
                // Format: KNHK_CONNECTOR_<NAME>_<FIELD>
                let parts: Vec<&str> = key.trim_start_matches("KNHK_CONNECTOR_").splitn(2, '_').collect();
                if parts.len() == 2 {
                    let connector_name = parts[0].to_lowercase();
                    let field = parts[1].to_lowercase();
                    
                    let connector = config.connectors
                        .entry(connector_name.clone())
                        .or_insert_with(ConnectorConfig::default);
                    
                    match field.as_str() {
                        "BOOTSTRAP_SERVERS" => {
                            connector.bootstrap_servers = value.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        "TOPIC" => connector.topic = value,
                        "SCHEMA" => connector.schema = value,
                        "MAX_RUN_LEN" => {
                            if let Ok(len) = value.parse::<u64>() {
                                connector.max_run_len = len;
                            }
                        }
                        "MAX_BATCH_SIZE" => {
                            if let Ok(size) = value.parse::<u64>() {
                                connector.max_batch_size = size;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // KNHK_EPOCH_* pattern
        for (key, value) in env::vars() {
            if key.starts_with("KNHK_EPOCH_") {
                let parts: Vec<&str> = key.trim_start_matches("KNHK_EPOCH_").splitn(2, '_').collect();
                if parts.len() == 2 {
                    let epoch_name = parts[0].to_lowercase();
                    let field = parts[1].to_lowercase();
                    
                    let epoch = config.epochs
                        .entry(epoch_name.clone())
                        .or_insert_with(EpochConfig::default);
                    
                    match field.as_str() {
                        "TAU" => {
                            if let Ok(tau) = value.parse::<u64>() {
                                epoch.tau = tau;
                            }
                        }
                        "ORDERING" => epoch.ordering = value,
                        _ => {}
                    }
                }
            }
        }
    }

    /// Validate configuration
    fn validate(config: &Config) -> Result<(), ConfigError> {
        // Validate max_run_len ≤ 8
        for (name, connector) in &config.connectors {
            if connector.max_run_len > 8 {
                return Err(ConfigError::ValidationError(format!(
                    "Connector '{}' max_run_len {} exceeds limit 8",
                    name, connector.max_run_len
                )));
            }
        }

        // Validate tau ≤ 8
        for (name, epoch) in &config.epochs {
            if epoch.tau > 8 {
                return Err(ConfigError::ValidationError(format!(
                    "Epoch '{}' tau {} exceeds limit 8",
                    name, epoch.tau
                )));
            }
        }

        Ok(())
    }

    /// Save configuration to file
    pub fn save(config: &Config) -> Result<(), ConfigError> {
        let config_file = Self::config_file()?;
        let config_dir = config_file.parent().ok_or_else(|| {
            ConfigError::SaveError("Invalid config file path".to_string())
        })?;
        
        fs::create_dir_all(config_dir)
            .map_err(|e| ConfigError::SaveError(format!("Failed to create config directory: {}", e)))?;
        
        let content = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::SaveError(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&config_file, content)
            .map_err(|e| ConfigError::SaveError(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
}

/// Configuration error types
#[derive(Debug, Clone)]
pub enum ConfigError {
    FileNotFound(PathBuf),
    ReadError(String),
    ParseError(String),
    ValidationError(String),
    SaveError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => {
                write!(f, "Config file not found: {}", path.display())
            }
            ConfigError::ReadError(msg) => write!(f, "Failed to read config: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Failed to parse config: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Config validation failed: {}", msg),
            ConfigError::SaveError(msg) => write!(f, "Failed to save config: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.knhk.version, "0.5.0");
        assert_eq!(config.knhk.context, "default");
    }

    #[test]
    fn test_config_dir() {
        let dir = ConfigLoader::config_dir();
        assert!(dir.is_ok());
    }

    #[test]
    fn test_validation() {
        let mut config = Config::default();
        let mut connector = ConnectorConfig::default();
        connector.max_run_len = 9; // Invalid
        config.connectors.insert("test".to_string(), connector);
        
        assert!(ConfigLoader::validate(&config).is_err());
    }
}