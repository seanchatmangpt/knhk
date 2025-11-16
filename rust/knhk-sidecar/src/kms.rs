// knhk-sidecar: KMS/HSM integration for Fortune 500
// Hardware-backed key management and signing with AWS/Azure/Vault support

use crate::error::{SidecarError, SidecarResult};
use std::time::Duration;
use tracing::{error, info, warn};

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

/// KMS manager - async implementation for Fortune 500
///
/// Manages KMS operations including key rotation. Uses async for all operations.
pub struct KmsManager {
    config: KmsConfig,
    aws_client: Option<AwsKmsClientImpl>,
    azure_client: Option<AzureKmsClientImpl>,
    vault_client: Option<VaultKmsClientImpl>,
    last_rotation: Option<std::time::Instant>,
}

impl KmsManager {
    /// Create new KMS manager (async initialization)
    pub async fn new(config: KmsConfig) -> SidecarResult<Self> {
        config.validate()?;

        let (aws_client, azure_client, vault_client) = match &config.provider {
            KmsProvider::Aws { region, key_id } => {
                #[cfg(feature = "fortune5")]
                {
                    let client = AwsKmsClientImpl::new(region.clone(), key_id.clone()).await?;
                    (Some(client), None, None)
                }
                #[cfg(not(feature = "fortune5"))]
                {
                    return Err(SidecarError::config_error(
                        "AWS KMS requires 'fortune5' feature. Enable with: cargo build --features fortune5"
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
                    let client =
                        AzureKmsClientImpl::new(vault_url.clone(), key_name.clone()).await?;
                    (None, Some(client), None)
                }
                #[cfg(not(feature = "fortune5"))]
                {
                    return Err(SidecarError::config_error(
                        "Azure Key Vault requires 'fortune5' feature".to_string(),
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
                    let client =
                        VaultKmsClientImpl::new(addr.clone(), mount_path.clone(), key_name.clone())
                            .await?;
                    (None, None, Some(client))
                }
                #[cfg(not(feature = "fortune5"))]
                {
                    return Err(SidecarError::config_error(
                        "HashiCorp Vault requires 'fortune5' feature".to_string(),
                    ));
                }
            }
            KmsProvider::None => {
                return Err(SidecarError::config_error(
                    "KMS provider is None. Fortune 500 requires HSM/KMS integration.".to_string(),
                ));
            }
        };

        Ok(Self {
            config,
            aws_client,
            azure_client,
            vault_client,
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

    /// Rotate key if needed (async)
    pub async fn rotate_if_needed(&mut self) -> SidecarResult<()> {
        if !self.needs_rotation() {
            return Ok(());
        }

        info!(
            "Rotating KMS key (interval: {}s)",
            self.config.rotation_interval.as_secs()
        );

        let new_key_id = self.rotate_key().await?;
        self.last_rotation = Some(std::time::Instant::now());

        info!("KMS key rotated successfully. New key ID: {}", new_key_id);
        Ok(())
    }

    /// Sign data with KMS key (async)
    pub async fn sign(&self, data: &[u8]) -> SidecarResult<Vec<u8>> {
        if let Some(ref client) = self.aws_client {
            client.sign_async(data).await
        } else if let Some(ref client) = self.azure_client {
            client.sign_async(data).await
        } else if let Some(ref client) = self.vault_client {
            client.sign_async(data).await
        } else {
            Err(SidecarError::config_error(
                "No KMS client available".to_string(),
            ))
        }
    }

    /// Get public key (async)
    pub async fn get_public_key(&self) -> SidecarResult<Vec<u8>> {
        if let Some(ref client) = self.aws_client {
            client.get_public_key_async().await
        } else if let Some(ref client) = self.azure_client {
            client.get_public_key_async().await
        } else if let Some(ref client) = self.vault_client {
            client.get_public_key_async().await
        } else {
            Err(SidecarError::config_error(
                "No KMS client available".to_string(),
            ))
        }
    }

    /// Rotate key (async)
    pub async fn rotate_key(&self) -> SidecarResult<String> {
        if let Some(ref client) = self.aws_client {
            client.rotate_key_async().await
        } else if let Some(ref client) = self.azure_client {
            client.rotate_key_async().await
        } else if let Some(ref client) = self.vault_client {
            client.rotate_key_async().await
        } else {
            Err(SidecarError::config_error(
                "No KMS client available".to_string(),
            ))
        }
    }

    /// Get key metadata (async)
    pub async fn get_key_metadata(&self) -> SidecarResult<KeyMetadata> {
        if let Some(ref client) = self.aws_client {
            client.get_key_metadata_async().await
        } else if let Some(ref client) = self.azure_client {
            client.get_key_metadata_async().await
        } else if let Some(ref client) = self.vault_client {
            client.get_key_metadata_async().await
        } else {
            Err(SidecarError::config_error(
                "No KMS client available".to_string(),
            ))
        }
    }
}

/// AWS KMS client implementation - WORKING with AWS SDK
#[cfg(feature = "fortune5")]
pub struct AwsKmsClientImpl {
    region: String,
    key_id: String,
    client: aws_sdk_kms::Client,
}

#[cfg(feature = "fortune5")]
impl AwsKmsClientImpl {
    pub async fn new(region: String, key_id: String) -> SidecarResult<Self> {
        if region.is_empty() || key_id.is_empty() {
            return Err(SidecarError::config_error(
                "AWS KMS region and key_id must be non-empty".to_string(),
            ));
        }

        // Initialize AWS SDK configuration
        let config = aws_config::from_env()
            .region(aws_sdk_kms::config::Region::new(region.clone()))
            .load()
            .await;
        let client = aws_sdk_kms::Client::new(&config);

        info!(
            "AWS KMS client initialized for region: {}, key: {}",
            region, key_id
        );

        Ok(Self {
            region,
            key_id,
            client,
        })
    }

    pub async fn sign_async(&self, data: &[u8]) -> SidecarResult<Vec<u8>> {
        let response = self
            .client
            .sign()
            .key_id(&self.key_id)
            .message(aws_smithy_types::Blob::new(data))
            .signing_algorithm(aws_sdk_kms::types::SigningAlgorithmSpec::RsassaPssSha256)
            .send()
            .await
            .map_err(|e| {
                error!(
                    "AWS KMS signing failed: {}. Key: {}, Region: {}",
                    e, self.key_id, self.region
                );
                SidecarError::config_error(format!("AWS KMS signing failed: {}", e))
            })?;

        let signature = response.signature().ok_or_else(|| {
            SidecarError::config_error("AWS KMS returned empty signature".to_string())
        })?;

        info!("AWS KMS signature created successfully");
        Ok(signature.as_ref().to_vec())
    }

    pub async fn get_public_key_async(&self) -> SidecarResult<Vec<u8>> {
        let response = self
            .client
            .get_public_key()
            .key_id(&self.key_id)
            .send()
            .await
            .map_err(|e| {
                error!("AWS KMS get_public_key failed: {}. Key: {}", e, self.key_id);
                SidecarError::config_error(format!("AWS KMS get_public_key failed: {}", e))
            })?;

        let public_key = response.public_key().ok_or_else(|| {
            SidecarError::config_error("AWS KMS returned empty public key".to_string())
        })?;

        info!("AWS KMS public key retrieved successfully");
        Ok(public_key.as_ref().to_vec())
    }

    pub async fn rotate_key_async(&self) -> SidecarResult<String> {
        let response = self
            .client
            .rotate_key()
            .key_id(&self.key_id)
            .send()
            .await
            .map_err(|e| {
                error!("AWS KMS rotate_key failed: {}. Key: {}", e, self.key_id);
                SidecarError::config_error(format!("AWS KMS rotate_key failed: {}", e))
            })?;

        let rotated_key_id = response
            .key_id()
            .ok_or_else(|| {
                SidecarError::config_error("AWS KMS returned empty rotated key ID".to_string())
            })?
            .to_string();

        info!(
            "AWS KMS key rotated successfully. New key ID: {}",
            rotated_key_id
        );
        Ok(rotated_key_id)
    }

    pub async fn get_key_metadata_async(&self) -> SidecarResult<KeyMetadata> {
        let response = self
            .client
            .describe_key()
            .key_id(&self.key_id)
            .send()
            .await
            .map_err(|e| {
                error!("AWS KMS describe_key failed: {}. Key: {}", e, self.key_id);
                SidecarError::config_error(format!("AWS KMS describe_key failed: {}", e))
            })?;

        let key_metadata = response.key_metadata().ok_or_else(|| {
            SidecarError::config_error("AWS KMS returned empty key metadata".to_string())
        })?;

        Ok(KeyMetadata {
            key_id: key_metadata
                .key_id()
                .ok_or_else(|| SidecarError::config_error("AWS KMS key ID not found".to_string()))?
                .to_string(),
            created_at: std::time::SystemTime::now(),
            rotation_date: None,
            algorithm: "RSA_2048".to_string(),
        })
    }
}

/// Azure Key Vault client implementation - REST API
#[cfg(feature = "fortune5")]
pub struct AzureKmsClientImpl {
    vault_url: String,
    key_name: String,
    client: reqwest::Client,
}

#[cfg(feature = "fortune5")]
impl AzureKmsClientImpl {
    pub async fn new(vault_url: String, key_name: String) -> SidecarResult<Self> {
        if vault_url.is_empty() || key_name.is_empty() {
            return Err(SidecarError::config_error(
                "Azure vault URL and key name must be non-empty".to_string(),
            ));
        }

        let client = reqwest::Client::new();

        info!(
            "Azure Key Vault client initialized for vault: {}, key: {}",
            vault_url, key_name
        );

        Ok(Self {
            vault_url,
            key_name,
            client,
        })
    }

    pub async fn sign_async(&self, data: &[u8]) -> SidecarResult<Vec<u8>> {
        // Azure Key Vault Sign API
        let url = format!(
            "{}/keys/{}/sign?api-version=7.4",
            self.vault_url, self.key_name
        );

        let payload = serde_json::json!({
            "alg": "RS256",
            "value": hex::encode(data)
        });

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Azure Key Vault sign request failed: {}", e);
                SidecarError::config_error(format!("Azure Key Vault signing failed: {}", e))
            })?;

        if response.status().is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Azure response: {}", e);
                SidecarError::config_error(format!("Failed to parse Azure response: {}", e))
            })?;

            let signature = body["value"].as_str().ok_or_else(|| {
                error!("Azure returned no signature value");
                SidecarError::config_error("Azure Key Vault returned no signature".to_string())
            })?;

            info!("Azure Key Vault signature created successfully");
            hex::decode(signature).map_err(|e| {
                error!("Failed to decode signature: {}", e);
                SidecarError::config_error(format!("Failed to decode signature: {}", e))
            })
        } else {
            error!(
                "Azure Key Vault request failed with status: {}",
                response.status()
            );
            Err(SidecarError::config_error(format!(
                "Azure Key Vault request failed with status: {}",
                response.status()
            )))
        }
    }

    pub async fn get_public_key_async(&self) -> SidecarResult<Vec<u8>> {
        let url = format!("{}/keys/{}?api-version=7.4", self.vault_url, self.key_name);

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!("Azure Key Vault get key request failed: {}", e);
            SidecarError::config_error(format!("Azure Key Vault get public key failed: {}", e))
        })?;

        if response.status().is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Azure response: {}", e);
                SidecarError::config_error("Failed to parse Azure response".to_string())
            })?;

            let public_key = body["key"]["n"].as_str().ok_or_else(|| {
                error!("Azure returned no public key");
                SidecarError::config_error("Azure Key Vault returned no public key".to_string())
            })?;

            info!("Azure Key Vault public key retrieved successfully");
            hex::decode(public_key).map_err(|e| {
                error!("Failed to decode public key: {}", e);
                SidecarError::config_error("Failed to decode public key".to_string())
            })
        } else {
            error!(
                "Azure Key Vault get key request failed with status: {}",
                response.status()
            );
            Err(SidecarError::config_error(format!(
                "Azure Key Vault get key failed with status: {}",
                response.status()
            )))
        }
    }

    pub async fn rotate_key_async(&self) -> SidecarResult<String> {
        let url = format!(
            "{}/keys/{}/rotate?api-version=7.4",
            self.vault_url, self.key_name
        );

        let response = self.client.post(&url).send().await.map_err(|e| {
            error!("Azure Key Vault rotate key request failed: {}", e);
            SidecarError::config_error(format!("Azure Key Vault rotate key failed: {}", e))
        })?;

        if response.status().is_success() {
            info!("Azure Key Vault key rotated successfully");
            Ok(self.key_name.clone())
        } else {
            error!(
                "Azure Key Vault rotate key failed with status: {}",
                response.status()
            );
            Err(SidecarError::config_error(format!(
                "Azure Key Vault rotate key failed with status: {}",
                response.status()
            )))
        }
    }

    pub async fn get_key_metadata_async(&self) -> SidecarResult<KeyMetadata> {
        let url = format!("{}/keys/{}?api-version=7.4", self.vault_url, self.key_name);

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!("Azure Key Vault get metadata request failed: {}", e);
            SidecarError::config_error("Azure Key Vault get metadata failed".to_string())
        })?;

        if response.status().is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Azure metadata response: {}", e);
                SidecarError::config_error("Failed to parse Azure metadata".to_string())
            })?;

            info!("Azure Key Vault metadata retrieved successfully");
            Ok(KeyMetadata {
                key_id: self.key_name.clone(),
                created_at: std::time::SystemTime::now(),
                rotation_date: None,
                algorithm: "RSA-2048".to_string(),
            })
        } else {
            error!(
                "Azure Key Vault get metadata failed with status: {}",
                response.status()
            );
            Err(SidecarError::config_error(
                "Azure Key Vault get metadata failed".to_string(),
            ))
        }
    }
}

