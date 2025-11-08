// knhk-sidecar: gRPC server implementation

use crate::batch::BatchConfig;
use crate::client::SidecarClient;
use crate::config::SidecarConfig;
use crate::error::{SidecarError, SidecarResult};
use crate::health::HealthChecker;
use crate::metrics::{LatencyTimer, MetricsCollector};
use crate::service::KgcSidecarService;
use crate::tls::{create_tls_server_config, TlsConfig};
use std::sync::Arc;

/// Server configuration for sidecar gRPC server
///
/// Contains all configuration needed to start and run the sidecar server,
/// including TLS, batching, retry, and beat admission settings.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address
    pub bind_address: String,

    /// Batch configuration
    pub batch_config: BatchConfig,

    /// TLS configuration
    pub tls_config: TlsConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:50051".to_string(),
            batch_config: BatchConfig::default(),
            tls_config: TlsConfig::default(),
        }
    }
}

/// Sidecar gRPC server
///
/// Main server struct that manages the gRPC service, client connections, metrics,
/// health checks, and beat-driven admission control.
///
/// # Features
///
/// - gRPC service with beat-driven admission
/// - TLS/mTLS support
/// - Request batching and retries
/// - Circuit breaking
/// - OTEL telemetry integration
/// - Weaver live-check validation
///
/// # Example
///
/// ```rust
/// use knhk_sidecar::{SidecarServer, SidecarConfig};
///
/// let config = SidecarConfig::default();
/// let server = SidecarServer::new(server_config, client, metrics, health).await?;
/// server.start().await?;
/// ```
pub struct SidecarServer {
    config: ServerConfig,
    client: Arc<SidecarClient>,
    metrics: Arc<MetricsCollector>,
    health: Arc<HealthChecker>,
    #[cfg(feature = "otel")]
    weaver_endpoint: Option<String>,
    /// Beat admission manager for 8-beat epoch system
    beat_admission: Option<Arc<crate::beat_admission::BeatAdmission>>,
}

impl SidecarServer {
    /// Create new sidecar server
    pub async fn new(
        server_config: ServerConfig,
        client: SidecarClient,
        metrics: Arc<MetricsCollector>,
        health: Arc<HealthChecker>,
    ) -> SidecarResult<Self> {
        Self::new_with_weaver(server_config, client, metrics, health, None, None).await
    }

    /// Create new sidecar server with Weaver endpoint and beat admission
    #[cfg(feature = "otel")]
    pub async fn new_with_weaver(
        server_config: ServerConfig,
        client: SidecarClient,
        metrics: Arc<MetricsCollector>,
        health: Arc<HealthChecker>,
        weaver_endpoint: Option<String>,
        beat_admission: Option<Arc<crate::beat_admission::BeatAdmission>>,
    ) -> SidecarResult<Self> {
        Ok(Self {
            config: server_config,
            client: Arc::new(client),
            metrics,
            health,
            weaver_endpoint,
            beat_admission,
        })
    }

    #[cfg(not(feature = "otel"))]
    pub async fn new_with_weaver(
        server_config: ServerConfig,
        client: SidecarClient,
        metrics: Arc<MetricsCollector>,
        health: Arc<HealthChecker>,
        _weaver_endpoint: Option<String>,
        beat_admission: Option<Arc<crate::beat_admission::BeatAdmission>>,
    ) -> SidecarResult<Self> {
        Ok(Self {
            config: server_config,
            client: Arc::new(client),
            metrics,
            health,
            beat_admission,
        })
    }

    /// Start server
    pub async fn start(&self) -> SidecarResult<()> {
        let addr = self
            .config
            .bind_address
            .parse()
            .map_err(|e| SidecarError::config_error(format!("Invalid bind address: {}", e)))?;

        // Create gRPC server builder
        let mut server_builder = tonic::transport::Server::builder();

        // Configure TLS if enabled
        if self.config.tls_config.enabled {
            let tls_config = create_tls_server_config(&self.config.tls_config)?;
            server_builder = server_builder
                .tls_config(tls_config)
                .map_err(|e| SidecarError::tls_error(format!("Failed to configure TLS: {}", e)))?;
        }

        // Create service with beat admission
        use crate::service::KgcSidecarService;
        let service = KgcSidecarService::new_with_weaver(
            self.config.clone(), // Pass actual config from server
            self.weaver_endpoint.clone(),
            self.beat_admission.clone(),
        );

        // Add service to server
        server_builder
            .add_service(crate::service::proto::kgc_sidecar_server::KgcSidecarServer::new(service))
            .serve(addr)
            .await
            .map_err(|e| SidecarError::internal_error(format!("Server failed to start: {}", e)))?;

        Ok(())
    }

    /// Handle execute transaction request
    pub async fn handle_execute_transaction(&self, rdf_delta: String) -> SidecarResult<String> {
        let _timer = LatencyTimer::start(Arc::clone(&self.metrics));

        // Forward to warm orchestrator
        self.client.execute_transaction(rdf_delta).await
    }

    /// Handle validate graph request
    pub async fn handle_validate_graph(
        &self,
        graph: String,
        schema_iri: String,
    ) -> SidecarResult<bool> {
        let _timer = LatencyTimer::start(Arc::clone(&self.metrics));

        self.client.validate_graph(graph, schema_iri).await
    }

    /// Handle evaluate hook request
    pub async fn handle_evaluate_hook(
        &self,
        hook_id: String,
        input_data: String,
    ) -> SidecarResult<String> {
        let _timer = LatencyTimer::start(Arc::clone(&self.metrics));

        self.client.evaluate_hook(hook_id, input_data).await
    }

    /// Handle health check request
    pub fn handle_health_check(&self, check_type: String) -> (bool, String) {
        use crate::health::HealthStatus;

        let status = match check_type.as_str() {
            "liveness" => self.health.check_liveness(),
            "readiness" => self.health.check_readiness(),
            _ => self.health.check_readiness(),
        };

        match status {
            HealthStatus::Healthy => (true, "Service is healthy".to_string()),
            HealthStatus::Degraded(reason) => (true, format!("Service is degraded: {}", reason)),
            HealthStatus::Unhealthy(reason) => (false, format!("Service is unhealthy: {}", reason)),
        }
    }

    /// Handle get metrics request
    pub fn handle_get_metrics(&self) -> crate::metrics::MetricsSnapshot {
        self.metrics.snapshot()
    }
}
