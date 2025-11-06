// rust/knhk-sidecar/src/server.rs
// gRPC server implementation

use crate::config::SidecarConfig;
use crate::service::KgcSidecarService;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{error, info};

pub async fn start_server(config: SidecarConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = config.listen_address.parse()
        .map_err(|e| format!("Invalid listen address: {}", e))?;

    info!("Starting KGC Sidecar server on {}", addr);

    let service = KgcSidecarService::new(config.clone());

    let mut server_builder = Server::builder();

    // Add TLS if enabled
    let server = if config.tls_enabled {
        // TODO: Load TLS certificates
        // For now, start without TLS
        warn!("TLS enabled but not configured, starting without TLS");
        server_builder
    } else {
        server_builder
    };

    server
        .add_service(
            crate::service::proto::kgc_sidecar_server::KgcSidecarServer::new(service)
        )
        .serve(addr)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

pub async fn run(config: SidecarConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    start_server(config).await
}

