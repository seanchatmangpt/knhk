// rust/knhk-cli/src/commands/connect.rs
// Connect commands - Connector management with ConnectorRegistry

use knhk_connectors::{DataFormat, SourceType};

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
