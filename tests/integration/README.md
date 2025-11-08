# OpenTelemetry Infrastructure for E2E Tests

This directory contains Docker Compose configurations for setting up OpenTelemetry infrastructure required for E2E tests.

## Quick Start

### Start Infrastructure
```bash
./scripts/setup-otel-infrastructure.sh
```

### Check Infrastructure Status
```bash
./scripts/check-otel-infrastructure.sh
```

### Stop Infrastructure
```bash
./scripts/stop-otel-infrastructure.sh
```

## Services

### OTLP Collector
- **HTTP Endpoint**: `http://localhost:4318`
- **gRPC Endpoint**: `http://localhost:4317`
- **Metrics**: `http://localhost:8888/metrics`
- **Config**: `otel-collector-config.yaml`

### Jaeger (Tracing Backend)
- **UI**: `http://localhost:16686`
- **gRPC**: `localhost:14250`
- **HTTP**: `localhost:14268`

### Prometheus (Metrics Backend)
- **UI**: `http://localhost:9090`
- **Config**: `prometheus-config.yaml`

### Grafana (Visualization)
- **UI**: `http://localhost:3000`
- **Username**: `admin`
- **Password**: `admin`

## Running E2E Tests

Once infrastructure is running, you can run E2E tests:

```bash
# knhk-otel
cd rust/knhk-otel
cargo test --test chicago_tdd_e2e_validation --features std -- --ignored
cargo test --test chicago_tdd_collector_validation --features std -- --ignored

# knhk-cli
cd rust/knhk-cli
cargo test --test chicago_tdd_otel_e2e --features otel -- --ignored

# knhk-sidecar
cd rust/knhk-sidecar
cargo test --test chicago_tdd_otel_e2e --features fortune5 -- --ignored

# knhk-etl
cd rust/knhk-etl
cargo test --test chicago_tdd_otel_e2e --features std -- --ignored
```

## Manual Setup

If you prefer to set up infrastructure manually:

```bash
cd tests/integration
docker compose -f docker-compose.otel.yml up -d
```

## Troubleshooting

### Services Not Starting
```bash
# Check logs
docker compose -f docker-compose.otel.yml logs

# Check container status
docker compose -f docker-compose.otel.yml ps
```

### Port Conflicts
If ports are already in use, modify `docker-compose.otel.yml` to use different ports.

### Weaver Not Found
Weaver is optional. Install with:
```bash
cargo install weaver
# Or download from:
# https://github.com/open-telemetry/opentelemetry-collector-contrib/releases
```

## Configuration Files

- `docker-compose.otel.yml` - Main Docker Compose configuration
- `otel-collector-config.yaml` - OTLP collector configuration
- `prometheus-config.yaml` - Prometheus scrape configuration
