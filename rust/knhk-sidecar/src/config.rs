// knhk-sidecar: Configuration support

use serde::{Deserialize, Serialize};
use crate::batch::BatchConfig;
use crate::retry::RetryConfig;
use crate::tls::TlsConfig;
use crate::client::ClientConfig;
use crate::server::ServerConfig;

/// Sidecar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarConfig {
    pub server: ServerConfigSection,
    pub client: ClientConfigSection,
    pub retry: RetryConfigSection,
    pub circuit_breaker: CircuitBreakerConfigSection,
    pub tls: TlsConfigSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigSection {
    pub bind_address: String,
    pub batch_window_ms: u64,
    pub max_batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfigSection {
    pub warm_orchestrator_url: String,
    pub connection_timeout_ms: u64,
    pub request_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfigSection {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfigSection {
    pub failure_threshold: u32,
    pub reset_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfigSection {
    pub enabled: bool,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
    pub ca_file: Option<String>,
    pub mtls_enabled: bool,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            server: ServerConfigSection {
                bind_address: "127.0.0.1:50051".to_string(),
                batch_window_ms: 10,
                max_batch_size: 100,
            },
            client: ClientConfigSection {
                warm_orchestrator_url: "http://localhost:50052".to_string(),
                connection_timeout_ms: 5000,
                request_timeout_ms: 30000,
            },
            retry: RetryConfigSection {
                max_retries: 3,
                initial_delay_ms: 100,
                max_delay_ms: 5000,
                multiplier: 2.0,
            },
            circuit_breaker: CircuitBreakerConfigSection {
                failure_threshold: 5,
                reset_timeout_ms: 60000,
            },
            tls: TlsConfigSection {
                enabled: false,
                cert_file: None,
                key_file: None,
                ca_file: None,
                mtls_enabled: false,
            },
        }
    }
}

impl SidecarConfig {
    /// Load config from TOML file
    pub fn from_file(path: &str) -> Result<Self, crate::error::SidecarError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::SidecarError::ConfigError(
                format!("Failed to read config file {}: {}", path, e)
            ))?;
        
        let config: SidecarConfig = toml::from_str(&content)
            .map_err(|e| crate::error::SidecarError::ConfigError(
                format!("Failed to parse config file: {}", e)
            ))?;
        
        Ok(config)
    }

    /// Convert to ServerConfig
    pub fn to_server_config(&self) -> ServerConfig {
        ServerConfig {
            bind_address: self.server.bind_address.clone(),
            batch_config: BatchConfig {
                batch_window_ms: self.server.batch_window_ms,
                max_batch_size: self.server.max_batch_size,
            },
            tls_config: self.to_tls_config(),
        }
    }

    /// Convert to ClientConfig
    pub fn to_client_config(&self) -> ClientConfig {
        ClientConfig {
            warm_orchestrator_url: self.client.warm_orchestrator_url.clone(),
            connection_timeout_ms: self.client.connection_timeout_ms,
            request_timeout_ms: self.client.request_timeout_ms,
            retry_config: RetryConfig {
                max_retries: self.retry.max_retries,
                initial_delay_ms: self.retry.initial_delay_ms,
                max_delay_ms: self.retry.max_delay_ms,
                multiplier: self.retry.multiplier,
            },
            circuit_breaker_threshold: self.circuit_breaker.failure_threshold,
            circuit_breaker_reset_ms: self.circuit_breaker.reset_timeout_ms,
        }
    }

    /// Convert to TlsConfig
    pub fn to_tls_config(&self) -> TlsConfig {
        let mut tls_config = TlsConfig::new();
        
        if self.tls.enabled {
            if let (Some(cert_file), Some(key_file)) = (&self.tls.cert_file, &self.tls.key_file) {
                if self.tls.mtls_enabled {
                    if let Some(ca_file) = &self.tls.ca_file {
                        tls_config = tls_config.with_mtls(
                            cert_file.clone(),
                            key_file.clone(),
                            ca_file.clone(),
                        );
                    } else {
                        tls_config = tls_config.with_tls(cert_file.clone(), key_file.clone());
                    }
                } else {
                    tls_config = tls_config.with_tls(cert_file.clone(), key_file.clone());
                }
            }
        }
        
        tls_config
    }
}

