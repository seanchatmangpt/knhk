# Runbook: Database Connection Pool Exhausted

**Severity**: P1 (High)
**Owner**: SRE Team
**Last Updated**: 2025-11-16

## Symptoms

- API returning 503 Service Unavailable
- Error logs: "connection pool timeout" or "too many clients"
- Prometheus alert: `knhk_db_pool_active / knhk_db_pool_total > 0.95`
- Increased API latency (p95 >500ms)

## Diagnosis

### 1. Check Pool Metrics

```bash
# Check pool metrics from application
kubectl exec -it deploy/knhk-closed-loop -- \
    curl localhost:9090/metrics | grep knhk_db_pool

# Expected output:
# knhk_db_pool_total 50
# knhk_db_pool_active 48  # ← High utilization
# knhk_db_pool_idle 2
```

### 2. Check Database Connections

```bash
# Check active connections in PostgreSQL
kubectl exec -it postgres-0 -- \
    psql -U postgres -c "
        SELECT count(*) as total_connections,
               count(*) FILTER (WHERE state = 'active') as active,
               count(*) FILTER (WHERE state = 'idle') as idle
        FROM pg_stat_activity
        WHERE datname = 'knhk';
    "

# Check for connection leaks (long-running idle connections)
kubectl exec -it postgres-0 -- \
    psql -U postgres -c "
        SELECT pid,
               now() - state_change AS duration,
               state,
               query
        FROM pg_stat_activity
        WHERE datname = 'knhk'
          AND state = 'idle in transaction'
        ORDER BY duration DESC
        LIMIT 10;
    "
```

### 3. Check Application Logs

```bash
# Look for connection timeout errors
kubectl logs deploy/knhk-closed-loop --tail=100 | \
    grep -i "connection\|pool\|timeout"

# Check for slow queries
kubectl logs deploy/knhk-closed-loop --tail=100 | \
    grep -i "slow query\|duration_ms"
```

## Resolution

### Immediate (Within 5 minutes)

**Option 1: Scale Up Application Replicas** (distributes load)

```bash
# Increase replicas to distribute connection load
kubectl scale deployment knhk-closed-loop --replicas=6

# Monitor pool utilization
watch -n 2 "kubectl exec -it deploy/knhk-closed-loop -- \
    curl -s localhost:9090/metrics | grep knhk_db_pool"
```

**Option 2: Kill Idle Connections** (if connection leaks detected)

```bash
# Terminate idle connections older than 5 minutes
kubectl exec -it postgres-0 -- \
    psql -U postgres -c "
        SELECT pg_terminate_backend(pid)
        FROM pg_stat_activity
        WHERE datname = 'knhk'
          AND state = 'idle in transaction'
          AND now() - state_change > interval '5 minutes';
    "
```

### Short-Term (Within 1 hour)

**Increase Pool Size** (requires application restart)

```bash
# Update ConfigMap with new pool size
kubectl patch configmap knhk-config -p \
    '{"data":{"DB_POOL_MAX_CONNECTIONS":"100"}}'

# Rolling restart to pick up new config
kubectl rollout restart deployment/knhk-closed-loop

# Wait for rollout to complete
kubectl rollout status deployment/knhk-closed-loop --timeout=5m

# Verify new pool size
kubectl exec -it deploy/knhk-closed-loop -- \
    curl -s localhost:9090/metrics | grep knhk_db_pool_total
# Should show: knhk_db_pool_total 100
```

### Long-Term (Within 1 week)

**Identify Connection Leaks**

```bash
# Enable connection tracing in application
kubectl set env deployment/knhk-closed-loop RUST_LOG=sqlx=trace

# Monitor for connections that are acquired but never released
kubectl logs -f deploy/knhk-closed-loop | grep "connection acquired\|connection released"

# Common causes:
# - Missing .await on async queries
# - Transactions not committed/rolled back
# - Panic in query handler (connection not returned)
```

**Implement Connection Monitoring**

Add to Rust application:
```rust
// Periodic connection pool health check
tokio::spawn(async move {
    loop {
        let pool_size = pool.size();
        let idle = pool.num_idle();
        let active = pool_size - idle;

        // Alert if utilization >90%
        if active as f64 / pool_size as f64 > 0.9 {
            tracing::warn!(
                "connection.pool.high_utilization",
                active = active,
                total = pool_size,
                utilization = active as f64 / pool_size as f64,
            );
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
});
```

**Consider PgBouncer** (for very high connection counts)

```yaml
# deployment/kubernetes/pgbouncer.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pgbouncer
spec:
  replicas: 2
  template:
    spec:
      containers:
      - name: pgbouncer
        image: pgbouncer/pgbouncer:latest
        env:
        - name: DATABASES_HOST
          value: postgres-primary
        - name: POOL_MODE
          value: transaction
        - name: MAX_CLIENT_CONN
          value: "1000"
        - name: DEFAULT_POOL_SIZE
          value: "25"
```

## Validation

After resolution, verify:

```bash
# 1. Pool utilization is healthy (<80%)
kubectl exec -it deploy/knhk-closed-loop -- \
    curl -s localhost:9090/metrics | grep knhk_db_pool

# 2. API latency returned to normal
curl -s http://prometheus:9090/api/v1/query?query='histogram_quantile(0.95, rate(knhk_request_duration_ms_bucket[5m]))' | jq

# 3. Error rate dropped
curl -s http://prometheus:9090/api/v1/query?query='rate(knhk_requests_total{status="error"}[5m])' | jq

# 4. No connection timeout errors in last 5 minutes
kubectl logs deploy/knhk-closed-loop --since=5m | grep -i "connection timeout"
```

## Prevention

1. **Set Aggressive Timeouts**
   ```rust
   PgPoolOptions::new()
       .idle_timeout(Duration::from_secs(60))      // Close idle connections
       .max_lifetime(Duration::from_secs(1800))    // Recycle old connections
       .acquire_timeout(Duration::from_secs(5))    // Fail fast
   ```

2. **Add Connection Pool Alerts**
   ```yaml
   # Prometheus alert
   - alert: KNHKDatabasePoolHighUtilization
     expr: knhk_db_pool_active / knhk_db_pool_total > 0.80
     for: 5m
     labels:
       severity: warning
   ```

3. **Regular Load Testing**
   ```bash
   # Run monthly load tests to find connection bottlenecks
   k6 run deployment/performance/load-test.js
   ```

4. **Code Review Checklist**
   - [ ] All database queries use `.await`
   - [ ] All transactions are committed or rolled back
   - [ ] No connections held across async boundaries
   - [ ] Connection pool metrics emitted

## Escalation

- **If not resolved in 30 minutes**: Escalate to Lead Engineer
- **If data loss risk**: Escalate to Engineering Manager immediately
- **If customer-facing**: Update status page and notify Customer Success

## Related Runbooks

- [PostgreSQL High CPU Usage](postgres-high-cpu.md)
- [Slow Query Investigation](slow-queries.md)
- [Database Failover Procedure](postgres-failover.md)

## Post-Incident

After incident is resolved:

1. **Create Post-Incident Review** (PIR)
   - Root cause analysis
   - Timeline of events
   - What went well / What went poorly
   - Action items to prevent recurrence

2. **Update Monitoring**
   - Adjust alert thresholds if needed
   - Add new metrics if gaps identified

3. **Update Runbook**
   - Document new learnings
   - Add resolution steps that worked
   - Update prevention measures
