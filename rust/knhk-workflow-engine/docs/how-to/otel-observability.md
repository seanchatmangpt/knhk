# How-To: Setup OpenTelemetry Observability

**Complete Guide to Monitoring KNHK Workflow Engine with OTEL**

- **Time to Complete**: ~45 minutes
- **Difficulty Level**: Intermediate
- **Prerequisites**: KNHK Engine running, Docker/Kubernetes
- **You'll Learn**: Setup distributed tracing, metrics, and logging

---

## Table of Contents

1. [OTEL Architecture Overview](#otel-architecture-overview)
2. [Deploy OTEL Collector](#deploy-otel-collector)
3. [Configure KNHK for OTEL](#configure-knhk-for-otel)
4. [Setup Jaeger for Tracing](#setup-jaeger-for-tracing)
5. [Setup Prometheus for Metrics](#setup-prometheus-for-metrics)
6. [Monitor Workflows](#monitor-workflows)
7. [Troubleshooting](#troubleshooting)

---

## OTEL Architecture Overview

**OpenTelemetry** provides standardized observability:

```
KNHK Workflow Engine
       ↓ (OTEL SDK)
   [Traces, Metrics, Logs]
       ↓ (gRPC)
   OTEL Collector
       ↓
   ├─→ Jaeger (Distributed Tracing)
   ├─→ Prometheus (Metrics)
   └─→ Loki (Log Aggregation)
       ↓
   Grafana (Unified Dashboard)
```

### What We'll Monitor

| Component | Tool | Purpose |
|-----------|------|---------|
| **Traces** | Jaeger | See execution flow, latency, errors |
| **Metrics** | Prometheus | Performance, throughput, resource usage |
| **Logs** | Loki | Detailed events, debugging |
| **Dashboard** | Grafana | Unified view of all signals |

---

## Deploy OTEL Collector

### Step 1: Create OTEL Collector Configuration

Create file: `otel-collector-config.yaml`

```yaml
receivers:
  # Receive OTEL protocol data from applications
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

  # Prometheus scraping (optional)
  prometheus:
    config:
      scrape_configs:
      - job_name: 'otel-collector'
        scrape_interval: 10s
        static_configs:
        - targets: ['localhost:8888']

processors:
  # Add attributes to spans
  attributes:
    actions:
    - key: service.version
      value: "1.0.0"
      action: upsert
    - key: environment
      value: "production"
      action: upsert

  # Sample spans for high-volume data
  probabilistic_sampler:
    sampling_percentage: 10

  # Batch spans before sending
  batch:
    send_batch_size: 512
    timeout: 10s

exporters:
  # Export traces to Jaeger
  jaeger:
    endpoint: http://jaeger:14250

  # Export metrics to Prometheus
  prometheus:
    endpoint: 0.0.0.0:8889
    namespace: knhk

  # Export logs to Loki (optional)
  loki:
    endpoint: http://loki:3100/loki/api/v1/push

extensions:
  # Health check endpoint
  health_check:
    endpoint: 0.0.0.0:13133

  # Profiling (if enabled)
  pprof:
    endpoint: localhost:6060

service:
  extensions: [health_check, pprof]

  pipelines:
    # Traces
    traces:
      receivers: [otlp]
      processors: [attributes, probabilistic_sampler, batch]
      exporters: [jaeger]

    # Metrics
    metrics:
      receivers: [otlp, prometheus]
      processors: [batch]
      exporters: [prometheus]

    # Logs
    logs:
      receivers: [otlp]
      processors: [batch]
      exporters: [loki]
```

### Step 2: Deploy OTEL Collector (Docker)

```bash
# Create docker-compose.yaml
cat > docker-compose.yaml <<'EOF'
version: '3.9'

services:
  # OpenTelemetry Collector
  otel-collector:
    image: otel/opentelemetry-collector-k8s:0.88.0
    container_name: otel-collector
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
      - "8889:8889"   # Prometheus metrics
      - "13133:13133" # Health check
    depends_on:
      - jaeger
      - prometheus
    networks:
      - otel-network

  # Jaeger for distributed tracing
  jaeger:
    image: jaegertracing/all-in-one:latest
    container_name: jaeger
    ports:
      - "6831:6831/udp"  # Jaeger agent
      - "16686:16686"    # UI
    environment:
      COLLECTOR_OTLP_ENABLED: "true"
    networks:
      - otel-network

  # Prometheus for metrics
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    volumes:
      - ./prometheus.yaml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
    networks:
      - otel-network

  # Grafana for visualization
  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    ports:
      - "3000:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
    volumes:
      - grafana-data:/var/lib/grafana
    depends_on:
      - prometheus
      - jaeger
    networks:
      - otel-network

networks:
  otel-network:
    driver: bridge

volumes:
  prometheus-data:
  grafana-data:
EOF

# Start the stack
docker-compose up -d

# Verify
docker-compose ps
```

### Step 3: Verify OTEL Collector is Running

```bash
# Check health
curl http://localhost:13133/

# Check Jaeger UI
open http://localhost:16686

# Check Prometheus
open http://localhost:9090

# Check Grafana
open http://localhost:3000
# Default credentials: admin/admin
```

---

## Configure KNHK for OTEL

### Step 1: Enable OTEL in Configuration

Edit `config.toml`:

```toml
[observability.otel]
# Enable OpenTelemetry
enabled = true

# OTEL Collector endpoint
collector_endpoint = "http://localhost:4317"

# Sampling rate (0.0 to 1.0)
# 0.1 = sample 10% of traces (reduces overhead)
sampling_rate = 0.1

# Batch size for spans
batch_size = 512

# Export interval in seconds
export_interval = 5

# Service metadata
service_name = "knhk-workflow-engine"
service_version = "1.0.0"
service_namespace = "production"

# Include environment information
include_environment = true

# Custom attributes
[observability.otel.attributes]
deployment = "kubernetes"
region = "us-east-1"
team = "platform"
```

### Step 2: Enable Metrics Export

```toml
[observability.metrics]
# Enable metrics collection
enabled = true

# Export interval in seconds
export_interval = 60

# Detailed metrics (higher cardinality, more data)
detailed_metrics = false

# What metrics to include
enable_execution_metrics = true
enable_resource_metrics = true
enable_queue_metrics = true
enable_latency_metrics = true
enable_database_metrics = true
```

### Step 3: Enable Structured Logging

```toml
[observability.logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log format: text or json
format = "json"

# Structured logging
structured = true

# OTEL log exporting
otel_logs_enabled = true

# Include trace context in logs
include_trace_id = true
include_span_id = true

# Performance logging
perf_logging_enabled = true
perf_threshold_ms = 100  # Log operations > 100ms
```

### Step 4: Restart KNHK Engine

```bash
# Stop old instance
pkill knhk-workflow

# Start with new config
cargo run --release --bin knhk-workflow

# Verify OTEL is enabled
curl http://localhost:8080/health | jq '.otel_enabled'
# Should return: true
```

---

## Setup Jaeger for Tracing

### Step 1: Jaeger Dashboard

Open browser: http://localhost:16686

### Step 2: Find KNHK Service

1. Click "Service" dropdown
2. Look for "knhk-workflow-engine"
3. Should see recent traces

### Step 3: Analyze a Trace

```bash
# Execute a workflow to generate traces
curl -X POST http://localhost:8080/api/v1/workflows/wf_id/cases \
  -d '{"data": {...}}'

# Get case details (shows trace ID)
curl http://localhost:8080/api/v1/cases/case_id | jq '.traces.otel_trace_id'

# Open trace in Jaeger
# Go to: http://localhost:16686
# Search by Trace ID
```

### Understanding Trace Data

```
Trace Example:
├─ CreateCaseSpan (1.2ms)
│  ├─ ValidateData (0.5ms)
│  └─ SaveToDatabase (0.6ms)
├─ ExecuteWorkflow (45.3ms)
│  ├─ ProcessingTask (30ms)
│  │  ├─ CallExternalAPI (25ms)
│  │  └─ UpdateState (5ms)
│  └─ CompleteTask (15.3ms)
└─ ReturnResponse (0.1ms)
```

**Useful Metrics from Traces**:
- **Latency**: End-to-end time
- **Span Duration**: Individual operation time
- **Critical Path**: Longest chain of operations
- **Bottlenecks**: Slowest span

---

## Setup Prometheus for Metrics

### Step 1: Create Prometheus Configuration

Create file: `prometheus.yaml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  # Scrape KNHK workflow engine
  - job_name: 'knhk-workflow-engine'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # Scrape OTEL Collector
  - job_name: 'otel-collector'
    static_configs:
      - targets: ['localhost:8889']

  # Scrape Jaeger
  - job_name: 'jaeger'
    static_configs:
      - targets: ['localhost:14269']
```

### Step 2: Verify Prometheus Scraping

Open: http://localhost:9090

1. Click "Status" → "Targets"
2. Should show KNHK endpoint
3. Should show "UP"

### Step 3: Query Metrics

In Prometheus UI, try these queries:

```promql
# Case throughput (cases created per second)
rate(knhk_cases_created_total[1m])

# Average case duration (in milliseconds)
avg(knhk_case_duration_ms)

# P95 case latency (95th percentile)
histogram_quantile(0.95, knhk_case_duration_ms_bucket)

# Active cases
knhk_cases_active

# Task completion rate
rate(knhk_tasks_completed_total[1m])

# Workflow validation time
knhk_workflow_validation_time_ms

# Cache hit ratio
knhk_cache_hits / (knhk_cache_hits + knhk_cache_misses)
```

---

## Monitor Workflows

### Step 1: Create Grafana Dashboard

1. Open Grafana: http://localhost:3000
2. Add Prometheus data source:
   - Click "Configuration" → "Data Sources"
   - Click "Add data source"
   - Type: Prometheus
   - URL: http://prometheus:9090
   - Click "Save & Test"

3. Create Dashboard:
   - Click "+" → "Dashboard"
   - Click "Add new panel"

### Step 2: Add Performance Panel

```
Panel Name: Case Throughput
Query: rate(knhk_cases_created_total[1m])
Visualization: Graph
Y-Axis: Cases/sec
```

### Step 3: Add Latency Panel

```
Panel Name: Case Duration (P95)
Query: histogram_quantile(0.95, knhk_case_duration_ms_bucket)
Visualization: Graph
Y-Axis: Milliseconds
Thresholds:
  - Green: < 100ms
  - Yellow: < 500ms
  - Red: > 500ms
```

### Step 4: Add Resource Usage Panel

```
Panel Name: Memory Usage
Query: process_resident_memory_bytes
Visualization: Gauge
Unit: Bytes
```

### Step 5: View Jaeger Traces

In Grafana:
1. Click "+" → "Dashboard"
2. Click "Add panel"
3. Data source: Jaeger
4. Show traces in Grafana (if enabled)

---

## Monitor Workflows

### Real-Time Monitoring

```bash
# Terminal 1: Watch throughput
watch -n 1 'curl -s http://localhost:8080/api/v1/workflows/wf_id/metrics | jq ".metrics"'

# Terminal 2: Check Prometheus
open http://localhost:9090

# Terminal 3: Check traces
open http://localhost:16686
```

### Key Metrics to Monitor

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Case Throughput | 100+ cases/sec | < 10 cases/sec |
| P99 Latency | < 500ms | > 1000ms |
| Error Rate | < 0.1% | > 1% |
| Memory Usage | Stable | > 80% limit |
| Cache Hit Ratio | > 80% | < 50% |

### Example Monitoring Script

```bash
#!/bin/bash

while true; do
  echo "=== KNHK Metrics $(date) ==="

  # Get metrics
  METRICS=$(curl -s http://localhost:8080/api/v1/workflows/wf_id/metrics)

  # Extract key values
  THROUGHPUT=$(echo $METRICS | jq '.metrics.cases_created')
  LATENCY=$(echo $METRICS | jq '.metrics.avg_duration_ms')

  echo "Throughput: $THROUGHPUT cases created"
  echo "Avg Latency: ${LATENCY}ms"

  # Check thresholds
  if (( $(echo "$LATENCY > 500" | bc -l) )); then
    echo "⚠️  WARNING: High latency detected!"
  fi

  sleep 30
done
```

---

## Create Alert Rules

### Prometheus Alerts

Create file: `alert-rules.yaml`

```yaml
groups:
- name: knhk-alerts
  interval: 30s
  rules:
  # High latency alert
  - alert: HighWorkflowLatency
    expr: histogram_quantile(0.95, knhk_case_duration_ms_bucket) > 500
    for: 5m
    annotations:
      summary: "Workflow latency is high ({{ $value }}ms)"

  # High error rate alert
  - alert: HighErrorRate
    expr: rate(knhk_cases_failed_total[1m]) / rate(knhk_cases_created_total[1m]) > 0.01
    for: 2m
    annotations:
      summary: "Error rate is {{ $value }}%"

  # Low throughput alert
  - alert: LowThroughput
    expr: rate(knhk_cases_created_total[1m]) < 10
    for: 10m
    annotations:
      summary: "Workflow throughput dropped to {{ $value }} cases/sec"

  # Memory pressure
  - alert: HighMemoryUsage
    expr: process_resident_memory_bytes / (100 * 1024 * 1024) > 80
    for: 5m
    annotations:
      summary: "Memory usage is {{ $value }}%"
```

---

## Troubleshooting

### Issue: No Traces in Jaeger

```bash
# 1. Verify collector is running
curl http://localhost:13133/

# 2. Check KNHK OTEL configuration
grep "collector_endpoint" config.toml

# 3. Verify connectivity
curl -v http://localhost:4317/

# 4. Check KNHK logs
tail -f logs/knhk.log | grep -i otel

# 5. Restart KNHK
pkill knhk-workflow
cargo run --release
```

### Issue: Missing Metrics in Prometheus

```bash
# 1. Check targets in Prometheus
# http://localhost:9090/targets

# 2. Verify metrics endpoint
curl http://localhost:8080/metrics

# 3. Check metrics configuration
grep -A 10 "\[observability.metrics\]" config.toml

# 4. Restart and check again
```

### Issue: High OTEL Overhead

If OTEL collection is slowing down workflows:

```toml
# Reduce sampling rate
[observability.otel]
sampling_rate = 0.05  # Sample only 5%

# Disable detailed metrics
[observability.metrics]
detailed_metrics = false

# Increase batch size
batch_size = 1024

# Increase export interval
export_interval = 10
```

---

## Related Documentation

- [Configuration Guide](../reference/configuration.md) - OTEL settings
- [How-To: Kubernetes Deployment](./kubernetes-deployment.md) - K8s setup
- [How-To: Troubleshooting](./troubleshooting.md) - Problem diagnosis
