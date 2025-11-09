// knhk-sidecar: TLS configuration and setup

use crate::error::{SidecarError, SidecarResult};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
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

    /// Load certificate
    pub fn load_cert(&self) -> SidecarResult<Vec<u8>> {
        if let Some(ref cert_file) = self.cert_file {
            fs::read(cert_file).map_err(|e| {
                SidecarError::tls_error(format!(
                    "Failed to read certificate file {}: {}",
                    cert_file, e
                ))
            })
        } else {
            Err(SidecarError::tls_error(
                "Certificate file not configured".to_string(),
            ))
        }
    }

    /// Load private key
    pub fn load_key(&self) -> SidecarResult<Vec<u8>> {
        if let Some(ref key_file) = self.key_file {
            fs::read(key_file).map_err(|e| {
                SidecarError::tls_error(format!("Failed to read key file {}: {}", key_file, e))
            })
        } else {
            Err(SidecarError::tls_error(
                "Key file not configured".to_string(),
            ))
        }
    }

    /// Load CA certificate
    pub fn load_ca(&self) -> SidecarResult<Vec<u8>> {
        if let Some(ref ca_file) = self.ca_file {
            fs::read(ca_file).map_err(|e| {
                SidecarError::tls_error(format!(
                    "Failed to read CA certificate file {}: {}",
                    ca_file, e
                ))
            })
        } else {
            Err(SidecarError::tls_error(
                "CA certificate file not configured".to_string(),
            ))
        }
    }
}

/// Create TLS server config for tonic 0.14 using rustls
pub fn create_tls_server_config(config: &TlsConfig) -> SidecarResult<rustls::ServerConfig> {
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
    let certs: Vec<CertificateDer> = certs(&mut cert_reader)
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
    let keys: Vec<PrivateKeyDer> = pkcs8_private_keys(&mut key_reader)
        .map(|key| key.map(|k| PrivateKeyDer::Pkcs8(k)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SidecarError::tls_error(format!("Failed to parse private key: {}", e)))?;

    if keys.is_empty() {
        return Err(SidecarError::tls_error("No private keys found".to_string()));
    }

    // Configure mTLS if enabled
    if config.mtls_enabled {
        let ca_file = config
            .ca_file
            .as_ref()
            .ok_or_else(|| SidecarError::tls_error("CA file not configured".to_string()))?;
        let ca_data = fs::read(ca_file).map_err(|e| {
            SidecarError::tls_error(format!(
                "Failed to read CA certificate file {}: {}",
                ca_file, e
            ))
        })?;
        let mut ca_reader = BufReader::new(ca_data.as_slice());
        let ca_certs: Vec<CertificateDer> = certs(&mut ca_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                SidecarError::tls_error(format!("Failed to parse CA certificate: {}", e))
            })?;

        // Build root cert store for client authentication
        let mut root_store = rustls::RootCertStore::empty();
        for ca_cert in ca_certs {
            root_store.add(ca_cert).map_err(|e| {
                SidecarError::tls_error(format!("Failed to add CA certificate: {}", e))
            })?;
        }

        // Create client verifier
        let client_verifier = rustls::server::WebPkiClientVerifier::builder(root_store.into())
            .build()
            .map_err(|e| {
                SidecarError::tls_error(format!("Failed to create client verifier: {}", e))
            })?;

        rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(certs, keys[0].clone_key())
            .map_err(|e| {
                SidecarError::tls_error(format!("Failed to create mTLS server config: {}", e))
            })
    } else {
        // Regular TLS without client authentication
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, keys[0].clone_key())
            .map_err(|e| {
                SidecarError::tls_error(format!("Failed to create server TLS config: {}", e))
            })
    }
}

/// Create TLS client config for tonic 0.14 using rustls
pub fn create_tls_client_config(config: &TlsConfig) -> SidecarResult<rustls::ClientConfig> {
    config.validate()?;

    // Build root cert store
    let mut root_store = rustls::RootCertStore::empty();

    // Add system root certificates
    // Use rustls-native-certs to load system root certificates
    let native_certs = rustls_native_certs::load_native_certs().map_err(|e| {
        SidecarError::tls_error(format!("Failed to load system certificates: {}", e))
    })?;
    for cert in native_certs {
        root_store.add(cert).map_err(|e| {
            SidecarError::tls_error(format!("Failed to add system certificate: {}", e))
        })?;
    }

    // Configure mTLS if enabled
    if config.mtls_enabled {
        // Load CA certificate if provided
        if let Some(ref ca_file) = config.ca_file {
            let ca_data = fs::read(ca_file).map_err(|e| {
                SidecarError::tls_error(format!(
                    "Failed to read CA certificate file {}: {}",
                    ca_file, e
                ))
            })?;
            let mut ca_reader = BufReader::new(ca_data.as_slice());
            let ca_certs: Vec<CertificateDer> = certs(&mut ca_reader)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    SidecarError::tls_error(format!("Failed to parse CA certificate: {}", e))
                })?;

            for ca_cert in ca_certs {
                root_store.add(ca_cert).map_err(|e| {
                    SidecarError::tls_error(format!("Failed to add CA certificate: {}", e))
                })?;
            }
        }

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
        let certs: Vec<CertificateDer> = certs(&mut cert_reader)
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
        let keys: Vec<PrivateKeyDer> = pkcs8_private_keys(&mut key_reader)
            .map(|key| key.map(|k| PrivateKeyDer::Pkcs8(k)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SidecarError::tls_error(format!("Failed to parse private key: {}", e)))?;

        if keys.is_empty() {
            return Err(SidecarError::tls_error("No private keys found".to_string()));
        }

        rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(certs, keys[0].clone_key())
            .map_err(|e| {
                SidecarError::tls_error(format!("Failed to create mTLS client config: {}", e))
            })
    } else {
        // Regular TLS without client certificate
        Ok(rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth())
    }
}
