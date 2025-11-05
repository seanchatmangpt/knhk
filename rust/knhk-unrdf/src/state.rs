// State management for unrdf integration

use crate::errors::{UnrdfError, UnrdfResult};
use std::path::PathBuf;
use std::sync::{OnceLock, Mutex};
use tokio::runtime::Runtime;

/// Internal state for unrdf integration
pub(crate) struct UnrdfState {
    pub runtime: Runtime,
    pub unrdf_path: String,
    pub state_file: PathBuf,
}

pub(crate) static UNRDF_STATE: OnceLock<Mutex<UnrdfState>> = OnceLock::new();

/// Initialize unrdf integration layer
/// Must be called before any other operations
pub fn init_unrdf(unrdf_path: &str) -> UnrdfResult<()> {
    let runtime = Runtime::new()
        .map_err(|e| UnrdfError::InitializationFailed(format!("Failed to create runtime: {}", e)))?;
    
    // Create state file path in temp directory
    let state_file = std::env::temp_dir().join("knhk_unrdf_state.json");
    
    let state = UnrdfState {
        runtime,
        unrdf_path: unrdf_path.to_string(),
        state_file,
    };
    
    UNRDF_STATE.set(Mutex::new(state))
        .map_err(|_| UnrdfError::InitializationFailed("unrdf already initialized".to_string()))?;
    
    Ok(())
}

