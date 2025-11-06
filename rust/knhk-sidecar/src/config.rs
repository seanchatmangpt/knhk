// rust/knhk-sidecar/src/config.rs
// Configuration management for KGC Sidecar

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SidecarConfig {
    pub listen_address: String,
    pub tls_enabled: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub tls_ca_path: Option<String>,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub retry_max_attempts: u32,
    pub retry_initial_delay_ms: u64,
    pub retry_max_delay_ms: u64,
    pub circuit_breaker_failure_threshold: u32,
    pub circuit_breaker_reset_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub enable_otel: bool,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:50051".to_string(),
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            tls_ca_path: None,
            batch_size: 100,
            batch_timeout_ms: 100,
            retry_max_attempts: 3,
            retry_initial_delay_ms: 100,
            retry_max_delay_ms: 1000,
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_reset_timeout_ms: 60000,
            request_timeout_ms: 5000,
            enable_otel: true,
        }
    }
}

impl SidecarConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(addr) = std::env::var("KGC_SIDECAR_ADDRESS") {
            config.listen_address = addr;
        }

        if let Ok(enabled) = std::env::var("KGC_SIDECAR_TLS_ENABLED") {
            config.tls_enabled = enabled == "true" || enabled == "1";
        }

        if let Ok(cert) = std::env::var("KGC_SIDECAR_TLS_CERT") {
            config.tls_cert_path = Some(cert);
        }

        if let Ok(key) = std::env::var("KGC_SIDECAR_TLS_KEY") {
            config.tls_key_path = Some(key);
        }

        if let Ok(ca) = std::env::var("KGC_SIDECAR_TLS_CA") {
            config.tls_ca_path = Some(ca);
        }

        if let Ok(size) = std::env::var("KGC_SIDECAR_BATCH_SIZE") {
            if let Ok(parsed) = size.parse::<usize>() {
                config.batch_size = parsed;
            }
        }

        if let Ok(timeout) = std::env::var("KGC_SIDECAR_BATCH_TIMEOUT_MS") {
            if let Ok(parsed) = timeout.parse::<u64>() {
                config.batch_timeout_ms = parsed;
            }
        }

        if let Ok(attempts) = std::env::var("KGC_SIDECAR_RETRY_MAX_ATTEMPTS") {
            if let Ok(parsed) = attempts.parse::<u32>() {
                config.retry_max_attempts = parsed;
            }
        }

        if let Ok(timeout) = std::env::var("KGC_SIDECAR_REQUEST_TIMEOUT_MS") {
            if let Ok(parsed) = timeout.parse::<u64>() {
                config.request_timeout_ms = parsed;
            }
        }

        config
    }

    pub fn batch_timeout(&self) -> Duration {
        Duration::from_millis(self.batch_timeout_ms)
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.request_timeout_ms)
    }
}

