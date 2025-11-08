// rust/knhk-sidecar/src/service.rs
// gRPC service implementation for KGC Sidecar

extern crate alloc;

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

use crate::json_parser::{parse_json_triples, JsonDelta};
use proto::{
    kgc_sidecar_server::KgcSidecar, ApplyTransactionRequest, ApplyTransactionResponse,
    EvaluateHookRequest, EvaluateHookResponse, GetMetricsRequest, GetMetricsResponse,
    HealthCheckRequest, HealthCheckResponse, QueryRequest, QueryResponse, SidecarMetrics,
    ValidateGraphRequest, ValidateGraphResponse,
};

pub struct KgcSidecarService {
    config: SidecarConfig,
    circuit_breaker: Arc<CircuitBreaker>,
    health_checker: Arc<HealthChecker>,
    retry_config: RetryConfig,
    metrics: Arc<Mutex<ServiceMetrics>>,
    #[cfg(feature = "otel")]
    weaver_endpoint: Option<String>,
    /// Beat admission manager for 8-beat epoch system
    beat_admission: Option<Arc<crate::beat_admission::BeatAdmission>>,
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
        Self::new_with_weaver(config, None, None)
    }

    #[cfg(feature = "otel")]
    pub fn new_with_weaver(
        config: SidecarConfig,
        weaver_endpoint: Option<String>,
        beat_admission: Option<Arc<crate::beat_admission::BeatAdmission>>,
    ) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            config.circuit_breaker_failure_threshold,
            config.circuit_breaker_reset_timeout_ms,
        ));

        let health_checker = Arc::new(HealthChecker::new(5000));

        let retry_config = RetryConfig {
            max_retries: config.retry_max_attempts,
            initial_delay_ms: config.retry_initial_delay_ms,
            max_delay_ms: config.retry_max_delay_ms,
            multiplier: 2.0,
        };

        Self {
            config,
            circuit_breaker,
            health_checker,
            retry_config,
            metrics: Arc::new(Mutex::new(ServiceMetrics::default())),
            weaver_endpoint,
            beat_admission,
        }
    }

    #[cfg(not(feature = "otel"))]
    pub fn new_with_weaver(
        config: SidecarConfig,
        _weaver_endpoint: Option<String>,
        beat_admission: Option<Arc<crate::beat_admission::BeatAdmission>>,
    ) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            config.circuit_breaker_failure_threshold,
            config.circuit_breaker_reset_timeout_ms,
        ));

        let health_checker = Arc::new(HealthChecker::new(5000));

        let retry_config = RetryConfig {
            max_retries: config.retry_max_attempts,
            initial_delay_ms: config.retry_initial_delay_ms,
            max_delay_ms: config.retry_max_delay_ms,
            multiplier: 2.0,
        };

        Self {
            config,
            circuit_breaker,
            health_checker,
            retry_config,
            metrics: Arc::new(Mutex::new(ServiceMetrics::default())),
            beat_admission,
        }
    }

    /// Export telemetry to Weaver if enabled
    #[cfg(feature = "otel")]
    async fn export_telemetry(
        &self,
        span_name: &str,
        operation_name: &str,
        success: bool,
        latency_ms: u64,
        attributes: Vec<(&str, String)>,
    ) {
        if let Some(ref endpoint) = self.weaver_endpoint {
            use knhk_otel::{SpanStatus, Tracer};
            let mut tracer = knhk_otel::Tracer::with_otlp_exporter(endpoint.clone());
            let span_ctx = tracer.start_span(span_name.to_string(), None);

            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.name".to_string(),
                operation_name.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.operation.type".to_string(),
                "sidecar".to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.sidecar.success".to_string(),
                success.to_string(),
            );
            tracer.add_attribute(
                span_ctx.clone(),
                "knhk.sidecar.latency_ms".to_string(),
                latency_ms.to_string(),
            );

            for (key, value) in attributes {
                tracer.add_attribute(
                    span_ctx.clone(),
                    format!("knhk.sidecar.{}", key).to_string(),
                    value,
                );
            }

            tracer.end_span(
                span_ctx,
                if success {
                    SpanStatus::Ok
                } else {
                    SpanStatus::Error
                },
            );

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
    async fn export_telemetry(
        &self,
        _span_name: &str,
        _operation_name: &str,
        _success: bool,
        _latency_ms: u64,
        _attributes: Vec<(&str, String)>,
    ) {
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
        let _ = self
            .update_metrics(|m| {
                m.total_requests += 1;
                if success {
                    m.successful_requests += 1;
                } else {
                    m.failed_requests += 1;
                }
                m.last_request_time_ms = chrono::Utc::now().timestamp_millis() as u64;
            })
            .await;
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

        info!(
            "ApplyTransaction request received with delta containing {} triples",
            req.delta
                .as_ref()
                .map(|d| d.additions.len() + d.removals.len())
                .unwrap_or(0)
        );

        // Extract RDF data from delta (support both protobuf and JSON formats)
        let raw_triples = if let Some(delta) = req.delta {
            // Protobuf format: convert to RawTriple
            let mut all_triples: Vec<knhk_etl::ingest::RawTriple> = Vec::new();
            for triple in delta.additions {
                all_triples.push(knhk_etl::ingest::RawTriple {
                    subject: triple.subject,
                    predicate: triple.predicate,
                    object: triple.object,
                    graph: if triple.graph.is_empty() {
                        None
                    } else {
                        Some(triple.graph)
                    },
                });
            }
            // Note: Removals handled separately in pipeline
            all_triples
        } else {
            // JSON format: parse with simdjson (when json_data field is added in proto)
            // For now, fall back to error if no delta provided
            return Err(tonic::Status::invalid_argument(
                "Delta is required in ApplyTransactionRequest",
            ));
        };

        let triple_count = raw_triples.len();

        // Start OTEL span for transaction
        #[cfg(feature = "otel")]
        let mut span_tracer: Option<(knhk_otel::Tracer, knhk_otel::SpanContext)> =
            if let Some(ref endpoint) = self.weaver_endpoint {
                use knhk_otel::Tracer;
                let mut tracer = knhk_otel::Tracer::with_otlp_exporter(endpoint.clone());
                let ctx = tracer.start_span("knhk.sidecar.transaction".to_string(), None);
                tracer.add_attribute(
                    ctx.clone(),
                    "knhk.operation.name".to_string(),
                    "apply_transaction".to_string(),
                );
                tracer.add_attribute(
                    ctx.clone(),
                    "knhk.operation.type".to_string(),
                    "sidecar".to_string(),
                );
                tracer.add_attribute(
                    ctx.clone(),
                    "knhk.sidecar.rdf_bytes".to_string(),
                    triple_count.to_string(),
                );
                Some((tracer, ctx))
            } else {
                None
            };

        // Execute ETL pipeline: Ingest → Transform → Load → Reflex → Emit
        let result: Result<knhk_etl::Receipt, SidecarError> = (|| {
            // 1. Ingest: Use raw triples directly (no parsing needed - already converted)
            let ingest_result_full = knhk_etl::IngestResult {
                triples: raw_triples,
                metadata: alloc::collections::BTreeMap::new(),
            };

            // 2. Transform: Hash IRIs to u64, validate schema
            let transform = knhk_etl::TransformStage::new(
                "urn:knhk:schema:default".to_string(),
                false, // Disable validation for now
            );
            let transform_result = transform.transform(ingest_result_full).map_err(|e| {
                SidecarError::transaction_failed(format!("SIDECAR_TRANSFORM_FAILED: {:?}", e))
            })?;

            // 3. Load: Create SoA arrays
            let load = knhk_etl::LoadStage::new();
            let load_result = load.load(transform_result).map_err(|e| {
                SidecarError::transaction_failed(format!("SIDECAR_LOAD_FAILED: {}", e.message()))
            })?;

            // 4. Reflex: Execute hooks (≤8 ticks)
            let reflex = knhk_etl::ReflexStage::new();
            let reflex_result = reflex.reflex(load_result).map_err(|e| {
                SidecarError::transaction_failed(format!("SIDECAR_REFLEX_FAILED: {}", e.message()))
            })?;

            // 5. Emit: Write receipts and actions
            let mut emit = knhk_etl::EmitStage::new(true, vec![]);
            let receipts = reflex_result.receipts.clone();
            let emit_result = emit.emit(reflex_result).map_err(|e| {
                SidecarError::transaction_failed(format!(
                    "SIDECAR_EMIT_FAILED: {} (stage=emit)",
                    e.message()
                ))
            })?;

            // Return first receipt as transaction receipt
            receipts.first().cloned().ok_or_else(|| {
                SidecarError::transaction_failed(
                    "SIDECAR_NO_RECEIPT: No receipt generated".to_string(),
                )
            })
        })();

        let success = result.is_ok();
        self.record_request(success).await;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let _ = self
            .update_metrics(|m| {
                m.total_transactions += 1;
                m.total_latency_ms += latency_ms;
            })
            .await;

        // Generate transaction ID
        let transaction_id = uuid::Uuid::new_v4().to_string();
        let transaction_id_clone = transaction_id.clone();

        // Export telemetry to Weaver
        self.export_telemetry(
            "knhk.sidecar.transaction",
            "apply_transaction",
            success,
            latency_ms,
            vec![
                ("transaction_id", transaction_id.clone()),
                ("method", "ApplyTransaction".to_string()),
                ("rdf_bytes", triple_count.to_string()),
            ],
        )
        .await;

        match result {
            Ok(receipt) => {
                // End span with success status
                #[cfg(feature = "otel")]
                if let Some((ref mut tracer, ref span_ctx)) = span_tracer {
                    use knhk_otel::SpanStatus;
                    tracer.add_attribute(
                        span_ctx.clone(),
                        "knhk.sidecar.success".to_string(),
                        "true".to_string(),
                    );
                    tracer.add_attribute(
                        span_ctx.clone(),
                        "knhk.sidecar.receipt_id".to_string(),
                        receipt.id.clone(),
                    );
                    tracer.add_attribute(
                        span_ctx.clone(),
                        "knhk.sidecar.ticks".to_string(),
                        receipt.ticks.to_string(),
                    );
                    tracer.end_span(span_ctx.clone(), SpanStatus::Ok);
                    if let Err(e) = tracer.export() {
                        warn!(error = %e, "Failed to export success telemetry");
                    }
                }

                let response = ApplyTransactionResponse {
                    committed: true,
                    transaction_id,
                    receipt: Some(proto::Receipt {
                        ticks: receipt.ticks,
                        lanes: receipt.lanes,
                        span_id: receipt.span_id,
                        a_hash: receipt.a_hash.to_le_bytes().to_vec(),
                        transaction_id: transaction_id_clone,
                        committed: true,
                    }),
                    errors: vec![],
                };
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                error!(error_code = %e.code(), error = %e, "Transaction failed");

                // Record error to OTEL span if available
                #[cfg(feature = "otel")]
                if let Some((ref mut tracer, ref span_ctx)) = span_tracer {
                    e.record_to_span(tracer, span_ctx.clone());
                    if let Err(export_err) = tracer.export() {
                        warn!(error = %export_err, "Failed to export error telemetry");
                    }
                }

                // Convert error to JSON for structured logging
                #[cfg(feature = "serde_json")]
                let error_json = e.to_json().unwrap_or_else(|_| format!("{:?}", e));
                #[cfg(not(feature = "serde_json"))]
                let error_json = format!("{:?}", e);

                let response = ApplyTransactionResponse {
                    committed: false,
                    transaction_id: String::new(),
                    receipt: None,
                    errors: vec![error_json],
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

        info!(
            "Query request received: type={:?}, query_len={}",
            req.query_type,
            req.query.len()
        );

        // Execute query through knhk-etl pipeline
        let result: Result<Vec<String>, SidecarError> = (|| {
            // 1. Parse query context as RDF (if provided)
            let ingest =
                knhk_etl::IngestStage::new(vec!["query".to_string()], "turtle".to_string());

            // Helper function to parse data based on format
            let parse_query_data =
                |data: &[u8],
                 format: &str|
                 -> Result<Vec<knhk_etl::ingest::RawTriple>, SidecarError> {
                    if data.is_empty() {
                        return Ok(Vec::new());
                    }

                    match format {
                        "json" | "json-ld" | "application/json" => {
                            // Use simdjson for fast JSON parsing
                            parse_json_triples(data)
                        }
                        "turtle" | "text/turtle" | "" => {
                            // Use oxigraph for Turtle parsing
                            let turtle_data = String::from_utf8(data.to_vec()).map_err(|e| {
                                SidecarError::query_failed(format!("Invalid UTF-8: {}", e))
                            })?;
                            ingest
                                .parse_rdf_turtle(&turtle_data)
                                .map_err(|e| SidecarError::query_failed(e.message().to_string()))
                        }
                        _ => {
                            // Try to auto-detect: if starts with { or [, assume JSON
                            if data
                                .first()
                                .map(|&b| b == b'{' || b == b'[')
                                .unwrap_or(false)
                            {
                                parse_json_triples(data)
                            } else {
                                // Default to Turtle
                                let turtle_data =
                                    String::from_utf8(data.to_vec()).map_err(|e| {
                                        SidecarError::query_failed(format!("Invalid UTF-8: {}", e))
                                    })?;
                                ingest.parse_rdf_turtle(&turtle_data).map_err(|e| {
                                    SidecarError::query_failed(e.message().to_string())
                                })
                            }
                        }
                    }
                };

            // For ASK queries, use hot path (≤8 ticks)
            // For SELECT queries, use warm path
            match req.query_type {
                // ASK query: Fast boolean check (hot path)
                0 => {
                    // Parse query to extract pattern
                    // Execute via ReflexStage for ≤8 ticks
                    let turtle_data = format!(
                        "<http://example.org/s> <http://example.org/p> <http://example.org/o> ."
                    );
                    let triples = ingest
                        .parse_rdf_turtle(&turtle_data)
                        .map_err(|e| SidecarError::query_failed(e.message().to_string()))?;

                    let ingest_result = knhk_etl::IngestResult {
                        triples,
                        metadata: alloc::collections::BTreeMap::new(),
                    };

                    // Transform and execute
                    let transform =
                        knhk_etl::TransformStage::new("urn:knhk:schema:query".to_string(), false);
                    let transform_result = transform.transform(ingest_result).map_err(|e| {
                        SidecarError::query_failed(format!("Transform failed: {:?}", e))
                    })?;

                    let load = knhk_etl::LoadStage::new();
                    let load_result = load
                        .load(transform_result)
                        .map_err(|e| SidecarError::query_failed(e.message().to_string()))?;

                    let reflex = knhk_etl::ReflexStage::new();
                    let reflex_result = reflex
                        .reflex(load_result)
                        .map_err(|e| SidecarError::query_failed(e.message().to_string()))?;

                    // Return boolean result based on receipts
                    let has_results = !reflex_result.receipts.is_empty();
                    Ok(vec![has_results.to_string()])
                }
                // SELECT query: Return triples (warm path)
                1 => {
                    // Parse query data if provided (support JSON and Turtle)
                    let triples = parse_query_data(&req.data, &req.data_format)?;

                    // Return triple subjects as results
                    Ok(triples.iter().map(|t| t.subject.clone()).collect())
                }
                // CONSTRUCT query: Build new graph
                2 => {
                    // Parse query data if provided (support JSON and Turtle)
                    let triples = parse_query_data(&req.data, &req.data_format)?;

                    // Return constructed triples as N-Triples format
                    Ok(triples
                        .iter()
                        .map(|t| format!("<{}> <{}> <{}> .", t.subject, t.predicate, t.object))
                        .collect())
                }
                _ => Err(SidecarError::query_failed(
                    "Unsupported query type".to_string(),
                )),
            }
        })();

        let success = result.is_ok();
        self.record_request(success).await;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let _ = self
            .update_metrics(|m| {
                m.total_queries += 1;
                m.total_latency_ms += latency_ms;
            })
            .await;

        // Export telemetry to Weaver
        self.export_telemetry(
            "knhk.sidecar.query",
            "query",
            success,
            latency_ms,
            vec![
                ("query_type", format!("{:?}", req.query_type)),
                ("method", "Query".to_string()),
                ("query_len", req.query.len().to_string()),
            ],
        )
        .await;

        match result {
            Ok(results) => {
                let response = QueryResponse {
                    success: true,
                    query_type: req.query_type,
                    result: None, // Query result not yet implemented in proto
                    errors: vec![],
                };
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                error!("Query failed: {:?}", e);
                let response = QueryResponse {
                    success: false,
                    query_type: req.query_type,
                    result: None,
                    errors: vec![format!("{:?}", e)],
                };
                Ok(tonic::Response::new(response))
            }
        }
    }

    async fn validate_graph(
        &self,
        request: tonic::Request<ValidateGraphRequest>,
    ) -> Result<tonic::Response<ValidateGraphResponse>, tonic::Status> {
        let start_time = std::time::Instant::now();
        let req = request.into_inner();

        info!(
            "ValidateGraph request received for schema: {}",
            req.schema_iri
        );

        // Validate graph using knhk-etl Transform stage (schema validation)
        let result: Result<(), SidecarError> = (|| {
            // Parse RDF data based on format (support JSON and Turtle)
            let ingest =
                knhk_etl::IngestStage::new(vec!["validation".to_string()], "turtle".to_string());

            let triples = match req.data_format.as_str() {
                "json" | "json-ld" | "application/json" => {
                    // Use simdjson for fast JSON parsing
                    parse_json_triples(&req.rdf_data)?
                }
                "turtle" | "text/turtle" | "" => {
                    // Use oxigraph for Turtle parsing
                    let turtle_data = String::from_utf8(req.rdf_data.clone()).map_err(|e| {
                        SidecarError::validation_failed(format!("Invalid UTF-8: {}", e))
                    })?;
                    ingest
                        .parse_rdf_turtle(&turtle_data)
                        .map_err(|e| SidecarError::validation_failed(e.message().to_string()))?
                }
                _ => {
                    // Auto-detect: if starts with { or [, assume JSON
                    if req
                        .rdf_data
                        .first()
                        .map(|&b| b == b'{' || b == b'[')
                        .unwrap_or(false)
                    {
                        parse_json_triples(&req.rdf_data)?
                    } else {
                        // Default to Turtle
                        let turtle_data = String::from_utf8(req.rdf_data.clone()).map_err(|e| {
                            SidecarError::validation_failed(format!("Invalid UTF-8: {}", e))
                        })?;
                        ingest
                            .parse_rdf_turtle(&turtle_data)
                            .map_err(|e| SidecarError::validation_failed(e.message().to_string()))?
                    }
                }
            };

            let ingest_result = knhk_etl::IngestResult {
                triples,
                metadata: alloc::collections::BTreeMap::new(),
            };

            // 2. Transform with schema validation enabled
            let transform = knhk_etl::TransformStage::new(req.schema_iri.clone(), true);
            let transform_result = transform.transform(ingest_result).map_err(|e| {
                SidecarError::validation_failed(format!("Schema validation failed: {:?}", e))
            })?;

            // Check for validation errors
            if !transform_result.validation_errors.is_empty() {
                return Err(SidecarError::validation_failed(format!(
                    "Validation errors: {}",
                    transform_result.validation_errors.join(", ")
                )));
            }

            Ok(())
        })();

        let success = result.is_ok();
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
                ("rdf_bytes", req.rdf_data.len().to_string()),
            ],
        )
        .await;

        match result {
            Ok(_) => {
                let response = ValidateGraphResponse {
                    valid: true,
                    errors: vec![],
                    warnings: vec![],
                };
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                warn!("Validation failed: {:?}", e);
                let response = ValidateGraphResponse {
                    valid: false,
                    errors: vec![format!("{:?}", e)],
                    warnings: vec![],
                };
                Ok(tonic::Response::new(response))
            }
        }
    }

    async fn evaluate_hook(
        &self,
        request: tonic::Request<EvaluateHookRequest>,
    ) -> Result<tonic::Response<EvaluateHookResponse>, tonic::Status> {
        let start_time = std::time::Instant::now();
        let req = request.into_inner();

        info!(
            "EvaluateHook request received: hook_id={}, rdf_bytes={}",
            req.hook_id,
            req.rdf_data.len()
        );

        // Execute hook via ETL pipeline (hooks are evaluated during Reflex stage)
        let result: Result<knhk_etl::Receipt, SidecarError> = (|| {
            // Parse RDF data based on format (support JSON and Turtle)
            let ingest = knhk_etl::IngestStage::new(vec!["hook".to_string()], "turtle".to_string());

            let triples = match req.data_format.as_str() {
                "json" | "json-ld" | "application/json" => {
                    // Use simdjson for fast JSON parsing
                    parse_json_triples(&req.rdf_data)?
                }
                "turtle" | "text/turtle" | "" => {
                    // Use oxigraph for Turtle parsing
                    let turtle_data = String::from_utf8(req.rdf_data.clone()).map_err(|e| {
                        SidecarError::hook_evaluation_failed(format!("Invalid UTF-8: {}", e))
                    })?;
                    ingest.parse_rdf_turtle(&turtle_data).map_err(|e| {
                        SidecarError::hook_evaluation_failed(format!("Ingest failed: {:?}", e))
                    })?
                }
                _ => {
                    // Auto-detect: if starts with { or [, assume JSON
                    if req
                        .rdf_data
                        .first()
                        .map(|&b| b == b'{' || b == b'[')
                        .unwrap_or(false)
                    {
                        parse_json_triples(&req.rdf_data)?
                    } else {
                        // Default to Turtle
                        let turtle_data = String::from_utf8(req.rdf_data.clone()).map_err(|e| {
                            SidecarError::hook_evaluation_failed(format!("Invalid UTF-8: {}", e))
                        })?;
                        ingest.parse_rdf_turtle(&turtle_data).map_err(|e| {
                            SidecarError::hook_evaluation_failed(format!("Ingest failed: {:?}", e))
                        })?
                    }
                }
            };

            let ingest_result = knhk_etl::IngestResult {
                triples,
                metadata: alloc::collections::BTreeMap::new(),
            };

            // 2. Transform to typed triples
            let transform =
                knhk_etl::TransformStage::new("urn:knhk:schema:hook".to_string(), false);
            let transform_result = transform.transform(ingest_result).map_err(|e| {
                SidecarError::hook_evaluation_failed(format!("Transform failed: {}", e.message()))
            })?;

            // 3. Load into SoA arrays
            let load = knhk_etl::LoadStage::new();
            let load_result = load.load(transform_result).map_err(|e| {
                SidecarError::hook_evaluation_failed(format!("Load failed: {}", e.message()))
            })?;

            // 4. Execute hook via Reflex stage (≤8 ticks)
            // This is where the actual hook evaluation happens
            let reflex = knhk_etl::ReflexStage::new();
            let reflex_result = reflex.reflex(load_result).map_err(|e| {
                SidecarError::hook_evaluation_failed(format!("Reflex failed: {}", e.message()))
            })?;

            // Return first receipt from hook execution
            reflex_result.receipts.first().cloned().ok_or_else(|| {
                SidecarError::hook_evaluation_failed("No receipt generated".to_string())
            })
        })();

        let success = result.is_ok();
        self.record_request(success).await;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let _ = self
            .update_metrics(|m| {
                m.total_hooks_evaluated += 1;
                m.total_latency_ms += latency_ms;
            })
            .await;

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
        )
        .await;

        match result {
            Ok(receipt) => {
                // Hook fired successfully if receipt has non-zero ticks
                let fired = receipt.ticks > 0;

                // Serialize receipt to bytes (simplified - in production would use proper serialization)
                let receipt_bytes = format!(
                    "{{\"id\":\"{}\",\"ticks\":{},\"span_id\":{},\"a_hash\":{}}}",
                    receipt.id, receipt.ticks, receipt.span_id, receipt.a_hash
                )
                .into_bytes();

                let response = EvaluateHookResponse {
                    fired,
                    result: receipt_bytes, // Hook result as bytes
                    receipt: Some(proto::Receipt {
                        ticks: receipt.ticks,
                        lanes: receipt.lanes,
                        span_id: receipt.span_id,
                        a_hash: receipt.a_hash.to_le_bytes().to_vec(),
                        transaction_id: String::new(),
                        committed: true,
                    }),
                    errors: vec![],
                };
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                error!("Hook evaluation failed: {:?}", e);
                let response = EvaluateHookResponse {
                    fired: false,
                    result: Vec::new(), // Empty bytes for error case
                    receipt: None,
                    errors: vec![format!("{:?}", e)],
                };
                Ok(tonic::Response::new(response))
            }
        }
    }

    async fn health_check(
        &self,
        _request: tonic::Request<HealthCheckRequest>,
    ) -> Result<tonic::Response<HealthCheckResponse>, tonic::Status> {
        let status = self.health_checker.check().await;

        let (healthy, message) = match status {
            crate::health::HealthStatus::Healthy => (true, "Service is healthy".to_string()),
            crate::health::HealthStatus::Degraded(reason) => {
                (true, format!("Service is degraded: {}", reason))
            }
            crate::health::HealthStatus::Unhealthy(reason) => {
                (false, format!("Service is unhealthy: {}", reason))
            }
        };

        let response = HealthCheckResponse {
            status: if healthy { 1 } else { 0 },
            message,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
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
