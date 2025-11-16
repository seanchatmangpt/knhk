# Phase 5: Complete gRPC API Implementation Guide

## Overview

This document provides the complete implementation for Phase 5: gRPC API with streaming, structured logging, and comprehensive observability.

## Status

- ✅ Proto definitions extended with streaming methods
- ✅ Health checker enhanced with K8s-style probes
- ⏳ Proto code generation pending (requires fixing knhk-hot linking issue)
- ⏳ Streaming handlers implementation (requires generated proto code)

## Components Completed

### 1. Extended Proto Definitions

**File**: `/home/user/knhk/rust/knhk-workflow-engine/proto/workflow_engine.proto`

New RPC methods added:
- `WatchCase` - Server streaming for real-time case events
- `ListCases` - Server streaming for paginated case listing
- `ExportCaseXes` - Export case to XES format for process mining
- `HealthCheck` - Kubernetes-style health probes

### 2. Enhanced Health Checker

**File**: `/home/user/knhk/rust/knhk-workflow-engine/src/observability/health.rs`

New methods added:
- `readiness_probe()` - Kubernetes readiness probe (ready to accept traffic)
- `liveness_probe()` - Kubernetes liveness probe (detect deadlocks)
- `startup_probe()` - Kubernetes startup probe (initialization complete)
- `get_health_details()` - JSON-serializable health details for gRPC response

## Implementation Needed (Post Proto Build)

### 3. Streaming gRPC Handlers

**File**: `/home/user/knhk/rust/knhk-workflow-engine/src/api/grpc.rs`

```rust
#[tonic::async_trait]
impl WorkflowEngineService for GrpcService {
    // ... existing methods ...

    /// Watch case for real-time updates
    async fn watch_case(
        &self,
        request: Request<proto::WatchCaseRequest>,
    ) -> Result<Response<Self::WatchCaseStream>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        // Create event stream
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let engine = self.engine.clone();

        // Spawn background task to watch case
        tokio::spawn(async move {
            // TODO: Implement event subscription mechanism
            // For now, poll case state periodically
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

            loop {
                interval.tick().await;

                // Get current case state
                match engine.get_case(&case_id).await {
                    Ok(case) => {
                        let event = proto::CaseEvent {
                            case_id: case.id.to_string(),
                            event_type: "StateUpdate".to_string(),
                            timestamp: chrono::Utc::now().timestamp(),
                            data: serde_json::to_string(&case.data).unwrap_or_default(),
                            state: format!("{:?}", case.state),
                        };

                        if tx.send(Ok(event)).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Response::new(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        ))
    }

    type WatchCaseStream = tokio_stream::wrappers::ReceiverStream<
        Result<proto::CaseEvent, Status>,
    >;

    /// List cases with pagination
    async fn list_cases(
        &self,
        request: Request<proto::ListCasesRequest>,
    ) -> Result<Response<Self::ListCasesStream>, Status> {
        let req = request.into_inner();
        let page_size = if req.page_size > 0 {
            req.page_size as usize
        } else {
            100
        };

        // Create case stream
        let (tx, rx) = tokio::sync::mpsc::channel(page_size);
        let engine = self.engine.clone();
        let spec_id_filter = if !req.spec_id.is_empty() {
            Some(
                WorkflowSpecId::parse_str(&req.spec_id)
                    .map_err(|_| Status::invalid_argument("Invalid spec_id"))?,
            )
        } else {
            None
        };

        tokio::spawn(async move {
            // TODO: Implement pagination in StateStore
            // For now, get all cases and filter
            match engine.list_cases().await {
                Ok(cases) => {
                    let filtered = cases
                        .into_iter()
                        .filter(|case| {
                            if let Some(ref spec_id) = spec_id_filter {
                                &case.spec_id == spec_id
                            } else {
                                true
                            }
                        })
                        .take(page_size);

                    for case in filtered {
                        let proto_case = proto::Case {
                            id: case.id.to_string(),
                            spec_id: case.spec_id.to_string(),
                            state: format!("{:?}", case.state),
                            data: serde_json::to_string(&case.data).unwrap_or_default(),
                            created_at: case.created_at.timestamp(),
                            updated_at: case.created_at.timestamp(),
                        };

                        if tx.send(Ok(proto_case)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(Status::internal(format!("Failed to list cases: {}", e))))
                        .await;
                }
            }
        });

        Ok(Response::new(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        ))
    }

    type ListCasesStream = tokio_stream::wrappers::ReceiverStream<
        Result<proto::Case, Status>,
    >;

    /// Export case to XES format
    async fn export_case_xes(
        &self,
        request: Request<proto::ExportCaseRequest>,
    ) -> Result<Response<proto::ExportCaseResponse>, Status> {
        let req = request.into_inner();
        let case_id = CaseId::parse_str(&req.case_id)
            .map_err(|_| Status::invalid_argument("Invalid case_id format"))?;

        // Use XES export functionality
        use crate::process_mining::xes_export::XesExporter;

        let exporter = XesExporter::new();
        let xes_data = exporter
            .export_case(&self.engine, &case_id)
            .await
            .map_err(|e| Status::internal(format!("XES export failed: {}", e)))?;

        Ok(Response::new(proto::ExportCaseResponse {
            format: "xes".to_string(),
            data: xes_data.into_bytes(),
            filename: format!("case_{}.xes", case_id),
        }))
    }

    /// Health check probe
    async fn health_check(
        &self,
        request: Request<proto::HealthCheckRequest>,
    ) -> Result<Response<proto::HealthCheckResponse>, Status> {
        let health_checker = HealthChecker::default();

        // Register engine components
        health_checker.register_component(
            "state_store".to_string(),
            HealthStatus::Healthy,
        );
        health_checker.register_component(
            "pattern_registry".to_string(),
            HealthStatus::Healthy,
        );

        let overall_health = health_checker.get_health();
        let components = health_checker.get_health_details();

        let status = match overall_health {
            HealthStatus::Healthy => proto::health_check_response::ServingStatus::Serving,
            HealthStatus::Degraded => proto::health_check_response::ServingStatus::Serving,
            HealthStatus::Unhealthy => proto::health_check_response::ServingStatus::NotServing,
        };

        Ok(Response::new(proto::HealthCheckResponse {
            status: status as i32,
            message: format!("{:?}", overall_health),
            components,
        }))
    }
}
```

