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
    // Weaver live-check configuration
    pub weaver_enabled: bool,
    pub weaver_registry_path: Option<String>,
    pub weaver_otlp_port: u16,
    pub weaver_admin_port: u16,
    pub weaver_output_path: Option<String>,
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
            weaver_enabled: false,
            weaver_registry_path: None,
            weaver_otlp_port: 4317,
            weaver_admin_port: 8080,
            weaver_output_path: None,
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

        // Weaver configuration
        if let Ok(enabled) = std::env::var("KGC_SIDECAR_WEAVER_ENABLED") {
            config.weaver_enabled = enabled == "true" || enabled == "1";
        }

        if let Ok(registry) = std::env::var("KGC_SIDECAR_WEAVER_REGISTRY") {
            config.weaver_registry_path = Some(registry);
        }

        if let Ok(port) = std::env::var("KGC_SIDECAR_WEAVER_OTLP_PORT") {
            if let Ok(parsed) = port.parse::<u16>() {
                config.weaver_otlp_port = parsed;
            }
        }

        if let Ok(port) = std::env::var("KGC_SIDECAR_WEAVER_ADMIN_PORT") {
            if let Ok(parsed) = port.parse::<u16>() {
                config.weaver_admin_port = parsed;
            }
        }

        if let Ok(output) = std::env::var("KGC_SIDECAR_WEAVER_OUTPUT") {
            config.weaver_output_path = Some(output);
        }

        config
    }

    pub fn batch_timeout(&self) -> Duration {
        Duration::from_millis(self.batch_timeout_ms)
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.request_timeout_ms)
    }

    /// Validate Weaver configuration
    /// Returns error if Weaver is enabled but configuration is invalid
    pub fn validate_weaver_config(&self) -> Result<(), String> {
        if !self.weaver_enabled {
            return Ok(()); // Validation not needed if Weaver is disabled
        }

        if !self.enable_otel {
            return Err("Weaver requires OTEL to be enabled. Set enable_otel=true".to_string());
        }

        // Check Weaver binary availability
        #[cfg(feature = "otel")]
        {
            use knhk_otel::WeaverLiveCheck;
            WeaverLiveCheck::check_weaver_available()
                .map_err(|e| format!("Weaver validation failed: {}", e))?;
        }

        // Validate registry path if specified
        if let Some(ref registry_path) = self.weaver_registry_path {
            if !std::path::Path::new(registry_path).exists() {
                return Err(format!("Weaver registry path does not exist: {}", registry_path));
            }
            if !std::path::Path::new(registry_path).is_dir() {
                return Err(format!("Weaver registry path is not a directory: {}", registry_path));
            }
        }

        // Check if ports are available (basic check - try to bind)
        // Note: This is a best-effort check, ports might be taken between check and use
        if let Err(e) = std::net::TcpStream::connect(format!("127.0.0.1:{}", self.weaver_otlp_port)) {
            // If connection fails, port might be available (good)
            // If connection succeeds, port is in use (bad)
            if e.kind() != std::io::ErrorKind::ConnectionRefused {
                // Connection succeeded or unexpected error
                return Err(format!("Weaver OTLP port {} appears to be in use", self.weaver_otlp_port));
            }
        } else {
            return Err(format!("Weaver OTLP port {} is already in use", self.weaver_otlp_port));
        }

        if let Err(e) = std::net::TcpStream::connect(format!("127.0.0.1:{}", self.weaver_admin_port)) {
            if e.kind() != std::io::ErrorKind::ConnectionRefused {
                return Err(format!("Weaver admin port {} appears to be in use", self.weaver_admin_port));
            }
        } else {
            return Err(format!("Weaver admin port {} is already in use", self.weaver_admin_port));
        }

        Ok(())
    }
}

