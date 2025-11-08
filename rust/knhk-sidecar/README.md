# KGC Sidecar

KGC Sidecar gRPC service for local proxy, batching, retries, and circuit-breaking.

## Features

- **gRPC Service**: Exposes KNHK operations via gRPC
- **Request Batching**: Groups multiple RDF operations for efficiency
- **Retry Logic**: Exponential backoff with idempotence support (μ∘μ = μ)
- **Circuit Breaker**: Prevents cascading failures
- **Health Checks**: Service health monitoring
- **OTEL Integration**: Observability and metrics
- **Weaver Live-Check**: Telemetry validation (when `otel` feature enabled)

## Usage

### Simple Usage (Recommended)

The easiest way to run the sidecar is using the `run()` function:

```rust
use knhk_sidecar::{run, SidecarConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SidecarConfig::from_env();
    run(config).await
}
```

Or from the command line:

```bash
# Using environment variables
export KGC_SIDECAR_ADDRESS="0.0.0.0:50051"
knhk-sidecar

# Or with custom config
KGC_SIDECAR_ADDRESS=localhost:50051 KGC_SIDECAR_TLS_ENABLED=false knhk-sidecar
```

The `run()` function automatically:
- Initializes tracing
- Starts Weaver live-check (if enabled and `otel` feature is active)
- Creates metrics collector and health checker
- Configures client with retry and circuit breaker settings
- Starts the gRPC server
- Handles graceful shutdown

### Advanced Usage

For more control, you can manually create and configure components:

```rust
use knhk_sidecar::{SidecarServer, SidecarClient, SidecarConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SidecarConfig::from_env();
    
    // Create components manually
    let metrics = Arc::new(MetricsCollector::new(1000));
    let health = Arc::new(HealthChecker::new());
    let client = SidecarClient::new(client_config, Arc::clone(&metrics)).await?;
    
    let server = SidecarServer::new(server_config, client, metrics, health).await?;
    server.start().await?;
    
    Ok(())
}
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
- `KGC_SIDECAR_WEAVER_ENABLED`: Enable Weaver live-check (default: `false`, requires `otel` feature)
- `KGC_SIDECAR_WEAVER_REGISTRY`: Weaver registry path (optional)
- `KGC_SIDECAR_WEAVER_OTLP_PORT`: Weaver OTLP port (default: `4317`)
- `KGC_SIDECAR_WEAVER_ADMIN_PORT`: Weaver admin port (default: `8080`)
- `KGC_SIDECAR_WEAVER_OUTPUT`: Weaver output directory (default: `./weaver-reports`)

### Weaver Live-Check Integration

When the `otel` feature is enabled, the sidecar can automatically start Weaver live-check for telemetry validation:

```bash
# Enable Weaver
export KGC_SIDECAR_WEAVER_ENABLED=true
export KGC_SIDECAR_WEAVER_REGISTRY=./registry
knhk-sidecar
```

Weaver will:
- Validate all telemetry against semantic conventions
- Generate validation reports in `./weaver-reports/`
- Stop gracefully when the sidecar shuts down

See `docs/WEAVER_INTEGRATION.md` for detailed Weaver configuration.

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

- ✅ Proper error handling (`Result<T, E>`)
- ✅ Circuit breaker pattern
- ✅ Retry logic with exponential backoff
- ✅ Health checks
- ✅ Metrics collection
- ✅ Server startup implementation (fixed - server actually starts)
- ✅ Weaver live-check integration for telemetry validation
- ✅ Basic mTLS support
- ⚠️ Feature-gated implementations: Some gRPC integrations pending warm orchestrator service

## Fortune 5 Readiness

For Fortune 5 enterprise deployment requirements, see [Fortune 5 Readiness Plan](docs/FORTUNE5_READINESS_PLAN.md).

**Current Status**: Basic production-ready features complete. Fortune 5 features (SPIFFE/SPIRE, HSM/KMS, multi-region) planned for implementation.

**Fortune 5 Requirements**:
- ⏳ SPIFFE/SPIRE integration (planned)
- ⏳ HSM/KMS integration (planned)
- ⏳ Automatic key rotation ≤24h (planned)
- ⏳ Multi-region support (planned)
- ⏳ Legal hold functionality (planned)
- ⏳ SLO-based admission control (planned)
- ⏳ Formal promotion gates (planned)

