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
    #[cfg(feature = "otel")]
    weaver_endpoint: Option<String>,
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
        Self::new_with_weaver(config, None)
    }

    #[cfg(feature = "otel")]
    pub fn new_with_weaver(config: SidecarConfig, weaver_endpoint: Option<String>) -> Self {
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
            weaver_endpoint,
        }
    }

    #[cfg(not(feature = "otel"))]
    pub fn new_with_weaver(config: SidecarConfig, _weaver_endpoint: Option<String>) -> Self {
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

    /// Export telemetry to Weaver if enabled
    #[cfg(feature = "otel")]
    async fn export_telemetry(&self, span_name: &str, operation_name: &str, success: bool, latency_ms: u64, attributes: Vec<(&str, String)>) {
        if let Some(ref endpoint) = self.weaver_endpoint {
            use knhk_otel::{Tracer, SpanStatus};
            let mut tracer = knhk_otel::Tracer::with_otlp_exporter(endpoint.clone());
            let span_ctx = tracer.start_span(span_name.to_string(), None);
            
            tracer.add_attribute(span_ctx.clone(), "knhk.operation.name".to_string(), operation_name.to_string());
            tracer.add_attribute(span_ctx.clone(), "knhk.operation.type".to_string(), "sidecar".to_string());
            tracer.add_attribute(span_ctx.clone(), "knhk.sidecar.success".to_string(), success.to_string());
            tracer.add_attribute(span_ctx.clone(), "knhk.sidecar.latency_ms".to_string(), latency_ms.to_string());
            
            for (key, value) in attributes {
                tracer.add_attribute(span_ctx.clone(), format!("knhk.sidecar.{}", key).to_string(), value);
            }
            
            tracer.end_span(span_ctx, if success { SpanStatus::Ok } else { SpanStatus::Error });
            
            // Record metrics using MetricsHelper
            use knhk_otel::MetricsHelper;
            MetricsHelper::record_operation(&mut tracer, operation_name, success);
            MetricsHelper::record_warm_path_latency(&mut tracer, latency_ms * 1000, operation_name); // Convert ms to us
            
            if let Err(e) = tracer.export() {
                warn!(error = %e, "Failed to export telemetry to Weaver");
            }
        }
    }

    #[cfg(not(feature = "otel"))]
    async fn export_telemetry(&self, _span_name: &str, _operation_name: &str, _success: bool, _latency_ms: u64, _attributes: Vec<(&str, String)>) {
        // No-op when OTEL feature is disabled
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

        info!("ApplyTransaction request received with {} RDF bytes", req.rdf_data.len());

        // Convert RDF data to string
        let turtle_data = String::from_utf8(req.rdf_data.clone())
            .map_err(|e| tonic::Status::invalid_argument(format!("Invalid UTF-8 in RDF data: {}", e)))?;

        // Execute ETL pipeline: Ingest → Transform → Load → Reflex → Emit
        let result: Result<knhk_etl::Receipt, SidecarError> = (|| {
            // 1. Ingest: Parse RDF/Turtle
            let ingest = knhk_etl::IngestStage::new(vec!["grpc".to_string()], "turtle".to_string());
            let ingest_result = ingest.parse_rdf_turtle(&turtle_data)
                .map_err(|e| SidecarError::TransactionFailed(format!("Ingest failed: {}", e)))?;

            let ingest_result_full = knhk_etl::IngestResult {
                triples: ingest_result,
                metadata: alloc::collections::BTreeMap::new(),
            };

            // 2. Transform: Hash IRIs to u64, validate schema
            let transform = knhk_etl::TransformStage::new(
                "urn:knhk:schema:default".to_string(),
                false, // Disable validation for now
            );
            let transform_result = transform.transform(ingest_result_full)
                .map_err(|e| SidecarError::TransactionFailed(format!("Transform failed: {}", e)))?;

            // 3. Load: Create SoA arrays
            let load = knhk_etl::LoadStage::new();
            let load_result = load.load(transform_result)
                .map_err(|e| SidecarError::TransactionFailed(format!("Load failed: {}", e)))?;

            // 4. Reflex: Execute hooks (≤8 ticks)
            let reflex = knhk_etl::ReflexStage::new();
            let reflex_result = reflex.reflex(load_result)
                .map_err(|e| SidecarError::TransactionFailed(format!("Reflex failed: {}", e)))?;

            // 5. Emit: Write receipts and actions
            let emit = knhk_etl::EmitStage::new(true, vec![]);
            let emit_result = emit.emit(reflex_result)
                .map_err(|e| SidecarError::TransactionFailed(format!("Emit failed: {}", e)))?;

            // Return first receipt as transaction receipt
            emit_result.receipts.first()
                .cloned()
                .ok_or_else(|| SidecarError::TransactionFailed("No receipt generated".to_string()))
        })();

        let success = result.is_ok();
        self.record_request(success).await;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let _ = self.update_metrics(|m| {
            m.total_transactions += 1;
            m.total_latency_ms += latency_ms;
        }).await;

        // Generate transaction ID
        let transaction_id = uuid::Uuid::new_v4().to_string();

        // Export telemetry to Weaver
        self.export_telemetry(
            "knhk.sidecar.transaction",
            "apply_transaction",
            success,
            latency_ms,
            vec![
                ("transaction_id", transaction_id.clone()),
                ("method", "ApplyTransaction".to_string()),
                ("rdf_bytes", req.rdf_data.len().to_string()),
            ],
        ).await;

        match result {
            Ok(receipt) => {
                let response = ApplyTransactionResponse {
                    committed: true,
                    transaction_id,
                    receipt: Some(proto::TransactionReceipt {
                        receipt_id: receipt.id,
                        ticks: receipt.ticks,
                        span_id: receipt.span_id,
                        a_hash: receipt.a_hash,
                    }),
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

        // Export telemetry to Weaver
        self.export_telemetry(
            "knhk.sidecar.query",
            "query",
            success,
            latency_ms,
            vec![
                ("query_type", format!("{:?}", req.query_type)),
                ("method", "Query".to_string()),
            ],
        ).await;

        Ok(tonic::Response::new(response))
    }

    async fn validate_graph(
        &self,
        request: tonic::Request<ValidateGraphRequest>,
    ) -> Result<tonic::Response<ValidateGraphResponse>, tonic::Status> {
        let start_time = std::time::Instant::now();
        let req = request.into_inner();

        info!("ValidateGraph request received");

        // Note: Graph validation planned for v1.0
        let response = ValidateGraphResponse {
            valid: false,
            errors: vec!["Graph validation not yet implemented".to_string()],
            warnings: vec![],
        };

        let success = false;
        self.record_request(success).await;

        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Export telemetry to Weaver
        self.export_telemetry(
            "knhk.sidecar.validate_graph",
            "validate_graph",
            success,
            latency_ms,
            vec![
                ("method", "ValidateGraph".to_string()),
                ("schema_iri", req.schema_iri.clone()),
            ],
        ).await;

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
        let turtle_data = String::from_utf8(req.rdf_data.clone())
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

        // Export telemetry to Weaver
        self.export_telemetry(
            "knhk.sidecar.evaluate_hook",
            "evaluate_hook",
            success,
            latency_ms,
            vec![
                ("hook_id", req.hook_id.clone()),
                ("method", "EvaluateHook".to_string()),
                ("rdf_data_size", req.rdf_data.len().to_string()),
            ],
        ).await;

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

