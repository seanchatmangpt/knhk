//! OnceLock-based lazy initialization for global resources
//!
//! Replaces unsafe static initialization with safe OnceLock patterns.

use crate::error::WorkflowResult;
use crate::patterns::PatternId;
use crate::resource::{ResourceId, ResourcePool};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

/// Thread-safe pattern registry with lazy initialization
pub struct PatternRegistry {
    patterns: OnceLock<HashMap<PatternId, Arc<dyn PatternExecutor>>>,
}

/// Trait for pattern executors (simplified for initialization)
pub trait PatternExecutor: Send + Sync {
    fn pattern_id(&self) -> PatternId;
    fn execute(&self) -> WorkflowResult<()>;
}

impl PatternRegistry {
    /// Create a new pattern registry
    pub const fn new() -> Self {
        Self {
            patterns: OnceLock::new(),
        }
    }

    /// Get or initialize the pattern registry
    pub fn get_or_init<F>(&self, f: F) -> &HashMap<PatternId, Arc<dyn PatternExecutor>>
    where
        F: FnOnce() -> HashMap<PatternId, Arc<dyn PatternExecutor>>,
    {
        self.patterns.get_or_init(f)
    }

    /// Get a pattern by ID (returns None if not initialized or pattern not found)
    pub fn get(&self, id: &PatternId) -> Option<Arc<dyn PatternExecutor>> {
        self.patterns.get()?.get(id).cloned()
    }

    /// Check if registry is initialized
    pub fn is_initialized(&self) -> bool {
        self.patterns.get().is_some()
    }

    /// Get count of registered patterns
    pub fn count(&self) -> usize {
        self.patterns.get().map(|m| m.len()).unwrap_or(0)
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global resource registry with lazy initialization
pub struct GlobalResourceRegistry {
    pools: OnceLock<HashMap<String, Arc<ResourcePool>>>,
    config: OnceLock<ResourceConfig>,
}

/// Resource configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub max_pools: usize,
    pub default_pool_size: usize,
    pub enable_auto_scaling: bool,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_pools: 100,
            default_pool_size: 10,
            enable_auto_scaling: true,
        }
    }
}

impl GlobalResourceRegistry {
    /// Create a new resource registry
    pub const fn new() -> Self {
        Self {
            pools: OnceLock::new(),
            config: OnceLock::new(),
        }
    }

    /// Initialize configuration (can only be called once)
    pub fn init_config(&self, config: ResourceConfig) -> Result<(), ResourceConfig> {
        self.config.set(config)
    }

    /// Get configuration (initializes with default if not set)
    pub fn config(&self) -> &ResourceConfig {
        self.config.get_or_init(ResourceConfig::default)
    }

    /// Get or initialize resource pools
    pub fn get_or_init_pools<F>(&self, f: F) -> &HashMap<String, Arc<ResourcePool>>
    where
        F: FnOnce() -> HashMap<String, Arc<ResourcePool>>,
    {
        self.pools.get_or_init(f)
    }

    /// Get a specific pool by name
    pub fn get_pool(&self, name: &str) -> Option<Arc<ResourcePool>> {
        self.pools.get()?.get(name).cloned()
    }

    /// Check if pools are initialized
    pub fn is_initialized(&self) -> bool {
        self.pools.get().is_some()
    }
}

impl Default for GlobalResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global pattern registry instance
pub static GLOBAL_PATTERN_REGISTRY: PatternRegistry = PatternRegistry::new();

/// Global resource registry instance
pub static GLOBAL_RESOURCE_REGISTRY: GlobalResourceRegistry = GlobalResourceRegistry::new();

#[cfg(test)]
mod tests {
    use super::*;

    // Mock pattern executor for testing
    struct MockPatternExecutor {
        id: PatternId,
    }

    impl PatternExecutor for MockPatternExecutor {
        fn pattern_id(&self) -> PatternId {
            self.id
        }

        fn execute(&self) -> WorkflowResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_pattern_registry_lazy_init() {
        let registry = PatternRegistry::new();
        assert!(!registry.is_initialized());
        assert_eq!(registry.count(), 0);

        // Initialize with patterns
        let patterns = registry.get_or_init(|| {
            let mut map = HashMap::new();
            map.insert(
                PatternId(1),
                Arc::new(MockPatternExecutor { id: PatternId(1) }) as Arc<dyn PatternExecutor>,
            );
            map
        });

        assert_eq!(patterns.len(), 1);
        assert!(registry.is_initialized());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_pattern_registry_get() {
        let registry = PatternRegistry::new();

        // Should return None before initialization
        assert!(registry.get(&PatternId(1)).is_none());

        // Initialize
        registry.get_or_init(|| {
            let mut map = HashMap::new();
            map.insert(
                PatternId(1),
                Arc::new(MockPatternExecutor { id: PatternId(1) }) as Arc<dyn PatternExecutor>,
            );
            map
        });

        // Now should return the pattern
        let pattern = registry.get(&PatternId(1));
        assert!(pattern.is_some());
        assert_eq!(pattern.unwrap().pattern_id(), PatternId(1));
    }

    #[test]
    fn test_resource_registry_config() {
        let registry = GlobalResourceRegistry::new();

        // Default config
        let config = registry.config();
        assert_eq!(config.max_pools, 100);
        assert_eq!(config.default_pool_size, 10);

        // Try to set custom config (will fail because already initialized with default)
        let custom_config = ResourceConfig {
            max_pools: 50,
            default_pool_size: 5,
            enable_auto_scaling: false,
        };
        assert!(registry.init_config(custom_config).is_err());
    }

    #[test]
    fn test_resource_registry_pools() {
        let registry = GlobalResourceRegistry::new();
        assert!(!registry.is_initialized());

        // Initialize pools
        let pools = registry.get_or_init_pools(|| {
            let mut map = HashMap::new();
            map.insert("default".to_string(), Arc::new(ResourcePool::new()));
            map
        });

        assert_eq!(pools.len(), 1);
        assert!(registry.is_initialized());

        // Get specific pool
        let pool = registry.get_pool("default");
        assert!(pool.is_some());

        let missing_pool = registry.get_pool("missing");
        assert!(missing_pool.is_none());
    }
}
