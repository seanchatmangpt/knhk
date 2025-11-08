# KNHK Sidecar

KGC Sidecar is a Rust/gRPC proxy service for enterprise applications, providing resilience patterns (batching, retries, circuit-breaking) and secure communication (TLS) between apps and the KNHK warm orchestrator.

## Architecture

The sidecar acts as a local proxy that:

- **Accepts gRPC requests** from local enterprise apps
- **Batches requests** when possible (time and size windows)
- **Retries failed requests** with exponential backoff
- **Implements circuit breaker** pattern to prevent cascading failures
- **Forwards requests** to warm orchestrator via gRPC
- **Supports TLS/mTLS** for secure communication
- **Provides health checks** and metrics

## Features

### Request Batching

- **Time-based batching**: Collects requests within a time window (default: 10ms)
- **Size-based batching**: Groups requests up to maximum batch size (default: 100)
- **Automatic timeout**: Flushes batches when timeout is reached

### Retry Logic

- **Exponential backoff**: Initial delay 100ms, max delay 5s, multiplier 2.0
- **Max retries**: Configurable (default: 3)
- **Smart retry**: Only retries transient errors (network, timeout, circuit breaker)
- **No retry on validation errors**: Guard violations and validation errors are not retried

### Circuit Breaker

- **Three states**: Closed → Open → HalfOpen
- **Failure threshold**: Configurable (default: 5 failures)
- **Reset timeout**: Configurable (default: 60s)
- **Per-endpoint**: Each endpoint has its own circuit breaker

### TLS Support

- **TLS 1.2+**: Secure communication
- **mTLS support**: Mutual TLS for client authentication
- **Certificate validation**: Validates certificates on both sides

### Health Checks

- **Liveness probe**: Server is running
- **Readiness probe**: Can connect to warm orchestrator
- **Component status**: Individual component health tracking

### Metrics

- **Request counts**: Total, success, failure
- **Latency metrics**: P50, P95, P99
- **Batch metrics**: Total batches, average size, max size
- **Circuit breaker metrics**: State, failure count, success count
- **Retry metrics**: Total retries, successful, failed
- **OTEL integration**: Exports metrics to OTEL backend

## Configuration

### Configuration File

Create a `knhk-sidecar.toml` file:

```toml
[server]
bind_address = "127.0.0.1:50051"
batch_window_ms = 10
max_batch_size = 100

[client]
warm_orchestrator_url = "http://localhost:50052"
connection_timeout_ms = 5000
request_timeout_ms = 30000

[retry]
max_retries = 3
initial_delay_ms = 100
max_delay_ms = 5000
multiplier = 2.0

[circuit_breaker]
failure_threshold = 5
reset_timeout_ms = 60000

[tls]
enabled = true
cert_file = "/path/to/cert.pem"
key_file = "/path/to/key.pem"
ca_file = "/path/to/ca.pem"
mtls_enabled = false
```

### Environment Variables

Configuration can be overridden with environment variables:

- `KNHK_SIDECAR_SERVER_BIND_ADDRESS`
- `KNHK_SIDECAR_CLIENT_WARM_ORCHESTRATOR_URL`
- `KNHK_SIDECAR_RETRY_MAX_RETRIES`
- etc.

## API Reference

### gRPC Services

#### KgcService

- `ExecuteTransaction` - Execute RDF transaction with hooks (supports JSON and protobuf)
- `ValidateGraph` - Validate RDF graph against schema (supports JSON and Turtle)
- `EvaluateHook` - Evaluate a single hook (supports JSON and Turtle)
- `QueryPolicy` - Query policy packs (supports JSON and Turtle)
- `HealthCheck` - Health check endpoint
- `GetMetrics` - Get sidecar metrics

### JSON Format Support

The sidecar now supports JSON as the primary external format, using simdjson for fast parsing (GB/s speeds).

**Transaction Request (JSON)**:
```json
{
  "additions": [
    {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
  ],
  "removals": []
}
```

**Query Request (JSON)**:
```json
{
  "query_type": 1,
  "query": "SELECT ?s WHERE { ?s ?p ?o }",
  "data_format": "json",
  "json_data": "[{\"s\": \"http://example.org/s\", \"p\": \"http://example.org/p\", \"o\": \"http://example.org/o\"}]"
}
```

### Performance Improvements

- **JSON Parsing**: 2-3 GB/s with simdjson (vs ~100-200 MB/s with serde_json)
- **Sidecar Latency**: <10ms for JSON parsing (vs ~50ms for Turtle)
- **Kafka Connector**: 10-50x faster JSON parsing
- **Overall ETL Throughput**: 2-5x improvement for JSON-heavy workloads

### Request/Response Types

See `proto/kgc-sidecar.proto` for complete protocol buffer definitions. JSON format is preferred for new integrations.

