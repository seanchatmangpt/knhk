# KNHK v1.0 Infrastructure & Backend Requirements

## Executive Summary

This document defines the infrastructure validation criteria for KNHK v1.0 Definition of Done. Based on 80/20 principles, these requirements focus on the critical 20% of infrastructure that ensures 80% of operational reliability.

## 1. Docker Container Validation

### 1.1 Core Requirements

**MANDATORY:**
- [ ] Docker daemon accessible and running (`docker info` succeeds)
- [ ] OTEL Collector container starts successfully
- [ ] Jaeger backend container starts successfully
- [ ] Container health checks pass within 30s
- [ ] All containers accessible via configured networks

**VALIDATION COMMANDS:**
```bash
# Docker availability
docker info >/dev/null 2>&1 || exit 1

# OTEL stack startup
docker-compose -f tests/integration/docker-compose.otel.yml up -d

# Health check verification
docker ps --filter "health=healthy" --filter "name=knhk-otel-collector"
docker ps --filter "health=healthy" --filter "name=knhk-jaeger"
```

### 1.2 Container Configuration Standards

**OTEL Collector Requirements:**
- Image: `otel/opentelemetry-collector:latest`
- Required ports exposed:
  - 4317 (OTLP gRPC)
  - 4318 (OTLP HTTP)
  - 8888 (Prometheus metrics)
  - 8889 (Prometheus exporter)
- Health check: `wget --spider -q http://localhost:8888/metrics`
- Startup timeout: ≤10s
- Health check interval: 10s

**Jaeger Backend Requirements:**
- Image: `jaegertracing/all-in-one:latest`
- Required ports exposed:
  - 16686 (Jaeger UI)
  - 14250 (Jaeger gRPC)
  - 14268 (Jaeger HTTP)
- OTLP enabled via `COLLECTOR_OTLP_ENABLED=true`
- Health check: `wget --spider -q http://localhost:16686`

## 2. OTLP Telemetry Configuration

### 2.1 Collector Configuration Standards

**Receivers (MANDATORY):**
```yaml
receivers:
  otlp:
    protocols:
      http:
        endpoint: 0.0.0.0:4318  # Must be accessible
      grpc:
        endpoint: 0.0.0.0:4317  # Must be accessible
```

**Processors (MANDATORY):**
```yaml
processors:
  batch:
    timeout: 1s              # ≤1s for low latency
    send_batch_size: 1024

  memory_limiter:
    limit_mib: 512           # Prevent OOM
    spike_limit_mib: 128
    check_interval: 1s
```

**Exporters (MANDATORY for validation):**
```yaml
exporters:
  logging:
    loglevel: info           # Debug telemetry flow

  otlp/jaeger:
    endpoint: jaeger:14250   # Backend integration
    tls:
      insecure: true         # Test environment only
```

### 2.2 Pipeline Configuration

**MANDATORY Pipelines:**
```yaml
service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [memory_limiter, batch]
      exporters: [logging, otlp/jaeger]

    metrics:
      receivers: [otlp]
      processors: [memory_limiter, batch]
      exporters: [logging, prometheus]

    logs:
      receivers: [otlp]
      processors: [memory_limiter, batch]
      exporters: [logging]
```

**VALIDATION:**
- All three signal types (traces, metrics, logs) configured
- Batch processor timeout ≤1s (performance requirement)
- Memory limiter enabled (stability requirement)
- Logging exporter enabled (debugging requirement)

## 3. Testcontainers Integration

### 3.1 Dependency Requirements

**Cargo.toml Configuration:**
```toml
[dependencies]
testcontainers = { version = "0.25", features = ["blocking"] }
testcontainers-modules = { version = "0.13", features = ["kafka", "postgres"] }
```

**VALIDATION:**
- Compatible version alignment (0.25 + 0.13)
- Blocking feature enabled (synchronous tests)
- Required modules available (kafka, postgres)

### 3.2 Runtime Requirements

**Container Lifecycle:**
- [ ] Containers spawn within 5s
- [ ] Containers expose correct ports
- [ ] Containers clean up on test completion
- [ ] No port conflicts (dynamic port allocation)
- [ ] Container logs accessible for debugging