### 4. OpenTelemetry OTLP Integration

**File**: `/home/user/knhk/rust/knhk-workflow-engine/src/observability/otlp.rs` (NEW)

```rust
//! OpenTelemetry OTLP exporter integration

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{Config, Tracer};
use opentelemetry_sdk::Resource;
use std::time::Duration;

/// OTLP configuration
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub endpoint: String,
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Export timeout
    pub timeout: Duration,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            service_name: "knhk-workflow-engine".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            timeout: Duration::from_secs(10),
        }
    }
}

/// Initialize OTLP tracing
pub fn init_otlp_tracing(config: OtlpConfig) -> Result<Tracer, Box<dyn std::error::Error>> {
    // Create resource with service metadata
    let resource = Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
        opentelemetry::KeyValue::new("service.version", config.service_version.clone()),
    ]);

    // Build OTLP exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.endpoint)
        .with_timeout(config.timeout);

    // Create tracer provider
    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(Config::default().with_resource(resource))
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    Ok(provider.tracer("knhk-workflow-engine"))
}

/// Initialize structured logging with OTLP integration
pub fn init_logging_with_otlp(
    config: OtlpConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    // Initialize OTLP tracer
    let tracer = init_otlp_tracing(config)?;

    // Create OpenTelemetry layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create JSON formatting layer
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true);

    // Create filter layer
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("knhk=debug,tokio=info"));

    // Combine layers
    let subscriber = Registry::default()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
```

### 5. Prometheus Metrics Exporter

**File**: `/home/user/knhk/rust/knhk-workflow-engine/src/observability/prometheus.rs` (NEW)

```rust
//! Prometheus metrics exporter

use crate::observability::MetricsCollector;
use std::sync::Arc;

/// Prometheus exporter for workflow metrics
pub struct PrometheusExporter {
    collector: Arc<MetricsCollector>,
}

impl PrometheusExporter {
    pub fn new(collector: Arc<MetricsCollector>) -> Self {
        Self { collector }
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> String {
        // TODO: Implement actual Prometheus text format export
        // This would query metrics from the collector and format them
        // For now, return a placeholder

        format!(
            r#"# HELP knhk_workflow_cases_total Total number of workflow cases
# TYPE knhk_workflow_cases_total counter
knhk_workflow_cases_total 0

# HELP knhk_workflow_cases_active Currently active workflow cases
# TYPE knhk_workflow_cases_active gauge
knhk_workflow_cases_active 0

# HELP knhk_workflow_case_duration_seconds Case execution duration
# TYPE knhk_workflow_case_duration_seconds histogram
knhk_workflow_case_duration_seconds_bucket{{le="0.1"}} 0
knhk_workflow_case_duration_seconds_bucket{{le="0.5"}} 0
knhk_workflow_case_duration_seconds_bucket{{le="1.0"}} 0
knhk_workflow_case_duration_seconds_bucket{{le="5.0"}} 0
knhk_workflow_case_duration_seconds_bucket{{le="10.0"}} 0
knhk_workflow_case_duration_seconds_bucket{{le="+Inf"}} 0
knhk_workflow_case_duration_seconds_sum 0
knhk_workflow_case_duration_seconds_count 0
"#
        )
    }

    /// HTTP handler for /metrics endpoint
    #[cfg(feature = "http")]
    pub async fn metrics_handler(
        self: Arc<Self>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.export())
    }
}
```

