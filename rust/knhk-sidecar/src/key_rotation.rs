// knhk-sidecar: Automatic key rotation manager for Fortune 5
// Enforces ≤24h key rotation requirement

use crate::error::{SidecarError, SidecarResult};
use crate::kms::KmsManager;
use crate::spiffe::SpiffeCertManager;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Key rotation manager
///
/// Manages automatic rotation of both KMS keys and SPIFFE certificates
/// to meet Fortune 5 ≤24h rotation requirement.
pub struct KeyRotationManager {
    rotation_interval: Duration,
    enabled: bool,
    last_rotation_check: Option<Instant>,
}

impl KeyRotationManager {
    /// Create new key rotation manager
    pub fn new(rotation_interval: Duration) -> SidecarResult<Self> {
        // Validate rotation interval (must be ≤24h for Fortune 5)
        if rotation_interval > Duration::from_secs(86400) {
            return Err(SidecarError::config_error(format!(
                "Key rotation interval {}s exceeds Fortune 5 requirement of 24 hours (86400s)",
                rotation_interval.as_secs()
            )));
        }

        Ok(Self {
            rotation_interval,
            enabled: true,
            last_rotation_check: None,
        })
    }

    /// Enable or disable automatic rotation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!(
                "Key rotation enabled (interval: {}s)",
                self.rotation_interval.as_secs()
            );
        } else {
            warn!("Key rotation disabled");
        }
    }

    /// Check if rotation is needed
    pub fn needs_rotation(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(last_check) = self.last_rotation_check {
            last_check.elapsed() >= self.rotation_interval
        } else {
            true // Never checked, needs initial check
        }
    }

    /// Perform rotation check and rotate if needed
    ///
    /// Takes KMS and SPIFFE managers as parameters to avoid ownership issues
    pub async fn check_and_rotate(
        &mut self,
        kms_manager: Option<&mut KmsManager>,
        spiffe_manager: Option<&mut SpiffeCertManager>,
    ) -> SidecarResult<()> {
        if !self.needs_rotation() {
            return Ok(());
        }

        info!(
            "Performing key rotation check (interval: {}s)",
            self.rotation_interval.as_secs()
        );

        // Rotate KMS key if configured
        if let Some(kms) = kms_manager {
            if kms.needs_rotation() {
                kms.rotate_if_needed().await?;
            }
        }

        // Refresh SPIFFE certificate if configured
        if let Some(spiffe) = spiffe_manager {
            if spiffe.needs_refresh() {
                spiffe.load_certificate().await?;
            }
        }

        self.last_rotation_check = Some(Instant::now());
        Ok(())
    }

    /// Start background rotation task
    ///
    /// Spawns a background task that periodically checks and rotates keys.
    /// Takes Arc<Mutex<>> references to KMS and SPIFFE managers for shared access.
    pub fn start_background_task(
        self,
        kms_manager: Option<std::sync::Arc<tokio::sync::Mutex<Option<KmsManager>>>>,
        spiffe_manager: Option<std::sync::Arc<tokio::sync::Mutex<Option<SpiffeCertManager>>>>,
    ) -> tokio::task::JoinHandle<()> {
        let check_interval = Duration::from_secs(3600); // Check every hour
        let rotation_interval = self.rotation_interval;

        tokio::spawn(async move {
            let mut rotation_manager = self;
            loop {
                sleep(check_interval).await;

                // Perform rotation check with managers
                let mut kms_guard = if let Some(ref kms) = kms_manager {
                    Some(kms.lock().await)
                } else {
                    None
                };

                let mut spiffe_guard = if let Some(ref spiffe) = spiffe_manager {
                    Some(spiffe.lock().await)
                } else {
                    None
                };

                if let Err(e) = rotation_manager
                    .check_and_rotate(
                        kms_guard.as_mut().and_then(|m| m.as_mut()),
                        spiffe_guard.as_mut().and_then(|m| m.as_mut()),
                    )
                    .await
                {
                    error!("Key rotation check failed: {}", e);
                }
            }
        })
    }

    /// Get rotation status
    pub fn get_status(
        &self,
        kms_manager: Option<&KmsManager>,
        spiffe_manager: Option<&SpiffeCertManager>,
    ) -> RotationStatus {
        let kms_status = kms_manager
            .map(|kms| {
                if kms.needs_rotation() {
                    "needs_rotation"
                } else {
                    "ok"
                }
            })
            .unwrap_or("not_configured");

        let spiffe_status = spiffe_manager
            .map(|spiffe| {
                if spiffe.needs_refresh() {
                    "needs_refresh"
                } else {
                    "ok"
                }
            })
            .unwrap_or("not_configured");

        RotationStatus {
            enabled: self.enabled,
            rotation_interval_seconds: self.rotation_interval.as_secs(),
            kms_status: kms_status.to_string(),
            spiffe_status: spiffe_status.to_string(),
            last_check: self.last_rotation_check,
        }
    }
}

/// Rotation status
#[derive(Debug, Clone)]
pub struct RotationStatus {
    pub enabled: bool,
    pub rotation_interval_seconds: u64,
    pub kms_status: String,
    pub spiffe_status: String,
    pub last_check: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_interval_validation() {
        // Valid: 24 hours
        assert!(KeyRotationManager::new(Duration::from_secs(86400)).is_ok());

        // Valid: Less than 24 hours
        assert!(KeyRotationManager::new(Duration::from_secs(3600)).is_ok());

        // Invalid: More than 24 hours
        assert!(KeyRotationManager::new(Duration::from_secs(86401)).is_err());
    }
}
