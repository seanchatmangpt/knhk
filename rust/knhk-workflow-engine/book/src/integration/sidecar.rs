//! Sidecar Integration for Workflow Engine
//!
//! Provides integration with KNHK sidecar for Fortune 5 features:
//! - SPIFFE/SPIRE authentication
//! - KMS integration for signing and key management
//! - Service mesh capabilities
//! - mTLS communication
//! - Health checks and readiness probes

use crate::error::{WorkflowError, WorkflowResult};
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "sidecar")]
use knhk_sidecar::{
    spiffe::{SpiffeCertManager, SpiffeConfig},
    kms::{KmsManager, KmsConfig},
    error::SidecarError,
};

/// Sidecar integration for workflow engine
pub struct SidecarIntegration {
    #[cfg(feature = "sidecar")]
    spiffe_manager: Arc<RwLock<Option<SpiffeCertManager>>>,
    #[cfg(feature = "sidecar")]
    kms_manager: Arc<RwLock<Option<KmsManager>>>,
    #[cfg(feature = "sidecar")]
    spiffe_config: Option<SpiffeConfig>,
    #[cfg(feature = "sidecar")]
    kms_config: Option<KmsConfig>,
    enabled: bool,
}

impl SidecarIntegration {
    /// Create new sidecar integration
    pub fn new(enabled: bool) -> Self {
        #[cfg(feature = "sidecar")]
        {
            Self {
                spiffe_manager: Arc::new(RwLock::new(None)),
                kms_manager: Arc::new(RwLock::new(None)),
                spiffe_config: None,
                kms_config: None,
                enabled,
            }
        }

        #[cfg(not(feature = "sidecar"))]
        {
            Self { enabled }
        }
    }

    /// Create new sidecar integration with SPIFFE config
    #[cfg(feature = "sidecar")]
    pub fn with_spiffe(spiffe_config: SpiffeConfig) -> Self {
        Self {
            spiffe_manager: Arc::new(RwLock::new(None)),
            kms_manager: Arc::new(RwLock::new(None)),
            spiffe_config: Some(spiffe_config),
            kms_config: None,
            enabled: true,
        }
    }

    /// Create new sidecar integration with KMS config
    #[cfg(feature = "sidecar")]
    pub fn with_kms(kms_config: KmsConfig) -> Self {
        Self {
            spiffe_manager: Arc::new(RwLock::new(None)),
            kms_manager: Arc::new(RwLock::new(None)),
            spiffe_config: None,
            kms_config: Some(kms_config),
            enabled: true,
        }
    }

    /// Create new sidecar integration with both SPIFFE and KMS configs
    #[cfg(feature = "sidecar")]
    pub fn with_configs(spiffe_config: Option<SpiffeConfig>, kms_config: Option<KmsConfig>) -> Self {
        Self {
            spiffe_manager: Arc::new(RwLock::new(None)),
            kms_manager: Arc::new(RwLock::new(None)),
            spiffe_config,
            kms_config,
            enabled: true,
        }
    }

