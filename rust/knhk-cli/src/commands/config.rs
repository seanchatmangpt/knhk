// rust/knhk-cli/src/commands/config.rs
// Config command - show current configuration

use knhk_config::{load_config, load_env_config, apply_env_overrides};

/// Show current configuration
pub fn show() -> Result<(), String> {
    println!("Loading configuration...");
    
    // Load config file
    let mut config = load_config(None)?;
    
    // Load environment variables
    let env_vars = load_env_config();
    
    // Apply environment variable overrides
    apply_env_overrides(&mut config, &env_vars);
    
    // Display configuration
    println!("KNHK Configuration:");
    println!("  Version: {}", config.knhk.version);
    println!("  Context: {}", config.knhk.context);
    
    if !config.connectors.is_empty() {
        println!("\nConnectors:");
        for (name, connector) in &config.connectors {
            println!("  {}: type={}, schema={:?}", 
                name, connector.r#type, connector.schema);
        }
    }
    
    if !config.epochs.is_empty() {
        println!("\nEpochs:");
        for (name, epoch) in &config.epochs {
            println!("  {}: tau={}, ordering={}", 
                name, epoch.tau, epoch.ordering);
        }
    }
    
    if !config.routes.is_empty() {
        println!("\nRoutes:");
        for (name, route) in &config.routes {
            println!("  {}: kind={}, target={}", 
                name, route.kind, route.target);
        }
    }
    
    println!("\n✓ Configuration loaded successfully");
    Ok(())
}