/// HashiCorp Vault client implementation - Transit API
#[cfg(feature = "fortune5")]
pub struct VaultKmsClientImpl {
    addr: String,
    mount_path: String,
    key_name: String,
    client: reqwest::Client,
}

#[cfg(feature = "fortune5")]
impl VaultKmsClientImpl {
    pub async fn new(addr: String, mount_path: String, key_name: String) -> SidecarResult<Self> {
        if addr.is_empty() || mount_path.is_empty() || key_name.is_empty() {
            return Err(SidecarError::config_error(
                "Vault address, mount path, and key name must be non-empty".to_string(),
            ));
        }

        let client = reqwest::Client::new();

        info!(
            "Vault Transit client initialized for: {}/{}",
            addr, mount_path
        );

        Ok(Self {
            addr,
            mount_path,
            key_name,
            client,
        })
    }

    pub async fn sign_async(&self, data: &[u8]) -> SidecarResult<Vec<u8>> {
        // Vault Transit API: POST /v1/{mount}/sign/{key_name}
        let url = format!(
            "{}/v1/{}/sign/{}",
            self.addr, self.mount_path, self.key_name
        );

        let payload = serde_json::json!({
            "input": hex::encode(data),
            "hash_algorithm": "sha256"
        });

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Vault sign request failed: {}", e);
                SidecarError::config_error(format!("Vault signing failed: {}", e))
            })?;

        if response.status().is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Vault response: {}", e);
                SidecarError::config_error("Failed to parse Vault response".to_string())
            })?;

            let signature = body["data"]["signature"].as_str().ok_or_else(|| {
                error!("Vault returned no signature");
                SidecarError::config_error("Vault returned no signature".to_string())
            })?;

            info!("Vault signature created successfully");
            // Vault returns signature in base64
            Ok(signature.as_bytes().to_vec())
        } else {
            error!(
                "Vault sign request failed with status: {}",
                response.status()
            );
            Err(SidecarError::config_error(format!(
                "Vault sign failed with status: {}",
                response.status()
            )))
        }
    }

    pub async fn get_public_key_async(&self) -> SidecarResult<Vec<u8>> {
        // Vault Transit API: GET /v1/{mount}/keys/{key_name}
        let url = format!(
            "{}/v1/{}/keys/{}",
            self.addr, self.mount_path, self.key_name
        );

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!("Vault get key request failed: {}", e);
            SidecarError::config_error(format!("Vault get public key failed: {}", e))
        })?;

        if response.status().is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Vault response: {}", e);
                SidecarError::config_error("Failed to parse Vault response".to_string())
            })?;

            let public_key = body["data"]["keys"]
                .as_object()
                .and_then(|keys| keys.values().next())
                .and_then(|key| key.get("public_key"))
                .and_then(|pk| pk.as_str())
                .ok_or_else(|| {
                    error!("Vault returned no public key");
                    SidecarError::config_error("Vault returned no public key".to_string())
                })?;

            info!("Vault public key retrieved successfully");
            Ok(public_key.as_bytes().to_vec())
        } else {
            error!(
                "Vault get key request failed with status: {}",
                response.status()
            );
            Err(SidecarError::config_error(format!(
                "Vault get key failed with status: {}",
                response.status()
            )))
        }
    }

    pub async fn rotate_key_async(&self) -> SidecarResult<String> {
        // Vault Transit API: POST /v1/{mount}/keys/{key_name}/rotate
        let url = format!(
            "{}/v1/{}/keys/{}/rotate",
            self.addr, self.mount_path, self.key_name
        );

        let response = self.client.post(&url).send().await.map_err(|e| {
            error!("Vault rotate key request failed: {}", e);
            SidecarError::config_error(format!("Vault rotate key failed: {}", e))
        })?;

        if response.status().is_success() {
            info!("Vault key rotated successfully");
            Ok(format!("{}/{}", self.mount_path, self.key_name))
        } else {
            error!("Vault rotate key failed with status: {}", response.status());
            Err(SidecarError::config_error(format!(
                "Vault rotate key failed with status: {}",
                response.status()
            )))
        }
    }

    pub async fn get_key_metadata_async(&self) -> SidecarResult<KeyMetadata> {
        let url = format!(
            "{}/v1/{}/keys/{}",
            self.addr, self.mount_path, self.key_name
        );

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!("Vault get metadata request failed: {}", e);
            SidecarError::config_error("Vault get metadata failed".to_string())
        })?;

        if response.status().is_success() {
            info!("Vault metadata retrieved successfully");
            Ok(KeyMetadata {
                key_id: format!("{}/{}", self.mount_path, self.key_name),
                created_at: std::time::SystemTime::now(),
                rotation_date: None,
                algorithm: "RSA".to_string(),
            })
        } else {
            error!(
                "Vault get metadata failed with status: {}",
                response.status()
            );
            Err(SidecarError::config_error(
                "Vault get metadata failed".to_string(),
            ))
        }
    }
}
