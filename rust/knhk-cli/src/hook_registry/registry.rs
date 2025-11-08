//! Hook registry integration - Integrates with knhk-etl HookRegistry

use crate::hook_registry::store::HookStore;

use knhk_etl::HookRegistry;
use std::sync::Arc;

/// Hook registry integration - Manages hooks with system
pub struct HookRegistryIntegration {
    registry: Arc<HookRegistry>,
    store: HookStore,
}

impl HookRegistryIntegration {
    /// Create new hook registry integration
    pub fn new() -> Result<Self, String> {
        let registry = Arc::new(HookRegistry::new());
        let store = HookStore::new()?;

        let mut integration = Self { registry, store };
        integration.load_hooks()?;

        Ok(integration)
    }

    /// Load hooks from storage into registry
    fn load_hooks(&mut self) -> Result<(), String> {
        let hooks = self.store.load_all()?;

        for hook in hooks {
            // Register hook with registry
            // Implementation depends on HookRegistry API
        }

        Ok(())
    }

    /// Get hook registry
    pub fn registry(&self) -> &Arc<HookRegistry> {
        &self.registry
    }

    /// Register a hook
    pub fn register(&mut self, hook: crate::hook_registry::store::HookEntry) -> Result<(), String> {
        // Save to store
        self.store.save(&hook)?;

        // Register with knhk-etl HookRegistry
        // Implementation depends on HookRegistry API
        // For now, just save to store

        Ok(())
    }

    /// Get a hook by name
    pub fn get(&self, name: &str) -> Result<crate::hook_registry::store::HookEntry, String> {
        self.store.load(name)
    }

    /// List all hook names
    pub fn list(&self) -> Result<Vec<String>, String> {
        let hooks = self.store.load_all()?;
        Ok(hooks.iter().map(|h| h.name.clone()).collect())
    }
}

impl Default for HookRegistryIntegration {
    fn default() -> Self {
        Self::new().expect("Failed to create hook registry integration")
    }
}
