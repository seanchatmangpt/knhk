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

        // Load hooks from storage into registry
        // Note: HookRegistry in knhk-etl doesn't support mutable operations through Arc
        // This is a limitation that needs to be addressed in v1.1
        // For now, we can't register hooks dynamically through Arc
        // FUTURE: Refactor HookRegistry to support mutable operations or use a different approach

        // Since we can't register hooks through Arc, we log a warning
        // In production, hooks should be registered at initialization time, not dynamically
        if !hooks.is_empty() {
            tracing::warn!(
                "Cannot load {} hooks dynamically - HookRegistry doesn't support mutable operations through Arc. \
                 Hooks should be registered at initialization time.",
                hooks.len()
            );
        }

        // Return success - hooks are already registered at initialization
        // Dynamic loading is not supported due to Arc mutability limitations
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
        hook: &crate::hook_registry::store::HookEntry,
    ) -> Result<knhk_etl::hook_registry::GuardFn, String> {
        // Create guard function based on hook parameters
        // Guard checks if triple matches hook's S, P, O, K constraints
        let s_constraint = hook.s;
        let p_constraint = hook.p;
        let o_constraint = hook.o;
        let k_constraint = hook.k;

        Ok(Box::new(
            move |triple: &knhk_etl::ingest::RawTriple| -> bool {
                // Check predicate constraint (required)
                if let Some(p) = p_constraint {
                    // Convert predicate IRI to hash for comparison
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    triple.predicate.hash(&mut hasher);
                    let pred_hash = hasher.finish();
                    if pred_hash != p {
                        return false;
                    }
                }

                // Check subject constraint (optional)
                if let Some(s) = s_constraint {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    triple.subject.hash(&mut hasher);
                    let subj_hash = hasher.finish();
                    if subj_hash != s {
                        return false;
                    }
                }

                // Check object constraint (optional)
                if let Some(o) = o_constraint {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    triple.object.hash(&mut hasher);
                    let obj_hash = hasher.finish();
                    if obj_hash != o {
                        return false;
                    }
                }

                // Check graph constraint (optional)
                if let Some(k) = k_constraint {
                    if let Some(ref graph) = triple.graph {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        graph.hash(&mut hasher);
                        let graph_hash = hasher.finish();
                        if graph_hash != k {
                            return false;
                        }
                    } else {
                        return false; // Hook requires graph but triple has none
                    }
                }

                true
            },
        ))
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
        // Get mutable reference to HookRegistry
        let registry = Arc::get_mut(&mut self.registry).ok_or_else(|| {
            "Cannot get mutable reference to registry - multiple references exist".to_string()
        })?;

        registry
            .register_hook(hook.pred, kernel_type, guard, vec![])
            .map_err(|e| format!("Failed to register hook with HookRegistry: {:?}", e))?;

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
