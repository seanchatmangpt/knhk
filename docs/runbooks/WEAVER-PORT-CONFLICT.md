# Weaver Port 4318 Conflict Resolution Runbook

## Problem

`weaver registry live-check` requires an OTLP collector on port 4318, but this port is occupied by:
- Docker Desktop's built-in OTLP telemetry (`user-analytics.otlp.grpc.sock`)
- Test OTEL collectors from integration tests
- Other development services

## Root Cause

Port 4318 is the default OTLP HTTP port. Multiple services attempt to bind to it:
1. Docker Desktop analytics (primary conflict)
2. Test containers from `tests/integration/docker-compose.yml`
3. Development OTEL collectors

## Solution

Use port 4319 for KNHK's OTLP collector during development and testing.

### Step 1: Check Port Status

```bash
# Identify what's using port 4318
lsof -i :4318

# Expected output: Docker Desktop (com.docker PID)
# COMMAND     PID USER   FD   TYPE   DEVICE SIZE/OFF NODE NAME
# com.docke 21539  sac  220u  IPv6 0x... 0t0  TCP *:4318 (LISTEN)
```

### Step 2: Configure KNHK to Use Port 4319

Set environment variable for OTLP endpoint:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4319
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4319
```

### Step 3: Start Test OTEL Collector on Port 4319

```bash
# Option A: Docker Compose (recommended)
cd tests/integration
docker-compose -f docker-compose-weaver.yml up -d

# Option B: Docker run
docker run -d \
  --name knhk-weaver-otel \
  -p 4317:4317 \
  -p 4319:4318 \
  -v $(pwd)/tests/integration/otel-collector-config.yaml:/etc/otelcol/config.yaml \
  otel/opentelemetry-collector:latest \
  --config=/etc/otelcol/config.yaml
```

### Step 4: Run Weaver Live-Check

```bash
# Start KNHK application (emits telemetry to localhost:4319)
cargo run --example emit_telemetry &
PID=$!

# Wait for startup
sleep 2

# Run live-check
weaver registry live-check --registry registry/

# Cleanup
kill $PID
```

### Step 5: CI/CD Configuration

Add to GitHub Actions workflow:

```yaml
- name: Start OTEL Collector on Port 4319
  run: |
    docker run -d \
      --name otel-collector \
      -p 4319:4318 \
      otel/opentelemetry-collector:latest

    echo "OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4319" >> $GITHUB_ENV

- name: Run Weaver Validation
  run: |
    cargo run --example emit_telemetry &
    sleep 2
    weaver registry live-check --registry registry/
```

## Prevention

### Docker Compose for Weaver Testing

Create `tests/integration/docker-compose-weaver.yml`:

```yaml
version: '3.8'

services:
  otel-collector:
    image: otel/opentelemetry-collector:latest
    container_name: knhk-weaver-otel
    ports:
      - "4317:4317"  # gRPC
      - "4319:4318"  # HTTP (mapped to avoid conflict)
    volumes:
      - ./otel-collector-config.yaml:/etc/otelcol/config.yaml
    command: ["--config=/etc/otelcol/config.yaml"]
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:4318/v1/traces"]
      interval: 5s
      timeout: 3s
      retries: 3
```

### Automated Check Script

Create `scripts/check-weaver-port.sh`:

```bash
#!/bin/bash
set -e

# Check if port 4318 is available
if lsof -i :4318 > /dev/null 2>&1; then
    echo "❌ Port 4318 is in use"
    echo "Using alternative port 4319 for Weaver testing"
    export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4319
    PORT=4319
else
    echo "✅ Port 4318 is available"
    export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
    PORT=4318
fi

# Start collector on appropriate port
docker run -d \
  --name knhk-weaver-otel \
  -p 4317:4317 \
  -p ${PORT}:4318 \
  otel/opentelemetry-collector:latest

echo "OTEL collector started on port ${PORT}"
```

## Troubleshooting

### Port Still Occupied After Stopping Containers

```bash
# Kill all processes on port 4318
lsof -ti :4318 | xargs kill -9

# If Docker Desktop is the issue, restart it
osascript -e 'quit app "Docker"'
open -a Docker
```

### Weaver Live-Check Fails with Connection Refused

```bash
# Verify collector is running
docker ps --filter name=otel

# Check collector logs
docker logs knhk-weaver-otel

# Test OTLP endpoint
curl -v http://localhost:4319/v1/traces \
  -H "Content-Type: application/json" \
  -d '{"resourceSpans":[]}'
```

### Environment Variable Not Applied

```bash
# Verify environment
echo $OTEL_EXPORTER_OTLP_ENDPOINT

# Set globally for session
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4319

# Or per-command
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4319 cargo run
```

## Related Documentation

- [OpenTelemetry Collector Configuration](https://opentelemetry.io/docs/collector/configuration/)
- [OTLP Exporter Specification](https://opentelemetry.io/docs/specs/otlp/)
- [Weaver Live-Check Documentation](https://github.com/open-telemetry/weaver)

## Success Criteria

- [ ] Port conflict identified and documented
- [ ] OTEL collector running on alternative port (4319)
- [ ] `weaver registry live-check` passes
- [ ] CI/CD pipeline updated
- [ ] Automated checks in place
