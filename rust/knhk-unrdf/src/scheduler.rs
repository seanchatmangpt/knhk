// rust/knhk-unrdf/src/scheduler.rs
// Epoch scheduler for warm path hook selection by Λ order

use crate::error::{UnrdfError, UnrdfResult};
use crate::hooks_native::NativeHookRegistry;
use crate::types::HookDefinition;

/// Epoch scheduler for selecting hooks by Λ total order
pub struct EpochScheduler {
    registry: NativeHookRegistry,
}

impl EpochScheduler {
    pub fn new(registry: NativeHookRegistry) -> Self {
        Self { registry }
    }

    /// Select hooks by epoch order (Λ)
    /// Lambda is a ≺-total ordered list of hook IDs
    /// Returns hooks in the specified order for warm path execution
    pub fn select_hooks_by_lambda(&self, lambda: &[String]) -> UnrdfResult<Vec<HookDefinition>> {
        // Validate Λ is ≺-total (no duplicates)
        NativeHookRegistry::validate_epoch_order(lambda)?;

        // Select hooks by order
        self.registry.select_by_epoch_order(lambda)
    }

    /// Execute hooks in epoch order
    /// Enforces Λ ≺-total ordering and τ ≤ 8 ticks constraint
    pub fn execute_epoch(
        &self,
        lambda: &[String],
        turtle_data: &str,
        tau: u32,
    ) -> UnrdfResult<Vec<crate::types::HookResult>> {
        // Validate τ ≤ 8 (Chatman Constant)
        if tau > 8 {
            return Err(UnrdfError::InvalidInput(format!(
                "τ {} exceeds Chatman Constant (8 ticks)",
                tau
            )));
        }

        // Validate Λ is ≺-total
        NativeHookRegistry::validate_epoch_order(lambda)?;

        // Select hooks by order
        let hooks = self.select_hooks_by_lambda(lambda)?;

        if hooks.is_empty() {
            return Ok(Vec::new());
        }

        // Execute hooks in batch (warm path)
        // Note: For hot path (≤8 ticks), would use single hook execution
        crate::hooks_native::evaluate_hooks_batch_native(&hooks, turtle_data)
    }
}

impl Default for EpochScheduler {
    fn default() -> Self {
        Self::new(NativeHookRegistry::new())
    }
}
