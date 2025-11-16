# KNHK Production Deployment Guide

This directory contains all production deployment artifacts, configurations, and operational procedures for the Knowledge-Native Hyper-Kernel (KNHK) system.

## Directory Structure

```
deployment/
├── kubernetes/          # Kubernetes manifests
│   ├── deployment.yaml         # Main application deployment
│   ├── postgres-statefulset.yaml  # PostgreSQL HA setup
│   └── ...
├── monitoring/          # Monitoring and observability
│   ├── prometheus-config.yaml     # Prometheus configuration
│   ├── grafana-dashboard-overview.json  # Grafana dashboards
│   └── ...
├── security/           # Security configurations
│   ├── oidc-config.yaml          # Authentication setup
│   └── ...
├── runbooks/           # Operational runbooks
│   ├── database-connection-pool-exhausted.md
│   ├── receipt-chain-integrity-failure.md
│   └── ...
├── scripts/            # Deployment and operational scripts
│   ├── blue-green-deploy.sh     # Zero-downtime deployment
│   ├── rollback.sh              # Emergency rollback
│   ├── smoke-test.sh            # Post-deployment validation
│   └── ...
├── postgres/           # PostgreSQL configurations
│   └── ...
└── performance/        # Load testing and benchmarking
    └── ...
```

## Quick Start

### Prerequisites

1. **Kubernetes cluster** (v1.24+)
2. **kubectl** configured with cluster access
3. **Vault** for secrets management
4. **Prometheus & Grafana** for monitoring
5. **Weaver** for OpenTelemetry validation

### Initial Deployment

```bash
# 1. Create namespace
kubectl create namespace knhk

# 2. Deploy PostgreSQL (with Patroni HA)
kubectl apply -f kubernetes/postgres-statefulset.yaml

# 3. Deploy Redis
kubectl apply -f kubernetes/redis-statefulset.yaml

# 4. Deploy KNHK application
kubectl apply -f kubernetes/deployment.yaml

# 5. Deploy monitoring
kubectl apply -f monitoring/prometheus-config.yaml
kubectl apply -f monitoring/grafana-dashboard-overview.json

# 6. Verify deployment
kubectl get pods -n knhk
kubectl logs -f deploy/knhk-closed-loop -n knhk
```

### Blue-Green Deployment

```bash
# Deploy new version with zero downtime
./scripts/blue-green-deploy.sh v1.3.0

# If issues occur, rollback immediately
./scripts/rollback.sh
```

## Key Files

### Kubernetes Manifests

- **deployment.yaml**: Main KNHK application deployment with:
  - 3 replicas for high availability
  - Health probes (liveness, readiness, startup)
  - Resource limits based on profiling
  - Vault secrets integration
  - Prometheus metrics endpoint

- **postgres-statefulset.yaml**: PostgreSQL with Patroni for automated failover:
  - Primary + 1 replica
  - Synchronous replication (RPO=0)
  - Automated failover (RTO <30s)

### Monitoring

- **prometheus-config.yaml**: Scrapes metrics from:
  - KNHK application (hot path ticks, warm path latency)
  - PostgreSQL (connection pool, query performance)
  - Redis (cache hit rate, memory usage)
  - Kubernetes (pod/node metrics)

- **grafana-dashboard-overview.json**: Production dashboard showing:
  - Request rate and error rate
  - Hot path latency (CPU ticks)
  - Warm path latency (ms)
  - Database connection pool utilization
  - MAPE-K loop cycles
  - Receipt chain integrity
  - Weaver validation status

### Security

- **oidc-config.yaml**: OpenID Connect authentication with:
  - Multiple IdP support (Okta, Azure AD, Keycloak)
  - Role-based access control (RBAC)
  - Session management
  - Token validation

### Runbooks

Operational procedures for common failure scenarios:

- **database-connection-pool-exhausted.md**: Diagnose and fix connection pool issues
- **receipt-chain-integrity-failure.md**: CRITICAL - handle audit trail corruption

See [runbooks/](runbooks/) for full list.

### Scripts

- **blue-green-deploy.sh**: Zero-downtime deployment with automated rollback
- **rollback.sh**: Emergency rollback to previous version
- **smoke-test.sh**: Validates deployment health

## Operational Procedures

### Deploying a New Version

1. **Pre-deployment**:
   ```bash
   # Run full test suite
   cargo test --workspace

   # Verify Weaver validation
   weaver registry check -r registry/

   # Run load tests on staging
   k6 run performance/load-test.js
   ```

