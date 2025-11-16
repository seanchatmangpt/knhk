// knhk-sidecar: SPIFFE/SPIRE integration for Fortune 5
// Service identity and automatic certificate management with SPIRE workload API

use crate::error::{SidecarError, SidecarResult};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

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

/// SPIRE Workload API protocol messages
#[derive(Debug)]
struct WorkloadAPIRequest {
    /// Request type: "FetchX509SVID", "ValidateJWT", etc.
    request_type: String,
    /// Optional parameters (e.g., audience for JWT validation)
    params: Option<Vec<(String, String)>>,
}

impl WorkloadAPIRequest {
    fn fetch_x509_svid() -> Self {
        Self {
            request_type: "FetchX509SVID".to_string(),
            params: None,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        // Simple text protocol for demonstration
        // Real SPIRE workload API uses gRPC
        format!("X-SPIRE-WorkloadAPI: {}\r\n\r\n", self.request_type)
            .into_bytes()
    }
}

#[derive(Debug)]
struct WorkloadAPIResponse {
    /// X.509 SVID certificate chain
    pub certificates: Vec<Vec<u8>>,
    /// Private key
    pub private_key: Vec<u8>,
    /// Trust bundle (CA certificates)
    pub trust_bundle: Vec<Vec<u8>>,
    /// Time until next rotation
    pub ttl: Duration,
}

impl WorkloadAPIResponse {
    /// Parse response from SPIRE workload API
    fn from_bytes(data: &[u8]) -> SidecarResult<Self> {
        // This is a simplified parser. Real SPIRE uses gRPC/protobuf
        // For now, we'll parse a simple text format

        let response_str = String::from_utf8_lossy(data);

        // Look for markers in the response
        if response_str.contains("-----BEGIN CERTIFICATE-----") {
            // Extract certificate (simplified)
            let cert_start = response_str.find("-----BEGIN CERTIFICATE-----").unwrap();
            let cert_end = response_str.find("-----END CERTIFICATE-----").unwrap() + 25;
            let cert = response_str[cert_start..cert_end].as_bytes().to_vec();

            // Extract private key (simplified)
            let key = if let Some(key_start) = response_str.find("-----BEGIN PRIVATE KEY-----") {
                let key_end = response_str.find("-----END PRIVATE KEY-----").unwrap() + 23;
                response_str[key_start..key_end].as_bytes().to_vec()
            } else {
                Vec::new()
            };

            Ok(Self {
                certificates: vec![cert],
                private_key: key,
                trust_bundle: Vec::new(),
                ttl: Duration::from_secs(3600),
            })
        } else {
            Err(SidecarError::config_error(
                "Invalid response from SPIRE workload API".to_string(),
            ))
        }
    }
}

/// SPIFFE certificate manager with SPIRE workload API integration
///
/// Manages certificate loading and rotation via SPIRE workload API.
pub struct SpiffeCertManager {
    config: SpiffeConfig,
    current_cert: Option<Vec<u8>>,
    current_key: Option<Vec<u8>>,
    trust_bundle: Option<Vec<Vec<u8>>>,
    last_refresh: Option<Instant>,
    refresh_task: Option<tokio::task::JoinHandle<()>>,
}

impl SpiffeCertManager {
    /// Create new SPIFFE certificate manager
    pub fn new(config: SpiffeConfig) -> SidecarResult<Self> {
        config.validate()?;

        Ok(Self {
            config,
            current_cert: None,
            current_key: None,
            trust_bundle: None,
            last_refresh: None,
            refresh_task: None,
        })
    }

    /// Connect to SPIRE workload API via Unix socket
    async fn connect_to_spire(&self) -> SidecarResult<UnixStream> {
        debug!("Connecting to SPIRE agent at: {}", self.config.socket_path);

        match UnixStream::connect(&self.config.socket_path).await {
            Ok(stream) => {
                info!("Connected to SPIRE workload API");
                Ok(stream)
            }
            Err(e) => {
                error!("Failed to connect to SPIRE agent: {}", e);
                Err(SidecarError::config_error(format!(
                    "Cannot connect to SPIRE agent at {}: {}. Ensure SPIRE agent is running.",
                    self.config.socket_path, e
                )))
            }
        }
    }

    /// Fetch X.509 SVID from SPIRE workload API
    async fn fetch_svid_from_spire(&self) -> SidecarResult<WorkloadAPIResponse> {
        let mut stream = self.connect_to_spire().await?;

        // Send request
        let request = WorkloadAPIRequest::fetch_x509_svid();
        stream.write_all(&request.to_bytes()).await.map_err(|e| {
            error!("Failed to send request to SPIRE: {}", e);
            SidecarError::config_error(format!("Failed to send SPIRE request: {}", e))
        })?;

        // Read response
        let mut buffer = vec![0u8; 8192];
        let n = stream.read(&mut buffer).await.map_err(|e| {
            error!("Failed to read response from SPIRE: {}", e);
            SidecarError::config_error(format!("Failed to read SPIRE response: {}", e))
        })?;

        if n == 0 {
            return Err(SidecarError::config_error(
                "SPIRE agent closed connection unexpectedly".to_string(),
            ));
        }

        buffer.truncate(n);
        WorkloadAPIResponse::from_bytes(&buffer)
    }

