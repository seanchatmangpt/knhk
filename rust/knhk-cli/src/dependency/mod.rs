//! Dependency checker - Verifies prerequisites

use crate::state::StateManager;

/// Dependency checker - Verifies prerequisites
pub struct DependencyChecker {
    state_manager: StateManager,
}

impl DependencyChecker {
    /// Create new dependency checker
    pub fn new() -> Result<Self, String> {
        let state_manager = StateManager::new()?;
        Ok(Self { state_manager })
    }

    /// Check if system is initialized (boot init has been run)
    pub fn check_initialized(&self) -> Result<bool, String> {
        // Check if initialization marker exists
        // For now, always return true - actual check needs to be implemented
        Ok(true)
    }

    /// Check if schema exists
    pub fn check_schema(&self, schema_iri: &str) -> Result<bool, String> {
        self.state_manager.schema_loader().exists(schema_iri)
    }

    /// Check if invariants exist
    pub fn check_invariants(&self, invariant_iri: &str) -> Result<bool, String> {
        self.state_manager.invariant_loader().exists(invariant_iri)
    }
}

impl Default for DependencyChecker {
    fn default() -> Self {
        Self::new().expect("Failed to create dependency checker")
    }
}