## Usage

### Basic Usage

The recommended way to run the sidecar is using the `run()` function:

```rust
use knhk_sidecar::{run, SidecarConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment variables
    let config = SidecarConfig::from_env();
    
    // Run the sidecar (handles all initialization and startup)
    run(config).await
}
```

The `run()` function automatically:
- Initializes tracing
- Starts Weaver live-check if enabled (requires `otel` feature)
- Creates metrics collector and health checker
- Configures client with retry and circuit breaker settings
- Starts the gRPC server
- Handles graceful shutdown

### Advanced Usage (Manual Setup)

For more control, you can manually create and configure components:

```rust
use knhk_sidecar::{SidecarServer, SidecarClient, SidecarConfig, MetricsCollector, HealthChecker};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = SidecarConfig::from_env();
    
    // Create metrics collector
    let metrics = Arc::new(MetricsCollector::new(1000));
    
    // Create health checker
    let health = Arc::new(HealthChecker::new());
    
    // Create client configuration
    let mut client_config = crate::client::ClientConfig::default();
    client_config.warm_orchestrator_url = std::env::var("KGC_SIDECAR_CLIENT_WARM_ORCHESTRATOR_URL")
        .unwrap_or_else(|_| "http://localhost:50052".to_string());
    client_config.request_timeout_ms = config.request_timeout_ms;
    client_config.retry_config.max_retries = config.retry_max_attempts;
    client_config.retry_config.initial_delay_ms = config.retry_initial_delay_ms;
    client_config.retry_config.max_delay_ms = config.retry_max_delay_ms;
    client_config.circuit_breaker_threshold = config.circuit_breaker_failure_threshold;
    client_config.circuit_breaker_reset_ms = config.circuit_breaker_reset_timeout_ms;
    
    // Create client
    let client = SidecarClient::new(client_config, Arc::clone(&metrics)).await?;
    
    // Create server configuration
    let server_config = ServerConfig {
        bind_address: config.listen_address.clone(),
        batch_config: BatchConfig {
            max_size: config.batch_size,
            timeout: config.batch_timeout(),
        },
        tls_config: TlsConfig {
            enabled: config.tls_enabled,
            cert_file: config.tls_cert_path.clone().unwrap_or_default(),
            key_file: config.tls_key_path.clone().unwrap_or_default(),
            ca_file: config.tls_ca_path.clone(),
        },
    };
    
    // Create server
    let server = SidecarServer::new(
        server_config,
        client,
        Arc::clone(&metrics),
        Arc::clone(&health),
    ).await?;
    
    // Start server (this blocks until shutdown)
    server.start().await?;
    
    Ok(())
}
```

### Execute Transaction

```rust
let result = client.execute_transaction(rdf_delta).await?;
```

### Validate Graph

```rust
let valid = client.validate_graph(graph, schema_iri).await?;
```

### Evaluate Hook

```rust
let result = client.evaluate_hook(hook_id, input_data).await?;
```

## Deployment

### Kubernetes

The sidecar can be deployed as a sidecar container in Kubernetes pods:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-sidecar
spec:
  containers:
  - name: app
    image: enterprise-app:latest
  - name: knhk-sidecar
    image: knhk-sidecar:latest
    ports:
    - containerPort: 50051
    env:
    - name: KNHK_SIDECAR_CLIENT_WARM_ORCHESTRATOR_URL
      value: "http://knhk-warm:50052"
```

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/knhk-sidecar /usr/local/bin/
CMD ["knhk-sidecar"]
```

## Error Handling

All operations return `SidecarResult<T>` which is `Result<T, SidecarError>`.

### Error Types

- `NetworkError` - Network/connection errors (retryable)
- `ValidationError` - Request validation errors (non-retryable)
- `TimeoutError` - Request timeout (retryable)
- `CircuitBreakerOpen` - Circuit breaker is open (retryable)
- `TlsError` - TLS handshake/configuration errors (non-retryable)
- `BatchError` - Batching errors (non-retryable)
- `RetryExhausted` - Max retries exceeded (non-retryable)

### Retryable Errors

Use `is_retryable_error()` to check if an error should be retried:

```rust
if is_retryable_error(&error) {
    // Retry logic
}
```

## Performance

### Latency

- **P50**: < 10ms (batched requests)
- **P95**: < 50ms
- **P99**: < 100ms

### Throughput

- **Max batch size**: 100 requests
- **Batch window**: 10ms
- **Max requests/second**: ~10,000 (with batching)

## Testing

### Unit Tests

```bash
cargo test --lib
```

### Integration Tests

```bash
cargo test --test integration
```

## Examples

See `examples/` directory for complete working examples.

## License

MIT OR Apache-2.0

