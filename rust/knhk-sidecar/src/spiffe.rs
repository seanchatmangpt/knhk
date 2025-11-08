// knhk-sidecar: SPIFFE/SPIRE integration for Fortune 5
// Service identity and automatic certificate management

use crate::error::{SidecarError, SidecarResult};
use std::path::Path;
use std::time::Duration;
use tracing::info;

/// SPIFFE configuration
#[derive(Debug, Clone)]
pub struct SpiffeConfig {
    /// SPIFFE socket path (SPIRE agent socket)
    pub socket_path: String,
    /// Trust domain
    pub trust_domain: String,
    /// Explicit SPIFFE ID (optional, will be extracted from certificate if not provided)
    pub spiffe_id: Option<String>,
    /// Certificate refresh interval (default: 1 hour)
    pub refresh_interval: Duration,
}

impl Default for SpiffeConfig {
    fn default() -> Self {
        Self {
            socket_path: "/tmp/spire-agent/public/api.sock".to_string(),
            trust_domain: "example.com".to_string(),
            spiffe_id: None,
            refresh_interval: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl SpiffeConfig {
    /// Create new SPIFFE config
    pub fn new(trust_domain: String) -> Self {
        Self {
            trust_domain,
            ..Self::default()
        }
    }

    /// Validate SPIFFE configuration
    pub fn validate(&self) -> SidecarResult<()> {
        // Check if socket path exists (SPIRE agent must be running)
        if !Path::new(&self.socket_path).exists() {
            return Err(SidecarError::config_error(format!(
                "SPIRE agent socket not found: {}",
                self.socket_path
            )));
        }

        if self.trust_domain.is_empty() {
            return Err(SidecarError::config_error(
                "Trust domain cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Extract SPIFFE ID from certificate
    ///
    /// In production, this would parse the X.509 certificate's URI SAN
    /// to extract the SPIFFE ID. For now, we return the configured ID
    /// or construct it from trust domain.
    pub fn extract_spiffe_id(&self) -> String {
        if let Some(ref id) = self.spiffe_id {
            id.clone()
        } else {
            format!("spiffe://{}/sidecar", self.trust_domain)
        }
    }
}

/// SPIFFE certificate manager
///
/// Manages certificate loading and rotation via SPIRE workload API.
/// In production, this would integrate with SPIRE agent's workload API.
pub struct SpiffeCertManager {
    config: SpiffeConfig,
    current_cert: Option<Vec<u8>>,
    current_key: Option<Vec<u8>>,
    last_refresh: Option<std::time::Instant>,
}

impl SpiffeCertManager {
    /// Create new SPIFFE certificate manager
    pub fn new(config: SpiffeConfig) -> SidecarResult<Self> {
        config.validate()?;

        Ok(Self {
            config,
            current_cert: None,
            current_key: None,
            last_refresh: None,
        })
    }

    /// Load certificate and key from SPIRE workload API
    ///
    /// In production, this would call SPIRE agent's workload API:
    /// - Connect to Unix domain socket at config.socket_path
    /// - Request X.509-SVID bundle
    /// - Extract certificate and private key
    /// - Cache until refresh_interval expires
    pub async fn load_certificate(&mut self) -> SidecarResult<()> {
        // Check if refresh is needed
        if let Some(last_refresh) = self.last_refresh {
            if last_refresh.elapsed() < self.config.refresh_interval {
                // Still valid, no refresh needed
                return Ok(());
            }
        }

        // Validate SPIRE socket exists
        if !Path::new(&self.config.socket_path).exists() {
            return Err(SidecarError::config_error(
                format!("SPIRE agent socket not found: {}. SPIFFE integration requires SPIRE agent to be running.", self.config.socket_path)
            ));
        }

        // SPIRE agent writes certificates to files in the same directory as the socket
        // Extract directory from socket path
        let socket_dir = Path::new(&self.config.socket_path)
            .parent()
            .ok_or_else(|| {
                SidecarError::config_error(format!(
                    "Invalid SPIRE socket path: {}",
                    self.config.socket_path
                ))
            })?;

        let cert_path = socket_dir.join("svid.pem");
        let key_path = socket_dir.join("key.pem");

        // Load certificates from files (SPIRE agent writes them here)
        if cert_path.exists() && key_path.exists() {
            self.current_cert = Some(std::fs::read(&cert_path).map_err(|e| {
                SidecarError::tls_error(format!(
                    "Failed to read SPIFFE certificate from {}: {}",
                    cert_path.display(),
                    e
                ))
            })?);
            self.current_key = Some(std::fs::read(&key_path).map_err(|e| {
                SidecarError::tls_error(format!(
                    "Failed to read SPIFFE private key from {}: {}",
                    key_path.display(),
                    e
                ))
            })?);

            info!("SPIFFE certificate loaded from SPIRE agent directory");
            self.last_refresh = Some(std::time::Instant::now());
            Ok(())
        } else {
            // Certificates not found - SPIRE agent may not be configured
            Err(SidecarError::config_error(
                format!("SPIRE certificate files not found at {} or {}. Ensure SPIRE agent is running and configured.", 
                    cert_path.display(), key_path.display())
            ))
        }
    }

    /// Get current certificate
    pub fn get_certificate(&self) -> SidecarResult<&[u8]> {
        self.current_cert.as_deref().ok_or_else(|| {
            SidecarError::tls_error(
                "SPIFFE certificate not loaded. Call load_certificate() first.".to_string(),
            )
        })
    }

    /// Get current private key
    pub fn get_private_key(&self) -> SidecarResult<&[u8]> {
        self.current_key.as_deref().ok_or_else(|| {
            SidecarError::tls_error(
                "SPIFFE private key not loaded. Call load_certificate() first.".to_string(),
            )
        })
    }

    /// Get SPIFFE ID
    pub fn get_spiffe_id(&self) -> String {
        self.config.extract_spiffe_id()
    }

    /// Check if certificate needs refresh
    pub fn needs_refresh(&self) -> bool {
        if let Some(last_refresh) = self.last_refresh {
            last_refresh.elapsed() >= self.config.refresh_interval
        } else {
            true // Never refreshed, needs initial load
        }
    }
}

/// Validate SPIFFE ID format
pub fn validate_spiffe_id(id: &str) -> bool {
    // SPIFFE ID format: spiffe://trust-domain/path
    id.starts_with("spiffe://") && id.len() > 10
}

/// Extract trust domain from SPIFFE ID
pub fn extract_trust_domain(spiffe_id: &str) -> Option<String> {
    if !validate_spiffe_id(spiffe_id) {
        return None;
    }

    // Remove "spiffe://" prefix
    let without_prefix = &spiffe_id[9..];

    // Find first '/' to separate trust domain from path
    if let Some(slash_pos) = without_prefix.find('/') {
        Some(without_prefix[..slash_pos].to_string())
    } else {
        Some(without_prefix.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_spiffe_id() {
        assert!(validate_spiffe_id("spiffe://example.com/sidecar"));
        assert!(validate_spiffe_id("spiffe://trust.domain/path/to/service"));
        assert!(!validate_spiffe_id("invalid"));
        assert!(!validate_spiffe_id("spiffe://"));
    }

    #[test]
    fn test_extract_trust_domain() {
        assert_eq!(
            extract_trust_domain("spiffe://example.com/sidecar"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_trust_domain("spiffe://trust.domain/path"),
            Some("trust.domain".to_string())
        );
        assert_eq!(extract_trust_domain("invalid"), None);
    }
}