**Example Validation:**
```rust
use testcontainers::ImageExt;
use testcontainers_modules::kafka::Kafka;

let kafka = Kafka::default()
    .with_tag("latest")
    .start();

assert!(kafka.get_host_port_ipv4(9092).is_ok());
```

## 4. Weaver Installation Verification

### 4.1 Binary Availability

**MANDATORY:**
- [ ] `weaver` command in PATH or ~/.cargo/bin
- [ ] Version ≥0.16.0 (current: 0.16.1)
- [ ] `weaver --version` executes successfully
- [ ] `weaver registry check` available
- [ ] `weaver registry live-check` available

**VALIDATION COMMANDS:**
```bash
# Binary existence
which weaver || exit 1

# Version check
weaver --version | grep -E "0\.(1[6-9]|[2-9][0-9])" || exit 1

# Subcommand availability
weaver registry --help | grep -q "check" || exit 1
weaver registry --help | grep -q "live-check" || exit 1
```

### 4.2 Registry Validation Capability

**Schema Validation:**
```bash
# Static schema validation (MANDATORY)
weaver registry check -r registry/

# Expected output: ✅ All checks passed
```

**Live Telemetry Validation:**
```bash
# Runtime telemetry validation (MANDATORY)
weaver registry live-check --registry registry/

# Expected: Validates actual emitted telemetry against schema
```

## 5. Environment Setup & Verification

### 5.1 Environment Variables

**MANDATORY for OTLP:**
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
export OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
export OTEL_SERVICE_NAME=knhk-test
export OTEL_RESOURCE_ATTRIBUTES=service.name=knhk-test
```

**OPTIONAL for debugging:**
```bash
export OTEL_LOG_LEVEL=debug
export OTEL_TRACES_EXPORTER=otlp
export OTEL_METRICS_EXPORTER=otlp
export OTEL_LOGS_EXPORTER=otlp
```

### 5.2 Network Validation

**Docker Network Requirements:**
```yaml
networks:
  knhk-test-network:
    driver: bridge
```

**VALIDATION:**
- [ ] Network created before containers start
- [ ] All containers on same network
- [ ] Container-to-container communication works
- [ ] Host-to-container communication works

**Test Command:**
```bash
# Network connectivity test
docker network inspect knhk-test-network
docker exec knhk-otel-collector wget -O- http://jaeger:14250
```

### 5.3 Port Availability

**MANDATORY Port Checks:**
```bash
# Check ports before starting containers
lsof -i :4317 || echo "4317 available"  # OTLP gRPC
lsof -i :4318 || echo "4318 available"  # OTLP HTTP
lsof -i :16686 || echo "16686 available" # Jaeger UI
```

## 6. Integration Test Infrastructure

### 6.1 Test Execution Requirements

**Script Availability:**
- [ ] `tests/integration/docker_test.sh` executable
- [ ] Docker Compose files validated
- [ ] Integration test binaries build successfully

**VALIDATION:**
```bash
# Build integration tests
cd rust/knhk-integration-tests && cargo build --release

# Verify test script
test -x tests/integration/docker_test.sh || exit 1
```

### 6.2 CI/CD Compatibility

**MANDATORY for production:**
- [ ] Tests run in isolated containers
- [ ] No persistent state between runs
- [ ] Cleanup on success and failure
- [ ] Exit codes properly propagated
- [ ] Logs captured for debugging

**Example CI Check:**
```bash
# Run integration test suite
docker-compose -f tests/integration/docker-compose.otel.yml up -d
cargo test --package knhk-integration-tests
docker-compose -f tests/integration/docker-compose.otel.yml down -v
```

## 7. Performance Requirements

### 7.1 Startup Performance

**Container Startup:**
- OTEL Collector: ≤10s to healthy
- Jaeger: ≤10s to healthy
- Total stack: ≤30s to fully operational

**VALIDATION:**
```bash
start_time=$(date +%s)
docker-compose up -d
# Wait for health checks
docker-compose ps --filter "health=healthy" | grep -q "knhk-otel-collector"
end_time=$(date +%s)
duration=$((end_time - start_time))
test $duration -le 30 || exit 1
```

### 7.2 Telemetry Pipeline Latency

**MANDATORY:**
- Batch processor timeout: ≤1s
- End-to-end telemetry latency: ≤2s
- Memory limiter overhead: <5%

## 8. Production Readiness Criteria

### 8.1 Infrastructure Checklist

**BEFORE declaring v1.0 production-ready:**
- [x] Docker available and functional
- [x] OTEL Collector configuration validated
- [x] Jaeger backend integration tested
- [x] Testcontainers dependencies aligned
- [x] Weaver 0.16+ installed and functional
- [ ] All health checks passing
- [ ] Integration tests passing
- [ ] Performance benchmarks met
- [ ] Cleanup procedures verified
- [ ] Documentation complete

### 8.2 Validation Script

**Master validation command:**
```bash
#!/bin/bash
# scripts/validate-infrastructure.sh

