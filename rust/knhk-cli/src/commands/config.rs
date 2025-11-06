// rust/knhk-cli/src/commands/config.rs
// Config command - show current configuration

use knhk_config::{load_config, load_env_config, apply_env_overrides};
use serde_json;

/// Show current configuration
pub fn show() -> Result<String, String> {
    // Load config file
    let mut config = load_config(None)?;
    
    // Load environment variables
    let env_vars = load_env_config();
    
    // Apply environment variable overrides
    apply_env_overrides(&mut config, &env_vars);
    
    // Serialize to JSON string
    serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))
}

