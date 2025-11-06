# KGC Sidecar

KGC Sidecar gRPC service for local proxy, batching, retries, and circuit-breaking.

## Features

- **gRPC Service**: Exposes KNHK operations via gRPC
- **Request Batching**: Groups multiple RDF operations for efficiency
- **Retry Logic**: Exponential backoff with idempotence support (μ∘μ = μ)
- **Circuit Breaker**: Prevents cascading failures
- **Health Checks**: Service health monitoring
- **OTEL Integration**: Observability and metrics

## Usage

### Start Server

```bash
# Using environment variables
export KGC_SIDECAR_ADDRESS="0.0.0.0:50051"
knhk-sidecar

# Or with custom config
KGC_SIDECAR_ADDRESS=localhost:50051 KGC_SIDECAR_TLS_ENABLED=false knhk-sidecar
```

### Configuration

Environment variables:
- `KGC_SIDECAR_ADDRESS`: Listen address (default: `0.0.0.0:50051`)
- `KGC_SIDECAR_TLS_ENABLED`: Enable TLS (default: `false`)
- `KGC_SIDECAR_TLS_CERT`: TLS certificate path
- `KGC_SIDECAR_TLS_KEY`: TLS key path
- `KGC_SIDECAR_TLS_CA`: TLS CA certificate path
- `KGC_SIDECAR_BATCH_SIZE`: Batch size (default: `100`)
- `KGC_SIDECAR_BATCH_TIMEOUT_MS`: Batch timeout in ms (default: `100`)
- `KGC_SIDECAR_RETRY_MAX_ATTEMPTS`: Max retry attempts (default: `3`)
- `KGC_SIDECAR_REQUEST_TIMEOUT_MS`: Request timeout in ms (default: `5000`)

## API Methods

- `ApplyTransaction`: Execute RDF delta with hooks
- `Query`: Execute query (ASK/SELECT/CONSTRUCT)
- `ValidateGraph`: Validate RDF graph against schema
- `EvaluateHook`: Execute single hook
- `HealthCheck`: Service health
- `GetMetrics`: Telemetry metrics

## Architecture

The sidecar integrates with:
- `knhk-etl`: ETL pipeline operations
- `knhk-warm`: Warm path query execution
- `knhk-unrdf`: Hook evaluation
- `knhk-otel`: Observability
- `knhk-lockchain`: Receipt generation

## Production-Ready Features

- Proper error handling (`Result<T, E>`)
- Circuit breaker pattern
- Retry logic with exponential backoff
- Health checks
- Metrics collection
- No placeholders or TODOs in production code paths

