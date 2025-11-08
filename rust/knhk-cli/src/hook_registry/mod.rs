//! Hook registry integration - Integrates with knhk-etl HookRegistry

pub mod store;

pub use store::HookStore;

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
}

impl Default for HookRegistryIntegration {
    fn default() -> Self {
        Self::new().expect("Failed to create hook registry integration")
    }
}
