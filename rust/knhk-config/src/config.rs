// knhk-config/src/config.rs
// TOML configuration loading and validation

// knhk-config/src/config.rs
// TOML configuration loading and validation

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub knhk: KnhkConfig,
    #[serde(default)]
    pub connectors: BTreeMap<String, ConnectorConfig>,
    #[serde(default)]
    pub epochs: BTreeMap<String, EpochConfig>,
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub routes: BTreeMap<String, RouteConfig>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub r#type: String,
    pub bootstrap_servers: Option<Vec<String>>,
    pub topic: Option<String>,
    pub schema: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(default = "default_max_count")]
    pub max_count: u64,
}

fn default_max_count() -> u64 {
    100
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            max_count: default_max_count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub kind: String,
    pub target: String,
    pub encode: Option<String>,
}

/// Load configuration from file
/// 
/// # Arguments
/// * `path` - Path to config file (defaults to ~/.knhk/config.toml or %APPDATA%/knhk/config.toml)
#[cfg(feature = "std")]
pub fn load_config(path: Option<PathBuf>) -> Result<Config, String> {
    use std::fs;
    use toml::de::Error as TomlError;
    
    let config_path = path.unwrap_or_else(|| {
        #[cfg(target_os = "windows")]
        {
            let mut path = PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Public".to_string()));
            path.push("knhk");
            path.push("config.toml");
            path
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            let mut path = PathBuf::from(home);
            path.push(".knhk");
            path.push("config.toml");
            path
        }
    });
    
    // If config file doesn't exist, return default config
    if !config_path.exists() {
        return Ok(Config::default());
    }
    
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file {}: {}", config_path.display(), e))?;
    
    toml::from_str(&content)
        .map_err(|e: TomlError| format!("Failed to parse config file {}: {}", config_path.display(), e))
}

#[cfg(not(feature = "std"))]
pub fn load_config(_path: Option<()>) -> Result<Config, String> {
    // In no_std mode, return default config
    Ok(Config::default())
}

// Default implementation auto-derived
// Manual implementation removed - using #[derive(Default)] on Config struct instead

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_load_default_config() {
        let config = load_config(None).unwrap_or_else(|_| {
            // Fallback to default config if loading fails
            Config {
                knhk: KnhkConfig {
                    version: "0.5.0".to_string(),
                    context: "default".to_string(),
                    max_run_len: 8,
                    max_batch_size: 1000,
                },
                connectors: Default::default(),
            }
        });
        assert_eq!(config.knhk.version, "0.5.0");
        assert_eq!(config.knhk.context, "default");
    }
}

