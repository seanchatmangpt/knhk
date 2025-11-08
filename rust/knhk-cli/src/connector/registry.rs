//! Connector registry - Manages connector instances

use crate::commands::connect;
use knhk_connectors::Connector;
use std::collections::HashMap;
use std::sync::Arc;

// Re-export for use in save_to_storage
use connect::{save_connectors, ConnectorStorage, ConnectorStorageEntry};

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
            match crate::connector::factory::ConnectorFactory::create(&entry.source) {
                Ok(connector) => {
                    self.connectors.insert(entry.name, Arc::from(connector));
                }
                Err(e) => {
                    // Log error but continue loading other connectors
                    eprintln!("Warning: Failed to load connector '{}': {}", entry.name, e);
                }
            }
        }

        Ok(())
    }

    /// Save connectors to storage
    fn save_to_storage(&self) -> Result<(), String> {
        // Load existing storage to preserve source/schema info
        let existing_storage = connect::load_connectors().unwrap_or_else(|_| ConnectorStorage {
            connectors: Vec::new(),
        });

        // Create map of existing entries
        let mut entry_map: std::collections::HashMap<String, ConnectorStorageEntry> =
            existing_storage
                .connectors
                .into_iter()
                .map(|e| (e.name.clone(), e))
                .collect();

        // Keep only entries for connectors that exist in registry
        entry_map.retain(|name, _| self.connectors.contains_key(name));

        let storage = ConnectorStorage {
            connectors: entry_map.into_values().collect(),
        };

        save_connectors(&storage)
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
        self.connectors.insert(name.clone(), Arc::from(connector));

        // Save to persistent storage
        self.save_to_storage()?;

        Ok(())
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create connector registry")
    }
}
