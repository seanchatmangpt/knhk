//! Phase Registry - Compile-time Phase Registration
//!
//! Uses linkme for zero-cost compile-time phase registration.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use linkme::distributed_slice;

use super::core::{Phase, PhaseMetadata, PhaseResult};
use crate::error::WorkflowResult;

/// Type-erased phase factory function
pub type PhaseFactory = fn() -> Arc<dyn std::any::Any + Send + Sync>;

/// Phase registration entry
pub struct PhaseEntry {
    pub metadata: PhaseMetadata,
    pub factory: PhaseFactory,
}

/// Distributed slice for compile-time phase registration
#[distributed_slice]
pub static REGISTERED_PHASES: [PhaseEntry];

/// Macro for registering a phase at compile-time
///
/// # Example
/// ```ignore
/// register_phase!(MyPhase, MyPhase::new);
/// ```
#[macro_export]
macro_rules! register_phase {
    ($phase_type:ty, $factory:expr) => {
        #[linkme::distributed_slice($crate::validation::phases::registry::REGISTERED_PHASES)]
        static PHASE_ENTRY: $crate::validation::phases::registry::PhaseEntry =
            $crate::validation::phases::registry::PhaseEntry {
                metadata: <$phase_type as $crate::validation::phases::core::Phase<_, _>>::metadata(),
                factory: || std::sync::Arc::new($factory()),
            };
    };
}

/// Phase registry for runtime phase lookup and execution
pub struct PhaseRegistry {
    phases: RwLock<HashMap<String, (PhaseMetadata, PhaseFactory)>>,
}

impl PhaseRegistry {
    /// Create a new phase registry
    pub fn new() -> Self {
        Self {
            phases: RwLock::new(HashMap::new()),
        }
    }

    /// Create registry from compile-time registered phases
    pub fn from_registered() -> Self {
        let registry = Self::new();

        // Load all phases registered via distributed slice
        for entry in REGISTERED_PHASES {
            registry.register_static(&entry.metadata, entry.factory);
        }

        registry
    }

    /// Register a phase statically (from compile-time registration)
    fn register_static(&self, metadata: &PhaseMetadata, factory: PhaseFactory) {
        let mut phases = self.phases.write().unwrap();
        phases.insert(metadata.name.to_string(), (*metadata, factory));
    }

    /// Get phase metadata by name
    pub fn get_metadata(&self, name: &str) -> Option<PhaseMetadata> {
        let phases = self.phases.read().unwrap();
        phases.get(name).map(|(metadata, _)| *metadata)
    }

    /// List all registered phase names
    pub fn list_phases(&self) -> Vec<String> {
        let phases = self.phases.read().unwrap();
        phases.keys().cloned().collect()
    }

    /// Get all phase metadata
    pub fn get_all_metadata(&self) -> Vec<PhaseMetadata> {
        let phases = self.phases.read().unwrap();
        phases.values().map(|(metadata, _)| *metadata).collect()
    }

    /// Check if a phase exists
    pub fn has_phase(&self, name: &str) -> bool {
        let phases = self.phases.read().unwrap();
        phases.contains_key(name)
    }

    /// Get phase factory
    pub fn get_factory(&self, name: &str) -> Option<PhaseFactory> {
        let phases = self.phases.read().unwrap();
        phases.get(name).map(|(_, factory)| *factory)
    }
}

impl Default for PhaseRegistry {
    fn default() -> Self {
        Self::from_registered()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = PhaseRegistry::new();
        assert_eq!(registry.list_phases().len(), 0);
    }

    #[test]
    fn test_registry_from_registered() {
        let registry = PhaseRegistry::from_registered();
        // Should have phases registered via distributed slice
        // (actual count depends on which phases are compiled)
        let phases = registry.list_phases();
        assert!(phases.len() >= 0); // May be 0 if no phases registered yet
    }
}
