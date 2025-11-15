# Deployment Checklist

**Purpose**: One-page production deployment verification
**Target**: Zero-downtime deployment with rollback capability
**Validation**: All pre-deployment checks must pass

---

## Pre-Deployment (MANDATORY)

**All items must pass before deploying:**

### 1. Build & Test Validation

- [ ] `cargo build --workspace --release` succeeds (zero warnings)
- [ ] `cargo clippy --workspace -- -D warnings` passes (zero issues)
- [ ] `cargo test --workspace` passes (100% success)
- [ ] `make test-chicago-v04` passes (C hot path)
- [ ] `make test-performance-v04` passes (≤8 ticks verified)
- [ ] `make test-integration-v2` passes (integration)
- [ ] `make test-enterprise` passes (enterprise use cases)

**Pass Criteria**: All 7 items checked ✅

---

### 2. Weaver Validation (MANDATORY)

- [ ] **`weaver registry check -r /home/user/knhk/registry/` passes** (schema valid)
- [ ] **`weaver registry live-check --registry /home/user/knhk/registry/` passes** (runtime valid)
- [ ] All telemetry endpoints tested (OTLP collector reachable)
- [ ] Telemetry overhead measured (≤5% CPU, ≤10MB memory)

**Pass Criteria**: All 4 items checked ✅ - First 2 are MANDATORY

---

### 3. Security & Configuration

- [ ] No secrets in code/config (use environment variables/secrets manager)
- [ ] TLS certificates valid and not expiring soon (>30 days)
- [ ] Authentication/authorization configured
- [ ] Rate limiting configured (protect against abuse)
- [ ] Resource limits set (CPU, memory, connections)
- [ ] Security scanning completed (`cargo audit`)
- [ ] Dependency updates reviewed (no critical vulnerabilities)
- [ ] Firewall rules configured (only required ports open)

**Pass Criteria**: All 8 items checked ✅

---

### 4. Infrastructure Readiness

- [ ] Docker image built and pushed to registry
- [ ] Kubernetes manifests validated (`kubectl apply --dry-run`)
- [ ] Database migrations tested (rollback capability verified)
- [ ] Health check endpoints implemented (`/health`, `/ready`)
- [ ] Graceful shutdown implemented (SIGTERM handling)
- [ ] Resource quotas configured (CPU, memory, storage)
- [ ] Persistent volumes configured (for state storage)
- [ ] Load balancer configured (traffic distribution)

**Pass Criteria**: All 8 items checked ✅

---

### 5. Monitoring & Alerting

- [ ] OTLP collector configured and tested
- [ ] Prometheus metrics scraped successfully
- [ ] Grafana dashboards deployed (service health, performance)
- [ ] Alert rules defined (error rate, latency, resource usage)
- [ ] Alert routing configured (PagerDuty, Slack, email)
- [ ] Log aggregation configured (ELK, Loki, CloudWatch)
- [ ] Distributed tracing configured (Jaeger, Zipkin)
- [ ] SLO/SLA thresholds defined (99.9% uptime, p99 latency)

**Pass Criteria**: All 8 items checked ✅

---

### 6. Backup & Recovery

- [ ] Database backup configured (automated, tested)
- [ ] Backup restoration tested (RTO ≤4 hours)
- [ ] Configuration backup stored (version controlled)
- [ ] Disaster recovery plan documented
- [ ] Rollback procedure tested (can revert to previous version)
- [ ] Data retention policy defined (logs, metrics, backups)
- [ ] Off-site backup configured (geo-redundancy)
- [ ] Recovery tested in staging environment

**Pass Criteria**: All 8 items checked ✅

---

### 7. Documentation & Communication

