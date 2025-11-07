# KNHK v1.0 Monitoring Infrastructure Setup Guide

## Overview

This guide covers the complete production monitoring infrastructure for KNHK v1.0, including Prometheus metrics collection, Grafana dashboards, and Alertmanager alerting.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    KNHK Applications                         │
│  (knhk-etl, knhk-hot, knhk-warm, knhk-sidecar)             │
└────────────────────┬────────────────────────────────────────┘
                     │ OTLP (gRPC/HTTP)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              OTEL Collector (Port 4317/4318)                 │
│  • Receives traces, metrics, logs via OTLP                   │
│  • Batch processing & resource detection                     │
│  • Exports to Prometheus (port 8888)                         │
└────────────────────┬────────────────────────────────────────┘
                     │ Prometheus scrape
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                Prometheus (Port 9090)                        │
│  • Scrapes OTEL metrics (15s interval)                       │
│  • Evaluates alerting rules                                  │
│  • 30-day retention                                          │
│  • PromQL query interface                                    │
└────────────┬────────────────────┬────────────────────────────┘
             │                     │ Alert notifications
             │ Metrics queries     ▼
             ▼            ┌──────────────────────┐
┌──────────────────────┐ │   Alertmanager       │
│  Grafana (Port 3000) │ │    (Port 9093)       │
│  • 3 dashboards      │ │  • Alert routing     │
│  • Auto-provisioned  │ │  • Deduplication     │
│  • Real-time refresh │ │  • Notifications     │
└──────────────────────┘ └──────────────────────┘
```

## Quick Start

### Prerequisites

- Docker and Docker Compose installed
- KNHK base infrastructure running (Kafka, Postgres, OTEL, Redis)
- Ports available: 3000 (Grafana), 9090 (Prometheus), 9093 (Alertmanager)

### Start Monitoring Stack

```bash
cd tests/integration/monitoring/scripts
./start_monitoring.sh
```

This script will:
1. Create Docker network if needed
2. Start base infrastructure (Kafka, Postgres, OTEL, Redis)
3. Wait for services to be healthy
4. Update OTEL collector to export Prometheus metrics
5. Start Prometheus, Grafana, and Alertmanager
6. Wait for monitoring services to be healthy

### Access Dashboards

Once started, access:

- **Grafana**: http://localhost:3000
  - Username: `admin`
  - Password: `knhk_admin`

- **Prometheus**: http://localhost:9090

- **Alertmanager**: http://localhost:9093

### Stop Monitoring Stack

```bash
cd tests/integration/monitoring/scripts
./stop_monitoring.sh
```

## Grafana Dashboards

### 1. KNHK Beat System (8-Beat Epoch)

**UID**: `knhk-beat-system`

Monitors the 8-beat epoch reconciliation system:

- **Beat System Rate**: Cycles/sec and Pulses/sec
- **Beat Pulse Accuracy**: 1:8 ratio gauge (target: 1.0)
- **Tick Distribution**: Bar chart showing tick 0-7 distribution
- **Total Counters**: Total cycles, pulses
- **Beat Timing Deviation**: Deviation from expected 1:8 ratio
- **Beat System Health**: Alert status

**Key Metrics**:
- `knhk_beat_cycles_total` - Total beat cycles executed
- `knhk_beat_pulses_total` - Total commit pulses (every 8th tick)
- `knhk_fiber_process_tick_total` - Fiber executions by tick

### 2. KNHK Performance & SLO Compliance

**UID**: `knhk-performance-slo`

Monitors hot path performance and SLO compliance:

- **SLO Compliance Gauge**: % operations ≤8 ticks (target: ≥95%)
- **Operation Latency**: P50/P95/P99 by operation type
- **Fiber Execution Time**: Ticks per delta by shard
- **R1 Violations**: Operations exceeding 8-tick budget
- **Fiber Park Rate**: % deltas parked to W1
- **Delta Throughput**: Deltas processed per second
- **Quick Stats**: Total violations, P50/P95/P99, throughput

**Key Metrics**:
- `knhk_operation_duration` - Operation latency histogram
- `knhk_operation_r1_violations_total` - Hot path violations
- `knhk_fiber_ticks_per_unit` - Fiber execution time
- `knhk_fiber_park_rate` - W1 parking rate (0.0-1.0)
- `knhk_fiber_deltas_processed_total` - Delta throughput

**SLO Definition**: ≥95% of R1 operations must complete within 8 ticks (Chatman Constant).

### 3. KNHK Receipt Metrics

**UID**: `knhk-receipts`

Monitors receipt generation and validation:

- **Hash Validation Status**: Critical alert for any mismatches
- **Receipt Processing Rate**: Created, validated, mismatches
- **Receipt Generation Ticks**: P50/P95/P99 tick latency
- **Receipt Distribution**: By lanes
- **Top 20 Receipts**: Sorted by tick count
- **Quick Stats**: Total created, validated, mismatches, P95 ticks

**Key Metrics**:
- `knhk_receipt_created_total` - Receipts created
- `knhk_receipt_validated_total` - Receipts validated
- `knhk_receipt_hash_mismatches_total` - Hash validation failures (CRITICAL)
- `knhk_receipt_ticks` - Receipt generation time

## Prometheus Alerting Rules

### Critical Alerts

#### TickBudgetViolation
- **Trigger**: <95% of operations within 8-tick budget
- **Duration**: 1 minute
- **Severity**: Critical
- **Action**: Immediate investigation of hot path performance

#### R1ViolationRateHigh
- **Trigger**: >5% violation rate
- **Duration**: 2 minutes
- **Severity**: Critical
- **Action**: Hot path optimization required

#### ReceiptHashMismatch
- **Trigger**: Any hash mismatch detected
- **Duration**: 0 seconds (immediate)
- **Severity**: Critical
- **Action**: Data integrity issue - investigate immediately

#### SLOViolation
- **Trigger**: <95% operations within budget over 5 minutes
- **Duration**: 5 minutes
- **Severity**: Critical
- **Action**: SLO breach - escalate to SRE team

### Warning Alerts

#### FiberExecutionNearLimit
- **Trigger**: Fiber execution >7 ticks
- **Duration**: 5 minutes
- **Severity**: Warning
- **Action**: Monitor for potential parking to W1

#### HighParkRate
- **Trigger**: Park rate >20%
- **Duration**: 3 minutes
- **Severity**: Warning
- **Action**: May indicate undersized tick budget or complex deltas

#### BeatPulseIrregular
- **Trigger**: Pulse rate deviates >10% from 1:8 ratio
- **Duration**: 2 minutes
- **Severity**: Warning
- **Action**: Beat system timing degraded

### System Health Alerts

#### ContainerCrashLoop
- **Trigger**: >0.1 restarts per 10 minutes
- **Duration**: 5 minutes
- **Severity**: Critical

#### HighMemoryUsage
- **Trigger**: >85% memory limit
- **Duration**: 5 minutes
- **Severity**: Warning

#### OTELCollectorDown
- **Trigger**: OTEL collector unreachable
- **Duration**: 1 minute
- **Severity**: Critical

## Alertmanager Configuration

### Default Route
- **Grouping**: By `alertname`, `severity`, `component`
- **Group Wait**: 10 seconds
- **Group Interval**: 10 seconds
- **Repeat Interval**: 12 hours

### Custom Routes

#### Critical Alerts
- **Receiver**: `knhk-critical`
- **Group Wait**: 0 seconds (immediate)
- **Repeat Interval**: 5 minutes

#### Warning Alerts
- **Receiver**: `knhk-warnings`
- **Group Wait**: 30 seconds (batched)
- **Repeat Interval**: 4 hours

#### Hot Path Alerts
- **Receiver**: `knhk-hotpath`
- **Group Wait**: 0 seconds (immediate)
- **Repeat Interval**: 5 minutes

#### SLO Alerts
- **Receiver**: `knhk-slo`
- **Group Wait**: 0 seconds (immediate)
- **Repeat Interval**: 15 minutes

### Notification Channels (Configure for Production)

Edit `tests/integration/monitoring/alertmanager/alertmanager.yml`:

```yaml
receivers:
  - name: 'knhk-critical'
    email_configs:
      - to: 'oncall@example.com'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
        channel: '#knhk-critical'
    pagerduty_configs:
      - service_key: 'your-pagerduty-service-key'