echo "🔍 Validating KNHK Infrastructure..."

# 1. Docker availability
docker info >/dev/null 2>&1 || {
  echo "❌ Docker not available"
  exit 1
}

# 2. Weaver installation
weaver --version | grep -E "0\.(1[6-9]|[2-9][0-9])" || {
  echo "❌ Weaver 0.16+ not installed"
  exit 1
}

# 3. Start OTEL stack
docker-compose -f tests/integration/docker-compose.otel.yml up -d

# 4. Wait for health checks (max 30s)
timeout 30 bash -c '
  until docker ps --filter "health=healthy" | grep -q "knhk-otel-collector"; do
    sleep 1
  done
'

# 5. Verify connectivity
docker exec knhk-otel-collector wget -O- http://jaeger:14250 >/dev/null 2>&1 || {
  echo "❌ Container connectivity failed"
  exit 1
}

# 6. Cleanup
docker-compose -f tests/integration/docker-compose.otel.yml down -v

echo "✅ Infrastructure validation complete"
```

## 9. Critical Success Factors (80/20)

### Top 20% Infrastructure Requirements:

1. **Docker running** (blocks all container-based testing)
2. **OTLP endpoints accessible** (blocks telemetry validation)
3. **Weaver installed** (blocks schema validation)
4. **Health checks passing** (indicates proper configuration)
5. **Container cleanup working** (prevents resource leaks)

### Validation Priority:

**HIGH PRIORITY (must pass):**
- Docker availability
- Weaver installation
- OTLP collector startup
- Basic connectivity

**MEDIUM PRIORITY (should pass):**
- Jaeger integration
- Prometheus integration
- Performance benchmarks

**LOW PRIORITY (nice to have):**
- Grafana dashboards
- Advanced monitoring
- Distributed tracing features

## 10. Troubleshooting Guide

### Common Issues:

**Docker not available:**
```bash
# Start Docker Desktop or Docker daemon
# macOS: open -a Docker
# Linux: sudo systemctl start docker
```

**Port conflicts:**
```bash
# Kill conflicting processes
lsof -ti :4317 | xargs kill -9
lsof -ti :4318 | xargs kill -9
```

**Container health check failures:**
```bash
# Check logs
docker logs knhk-otel-collector
docker logs knhk-jaeger

# Verify configuration
docker exec knhk-otel-collector cat /etc/otel-collector-config.yaml
```

**Weaver not found:**
```bash
# Install via cargo
cargo install otel-weaver-cli

# Verify installation
which weaver
weaver --version
```

## 11. Integration with DoD Validation

This infrastructure setup integrates with the overall v1.0 Definition of Done:

- **Build & Code Quality**: Infrastructure must build successfully
- **Weaver Validation**: Requires OTLP collector running
- **Functional Validation**: Requires telemetry backend
- **Performance Constraints**: Validated via infrastructure benchmarks

**Master validation flow:**
```
Infrastructure Setup
  ↓
Weaver Schema Validation (static)
  ↓
Build & Compilation
  ↓
Traditional Tests
  ↓
Weaver Live Validation (runtime)
  ↓
Production Ready ✅
```

---

**Document Version:** 1.0
**Last Updated:** 2025-11-09
**Owner:** Backend Infrastructure Team
**Status:** DRAFT → REVIEW → APPROVED
