# Error Codes Reference

**Complete Guide to KNHK Workflow Engine Error Messages and Solutions**

- **Error Format**: Structured JSON with code, message, and details
- **Trace IDs**: All errors include OpenTelemetry trace ID for debugging
- **Categories**: Validation, Execution, State, Resource, Configuration
- **Last Updated**: 2025-11-17

---

## Table of Contents

1. [Validation Errors (4xx)](#validation-errors)
2. [Execution Errors (5xx)](#execution-errors)
3. [State Errors (4xx-5xx)](#state-errors)
4. [Resource Errors (4xx-5xx)](#resource-errors)
5. [Configuration Errors (4xx-5xx)](#configuration-errors)
6. [Troubleshooting Guide](#troubleshooting-guide)

---

## Validation Errors

### INVALID_WORKFLOW_SPEC (400)

**When**: Turtle/RDF workflow specification has syntax errors

**Example Error**:
```json
{
  "error": "BadRequest",
  "code": "INVALID_WORKFLOW_SPEC",
  "message": "Turtle syntax error at line 42: expected ';' or '.'",
  "details": {
    "line": 42,
    "column": 25,
    "token": "invalid_char",
    "context": "workflow:process @prefix"
  },
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736"
}
```

**Solutions**:

1. **Check Turtle Syntax**
   ```bash
   # Validate your Turtle file
   rapper -i turtle -o turtle workflow.ttl
   ```

2. **Common Issues**
   - Missing semicolons (`;`) in property lists
   - Malformed IRIs (missing angle brackets `<>`)
   - Undefined prefixes
   - Invalid YAWL pattern references

3. **Debug Steps**
   - Use online Turtle validator: https://www.w3.org/Team/edit-cvs/2006/04/turtle-play/
   - Check line number in error message
   - Compare with YAWL examples in documentation
   - Look for non-ASCII characters

---

### DEADLOCK_DETECTED (400)

**When**: Workflow validation detects potential deadlock

**Example Error**:
```json
{
  "error": "BadRequest",
  "code": "DEADLOCK_DETECTED",
  "message": "Sync join without matching split detected",
  "details": {
    "issue": "join-task has no corresponding split",
    "location": "task:approve-order",
    "line": 28,
    "suggestion": "Check split/join nesting levels"
  }
}
```

**Common Deadlock Patterns**:

1. **Mismatched Split-Join**
   ```turtle
   # WRONG: Join at different level than split
   :process1 yawl:hasPattern [
     yawl:split :split1 ;
     yawl:child :task1, :task2
   ] .

   :task1 yawl:next [
     yawl:join :join1  # Wrong! Join level doesn't match split
   ] .
   ```

2. **Solution**: Ensure split and join levels match
   ```turtle
   # CORRECT: Join at same level as split
   :process1 yawl:hasPattern [
     yawl:split :split1 ;
     yawl:child :parallel1 [ yawl:join :join1 ]
   ] .
   ```

**Solutions**:

1. **Run Deadlock Detection First**
   ```bash
   curl -X POST http://localhost:8080/api/v1/workflows/validate \
     -H "Content-Type: application/json" \
     -d '{"spec": "...", "check_deadlock": true}'
   ```

2. **Review Pattern Nesting**
   - Verify each split has matching join
   - Check nesting levels are balanced
   - Use `yawl:child` for nested patterns

3. **Use Process Mining**
   - Export workflow as Petri Net
   - Analyze with process mining tools
   - Verify reachability of all tasks

---

### GUARD_VIOLATION (400)

**When**: Guard condition prevents task enabling

**Example Error**:
```json
{
  "error": "BadRequest",
  "code": "GUARD_VIOLATION",
  "message": "Cannot enable task: guard condition not satisfied",
  "details": {
    "task": "approve-order",
    "guard": "amount < 10000",
    "case_data": {"amount": 50000},
    "reason": "Amount exceeds guard threshold"
  }
}
```

**Solutions**:

1. **Check Guard Conditions**
   ```bash
   # Get task details
   curl http://localhost:8080/api/v1/cases/case_001/tasks/approve-order
   ```

2. **Understand Guard Logic**
   - Guards prevent invalid task execution
   - Check case data against guard predicate
   - May need to update case data first

3. **Debug Steps**
   - Print guard condition from workflow definition
   - Log case data at task enabling point
   - Compare values manually

4. **Fix Options**:
   - Update case data to satisfy guard
   - Modify workflow guard (re-register)
   - Use different workflow version if available

---

### UNSOUND_WORKFLOW (400)

**When**: Workflow doesn't satisfy soundness criteria

**Example Error**:
```json
{
  "error": "BadRequest",
  "code": "UNSOUND_WORKFLOW",
  "message": "Workflow is not sound: unreachable tasks detected",
  "details": {
    "unreachable_tasks": ["send-confirmation"],
    "fitness_score": 0.75,
    "issues": [
      "Task 'send-confirmation' is unreachable from entry tasks"
    ]
  }
}
```

**Solutions**:

1. **Find Unreachable Tasks**
   - Review error details for task list
   - Trace from entry tasks forward
   - Check for missing edges

2. **Common Causes**:
   - Tasks without incoming edges
   - Split without corresponding path
   - Wrong join configuration

3. **Fix**:
   ```turtle
   # Add missing edge
   :task1 yawl:next :send-confirmation .
   :send-confirmation yawl:next :exit_task .
   ```

---

## Execution Errors

### TASK_EXECUTION_FAILED (500)

**When**: Task execution throws exception

**Example Error**:
```json
{
  "error": "InternalServerError",
  "code": "TASK_EXECUTION_FAILED",
  "message": "Task execution failed: timeout after 30s",
  "details": {
    "task": "process-order",
    "case": "case_2025_001",
    "duration_ms": 30000,
    "cause": "External API timeout"
  },
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736"
}
```

**Solutions**:

1. **Increase Timeout**
   ```toml
   # config.toml
   [execution]
   task_timeout = 600  # Increase to 10 minutes
   ```

2. **Check External Dependencies**
   ```bash
   # Test external API
   curl -v https://api.external-service.com/endpoint

   # Check latency
   ping api.external-service.com
   ```

3. **Review Execution Trace**
   ```bash
   curl http://localhost:8080/api/v1/cases/case_2025_001/history
   ```

4. **Implement Retry Logic**
   ```toml
   [hooks]
   retry_enabled = true
   retry_max_attempts = 3
   retry_backoff_ms = 100
   ```

---

### CASE_CREATION_FAILED (500)

**When**: Cannot create new case instance

**Example Error**:
```json
{
  "error": "InternalServerError",
  "code": "CASE_CREATION_FAILED",
  "message": "Failed to create case instance",
  "details": {
    "workflow": "wf_001",
    "reason": "Data validation failed",
    "validation_errors": [
      "Field 'order_id' is required"
    ]
  }
}
```

**Solutions**:

1. **Check Required Fields**
   - Review workflow schema
   - Ensure all required data provided

2. **Validate Data**
   ```bash
   curl -X POST http://localhost:8080/api/v1/workflows/validate \
     -d '{"spec": "...", "check_soundness": true}'
   ```

3. **Check Case Limits**
   ```toml
   [execution]
   max_concurrent_cases = 5000  # May need increase
   ```

---

## State Errors

### WORKFLOW_NOT_FOUND (404)

**When**: Referenced workflow doesn't exist

**Example Error**:
```json
{
  "error": "NotFound",
  "code": "WORKFLOW_NOT_FOUND",
  "message": "Workflow not found: wf_999",
  "details": {
    "workflow_id": "wf_999",
    "available_workflows": ["wf_001", "wf_002", "wf_003"]
  }
}
```

**Solutions**:

1. **List Available Workflows**
   ```bash
   curl http://localhost:8080/api/v1/workflows
   ```

2. **Register Workflow**
   ```bash
   curl -X POST http://localhost:8080/api/v1/workflows \
     -H "Content-Type: application/json" \
     -d '{"spec": "...", "name": "my-workflow"}'
   ```

3. **Check Workflow Status**
   - Verify not deleted
   - Check if archived/deprecated

---

### CASE_NOT_FOUND (404)

**When**: Referenced case doesn't exist

**Example Error**:
```json
{
  "error": "NotFound",
  "code": "CASE_NOT_FOUND",
  "message": "Case instance not found: case_999"
}
```

**Solutions**:

1. **Check Case ID**
   ```bash
   # List cases for workflow
   curl "http://localhost:8080/api/v1/cases?workflow_id=wf_001"
   ```

2. **Verify Case Created**
   - May be recently deleted
   - Check case history

---

### CASE_IN_WRONG_STATE (409)

**When**: Cannot perform action in current case state

**Example Error**:
```json
{
  "error": "Conflict",
  "code": "CASE_IN_WRONG_STATE",
  "message": "Cannot enable task: case is already completed",
  "details": {
    "case": "case_2025_001",
    "current_state": "completed",
    "requested_action": "enable_task"
  }
}
```

**Solutions**:

1. **Check Case State**
   ```bash
   curl http://localhost:8080/api/v1/cases/case_2025_001
   ```

2. **Allowed Actions by State**:
   - `active`: enable_task, complete_task, suspend, cancel
   - `suspended`: resume, cancel
   - `completed`: view_history
   - `failed`: retry (if configured), cancel
   - `cancelled`: view_history

3. **Resume Suspended Case**
   ```bash
   curl -X POST http://localhost:8080/api/v1/cases/case_2025_001/resume
   ```

---

### TASK_ALREADY_COMPLETED (409)

**When**: Trying to complete already-completed task

**Example Error**:
```json
{
  "error": "Conflict",
  "code": "TASK_ALREADY_COMPLETED",
  "message": "Task already completed",
  "details": {
    "task": "approve-order",
    "completed_at": "2025-11-17T10:30:00Z",
    "case": "case_2025_001"
  }
}
```

**Solutions**:

1. **Check Task State**
   ```bash
   curl http://localhost:8080/api/v1/cases/case_2025_001
   ```

2. **View History** to see completion details
   ```bash
   curl http://localhost:8080/api/v1/cases/case_2025_001/history
   ```

---

## Resource Errors

### RESOURCE_EXHAUSTED (429)

**When**: Server limits exceeded

**Example Error**:
```json
{
  "error": "ResourceExhausted",
  "code": "RESOURCE_EXHAUSTED",
  "message": "Too many concurrent cases",
  "details": {
    "current": 5000,
    "limit": 5000,
    "suggestion": "Wait for cases to complete or increase limit"
  }
}
```

**Solutions**:

1. **Increase Limits**
   ```toml
   [execution]
   max_concurrent_cases = 10000
   queue_depth = 5000
   ```

2. **Monitor Case Completion**
   ```bash
   curl http://localhost:8080/api/v1/cases?state=completed
   ```

3. **Scale Horizontally**
   - Add more worker threads
   - Deploy multiple instances

---

### RATE_LIMIT_EXCEEDED (429)

**When**: API rate limit exceeded

**Example Error**:
```json
{
  "error": "ResourceExhausted",
  "code": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded: 1000 requests/minute",
  "headers": {
    "X-RateLimit-Limit": "1000",
    "X-RateLimit-Remaining": "0",
    "X-RateLimit-Reset": "1700137800"
  }
}
```

**Solutions**:

1. **Implement Backoff**
   ```bash
   # Exponential backoff: wait 2s, 4s, 8s
   curl --retry 3 --retry-delay 2 http://localhost:8080/api/v1/workflows
   ```

2. **Increase Rate Limit** (admin only)
   ```toml
   [api]
   rate_limit_per_minute = 2000
   ```

---

## Configuration Errors

### INVALID_CONFIGURATION (400)

**When**: Configuration file has errors

**Example Error**:
```
Error: Invalid configuration
  at config.toml:42

Expected integer for 'worker_threads', got string 'eight'
```

**Solutions**:

1. **Check Syntax**
   - TOML files are strict about types
   - Use proper types: `port = 8080` not `port = "8080"`

2. **Validate Configuration**
   ```bash
   knhk --validate-config config.toml
   ```

3. **Example Valid Config**
   ```toml
   [server]
   port = 8080                    # integer, not string
   tls_enabled = false            # boolean
   request_timeout = 30           # integer
   bind_address = "0.0.0.0"       # string
   ```

---

## Troubleshooting Guide

### Workflow Won't Register

**Checklist**:

1. ✅ Syntax: Valid Turtle/RDF?
   ```bash
   rapper -i turtle -o turtle workflow.ttl
   ```

2. ✅ Deadlock: No potential deadlocks?
   ```bash
   curl -X POST http://localhost:8080/api/v1/workflows/validate \
     -d '{"check_deadlock": true}'
   ```

3. ✅ Soundness: Is workflow sound?
   ```bash
   curl -X POST http://localhost:8080/api/v1/workflows/validate \
     -d '{"check_soundness": true}'
   ```

4. ✅ Patterns: All patterns supported?
   - Review YAWL pattern requirements
   - Check for custom patterns not in standard set

---

### Case Won't Start

**Checklist**:

1. ✅ Workflow registered: Does `wf_id` exist?
   ```bash
   curl http://localhost:8080/api/v1/workflows/{wf_id}
   ```

2. ✅ Valid data: Does data match schema?
   - Check required fields
   - Validate data types

3. ✅ Resource limits: Are limits exceeded?
   ```bash
   # Check current load
   curl http://localhost:8080/api/v1/workflows/wf_id/metrics
   ```

4. ✅ Engine running: Is engine responsive?
   ```bash
   curl http://localhost:8080/health
   ```

---

### Task Won't Complete

**Checklist**:

1. ✅ Task enabled: Is task in enabled list?
   ```bash
   curl http://localhost:8080/api/v1/cases/{case_id}
   ```

2. ✅ Guard met: Does guard condition pass?
   - Check case data
   - Compare with guard predicate

3. ✅ No timeout: Is task still executing?
   - Check execution history
   - Monitor logs

4. ✅ Try Resume: If suspended
   ```bash
   curl -X POST http://localhost:8080/api/v1/cases/{case_id}/resume
   ```

---

### Getting Help

1. **Check Trace ID**
   - All errors include `trace_id`
   - Use to find detailed logs

2. **Review Logs**
   ```bash
   grep "4bf92f3577b34da6a3ce929d0e0e4736" logs/knhk.log
   ```

3. **Enable Debug Logging**
   ```toml
   [observability.logging]
   level = "debug"
   format = "json"
   ```

4. **Related Documentation**
   - [Configuration Guide](./configuration.md)
   - [API Endpoints](./api-endpoints.md)
   - [How-To: Troubleshooting](../how-to/troubleshooting.md)
   - [How-To: Deadlock Debugging](../how-to/deadlock-debugging.md)
