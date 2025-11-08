//! Oxigraph store wrapper for state management

use oxigraph::store::Store;
use std::path::PathBuf;
use std::sync::Arc;

/// State store - Wraps Oxigraph store for O, Σ, Q
#[derive(Clone)]
pub struct StateStore {
    store: Arc<Store>,
    base_path: PathBuf,
}

impl StateStore {
    /// Create new state store
    pub fn new() -> Result<Self, String> {
        let base_path = get_config_dir()?;

        // Create store (in-memory for now, can be persisted)
        let store = Store::new().map_err(|e| format!("Failed to create Oxigraph store: {}", e))?;

        Ok(Self {
            store: Arc::new(store),
            base_path,
        })
    }

    /// Get Oxigraph store reference
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Get base path
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }
}

fn get_config_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let mut path = PathBuf::from(std::env::var("APPDATA").map_err(|_| "APPDATA not set")?);
        path.push("knhk");
        Ok(path)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
        let mut path = PathBuf::from(home);
        path.push(".knhk");
        Ok(path)
    }
}
