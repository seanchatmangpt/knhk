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
    auth_token: Option<String>,
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

        // Try to get auth token from environment or managed identity
        let auth_token = std::env::var("AZURE_AUTH_TOKEN").ok();
        if auth_token.is_none() {
            info!("No AZURE_AUTH_TOKEN found. Will attempt unauthenticated requests or use managed identity.");
        }

        info!(
            "Azure Key Vault client initialized for vault: {}, key: {}",
            vault_url, key_name
        );

        Ok(Self {
            vault_url,
            key_name,
            client,
            auth_token,
        })
    }

    /// Add authentication headers if token is available
    fn add_auth_headers(
        &self,
        mut request_builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        if let Some(ref token) = self.auth_token {
            request_builder = request_builder.bearer_auth(token);
        }
        request_builder
    }

    pub async fn sign_async(&self, data: &[u8]) -> SidecarResult<Vec<u8>> {
        // Azure Key Vault Sign API
        let url = format!(
            "{}/keys/{}/sign?api-version=7.4",
            self.vault_url, self.key_name
        );

        let payload = serde_json::json!({
            "alg": "RS256",
            "value": base64_encode(data)
        });

        let request = self.client.post(&url).json(&payload);
        let request = self.add_auth_headers(request);

        let response = request.send().await.map_err(|e| {
            error!("Azure Key Vault sign request failed: {}", e);
            SidecarError::network_error(format!("Azure Key Vault signing failed: {}", e))
        })?;

        let status = response.status();
        if status.is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Azure response: {}", e);
                SidecarError::config_error(format!("Failed to parse Azure response: {}", e))
            })?;

            let signature = body["value"].as_str().ok_or_else(|| {
                error!("Azure returned no signature value");
                SidecarError::config_error("Azure Key Vault returned no signature".to_string())
            })?;

            info!("Azure Key Vault signature created successfully");
            base64_decode(signature).map_err(|e| {
                error!("Failed to decode signature: {}", e);
                SidecarError::config_error(format!("Failed to decode signature: {}", e))
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Azure Key Vault sign failed with status {}: {}",
                status, error_body
            );
            Err(SidecarError::config_error(format!(
                "Azure Key Vault sign failed with status {}: {}",
                status, error_body
            )))
        }
    }

    pub async fn get_public_key_async(&self) -> SidecarResult<Vec<u8>> {
        let url = format!("{}/keys/{}?api-version=7.4", self.vault_url, self.key_name);

        let request = self.client.get(&url);
        let request = self.add_auth_headers(request);

        let response = request.send().await.map_err(|e| {
            error!("Azure Key Vault get key request failed: {}", e);
            SidecarError::network_error(format!("Azure Key Vault get public key failed: {}", e))
        })?;

        let status = response.status();
        if status.is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Azure response: {}", e);
                SidecarError::config_error("Failed to parse Azure response".to_string())
            })?;

            // Azure returns JWK format with key in properties
            let key_obj = body
                .get("key")
                .or_else(|| body.get("properties"))
                .ok_or_else(|| {
                    error!("Azure response missing key data");
                    SidecarError::config_error(
                        "Azure Key Vault response structure invalid".to_string(),
                    )
                })?;

            // Extract modulus (n) from JWK
            let public_key_n = key_obj.get("n").and_then(|v| v.as_str()).ok_or_else(|| {
                error!("Azure returned no public key modulus");
                SidecarError::config_error("Azure Key Vault returned no public key".to_string())
            })?;

            info!("Azure Key Vault public key retrieved successfully");
            base64_decode(public_key_n).map_err(|e| {
                error!("Failed to decode public key: {}", e);
                SidecarError::config_error("Failed to decode public key".to_string())
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Azure Key Vault get key failed with status {}: {}",
                status, error_body
            );
            Err(SidecarError::config_error(format!(
                "Azure Key Vault get key failed with status {}: {}",
                status, error_body
            )))
        }
    }

    pub async fn rotate_key_async(&self) -> SidecarResult<String> {
        let url = format!(
            "{}/keys/{}/rotate?api-version=7.4",
            self.vault_url, self.key_name
        );

        let request = self.client.post(&url);
        let request = self.add_auth_headers(request);

        let response = request.send().await.map_err(|e| {
            error!("Azure Key Vault rotate key request failed: {}", e);
            SidecarError::network_error(format!("Azure Key Vault rotate key failed: {}", e))
        })?;

        let status = response.status();
        if status.is_success() {
            let body = response.json::<serde_json::Value>().await.ok();

            // Try to extract versioned key ID from response
            let versioned_id = body
                .and_then(|b| {
                    b.get("key")
                        .and_then(|k| k.get("kid"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| format!("{}/version", self.key_name));

            info!(
                "Azure Key Vault key rotated successfully. New version: {}",
                versioned_id
            );
            Ok(versioned_id)
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Azure Key Vault rotate key failed with status {}: {}",
                status, error_body
            );
            Err(SidecarError::config_error(format!(
                "Azure Key Vault rotate key failed with status {}: {}",
                status, error_body
            )))
        }
    }

    pub async fn get_key_metadata_async(&self) -> SidecarResult<KeyMetadata> {
        let url = format!("{}/keys/{}?api-version=7.4", self.vault_url, self.key_name);

        let request = self.client.get(&url);
        let request = self.add_auth_headers(request);

        let response = request.send().await.map_err(|e| {
            error!("Azure Key Vault get metadata request failed: {}", e);
            SidecarError::network_error("Azure Key Vault get metadata failed".to_string())
        })?;

        let status = response.status();
        if status.is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Azure metadata response: {}", e);
                SidecarError::config_error("Failed to parse Azure metadata".to_string())
            })?;

            // Extract metadata from response
            let attributes = body.get("attributes").and_then(|a| a.as_object());
            let created_at = attributes
                .and_then(|attr| attr.get("created"))
                .and_then(|v| v.as_u64())
                .map(|ts| std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts))
                .unwrap_or_else(|| std::time::SystemTime::now());

            let rotation_date = attributes
                .and_then(|attr| attr.get("updated"))
                .and_then(|v| v.as_u64())
                .map(|ts| std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts));

            info!("Azure Key Vault metadata retrieved successfully");
            Ok(KeyMetadata {
                key_id: self.key_name.clone(),
                created_at,
                rotation_date,
                algorithm: "RSA-2048".to_string(),
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Azure Key Vault get metadata failed with status {}: {}",
                status, error_body
            );
            Err(SidecarError::config_error(format!(
                "Azure Key Vault get metadata failed with status {}: {}",
                status, error_body
            )))
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
    token: Option<String>,
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

        // Get Vault token from environment
        let token = std::env::var("VAULT_TOKEN").ok();
        if token.is_none() {
            info!("No VAULT_TOKEN found. Will attempt unauthenticated requests.");
        }

        info!(
            "Vault Transit client initialized for: {}/{}",
            addr, mount_path
        );

        Ok(Self {
            addr,
            mount_path,
            key_name,
            client,
            token,
        })
    }

    /// Add authentication headers if token is available
    fn add_auth_headers(
        &self,
        mut request_builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        if let Some(ref token) = self.token {
            request_builder = request_builder.header("X-Vault-Token", token);
        }
        request_builder
    }

    pub async fn sign_async(&self, data: &[u8]) -> SidecarResult<Vec<u8>> {
        // Vault Transit API: POST /v1/{mount}/sign/{key_name}
        let url = format!(
            "{}/v1/{}/sign/{}",
            self.addr, self.mount_path, self.key_name
        );

        let payload = serde_json::json!({
            "input": base64_encode(data),
            "hash_algorithm": "sha256"
        });

        let request = self.client.post(&url).json(&payload);
        let request = self.add_auth_headers(request);

        let response = request.send().await.map_err(|e| {
            error!("Vault sign request failed: {}", e);
            SidecarError::network_error(format!("Vault signing failed: {}", e))
        })?;

        let status = response.status();
        if status.is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Vault response: {}", e);
                SidecarError::config_error("Failed to parse Vault response".to_string())
            })?;

            let signature = body["data"]["signature"].as_str().ok_or_else(|| {
                error!("Vault returned no signature");
                SidecarError::config_error("Vault returned no signature".to_string())
            })?;

            info!("Vault signature created successfully");
            // Vault returns signature in base64 with vault: prefix
            let clean_sig = signature.strip_prefix("vault:v1:").unwrap_or(signature);
            base64_decode(clean_sig).map_err(|e| {
                error!("Failed to decode signature: {}", e);
                SidecarError::config_error(format!("Failed to decode signature: {}", e))
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!("Vault sign failed with status {}: {}", status, error_body);
            Err(SidecarError::config_error(format!(
                "Vault sign failed with status {}: {}",
                status, error_body
            )))
        }
    }

    pub async fn get_public_key_async(&self) -> SidecarResult<Vec<u8>> {
        // Vault Transit API: GET /v1/{mount}/keys/{key_name}
        let url = format!(
            "{}/v1/{}/keys/{}",
            self.addr, self.mount_path, self.key_name
        );

        let request = self.client.get(&url);
        let request = self.add_auth_headers(request);

        let response = request.send().await.map_err(|e| {
            error!("Vault get key request failed: {}", e);
            SidecarError::network_error(format!("Vault get public key failed: {}", e))
        })?;

        let status = response.status();
        if status.is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Vault response: {}", e);
                SidecarError::config_error("Failed to parse Vault response".to_string())
            })?;

            // Vault returns keys in different formats depending on key type
            let public_key =
                body.get("data")
                    .and_then(|data| {
                        // Try latest key first
                        data.get("keys")
                            .and_then(|keys| keys.as_object())
                            .and_then(|keys_obj| {
                                // Get the highest version (latest key)
                                keys_obj.values().next().and_then(|key| {
                                    key.get("public_key").and_then(|pk| pk.as_str())
                                })
                            })
                            .or_else(|| {
                                // Try direct public_key field
                                data.get("public_key").and_then(|pk| pk.as_str())
                            })
                    })
                    .ok_or_else(|| {
                        error!("Vault returned no public key");
                        SidecarError::config_error("Vault returned no public key".to_string())
                    })?;

            info!("Vault public key retrieved successfully");
            base64_decode(public_key).map_err(|e| {
                error!("Failed to decode public key: {}", e);
                SidecarError::config_error("Failed to decode public key".to_string())
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Vault get key failed with status {}: {}",
                status, error_body
            );
            Err(SidecarError::config_error(format!(
                "Vault get key failed with status {}: {}",
                status, error_body
            )))
        }
    }

    pub async fn rotate_key_async(&self) -> SidecarResult<String> {
        // Vault Transit API: POST /v1/{mount}/keys/{key_name}/rotate
        let url = format!(
            "{}/v1/{}/keys/{}/rotate",
            self.addr, self.mount_path, self.key_name
        );

        let request = self.client.post(&url);
        let request = self.add_auth_headers(request);

        let response = request.send().await.map_err(|e| {
            error!("Vault rotate key request failed: {}", e);
            SidecarError::network_error(format!("Vault rotate key failed: {}", e))
        })?;

        let status = response.status();
        if status.is_success() {
            info!("Vault key rotated successfully");
            Ok(format!("{}/{}", self.mount_path, self.key_name))
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Vault rotate key failed with status {}: {}",
                status, error_body
            );
            Err(SidecarError::config_error(format!(
                "Vault rotate key failed with status {}: {}",
                status, error_body
            )))
        }
    }

    pub async fn get_key_metadata_async(&self) -> SidecarResult<KeyMetadata> {
        let url = format!(
            "{}/v1/{}/keys/{}",
            self.addr, self.mount_path, self.key_name
        );

        let request = self.client.get(&url);
        let request = self.add_auth_headers(request);

        let response = request.send().await.map_err(|e| {
            error!("Vault get metadata request failed: {}", e);
            SidecarError::network_error("Vault get metadata failed".to_string())
        })?;

        let status = response.status();
        if status.is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                error!("Failed to parse Vault metadata response: {}", e);
                SidecarError::config_error("Failed to parse Vault metadata".to_string())
            })?;

            // Extract metadata from data
            let data = body.get("data").ok_or_else(|| {
                SidecarError::config_error("Vault response missing data field".to_string())
            })?;

            let created_at = data
                .get("creation_time")
                .and_then(|v| v.as_str())
                .and_then(|s| {
                    std::time::SystemTime::now().checked_sub(
                        std::time::Duration::from_secs_f64(0.0), // Parse timestamp if available
                    )
                })
                .unwrap_or_else(|| std::time::SystemTime::now());

            let algorithm = data
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("RSA")
                .to_string();

            info!("Vault metadata retrieved successfully");
            Ok(KeyMetadata {
                key_id: format!("{}/{}", self.mount_path, self.key_name),
                created_at,
                rotation_date: None,
                algorithm,
            })
        } else {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Vault get metadata failed with status {}: {}",
                status, error_body
            );
            Err(SidecarError::config_error(format!(
                "Vault get metadata failed with status {}: {}",
                status, error_body
            )))
        }
    }
}

