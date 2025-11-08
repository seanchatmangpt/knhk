// knhk-sidecar: KGC Sidecar Service
// gRPC proxy for enterprise apps with batching, retries, circuit-breaking, and TLS

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
// EXCEPTION: Mutex poisoning expect() calls are acceptable (see metrics.rs, health.rs, batch.rs)
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod batch;
pub mod beat_admission; // Beat-driven admission for 8-beat epoch
pub mod circuit_breaker;
pub mod client;
pub mod config;
pub mod error;
pub mod health;
pub mod json_parser; // JSON parsing with simdjson
pub mod metrics;
pub mod retry;
pub mod server;
pub mod service;
pub mod tls; // gRPC service with beat-driven admission
             // Fortune 5 modules
pub mod capacity; // Capacity planning
pub mod key_rotation; // Automatic key rotation (≤24h)
pub mod kms; // HSM/KMS integration
pub mod multi_region; // Multi-region support
pub mod promotion;
pub mod slo_admission; // SLO-based admission control
pub mod spiffe; // SPIFFE/SPIRE integration // Formal promotion gates

pub use client::SidecarClient;
pub use config::SidecarConfig;
pub use error::{SidecarError, SidecarResult};
pub use server::SidecarServer;

use std::process::Child;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

/// Run the sidecar server with Weaver live-check integration
#[cfg(feature = "otel")]
pub async fn run(config: SidecarConfig) -> Result<(), Box<dyn std::error::Error>> {
    use crate::batch::BatchConfig;
    use crate::health::HealthChecker;
    use crate::metrics::MetricsCollector;
    use crate::server::{ServerConfig, SidecarServer};
    use crate::tls::TlsConfig;
    use knhk_otel::WeaverLiveCheck;
    use tracing::{error, info, warn};

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Validate Weaver configuration if enabled
    if config.weaver_enabled {
        config
            .validate_weaver_config()
            .map_err(|e| format!("Weaver configuration validation failed: {}", e))?;
    }

    // Helper function to start Weaver with retry logic
    async fn start_weaver_with_verification(
        config: &SidecarConfig,
    ) -> Result<(Child, String), String> {
        let mut weaver_builder = WeaverLiveCheck::new()
            .with_otlp_port(config.weaver_otlp_port)
            .with_admin_port(config.weaver_admin_port)
            .with_format("json".to_string())
            .with_inactivity_timeout(3600); // 1 hour default timeout

        if let Some(ref registry) = config.weaver_registry_path {
            weaver_builder = weaver_builder.with_registry(registry.clone());
        } else {
            // Use default registry path if not specified
            let default_registry = "./registry".to_string();
            if std::path::Path::new(&default_registry).exists() {
                weaver_builder = weaver_builder.with_registry(default_registry);
            } else {
                warn!("Weaver registry not found at default path ./registry, running without registry validation");
            }
        }

        if let Some(ref output) = config.weaver_output_path {
            weaver_builder = weaver_builder.with_output(output.clone());
        } else {
            // Use default output directory
            let default_output = "./weaver-reports".to_string();
            std::fs::create_dir_all(&default_output)
                .map_err(|e| format!("Failed to create Weaver output directory: {}", e))?;
            weaver_builder = weaver_builder.with_output(default_output);
        }

        // Start Weaver process
        let process = weaver_builder.start()?;
        let endpoint = format!("http://127.0.0.1:{}", config.weaver_otlp_port);

        // Wait for Weaver to initialize (2 seconds)
        sleep(Duration::from_secs(2)).await;

        // Verify Weaver is healthy (retry up to 3 times)
        let mut health_check_passed = false;
        for attempt in 1..=3 {
            match weaver_builder.check_health() {
                Ok(true) => {
                    health_check_passed = true;
                    break;
                }
                Ok(false) => {
                    warn!(attempt, "Weaver health check returned false, retrying...");
                }
                Err(e) => {
                    warn!(attempt, error = %e, "Weaver health check failed, retrying...");
                }
            }
            if attempt < 3 {
                sleep(Duration::from_secs(1)).await;
            }
        }

        if !health_check_passed {
            // Try to stop the process
            let _ = weaver_builder.stop();
            return Err("Weaver started but health check failed after 3 attempts".to_string());
        }

        Ok((process, endpoint))
    }

    // Start Weaver live-check if enabled
    let weaver_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let weaver_endpoint: Option<String> = if config.weaver_enabled && config.enable_otel {
        info!("Starting Weaver live-check for sidecar telemetry validation");

        match start_weaver_with_verification(&config).await {
            Ok((process, endpoint)) => {
                info!(endpoint = %endpoint, "Weaver live-check started and verified successfully");
                *weaver_process
                    .lock()
                    .map_err(|e| format!("Mutex poisoned: {}", e))? = Some(process);
                Some(endpoint)
            }
            Err(e) => {
                error!(error = %e, "Failed to start Weaver live-check");
                return Err(format!("Failed to start Weaver live-check: {}", e).into());
            }
        }
    } else {
        None
    };

    // Start background task to monitor Weaver process health
    let weaver_monitor_process = Arc::clone(&weaver_process);
    let weaver_monitor_config = config.clone();

    if config.weaver_enabled && weaver_endpoint.is_some() {
        // Clone builder configuration for restart
        let weaver_builder_config = {
            let mut builder = WeaverLiveCheck::new()
                .with_otlp_port(config.weaver_otlp_port)
                .with_admin_port(config.weaver_admin_port)
                .with_format("json".to_string())
                .with_inactivity_timeout(3600);

            if let Some(ref registry) = config.weaver_registry_path {
                builder = builder.with_registry(registry.clone());
            } else {
                let default_registry = "./registry".to_string();
                if std::path::Path::new(&default_registry).exists() {
                    builder = builder.with_registry(default_registry);
                }
            }

            if let Some(ref output) = config.weaver_output_path {
                builder = builder.with_output(output.clone());
            } else {
                let default_output = "./weaver-reports".to_string();
                let _ = std::fs::create_dir_all(&default_output);
                builder = builder.with_output(default_output);
            }

            builder
        };

        tokio::spawn(async move {
            let mut restart_count = 0u32;
            let mut last_restart_time = std::time::Instant::now();
            const MAX_RESTARTS_PER_MINUTE: u32 = 5;

            loop {
                sleep(Duration::from_secs(5)).await;

                let process_needs_restart = {
                    let mut process_guard = match weaver_monitor_process.lock() {
                        Ok(guard) => guard,
                        Err(e) => {
                            error!("Mutex poisoned in weaver monitor: {}", e);
                            continue; // Skip this iteration
                        }
                    };
                    match process_guard.as_mut() {
                        Some(process) => {
                            // Check if process is still running
                            match process.try_wait() {
                                Ok(Some(status)) => {
                                    warn!(exit_code = ?status.code(), "Weaver process exited unexpectedly");
                                    true
                                }
                                Ok(None) => {
                                    // Process still running, check health
                                    drop(process_guard); // Drop guard before await
                                    let weaver = WeaverLiveCheck::new()
                                        .with_admin_port(weaver_monitor_config.weaver_admin_port);
                                    match weaver.check_health() {
                                        Ok(true) => false, // Healthy
                                        Ok(false) | Err(_) => {
                                            warn!("Weaver health check failed, process may be unresponsive");
                                            false // Don't restart yet, might be transient
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(error = %e, "Error checking Weaver process status");
                                    false
                                }
                            }
                        }
                        None => false, // No process to monitor
                    }
                };

                if process_needs_restart {
                    // Check restart rate limit
                    let elapsed = last_restart_time.elapsed();
                    if elapsed.as_secs() < 60 {
                        restart_count += 1;
                        if restart_count > MAX_RESTARTS_PER_MINUTE {
                            error!("Weaver restart rate limit exceeded ({} restarts in {}s), stopping monitoring", restart_count, elapsed.as_secs());
                            break;
                        }
                    } else {
                        restart_count = 1;
                        last_restart_time = std::time::Instant::now();
                    }

                    // Attempt restart using the builder config
                    info!(
                        "Attempting to restart Weaver process (attempt {})",
                        restart_count
                    );

                    // Start process
                    let new_process_result = weaver_builder_config.start();

                    let new_process = match new_process_result {
                        Ok(process) => process,
                        Err(e) => {
                            error!(error = %e, "Failed to start Weaver process");
                            continue; // Skip this iteration
                        }
                    };

                    // Wait for initialization
                    sleep(Duration::from_secs(2)).await;

                    // Check health
                    let mut health_ok = false;
                    for _ in 1..=3 {
                        match weaver_builder_config.check_health() {
                            Ok(true) => {
                                health_ok = true;
                                break;
                            }
                            _ => {
                                sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }

                    if health_ok {
                        info!("Weaver restarted successfully");
                        let mut process_guard = match weaver_monitor_process.lock() {
                            Ok(guard) => guard,
                            Err(e) => {
                                error!("Mutex poisoned while updating process: {}", e);
                                continue;
                            }
                        };
                        *process_guard = Some(new_process);
                    } else {
                        error!("Weaver restart failed health check");
                        // Will retry on next iteration
                    }
                }
            }
        });
    }

    // Create metrics collector
    let metrics = Arc::new(MetricsCollector::new(1000));

    // Create health checker
    let health = Arc::new(HealthChecker::new(5000));

    // Create beat scheduler for 8-beat epoch system
    use knhk_etl::beat_scheduler::BeatScheduler;
    let beat_scheduler = Arc::new(Mutex::new(
        BeatScheduler::new(
            config.beat_shard_count,
            config.beat_domain_count,
            config.beat_ring_capacity,
        )
        .map_err(|e| format!("Failed to create beat scheduler: {:?}", e))?,
    ));

    // Start beat advancement task (runs continuously)
    // Note: Beat scheduler uses raw pointers and is not Send/Sync
    // We run it on the current thread to avoid thread safety issues
    // In production, this would be handled by a dedicated beat thread
    // For now, we'll skip the beat advancement task to avoid thread safety issues
    // FUTURE: Implement proper beat scheduler thread safety or use LocalSet
    info!(
        "Beat scheduler initialized (advancement task disabled due to thread safety constraints)"
    );

    info!(
        shards = config.beat_shard_count,
        domains = config.beat_domain_count,
        ring_capacity = config.beat_ring_capacity,
        "Beat scheduler initialized"
    );

    // Create beat admission manager
    use crate::beat_admission::BeatAdmission;
    let beat_admission = Arc::new(BeatAdmission::new(
        Arc::clone(&beat_scheduler),
        0, // default_domain_id
    ));

    // Create client
    let mut client_config = crate::client::ClientConfig::default();
    client_config.warm_orchestrator_url = std::env::var("KGC_SIDECAR_CLIENT_WARM_ORCHESTRATOR_URL")
        .unwrap_or_else(|_| "http://localhost:50052".to_string());
    client_config.request_timeout_ms = config.request_timeout_ms;
    client_config.retry_config.max_retries = config.retry_max_attempts;
    client_config.retry_config.initial_delay_ms = config.retry_initial_delay_ms;
    client_config.retry_config.max_delay_ms = config.retry_max_delay_ms;
    client_config.circuit_breaker_threshold = config.circuit_breaker_failure_threshold;
    client_config.circuit_breaker_reset_ms = config.circuit_breaker_reset_timeout_ms;

    let client = SidecarClient::new(client_config, Arc::clone(&metrics))
        .await
        .map_err(|e| format!("Failed to create sidecar client: {}", e))?;

    // Create server config
    let server_config = ServerConfig {
        bind_address: config.listen_address.clone(),
        batch_config: BatchConfig {
            batch_window_ms: config.batch_timeout_ms,
            max_batch_size: config.batch_size,
        },
        tls_config: TlsConfig {
            enabled: config.tls_enabled,
            cert_file: config.tls_cert_path.clone(),
            key_file: config.tls_key_path.clone(),
            ca_file: config.tls_ca_path.clone(),
            mtls_enabled: false, // mTLS not enabled by default
        },
    };

    // Create and start server with beat admission
    let server = SidecarServer::new_with_weaver(
        server_config,
        client,
        Arc::clone(&metrics),
        Arc::clone(&health),
        weaver_endpoint.clone(),
        Some(beat_admission),
    )
    .await
    .map_err(|e| format!("Failed to create sidecar server: {}", e))?;

    info!("Sidecar server starting on {}", config.listen_address);

    // Export initial telemetry to Weaver if enabled
    if let Some(ref endpoint) = weaver_endpoint {
        #[cfg(feature = "otel")]
        {
            use knhk_otel::SpanStatus;
            let mut tracer = knhk_otel::Tracer::with_otlp_exporter(endpoint.clone());
            let span_ctx = tracer.start_span("knhk.sidecar.start".to_string(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.name".to_string(),
                "sidecar.start".to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.type".to_string(),
                "system".to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.sidecar.address".to_string(),
                config.listen_address.clone(),
            );
            tracer.end_span(span_ctx, SpanStatus::Ok);

            if let Err(e) = tracer.export() {
                warn!(error = %e, "Failed to export initial telemetry to Weaver");
            }
        }
    }

    // Start server (this blocks)
    if let Err(e) = server.start().await {
        error!(error = %e, "Sidecar server failed");

        // Stop Weaver if it was started
        if config.weaver_enabled {
            let weaver = WeaverLiveCheck::new().with_admin_port(config.weaver_admin_port);
            let _ = weaver.stop();

            // Also kill the process if still running
            let mut process_guard = match weaver_process.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    error!("Mutex poisoned: {}", e);
                    return Ok(()); // Continue with cleanup
                }
            };
            if let Some(mut process) = process_guard.take() {
                let _ = process.kill();
            }
        }

        return Err(e.into());
    }

    // Stop Weaver on shutdown
    if config.weaver_enabled {
        info!("Stopping Weaver live-check");
        let weaver = WeaverLiveCheck::new().with_admin_port(config.weaver_admin_port);
        let _ = weaver.stop();

        // Also kill the process if still running
        let mut process_guard = match weaver_process.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Mutex poisoned: {}", e);
                return Ok(());
            }
        };
        if let Some(mut process) = process_guard.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }

    Ok(())
}

/// Run the sidecar server without Weaver integration (when OTEL feature disabled)
#[cfg(not(feature = "otel"))]
pub async fn run(config: SidecarConfig) -> Result<(), Box<dyn std::error::Error>> {
    use crate::batch::BatchConfig;
    use crate::health::HealthChecker;
    use crate::metrics::MetricsCollector;
    use crate::server::{ServerConfig, SidecarServer};
    use crate::tls::TlsConfig;
    use tracing::{error, info, warn};

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create metrics collector
    let metrics = Arc::new(MetricsCollector::new(1000));

    // Create health checker
    let health = Arc::new(HealthChecker::new(5000));

    // Create client
    let mut client_config = crate::client::ClientConfig::default();
    client_config.warm_orchestrator_url = std::env::var("KGC_SIDECAR_CLIENT_WARM_ORCHESTRATOR_URL")
        .unwrap_or_else(|_| "http://localhost:50052".to_string());
    client_config.request_timeout_ms = config.request_timeout_ms;
    client_config.retry_config.max_retries = config.retry_max_attempts;
    client_config.retry_config.initial_delay_ms = config.retry_initial_delay_ms;
    client_config.retry_config.max_delay_ms = config.retry_max_delay_ms;
    client_config.circuit_breaker_threshold = config.circuit_breaker_failure_threshold;
    client_config.circuit_breaker_reset_ms = config.circuit_breaker_reset_timeout_ms;

    let client = SidecarClient::new(client_config, Arc::clone(&metrics))
        .await
        .map_err(|e| format!("Failed to create sidecar client: {}", e))?;

    // Create server config
    let server_config = ServerConfig {
        bind_address: config.listen_address.clone(),
        batch_config: BatchConfig {
            batch_window_ms: config.batch_timeout_ms,
            max_batch_size: config.batch_size,
        },
        tls_config: TlsConfig {
            enabled: config.tls_enabled,
            cert_file: config.tls_cert_path.clone(),
            key_file: config.tls_key_path.clone(),
            ca_file: config.tls_ca_path.clone(),
            mtls_enabled: false, // mTLS not enabled by default
        },
    };

    // Create and start server
    let server = SidecarServer::new(
        server_config,
        client,
        Arc::clone(&metrics),
        Arc::clone(&health),
    )
    .await
    .map_err(|e| format!("Failed to create sidecar server: {}", e))?;

    // Fortune 5: Initialize SPIFFE/SPIRE if enabled
    let spiffe_manager = if config.spiffe_enabled {
        use crate::spiffe::{SpiffeCertManager, SpiffeConfig};
        let spiffe_config = SpiffeConfig {
            socket_path: config
                .spiffe_socket_path
                .clone()
                .unwrap_or_else(|| "/tmp/spire-agent/public/api.sock".to_string()),
            trust_domain: config
                .spiffe_trust_domain
                .clone()
                .unwrap_or_else(|| "example.com".to_string()),
            spiffe_id: config.spiffe_id.clone(),
            refresh_interval: std::time::Duration::from_secs(3600),
        };

        match SpiffeCertManager::new(spiffe_config) {
            Ok(mut manager) => {
                if let Err(e) = manager.load_certificate().await {
                    warn!(
                        "SPIFFE certificate loading failed: {}. Continuing without SPIFFE.",
                        e
                    );
                    None
                } else {
                    info!("SPIFFE/SPIRE integration initialized");
                    Some(manager)
                }
            }
            Err(e) => {
                warn!(
                    "SPIFFE initialization failed: {}. Continuing without SPIFFE.",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    // Fortune 5: Initialize KMS if configured
    let kms_manager = if let Some(ref provider) = config.kms_provider {
        use crate::kms::{KmsConfig, KmsProvider};
        let kms_config = match provider.as_str() {
            "aws" => {
                let region = config
                    .kms_region
                    .clone()
                    .unwrap_or_else(|| "us-east-1".to_string());
                let key_id = config.kms_key_id.clone().unwrap_or_else(|| "".to_string());
                KmsConfig::aws(region, key_id)
            }
            "azure" => {
                let vault_url = config
                    .kms_vault_url
                    .clone()
                    .unwrap_or_else(|| "".to_string());
                let key_name = config.kms_key_id.clone().unwrap_or_else(|| "".to_string());
                KmsConfig::azure(vault_url, key_name)
            }
            "vault" => {
                let addr = config
                    .kms_vault_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:8200".to_string());
                let mount_path = config
                    .kms_vault_mount
                    .clone()
                    .unwrap_or_else(|| "secret".to_string());
                let key_name = config.kms_key_id.clone().unwrap_or_else(|| "".to_string());
                KmsConfig::vault(addr, mount_path, key_name)
            }
            _ => {
                warn!(
                    "Unknown KMS provider: {}. KMS integration disabled.",
                    provider
                );
                KmsConfig::default()
            }
        };

        match crate::kms::KmsManager::new(kms_config) {
            Ok(manager) => {
                info!("KMS integration initialized");
                Some(manager)
            }
            Err(e) => {
                warn!("KMS initialization failed: {}. Continuing without KMS.", e);
                None
            }
        }
    } else {
        None
    };

    // Fortune 5: Initialize key rotation manager
    let key_rotation_manager = if kms_manager.is_some() || spiffe_manager.is_some() {
        use crate::key_rotation::KeyRotationManager;
        let rotation_interval =
            std::time::Duration::from_secs(config.key_rotation_interval_hours * 3600);

        match KeyRotationManager::new(rotation_interval) {
            Ok(manager) => {
                // Start background rotation task
                let _handle = manager.start_background_task();
                info!(
                    "Key rotation manager started (interval: {}h)",
                    config.key_rotation_interval_hours
                );
                Some(manager)
            }
            Err(e) => {
                warn!("Key rotation manager initialization failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Fortune 5: Initialize multi-region support
    let receipt_sync_manager = if config.cross_region_sync_enabled {
        use crate::multi_region::{ReceiptSyncManager, RegionConfig};
        let region_config = RegionConfig {
            region: config
                .region
                .clone()
                .unwrap_or_else(|| "us-east-1".to_string()),
            primary_region: config.primary_region.clone(),
            cross_region_sync_enabled: config.cross_region_sync_enabled,
            receipt_sync_endpoints: config.receipt_sync_endpoints.clone(),
            quorum_threshold: config.quorum_threshold,
        };

        match ReceiptSyncManager::new(region_config) {
            Ok(manager) => {
                info!("Multi-region receipt sync initialized");
                Some(manager)
            }
            Err(e) => {
                warn!(
                    "Multi-region initialization failed: {}. Continuing without multi-region.",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    // Fortune 5: Initialize SLO admission controller
    use crate::slo_admission::{AdmissionStrategy, SloAdmissionController, SloConfig};
    let slo_config = SloConfig {
        r1_p99_max_ns: config.slo_r1_p99_max_ns,
        w1_p99_max_ms: config.slo_w1_p99_max_ms,
        c1_p99_max_ms: config.slo_c1_p99_max_ms,
        admission_strategy: if config.slo_admission_strategy == "degrade" {
            AdmissionStrategy::Degrade
        } else {
            AdmissionStrategy::Strict
        },
    };

    let slo_controller = match SloAdmissionController::new(slo_config) {
        Ok(controller) => {
            info!("SLO admission controller initialized");
            controller
        }
        Err(e) => {
            return Err(format!("SLO admission controller initialization failed: {}", e).into());
        }
    };

    // Fortune 5: Initialize capacity manager
    use crate::capacity::CapacityManager;
    let capacity_manager = CapacityManager::new(0.95); // 95% cache hit rate threshold
    info!("Capacity manager initialized");

    // Fortune 5: Initialize promotion gate manager
    use crate::promotion::{Environment, PromotionConfig, PromotionGateManager};
    let promotion_config = PromotionConfig {
        environment: match config.promotion_environment.as_deref() {
            Some("canary") => {
                let traffic = config.promotion_traffic_percent.unwrap_or(10.0);
                Environment::Canary {
                    traffic_percent: traffic,
                }
            }
            Some("staging") => Environment::Staging,
            Some("production") | None => Environment::Production,
            Some(env) => {
                warn!(
                    "Unknown promotion environment: {}. Defaulting to production.",
                    env
                );
                Environment::Production
            }
        },
        feature_flags: Vec::new(), // Can be populated from config
        auto_rollback_enabled: config.auto_rollback_enabled,
        slo_threshold: config.slo_threshold,
        rollback_window_seconds: 300,
    };

    let promotion_manager = match PromotionGateManager::new(promotion_config, slo_controller) {
        Ok(manager) => {
            info!("Promotion gate manager initialized");
            manager
        }
        Err(e) => {
            return Err(format!("Promotion gate manager initialization failed: {}", e).into());
        }
    };

    info!(
        "Fortune 5 features initialized: SPIFFE={}, KMS={}, Multi-Region={}, SLO={}, Promotion={}",
        config.spiffe_enabled,
        config.kms_provider.is_some(),
        config.cross_region_sync_enabled,
        true,
        true
    );

    // Fortune 5: Initialize SPIFFE/SPIRE if enabled
    let _spiffe_manager = if config.spiffe_enabled {
        use crate::spiffe::{SpiffeCertManager, SpiffeConfig};
        let spiffe_config = SpiffeConfig {
            socket_path: config
                .spiffe_socket_path
                .clone()
                .unwrap_or_else(|| "/tmp/spire-agent/public/api.sock".to_string()),
            trust_domain: config
                .spiffe_trust_domain
                .clone()
                .unwrap_or_else(|| "example.com".to_string()),
            spiffe_id: config.spiffe_id.clone(),
            refresh_interval: std::time::Duration::from_secs(3600),
        };

        match SpiffeCertManager::new(spiffe_config) {
            Ok(mut manager) => {
                if let Err(e) = manager.load_certificate().await {
                    warn!(
                        "SPIFFE certificate loading failed: {}. Continuing without SPIFFE.",
                        e
                    );
                    None
                } else {
                    info!("SPIFFE/SPIRE integration initialized");
                    Some(manager)
                }
            }
            Err(e) => {
                warn!(
                    "SPIFFE initialization failed: {}. Continuing without SPIFFE.",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    // Fortune 5: Initialize KMS if configured
    let _kms_manager = if let Some(ref provider) = config.kms_provider {
        use crate::kms::{KmsConfig, KmsProvider};
        let kms_config = match provider.as_str() {
            "aws" => {
                let region = config
                    .kms_region
                    .clone()
                    .unwrap_or_else(|| "us-east-1".to_string());
                let key_id = config.kms_key_id.clone().unwrap_or_else(|| "".to_string());
                KmsConfig::aws(region, key_id)
            }
            "azure" => {
                let vault_url = config
                    .kms_vault_url
                    .clone()
                    .unwrap_or_else(|| "".to_string());
                let key_name = config.kms_key_id.clone().unwrap_or_else(|| "".to_string());
                KmsConfig::azure(vault_url, key_name)
            }
            "vault" => {
                let addr = config
                    .kms_vault_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:8200".to_string());
                let mount_path = config
                    .kms_vault_mount
                    .clone()
                    .unwrap_or_else(|| "secret".to_string());
                let key_name = config.kms_key_id.clone().unwrap_or_else(|| "".to_string());
                KmsConfig::vault(addr, mount_path, key_name)
            }
            _ => {
                warn!(
                    "Unknown KMS provider: {}. KMS integration disabled.",
                    provider
                );
                KmsConfig::default()
            }
        };

        match crate::kms::KmsManager::new(kms_config) {
            Ok(manager) => {
                info!("KMS integration initialized");
                Some(manager)
            }
            Err(e) => {
                warn!("KMS initialization failed: {}. Continuing without KMS.", e);
                None
            }
        }
    } else {
        None
    };

    // Fortune 5: Initialize key rotation manager
    let _key_rotation_manager = if _kms_manager.is_some() || _spiffe_manager.is_some() {
        use crate::key_rotation::KeyRotationManager;
        let rotation_interval =
            std::time::Duration::from_secs(config.key_rotation_interval_hours * 3600);

        match KeyRotationManager::new(rotation_interval) {
            Ok(manager) => {
                let _handle = manager.start_background_task();
                info!(
                    "Key rotation manager started (interval: {}h)",
                    config.key_rotation_interval_hours
                );
                Some(manager)
            }
            Err(e) => {
                warn!("Key rotation manager initialization failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Fortune 5: Initialize multi-region support
    let _receipt_sync_manager = if config.cross_region_sync_enabled {
        use crate::multi_region::{ReceiptSyncManager, RegionConfig};
        let region_config = RegionConfig {
            region: config
                .region
                .clone()
                .unwrap_or_else(|| "us-east-1".to_string()),
            primary_region: config.primary_region.clone(),
            cross_region_sync_enabled: config.cross_region_sync_enabled,
            receipt_sync_endpoints: config.receipt_sync_endpoints.clone(),
            quorum_threshold: config.quorum_threshold,
        };

        match ReceiptSyncManager::new(region_config) {
            Ok(manager) => {
                info!("Multi-region receipt sync initialized");
                Some(manager)
            }
            Err(e) => {
                warn!(
                    "Multi-region initialization failed: {}. Continuing without multi-region.",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    // Fortune 5: Initialize SLO admission controller
    use crate::slo_admission::{AdmissionStrategy, SloAdmissionController, SloConfig};
    let slo_config = SloConfig {
        r1_p99_max_ns: config.slo_r1_p99_max_ns,
        w1_p99_max_ms: config.slo_w1_p99_max_ms,
        c1_p99_max_ms: config.slo_c1_p99_max_ms,
        admission_strategy: if config.slo_admission_strategy == "degrade" {
            AdmissionStrategy::Degrade
        } else {
            AdmissionStrategy::Strict
        },
    };

    let slo_controller = match SloAdmissionController::new(slo_config) {
        Ok(controller) => {
            info!("SLO admission controller initialized");
            controller
        }
        Err(e) => {
            return Err(format!("SLO admission controller initialization failed: {}", e).into());
        }
    };

    // Fortune 5: Initialize capacity manager
    use crate::capacity::CapacityManager;
    let _capacity_manager = CapacityManager::new(0.95);
    info!("Capacity manager initialized");

    // Fortune 5: Initialize promotion gate manager
    use crate::promotion::{Environment, PromotionConfig, PromotionGateManager};
    let promotion_config = PromotionConfig {
        environment: match config.promotion_environment.as_deref() {
            Some("canary") => {
                let traffic = config.promotion_traffic_percent.unwrap_or(10.0);
                Environment::Canary {
                    traffic_percent: traffic,
                }
            }
            Some("staging") => Environment::Staging,
            Some("production") | None => Environment::Production,
            Some(env) => {
                warn!(
                    "Unknown promotion environment: {}. Defaulting to production.",
                    env
                );
                Environment::Production
            }
        },
        feature_flags: Vec::new(),
        auto_rollback_enabled: config.auto_rollback_enabled,
        slo_threshold: config.slo_threshold,
        rollback_window_seconds: 300,
    };

    let _promotion_manager = match PromotionGateManager::new(promotion_config, slo_controller) {
        Ok(manager) => {
            info!("Promotion gate manager initialized");
            manager
        }
        Err(e) => {
            return Err(format!("Promotion gate manager initialization failed: {}", e).into());
        }
    };

    info!(
        "Fortune 5 features initialized: SPIFFE={}, KMS={}, Multi-Region={}, SLO={}, Promotion={}",
        config.spiffe_enabled,
        config.kms_provider.is_some(),
        config.cross_region_sync_enabled,
        true,
        true
    );

    info!("Sidecar server starting on {}", config.listen_address);
    server.start().await.map_err(|e| e.into())
}
