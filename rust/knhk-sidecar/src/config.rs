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
    // Beat scheduler configuration
    pub beat_shard_count: usize,
    pub beat_domain_count: usize,
    pub beat_ring_capacity: usize,
    pub beat_advance_interval_ms: u64,
    // Fortune 5 configuration
    pub spiffe_enabled: bool,
    pub spiffe_socket_path: Option<String>,
    pub spiffe_trust_domain: Option<String>,
    pub spiffe_id: Option<String>,
    pub kms_provider: Option<String>, // "aws", "azure", "vault", "none"
    pub kms_region: Option<String>,
    pub kms_key_id: Option<String>,
    pub kms_vault_url: Option<String>,
    pub kms_vault_mount: Option<String>,
    pub key_rotation_interval_hours: u64,
    pub region: Option<String>,
    pub primary_region: Option<String>,
    pub cross_region_sync_enabled: bool,
    pub receipt_sync_endpoints: Vec<String>,
    pub quorum_threshold: usize,
    pub slo_r1_p99_max_ns: Option<u64>,
    pub slo_w1_p99_max_ms: Option<u64>,
    pub slo_c1_p99_max_ms: Option<u64>,
    pub slo_window_size_seconds: Option<u64>,
    pub slo_admission_strategy: String, // "strict" or "degrade"
    pub slo_enabled: bool,
    pub promotion_enabled: bool,
    pub promotion_environment: Option<String>, // "development", "staging", "production"
    pub promotion_feature_flags: Option<Vec<String>>,
    pub promotion_auto_rollback_enabled: Option<bool>,
    pub promotion_slo_threshold: Option<f64>,
    pub promotion_rollback_window_seconds: Option<u64>,
    pub promotion_traffic_percent: Option<f64>,
    pub auto_rollback_enabled: bool,
    pub slo_threshold: f64,
    // Workflow engine configuration
    #[cfg(feature = "workflow")]
    pub workflow_enabled: bool,
    #[cfg(feature = "workflow")]
    pub workflow_db_path: Option<String>,
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
            beat_shard_count: 4,
            beat_domain_count: 1,
            beat_ring_capacity: 16, // Power-of-2
            beat_advance_interval_ms: 1,
            // Fortune 5 defaults
            spiffe_enabled: false,
            spiffe_socket_path: None,
            spiffe_trust_domain: None,
            spiffe_id: None,
            kms_provider: None,
            kms_region: None,
            kms_key_id: None,
            kms_vault_url: None,
            kms_vault_mount: None,
            key_rotation_interval_hours: 24, // Fortune 5 requirement: ≤24h
            region: None,
            primary_region: None,
            cross_region_sync_enabled: false,
            receipt_sync_endpoints: Vec::new(),
            quorum_threshold: 1,
            slo_r1_p99_max_ns: 2,   // Fortune 5 requirement
            slo_w1_p99_max_ms: 1,   // Fortune 5 requirement
            slo_c1_p99_max_ms: 500, // Fortune 5 requirement
            slo_admission_strategy: "strict".to_string(),
            promotion_environment: None,
            promotion_traffic_percent: None,
            auto_rollback_enabled: false,
            slo_threshold: 0.95,
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

        // Fortune 5: SPIFFE/SPIRE configuration
        if let Ok(enabled) = std::env::var("KGC_SIDECAR_SPIFFE_ENABLED") {
            config.spiffe_enabled = enabled == "true" || enabled == "1";
        }

        if let Ok(socket) = std::env::var("KGC_SIDECAR_SPIFFE_SOCKET") {
            config.spiffe_socket_path = Some(socket);
        }

        if let Ok(trust_domain) = std::env::var("KGC_SIDECAR_SPIFFE_TRUST_DOMAIN") {
            config.spiffe_trust_domain = Some(trust_domain);
        }

        if let Ok(spiffe_id) = std::env::var("KGC_SIDECAR_SPIFFE_ID") {
            config.spiffe_id = Some(spiffe_id);
        }

        // Fortune 5: KMS configuration
        if let Ok(provider) = std::env::var("KGC_SIDECAR_KMS_PROVIDER") {
            config.kms_provider = Some(provider);
        }

        if let Ok(region) = std::env::var("KGC_SIDECAR_KMS_REGION") {
            config.kms_region = Some(region);
        }

        if let Ok(key_id) = std::env::var("KGC_SIDECAR_KMS_KEY_ID") {
            config.kms_key_id = Some(key_id);
        }

        if let Ok(vault_url) = std::env::var("KGC_SIDECAR_KMS_VAULT_URL") {
            config.kms_vault_url = Some(vault_url);
        }

        if let Ok(vault_mount) = std::env::var("KGC_SIDECAR_KMS_VAULT_MOUNT") {
            config.kms_vault_mount = Some(vault_mount);
        }

        if let Ok(interval) = std::env::var("KGC_SIDECAR_KEY_ROTATION_INTERVAL_HOURS") {
            if let Ok(parsed) = interval.parse::<u64>() {
                config.key_rotation_interval_hours = parsed;
            }
        }

        // Fortune 5: Multi-region configuration
        if let Ok(region) = std::env::var("KGC_SIDECAR_REGION") {
            config.region = Some(region);
        }

        if let Ok(primary) = std::env::var("KGC_SIDECAR_PRIMARY_REGION") {
            config.primary_region = Some(primary);
        }

        if let Ok(enabled) = std::env::var("KGC_SIDECAR_CROSS_REGION_SYNC_ENABLED") {
            config.cross_region_sync_enabled = enabled == "true" || enabled == "1";
        }

        if let Ok(endpoints) = std::env::var("KGC_SIDECAR_RECEIPT_SYNC_ENDPOINTS") {
            config.receipt_sync_endpoints =
                endpoints.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(threshold) = std::env::var("KGC_SIDECAR_QUORUM_THRESHOLD") {
            if let Ok(parsed) = threshold.parse::<usize>() {
                config.quorum_threshold = parsed;
            }
        }

        // Fortune 5: SLO configuration
        if let Ok(ns) = std::env::var("KGC_SIDECAR_SLO_R1_P99_MAX_NS") {
            if let Ok(parsed) = ns.parse::<u64>() {
                config.slo_r1_p99_max_ns = parsed;
            }
        }

        if let Ok(ms) = std::env::var("KGC_SIDECAR_SLO_W1_P99_MAX_MS") {
            if let Ok(parsed) = ms.parse::<u64>() {
                config.slo_w1_p99_max_ms = parsed;
            }
        }

        if let Ok(ms) = std::env::var("KGC_SIDECAR_SLO_C1_P99_MAX_MS") {
            if let Ok(parsed) = ms.parse::<u64>() {
                config.slo_c1_p99_max_ms = parsed;
            }
        }

        if let Ok(strategy) = std::env::var("KGC_SIDECAR_SLO_ADMISSION_STRATEGY") {
            config.slo_admission_strategy = strategy;
        }

        // Fortune 5: Promotion gates configuration
        if let Ok(env) = std::env::var("KGC_SIDECAR_PROMOTION_ENVIRONMENT") {
            config.promotion_environment = Some(env);
        }

        if let Ok(percent) = std::env::var("KGC_SIDECAR_PROMOTION_TRAFFIC_PERCENT") {
            if let Ok(parsed) = percent.parse::<f64>() {
                config.promotion_traffic_percent = Some(parsed);
            }
        }

        if let Ok(enabled) = std::env::var("KGC_SIDECAR_AUTO_ROLLBACK_ENABLED") {
            config.auto_rollback_enabled = enabled == "true" || enabled == "1";
        }

        if let Ok(threshold) = std::env::var("KGC_SIDECAR_SLO_THRESHOLD") {
            if let Ok(parsed) = threshold.parse::<f64>() {
                config.slo_threshold = parsed;
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
                return Err(format!(
                    "Weaver registry path does not exist: {}",
                    registry_path
                ));
            }
            if !std::path::Path::new(registry_path).is_dir() {
                return Err(format!(
                    "Weaver registry path is not a directory: {}",
                    registry_path
                ));
            }
        }

        // Check if ports are available (basic check - try to bind)
        // Note: This is a best-effort check, ports might be taken between check and use
        if let Err(e) = std::net::TcpStream::connect(format!("127.0.0.1:{}", self.weaver_otlp_port))
        {
            // If connection fails, port might be available (good)
            // If connection succeeds, port is in use (bad)
            if e.kind() != std::io::ErrorKind::ConnectionRefused {
                // Connection succeeded or unexpected error
                return Err(format!(
                    "Weaver OTLP port {} appears to be in use",
                    self.weaver_otlp_port
                ));
            }
        } else {
            return Err(format!(
                "Weaver OTLP port {} is already in use",
                self.weaver_otlp_port
            ));
        }

        if let Err(e) =
            std::net::TcpStream::connect(format!("127.0.0.1:{}", self.weaver_admin_port))
        {
            if e.kind() != std::io::ErrorKind::ConnectionRefused {
                return Err(format!(
                    "Weaver admin port {} appears to be in use",
                    self.weaver_admin_port
                ));
            }
        } else {
            return Err(format!(
                "Weaver admin port {} is already in use",
                self.weaver_admin_port
            ));
        }

        Ok(())
    }
}
