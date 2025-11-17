# How-To: Troubleshoot KNHK Workflow Engine

**Comprehensive Guide to Diagnosing and Fixing Common Issues**

- **Time to Complete**: Varies by issue
- **Difficulty Level**: Beginner to Intermediate
- **You'll Learn**: Debug workflows, identify bottlenecks, solve common problems

---

## Table of Contents

1. [Quick Diagnosis Checklist](#quick-diagnosis-checklist)
2. [Workflow Registration Issues](#workflow-registration-issues)
3. [Case Execution Issues](#case-execution-issues)
4. [Performance Issues](#performance-issues)
5. [Deadlock Problems](#deadlock-problems)
6. [Data & State Issues](#data--state-issues)
7. [Infrastructure Issues](#infrastructure-issues)
8. [Getting Help](#getting-help)

---

## Quick Diagnosis Checklist

### Step 1: Engine Health

```bash
# Is the engine running?
curl http://localhost:8080/health

# Expected response:
# {"status":"healthy","timestamp":"..."}

# If not responding:
# - Check if process is running: ps aux | grep knhk
# - Check logs: tail -f logs/knhk.log
# - Verify port is correct: netstat -tlnp | grep 8080
# - Check config for errors: knhk --validate-config config.toml
```

### Step 2: Can You Connect?

```bash
# Test connectivity
curl -v http://localhost:8080/health

# Look for:
# - Connection refused → engine not running
# - Connection timeout → firewall or wrong host
# - 404 Not Found → wrong endpoint
# - 500 error → engine problem
```

### Step 3: Check Logs

```bash
# Real-time logs
tail -f logs/knhk.log

# Search for errors
grep ERROR logs/knhk.log | tail -20

# Filter by component
grep "execution" logs/knhk.log

# Filter by trace ID
grep "4bf92f3577b34da6a3ce929d0e0e4736" logs/knhk.log
```

---

## Workflow Registration Issues

### Problem: Workflow Won't Register

**Symptoms**:
- API returns 400 BadRequest
- Error: "INVALID_WORKFLOW_SPEC"
- Error: "DEADLOCK_DETECTED"

**Diagnosis**:

```bash
# Step 1: Validate syntax
rapper -i turtle -o turtle workflow.ttl

# Step 2: Check with API
curl -X POST http://localhost:8080/api/v1/workflows/validate \
  -H "Content-Type: application/json" \
  -d '{
    "spec": "...your turtle here...",
    "check_deadlock": true,
    "check_soundness": true
  }' | jq .

# Step 3: Review error details
# - Check line number in error
# - Look at context around error
# - Compare with working examples
```

**Solutions**:

| Error | Cause | Fix |
|-------|-------|-----|
| Syntax error | Malformed Turtle | Use Turtle validator online |
| Undefined prefix | Missing `@prefix` | Add prefix definition |
| Deadlock detected | Bad split/join | Check nesting levels |
| Unsound | Unreachable tasks | Add missing edges |
| Unknown pattern | Unsupported YAWL pattern | Use supported patterns |

**Common Turtle Mistakes**:

```turtle
# ❌ WRONG: Missing semicolons
:task1 workflow:name "Task" workflow:next :task2 .

# ✅ CORRECT: Proper syntax
:task1
  workflow:name "Task" ;
  workflow:next :task2 .

# ❌ WRONG: Undefined prefix
:task1 yawl:pattern workflow:Sequence .

# ✅ CORRECT: Define prefix first
@prefix yawl: <http://example.org/yawl/> .
:task1 yawl:pattern workflow:Sequence .

# ❌ WRONG: IRIs without angle brackets
workflow:task1 workflow:next workflow:task2 .

# ✅ CORRECT: Use angle brackets for full IRIs
<http://example.org/workflow/task1> workflow:next <http://example.org/workflow/task2> .
```

---

### Problem: Deadlock Detected

**What is Deadlock?**

Deadlock occurs when workflow gets stuck with no available tasks to execute.

**Common Patterns**:

1. **Mismatched Split-Join**
```turtle
# ❌ WRONG: Join not matching split
:process1 [
  yawl:split :split1 ;
  yawl:child :task1, :task2
] .

# Join at wrong level
:task1 yawl:next [
  yawl:join :join1  # Problem!
] .
```

2. **Circular Dependency**
```turtle
# ❌ WRONG: Tasks waiting for each other
:task1 workflow:next :task2 .
:task2 workflow:next :task1 .  # Circular!
```

3. **Missing Synchronization**
```turtle
# ❌ WRONG: Two splits without join
:split1 [
  yawl:child :task1 ;
  yawl:child :split2 [
    yawl:child :task3 ;
    yawl:child :task4
  ]
] .
# Need join for split2 before task2
```

**Diagnosis**:

```bash
# Check for deadlock
curl -X POST http://localhost:8080/api/v1/workflows/wf_id/check-deadlock | jq .

# Detailed validation
curl -X POST http://localhost:8080/api/v1/workflows/validate \
  -d '{
    "spec": "...",
    "check_deadlock": true
  }' | jq '.details'

# Use process mining
# Export workflow as Petri net
# Analyze with Tools4BPMA or ProM
```

**Solutions**:

1. **Fix Split-Join Nesting**
```turtle
# ✅ CORRECT: Balanced split/join
:process [
  yawl:split :split1 ;
  yawl:splitType "AND" ;
  yawl:child :parallel_branch [
    yawl:child :task1 ;
    yawl:join :join1
  ] ;
  yawl:child :parallel_branch [
    yawl:child :task2 ;
    yawl:join :join1
  ]
] .
```

2. **Remove Circular References**
```turtle
# ✅ CORRECT: Linear flow
:task1 workflow:next :task2 .
:task2 workflow:next :task3 .
:task3 workflow:next :exit .
```

---

## Case Execution Issues

### Problem: Case Won't Start

**Symptoms**:
- Cannot create case instance
- Error: 400 BadRequest
- Error: 404 Workflow not found

**Diagnosis**:

```bash
# Step 1: Verify workflow exists
curl http://localhost:8080/api/v1/workflows/wf_id

# Step 2: Check case creation request
curl -X POST http://localhost:8080/api/v1/workflows/wf_id/cases \
  -H "Content-Type: application/json" \
  -d '{
    "case_id": "test_case",
    "data": {"field": "value"}
  }' | jq '.error'

# Step 3: Check system resources
curl http://localhost:8080/api/v1/workflows/wf_id/metrics
```

**Common Causes**:

| Issue | Fix |
|-------|-----|
| Workflow not found | Register workflow first |
| Invalid data format | Check schema requirements |
| Max cases reached | Increase `max_concurrent_cases` |
| Database down | Check database connection |
| Missing required fields | Provide all required data |

**Solution Examples**:

```bash
# Case 1: Workflow not found
curl http://localhost:8080/api/v1/workflows  # List all
curl -X POST http://localhost:8080/api/v1/workflows \
  -d '{"spec": "..."}'  # Register new

# Case 2: Invalid data
# Check workflow requirements, then provide:
curl -X POST http://localhost:8080/api/v1/workflows/wf_id/cases \
  -d '{
    "case_id": "case_001",
    "data": {
      "required_field_1": "value",
      "required_field_2": 123
    }
  }'

# Case 3: Resource limits
# Edit config.toml:
# [execution]
# max_concurrent_cases = 10000  # Increase from 5000
```

---

### Problem: Task Won't Complete

**Symptoms**:
- Guard violation error
- Task status not changing
- Case stuck in intermediate state

**Diagnosis**:

```bash
# Check case status
curl http://localhost:8080/api/v1/cases/case_id | jq .

# Check enabled tasks
curl http://localhost:8080/api/v1/cases/case_id | jq '.enabled_tasks'

# Check case data
curl http://localhost:8080/api/v1/cases/case_id | jq '.data'

# Check execution history
curl http://localhost:8080/api/v1/cases/case_id/history | jq .
```

**Common Causes**:

1. **Guard Condition Not Met**
```bash
# Example: Guard requires amount < 1000
# But case data has amount = 5000

# Solution: Provide compliant data
curl -X POST http://localhost:8080/api/v1/cases/case_id/tasks/task_id/complete \
  -d '{
    "output_data": {
      "amount": 500  # Now satisfies guard
    }
  }'
```

2. **Task Not Enabled**
```bash
# May need to complete prerequisites first
curl http://localhost:8080/api/v1/cases/case_id
# Check which tasks are in enabled_tasks

# Enable task manually if allowed
curl -X POST http://localhost:8080/api/v1/cases/case_id/tasks/task_id/enable
```

3. **Timeout Occurred**
```bash
# Check if task timed out
curl http://localhost:8080/api/v1/cases/case_id/history | grep timeout

# Increase timeout in config
# [execution]
# task_timeout = 600  # Increase to 10 minutes
```

---

### Problem: Case Gets Stuck

**Symptoms**:
- Case state is "active" but hasn't progressed
- No tasks are enabled
- No errors in logs

**Diagnosis**:

```bash
# Check case state
curl http://localhost:8080/api/v1/cases/case_id | jq '.'

# Review complete history
curl http://localhost:8080/api/v1/cases/case_id/history | jq '.'

# Look for OTEL traces
# Search logs for case_id

# Check if deadlock during execution
# (different from design-time deadlock)
```

**Possible Causes**:

1. **Workflow Has Design-Time Deadlock**
```bash
# Solution: Fix workflow definition
# Redeploy fixed version
# (May need to cancel stuck cases)
```

2. **Guard Condition Prevents All Tasks**
```bash
# No task can proceed due to guards
# Solution: Fix case data to satisfy a guard
```

3. **Missing External Integration**
```bash
# Task waiting for external system
# Check if external system is responding
# May need timeout adjustment
```

---

## Performance Issues

### Problem: High Latency

**Symptoms**:
- Tasks taking too long to complete
- P99 latency > 1 second
- Throughput lower than expected

**Diagnosis**:

```bash
# Check performance metrics
curl http://localhost:8080/api/v1/workflows/wf_id/metrics | jq '.metrics'

# Monitor in real-time
watch -n 1 'curl -s http://localhost:8080/api/v1/workflows/wf_id/metrics | jq ".metrics"'

# Check resource utilization
top
# Look for knhk-workflow process CPU/memory

# Check database latency
time curl http://localhost:8080/api/v1/cases
```

**Common Causes & Solutions**:

| Cause | Symptom | Solution |
|-------|---------|----------|
| Insufficient CPU | High CPU % | Increase `worker_threads` |
| Memory pressure | Growing memory use | Reduce `cache_size`, increase RAM |
| Slow disk | High I/O wait | Use SSD, reduce `log_level` |
| Database bottleneck | DB queries slow | Add DB connection pool |
| Network latency | External calls slow | Check external service |
| Logging overhead | Lower throughput | Set `log_level = "warn"` |

**Tuning for Performance**:

```toml
# config.toml - High performance settings

[execution]
worker_threads = 16  # Match CPU cores
queue_depth = 5000
execution_model = "eager"

[storage]
compression = false  # Trade space for speed
backend = "postgres"

[storage.postgres]
pool_size = 50  # More connections

[performance.cache]
spec_cache_enabled = true
spec_cache_size = 10000
pattern_cache_enabled = true
pattern_cache_size = 50000

[performance.concurrency]
work_stealing = true
lock_free = true

[observability.logging]
level = "warn"  # Reduce logging overhead
perf_logging_enabled = false
```

Restart after changes:

```bash
# Edit config
nano config.toml

# Restart engine
# Kill old process and restart
```

---

### Problem: Out of Memory

**Symptoms**:
- Process killed by OOMKiller
- Memory usage grows over time
- Slow response times before crash

**Diagnosis**:

```bash
# Monitor memory
watch -n 1 'ps aux | grep knhk'

# Check cache sizes
curl http://localhost:8080/api/v1/metrics | grep cache

# Check for memory leaks
# Monitor over 1 hour, look for continuous growth
```

**Solutions**:

1. **Reduce Cache Sizes**
```toml
[performance.cache]
spec_cache_size = 1000  # Was 10000
pattern_cache_size = 5000  # Was 50000
```

2. **Reduce Queue Depth**
```toml
[execution]
queue_depth = 1000  # Was 5000
max_concurrent_cases = 2000  # Was 5000
```

3. **Enable Compression**
```toml
[storage]
compression = true
```

4. **Increase Available Memory**
```bash
# In Kubernetes
# Edit deployment.yaml:
# resources:
#   limits:
#     memory: 4Gi  # Increase from 2Gi
```

---

## Deadlock Problems

### Runtime Deadlock Detection

Even well-designed workflows can deadlock at runtime due to:
- Race conditions
- External system failures
- Unexpected data values

**Detection**:

```bash
# Monitor case progress
while true; do
  curl http://localhost:8080/api/v1/cases/case_id | jq '.progress'
  sleep 10
done

# If progress stalls for > 5 minutes:
# 1. Check execution history
# 2. Look for stuck task
# 3. Review case data
# 4. Check external dependencies
```

**Recovery**:

```bash
# Option 1: Cancel stuck case
curl -X POST http://localhost:8080/api/v1/cases/case_id/cancel \
  -d '{"reason": "Deadlock detected"}'

# Option 2: Manual task enablement
curl -X POST http://localhost:8080/api/v1/cases/case_id/tasks/task_id/enable

# Option 3: Update case data to unblock
curl -X POST http://localhost:8080/api/v1/cases/case_id/tasks/task_id/complete \
  -d '{"output_data": {...}}'
```

---

## Data & State Issues

### Problem: Data Corruption

**Symptoms**:
- Case data doesn't match what was sent
- Guards check fails unexpectedly
- Numbers or strings changed

**Prevention**:

```bash
# Always validate before completion
curl http://localhost:8080/api/v1/cases/case_id | jq '.data'

# Verify before each step
echo "Checking case data..."
curl http://localhost:8080/api/v1/cases/case_id | jq '.data' > before.json

# Complete task
curl -X POST http://localhost:8080/api/v1/cases/case_id/tasks/task_id/complete \
  -d '{"output_data": {"field": "value"}}'

# Verify after
curl http://localhost:8080/api/v1/cases/case_id | jq '.data' > after.json
```

**Recovery**:

```bash
# Export case history
curl http://localhost:8080/api/v1/cases/case_id/history > history.json

# Review all data changes
grep "data" history.json

# If data is corrupted:
# 1. Cancel current case
# 2. Create new case with correct data
# 3. Investigate root cause
```

---

## Infrastructure Issues

### Problem: Database Connection Failed

**Symptoms**:
- Error: "Cannot connect to database"
- Workflows can't be registered
- Cases can't be created

**Diagnosis**:

```bash
# Test database connection
curl http://localhost:8080/health | jq '.database'

# Check configuration
grep "connection_string" config.toml

# Test manually
psql postgresql://user:pass@localhost:5432/knhk -c "SELECT 1;"

# Check database server
# Is it running? Is port open? Is firewall allowing?
```

**Solutions**:

1. **Verify Credentials**
```bash
# Test connection string
postgresql://user:password@host:5432/database

# Check:
# - Correct username/password
# - Correct host/port
# - User has access to database
# - Database exists
```

2. **Increase Connection Pool**
```toml
[storage.postgres]
pool_size = 50  # Increase from 20
connection_timeout = 10  # Increase from 5
```

3. **Restart Service**
```bash
# Stop and restart engine
# Stop database if needed and restart
```

---

### Problem: Out of Disk Space

**Symptoms**:
- Cannot create cases
- Storage errors in logs
- Disk full message

**Solutions**:

```bash
# Check disk usage
du -sh /var/lib/knhk
df -h /var/lib/knhk

# Cleanup old cases
# Export to archive first
curl http://localhost:8080/api/v1/cases?state=completed \
  > completed_cases.json

# Then delete in bulk (if supported)
# Or delete via retention policy in config
```

---

## Getting Help

### Collect Diagnostic Information

```bash
#!/bin/bash

echo "=== KNHK Diagnostic Report ===" > diagnostic.txt
date >> diagnostic.txt

echo -e "\n=== System Info ===" >> diagnostic.txt
uname -a >> diagnostic.txt
docker version >> diagnostic.txt 2>&1
kubernetes version >> diagnostic.txt 2>&1

echo -e "\n=== Engine Status ===" >> diagnostic.txt
curl -s http://localhost:8080/health | jq . >> diagnostic.txt

echo -e "\n=== Configuration ===" >> diagnostic.txt
cat config.toml >> diagnostic.txt

echo -e "\n=== Recent Logs ===" >> diagnostic.txt
tail -100 logs/knhk.log >> diagnostic.txt

echo -e "\n=== Database Status ===" >> diagnostic.txt
curl -s http://localhost:8080/api/v1/workflows | jq '.count' >> diagnostic.txt

echo -e "\n=== Resource Usage ===" >> diagnostic.txt
ps aux | grep knhk >> diagnostic.txt
top -b -n 1 | head -20 >> diagnostic.txt

echo "Diagnostic report saved to diagnostic.txt"
```

Run and share with support:

```bash
chmod +x diagnostic.sh
./diagnostic.sh

# Include in bug report
cat diagnostic.txt
```

### Getting Support

When reporting issues, include:
1. Error message (exact text)
2. Trace ID from error
3. Steps to reproduce
4. Configuration (sanitized)
5. Logs from time of error
6. Diagnostic report from above

---

## Related Documentation

- [Error Codes Reference](../reference/error-codes.md) - Detailed error information
- [Configuration Guide](../reference/configuration.md) - Tuning options
- [How-To: Deadlock Debugging](./deadlock-debugging.md) - Advanced deadlock help
- [How-To: OTEL Observability](./otel-observability.md) - Setup detailed monitoring
