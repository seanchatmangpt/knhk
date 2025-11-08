#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Secret management for workflow engine

use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Secret provider trait
pub trait SecretProvider: Send + Sync {
    /// Get a secret value
    fn get_secret(&self, key: &str) -> WorkflowResult<String>;

    /// Rotate a secret
    fn rotate_secret(&self, key: &str) -> WorkflowResult<()>;
}

/// In-memory secret provider (for testing)
pub struct InMemorySecretProvider {
    secrets: Arc<Mutex<HashMap<String, String>>>,
}

impl InMemorySecretProvider {
    /// Create a new in-memory secret provider
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set a secret
    pub fn set_secret(&self, key: String, value: String) {
        let mut secrets = self.secrets.lock().unwrap();
        secrets.insert(key, value);
    }
}

impl SecretProvider for InMemorySecretProvider {
    fn get_secret(&self, key: &str) -> WorkflowResult<String> {
        let secrets = self.secrets.lock().map_err(|e| {
            WorkflowError::Internal(format!("Failed to acquire secret lock: {}", e))
        })?;

        secrets
            .get(key)
            .cloned()
            .ok_or_else(|| WorkflowError::ResourceUnavailable(format!("Secret {} not found", key)))
    }

    fn rotate_secret(&self, _key: &str) -> WorkflowResult<()> {
        // In-memory provider doesn't support rotation
        Ok(())
    }
}

impl Default for InMemorySecretProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Secret rotation metadata
#[derive(Debug, Clone)]
struct SecretMetadata {
    /// Last rotation time
    last_rotation: Instant,
    /// Rotation interval
    rotation_interval: Duration,
}

/// Secret manager with rotation support
pub struct SecretManager {
    provider: Arc<dyn SecretProvider>,
    metadata: Arc<Mutex<HashMap<String, SecretMetadata>>>,
    rotation_interval: Duration,
}

impl SecretManager {
    /// Create a new secret manager
    pub fn new(provider: Arc<dyn SecretProvider>, rotation_interval_hours: u32) -> Self {
        if rotation_interval_hours > 24 {
            panic!("Rotation interval must be ≤24 hours");
        }

        Self {
            provider,
            metadata: Arc::new(Mutex::new(HashMap::new())),
            rotation_interval: Duration::from_secs(rotation_interval_hours as u64 * 3600),
        }
    }

    /// Get a secret (with automatic rotation check)
    pub fn get_secret(&self, key: &str) -> WorkflowResult<String> {
        // Check if rotation is needed
        let needs_rotation = {
            let metadata = self.metadata.lock().map_err(|e| {
                WorkflowError::Internal(format!("Failed to acquire metadata lock: {}", e))
            })?;

            metadata
                .get(key)
                .map(|m| m.last_rotation.elapsed() >= m.rotation_interval)
                .unwrap_or(true)
        };

        if needs_rotation {
            // Rotate secret
            self.provider.rotate_secret(key)?;

            // Update metadata
            let mut metadata = self.metadata.lock().map_err(|e| {
                WorkflowError::Internal(format!("Failed to acquire metadata lock: {}", e))
            })?;

            metadata.insert(
                key.to_string(),
                SecretMetadata {
                    last_rotation: Instant::now(),
                    rotation_interval: self.rotation_interval,
                },
            );
        }

        self.provider.get_secret(key)
    }

    /// Manually rotate a secret
    pub fn rotate_secret(&self, key: &str) -> WorkflowResult<()> {
        self.provider.rotate_secret(key)?;

        // Update metadata
        let mut metadata = self.metadata.lock().map_err(|e| {
            WorkflowError::Internal(format!("Failed to acquire metadata lock: {}", e))
        })?;

        metadata.insert(
            key.to_string(),
            SecretMetadata {
                last_rotation: Instant::now(),
                rotation_interval: self.rotation_interval,
            },
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_manager() {
        let provider = Arc::new(InMemorySecretProvider::new());
        provider.set_secret("test-key".to_string(), "test-value".to_string());

        let manager = SecretManager::new(provider, 24);

        let value = manager.get_secret("test-key").unwrap();
        assert_eq!(value, "test-value");
    }
}
