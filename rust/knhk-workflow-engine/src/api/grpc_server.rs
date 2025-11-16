//! gRPC server startup and management
//!
//! Provides server lifecycle management for the gRPC API.

use crate::api::grpc::{GrpcService, WorkflowEngineServiceServer};
use crate::executor::WorkflowEngine;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{info, warn};

/// gRPC server configuration
#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
    /// Address to bind to (e.g., "0.0.0.0:50051")
    pub bind_addr: SocketAddr,
    /// Maximum concurrent streams per connection
    pub max_concurrent_streams: Option<u32>,
    /// TCP keepalive interval
    pub tcp_keepalive: Option<std::time::Duration>,
    /// TCP nodelay
    pub tcp_nodelay: bool,
    /// Enable HTTP/2 adaptive window
    pub http2_adaptive_window: bool,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:50051".parse().expect("Valid socket address"),
            max_concurrent_streams: Some(100),
            tcp_keepalive: Some(std::time::Duration::from_secs(60)),
            tcp_nodelay: true,
            http2_adaptive_window: true,
        }
    }
}

/// gRPC server instance
pub struct GrpcServer {
    config: GrpcServerConfig,
    engine: Arc<WorkflowEngine>,
}

impl GrpcServer {
    /// Create a new gRPC server
    pub fn new(engine: Arc<WorkflowEngine>, config: GrpcServerConfig) -> Self {
        Self { config, engine }
    }

    /// Create a gRPC server with default configuration
    pub fn with_defaults(engine: Arc<WorkflowEngine>) -> Self {
        Self::new(engine, GrpcServerConfig::default())
    }

    /// Start the gRPC server
    ///
    /// This method blocks until the server is shutdown.
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.config.bind_addr;

        info!("Starting gRPC server on {}", addr);

        // Create gRPC service
        let service = GrpcService::new(self.engine.clone());
        let svc = WorkflowEngineServiceServer::new(service);

        // Build server with configuration
        let mut server = Server::builder();

        // Apply configuration
        if let Some(max_concurrent_streams) = self.config.max_concurrent_streams {
            server = server.max_concurrent_streams(max_concurrent_streams);
        }

        if let Some(tcp_keepalive) = self.config.tcp_keepalive {
            server = server.tcp_keepalive(Some(tcp_keepalive));
        }

        server = server.tcp_nodelay(self.config.tcp_nodelay);

        if self.config.http2_adaptive_window {
            server = server.http2_adaptive_window(true);
        }

        // Start server
        info!("gRPC server listening on {}", addr);
        server
            .add_service(svc)
            .serve(addr)
            .await
            .map_err(|e| {
                warn!("gRPC server error: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        info!("gRPC server shutdown");
        Ok(())
    }

    /// Start the gRPC server with graceful shutdown on CTRL-C
    pub async fn serve_with_shutdown(
        self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.config.bind_addr;

        info!("Starting gRPC server on {} (with graceful shutdown)", addr);

        // Create gRPC service
        let service = GrpcService::new(self.engine.clone());
        let svc = WorkflowEngineServiceServer::new(service);

        // Build server with configuration
        let mut server = Server::builder();

        // Apply configuration
        if let Some(max_concurrent_streams) = self.config.max_concurrent_streams {
            server = server.max_concurrent_streams(max_concurrent_streams);
        }

        if let Some(tcp_keepalive) = self.config.tcp_keepalive {
            server = server.tcp_keepalive(Some(tcp_keepalive));
        }

        server = server.tcp_nodelay(self.config.tcp_nodelay);

        if self.config.http2_adaptive_window {
            server = server.http2_adaptive_window(true);
        }

        // Setup graceful shutdown
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn CTRL-C handler
        tokio::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    info!("Received CTRL-C, initiating graceful shutdown");
                    let _ = tx.send(());
                }
                Err(e) => {
                    warn!("Failed to listen for CTRL-C: {}", e);
                }
            }
        });

        // Start server with shutdown signal
        info!("gRPC server listening on {}", addr);
        server
            .add_service(svc)
            .serve_with_shutdown(addr, async {
                rx.await.ok();
            })
            .await
            .map_err(|e| {
                warn!("gRPC server error: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

        info!("gRPC server shutdown gracefully");
        Ok(())
    }

    /// Get the configured bind address
    pub fn bind_addr(&self) -> SocketAddr {
        self.config.bind_addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateStore;
    use crate::WorkflowEngine;

    #[tokio::test]
    async fn test_grpc_server_creation() {
        let state_store = StateStore::new_in_memory();
        let engine = WorkflowEngine::new(state_store);
        let server = GrpcServer::with_defaults(Arc::new(engine));

        assert_eq!(server.bind_addr().port(), 50051);
    }

    #[tokio::test]
    async fn test_grpc_server_custom_config() {
        let state_store = StateStore::new_in_memory();
        let engine = WorkflowEngine::new(state_store);

        let config = GrpcServerConfig {
            bind_addr: "127.0.0.1:8080".parse().unwrap(),
            max_concurrent_streams: Some(50),
            tcp_keepalive: Some(std::time::Duration::from_secs(30)),
            tcp_nodelay: false,
            http2_adaptive_window: false,
        };

        let server = GrpcServer::new(Arc::new(engine), config);
        assert_eq!(server.bind_addr().port(), 8080);
    }
}
