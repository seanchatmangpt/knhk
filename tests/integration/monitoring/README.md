# KNHK v1.0 Monitoring Infrastructure

Production-grade monitoring stack for KNHK with Prometheus, Grafana, and Alertmanager.

## 🚀 Quick Start

```bash
# Start monitoring stack
./scripts/start_monitoring.sh

# Access Grafana
open http://localhost:3000
# Username: admin
# Password: knhk_admin

# Query metrics
./scripts/query_metrics.sh

# Stop monitoring stack
./scripts/stop_monitoring.sh
```

## 📊 Components

### Prometheus (Port 9090)
- Scrapes OTEL Collector metrics every 15s
- 30-day retention
- Evaluates 15 alerting rules
- PromQL query interface

### Grafana (Port 3000)
- 3 pre-configured dashboards
- Auto-provisioned data sources
- 5-second refresh rate
- Admin credentials: `admin/knhk_admin`

### Alertmanager (Port 9093)
- Alert routing and deduplication
- Severity-based routing (critical/warning)
- Configurable notification channels
- Alert grouping and inhibition

### OTEL Collector (Port 4317/4318, 8888)
- OTLP receiver for traces/metrics/logs
- Batch processing (1s timeout, 1024 batch size)
- Prometheus exporter (`:8888/metrics`)
- Resource detection

## 📈 Dashboards

### 1. KNHK Beat System (8-Beat Epoch)
**UID**: `knhk-beat-system`

Monitors 8-beat epoch reconciliation:
- Beat cycle/pulse rate
- Pulse:cycle ratio accuracy (1:8 target)
- Tick distribution (0-7)
- Beat timing health

**Key Insight**: Ensures beat system maintains precise 8-tick cadence.

### 2. KNHK Performance & SLO Compliance
**UID**: `knhk-performance-slo`

Monitors hot path performance:
- SLO compliance gauge (≥95% target)
- Operation latency (P50/P95/P99)
- R1 violation tracking (>8 ticks)
- Fiber park rate
- Delta throughput

**Key Insight**: Ensures ≥95% operations complete within 8-tick budget (Chatman Constant).

### 3. KNHK Receipt Metrics
**UID**: `knhk-receipts`

Monitors receipt correctness:
- Hash validation status
- Receipt processing rate
- Generation latency
- Lane distribution
- Top 20 receipts by ticks

**Key Insight**: Detects data corruption via hash mismatches (critical alert).

## 🔔 Alerts

### Critical (Immediate Response Required)

| Alert | Trigger | Duration | Action |
|-------|---------|----------|--------|
| **TickBudgetViolation** | <95% ops ≤8 ticks | 1m | Investigate hot path |
| **R1ViolationRateHigh** | >5% violation rate | 2m | Optimize hot path |
| **ReceiptHashMismatch** | Any hash mismatch | 0s | Data integrity issue |
| **SLOViolation** | <95% SLO compliance | 5m | Escalate to SRE |

### Warning (Monitor)

| Alert | Trigger | Duration | Action |
|-------|---------|----------|--------|
| **FiberExecutionNearLimit** | >7 ticks | 5m | Watch for parking |
| **HighParkRate** | >20% park rate | 3m | Possible undersized budget |
| **BeatPulseIrregular** | >±10% pulse ratio | 2m | Beat system degraded |

### System Health

| Alert | Trigger | Duration | Action |
|-------|---------|----------|--------|
| **ContainerCrashLoop** | >0.1 restarts/10m | 5m | Check container logs |
| **HighMemoryUsage** | >85% memory | 5m | Risk of OOM |
| **OTELCollectorDown** | Collector unreachable | 1m | Observability compromised |

## 📁 Directory Structure

```
monitoring/
├── README.md                           # This file
├── docker-compose.monitoring.yml       # Monitoring services
├── prometheus/
│   ├── prometheus.yml                  # Prometheus config
│   └── alerts.yml                      # Alerting rules (15 rules)
├── grafana/
│   ├── provisioning/
│   │   ├── datasources/
│   │   │   └── prometheus.yml          # Auto-provision Prometheus DS
│   │   └── dashboards/
│   │       └── default.yml             # Auto-provision dashboards
│   └── dashboards/
│       ├── knhk-beat-system.json       # Beat monitoring dashboard
│       ├── knhk-performance-slo.json   # Performance & SLO dashboard
│       └── knhk-receipts.json          # Receipt metrics dashboard
├── alertmanager/
│   └── alertmanager.yml                # Alertmanager config
└── scripts/
    ├── start_monitoring.sh             # Start full stack
    ├── stop_monitoring.sh              # Stop stack
    └── query_metrics.sh                # CLI metrics query
```

## 🔧 Configuration

### Change Grafana Password

Edit `docker-compose.monitoring.yml`:
```yaml
environment:
  - GF_SECURITY_ADMIN_PASSWORD=<your-strong-password>
```

### Configure Alert Notifications

Edit `alertmanager/alertmanager.yml`:

