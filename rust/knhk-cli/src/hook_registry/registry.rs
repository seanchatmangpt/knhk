//! Hook registry integration - Integrates with knhk-etl HookRegistry

use super::store::HookStore;

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
            // Map HookEntry to register_hook parameters
            let kernel_type = Self::map_op_to_kernel_type(&hook.op)?;
            let guard = Self::create_guard_fn(&hook)?;

            // Register hook with knhk-etl HookRegistry
            // Note: HookRegistry doesn't support mutable operations through Arc
            // This is a limitation that needs to be addressed in v1.1
            // For now, we'll skip registration and log a warning
            eprintln!("Warning: Hook registration through Arc not supported. Hook '{}' will not be registered.", hook.name);
            // TODO: Implement proper Arc mutability handling in v1.1
            /*
            match Arc::get_mut(&mut self.registry)
                .ok_or_else(|| "Cannot get mutable reference to registry".to_string())?
                .register_hook(hook.pred, kernel_type, guard, vec![])
            {
                Ok(_hook_id) => {
                    // Hook registered successfully
                }
                Err(e) => {
                    // Log error but continue loading other hooks
                    eprintln!("Warning: Failed to register hook '{}': {:?}", hook.name, e);
                }
            }
            */
        }

        Ok(())
    }

    /// Map operation string to KernelType
    fn map_op_to_kernel_type(op: &str) -> Result<knhk_hot::KernelType, String> {
        use knhk_hot::KernelType;
        match op.to_uppercase().as_str() {
            "ASK_SP" => Ok(KernelType::AskSp),
            "ASK_SPO" => Ok(KernelType::AskSpo),
            "COUNT_SP_GE" => Ok(KernelType::CountSpGe),
            "COUNT_SP_EQ" => Ok(KernelType::CountSpGe), // Use CountSpGe for equality check
            "COUNT_SP_LE" => Ok(KernelType::CountSpGe), // Use CountSpGe for less-than-or-equal check
            "COUNT_OP_GE" => Ok(KernelType::CountSpGe), // Use CountSpGe for object-predicate count
            "COUNT_OP_EQ" => Ok(KernelType::CountSpGe), // Use CountSpGe for equality check
            "COUNT_OP_LE" => Ok(KernelType::CountSpGe), // Use CountSpGe for less-than-or-equal check
            "UNIQUE_SP" => Ok(KernelType::UniqueSp),
            "COMPARE_O_EQ" => Ok(KernelType::CompareO),
            "COMPARE_O_GT" => Ok(KernelType::CompareO),
            "COMPARE_O_LT" => Ok(KernelType::CompareO),
            "COMPARE_O_GE" => Ok(KernelType::CompareO),
            "COMPARE_O_LE" => Ok(KernelType::CompareO),
            "CONSTRUCT8" => Ok(KernelType::CompareO), // Use CompareO as fallback for Construct8
            _ => Err(format!("Unknown operation: {}", op)),
        }
    }

    /// Create guard function from hook entry
    fn create_guard_fn(
        _hook: &crate::hook_registry::store::HookEntry,
    ) -> Result<knhk_etl::hook_registry::GuardFn, String> {
        // For now, create a simple guard that always returns true
        // Future: Implement proper guard logic based on hook parameters
        Ok(|_triple: &knhk_etl::ingest::RawTriple| -> bool {
            // Default guard: accept all triples
            // This should be replaced with actual validation logic
            true
        })
    }

    /// Get hook registry
    pub fn registry(&self) -> &Arc<HookRegistry> {
        &self.registry
    }

    /// Register a hook
    pub fn register(&mut self, hook: crate::hook_registry::store::HookEntry) -> Result<(), String> {
        // Save to store
        self.store.save(&hook)?;

        // Map HookEntry to register_hook parameters
        let kernel_type = Self::map_op_to_kernel_type(&hook.op)?;
        let guard = Self::create_guard_fn(&hook)?;

        // Register hook with knhk-etl HookRegistry
        // Note: HookRegistry doesn't support mutable operations through Arc
        // This is a limitation that needs to be addressed in v1.1
        // For now, we'll return an error indicating this limitation
        Err(format!(
            "Hook registration through Arc not supported. Use a mutable HookRegistry instead."
        ))
        /*
        Arc::get_mut(&mut self.registry)
            .ok_or_else(|| "Cannot get mutable reference to registry".to_string())?
            .register_hook(hook.pred, kernel_type, guard, vec![])
            .map_err(|e| format!("Failed to register hook with HookRegistry: {:?}", e))?;
        */
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