- [ ] Deployment guide updated (step-by-step instructions)
- [ ] Runbook updated (troubleshooting, common issues)
- [ ] Change log updated (what's new, breaking changes)
- [ ] API documentation updated (if API changes)
- [ ] Team notified (deployment window, expected downtime)
- [ ] Stakeholders notified (release notes, impact)
- [ ] Support team briefed (new features, potential issues)
- [ ] Release notes published (user-facing changes)

**Pass Criteria**: All 8 items checked ✅

---

### 8. Staging Validation

- [ ] Deployed to staging environment
- [ ] Smoke tests passed in staging (basic functionality)
- [ ] Performance tests passed in staging (load testing)
- [ ] Integration tests passed in staging (end-to-end)
- [ ] Rollback tested in staging (can revert successfully)
- [ ] Monitoring validated in staging (metrics, logs, traces)
- [ ] Security scanning passed in staging
- [ ] Staging environment matches production (same config)

**Pass Criteria**: All 8 items checked ✅

---

## During Deployment

**Execute deployment steps carefully:**

### 9. Deployment Execution

- [ ] **Blue-green deployment** or **canary release** strategy
- [ ] Traffic shifted gradually (10% → 50% → 100%)
- [ ] Health checks monitored during rollout
- [ ] Error rates monitored (should not spike)
- [ ] Latency monitored (p99 should stay within SLO)
- [ ] Rollback triggered if error rate >1% increase
- [ ] Database migrations executed (if any)
- [ ] Configuration updated (if any)

**Pass Criteria**: All 8 items checked ✅

**Deployment Commands**:
```bash
# 1. Build and push Docker image
docker build -t knhk:v1.0.0 .
docker push registry.example.com/knhk:v1.0.0

# 2. Deploy to Kubernetes (blue-green)
kubectl apply -f k8s/deployment-green.yaml
kubectl wait --for=condition=ready pod -l app=knhk,version=green

# 3. Shift traffic gradually
kubectl patch service knhk -p '{"spec":{"selector":{"version":"green"}}}'

# 4. Monitor health
kubectl logs -l app=knhk,version=green --tail=100 -f
```

---

### 10. Monitoring During Deployment

**Watch for issues in real-time:**

- [ ] CPU usage stable (not spiking)
- [ ] Memory usage stable (no leaks)
- [ ] Error rate stable (≤0.1%)
- [ ] Latency stable (p99 within SLO)
- [ ] Health checks passing (100%)
- [ ] Database connections stable (no exhaustion)
- [ ] Queue depth stable (no backlog)
- [ ] Network I/O stable (no bottlenecks)

**Pass Criteria**: All 8 items checked ✅

**Monitoring Commands**:
```bash
# Watch pod health
watch kubectl get pods -l app=knhk

# Watch metrics
kubectl top pods -l app=knhk

# Check logs for errors
kubectl logs -l app=knhk | grep -i error

# Check service endpoints
kubectl get endpoints knhk
```

---

## Post-Deployment

**Verify successful deployment:**

### 11. Post-Deployment Validation

- [ ] Smoke tests executed in production (basic functionality)
- [ ] Health checks passing (all instances healthy)
- [ ] Metrics validated (CPU, memory, latency within normal range)
- [ ] Error logs reviewed (no new critical errors)
- [ ] User-facing features tested (key workflows work)
- [ ] Performance validated (≤8 ticks for hot path)
- [ ] Telemetry flowing to OTLP collector
- [ ] Alerts not firing (no incidents)

**Pass Criteria**: All 8 items checked ✅

**Validation Commands**:
```bash
# Smoke test
curl -f http://production.example.com/health || echo "FAILED"

# Check metrics
curl http://production.example.com/metrics | grep knhk_query_latency

# Check logs
kubectl logs -l app=knhk --tail=100 | grep -v INFO

# Run integration test
make test-integration-v2 ENDPOINT=http://production.example.com
```

---

### 12. Monitoring & Alerting Verification

- [ ] Prometheus scraping new version (check `/metrics`)
- [ ] Grafana dashboards showing new version metrics
- [ ] Distributed tracing working (spans visible in Jaeger)
- [ ] Logs aggregated and searchable (ELK, Loki)
- [ ] Alerts configured and tested (trigger test alert)
- [ ] SLO compliance verified (99.9% uptime, p99 latency)
- [ ] On-call runbook accessible (troubleshooting guide)
- [ ] Incident response tested (can page on-call)

**Pass Criteria**: All 8 items checked ✅

---

### 13. Cleanup & Documentation

- [ ] Old version decommissioned (if blue-green)
- [ ] Old Docker images archived (or deleted)
- [ ] Temporary resources cleaned up (staging pods, test data)
- [ ] Deployment log updated (who, when, what)
- [ ] Post-deployment report written (issues, lessons learned)
- [ ] Team notified of successful deployment
- [ ] Release tagged in Git (`git tag v1.0.0`)
- [ ] Deployment marked complete in ticketing system

**Pass Criteria**: All 8 items checked ✅

**Cleanup Commands**:
```bash
# Tag release
git tag -a v1.0.0 -m "Production release v1.0.0"
git push origin v1.0.0

# Delete old deployment
kubectl delete deployment knhk-blue

# Clean up old images
docker image prune -a --filter "until=720h"
```

---

## Rollback Triggers

**Automatically rollback if ANY of these occur:**

- [ ] Error rate >1% increase from baseline
- [ ] p99 latency >2x baseline
- [ ] Health checks failing for >1 minute
- [ ] CPU usage >90% sustained for >5 minutes
- [ ] Memory usage >90% sustained for >5 minutes
- [ ] Database connection errors >10/minute
- [ ] Critical alerts firing (data loss, security breach)
- [ ] Manual rollback requested (by team lead)

**Rollback Procedure**:
```bash
# 1. Shift traffic back to old version
kubectl patch service knhk -p '{"spec":{"selector":{"version":"blue"}}}'

# 2. Verify old version healthy
kubectl get pods -l app=knhk,version=blue

# 3. Delete new version
kubectl delete deployment knhk-green

# 4. Notify team
echo "ROLLBACK: Reverted to v0.9.0 due to error rate spike" | slack_notify
```

---

## Deployment Strategy Options

### Blue-Green Deployment (Recommended)

- **Pros**: Instant rollback, zero downtime, full testing before switch
- **Cons**: 2x infrastructure cost during deployment
- **Use When**: Production-critical, need instant rollback

### Canary Release

- **Pros**: Gradual rollout, early detection of issues, lower risk
- **Cons**: More complex, requires advanced traffic routing
- **Use When**: Large user base, want to minimize blast radius

### Rolling Update

- **Pros**: Simple, no extra infrastructure, built into Kubernetes
- **Cons**: Slower rollback, mixed versions during deployment
- **Use When**: Non-critical services, simple deployments

---

## Final Sign-Off

- [ ] **All 13 sections completed** (104 total checks)
- [ ] **Pre-deployment checks passed** (56 items)
- [ ] **Deployment executed successfully** (16 items)
- [ ] **Post-deployment validated** (24 items)
- [ ] **Monitoring & alerting working** (8 items)
- [ ] **Team notified of deployment**
- [ ] **Rollback tested and ready**

**Deployment Approved By**: ________________
**Date**: ________________
**Version Deployed**: ________________
**Rollback Plan**: ________________

---

**See Also**:
- [Production Guide](/home/user/knhk/docs/PRODUCTION.md)
- [Production Readiness Checklist](/home/user/knhk/docs/reference/cards/PRODUCTION_READINESS_CHECKLIST.md)
- [Monitoring & Alerting Guide](/home/user/knhk/docs/INTEGRATION.md)
