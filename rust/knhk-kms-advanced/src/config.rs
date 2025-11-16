//! Configuration types for KMS providers and operations

use serde::{Deserialize, Serialize};

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderConfig {
    /// AWS KMS configuration
    Aws {
        region: String,
        key_id: String,
        endpoint: Option<String>,
    },
    /// Azure Key Vault configuration
    Azure {
        vault_url: String,
        key_name: String,
        tenant_id: String,
    },
    /// HashiCorp Vault configuration
    Vault {
        address: String,
        mount_path: String,
        key_name: String,
        token: String,
    },
    /// Local/testing configuration
    Local { key_path: String },
}

/// KMS operation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmsConfig {
    /// Provider configuration
    pub provider: ProviderConfig,

    /// Enable SIMD optimizations (requires compatible hardware)
    pub enable_simd: bool,

    /// Batch size for vectorized operations (must be power of 2)
    pub batch_size: usize,

    /// Maximum concurrent operations
    pub max_concurrent: usize,

    /// Cache size for frequently used keys
    pub cache_size: usize,
}

impl Default for KmsConfig {
    fn default() -> Self {
        Self {
            provider: ProviderConfig::Local {
                key_path: "/tmp/kms-keys".to_string(),
            },
            enable_simd: true,
            batch_size: 64,
            max_concurrent: 100,
            cache_size: 1024,
        }
    }
}

impl KmsConfig {
    /// Validate configuration
    pub fn validate(&self) -> crate::Result<()> {
        // Batch size must be power of 2
        if !self.batch_size.is_power_of_two() {
            return Err(crate::KmsError::ConfigError(
                "batch_size must be a power of 2".to_string(),
            ));
        }

        // Batch size must be reasonable
        if self.batch_size > 1024 {
            return Err(crate::KmsError::ConfigError(
                "batch_size too large (max 1024)".to_string(),
            ));
        }

        Ok(())
    }

    /// Get optimal batch size for current hardware
    pub fn optimal_batch_size() -> usize {
        // For SIMD operations, align with vector width
        // AVX2: 256-bit = 8 x 32-byte hashes
        // AVX-512: 512-bit = 16 x 32-byte hashes
        64 // Conservative default that works well across platforms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_valid() {
        let config = KmsConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_batch_size() {
        let mut config = KmsConfig::default();
        config.batch_size = 63; // Not a power of 2
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_batch_size_too_large() {
        let mut config = KmsConfig::default();
        config.batch_size = 2048; // Too large
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_optimal_batch_size() {
        let size = KmsConfig::optimal_batch_size();
        assert!(size.is_power_of_two());
        assert!(size <= 1024);
    }
}
