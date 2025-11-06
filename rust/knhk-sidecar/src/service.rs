// rust/knhk-sidecar/src/service.rs
// gRPC service implementation for KGC Sidecar

use crate::circuit_breaker::CircuitBreaker;
use crate::config::SidecarConfig;
use crate::error::{SidecarError, SidecarResult};
use crate::health::HealthChecker;
use crate::retry::RetryConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

// Include generated proto code
pub mod proto {
    tonic::include_proto!("kgc.sidecar.v1");
}

use proto::{
    kgc_sidecar_server::KgcSidecar, 
    ApplyTransactionRequest, 
    ApplyTransactionResponse,
    EvaluateHookRequest, 
    EvaluateHookResponse, 
    GetMetricsRequest, 
    GetMetricsResponse,
    HealthCheckRequest, 
    HealthCheckResponse, 
    QueryRequest, 
    QueryResponse,
    SidecarMetrics, 
    ValidateGraphRequest, 
    ValidateGraphResponse,
};

pub struct KgcSidecarService {
    config: SidecarConfig,
    circuit_breaker: Arc<CircuitBreaker>,
    health_checker: Arc<HealthChecker>,
    retry_config: RetryConfig,
    metrics: Arc<Mutex<ServiceMetrics>>,
}

#[derive(Default)]
struct ServiceMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_transactions: u64,
    total_queries: u64,
    total_hooks_evaluated: u64,
    circuit_breaker_open_count: u64,
    retry_count: u64,
    total_latency_ms: u64,
    last_request_time_ms: u64,
}

impl KgcSidecarService {
    pub fn new(config: SidecarConfig) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            config.circuit_breaker_failure_threshold,
            config.circuit_breaker_reset_timeout_ms,
        ));

        let health_checker = Arc::new(HealthChecker::new(5000));

        let retry_config = RetryConfig::new(
            config.retry_max_attempts,
            config.retry_initial_delay_ms,
            config.retry_max_delay_ms,
        );

        Self {
            config,
            circuit_breaker,
            health_checker,
            retry_config,
            metrics: Arc::new(Mutex::new(ServiceMetrics::default())),
        }
    }

    async fn update_metrics<F>(&self, f: F) -> SidecarResult<()>
    where
        F: FnOnce(&mut ServiceMetrics),
    {
        let mut metrics = self.metrics.lock().await;
        f(&mut metrics);
        Ok(())
    }

    async fn record_request(&self, success: bool) {
        let _ = self.update_metrics(|m| {
            m.total_requests += 1;
            if success {
                m.successful_requests += 1;
            } else {
                m.failed_requests += 1;
            }
            m.last_request_time_ms = chrono::Utc::now().timestamp_millis() as u64;
        }).await;
    }
}

#[tonic::async_trait]
impl KgcSidecar for KgcSidecarService {
    async fn apply_transaction(
        &self,
        request: tonic::Request<ApplyTransactionRequest>,
    ) -> Result<tonic::Response<ApplyTransactionResponse>, tonic::Status> {
        let start_time = std::time::Instant::now();
        let req = request.into_inner();

        info!("ApplyTransaction request received");

        // Note: ETL pipeline integration planned for v1.0
        let result: Result<(), SidecarError> = Err(SidecarError::TransactionFailed(
            "ETL pipeline integration pending".to_string(),
        ));

        let success = result.is_ok();
        self.record_request(success).await;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let _ = self.update_metrics(|m| {
            m.total_transactions += 1;
            m.total_latency_ms += latency_ms;
        }).await;

        match result {
            Ok(_) => {
                let response = ApplyTransactionResponse {
                    committed: true,
                    transaction_id: uuid::Uuid::new_v4().to_string(),
                    receipt: None, // Note: Receipt generation planned for v1.0
                    errors: vec![],
                };
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                error!("Transaction failed: {:?}", e);
                let response = ApplyTransactionResponse {
                    committed: false,
                    transaction_id: String::new(),
                    receipt: None,
                    errors: vec![format!("{:?}", e)],
                };
                Ok(tonic::Response::new(response))
            }
        }
    }

