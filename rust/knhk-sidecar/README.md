# KGC Sidecar

Local proxy service for enterprise apps with batching, retries, and circuit-breaking.

## Overview

The KGC Sidecar is a gRPC service that acts as a local proxy for enterprise applications. It provides:

- **Batching**: Accumulates requests and flushes when batch_size=8 or timeout=100ms
- **Retries**: Exponential backoff with max_attempts=3 (idempotent operations)
- **Circuit Breaking**: Protects warm orchestrator connection
- **TLS**: Optional mTLS support
- **Observability**: OTEL metrics and traces

## Architecture

```
Enterprise App → KGC Sidecar (localhost:50051) → Warm Orchestrator (knhk-warm:50052)
```

The sidecar:
1. Receives gRPC requests from apps (Transaction, Query)
2. Batches transactions (up to 8, or 100ms timeout)
3. Sends batched requests to warm orchestrator with retry logic
4. Protects warm orchestrator with circuit breaker
5. Returns responses to apps

## Usage

### Configuration

Environment variables:
- `LISTEN_ADDR`: Server listen address (default: `0.0.0.0:50051`)
- `WARM_ORCHESTRATOR_ENDPOINT`: Warm orchestrator endpoint (default: `http://localhost:50052`)
- `BATCH_SIZE`: Batch size (default: `8`, guard: ≤8)
- `BATCH_TIMEOUT_MS`: Batch timeout in milliseconds (default: `100`)

### Running

```bash
# Set environment variables
export LISTEN_ADDR="0.0.0.0:50051"
export WARM_ORCHESTRATOR_ENDPOINT="http://knhk-warm:50052"
export BATCH_SIZE=8
export BATCH_TIMEOUT_MS=100

# Run sidecar
cargo run --bin knhk-sidecar
```

### gRPC Service

The sidecar exposes a gRPC service (`KgcSidecar`) with methods:

- `SubmitTransaction(Transaction) -> TransactionResponse`
- `SubmitQuery(Query) -> QueryResponse`
- `HealthCheck(HealthCheckRequest) -> HealthCheckResponse`

See `proto/knhk_sidecar.proto` for complete service definition.

## Components

### Batching Manager (`src/batching.rs`)

Accumulates transactions and flushes when:
- Batch size reaches `max_batch_size` (default: 8)
- Timeout exceeds `batch_timeout_ms` (default: 100ms)

**Guard Constraint**: `max_batch_size ≤ 8` (Chatman Constant)

### Retry Logic (`src/retry.rs`)

Exponential backoff:
- Attempt 0: immediate
- Attempt 1: 100ms
- Attempt 2: 200ms
- Attempt 3: 400ms

**Idempotency**: Retries are safe because `A = μ(O)` ensures idempotent operations.

### Circuit Breaker (`src/circuit_breaker.rs`)

Protects warm orchestrator connection:
- **Closed**: Normal operation
- **Open**: Failing, requests rejected immediately
- **HalfOpen**: Testing recovery

**Configuration**:
- `failure_threshold`: 5 failures before opening
- `success_threshold`: 1 success to close from half-open
- `reset_timeout_ms`: 60000ms (60 seconds)

### Warm Client (`src/warm_client.rs`)

gRPC client to warm orchestrator:
- Connection pooling
- TLS support (optional)
- Circuit breaker protection

**Note**: Requires warm orchestrator to expose matching gRPC service.

## Building

```bash
# Build sidecar
cd rust/knhk-sidecar
cargo build --release

# Generate protobuf code
cargo build
```

The build process will:
1. Generate Rust code from `proto/knhk_sidecar.proto`
2. Compile the sidecar binary

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_test
```

## Kubernetes Deployment

See `k8s/` directory for Kubernetes deployment manifests:

- `daemonset-sidecar.yaml`: DaemonSet for sidecar (runs on all nodes)
- `app-sidecar-pod.yaml`: Example pod with Enterprise App + KGC Sidecar

## Error Handling

All operations use `Result<T, SidecarError>`:
- No `unwrap()` or `expect()` in production paths
- Proper error propagation
- Guard validation (max_batch_size ≤ 8)

## Observability

OTEL integration:
- Spans for each request
- Metrics: `batch_size`, `batch_flush_count`, `retry_count`, `circuit_breaker_state`
- Traces: Request flow through sidecar

## Future Work

- [ ] Implement warm orchestrator gRPC service
- [ ] Add TLS/mTLS support
- [ ] Add request rate limiting
- [ ] Add metrics endpoint (Prometheus)
- [ ] Add distributed tracing (Jaeger)

