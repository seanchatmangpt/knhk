// knhk-sidecar: TLS configuration and setup

use crate::error::{SidecarError, SidecarResult};
use rustls::{Certificate, ClientConfig, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs;
use std::io::BufReader;
use std::path::Path;

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// TLS enabled
    pub enabled: bool,

    /// Certificate file path
    pub cert_file: Option<String>,

    /// Private key file path
    pub key_file: Option<String>,

    /// CA certificate file path (for mTLS)
    pub ca_file: Option<String>,

    /// mTLS enabled
    pub mtls_enabled: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_file: None,
            key_file: None,
            ca_file: None,
            mtls_enabled: false,
        }
    }
}

impl TlsConfig {
    /// Create new TLS config
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable TLS
    pub fn with_tls(mut self, cert_file: String, key_file: String) -> Self {
        self.enabled = true;
        self.cert_file = Some(cert_file);
        self.key_file = Some(key_file);
        self
    }

    /// Enable mTLS
    pub fn with_mtls(mut self, cert_file: String, key_file: String, ca_file: String) -> Self {
        self.enabled = true;
        self.mtls_enabled = true;
        self.cert_file = Some(cert_file);
        self.key_file = Some(key_file);
        self.ca_file = Some(ca_file);
        self
    }

    /// Validate TLS configuration
    pub fn validate(&self) -> SidecarResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Validate certificate file exists
        if let Some(ref cert_file) = self.cert_file {
            if !Path::new(cert_file).exists() {
                return Err(SidecarError::tls_error(format!(
                    "Certificate file not found: {}",
                    cert_file
                )));
            }
        } else {
            return Err(SidecarError::tls_error(
                "Certificate file required when TLS is enabled".to_string(),
            ));
        }

        // Validate key file exists
        if let Some(ref key_file) = self.key_file {
            if !Path::new(key_file).exists() {
                return Err(SidecarError::tls_error(format!(
                    "Key file not found: {}",
                    key_file
                )));
            }
        } else {
            return Err(SidecarError::tls_error(
                "Key file required when TLS is enabled".to_string(),
            ));
        }

