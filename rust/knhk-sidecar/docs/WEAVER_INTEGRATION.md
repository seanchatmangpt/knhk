# Weaver Live-Check Integration for KNHK Sidecar

## Overview

The KNHK Sidecar integrates with Weaver.ai live-check for semantic convention validation of OpenTelemetry telemetry. This ensures that all telemetry emitted by the sidecar conforms to OpenTelemetry semantic conventions.

The integration includes:
- **Automatic binary availability checking** - Verifies Weaver is installed before starting
- **Process health monitoring** - Background task monitors Weaver process health every 5 seconds
- **Automatic recovery** - Restarts Weaver if it crashes (with rate limiting)
- **Startup verification** - Health checks verify Weaver is ready before accepting requests
- **Continuous telemetry export** - All gRPC requests export telemetry to Weaver for validation

## Usage

### Environment Variables

The sidecar supports the following environment variables for Weaver configuration:

- `KGC_SIDECAR_WEAVER_ENABLED`: Enable Weaver live-check (default: `false`)
- `KGC_SIDECAR_WEAVER_REGISTRY`: Path to semantic convention registry (optional, defaults to `./registry`)
- `KGC_SIDECAR_WEAVER_OTLP_PORT`: OTLP gRPC port for Weaver (default: `4317`)
- `KGC_SIDECAR_WEAVER_ADMIN_PORT`: Admin HTTP port for Weaver (default: `8080`)
- `KGC_SIDECAR_WEAVER_OUTPUT`: Output directory for Weaver reports (optional, defaults to `./weaver-reports`)

### Example

```bash
export KGC_SIDECAR_WEAVER_ENABLED=true
export KGC_SIDECAR_WEAVER_REGISTRY=./registry
export KGC_SIDECAR_WEAVER_OTLP_PORT=4317
export KGC_SIDECAR_WEAVER_ADMIN_PORT=8080
export KGC_SIDECAR_WEAVER_OUTPUT=./weaver-reports

# Start sidecar
cargo run --bin knhk-sidecar
```

### Verification Script

Before starting the sidecar, verify Weaver is properly installed:

```bash
./scripts/verify-weaver.sh
```

This script checks:
- Weaver binary availability
- Weaver version
- Registry directory existence
- Weaver startup and health check endpoints
- Graceful shutdown

## Architecture

### Process Lifecycle

1. **Configuration Validation**: When `weaver_enabled=true`, the sidecar validates:
   - Weaver binary is available in PATH
   - Registry path exists (if specified)
   - Ports are not already in use
   - OTEL is enabled (required for Weaver)

2. **Startup with Verification**:
   - Weaver process is started as a child process
   - Sidecar waits 2 seconds for Weaver initialization
   - Health check is performed (retries up to 3 times)
   - If health check fails, sidecar fails to start with clear error message

3. **Background Monitoring**:
   - Background task checks Weaver process health every 5 seconds
   - Monitors process status (`try_wait()`)
   - Performs health checks via admin endpoint
   - If process exits unexpectedly, attempts automatic restart
   - Rate limiting: max 5 restarts per minute

4. **Shutdown**:
   - Weaver is stopped gracefully via HTTP admin endpoint (`/stop`)
   - Process is killed if still running after shutdown request
   - Process handle is cleaned up

### Telemetry Export

Telemetry is exported continuously during request handling:

- **Initial Export**: Startup telemetry exported when sidecar starts
- **Request Telemetry**: Each gRPC method exports telemetry:
  - `apply_transaction`: Transaction spans with transaction_id, method attributes
  - `query`: Query spans with query_type, method attributes
  - `validate_graph`: Validation spans with schema_iri, method attributes
  - `evaluate_hook`: Hook evaluation spans with hook_id, rdf_data_size attributes

Each span includes:
- Operation name and type
- Success/failure status
- Latency measurements
- Operation-specific attributes
- Metrics (latency histogram, operation count)

### Automatic Recovery

The sidecar includes automatic recovery mechanisms:

- **Process Monitoring**: Background task monitors Weaver process every 5 seconds
- **Health Checks**: Verifies Weaver admin endpoint is responding
- **Automatic Restart**: Restarts Weaver if process exits unexpectedly
- **Rate Limiting**: Prevents restart loops (max 5 restarts per minute)
- **Exponential Backoff**: Waits between restart attempts

## Troubleshooting

### Weaver Binary Not Found

If Weaver binary is not found in PATH:

```bash
# Install Weaver
cargo install weaver

# Or use installation script (if available)
./scripts/install-weaver.sh

# Verify installation
./scripts/verify-weaver.sh
```

The sidecar will fail to start with a clear error message if Weaver is not found when `weaver_enabled=true`.

### Weaver Process Crashes

If Weaver process crashes:

1. Check sidecar logs for restart attempts
2. Verify Weaver logs in output directory (`./weaver-reports` by default)
3. Check system resources (memory, file descriptors)
4. Verify registry path is valid (if specified)

The sidecar will automatically attempt to restart Weaver up to 5 times per minute. If restart rate limit is exceeded, monitoring stops and an error is logged.

### Health Check Failures

If Weaver health checks fail:

1. Verify Weaver admin port is accessible: `curl http://localhost:8080/health`
2. Check if port is already in use: `lsof -i :8080`
3. Increase startup wait time if Weaver is slow to initialize
4. Check Weaver logs for initialization errors

### Port Conflicts

If Weaver ports are already in use, change them via environment variables:

```bash
export KGC_SIDECAR_WEAVER_OTLP_PORT=14317
export KGC_SIDECAR_WEAVER_ADMIN_PORT=18080
```

The sidecar validates ports are available before starting Weaver.

### Configuration Validation Errors

If configuration validation fails:

- **Weaver requires OTEL**: Set `enable_otel=true` when `weaver_enabled=true`
- **Registry path invalid**: Ensure registry path exists and is a directory
- **Ports in use**: Change ports or stop conflicting services

## Monitoring

### Health Check Endpoints

Weaver admin endpoints (default: `http://localhost:8080`):

- `/health` - Health check endpoint
- `/status` - Status endpoint
- `/stop` - Graceful shutdown endpoint

### Logs

Sidecar logs include:

- Weaver startup/shutdown events
- Health check results
- Process monitoring status
- Restart attempts and failures
- Telemetry export errors

### Metrics

Telemetry exported to Weaver includes:

- `knhk.sidecar.{operation}.latency` - Operation latency histogram
- `knhk.sidecar.{operation}.count` - Operation count
- `knhk.warm_path.operations.latency` - Warm path latency
- `knhk.warm_path.operations.count` - Warm path operation count

## Best Practices

1. **Always verify Weaver installation** before starting sidecar:
   ```bash
   ./scripts/verify-weaver.sh
   ```

2. **Use dedicated ports** in production to avoid conflicts

3. **Monitor Weaver reports** in output directory for validation issues

4. **Set appropriate inactivity timeout** for Weaver (default: 3600 seconds)

5. **Enable Weaver in development** to catch semantic convention violations early

6. **Review restart logs** if Weaver crashes frequently - may indicate resource issues
