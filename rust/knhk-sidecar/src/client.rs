// knhk-sidecar: gRPC client to warm orchestrator

use std::sync::Arc;
use tokio::time::Duration;
use tonic::transport::Channel;
use crate::error::{SidecarError, SidecarResult};
use crate::circuit_breaker::SidecarCircuitBreaker;
use crate::retry::{RetryExecutor, RetryConfig};
use crate::metrics::{MetricsCollector, LatencyTimer};

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Warm orchestrator URL
    pub warm_orchestrator_url: String,
    
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    
    /// Retry configuration
    pub retry_config: RetryConfig,
    
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    
    /// Circuit breaker reset timeout in milliseconds
    pub circuit_breaker_reset_ms: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            warm_orchestrator_url: "http://localhost:50052".to_string(),
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            retry_config: RetryConfig::default(),
            circuit_breaker_threshold: 5,
            circuit_breaker_reset_ms: 60000,
        }
    }
}

/// Sidecar client for communicating with warm orchestrator
pub struct SidecarClient {
    channel: Channel,
    circuit_breaker: SidecarCircuitBreaker,
    retry_executor: RetryExecutor,
    metrics: Arc<MetricsCollector>,
    config: ClientConfig,
}

impl SidecarClient {
    /// Create new sidecar client
    pub async fn new(config: ClientConfig, metrics: Arc<MetricsCollector>) -> SidecarResult<Self> {
        // Parse URL
        let url = config.warm_orchestrator_url.clone();
        
        // Create channel with timeout
        let endpoint = tonic::transport::Endpoint::from_shared(url.clone())
            .map_err(|e| SidecarError::ConfigError(format!("Invalid URL: {}", e)))?
            .timeout(Duration::from_millis(config.connection_timeout_ms))
            .connect_timeout(Duration::from_millis(config.connection_timeout_ms));

        let channel = endpoint.connect()
            .await
            .map_err(|e| SidecarError::NetworkError(format!("Failed to connect to warm orchestrator: {}", e)))?;

        // Create circuit breaker
        let circuit_breaker = SidecarCircuitBreaker::new(
            url.clone(),
            config.circuit_breaker_threshold,
            config.circuit_breaker_reset_ms,
        );

        // Create retry executor
        let retry_executor = RetryExecutor::new(config.retry_config.clone());

        Ok(Self {
            channel,
            circuit_breaker,
            retry_executor,
            metrics,
            config,
        })
    }

    /// Execute transaction
    /// Note: Requires warm orchestrator gRPC service (planned for v1.0)
    pub async fn execute_transaction(&self, request: String) -> SidecarResult<String> {
        let _timer = LatencyTimer::start(Arc::clone(&self.metrics));
        
        // Use circuit breaker and retry
        let result = self.retry_executor.execute(|| async {
            // Check circuit breaker before making call
            if self.circuit_breaker.is_open()? {
                return Err(SidecarError::CircuitBreakerOpen(
                    "Circuit breaker is open".to_string()
                ));
            }
            
            // Note: gRPC call pending warm orchestrator service implementation
            // This will be implemented when knhk-warm exposes gRPC service
            Err(SidecarError::InternalError(
                "Warm orchestrator gRPC service not yet implemented".to_string()
            ))
        })
        .await;

        match &result {
            Ok(_) => {
                self.metrics.record_request(true);
                result
            }
            Err(_) => {
                self.metrics.record_request(false);
                result
            }
        }
    }

    /// Validate graph
    /// Note: Requires warm orchestrator gRPC service (planned for v1.0)
    pub async fn validate_graph(&self, _graph: String, _schema_iri: String) -> SidecarResult<bool> {
        let _timer = LatencyTimer::start(Arc::clone(&self.metrics));
        
        let result = self.retry_executor.execute(|| async {
            // Check circuit breaker before making call
            if self.circuit_breaker.is_open()? {
                return Err(SidecarError::CircuitBreakerOpen(
                    "Circuit breaker is open".to_string()
                ));
            }
            
            // Note: gRPC call pending warm orchestrator service implementation
            Err(SidecarError::InternalError(
                "Warm orchestrator gRPC service not yet implemented".to_string()
            ))
        })
        .await;

        match &result {
            Ok(_) => {
                self.metrics.record_request(true);
                result
            }
            Err(_) => {
                self.metrics.record_request(false);
                result
            }
        }
    }

    /// Evaluate hook
    /// Note: Requires warm orchestrator gRPC service (planned for v1.0)
    pub async fn evaluate_hook(&self, hook_id: String, _input_data: String) -> SidecarResult<String> {
        let _timer = LatencyTimer::start(Arc::clone(&self.metrics));
        
        let result = self.retry_executor.execute(|| async {
            // Check circuit breaker before making call
            if self.circuit_breaker.is_open()? {
                return Err(SidecarError::CircuitBreakerOpen(
                    "Circuit breaker is open".to_string()
                ));
            }
            
            // Note: gRPC call pending warm orchestrator service implementation
            Err(SidecarError::InternalError(
                format!("Warm orchestrator gRPC service not yet implemented for hook {}", hook_id)
            ))
        })
        .await;

        match &result {
            Ok(_) => {
                self.metrics.record_request(true);
                result
            }
            Err(_) => {
                self.metrics.record_request(false);
                result
            }
        }
    }

    /// Get channel for direct gRPC access (for advanced use)
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}