        // Validate CA file exists if mTLS is enabled
        if self.mtls_enabled {
            if let Some(ref ca_file) = self.ca_file {
                if !Path::new(ca_file).exists() {
                    return Err(SidecarError::tls_error(format!(
                        "CA certificate file not found: {}",
                        ca_file
                    )));
                }
            } else {
                return Err(SidecarError::tls_error(
                    "CA certificate file required when mTLS is enabled".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Create TLS server config for tonic 0.14 (using rustls)
pub fn create_tls_server_config(config: &TlsConfig) -> SidecarResult<ServerConfig> {
    config.validate()?;

    // Load certificate
    let cert_file = config
        .cert_file
        .as_ref()
        .ok_or_else(|| SidecarError::tls_error("Certificate file not configured".to_string()))?;
    let cert_data = fs::read(cert_file).map_err(|e| {
        SidecarError::tls_error(format!(
            "Failed to read certificate file {}: {}",
            cert_file, e
        ))
    })?;
    let mut cert_reader = BufReader::new(cert_data.as_slice());
    let certs = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SidecarError::tls_error(format!("Failed to parse certificate: {}", e)))?;

    // Load private key
    let key_file = config
        .key_file
        .as_ref()
        .ok_or_else(|| SidecarError::tls_error("Key file not configured".to_string()))?;
    let key_data = fs::read(key_file).map_err(|e| {
        SidecarError::tls_error(format!("Failed to read key file {}: {}", key_file, e))
    })?;
    let mut key_reader = BufReader::new(key_data.as_slice());
    let keys = pkcs8_private_keys(&mut key_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SidecarError::tls_error(format!("Failed to parse private key: {}", e)))?;

    if keys.is_empty() {
        return Err(SidecarError::tls_error("No private keys found".to_string()));
    }

    // Convert certificates and keys to rustls types
    let certs: Vec<Certificate> = certs.into_iter().map(Certificate).collect();
    let key = PrivateKey(keys[0].clone());

    // Build server config
    let mut server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| {
            SidecarError::tls_error(format!("Failed to create server TLS config: {}", e))
        })?;

    // Configure mTLS if enabled
    if config.mtls_enabled {
        let ca_file = config.ca_file.as_ref().ok_or_else(|| {
            SidecarError::tls_error("CA certificate file not configured".to_string())
        })?;
        let ca_data = fs::read(ca_file).map_err(|e| {
            SidecarError::tls_error(format!(
                "Failed to read CA certificate file {}: {}",
                ca_file, e
            ))
        })?;
        let mut ca_reader = BufReader::new(ca_data.as_slice());
        let ca_certs = certs(&mut ca_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                SidecarError::tls_error(format!("Failed to parse CA certificate: {}", e))
            })?;

        let ca_certs: Vec<Certificate> = ca_certs.into_iter().map(Certificate).collect();
        let mut client_auth = rustls::server::ClientAuthConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(
                ca_certs
                    .into_iter()
                    .map(|c| rustls::pki_types::CertificateDer::from(c.0))
                    .collect(),
            )
            .build();

        server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(client_auth)
            .with_single_cert(certs, key)
            .map_err(|e| {
                SidecarError::tls_error(format!("Failed to create mTLS server config: {}", e))
            })?;
    }

    Ok(server_config)
}

/// Create TLS client config for tonic 0.14 (using rustls)
pub fn create_tls_client_config(config: &TlsConfig) -> SidecarResult<ClientConfig> {
    config.validate()?;

    // Build client config
    let mut client_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates({
            let mut roots = rustls::RootCertStore::empty();
            // Add system root certificates
            roots.add_trust_anchors(rustls::crypto::aws_lc_rs::default_provider().trust_anchors());
            roots
        })
        .with_no_client_auth();

    // Configure mTLS if enabled
    if config.mtls_enabled {
        // Load client certificate and key
        let cert_file = config.cert_file.as_ref().ok_or_else(|| {
            SidecarError::tls_error("Certificate file not configured".to_string())
        })?;
        let cert_data = fs::read(cert_file).map_err(|e| {
            SidecarError::tls_error(format!(
                "Failed to read certificate file {}: {}",
                cert_file, e
            ))
        })?;
        let mut cert_reader = BufReader::new(cert_data.as_slice());
        let certs = certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SidecarError::tls_error(format!("Failed to parse certificate: {}", e)))?;

        let key_file = config
            .key_file
            .as_ref()
            .ok_or_else(|| SidecarError::tls_error("Key file not configured".to_string()))?;
        let key_data = fs::read(key_file).map_err(|e| {
            SidecarError::tls_error(format!("Failed to read key file {}: {}", key_file, e))
        })?;
        let mut key_reader = BufReader::new(key_data.as_slice());
        let keys = pkcs8_private_keys(&mut key_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SidecarError::tls_error(format!("Failed to parse private key: {}", e)))?;

        if keys.is_empty() {
            return Err(SidecarError::tls_error("No private keys found".to_string()));
        }

        let certs: Vec<Certificate> = certs.into_iter().map(Certificate).collect();
        let key = PrivateKey(keys[0].clone());

        // Add client identity
        client_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates({
                let mut roots = rustls::RootCertStore::empty();
                if let Some(ref ca_file) = config.ca_file {
                    let ca_data = fs::read(ca_file).map_err(|e| {
                        SidecarError::tls_error(format!(
                            "Failed to read CA certificate file {}: {}",
                            ca_file, e
                        ))
                    })?;
                    let mut ca_reader = BufReader::new(ca_data.as_slice());
                    let ca_certs = certs(&mut ca_reader)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| {
                            SidecarError::tls_error(format!(
                                "Failed to parse CA certificate: {}",
                                e
                            ))
                        })?;
                    for ca_cert in ca_certs {
                        roots
                            .add(rustls::pki_types::CertificateDer::from(ca_cert))
                            .map_err(|e| {
                                SidecarError::tls_error(format!(
                                    "Failed to add CA certificate: {}",
                                    e
                                ))
                            })?;
                    }
                }
                // Also add system root certificates
                roots.add_trust_anchors(
                    rustls::crypto::aws_lc_rs::default_provider().trust_anchors(),
                );
                roots
            })
            .with_client_auth_cert(certs, key)
            .map_err(|e| {
                SidecarError::tls_error(format!("Failed to create mTLS client config: {}", e))
            })?;
    }

    Ok(client_config)
}
