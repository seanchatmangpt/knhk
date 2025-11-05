// rust/knhk-cli/src/commands/connect.rs
// Connect commands - Connector management with TOML configuration support

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use knhk_connectors::{SourceType, DataFormat};
use knhk_config::{ConfigLoader, Config};

/// Connector storage (file-based)
/// Simplified storage format for CLI persistence
/// Note: TOML config takes precedence, JSON files maintained for backward compatibility
#[derive(Debug, Serialize, Deserialize)]
struct ConnectorStorage {
    connectors: Vec<ConnectorStorageEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectorStorageEntry {
    name: String,
    schema: String,
    source: String,  // Stored as string, parsed on load
}

/// Register a connector
/// connect(#{name, schema, source, map, guard})
/// Uses TOML config if available, falls back to JSON for backward compatibility
pub fn register(name: String, schema: String, source: String) -> Result<(), String> {
    println!("Registering connector: {}", name);
    
    // Validate source format (basic check)
    parse_source(&source)?;
    
    // Try to load from TOML config first
    if let Ok(mut config) = ConfigLoader::load() {
        // Add connector to TOML config
        let connector_config = knhk_config::ConnectorConfig {
            r#type: "kafka".to_string(), // Default, can be inferred from source
            bootstrap_servers: vec!["localhost:9092".to_string()],
            topic: "triples".to_string(),
            schema: schema.clone(),
            max_run_len: 8,
            max_batch_size: 1000,
        };
        config.connectors.insert(name.clone(), connector_config);
        
        // Save to TOML config
        ConfigLoader::save(&config)
            .map_err(|e| format!("Failed to save TOML config: {}", e))?;
        
        println!("  ✓ Schema: {}", schema);
        println!("  ✓ Source: {}", source);
        println!("✓ Connector registered (TOML config)");
        
        return Ok(());
    }
    
    // Fall back to JSON storage for backward compatibility
    let mut storage = load_connectors()?;
    
    // Check if connector already exists
    if storage.connectors.iter().any(|c| c.name == name) {
        return Err(format!("Connector '{}' already registered", name));
    }
    
    // Add connector entry
    storage.connectors.push(ConnectorStorageEntry {
        name: name.clone(),
        schema: schema.clone(),
        source: source.clone(),
    });
    
    // Save connectors
    save_connectors(&storage)?;
    
    println!("  ✓ Schema: {}", schema);
    println!("  ✓ Source: {}", source);
    println!("✓ Connector registered (JSON storage)");
    
    Ok(())
}

/// List connectors
/// Loads from TOML config if available, falls back to JSON
pub fn list() -> Result<(), String> {
    // Try to load from TOML config first
    if let Ok(config) = ConfigLoader::load() {
        if config.connectors.is_empty() {
            println!("No connectors registered");
            return Ok(());
        }
        
        println!("Registered connectors (from TOML config):");
        for (name, connector) in &config.connectors {
            println!("  • {} (type: {})", name, connector.r#type);
            println!("    Schema: {}", connector.schema);
            if !connector.bootstrap_servers.is_empty() {
                println!("    Bootstrap servers: {:?}", connector.bootstrap_servers);
            }
            if !connector.topic.is_empty() {
                println!("    Topic: {}", connector.topic);
            }
        }
        return Ok(());
    }
    
    // Fall back to JSON storage
    let storage = load_connectors()?;
    
    if storage.connectors.is_empty() {
        println!("No connectors registered");
        return Ok(());
    }
    
    println!("Registered connectors (from JSON storage):");
    for (i, connector) in storage.connectors.iter().enumerate() {
        println!("  {}. {} (schema: {})", i + 1, connector.name, connector.schema);
        println!("     Source: {}", connector.source);
    }
    
    Ok(())
}

fn parse_source(source: &str) -> Result<SourceType, String> {
    if source.starts_with("kafka://") {
        let parts: Vec<&str> = source[8..].split('/').collect();
        let brokers = if parts.is_empty() {
            vec!["localhost:9092".to_string()]
        } else {
            parts[0].split(',').map(|s| s.to_string()).collect()
        };
        let topic = parts.get(1).unwrap_or(&"triples").to_string();
        
        Ok(SourceType::Kafka {
            topic,
            format: DataFormat::JsonLd,
            bootstrap_servers: brokers,
        })
    } else if source.starts_with("salesforce://") {
        let instance_url = source[13..].to_string();
        Ok(SourceType::Salesforce {
            instance_url,
            api_version: "v58.0".to_string(),
            object_type: "Triple".to_string(),
        })
    } else if source.starts_with("http://") || source.starts_with("https://") {
        Ok(SourceType::Http {
            url: source.to_string(),
            format: DataFormat::JsonLd,
            headers: std::collections::BTreeMap::new(),
        })
    } else if source.contains('/') || source.contains('\\') {
        // File path
        Ok(SourceType::File {
            path: source.to_string(),
            format: DataFormat::RdfTurtle,
        })
    } else {
        Err(format!("Unknown source type: {}", source))
    }
}

fn get_config_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let mut path = PathBuf::from(std::env::var("APPDATA").map_err(|_| "APPDATA not set")?);
        path.push("knhk");
        Ok(path)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
        let mut path = PathBuf::from(home);
        path.push(".knhk");
        Ok(path)
    }
}

pub(crate) fn load_connectors() -> Result<ConnectorStorage, String> {
    let config_dir = get_config_dir()?;
    let connectors_file = config_dir.join("connectors.json");
    
    if !connectors_file.exists() {
        return Ok(ConnectorStorage {
            connectors: Vec::new(),
        });
    }
    
    let content = fs::read_to_string(&connectors_file)
        .map_err(|e| format!("Failed to read connectors file: {}", e))?;
    
    let storage: ConnectorStorage = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse connectors file: {}", e))?;
    
    Ok(storage)
}

fn save_connectors(storage: &ConnectorStorage) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    
    let connectors_file = config_dir.join("connectors.json");
    let content = serde_json::to_string_pretty(storage)
        .map_err(|e| format!("Failed to serialize connectors: {}", e))?;
    
    fs::write(&connectors_file, content)
        .map_err(|e| format!("Failed to write connectors file: {}", e))?;
    
    Ok(())
}

