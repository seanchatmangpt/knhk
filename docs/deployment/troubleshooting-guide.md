# KNHK Production Troubleshooting Guide

**Purpose**: Diagnose and resolve common production issues with KNHK.

**Principle**: Follow the diagnostic hierarchy: Weaver → Telemetry → Logs → Code

---

## Table of Contents

1. [Diagnostic Hierarchy](#diagnostic-hierarchy)
2. [Common Issues](#common-issues)
3. [Performance Issues](#performance-issues)
4. [Weaver Validation Failures](#weaver-validation-failures)
5. [Telemetry Issues](#telemetry-issues)
6. [Workflow Execution Issues](#workflow-execution-issues)
7. [Emergency Procedures](#emergency-procedures)

---

## Diagnostic Hierarchy

When troubleshooting KNHK issues, follow this hierarchy:

```
1. Weaver Validation (Source of Truth)
   ↓ If Weaver fails, runtime doesn't match schema

2. Telemetry Analysis
   ↓ If telemetry shows anomalies, check metrics/traces

3. Application Logs
   ↓ If logs show errors, check error messages

4. Code Analysis
   ↓ If code has bugs, fix and redeploy
```

**Key Rule**: Always start with Weaver validation. Don't trust anything else until Weaver passes.

---

## Common Issues

### Issue 1: Service Won't Start

**Symptoms**:
```bash
$ systemctl start knhk
$ systemctl status knhk
● knhk.service - KNHK Knowledge Hook System
   Loaded: loaded
   Active: failed (Result: exit-code)
```

**Diagnostic Steps**:

```bash
# 1. Check service logs
sudo journalctl -u knhk -n 100 --no-pager

# 2. Check binary exists
ls -la /usr/local/bin/knhk

# 3. Try manual start
/usr/local/bin/knhk server start

# 4. Check environment variables
env | grep -E 'OTEL|KNHK|RUST'

# 5. Check dependencies
ldd /usr/local/bin/knhk
```

**Common Causes**:

| Error | Cause | Solution |
|-------|-------|----------|
| "No such file or directory" | Binary not deployed | `sudo cp target/release/knhk /usr/local/bin/` |
| "Permission denied" | Wrong permissions | `sudo chmod +x /usr/local/bin/knhk` |
| "Cannot open shared library" | Missing C library | `sudo cp c/lib/libknhk.so /usr/local/lib/ && sudo ldconfig` |
| "OTEL_EXPORTER_OTLP_ENDPOINT not set" | Missing env vars | Set in systemd unit or /etc/environment |

### Issue 2: High Memory Usage

**Symptoms**:
```bash
$ top
  PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM
  1234 knhk      20   0   4.2g   3.8g    20m S   5.0  48.0  knhk
```

**Diagnostic Steps**:

```bash
# 1. Check memory metrics
curl http://localhost:8889/metrics | grep memory

# 2. Check for memory leaks
valgrind --leak-check=full /usr/local/bin/knhk server start

# 3. Check workflow state size
ls -lh /var/lib/knhk/state/

# 4. Check lockchain size
du -sh /var/lib/knhk/lockchain/
```

**Solutions**:

1. **Limit workflow state**:
   ```bash
   # Set max concurrent workflows
   export KNHK_MAX_CONCURRENT_WORKFLOWS=100
   ```

2. **Clean old receipts**:
   ```bash
   # Archive receipts older than 30 days
   find /var/lib/knhk/lockchain/ -mtime +30 -exec gzip {} \;
   ```

3. **Increase system memory**:
   ```bash
   # Add swap if needed
   sudo fallocate -l 4G /swapfile
   sudo chmod 600 /swapfile
   sudo mkswap /swapfile
   sudo swapon /swapfile
   ```

### Issue 3: High CPU Usage

**Symptoms**:
```bash
$ top
  PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM
  1234 knhk      20   0   2.1g   1.2g    20m R  95.0  15.0  knhk
```

**Diagnostic Steps**:

```bash
# 1. Check which operations are slow
curl http://localhost:8889/metrics | grep latency | sort -k2 -n

# 2. Profile CPU usage
perf record -p $(pidof knhk) sleep 30
perf report

# 3. Check for hot path violations
curl http://localhost:8889/metrics | grep chatman_compliance
```

**Solutions**:

1. **Check Chatman Constant compliance**:
   ```bash
   # Hot path ops MUST be ≤8 ticks
   make test-performance-v04
   ```

2. **Identify slow operations**:
   ```bash
   # Query telemetry for slow ops
   curl http://localhost:8889/metrics | grep "latency.*p99"
   ```

3. **Scale horizontally**:
   ```bash
   # Add more KNHK instances
   systemctl start knhk@2
   systemctl start knhk@3
   ```

---

## Performance Issues

### Chatman Constant Violations

**Symptom**: Hot path operations exceed 8 ticks (2ns)

**Diagnostic**:
```bash
# Check guard metrics
curl http://localhost:8889/metrics | grep knhk_guard_chatman_compliance

# Expected: knhk_guard_chatman_compliance 1.0
# If < 1.0: Some operations violating constraint
```

**Investigation**:
```bash
# Find which operations are slow
curl http://localhost:8889/metrics | \
  grep 'knhk_operation_latency.*{runtime_class="R1"}' | \
  awk '$2 > 8 {print}'
```

**Solutions**:

1. **Move slow operations to warm path**:
   ```rust
   // Change operation classification
   // From:
   #[runtime_class("R1")]  // Hot path
   // To:
   #[runtime_class("W1")]  // Warm path
   ```

2. **Optimize SIMD operations**:
   ```bash
   # Check if SIMD is enabled
   lscpu | grep -i avx

   # Rebuild with SIMD optimizations
   RUSTFLAGS="-C target-cpu=native" cargo build --release
   ```

3. **Profile hot path**:
   ```bash
   # Use perf to find bottlenecks
   cargo build --release
   perf record -g target/release/knhk_benchmark
   perf report
   ```

### High Latency

**Symptom**: P95/P99 latencies exceed SLO targets

**Diagnostic**:
```bash
# Check latency distribution
curl http://localhost:8889/metrics | \
  grep 'knhk.*latency.*quantile'

# Example output:
# knhk_workflow_latency{quantile="0.5"} 12.5
# knhk_workflow_latency{quantile="0.95"} 45.2
# knhk_workflow_latency{quantile="0.99"} 125.8
```

**Investigation**:
```bash
# Trace a slow request
curl http://localhost:16686/api/traces/<trace-id>

# Look for:
# - Long span durations
# - Excessive child spans
# - Blocking operations
```

**Solutions**:

1. **Enable async I/O**:
   ```rust
   // Use tokio for I/O operations
   #[tokio::main]
   async fn main() {
       // Async operations
   }
   ```

2. **Add caching**:
   ```rust
   // Cache frequently accessed data
   use lru::LruCache;
   let mut cache = LruCache::new(1000);
   ```

3. **Optimize database queries**:
   ```sql
   -- Add indexes
   CREATE INDEX idx_workflow_id ON workflows(id);
   CREATE INDEX idx_case_state ON cases(state);
   ```

---

## Weaver Validation Failures

### Schema Validation Fails

**Symptom**:
```bash
$ weaver registry check -r registry/
❌ Registry validation failed
   Error: Invalid attribute type at registry/knhk-workflow-engine.yaml:42
```

**Diagnostic**:
```bash
# Check schema file
cat registry/knhk-workflow-engine.yaml | head -50

# Validate individual files
for f in registry/*.yaml; do
    echo "Checking $f..."
    weaver registry check -r registry/ --schema $(basename $f)
done
```

**Common Schema Errors**:

1. **Invalid attribute type**:
   ```yaml
   # ❌ Wrong
   type: number

   # ✅ Correct
   type: int  # or double
   ```

2. **Missing required fields**:
   ```yaml
   # ❌ Wrong
   - id: knhk.op.execute
     type: span

   # ✅ Correct
   - id: knhk.op.execute
     type: span
     stability: experimental
     brief: "Execute operation"
   ```

3. **Undefined reference**:
   ```yaml
   # ❌ Wrong
   attributes:
     - ref: knhk.undefined.attr

   # ✅ Correct
   # First define attribute:
   - id: knhk.defined.attr
     type: string
   # Then reference it:
   attributes:
     - ref: knhk.defined.attr
   ```

### Live-Check Validation Fails

**Symptom**:
```bash
$ weaver registry live-check --registry registry/
❌ Live telemetry validation failed
   Violation: Declared span not found: knhk.workflow_engine.execute_case
```

**Diagnostic**:
```bash
# 1. Check if application is running
ps aux | grep knhk

# 2. Check if telemetry is being exported
curl http://localhost:8889/metrics | grep knhk

# 3. Check OTEL collector logs
docker logs otel-collector | grep knhk

# 4. Search code for span emission
grep -r "execute_case" rust/*/src
```

**Solutions**:

1. **Span not emitted (code missing)**:
   ```rust
   // Add span emission
   use opentelemetry::trace::{Tracer, TracerProvider};

   let span = tracer.start("knhk.workflow_engine.execute_case");
   // ... operation ...
   span.end();
   ```

2. **Span name mismatch**:
   ```rust
   // Code:
   span!("knhk.workflow_engine.execute-case")  // ❌ hyphen

   // Schema:
   # id: knhk.workflow_engine.execute_case  # ✅ underscore

   // Fix: Match schema exactly
   span!("knhk.workflow_engine.execute_case")
   ```

3. **Feature not executed**:
   ```bash
   # Execute the feature to trigger span
   ./target/release/knhk workflow execute test.ttl

   # Then rerun live-check
   weaver registry live-check --registry registry/
   ```

---

## Telemetry Issues

### No Telemetry Received

**Symptom**: OTEL collector shows no data from KNHK

**Diagnostic**:
```bash
# 1. Check OTEL environment
env | grep OTEL

# 2. Test collector endpoint
curl http://localhost:4318/v1/traces -I

# 3. Check collector logs
docker logs otel-collector 2>&1 | tail -50

# 4. Test telemetry export
cargo run --example emit_telemetry
```

**Solutions**:

1. **Wrong OTLP endpoint**:
   ```bash
   # Check current setting
   echo $OTEL_EXPORTER_OTLP_ENDPOINT

   # Set correct endpoint
   export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
   ```

2. **Collector not running**:
   ```bash
   # Start collector
   docker run -d \
     --name otel-collector \
     -p 4318:4318 \
     otel/opentelemetry-collector:latest
   ```

3. **Port conflict**:
   ```bash
   # Check port usage
   lsof -i :4318

   # Use alternative port
   export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4319"
   docker run -d -p 4319:4318 otel/opentelemetry-collector:latest
   ```

### Telemetry Export Delays

**Symptom**: Telemetry appears in backend with significant delay

**Diagnostic**:
```bash
# Check batch processor settings
docker exec otel-collector cat /etc/otelcol/config.yaml | \
  grep -A5 batch

# Check queue size
curl http://localhost:8888/metrics | grep queue_size
```

**Solutions**:

1. **Reduce batch timeout**:
   ```yaml
   # otel-collector-config.yaml
   processors:
     batch:
       timeout: 1s  # Reduce from default 5s
       send_batch_size: 512
   ```

2. **Increase worker pool**:
   ```yaml
   exporters:
     otlp:
       endpoint: backend:4317
       sending_queue:
         num_consumers: 10  # Increase from default 5
   ```

---

## Workflow Execution Issues

### Workflows Stuck in "Running" State

**Symptom**: Workflows never reach "Completed" state

**Diagnostic**:
```bash
# 1. Check workflow state
curl http://localhost:8080/api/v1/workflows/<id>/state

# 2. Check workflow history
curl http://localhost:8080/api/v1/workflows/<id>/history

# 3. Check for deadlocks
curl http://localhost:8889/metrics | grep workflow_deadlock
```

**Common Causes**:

1. **Missing end condition**:
   ```turtle
   # Workflow definition must have end condition
   :workflow a yawl:Specification ;
       yawl:startCondition :start ;
       yawl:endCondition :end .  # ← Must be defined
   ```

2. **Circular dependencies**:
   ```bash
   # Check workflow graph for cycles
   ./target/release/knhk workflow analyze workflow.ttl --check-cycles
   ```

3. **Resource deadlock**:
   ```bash
   # Check resource allocation
   curl http://localhost:8889/metrics | grep resource_utilization
   ```

### Workflow Execution Errors

**Symptom**: Workflows fail with errors

**Diagnostic**:
```bash
# 1. Check error logs
sudo journalctl -u knhk | grep ERROR

# 2. Check workflow validation
./target/release/knhk workflow validate workflow.ttl

# 3. Check telemetry for errors
curl http://localhost:8889/metrics | grep error_count
```

**Solutions**:

1. **Validate workflow syntax**:
   ```bash
   # Use KNHK validator
   ./target/release/knhk workflow validate workflow.ttl

   # Expected output:
   # ✅ Workflow syntax valid
   # ✅ All tasks have valid connectors
   # ✅ All conditions reachable
   ```

2. **Check connector configuration**:
   ```bash
   # List configured connectors
   ./target/release/knhk connector list

   # Test connector
   ./target/release/knhk connector test <connector-id>
   ```

---

## Emergency Procedures

### Emergency Rollback

**When to use**: Critical bug in production, immediate rollback required

```bash
#!/bin/bash
# emergency-rollback.sh

echo "🚨 EMERGENCY ROLLBACK INITIATED"

# 1. Stop current service
sudo systemctl stop knhk
echo "✅ Current service stopped"

# 2. Restore previous binary
sudo cp /usr/local/bin/knhk.backup /usr/local/bin/knhk
sudo cp /usr/local/lib/libknhk.so.backup /usr/local/lib/libknhk.so
sudo ldconfig
echo "✅ Previous binaries restored"

# 3. Restore previous config
sudo cp -r /etc/knhk.backup/* /etc/knhk/
echo "✅ Previous config restored"

# 4. Start service
sudo systemctl start knhk
echo "✅ Service restarted"

# 5. Verify health
sleep 5
curl http://localhost:8080/health
echo "✅ Health check complete"

# 6. Run Weaver validation
weaver registry live-check --registry /etc/knhk/registry/
echo "✅ Weaver validation complete"

echo "🎉 ROLLBACK COMPLETE"
```

### Circuit Breaker Activation

**When to use**: External dependency failing, need to prevent cascading failures

```bash
# Activate circuit breaker manually
curl -X POST http://localhost:8080/api/v1/circuit-breaker/activate \
  -H "Content-Type: application/json" \
  -d '{
    "service": "external-api",
    "reason": "High error rate detected",
    "duration_seconds": 300
  }'

# Verify activation
curl http://localhost:8080/api/v1/circuit-breaker/status
```

### Force Shutdown

**When to use**: Service not responding to normal shutdown

```bash
# 1. Try graceful shutdown first
sudo systemctl stop knhk
sleep 10

# 2. If still running, force kill
sudo pkill -9 knhk

# 3. Clean up stale state
sudo rm -f /var/run/knhk.pid
sudo rm -f /var/lock/knhk.lock

# 4. Restart
sudo systemctl start knhk
```

---

## Getting Help

### Collect Diagnostic Information

```bash
#!/bin/bash
# collect-diagnostics.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DIAG_DIR="/tmp/knhk-diagnostics-$TIMESTAMP"
mkdir -p "$DIAG_DIR"

echo "Collecting KNHK diagnostics to $DIAG_DIR..."

# System info
uname -a > "$DIAG_DIR/system-info.txt"
lscpu > "$DIAG_DIR/cpu-info.txt"
free -h > "$DIAG_DIR/memory-info.txt"

# Service status
systemctl status knhk > "$DIAG_DIR/service-status.txt"
journalctl -u knhk -n 1000 > "$DIAG_DIR/service-logs.txt"

# Weaver validation
weaver registry check -r registry/ > "$DIAG_DIR/weaver-schema-check.txt" 2>&1
weaver registry live-check --registry registry/ > "$DIAG_DIR/weaver-live-check.txt" 2>&1

# Metrics snapshot
curl http://localhost:8889/metrics > "$DIAG_DIR/metrics-snapshot.txt"

# Telemetry sample
docker logs otel-collector > "$DIAG_DIR/otel-collector-logs.txt" 2>&1

# Configuration
cp -r /etc/knhk "$DIAG_DIR/config"
cp -r registry "$DIAG_DIR/registry"

# Create tarball
tar czf "$DIAG_DIR.tar.gz" "$DIAG_DIR"
rm -rf "$DIAG_DIR"

echo "✅ Diagnostics collected: $DIAG_DIR.tar.gz"
echo "   Share this file when reporting issues"
```

### Contact Support

When reporting issues, include:

1. **Diagnostic tarball** (from script above)
2. **Error description**: What were you trying to do?
3. **Expected vs actual**: What should have happened vs what happened?
4. **Weaver validation results**: Output of `weaver registry live-check`
5. **Reproduction steps**: How can we reproduce the issue?

---

**Last Updated**: 2025-11-16
**Status**: Production Troubleshooting Guide
