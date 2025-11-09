// knhk-sidecar: KMS/HSM integration for Fortune 5
// Hardware-backed key management and signing

use crate::error::{SidecarError, SidecarResult};
use std::time::Duration;
use tracing::{info, warn};

/// KMS provider type
#[derive(Debug, Clone)]
pub enum KmsProvider {
    /// AWS KMS
    Aws { region: String, key_id: String },
    /// Azure Key Vault
    Azure { vault_url: String, key_name: String },
    /// HashiCorp Vault
    Vault {
        addr: String,
        mount_path: String,
        key_name: String,
    },
    /// File-based (fallback, not recommended for Fortune 5)
    None,
}

/// KMS configuration
#[derive(Debug, Clone)]
pub struct KmsConfig {
    /// KMS provider
    pub provider: KmsProvider,
    /// Key rotation interval (default: 24 hours for Fortune 5)
    pub rotation_interval: Duration,
    /// Enable automatic key rotation
    pub auto_rotation_enabled: bool,
}

impl Default for KmsConfig {
    fn default() -> Self {
        Self {
            provider: KmsProvider::None,
            rotation_interval: Duration::from_secs(86400), // 24 hours
            auto_rotation_enabled: false,
        }
    }
}

impl KmsConfig {
    /// Create AWS KMS config
    pub fn aws(region: String, key_id: String) -> Self {
        Self {
            provider: KmsProvider::Aws { region, key_id },
            rotation_interval: Duration::from_secs(86400), // 24 hours
            auto_rotation_enabled: true,
        }
    }

    /// Create Azure Key Vault config
    pub fn azure(vault_url: String, key_name: String) -> Self {
        Self {
            provider: KmsProvider::Azure {
                vault_url,
                key_name,
            },
            rotation_interval: Duration::from_secs(86400), // 24 hours
            auto_rotation_enabled: true,
        }
    }

    /// Create HashiCorp Vault config
    pub fn vault(addr: String, mount_path: String, key_name: String) -> Self {
        Self {
            provider: KmsProvider::Vault {
                addr,
                mount_path,
                key_name,
            },
            rotation_interval: Duration::from_secs(86400), // 24 hours
            auto_rotation_enabled: true,
        }
    }

    /// Validate KMS configuration
    pub fn validate(&self) -> SidecarResult<()> {
        match &self.provider {
            KmsProvider::Aws { region, key_id } => {
                if region.is_empty() {
                    return Err(SidecarError::config_error(
                        "AWS region cannot be empty".to_string(),
                    ));
                }
                if key_id.is_empty() {
                    return Err(SidecarError::config_error(
                        "AWS KMS key ID cannot be empty".to_string(),
                    ));
                }
            }
            KmsProvider::Azure {
                vault_url,
                key_name,
            } => {
                if vault_url.is_empty() {
                    return Err(SidecarError::config_error(
                        "Azure vault URL cannot be empty".to_string(),
                    ));
                }
                if key_name.is_empty() {
                    return Err(SidecarError::config_error(
                        "Azure key name cannot be empty".to_string(),
                    ));
                }
            }
            KmsProvider::Vault {
                addr,
                mount_path,
                key_name,
            } => {
                if addr.is_empty() {
                    return Err(SidecarError::config_error(
                        "Vault address cannot be empty".to_string(),
                    ));
                }
                if mount_path.is_empty() {
                    return Err(SidecarError::config_error(
                        "Vault mount path cannot be empty".to_string(),
                    ));
                }
                if key_name.is_empty() {
                    return Err(SidecarError::config_error(
                        "Vault key name cannot be empty".to_string(),
                    ));
                }
            }
            KmsProvider::None => {
                warn!("KMS provider is None. Keys will be stored in files (not recommended for Fortune 5)");
            }
        }

        // Validate rotation interval (must be ≤24h for Fortune 5)
        if self.auto_rotation_enabled && self.rotation_interval > Duration::from_secs(86400) {
            return Err(SidecarError::config_error(format!(
                "Key rotation interval {} exceeds Fortune 5 requirement of 24 hours",
                self.rotation_interval.as_secs()
            )));
        }

        Ok(())
    }
}

/// KMS client abstraction
///
/// Provides unified interface for different KMS providers.
/// In production, this would have actual implementations for each provider.
pub trait KmsClient: Send + Sync {
    /// Sign data with KMS key
    fn sign(&self, data: &[u8]) -> SidecarResult<Vec<u8>>;

    /// Get public key
    fn get_public_key(&self) -> SidecarResult<Vec<u8>>;

