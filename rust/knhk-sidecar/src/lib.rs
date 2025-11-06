// knhk-sidecar: KGC Sidecar Service
// gRPC proxy for enterprise apps with batching, retries, circuit-breaking, and TLS

pub mod error;
pub mod batch;
pub mod retry;
pub mod circuit_breaker;
pub mod tls;
pub mod metrics;
pub mod health;
pub mod client;
pub mod server;
pub mod config;
pub mod service;

pub use error::{SidecarError, SidecarResult};
pub use server::SidecarServer;
pub use client::SidecarClient;
pub use config::SidecarConfig;

use std::sync::Arc;
use std::process::Child;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

/// Run the sidecar server with Weaver live-check integration
#[cfg(feature = "otel")]
pub async fn run(config: SidecarConfig) -> Result<(), Box<dyn std::error::Error>> {
    use crate::metrics::MetricsCollector;
    use crate::health::HealthChecker;
    use crate::server::{SidecarServer, ServerConfig};
    use crate::batch::BatchConfig;
    use crate::tls::TlsConfig;
    use knhk_otel::WeaverLiveCheck;
    use tracing::{info, error, warn};

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Validate Weaver configuration if enabled
    if config.weaver_enabled {
        config.validate_weaver_config()
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
                *weaver_process.lock().await = Some(process);
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
                
                let mut process_guard = weaver_monitor_process.lock().await;
                let process_needs_restart = match process_guard.as_mut() {
                    Some(process) => {
                        // Check if process is still running
                        match process.try_wait() {
                            Ok(Some(status)) => {
                                warn!(exit_code = ?status.code(), "Weaver process exited unexpectedly");
                                true
                            }
                            Ok(None) => {
                                // Process still running, check health
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
                    info!("Attempting to restart Weaver process (attempt {})", restart_count);
                    
                    // Start process
                    match weaver_builder_config.start() {
                        Ok(new_process) => {
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
                                *process_guard = Some(new_process);
                            } else {
                                error!("Weaver restart failed health check");
                                // Will retry on next iteration
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to restart Weaver process");
                            // Keep trying on next iteration
                        }
                    }
                }
            }
        });
    }

    // Create metrics collector
    let metrics = Arc::new(MetricsCollector::new(1000));

    // Create health checker
    let health = Arc::new(HealthChecker::new());

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
    
    let client = SidecarClient::new(client_config, Arc::clone(&metrics)).await
        .map_err(|e| format!("Failed to create sidecar client: {}", e))?;

    // Create server config
    let server_config = ServerConfig {
        bind_address: config.listen_address.clone(),
        batch_config: BatchConfig {
            max_size: config.batch_size,
            timeout: config.batch_timeout(),
        },
        tls_config: TlsConfig {
            enabled: config.tls_enabled,
            cert_file: config.tls_cert_path.clone().unwrap_or_default(),
            key_file: config.tls_key_path.clone().unwrap_or_default(),
            ca_file: config.tls_ca_path.clone(),
        },
    };

    // Create and start server
    let server = SidecarServer::new(
        server_config,
        client,
        Arc::clone(&metrics),
        Arc::clone(&health),
    ).await
        .map_err(|e| format!("Failed to create sidecar server: {}", e))?;

    info!("Sidecar server starting on {}", config.listen_address);
    
    // Export initial telemetry to Weaver if enabled
    if let Some(ref endpoint) = weaver_endpoint {
        #[cfg(feature = "otel")]
        {
            use knhk_otel::{Tracer, SpanStatus};
            let mut tracer = knhk_otel::Tracer::with_otlp_exporter(endpoint.clone());
            let span_ctx = tracer.start_span("knhk.sidecar.start".to_string(), None);
            tracer.add_attribute(span_ctx.clone(), "knhk.operation.name".to_string(), "sidecar.start".to_string());
            tracer.add_attribute(span_ctx.clone(), "knhk.operation.type".to_string(), "system".to_string());
            tracer.add_attribute(span_ctx.clone(), "knhk.sidecar.address".to_string(), config.listen_address.clone());
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
            let weaver = WeaverLiveCheck::new()
                .with_admin_port(config.weaver_admin_port);
            let _ = weaver.stop();
            
            // Also kill the process if still running
            let mut process_guard = weaver_process.lock().await;
            if let Some(mut process) = process_guard.take() {
                let _ = process.kill();
            }
        }
        
        return Err(e.into());
    }

    // Stop Weaver on shutdown
    if config.weaver_enabled {
        info!("Stopping Weaver live-check");
        let weaver = WeaverLiveCheck::new()
            .with_admin_port(config.weaver_admin_port);
        let _ = weaver.stop();
        
        // Also kill the process if still running
        let mut process_guard = weaver_process.lock().await;
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
    use crate::metrics::MetricsCollector;
    use crate::health::HealthChecker;
    use crate::server::{SidecarServer, ServerConfig};
    use crate::batch::BatchConfig;
    use crate::tls::TlsConfig;
    use tracing::{info, error};

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create metrics collector
    let metrics = Arc::new(MetricsCollector::new(1000));

    // Create health checker
    let health = Arc::new(HealthChecker::new());

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
    
    let client = SidecarClient::new(client_config, Arc::clone(&metrics)).await
        .map_err(|e| format!("Failed to create sidecar client: {}", e))?;

    // Create server config
    let server_config = ServerConfig {
        bind_address: config.listen_address.clone(),
        batch_config: BatchConfig {
            max_size: config.batch_size,
            timeout: config.batch_timeout(),
        },
        tls_config: TlsConfig {
            enabled: config.tls_enabled,
            cert_file: config.tls_cert_path.clone().unwrap_or_default(),
            key_file: config.tls_key_path.clone().unwrap_or_default(),
            ca_file: config.tls_ca_path.clone(),
        },
    };

    // Create and start server
    let server = SidecarServer::new(
        server_config,
        client,
        Arc::clone(&metrics),
        Arc::clone(&health),
    ).await
        .map_err(|e| format!("Failed to create sidecar server: {}", e))?;

    info!("Sidecar server starting on {}", config.listen_address);
    server.start().await.map_err(|e| e.into())
}