    /// Initialize sidecar integration
    pub async fn initialize(&self) -> WorkflowResult<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(feature = "sidecar")]
        {
            // Initialize SPIFFE cert manager if config provided
            if let Some(ref spiffe_config) = self.spiffe_config {
                let mut manager = SpiffeCertManager::new(spiffe_config.clone())
                    .map_err(|e| WorkflowError::Integration(format!("Failed to create SPIFFE cert manager: {}", e)))?;
                
                // Load certificate from SPIRE agent
                manager.load_certificate().await
                    .map_err(|e| WorkflowError::Integration(format!("Failed to load SPIFFE certificate: {}", e)))?;
                
                let mut guard = self.spiffe_manager.write().await;
                *guard = Some(manager);
                tracing::info!("SPIFFE integration initialized");
            }

            // Initialize KMS manager if config provided
            if let Some(ref kms_config) = self.kms_config {
                let manager = KmsManager::new(kms_config.clone())
                    .map_err(|e| WorkflowError::Integration(format!("Failed to create KMS manager: {}", e)))?;
                
                let mut guard = self.kms_manager.write().await;
                *guard = Some(manager);
                tracing::info!("KMS integration initialized");
            }
        }

        Ok(())
    }

    /// Get service identity (SPIFFE ID)
    pub async fn get_identity(&self) -> WorkflowResult<Option<String>> {
        if !self.enabled {
            return Ok(None);
        }

        #[cfg(feature = "sidecar")]
        {
            let guard = self.spiffe_manager.read().await;
            if let Some(ref manager) = *guard {
                Ok(Some(manager.get_spiffe_id()))
            } else {
                Ok(None)
            }
        }

        #[cfg(not(feature = "sidecar"))]
        {
            Ok(None)
        }
    }

    /// Sign data using KMS
    ///
    /// Note: KMS is for signing/encryption, not secret storage.
    /// For secrets, use a secrets manager (e.g., HashiCorp Vault, AWS Secrets Manager).
    pub async fn sign_data(&self, data: &[u8]) -> WorkflowResult<Option<Vec<u8>>> {
        if !self.enabled {
            return Ok(None);
        }

        #[cfg(feature = "sidecar")]
        {
            let guard = self.kms_manager.read().await;
            if let Some(ref manager) = *guard {
                let signature = manager.sign(data)
                    .map_err(|e| WorkflowError::Integration(format!("KMS signing failed: {}", e)))?;
                Ok(Some(signature))
            } else {
                Ok(None)
            }
        }

        #[cfg(not(feature = "sidecar"))]
        {
            Ok(None)
        }
    }

    /// Verify peer identity (for mTLS)
    pub async fn verify_peer(&self, peer_spiffe_id: &str) -> WorkflowResult<bool> {
        if !self.enabled {
            return Ok(true); // Skip verification if disabled
        }

        #[cfg(feature = "sidecar")]
        {
            use knhk_sidecar::spiffe::{validate_spiffe_id, extract_trust_domain};
            
            // Validate SPIFFE ID format
            if !validate_spiffe_id(peer_spiffe_id) {
                return Ok(false);
            }

            // Get our trust domain from SPIFFE config
            if let Some(ref spiffe_config) = self.spiffe_config {
                let our_trust_domain = &spiffe_config.trust_domain;
                let peer_trust_domain = extract_trust_domain(peer_spiffe_id);
                
                if let Some(peer_td) = peer_trust_domain {
                    Ok(peer_td == *our_trust_domain)
                } else {
                    Ok(false)
                }
            } else {
                // No SPIFFE config, can't verify
                Ok(false)
            }
        }

        #[cfg(not(feature = "sidecar"))]
        {
            Ok(true)
        }
    }

    /// Check if sidecar is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Rotate KMS key if needed
    #[cfg(feature = "sidecar")]
    pub async fn rotate_kms_key(&self) -> WorkflowResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut guard = self.kms_manager.write().await;
        if let Some(ref mut manager) = *guard {
            manager.rotate_if_needed().await
                .map_err(|e| WorkflowError::Integration(format!("KMS key rotation failed: {}", e)))?;
        }
        Ok(())
    }

    /// Refresh SPIFFE certificate if needed
    #[cfg(feature = "sidecar")]
    pub async fn refresh_spiffe_cert(&self) -> WorkflowResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut guard = self.spiffe_manager.write().await;
        if let Some(ref mut manager) = *guard {
            if manager.needs_refresh() {
                manager.load_certificate().await
                    .map_err(|e| WorkflowError::Integration(format!("SPIFFE certificate refresh failed: {}", e)))?;
            }
        }
        Ok(())
    }
}

impl Default for SidecarIntegration {
    fn default() -> Self {
        Self::new(false)
    }
}

