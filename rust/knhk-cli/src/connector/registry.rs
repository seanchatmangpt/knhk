//! Connector registry - Manages connector instances

use crate::commands::connect;
use knhk_connectors::Connector;
use std::collections::HashMap;
use std::sync::Arc;

/// Connector registry - Manages connector instances
pub struct ConnectorRegistry {
    connectors: HashMap<String, Arc<dyn Connector>>,
}

impl ConnectorRegistry {
    /// Create new connector registry
    pub fn new() -> Result<Self, String> {
        let mut registry = Self {
            connectors: HashMap::new(),
        };

        // Load connectors from storage
        registry.load_connectors()?;

        Ok(registry)
    }

    /// Load connectors from storage
    fn load_connectors(&mut self) -> Result<(), String> {
        let storage = connect::load_connectors()?;

        for entry in storage.connectors {
            // Create connector instance from storage entry
            let connector = crate::connector::factory::ConnectorFactory::create(&entry.source)?;
            self.connectors.insert(entry.name, Arc::from(connector));
        }

        Ok(())
    }

    /// Get connector by name
    pub fn get(&self, name: &str) -> Result<Arc<dyn Connector>, String> {
        self.connectors
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Connector '{}' not found", name))
    }

    /// List all connector names
    pub fn list(&self) -> Vec<String> {
        self.connectors.keys().cloned().collect()
    }

    /// Register a new connector
    pub fn register(&mut self, name: String, source: String) -> Result<(), String> {
        if self.connectors.contains_key(&name) {
            return Err(format!("Connector '{}' already registered", name));
        }

        let connector = crate::connector::factory::ConnectorFactory::create(&source)?;
        self.connectors.insert(name, Arc::from(connector));

        Ok(())
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create connector registry")
    }
}