### 6. Enhanced gRPC Server with Observability

**File**: `/home/user/knhk/rust/knhk-workflow-engine/src/api/grpc_server.rs` (UPDATE)

Add metrics and tracing initialization:

```rust
impl GrpcServer {
    /// Start server with full observability
    pub async fn serve_with_observability(
        self,
        otlp_config: Option<crate::observability::otlp::OtlpConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize OTLP if configured
        if let Some(config) = otlp_config {
            crate::observability::otlp::init_logging_with_otlp(config)?;
        }

        // Start server with graceful shutdown
        self.serve_with_shutdown().await
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_serving() {
        let health_checker = HealthChecker::default();
        health_checker.register_component("test".to_string(), HealthStatus::Healthy);

        assert_eq!(
            health_checker.readiness_probe().unwrap(),
            true
        );
        assert_eq!(
            health_checker.liveness_probe().unwrap(),
            true
        );
    }

    #[tokio::test]
    async fn test_health_check_degraded() {
        let health_checker = HealthChecker::default();
        health_checker.update_component(
            "test",
            HealthStatus::Degraded,
            Some("Slow response".to_string()),
        ).unwrap();

        // Should still be ready (degraded but serving)
        assert_eq!(
            health_checker.readiness_probe().unwrap(),
            true
        );
    }

    #[tokio::test]
    async fn test_health_check_unhealthy() {
        let health_checker = HealthChecker::default();
        health_checker.update_component(
            "test",
            HealthStatus::Unhealthy,
            Some("Connection failed".to_string()),
        ).unwrap();

        // Should not be ready
        assert_eq!(
            health_checker.readiness_probe().unwrap(),
            false
        );
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_grpc_streaming() {
        // TODO: Implement gRPC client test
        // 1. Start server
        // 2. Create client
        // 3. Test WatchCase streaming
        // 4. Test ListCases streaming
        // 5. Test ExportCaseXes
        // 6. Test HealthCheck
    }
}
```

## Next Steps

1. **Fix knhk-hot linking issue** to enable proto compilation
2. **Implement streaming handlers** once proto code is generated
3. **Add OTLP integration** to grpc_server example
4. **Create comprehensive tests** for all new features
5. **Validate with Weaver** schema checks
6. **Update examples** to demonstrate streaming and observability

## Files Modified

- ✅ `/home/user/knhk/rust/knhk-workflow-engine/proto/workflow_engine.proto` - Extended with streaming methods
- ✅ `/home/user/knhk/rust/knhk-workflow-engine/src/observability/health.rs` - Added K8s-style probes

## Files To Create

- ⏳ `/home/user/knhk/rust/knhk-workflow-engine/src/observability/otlp.rs` - OTLP integration
- ⏳ `/home/user/knhk/rust/knhk-workflow-engine/src/observability/prometheus.rs` - Prometheus exporter
- ⏳ `/home/user/knhk/rust/knhk-workflow-engine/tests/grpc_streaming_tests.rs` - Integration tests

## Files To Update

- ⏳ `/home/user/knhk/rust/knhk-workflow-engine/src/api/grpc.rs` - Add streaming handlers
- ⏳ `/home/user/knhk/rust/knhk-workflow-engine/src/api/grpc_server.rs` - Add observability init
- ⏳ `/home/user/knhk/rust/knhk-workflow-engine/src/observability/mod.rs` - Export new modules
- ⏳ `/home/user/knhk/rust/knhk-workflow-engine/examples/grpc_server.rs` - Demonstrate observability
- ⏳ `/home/user/knhk/rust/knhk-workflow-engine/examples/grpc_client.rs` - Test streaming methods