```

## CLI Metrics Query Tool

Query key metrics from the command line:

```bash
cd tests/integration/monitoring/scripts
./query_metrics.sh
```

**Output**:
- SLO Compliance (target ≥95%)
- Performance metrics (P50/P95/P99)
- R1 violations count and rate
- Beat system health (cycles, pulses, ratio)
- Fiber execution (park rate, throughput)
- Receipt metrics (created, validated, mismatches)
- Active alerts

**Example**:
```
================================================
🎯 SLO Compliance (Target: ≥95%)
================================================
✅ SLO: 98.75%

================================================
⚡ Performance Metrics
================================================
P50 Latency: 3.50 ticks
✅ P95 Latency: 6.80 ticks
✅ P99 Latency: 7.90 ticks

================================================
🚨 R1 Violations (Chatman Constant >8 ticks)
================================================
Total Violations: 0
✅ Current Rate: 0/sec
```

## Metric Reference

### Beat System Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `knhk_beat_cycles_total` | Counter | Total beat cycles executed |
| `knhk_beat_pulses_total` | Counter | Total commit pulses (every 8th tick) |

### Fiber Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `knhk_fiber_ticks_per_unit` | Histogram | Execution time in ticks per delta |
| `knhk_fiber_park_rate` | Gauge | % deltas parked to W1 (0.0-1.0) |
| `knhk_fiber_deltas_processed_total` | Counter | Total deltas processed |

### Operation Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `knhk_operation_duration` | Histogram | Operation latency in ticks |
| `knhk_operation_r1_violations_total` | Counter | R1 operations exceeding 8 ticks |

### Receipt Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `knhk_receipt_created_total` | Counter | Receipts created |
| `knhk_receipt_validated_total` | Counter | Receipts validated |
| `knhk_receipt_hash_mismatches_total` | Counter | Hash validation failures |
| `knhk_receipt_ticks` | Histogram | Receipt generation time |

## Performance Tuning

### Prometheus

**Storage Retention**:
- Default: 30 days
- Adjust: `--storage.tsdb.retention.time=30d` in docker-compose

**Scrape Interval**:
- Default: 15 seconds
- Adjust: `scrape_interval` in prometheus.yml

**Memory Usage**:
- ~1-2GB for 30-day retention
- Scale up if retention is increased

### Grafana

**Dashboard Refresh**:
- Default: 5 seconds
- Adjust per dashboard for high-cardinality metrics

**Data Source Timeout**:
- Default: 60 seconds
- Adjust for complex queries

### OTEL Collector

**Batch Processing**:
- Timeout: 1 second
- Batch size: 1024

**Prometheus Export**:
- Endpoint: `:8888/metrics`
- Metric expiration: 5 minutes

## Troubleshooting

### Metrics Not Appearing

1. **Check OTEL Collector**:
   ```bash
   curl http://localhost:8888/metrics | grep knhk
   ```

2. **Check Prometheus Scrape**:
   - Visit http://localhost:9090/targets
   - Verify `otel-collector` target is UP

3. **Check OTEL Config**:
   ```bash
   docker logs knhk-test-otel-collector
   ```

### Dashboards Empty

1. **Verify Prometheus Data Source**:
   - Grafana → Configuration → Data Sources
   - Test connection

2. **Check Metric Names**:
   ```bash
   curl -G http://localhost:9090/api/v1/label/__name__/values | jq
   ```

3. **Verify Time Range**:
   - Ensure dashboard time range includes recent data

### Alerts Not Firing

1. **Check Alert Rules**:
   ```bash
   curl http://localhost:9090/api/v1/rules | jq
   ```

2. **Verify Alertmanager**:
   - Visit http://localhost:9093

3. **Check Alert Evaluation**:
   - Prometheus → Alerts
   - See pending/firing status

### High Memory Usage

1. **Reduce Retention**:
   - Lower `--storage.tsdb.retention.time`

2. **Reduce Scrape Interval**:
   - Increase `scrape_interval` in prometheus.yml

3. **Limit Cardinality**:
   - Review label cardinality in metrics

## Production Deployment

### Security

1. **Change Default Credentials**:
   ```yaml
   environment:
     - GF_SECURITY_ADMIN_PASSWORD=<strong-password>
   ```

2. **Enable TLS**:
   - Configure reverse proxy (nginx/caddy)
   - Terminate TLS before Grafana/Prometheus

3. **Network Isolation**:
   - Use internal Docker network
   - Expose only necessary ports

### High Availability

1. **Prometheus HA**:
   - Run 2+ Prometheus instances
   - Use Thanos for long-term storage

2. **Alertmanager Clustering**:
   - 3+ Alertmanager instances
   - Gossip protocol for deduplication

3. **Grafana HA**:
   - Use external database (PostgreSQL)
   - Run behind load balancer

### Backup & Recovery

1. **Prometheus Data**:
   ```bash
   docker run --rm -v prometheus-data:/data -v $(pwd):/backup \
     alpine tar czf /backup/prometheus-backup.tar.gz /data
   ```

2. **Grafana Config**:
   ```bash
   docker run --rm -v grafana-data:/data -v $(pwd):/backup \
     alpine tar czf /backup/grafana-backup.tar.gz /data
   ```

3. **Restore**:
   ```bash
   docker run --rm -v prometheus-data:/data -v $(pwd):/backup \
     alpine tar xzf /backup/prometheus-backup.tar.gz -C /
   ```

## Maintenance

### Log Rotation

Prometheus and Alertmanager logs grow over time:

```bash
# View logs
docker logs knhk-prometheus
docker logs knhk-grafana
docker logs knhk-alertmanager