2. **Deployment**:
   ```bash
   # Blue-green deployment
   ./scripts/blue-green-deploy.sh v1.4.0
   ```

3. **Post-deployment**:
   ```bash
   # Monitor dashboards for 5 minutes
   # Check error rate, latency, receipt chain integrity
   # Verify Weaver validation passing
   ```

### Emergency Procedures

**System Outage**:
```bash
# Check pod status
kubectl get pods -n knhk

# Check logs
kubectl logs -f deploy/knhk-closed-loop -n knhk

# Scale up if resource exhaustion
kubectl scale deployment knhk-closed-loop --replicas=6 -n knhk
```

**Database Issues**:
```bash
# Check database health
kubectl exec -it postgres-0 -n knhk -- psql -U postgres -c "SELECT 1;"

# Trigger manual failover (if needed)
kubectl exec -it postgres-0 -n knhk -- \
    patronictl -c /etc/patroni/patroni.yaml failover
```

**Receipt Chain Integrity Failure** (CRITICAL):
```bash
# STOP ALL WRITES IMMEDIATELY
kubectl scale deployment knhk-closed-loop --replicas=0 -n knhk

# Create emergency backup
pgbackrest --stanza=knhk --type=full backup

# ESCALATE TO ENGINEERING MANAGER + SECURITY LEAD
# See runbooks/receipt-chain-integrity-failure.md
```

## Monitoring & Alerting

### Key Metrics

| Metric | Target | Critical Threshold |
|--------|--------|-------------------|
| Availability | ≥99.9% | <99% |
| Error Rate | <0.1% | >1% |
| Hot Path Latency | ≤8 ticks | >10 ticks |
| Warm Path Latency (p95) | <100ms | >200ms |
| DB Pool Utilization | <80% | >95% |

### Alert Escalation

- **P0 (Critical)**: Page on-call immediately → Escalate to manager after 30 min
- **P1 (High)**: Page on-call → Escalate after 1 hour
- **P2 (Medium)**: Slack notification → 4 hour response SLA
- **P3 (Low)**: Ticket only → 24 hour response SLA

### Dashboards

- **Production Overview**: http://grafana.company.com/d/knhk-overview
- **Database Performance**: http://grafana.company.com/d/knhk-postgres
- **MAPE-K Loop**: http://grafana.company.com/d/knhk-mape-k

## Compliance & Governance

### Data Retention

- **Receipts**: 7 years (immutable, never delete)
- **Observations**: 90 days → archive to S3 Glacier
- **Snapshots**: 1 year
- **Audit Logs**: 7 years (immutable)
- **Telemetry**: 30 days

### Backup Schedule

- **Full backup**: Weekly (Sunday 02:00 UTC)
- **Incremental backup**: Daily (02:00 UTC)
- **Continuous WAL archiving**: Every 5 minutes
- **Backup verification**: Monthly restore test

### Security Requirements

- **Authentication**: OIDC with MFA required
- **Encryption at rest**: PostgreSQL TDE, S3 SSE-KMS
- **Encryption in transit**: TLS 1.3 for all connections
- **Secrets management**: Vault with auto-rotation
- **Audit logging**: All access logged with ed25519 signatures

## Troubleshooting

### Common Issues

**Pods in CrashLoopBackOff**:
```bash
# Check logs
kubectl logs <pod-name> -n knhk

# Common causes:
# - Database connection failed (check DATABASE_URL secret)
# - Redis connection failed (check REDIS_URL)
# - Vault secrets not available (check Vault agent injection)
```

**High Latency**:
```bash
# Check metrics
kubectl exec -it deploy/knhk-closed-loop -n knhk -- \
    curl localhost:9090/metrics | grep duration

# Profile hot path
cargo bench --bench hot_path

# Check database slow queries
kubectl exec -it postgres-0 -n knhk -- \
    psql -U postgres -c "SELECT query, mean_time FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;"
```

**Weaver Validation Failures**:
```bash
# Check schema
weaver registry check -r registry/

# Common causes:
# - Missing span attributes in code
# - Type mismatch (string vs int)
# - Missing metric definitions
```

## Support

- **Documentation**: See [/docs/SPARC_PHASE9_PRODUCTION_HARDENING.md](/docs/SPARC_PHASE9_PRODUCTION_HARDENING.md)
- **Runbooks**: See [runbooks/](runbooks/)
- **Issues**: File bug in GitHub repository
- **On-call**: PagerDuty +1-XXX-XXX-XXXX
- **Slack**: #knhk-production

---

**Last Updated**: 2025-11-16
**Version**: 1.0.0
