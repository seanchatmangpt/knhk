// knhk-sidecar: gRPC server implementation

use std::sync::Arc;
use crate::error::{SidecarError, SidecarResult};
use crate::client::SidecarClient;
use crate::batch::BatchConfig;
use crate::tls::{TlsConfig, create_tls_server_config};
use crate::metrics::{MetricsCollector, LatencyTimer};
use crate::health::HealthChecker;
use crate::service::KgcSidecarService;
use crate::config::SidecarConfig;

/// Server configuration
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

/// Sidecar server
pub struct SidecarServer {
    config: ServerConfig,
    client: Arc<SidecarClient>,
    metrics: Arc<MetricsCollector>,
    health: Arc<HealthChecker>,
    #[cfg(feature = "otel")]
    weaver_endpoint: Option<String>,
}

impl SidecarServer {
    /// Create new sidecar server
    pub async fn new(
        server_config: ServerConfig,
        client: SidecarClient,
        metrics: Arc<MetricsCollector>,
        health: Arc<HealthChecker>,
    ) -> SidecarResult<Self> {
        Self::new_with_weaver(server_config, client, metrics, health, None).await
    }

    /// Create new sidecar server with Weaver endpoint
    #[cfg(feature = "otel")]
    pub async fn new_with_weaver(
        server_config: ServerConfig,
        client: SidecarClient,
        metrics: Arc<MetricsCollector>,
        health: Arc<HealthChecker>,
        weaver_endpoint: Option<String>,
    ) -> SidecarResult<Self> {
        Ok(Self {
            config: server_config,
            client: Arc::new(client),
            metrics,
            health,
            weaver_endpoint,
        })
    }

    #[cfg(not(feature = "otel"))]
    pub async fn new_with_weaver(
        server_config: ServerConfig,
        client: SidecarClient,
        metrics: Arc<MetricsCollector>,
        health: Arc<HealthChecker>,
        _weaver_endpoint: Option<String>,
    ) -> SidecarResult<Self> {
        Ok(Self {
            config: server_config,
            client: Arc::new(client),
            metrics,
            health,
        })
    }

    /// Start server
    pub async fn start(&self) -> SidecarResult<()> {
        let addr = self.config.bind_address.parse()
            .map_err(|e| SidecarError::ConfigError(format!("Invalid bind address: {}", e)))?;

        // Create gRPC server builder
        let mut server_builder = tonic::transport::Server::builder();

        // Configure TLS if enabled
        if self.config.tls_config.enabled {
            let tls_config = create_tls_server_config(&self.config.tls_config)?;
            server_builder = server_builder.tls_config(tls_config)
                .map_err(|e| SidecarError::TlsError(format!("Failed to configure TLS: {}", e)))?;
        }

        // Register health component
        self.health.register_component("warm_orchestrator".to_string());
        self.health.update_component(
            "warm_orchestrator",
            crate::health::HealthStatus::Healthy,
            "Connected".to_string(),
        );

        // Create service implementation
        // Use default config for service (can be enhanced to pass config)
        let service_config = SidecarConfig::default();
        #[cfg(feature = "otel")]
        let service = KgcSidecarService::new_with_weaver(service_config, self.weaver_endpoint.clone());
        #[cfg(not(feature = "otel"))]
        let service = KgcSidecarService::new(service_config);

        // Include generated proto code
        use crate::service::proto::kgc_sidecar_server::KgcSidecarServer;

        tracing::info!("Sidecar server starting on {}", addr);
        
        // Start the gRPC server
        server_builder
            .add_service(KgcSidecarServer::new(service))
            .serve(addr)
            .await
            .map_err(|e| SidecarError::NetworkError(format!("Failed to start server: {}", e)))?;

        tracing::info!("Sidecar server started successfully on {}", addr);
        Ok(())
    }

    /// Handle execute transaction request
    pub async fn handle_execute_transaction(&self, rdf_delta: String) -> SidecarResult<String> {
        let _timer = LatencyTimer::start(Arc::clone(&self.metrics));
        
        // Forward to warm orchestrator
        self.client.execute_transaction(rdf_delta).await
    }

    /// Handle validate graph request
    pub async fn handle_validate_graph(&self, graph: String, schema_iri: String) -> SidecarResult<bool> {
        let _timer = LatencyTimer::start(Arc::clone(&self.metrics));
        
        self.client.validate_graph(graph, schema_iri).await
    }

    /// Handle evaluate hook request
    pub async fn handle_evaluate_hook(&self, hook_id: String, input_data: String) -> SidecarResult<String> {
        let _timer = LatencyTimer::start(Arc::clone(&self.metrics));
        
        self.client.evaluate_hook(hook_id, input_data).await
    }

    /// Handle health check request
    pub fn handle_health_check(&self, check_type: String) -> (bool, String) {
        match check_type.as_str() {
            "liveness" => self.health.check_liveness(),
            "readiness" => self.health.check_readiness(),
            _ => self.health.check_readiness(),
        }
    }

    /// Handle get metrics request
    pub fn handle_get_metrics(&self) -> crate::metrics::MetricsSnapshot {
        self.metrics.snapshot()
    }
}