# Clear logs
docker-compose -f monitoring/docker-compose.monitoring.yml restart
```

### Metric Cleanup

Remove stale metrics from Prometheus:

```bash
curl -X POST http://localhost:9090/api/v1/admin/tsdb/clean_tombstones
```

### Dashboard Updates

1. Edit JSON in `grafana/dashboards/`
2. Restart Grafana to load changes:
   ```bash
   docker restart knhk-grafana
   ```

## Support & References

- **KNHK Documentation**: `/docs/`
- **Prometheus Docs**: https://prometheus.io/docs/
- **Grafana Docs**: https://grafana.com/docs/
- **OTEL Docs**: https://opentelemetry.io/docs/

## Appendix: Metric Naming Conventions

KNHK follows OpenTelemetry semantic conventions with custom namespace:

- **Namespace**: `knhk_*`
- **Component**: `knhk_<component>_*` (e.g., `knhk_beat_*`, `knhk_fiber_*`)
- **Type Suffix**: `_total` (counters), `_bucket` (histograms)

**Examples**:
- ✅ `knhk_beat_cycles_total` - Counter
- ✅ `knhk_operation_duration_bucket` - Histogram bucket
- ✅ `knhk_fiber_park_rate` - Gauge
- ❌ `beat_cycles` - Missing namespace
- ❌ `knhk_cycles_beat_total` - Incorrect component order

---

**End of KNHK v1.0 Monitoring Setup Guide**

For updates and issues, see: https://github.com/your-org/knhk