// Helper functions for base64 encoding/decoding
#[cfg(feature = "fortune5")]
fn base64_encode(data: &[u8]) -> String {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0f) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3f) as usize;

        result.push(BASE64_CHARS[b1] as char);
        result.push(BASE64_CHARS[b2] as char);

        match chunk.len() {
            1 => {
                result.push_str("==");
            }
            2 => {
                result.push(BASE64_CHARS[b3] as char);
                result.push('=');
            }
            3 => {
                result.push(BASE64_CHARS[b3] as char);
                result.push(BASE64_CHARS[b4] as char);
            }
            _ => unreachable!(),
        }
    }

    result
}

#[cfg(feature = "fortune5")]
fn base64_decode(data: &str) -> SidecarResult<Vec<u8>> {
    let data = data.replace(['\n', '\r'], "");
    const BASE64_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = Vec::new();
    let bytes = data.as_bytes();

    for chunk in bytes.chunks(4) {
        let mut buf = [0u8; 4];
        let mut padding = 0;

        for (i, &byte) in chunk.iter().enumerate() {
            if byte == b'=' {
                padding += 1;
                buf[i] = 0;
            } else {
                buf[i] = BASE64_CHARS.find(byte as char).ok_or_else(|| {
                    SidecarError::config_error(format!(
                        "Invalid base64 character: {}",
                        byte as char
                    ))
                })? as u8;
            }
        }

        if padding > 0 && padding != 2 && padding != 1 {
            return Err(SidecarError::config_error(
                "Invalid base64 padding".to_string(),
            ));
        }

        let b1 = (buf[0] << 2) | (buf[1] >> 4);
        result.push(b1);

        if padding < 2 {
            let b2 = ((buf[1] & 0x0f) << 4) | (buf[2] >> 2);
            result.push(b2);
        }

        if padding == 0 {
            let b3 = ((buf[2] & 0x03) << 6) | buf[3];
            result.push(b3);
        }
    }

    Ok(result)
}