```yaml
receivers:
  - name: 'knhk-critical'
    email_configs:
      - to: 'oncall@example.com'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK'
        channel: '#knhk-critical'
    pagerduty_configs:
      - service_key: 'your-pagerduty-key'
```

### Adjust Scrape Interval

Edit `prometheus/prometheus.yml`:
```yaml
global:
  scrape_interval: 15s  # Change as needed
```

## 🧪 Testing

### Verify Metrics Export

```bash
# Check OTEL Collector exports metrics
curl http://localhost:8888/metrics | grep knhk

# Check Prometheus scrapes successfully
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | select(.labels.job=="otel-collector")'

# Query specific metric
curl -G http://localhost:9090/api/v1/query \
  --data-urlencode 'query=knhk_beat_cycles_total' | jq
```

### Simulate Alert

```bash
# Trigger R1 violation alert (simulate slow operation)
# This would come from actual KNHK instrumentation exceeding 8 ticks

# Check pending alerts
curl http://localhost:9090/api/v1/alerts | jq '.data.alerts[] | select(.state=="pending" or .state=="firing")'
```

### Validate Dashboards

1. Open Grafana: http://localhost:3000
2. Navigate to Dashboards
3. Verify 3 dashboards appear:
   - KNHK Beat System (8-Beat Epoch)
   - KNHK Performance & SLO Compliance
   - KNHK Receipt Metrics
4. Check panels load without errors

## 📊 Key Metrics

### Beat System
- `knhk_beat_cycles_total` - Total beat cycles
- `knhk_beat_pulses_total` - Commit pulses (every 8th tick)

### Performance
- `knhk_operation_duration` - Operation latency (histogram)
- `knhk_operation_r1_violations_total` - Hot path violations
- `knhk_fiber_ticks_per_unit` - Fiber execution time

### Throughput
- `knhk_fiber_deltas_processed_total` - Delta processing rate
- `knhk_fiber_park_rate` - W1 parking rate (gauge 0.0-1.0)

### Correctness
- `knhk_receipt_created_total` - Receipts created
- `knhk_receipt_validated_total` - Receipts validated
- `knhk_receipt_hash_mismatches_total` - Hash failures (CRITICAL)

## 🐛 Troubleshooting

### No Metrics in Grafana

1. **Check OTEL Collector is exporting**:
   ```bash
   docker logs knhk-test-otel-collector | grep prometheus
   curl http://localhost:8888/metrics | head
   ```

2. **Check Prometheus is scraping**:
   ```bash
   curl http://localhost:9090/targets
   ```

3. **Verify metric names**:
   ```bash
   curl http://localhost:9090/api/v1/label/__name__/values | jq
   ```

### Dashboards Show "No Data"

1. **Check time range**: Ensure dashboard time range includes recent data
2. **Check data source**: Grafana → Configuration → Data Sources → Test
3. **Check metric queries**: Open panel editor, verify PromQL syntax

### Alerts Not Firing

1. **Check alert rules loaded**:
   ```bash
   curl http://localhost:9090/api/v1/rules | jq
   ```

2. **Verify alert evaluation**:
   - Open Prometheus → Alerts
   - Check pending/firing status

3. **Check Alertmanager connection**:
   ```bash
   curl http://localhost:9093/-/healthy
   ```

### High Memory Usage

1. **Reduce retention**: Lower `--storage.tsdb.retention.time` in docker-compose
2. **Increase scrape interval**: Scrape less frequently
3. **Limit cardinality**: Review label cardinality in metrics

## 📚 Documentation

- **Full Setup Guide**: `/docs/v1-monitoring-setup-guide.md`
- **Prometheus**: https://prometheus.io/docs/
- **Grafana**: https://grafana.com/docs/
- **OpenTelemetry**: https://opentelemetry.io/docs/

## 🔒 Security

### Production Checklist

- [ ] Change default Grafana password
- [ ] Configure TLS (use reverse proxy)
- [ ] Set up authentication (LDAP/OAuth)
- [ ] Enable audit logging
- [ ] Configure alert notifications
- [ ] Restrict network access
- [ ] Set up backups
- [ ] Review retention policies

### Network Security

By default, services bind to `0.0.0.0`. For production:

1. Use internal Docker network
2. Expose only necessary ports
3. Use reverse proxy (nginx/Caddy) with TLS
4. Enable Prometheus/Grafana authentication

## 🎯 SLO Definition

**KNHK v1.0 Service Level Objective**:

> **≥95% of R1 (hot path) operations must complete within 8 ticks (Chatman Constant)**

- **Measurement Window**: 5 minutes
- **Alert Threshold**: <95% compliance over 5 minutes
- **Critical Threshold**: <90% compliance (immediate escalation)

**Rationale**: 8-tick budget ensures predictable latency for real-time graph operations while allowing occasional complex queries to exceed budget (parked to W1).

## 📞 Support

For issues or questions:
1. Check this README
2. Review `/docs/v1-monitoring-setup-guide.md`
3. Check container logs: `docker-compose logs -f`
4. Open GitHub issue with monitoring stack logs

---

**KNHK v1.0 Monitoring Infrastructure** | Built with Prometheus, Grafana & Alertmanager
