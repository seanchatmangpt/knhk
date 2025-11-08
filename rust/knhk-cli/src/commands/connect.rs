// rust/knhk-cli/src/commands/connect.rs
// Connect commands - Connector management with ConnectorRegistry

use knhk_connectors::SourceType;
use std::fs;
use std::path::PathBuf;

/// Register a connector
/// connect(#{name, schema, source, map, guard})
/// Uses ConnectorRegistry for persistent storage
pub fn register(name: String, schema: String, source: String) -> Result<(), String> {
    #[cfg(feature = "otel")]
    let _span = tracing::span!(
        tracing::Level::INFO,
        "knhk.connect.register",
        knhk.operation.name = "connect.register",
        knhk.operation.type = "configuration",
        connector.name = %name
    );
    #[cfg(feature = "otel")]
    let _enter = _span.enter();

    println!("Registering connector: {}", name);

    // Validate source format (basic check)
    parse_source(&source)?;

    // Use ConnectorRegistry for persistent storage
    use crate::connector::ConnectorRegistry;
    let mut registry = ConnectorRegistry::new()?;

    // Check if connector already exists
    if registry.get(&name).is_ok() {
        return Err(format!("Connector '{}' already registered", name));
    }

    // Register connector (registry handles ConnectorSpec creation internally)
    registry.register(name.clone(), source.clone())?;

    // Save connector metadata to storage
    let mut storage = load_connectors()?;
    if !storage.connectors.iter().any(|c| c.name == name) {
        storage.connectors.push(ConnectorStorageEntry {
            name: name.clone(),
            schema: schema.clone(),
            source: source.clone(),
        });
        save_connectors(&storage)?;
    }

    println!("  ✓ Schema: {}", schema);
    println!("  ✓ Source: {}", source);
    println!("✓ Connector registered");

    #[cfg(feature = "otel")]
    {
        use knhk_otel::{MetricsHelper, Tracer};
        let mut tracer = Tracer::new();
        MetricsHelper::record_operation(&mut tracer, "connect.register", true);
    }

    Ok(())
}

/// List connectors
pub fn list() -> Result<Vec<String>, String> {
    #[cfg(feature = "otel")]
    let _span = tracing::span!(
        tracing::Level::INFO,
        "knhk.connect.list",
        knhk.operation.name = "connect.list",
        knhk.operation.type = "query"
    );
    #[cfg(feature = "otel")]
    let _enter = _span.enter();

    // Use ConnectorRegistry
    use crate::connector::ConnectorRegistry;
    let registry = ConnectorRegistry::new()?;
    registry.list()
}

fn parse_source(source: &str) -> Result<SourceType, String> {
    // Validate source format - delegate to ConnectorFactory
    use crate::connector::factory::ConnectorFactory;
    ConnectorFactory::parse_source(source)
}

/// Load connectors from storage (for ConnectorRegistry)
pub(crate) fn load_connectors() -> Result<ConnectorStorage, String> {
    let config_dir = get_config_dir()?;
    let storage_path = config_dir.join("connectors.json");

    if !storage_path.exists() {
        return Ok(ConnectorStorage { connectors: vec![] });
    }

    let content = fs::read_to_string(&storage_path)
        .map_err(|e| format!("Failed to read connector storage: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse connector storage: {}", e))
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConnectorStorage {
    pub connectors: Vec<ConnectorStorageEntry>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConnectorStorageEntry {
    pub name: String,
    pub schema: String,
    pub source: String,
}

/// Save connectors to storage
pub(crate) fn save_connectors(storage: &ConnectorStorage) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let storage_path = config_dir.join("connectors.json");
    let content = serde_json::to_string_pretty(storage)
        .map_err(|e| format!("Failed to serialize connector storage: {}", e))?;

    std::fs::write(&storage_path, content)
        .map_err(|e| format!("Failed to write connector storage: {}", e))?;

    Ok(())
}