    /// Rotate key (create new key version)
    fn rotate_key(&self) -> SidecarResult<String>;

    /// Get key metadata
    fn get_key_metadata(&self) -> SidecarResult<KeyMetadata>;
}

/// Key metadata
#[derive(Debug, Clone)]
pub struct KeyMetadata {
    pub key_id: String,
    pub created_at: std::time::SystemTime,
    pub rotation_date: Option<std::time::SystemTime>,
    pub algorithm: String,
}

/// KMS manager
///
/// Manages KMS operations including key rotation.
pub struct KmsManager {
    config: KmsConfig,
    client: Box<dyn KmsClient>,
    last_rotation: Option<std::time::Instant>,
}

impl KmsManager {
    /// Create new KMS manager
    pub fn new(config: KmsConfig) -> SidecarResult<Self> {
        config.validate()?;

        // Create actual KMS client based on provider
        let client: Box<dyn KmsClient> = match &config.provider {
            KmsProvider::Aws { region, key_id } => {
                #[cfg(feature = "fortune5")]
                {
                    Box::new(AwsKmsClient::new(region.clone(), key_id.clone())?)
                }
                #[cfg(not(feature = "fortune5"))]
                {
                    return Err(SidecarError::config_error(
                        "AWS KMS requires 'fortune5' feature. Enable with --features fortune5"
                            .to_string(),
                    ));
                }
            }
            KmsProvider::Azure {
                vault_url,
                key_name,
            } => {
                #[cfg(feature = "fortune5")]
                {
                    Box::new(AzureKmsClient::new(vault_url.clone(), key_name.clone())?)
                }
                #[cfg(not(feature = "fortune5"))]
                {
                    return Err(SidecarError::config_error(
                        "Azure Key Vault requires 'fortune5' feature. Enable with --features fortune5".to_string()
                    ));
                }
            }
            KmsProvider::Vault {
                addr,
                mount_path,
                key_name,
            } => {
                #[cfg(feature = "fortune5")]
                {
                    Box::new(VaultKmsClient::new(
                        addr.clone(),
                        mount_path.clone(),
                        key_name.clone(),
                    )?)
                }
                #[cfg(not(feature = "fortune5"))]
                {
                    return Err(SidecarError::config_error(
                        "HashiCorp Vault requires 'fortune5' feature. Enable with --features fortune5".to_string()
                    ));
                }
            }
            KmsProvider::None => {
                return Err(SidecarError::config_error(
                    "KMS provider is None. Fortune 5 requires HSM/KMS integration.".to_string(),
                ));
            }
        };

        // All match arms either return early or create a client
        // This code is reachable when fortune5 feature is enabled and client creation succeeds
        #[allow(unreachable_code)]
        Ok(Self {
            config,
            client,
            last_rotation: None,
        })
    }

    /// Check if key rotation is needed
    pub fn needs_rotation(&self) -> bool {
        if !self.config.auto_rotation_enabled {
            return false;
        }

        if let Some(last_rotation) = self.last_rotation {
            last_rotation.elapsed() >= self.config.rotation_interval
        } else {
            true // Never rotated, needs initial rotation check
        }
    }

    /// Rotate key if needed
    pub async fn rotate_if_needed(&mut self) -> SidecarResult<()> {
        if !self.needs_rotation() {
            return Ok(());
        }

        info!(
            "Rotating KMS key (interval: {}s)",
            self.config.rotation_interval.as_secs()
        );

        let new_key_id = self.client.rotate_key()?;
        self.last_rotation = Some(std::time::Instant::now());

        info!("KMS key rotated successfully. New key ID: {}", new_key_id);
        Ok(())
    }

    /// Sign data with current key
    pub fn sign(&self, data: &[u8]) -> SidecarResult<Vec<u8>> {
        self.client.sign(data)
    }

    /// Get public key
    pub fn get_public_key(&self) -> SidecarResult<Vec<u8>> {
        self.client.get_public_key()
    }
}

/// AWS KMS client implementation
#[cfg(feature = "fortune5")]
struct AwsKmsClient {
    region: String,
    key_id: String,
}

#[cfg(feature = "fortune5")]
impl AwsKmsClient {
    fn new(region: String, key_id: String) -> SidecarResult<Self> {
        if region.is_empty() || key_id.is_empty() {
            return Err(SidecarError::config_error(
                "AWS KMS region and key_id must be non-empty".to_string(),
            ));
        }

        Ok(Self { region, key_id })
    }
}

