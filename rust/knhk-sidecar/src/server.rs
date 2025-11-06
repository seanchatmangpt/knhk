// rust/knhk-sidecar/src/server.rs
// gRPC server for KGC Sidecar

use crate::error::{Result, SidecarError};
use crate::batching::{BatchConfig, BatchManager};
use crate::retry::{RetryConfig, RetryExecutor};
use crate::warm_client::{WarmClient, WarmClientConfig};
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

// Generated protobuf code will be available here after build
// For now, we'll use placeholder types
// use knhk_sidecar_v1::kgc_sidecar_server::{KgcSidecar, KgcSidecarServer};
// use knhk_sidecar_v1::{Transaction, TransactionResponse, Query, QueryResponse, HealthCheckRequest, HealthCheckResponse};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server listen address (e.g., "0.0.0.0:50051")
    pub listen_addr: String,
    /// Batch configuration
    pub batch_config: BatchConfig,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Warm client configuration
    pub warm_client_config: WarmClientConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:50051".to_string(),
            batch_config: BatchConfig::default(),
            retry_config: RetryConfig::default(),
            warm_client_config: WarmClientConfig::default(),
        }
    }
}

/// KGC Sidecar gRPC service implementation
pub struct KgcSidecarService {
    batch_manager: Arc<BatchManager>,
    retry_executor: Arc<RetryExecutor>,
    warm_client: Arc<WarmClient>,
    flush_rx: mpsc::Receiver<Vec<Vec<u8>>>,
}

impl KgcSidecarService {
    /// Create new sidecar service
    pub async fn new(config: ServerConfig) -> Result<Self> {
        // Create batch manager
        let (batch_manager, flush_rx) = BatchManager::new(config.batch_config.clone())?;
        let batch_manager = Arc::new(batch_manager);
        
        // Start flush task
        batch_manager.start_flush_task();
        
        // Create retry executor
        let retry_executor = Arc::new(RetryExecutor::new(config.retry_config.clone())?);
        
        // Create warm client
        let warm_client = Arc::new(WarmClient::new(config.warm_client_config.clone()).await?);
        
        Ok(Self {
            batch_manager,
            retry_executor,
            warm_client,
            flush_rx,
        })
    }

    /// Start background batch processing task
    pub fn start_batch_processor(&self) -> tokio::task::JoinHandle<()> {
        let warm_client = Arc::clone(&self.warm_client);
        let retry_executor = Arc::clone(&self.retry_executor);
        let mut flush_rx = self.flush_rx.clone();
        
        tokio::spawn(async move {
            while let Some(batch) = flush_rx.recv().await {
                // Process batch with retry logic
                let _ = retry_executor.execute_async(|| {
                    let warm_client = Arc::clone(&warm_client);
                    let batch = batch.clone();
                    async move {
                        warm_client.submit_batch(batch).await
                    }
                }).await;
            }
        })
    }
}

// Placeholder implementation - will be replaced with actual protobuf-generated code
// when build.rs generates the code from proto/knhk_sidecar.proto

/*
#[tonic::async_trait]
impl KgcSidecar for KgcSidecarService {
    async fn submit_transaction(
        &self,
        request: Request<Transaction>,
    ) -> std::result::Result<Response<TransactionResponse>, Status> {
        let transaction = request.into_inner();
        
        // Serialize transaction
        let serialized = transaction.encode_to_vec();
        
        // Add to batch
        match self.batch_manager.add(serialized) {
            Ok(_) => {
                // Return success response (transaction will be processed asynchronously)
                let response = TransactionResponse {
                    receipt: None,
                    success: true,
                    error: None,
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                Err(Status::invalid_argument(format!("Failed to add to batch: {}", e)))
            }
        }
    }

    async fn submit_query(
        &self,
        request: Request<Query>,
    ) -> std::result::Result<Response<QueryResponse>, Status> {
        let query = request.into_inner();
        
        // Execute query with retry logic
        let result = self.retry_executor.execute_async(|| {
            let warm_client = Arc::clone(&self.warm_client);
            let sparql = query.sparql.clone();
            let schema = query.schema.clone();
            async move {
                warm_client.submit_query(&sparql, schema.as_deref()).await
            }
        }).await;
        
        match result {
            Ok(serialized_response) => {
                // Deserialize response
                let response = QueryResponse::decode(&*serialized_response)
                    .map_err(|e| Status::internal(format!("Failed to decode response: {}", e)))?;
                Ok(Response::new(response))
            }
            Err(e) => {
                let response = QueryResponse {
                    bindings: vec![],
                    success: false,
                    error: Some(e.to_string()),
                };
                Ok(Response::new(response))
            }
        }
    }

    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> std::result::Result<Response<HealthCheckResponse>, Status> {
        // Check circuit breaker state
        let cb_state = self.warm_client.circuit_breaker_state()
            .map_err(|e| Status::internal(format!("Failed to get circuit breaker state: {}", e)))?;
        
        let healthy = matches!(cb_state, crate::circuit_breaker::CircuitBreakerState::Closed);
        
        let response = HealthCheckResponse {
            healthy,
            message: if healthy {
                None
            } else {
                Some("Circuit breaker is not closed".to_string())
            },
        };
        
        Ok(Response::new(response))
    }
}
*/

/// Start gRPC server
pub async fn start_server(config: ServerConfig) -> Result<()> {
    // Create service
    let service = KgcSidecarService::new(config.clone()).await?;
    
    // Start batch processor
    service.start_batch_processor();
    
    // Build server
    let addr = config.listen_addr.parse()
        .map_err(|e| SidecarError::ValidationFailed(
            format!("Invalid listen address: {}", e)
        ))?;
    
    // Placeholder: actual server will be created when protobuf code is generated
    // For now, return error indicating build step needed
    Err(SidecarError::InternalError(
        "gRPC server implementation requires protobuf code generation. \
         Run 'cargo build' to generate code from proto/knhk_sidecar.proto, \
         then uncomment the server implementation in src/server.rs".to_string()
    ))
    
    /*
    // Actual server implementation (uncomment after protobuf generation):
    let server = KgcSidecarServer::new(service);
    
    Server::builder()
        .add_service(server)
        .serve(addr)
        .await
        .map_err(|e| SidecarError::NetworkError(
            format!("Failed to start server: {}", e)
        ))?;
    
    Ok(())
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_config() {
        let config = ServerConfig::default();
        assert_eq!(config.listen_addr, "0.0.0.0:50051");
    }

    #[tokio::test]
    async fn test_service_creation() {
        let config = ServerConfig::default();
        // This will fail because warm client can't connect, but tests structure
        let result = KgcSidecarService::new(config).await;
        // Expected to fail in test environment
        assert!(result.is_err());
    }
}

