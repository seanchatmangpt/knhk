// knhk-unrdf: Internal state management
// State management for unrdf integration

use crate::types::{HookRegistryEntry, Transaction};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::runtime::Runtime;

/// Internal state for unrdf integration
pub struct UnrdfState {
    pub runtime: Runtime,
    pub unrdf_path: String,
    pub transactions: Arc<Mutex<HashMap<u32, Transaction>>>,
    pub next_transaction_id: Arc<Mutex<u32>>,
    pub hooks: Arc<Mutex<HashMap<String, HookRegistryEntry>>>,
}

static UNRDF_STATE: OnceLock<UnrdfState> = OnceLock::new();

/// Get the global unrdf state
pub fn get_state() -> Result<&'static UnrdfState, crate::error::UnrdfError> {
    UNRDF_STATE.get().ok_or_else(|| {
        crate::error::UnrdfError::InitializationFailed("unrdf not initialized".to_string())
    })
}

/// Initialize and set the global unrdf state
pub fn init_state(state: UnrdfState) -> Result<(), crate::error::UnrdfError> {
    UNRDF_STATE.set(state).map_err(|_| {
        crate::error::UnrdfError::InitializationFailed("unrdf already initialized".to_string())
    })
}

