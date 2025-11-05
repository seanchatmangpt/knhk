// Configuration loading and management

use crate::schema::*;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[cfg(feature = "std")]
use std::path::PathBuf;

/// Load configuration from file and environment variables
/// 
/// Loading hierarchy: env > file > defaults
#[cfg(feature = "std")]
pub fn load_config() -> Result<KnhkConfig, ConfigError> {
    let config_file = get_config_file_path()?;
    
    let mut config = if config_file.exists() {
        load_from_file(&config_file)?
    } else {
        KnhkConfig::default()
    };
    
    // Apply environment variable overrides
    apply_env_overrides(&mut config)?;
    
    // Validate configuration
    validate_config(&config)?;
    
    Ok(config)
}

/// Load configuration from file path
#[cfg(feature = "std")]
pub fn load_from_file(path: &PathBuf) -> Result<KnhkConfig, ConfigError> {
    use std::fs;
    
    let content = fs::read_to_string(path)
        .map_err(|e| ConfigError::IoError(e.to_string()))?;
    
    toml::from_str(&content)
        .map_err(|e| ConfigError::ParseError(e.to_string()))
}

/// Get default configuration file path
#[cfg(feature = "std")]
pub fn get_config_file_path() -> Result<PathBuf, ConfigError> {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| ConfigError::IoError("APPDATA not set".to_string()))?;
        let mut path = PathBuf::from(appdata);
        path.push("knhk");
        path.push("config.toml");
        Ok(path)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME")
            .map_err(|_| ConfigError::IoError("HOME not set".to_string()))?;
        let mut path = PathBuf::from(home);
        path.push(".knhk");
        path.push("config.toml");
        Ok(path)
    }
}

/// Apply environment variable overrides to configuration
#[cfg(feature = "std")]
pub fn apply_env_overrides(config: &mut KnhkConfig) -> Result<(), ConfigError> {
    use std::env;
    
    // Override context
    if let Ok(context) = env::var("KNHK_CONTEXT") {
        config.knhk.context = context;
    }
    
    // Override connector settings
    for (key, value) in env::vars() {
        if key.starts_with("KNHK_CONNECTOR_") {
            // Parse KNHK_CONNECTOR_<NAME>_<SETTING>=value
            let parts: Vec<&str> = key.strip_prefix("KNHK_CONNECTOR_")
                .unwrap()
                .splitn(2, '_')
                .collect();
            
            if parts.len() == 2 {
                let connector_name = parts[0].to_lowercase();
                let setting = parts[1].to_lowercase();
                
                let connector = config.connectors
                    .entry(connector_name)
                    .or_insert_with(ConnectorConfig::default);
                
                match setting.as_str() {
                    "bootstrap_servers" => {
                        connector.bootstrap_servers = value.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    "topic" => connector.topic = value,
                    "schema" => connector.schema = value,
                    "max_run_len" => {
                        connector.max_run_len = value.parse()
                            .map_err(|_| ConfigError::ValidationError(
                                format!("Invalid max_run_len: {}", value)
                            ))?;
                    }
                    "max_batch_size" => {
                        connector.max_batch_size = value.parse()
                            .map_err(|_| ConfigError::ValidationError(
                                format!("Invalid max_batch_size: {}", value)
                            ))?;
                    }
                    _ => {}
                }
            }
        }
        
        // Override epoch settings
        if key.starts_with("KNHK_EPOCH_") {
            let parts: Vec<&str> = key.strip_prefix("KNHK_EPOCH_")
                .unwrap()
                .splitn(2, '_')
                .collect();
            
            if parts.len() == 2 {
                let epoch_name = parts[0].to_lowercase();
                let setting = parts[1].to_lowercase();
                
                let epoch = config.epochs
                    .entry(epoch_name)
                    .or_insert_with(EpochConfig::default);
                
                match setting.as_str() {
                    "tau" => {
                        epoch.tau = value.parse()
                            .map_err(|_| ConfigError::ValidationError(
                                format!("Invalid tau: {}", value)
                            ))?;
                    }
                    "ordering" => epoch.ordering = value,
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}

/// Validate configuration
pub fn validate_config(config: &KnhkConfig) -> Result<(), ConfigError> {
    // Validate max_run_len ≤ 8 (Chatman Constant)
    for (name, connector) in &config.connectors {
        if connector.max_run_len > 8 {
            return Err(ConfigError::ValidationError(
                format!("Connector {} max_run_len {} exceeds 8", name, connector.max_run_len)
            ));
        }
    }
    
    // Validate tau ≤ 8
    for (name, epoch) in &config.epochs {
        if epoch.tau > 8 {
            return Err(ConfigError::ValidationError(
                format!("Epoch {} tau {} exceeds 8", name, epoch.tau)
            ));
        }
    }
    
    // Validate route endpoints
    for (name, route) in &config.routes {
        if route.target.is_empty() {
            return Err(ConfigError::ValidationError(
                format!("Route {} target cannot be empty", name)
            ));
        }
        
        // Validate endpoint format
        if !route.target.starts_with("http://") 
            && !route.target.starts_with("https://")
            && !route.target.starts_with("kafka://")
            && !route.target.starts_with("grpc://") {
            return Err(ConfigError::ValidationError(
                format!("Route {} target must be http://, https://, kafka://, or grpc://", name)
            ));
        }
    }
    
    Ok(())
}

impl Default for KnhkConfig {
    fn default() -> Self {
        Self {
            knhk: KnhkSection::default(),
            connectors: BTreeMap::new(),
            epochs: BTreeMap::new(),
            hooks: HooksSection::default(),
            routes: BTreeMap::new(),
        }
    }
}

#[cfg(not(feature = "std"))]
pub fn load_config() -> Result<KnhkConfig, ConfigError> {
    // In no_std mode, return default configuration
    Ok(KnhkConfig::default())
}

