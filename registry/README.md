# KNHK Sidecar Weaver Registry

This directory contains semantic convention schemas for Weaver live-check validation of sidecar telemetry.

## Usage

To enable Weaver live-check for the sidecar:

```bash
export KGC_SIDECAR_WEAVER_ENABLED=true
export KGC_SIDECAR_WEAVER_REGISTRY=./registry
export KGC_SIDECAR_WEAVER_OTLP_PORT=4317
export KGC_SIDECAR_WEAVER_ADMIN_PORT=8080
export KGC_SIDECAR_WEAVER_OUTPUT=./weaver-reports

knhk-sidecar
```

## Schema Structure

The registry should follow Weaver's semantic convention format. See:
- https://github.com/open-telemetry/opentelemetry-rust/tree/main/opentelemetry-semantic-conventions

## Sidecar Telemetry Conventions

The sidecar emits telemetry with the following conventions:

### Spans
- `knhk.sidecar.start` - Sidecar server startup
- `knhk.sidecar.request` - gRPC request handling
- `knhk.sidecar.batch` - Request batching operations
- `knhk.sidecar.retry` - Retry operations
- `knhk.sidecar.circuit_breaker` - Circuit breaker state changes

### Metrics
- `knhk.sidecar.requests.total` - Total requests
- `knhk.sidecar.requests.success` - Successful requests
- `knhk.sidecar.requests.failure` - Failed requests
- `knhk.sidecar.latency.p50_ms` - P50 latency
- `knhk.sidecar.latency.p95_ms` - P95 latency
- `knhk.sidecar.latency.p99_ms` - P99 latency
- `knhk.sidecar.batch.size` - Batch sizes
- `knhk.sidecar.retry.count` - Retry counts

### Attributes
- `knhk.operation.name` - Operation name
- `knhk.operation.type` - Operation type (system, request, batch, etc.)
- `knhk.sidecar.address` - Sidecar bind address
- `knhk.sidecar.method` - gRPC method name
- `knhk.sidecar.batch.size` - Batch size
- `knhk.sidecar.retry.attempt` - Retry attempt number

