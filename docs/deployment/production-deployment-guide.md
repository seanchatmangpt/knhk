# KNHK Production Deployment Guide

**Version**: 1.0.0
**Date**: 2025-11-16
**Status**: Official Production Guide

---

## Table of Contents

1. [Pre-Deployment Validation](#pre-deployment-validation)
2. [Infrastructure Requirements](#infrastructure-requirements)
3. [Deployment Steps](#deployment-steps)
4. [Weaver Validation](#weaver-validation)
5. [Post-Deployment Verification](#post-deployment-verification)
6. [Rollback Procedures](#rollback-procedures)
7. [Monitoring and Observability](#monitoring-and-observability)
8. [Troubleshooting](#troubleshooting)

---

## Pre-Deployment Validation

**CRITICAL**: Run ALL validation checks before deploying to production.

### 1. Run Production Readiness Check

```bash
# This validates the complete Definition of Done
./scripts/production-readiness-comprehensive.sh
```

**Expected**: All CRITICAL checks must pass (100% pass rate).

### 2. Run Weaver Schema Validation

```bash
# Validates that schemas are well-formed
weaver registry check -r registry/
```

**Expected**: Zero policy violations, all schemas valid.

### 3. Verify Build Artifacts

```bash
# Build release binaries
cargo build --workspace --release

# Verify binary exists
test -f target/release/knhk && echo "✅ CLI binary ready"

# Verify C library
cd c && make lib && echo "✅ C library ready"
```

### 4. Run Complete Test Suite

```bash
# All tests must pass
cargo test --workspace --all-features
make test-chicago-v04
make test-performance-v04
make test-integration-v2
```

**Expected**: 100% test pass rate.

### 5. Verify Performance Constraints

```bash
# Hot path operations MUST meet Chatman Constant (≤8 ticks)
make test-performance-v04
```

**Expected**: All R1 (hot path) operations ≤8 ticks.

---

## Infrastructure Requirements

### Minimum Requirements

| Component | Requirement | Notes |
|-----------|-------------|-------|
| **OS** | Linux (Ubuntu 20.04+, RHEL 8+) | Docker support required |
| **CPU** | 4+ cores, x86_64 with SIMD | AVX2 recommended for SIMD ops |
| **RAM** | 8GB minimum, 16GB recommended | For concurrent workflow execution |
| **Disk** | 20GB minimum, SSD recommended | For lockchain and receipts |
| **Network** | 1Gbps minimum | For OTLP telemetry export |

### Required Services

#### 1. OTEL Collector

```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

exporters:
  otlp:
    endpoint: "your-backend:4317"  # Jaeger, Tempo, etc.
  prometheus:
    endpoint: "0.0.0.0:8889"

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [otlp]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
```

Start collector:
```bash
docker run -d \
  --name otel-collector \
  -p 4317:4317 \
  -p 4318:4318 \
  -p 8889:8889 \
  -v $(pwd)/otel-collector-config.yaml:/etc/otelcol/config.yaml \
  otel/opentelemetry-collector:latest \
  --config=/etc/otelcol/config.yaml
```

#### 2. Storage (Lockchain)

```bash
# Initialize Git repository for lockchain
mkdir -p /var/lib/knhk/lockchain
cd /var/lib/knhk/lockchain
git init
git config user.name "KNHK System"
git config user.email "system@knhk.local"
```

#### 3. Database (Optional - for persistence)

For production deployments with high throughput:

```bash
# PostgreSQL for workflow state persistence
docker run -d \
  --name knhk-postgres \
  -e POSTGRES_DB=knhk \
  -e POSTGRES_USER=knhk \
  -e POSTGRES_PASSWORD=<secure-password> \
  -p 5432:5432 \
  -v knhk-postgres-data:/var/lib/postgresql/data \
  postgres:15
```

---

## Deployment Steps

### Step 1: Prepare Environment

```bash
# Set environment variables
export OTEL_EXPORTER_OTLP_ENDPOINT="http://otel-collector:4318"
export OTEL_SERVICE_NAME="knhk-production"
export KNHK_LOCKCHAIN_PATH="/var/lib/knhk/lockchain"
export KNHK_LOG_LEVEL="info"
export RUST_LOG="knhk=info,warn"

# Verify environment
env | grep -E 'OTEL|KNHK|RUST' && echo "✅ Environment configured"
```

### Step 2: Deploy Binaries

```bash
# Copy release binary to deployment location
sudo cp target/release/knhk /usr/local/bin/knhk
sudo chmod +x /usr/local/bin/knhk

# Copy C library
sudo cp c/lib/libknhk.so /usr/local/lib/
sudo ldconfig

# Verify installation
knhk --version && echo "✅ KNHK deployed"
```

### Step 3: Deploy Configuration

```bash
# Copy registry schemas
sudo mkdir -p /etc/knhk/registry
sudo cp -r registry/*.yaml /etc/knhk/registry/

# Copy ontology files (if using Σ)
sudo mkdir -p /etc/knhk/ontology
sudo cp -r ontology/*.ttl /etc/knhk/ontology/

# Set permissions
sudo chown -R knhk:knhk /etc/knhk
sudo chmod 755 /etc/knhk
```

### Step 4: Start Services

```bash
# Start KNHK service (systemd example)
sudo systemctl start knhk
sudo systemctl enable knhk

# Verify service is running
sudo systemctl status knhk
```

Example systemd unit (`/etc/systemd/system/knhk.service`):

```ini
[Unit]
Description=KNHK Knowledge Hook System
After=network.target otel-collector.service

[Service]
Type=simple
User=knhk
Group=knhk
Environment="OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318"
Environment="KNHK_LOCKCHAIN_PATH=/var/lib/knhk/lockchain"
ExecStart=/usr/local/bin/knhk server start
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

### Step 5: Smoke Tests

```bash
# Test 1: CLI is accessible
knhk --version

# Test 2: Execute a simple workflow
knhk workflow execute /path/to/test-workflow.ttl

# Test 3: Verify telemetry export
curl http://localhost:8889/metrics | grep knhk

# Test 4: Check lockchain receipts
ls -la /var/lib/knhk/lockchain/.git/objects/
```

---

## Weaver Validation

**CRITICAL**: This is the ONLY source of truth for production readiness.

### 1. Run Weaver Live-Check

```bash
# Start application with telemetry export
# (should already be running from Step 4)

# Run live validation
weaver registry live-check --registry /etc/knhk/registry/
```

**Expected Output**:
```
✅ Registry validation passed
✅ Live telemetry validation passed
   - All declared spans found in runtime traces
   - All declared metrics found in runtime metrics
   - No undeclared telemetry detected
   - 0 policy violations
```

### 2. Validate Specific Components

```bash
# Validate workflow engine telemetry
weaver registry live-check \
  --registry /etc/knhk/registry/ \
  --filter "knhk.workflow"

# Validate MAPE-K telemetry
weaver registry live-check \
  --registry /etc/knhk/registry/ \
  --filter "knhk.mape_k"

# Validate guard telemetry
weaver registry live-check \
  --registry /etc/knhk/registry/ \
  --filter "knhk.guard"
```

### 3. Continuous Validation

Set up continuous validation in CI/CD:

```bash
#!/bin/bash
# continuous-weaver-validation.sh

while true; do
    weaver registry live-check --registry /etc/knhk/registry/
    if [ $? -ne 0 ]; then
        echo "❌ Weaver validation failed at $(date)"
        # Alert on-call engineer
        curl -X POST https://alert-system/api/alert \
          -d '{"message": "KNHK Weaver validation failed"}'
    fi
    sleep 300  # Check every 5 minutes
done
```

---

## Post-Deployment Verification

### Verification Checklist

- [ ] **Service Running**: `systemctl status knhk` shows `active (running)`
- [ ] **Telemetry Export**: Traces/metrics visible in observability backend
- [ ] **Weaver Validation**: `weaver registry live-check` passes (0 violations)
- [ ] **Performance**: Hot path operations ≤8 ticks (verified via metrics)
- [ ] **Lockchain**: Receipts being generated in `/var/lib/knhk/lockchain`
- [ ] **Workflows**: Sample workflows execute successfully
- [ ] **Error Rate**: Error rate <1% (check metrics)
- [ ] **Latency**: P95 latency within SLO targets

### Health Check Endpoint

```bash
# KNHK should expose health endpoint
curl http://localhost:8080/health

# Expected response:
{
  "status": "healthy",
  "version": "0.4.0",
  "uptime_seconds": 3600,
  "telemetry_export": "ok",
  "lockchain": "ok"
}
```

---

## Rollback Procedures

### Scenario 1: Weaver Validation Fails

```bash
# Immediate rollback if Weaver validation fails
sudo systemctl stop knhk
sudo systemctl start knhk-previous  # Previous version

# Verify previous version passes Weaver
weaver registry live-check --registry /etc/knhk/registry/
```

### Scenario 2: Performance Degradation

```bash
# If hot path operations exceed 8 ticks
sudo systemctl stop knhk

# Restore previous binary
sudo cp /usr/local/bin/knhk.backup /usr/local/bin/knhk

sudo systemctl start knhk

# Verify performance
make test-performance-v04
```

### Scenario 3: Critical Bug

```bash
# Emergency rollback
sudo systemctl stop knhk
sudo apt-get install knhk=<previous-version>
sudo systemctl start knhk

# Verify system is stable
knhk --version
systemctl status knhk
```

---

## Monitoring and Observability

### Key Metrics to Monitor

#### 1. Performance Metrics

```promql
# Hot path operations latency (MUST be ≤8 ticks)
histogram_quantile(0.99, knhk_operation_latency_bucket{runtime_class="R1"})

# Chatman Constant compliance rate
knhk_guard_chatman_compliance{operation_type="hot_path"}
```

#### 2. MAPE-K Metrics

```promql
# MAPE-K cycle rate
rate(knhk_mape_k_cycle_count[5m])

# Anomaly detection rate
knhk_mape_k_anomaly_rate

# Adaptation success rate
knhk_mape_k_adaptation_success_rate
```

#### 3. Guard Violations

```promql
# Guard violation rate (should be 0)
rate(knhk_guard_violation_count[5m])

# Specific constraint violations
knhk_guard_violation_count{constraint_type="max_run_len"}
```

#### 4. Workflow Execution

```promql
# Workflow completion rate
rate(knhk_workflow_engine_execution_count{status="completed"}[5m])

# Pattern execution latency
histogram_quantile(0.95, knhk_workflow_engine_pattern_execution_latency_bucket)
```

### Alerting Rules

```yaml
# Prometheus alerting rules
groups:
  - name: knhk_critical
    rules:
      - alert: WeaverValidationFailed
        expr: knhk_weaver_validation_status == 0
        for: 1m
        severity: critical
        annotations:
          summary: "Weaver validation failed"

      - alert: ChatmanConstantViolation
        expr: knhk_guard_chatman_compliance < 1.0
        for: 5m
        severity: critical
        annotations:
          summary: "Hot path operations exceeding 8 ticks"

      - alert: HighErrorRate
        expr: rate(knhk_errors_total[5m]) > 0.01
        for: 5m
        severity: warning
        annotations:
          summary: "Error rate above 1%"
```

---

## Troubleshooting

See [Troubleshooting Guide](./troubleshooting-guide.md) for common issues and solutions.

### Quick Diagnostics

```bash
# Check service logs
sudo journalctl -u knhk -n 100 --no-pager

# Check telemetry export
curl http://localhost:4318/v1/traces -I

# Verify Weaver validation
weaver registry check -r /etc/knhk/registry/

# Check performance
cargo test --package knhk-integration-tests --test performance_tests
```

---

## Appendix: Constitutional Validation

KNHK's correctness is defined by the Constitution (formal laws). Production deployment must verify:

### Constitutional Constraints

1. **A = μ(O)**: Actions computed from observations via hooks
2. **μ∘μ = μ**: Idempotence (safe retries)
3. **O ⊨ Σ**: Typing (schema compliance)
4. **μ ⊂ τ, τ ≤ 8**: Epoch containment (Chatman Constant)
5. **hash(A) = hash(μ(O))**: Provenance (receipt validation)
6. **μ ⊣ Q**: Guard adjointness (constraint enforcement)

### Validation Methods

| Constraint | Validation Method |
|------------|------------------|
| **A = μ(O)** | Weaver live-check (telemetry proves execution) |
| **μ∘μ = μ** | Integration tests (retry safety) |
| **O ⊨ Σ** | Weaver registry check (schema validation) |
| **μ ⊂ τ, τ ≤ 8** | Performance tests (≤8 ticks verified) |
| **hash(A) = hash(μ(O))** | Receipt verification (lockchain integrity) |
| **μ ⊣ Q** | Guard enforcement tests (constraint validation) |

---

**Status**: Production deployment procedures validated
**Last Updated**: 2025-11-16
**Next Review**: Before each major release
