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
        // Load connectors from persistent storage
        // For now, connectors are loaded on-demand when registered
        // Future: Load from Oxigraph or file-based storage
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
    pub fn list(&self) -> Result<Vec<String>, String> {
        Ok(self.connectors.keys().cloned().collect())
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