    async fn query(
        &self,
        request: tonic::Request<QueryRequest>,
    ) -> Result<tonic::Response<QueryResponse>, tonic::Status> {
        let start_time = std::time::Instant::now();
        let req = request.into_inner();

        info!("Query request received: {:?}", req.query_type);

        // Note: Query execution using knhk-warm planned for v1.0
        let response = QueryResponse {
            success: false,
            query_type: req.query_type,
            result: None,
            errors: vec!["Query execution not yet implemented".to_string()],
        };

        let success = false;
        self.record_request(success).await;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let _ = self.update_metrics(|m| {
            m.total_queries += 1;
            m.total_latency_ms += latency_ms;
        }).await;

        Ok(tonic::Response::new(response))
    }

    async fn validate_graph(
        &self,
        request: tonic::Request<ValidateGraphRequest>,
    ) -> Result<tonic::Response<ValidateGraphResponse>, tonic::Status> {
        let req = request.into_inner();

        info!("ValidateGraph request received");

        // Note: Graph validation planned for v1.0
        let response = ValidateGraphResponse {
            valid: false,
            errors: vec!["Graph validation not yet implemented".to_string()],
            warnings: vec![],
        };

        self.record_request(false).await;

        Ok(tonic::Response::new(response))
    }

    async fn evaluate_hook(
        &self,
        request: tonic::Request<EvaluateHookRequest>,
    ) -> Result<tonic::Response<EvaluateHookResponse>, tonic::Status> {
        let start_time = std::time::Instant::now();
        let req = request.into_inner();

        info!("EvaluateHook request received: {}", req.hook_id);

        // Convert RDF data to string
        let turtle_data = String::from_utf8(req.rdf_data)
            .map_err(|e| tonic::Status::invalid_argument(format!("Invalid UTF-8: {}", e)))?;

        // Get hook from registry
        // Note: Integration with knhk-unrdf hooks_native planned for v1.0
        let response = EvaluateHookResponse {
            fired: false,
            result: None,
            receipt: None,
            errors: vec!["Hook evaluation not yet implemented".to_string()],
        };

        let success = false;
        self.record_request(success).await;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let _ = self.update_metrics(|m| {
            m.total_hooks_evaluated += 1;
            m.total_latency_ms += latency_ms;
        }).await;

        Ok(tonic::Response::new(response))
    }

    async fn health_check(
        &self,
        _request: tonic::Request<HealthCheckRequest>,
    ) -> Result<tonic::Response<HealthCheckResponse>, tonic::Status> {
        let status = self.health_checker.check().await;

        let proto_status = match status {
            crate::health::HealthStatus::Healthy => proto::health_status::HealthStatus::HealthStatusHealthy as i32,
            crate::health::HealthStatus::Degraded(_) => proto::health_status::HealthStatus::HealthStatusDegraded as i32,
            crate::health::HealthStatus::Unhealthy(_) => proto::health_status::HealthStatus::HealthStatusUnhealthy as i32,
        };

        let message = match status {
            crate::health::HealthStatus::Healthy => "Service is healthy".to_string(),
            crate::health::HealthStatus::Degraded(reason) => format!("Service is degraded: {}", reason),
            crate::health::HealthStatus::Unhealthy(reason) => format!("Service is unhealthy: {}", reason),
        };

        let response = HealthCheckResponse {
            status: proto_status,
            message,
            timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
        };

        Ok(tonic::Response::new(response))
    }

    async fn get_metrics(
        &self,
        _request: tonic::Request<GetMetricsRequest>,
    ) -> Result<tonic::Response<GetMetricsResponse>, tonic::Status> {
        let metrics = self.metrics.lock().await;

        let avg_latency_ms = if metrics.total_requests > 0 {
            metrics.total_latency_ms as f64 / metrics.total_requests as f64
        } else {
            0.0
        };

        let proto_metrics = SidecarMetrics {
            total_requests: metrics.total_requests,
            successful_requests: metrics.successful_requests,
            failed_requests: metrics.failed_requests,
            total_transactions: metrics.total_transactions,
            total_queries: metrics.total_queries,
            total_hooks_evaluated: metrics.total_hooks_evaluated,
            circuit_breaker_open_count: metrics.circuit_breaker_open_count,
            retry_count: metrics.retry_count,
            average_latency_ms: avg_latency_ms,
            last_request_time_ms: metrics.last_request_time_ms,
        };

        let response = GetMetricsResponse {
            metrics: Some(proto_metrics),
        };

        Ok(tonic::Response::new(response))
    }
}