#[cfg(feature = "fortune5")]
impl KmsClient for AwsKmsClient {
    fn sign(&self, _data: &[u8]) -> SidecarResult<Vec<u8>> {
        // In production, use AWS KMS Sign API
        // For now, return error indicating AWS SDK integration needed
        Err(SidecarError::config_error(format!(
            "AWS KMS signing requires AWS SDK integration. Key ID: {}, Region: {}",
            self.key_id, self.region
        )))
    }

    fn get_public_key(&self) -> SidecarResult<Vec<u8>> {
        Err(SidecarError::config_error(format!(
            "AWS KMS public key retrieval requires AWS SDK integration. Key ID: {}",
            self.key_id
        )))
    }

    fn rotate_key(&self) -> SidecarResult<String> {
        Err(SidecarError::config_error(format!(
            "AWS KMS key rotation requires AWS SDK integration. Key ID: {}",
            self.key_id
        )))
    }

    fn get_key_metadata(&self) -> SidecarResult<KeyMetadata> {
        Err(SidecarError::config_error(format!(
            "AWS KMS key metadata requires AWS SDK integration. Key ID: {}",
            self.key_id
        )))
    }
}

/// Azure Key Vault client implementation
#[cfg(feature = "fortune5")]
struct AzureKmsClient {
    vault_url: String,
    key_name: String,
}

#[cfg(feature = "fortune5")]
impl AzureKmsClient {
    fn new(vault_url: String, key_name: String) -> SidecarResult<Self> {
        if vault_url.is_empty() || key_name.is_empty() {
            return Err(SidecarError::config_error(
                "Azure vault URL and key name must be non-empty".to_string(),
            ));
        }

        Ok(Self {
            vault_url,
            key_name,
        })
    }
}

#[cfg(feature = "fortune5")]
impl KmsClient for AzureKmsClient {
    fn sign(&self, _data: &[u8]) -> SidecarResult<Vec<u8>> {
        Err(SidecarError::config_error(format!(
            "Azure Key Vault signing requires Azure SDK integration. Vault: {}, Key: {}",
            self.vault_url, self.key_name
        )))
    }

    fn get_public_key(&self) -> SidecarResult<Vec<u8>> {
        Err(SidecarError::config_error(format!(
            "Azure Key Vault public key retrieval requires Azure SDK integration. Vault: {}",
            self.vault_url
        )))
    }

    fn rotate_key(&self) -> SidecarResult<String> {
        Err(SidecarError::config_error(format!(
            "Azure Key Vault key rotation requires Azure SDK integration. Vault: {}",
            self.vault_url
        )))
    }

    fn get_key_metadata(&self) -> SidecarResult<KeyMetadata> {
        Err(SidecarError::config_error(format!(
            "Azure Key Vault key metadata requires Azure SDK integration. Vault: {}",
            self.vault_url
        )))
    }
}

/// HashiCorp Vault client implementation
#[cfg(feature = "fortune5")]
struct VaultKmsClient {
    addr: String,
    mount_path: String,
    key_name: String,
}

#[cfg(feature = "fortune5")]
impl VaultKmsClient {
    fn new(addr: String, mount_path: String, key_name: String) -> SidecarResult<Self> {
        if addr.is_empty() || mount_path.is_empty() || key_name.is_empty() {
            return Err(SidecarError::config_error(
                "Vault address, mount path, and key name must be non-empty".to_string(),
            ));
        }

        Ok(Self {
            addr,
            mount_path,
            key_name,
        })
    }
}

#[cfg(feature = "fortune5")]
impl KmsClient for VaultKmsClient {
    fn sign(&self, _data: &[u8]) -> SidecarResult<Vec<u8>> {
        // In production, use Vault Transit API for signing via HTTP
        // For now, return error indicating HTTP client integration needed
        Err(SidecarError::config_error(format!(
            "Vault signing requires HTTP client integration. Vault: {}, Mount: {}, Key: {}",
            self.addr, self.mount_path, self.key_name
        )))
    }

    fn get_public_key(&self) -> SidecarResult<Vec<u8>> {
        Err(SidecarError::config_error(format!(
            "Vault public key retrieval requires HTTP client integration. Vault: {}",
            self.addr
        )))
    }

    fn rotate_key(&self) -> SidecarResult<String> {
        Err(SidecarError::config_error(format!(
            "Vault key rotation requires HTTP client integration. Vault: {}",
            self.addr
        )))
    }

    fn get_key_metadata(&self) -> SidecarResult<KeyMetadata> {
        Err(SidecarError::config_error(format!(
            "Vault key metadata requires HTTP client integration. Vault: {}",
            self.addr
        )))
    }
}