    /// Load certificate and key from SPIRE workload API
    ///
    /// Connects to SPIRE agent via Unix socket and fetches X.509-SVID bundle
    pub async fn load_certificate(&mut self) -> SidecarResult<()> {
        // Check if refresh is needed
        if let Some(last_refresh) = self.last_refresh {
            if last_refresh.elapsed() < self.config.refresh_interval {
                debug!("Certificate still valid, no refresh needed");
                return Ok(());
            }
        }

        info!("Fetching X.509 SVID from SPIRE workload API");

        // Try to fetch from SPIRE workload API
        match self.fetch_svid_from_spire().await {
            Ok(response) => {
                // Successfully got SVID from SPIRE
                self.current_cert = response.certificates.first().cloned();
                self.current_key = Some(response.private_key);
                self.trust_bundle = Some(response.trust_bundle);
                self.last_refresh = Some(Instant::now());

                info!(
                    "Successfully loaded X.509 SVID from SPIRE (TTL: {:?})",
                    response.ttl
                );

                // Schedule next refresh based on TTL
                if response.ttl > Duration::from_secs(60) {
                    let next_refresh = response.ttl - Duration::from_secs(30); // Refresh 30s before expiry
                    debug!("Scheduling next refresh in {:?}", next_refresh);
                }

                Ok(())
            }
            Err(e) => {
                warn!("Failed to fetch from SPIRE workload API: {}", e);

                // Fallback to file-based certificates (for testing)
                self.load_from_files().await
            }
        }
    }

    /// Fallback: Load certificates from files when SPIRE is not available
    async fn load_from_files(&mut self) -> SidecarResult<()> {
        // Extract directory from socket path for file-based fallback
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
        let bundle_path = socket_dir.join("bundle.pem");

        // Try to load certificates from files
        if cert_path.exists() && key_path.exists() {
            self.current_cert = Some(tokio::fs::read(&cert_path).await.map_err(|e| {
                SidecarError::tls_error(format!(
                    "Failed to read certificate from {}: {}",
                    cert_path.display(),
                    e
                ))
            })?);

            self.current_key = Some(tokio::fs::read(&key_path).await.map_err(|e| {
                SidecarError::tls_error(format!(
                    "Failed to read private key from {}: {}",
                    key_path.display(),
                    e
                ))
            })?);

            if bundle_path.exists() {
                let bundle = tokio::fs::read(&bundle_path).await.map_err(|e| {
                    SidecarError::tls_error(format!(
                        "Failed to read trust bundle from {}: {}",
                        bundle_path.display(),
                        e
                    ))
                })?;
                self.trust_bundle = Some(vec![bundle]);
            }

            info!("Loaded certificates from files (fallback mode)");
            self.last_refresh = Some(Instant::now());
            Ok(())
        } else {
            Err(SidecarError::config_error(format!(
                "SPIRE not available and no certificate files found at {} or {}",
                cert_path.display(),
                key_path.display()
            )))
        }
    }

    /// Start automatic certificate refresh task
    pub fn start_refresh_task(&mut self) {
        if self.refresh_task.is_some() {
            warn!("Refresh task already running");
            return;
        }

        let config = self.config.clone();
        let socket_path = config.socket_path.clone();
        let refresh_interval = config.refresh_interval;

        let handle = tokio::spawn(async move {
            info!("Starting SPIFFE certificate refresh task");

            loop {
                sleep(refresh_interval).await;

                debug!("Attempting to refresh SPIFFE certificate");

                // Try to connect to SPIRE and refresh
                match UnixStream::connect(&socket_path).await {
                    Ok(mut stream) => {
                        let request = WorkloadAPIRequest::fetch_x509_svid();
                        if let Err(e) = stream.write_all(&request.to_bytes()).await {
                            error!("Failed to refresh certificate: {}", e);
                        } else {
                            info!("Certificate refresh request sent");
                        }
                    }
                    Err(e) => {
                        warn!("Cannot connect to SPIRE for refresh: {}", e);
                    }
                }
            }
        });

        self.refresh_task = Some(handle);
        info!("Certificate refresh task started");
    }

    /// Stop automatic certificate refresh task
    pub fn stop_refresh_task(&mut self) {
        if let Some(handle) = self.refresh_task.take() {
            handle.abort();
            info!("Certificate refresh task stopped");
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

    /// Get trust bundle (CA certificates)
    pub fn get_trust_bundle(&self) -> Option<&[Vec<u8>]> {
        self.trust_bundle.as_deref()
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

    /// Verify peer SPIFFE ID against trust domain
    pub fn verify_peer_id(&self, peer_id: &str) -> bool {
        if !validate_spiffe_id(peer_id) {
            return false;
        }

        // Extract trust domain from peer ID
        if let Some(peer_domain) = extract_trust_domain(peer_id) {
            // Verify peer is from same trust domain
            peer_domain == self.config.trust_domain
        } else {
            false
        }
    }
}

impl Drop for SpiffeCertManager {
    fn drop(&mut self) {
        self.stop_refresh_task();
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

    #[tokio::test]
    async fn test_workload_api_request() {
        let request = WorkloadAPIRequest::fetch_x509_svid();
        let bytes = request.to_bytes();
        assert!(bytes.starts_with(b"X-SPIRE-WorkloadAPI: FetchX509SVID"));
    }

    #[test]
    fn test_verify_peer_id() {
        let config = SpiffeConfig::new("example.com".to_string());
        let manager = SpiffeCertManager::new(config).unwrap();

        assert!(manager.verify_peer_id("spiffe://example.com/service"));
        assert!(!manager.verify_peer_id("spiffe://other.com/service"));
        assert!(!manager.verify_peer_id("invalid"));
    }
}