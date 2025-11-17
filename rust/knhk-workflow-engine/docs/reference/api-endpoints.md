# REST API Endpoints Reference

**Complete Guide to KNHK Workflow Engine REST API**

- **Base URL**: `http://localhost:8080/api/v1`
- **Authentication**: Bearer token (optional, configurable)
- **Content-Type**: `application/json`
- **Last Updated**: 2025-11-17

---

## Table of Contents

1. [Workflow Management](#workflow-management)
2. [Case/Instance Management](#case-instance-management)
3. [Execution & Control](#execution--control)
4. [Monitoring & Observability](#monitoring--observability)
5. [Validation & Analysis](#validation--analysis)
6. [Status Codes & Errors](#status-codes--errors)

---

## Workflow Management

### Register Workflow Specification

**Endpoint**: `POST /workflows`

**Purpose**: Register a new workflow specification from Turtle/RDF definition

**Request**:
```json
{
  "name": "order-processing",
  "description": "Main order processing workflow",
  "spec": "workflow:Process a @prefix...",
  "turtle_format": "turtle",
  "tags": ["production", "order"]
}
```

**Response** (201 Created):
```json
{
  "id": "wf_7kJ9nM2xQ8pR",
  "name": "order-processing",
  "version": "1.0.0",
  "created_at": "2025-11-17T10:30:00Z",
  "patterns": 12,
  "entry_tasks": ["receive-order"]
}
```

**Error Cases**:
- `400 BadRequest`: Invalid Turtle/RDF syntax
- `400 BadRequest`: Missing required fields
- `409 Conflict`: Workflow already registered
- `413 PayloadTooLarge`: Specification exceeds 10MB

**Example**:
```bash
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d @workflow.json
```

---

### Get Workflow Specification

**Endpoint**: `GET /workflows/{id}`

**Purpose**: Retrieve a registered workflow specification

**URL Parameters**:
- `id` (required): Workflow specification ID

**Query Parameters**:
- `include_schema` (optional): Include RDF schema (true/false)
- `include_patterns` (optional): Include pattern details (true/false)

**Response** (200 OK):
```json
{
  "id": "wf_7kJ9nM2xQ8pR",
  "name": "order-processing",
  "version": "1.0.0",
  "spec": "workflow:Process...",
  "patterns": {
    "sequence": 8,
    "parallel": 3,
    "split": 2,
    "join": 1
  },
  "entry_tasks": ["receive-order"],
  "exit_tasks": ["send-confirmation"],
  "created_at": "2025-11-17T10:30:00Z",
  "modified_at": "2025-11-17T10:30:00Z"
}
```

**Error Cases**:
- `404 NotFound`: Workflow not registered
- `410 Gone`: Workflow has been deleted

---

### List Workflows

**Endpoint**: `GET /workflows`

**Purpose**: List all registered workflows with filtering

**Query Parameters**:
- `limit` (optional): Max results (default: 20, max: 100)
- `offset` (optional): Pagination offset (default: 0)
- `tag` (optional): Filter by tag (repeatable)
- `status` (optional): active/archived/deprecated
- `search` (optional): Text search in name/description

**Response** (200 OK):
```json
{
  "total": 42,
  "count": 20,
  "offset": 0,
  "workflows": [
    {
      "id": "wf_7kJ9nM2xQ8pR",
      "name": "order-processing",
      "version": "1.0.0",
      "patterns": 12,
      "active_cases": 3847,
      "created_at": "2025-11-17T10:30:00Z"
    }
  ],
  "_links": {
    "next": "/api/v1/workflows?offset=20&limit=20"
  }
}
```

---

### Delete/Unregister Workflow

**Endpoint**: `DELETE /workflows/{id}`

**Purpose**: Unregister a workflow (only if no active cases)

**Request Parameters**:
- `id` (required): Workflow specification ID
- `force` (optional, query): Force delete even with active cases (requires admin)

**Response** (204 NoContent)

**Error Cases**:
- `404 NotFound`: Workflow not found
- `409 Conflict`: Cannot delete - has 2,847 active cases
- `403 Forbidden`: Insufficient permissions for force delete

---

## Case/Instance Management

### Create Case Instance

**Endpoint**: `POST /workflows/{id}/cases`

**Purpose**: Create a new workflow case instance

**URL Parameters**:
- `id` (required): Workflow specification ID

**Request**:
```json
{
  "case_id": "case_2025_001",
  "data": {
    "order_id": "ORD-2025-12345",
    "customer": "acme-corp",
    "amount": 50000.00,
    "priority": "high"
  },
  "deadline": "2025-12-17T17:00:00Z"
}
```

**Response** (201 Created):
```json
{
  "case_id": "case_2025_001",
  "workflow_id": "wf_7kJ9nM2xQ8pR",
  "state": "started",
  "created_at": "2025-11-17T11:00:00Z",
  "enabled_tasks": ["receive-order", "validate-order"],
  "data": {
    "order_id": "ORD-2025-12345"
  }
}
```

**Error Cases**:
- `404 NotFound`: Workflow not found
- `400 BadRequest`: Invalid data for workflow
- `400 BadRequest`: Deadline in the past
- `409 Conflict`: Case ID already exists

**Example**:
```bash
curl -X POST http://localhost:8080/api/v1/workflows/wf_7kJ9nM2xQ8pR/cases \
  -H "Content-Type: application/json" \
  -d @case.json
```

---

### Get Case Status

**Endpoint**: `GET /cases/{case_id}`

**Purpose**: Get current status of a workflow case

**URL Parameters**:
- `case_id` (required): Case instance ID

**Query Parameters**:
- `include_history` (optional): Include execution history (true/false)
- `include_data` (optional): Include case data payload (true/false)

**Response** (200 OK):
```json
{
  "case_id": "case_2025_001",
  "workflow_id": "wf_7kJ9nM2xQ8pR",
  "state": "active",
  "progress": {
    "completed_tasks": 3,
    "total_tasks": 12,
    "percentage": 25
  },
  "enabled_tasks": ["validate-order", "check-inventory"],
  "data": {
    "order_id": "ORD-2025-12345",
    "status": "validating"
  },
  "created_at": "2025-11-17T11:00:00Z",
  "deadline": "2025-12-17T17:00:00Z",
  "execution_time_ms": 1245,
  "traces": {
    "otel_trace_id": "4bf92f3577b34da6a3ce929d0e0e4736"
  }
}
```

**Error Cases**:
- `404 NotFound`: Case not found
- `410 Gone`: Case has been archived

---

### List Cases

**Endpoint**: `GET /cases`

**Purpose**: List cases with filtering and search

**Query Parameters**:
- `workflow_id` (optional): Filter by workflow
- `state` (optional): active/completed/failed/suspended
- `limit` (optional): Max results (default: 20)
- `offset` (optional): Pagination
- `created_after` (optional): ISO timestamp filter
- `created_before` (optional): ISO timestamp filter

**Response** (200 OK):
```json
{
  "total": 5243,
  "count": 20,
  "offset": 0,
  "cases": [
    {
      "case_id": "case_2025_001",
      "workflow_id": "wf_7kJ9nM2xQ8pR",
      "state": "active",
      "progress": 25,
      "enabled_tasks": 2,
      "created_at": "2025-11-17T11:00:00Z"
    }
  ],
  "_links": {
    "next": "/api/v1/cases?offset=20"
  }
}
```

---

## Execution & Control

### Enable Task

**Endpoint**: `POST /cases/{case_id}/tasks/{task_id}/enable`

**Purpose**: Manually enable a task (if guard allows)

**URL Parameters**:
- `case_id` (required): Case ID
- `task_id` (required): Task name/ID

**Request** (optional):
```json
{
  "data": {
    "approved_by": "manager@acme.com",
    "approval_timestamp": "2025-11-17T11:30:00Z"
  }
}
```

**Response** (200 OK):
```json
{
  "case_id": "case_2025_001",
  "task_id": "approval",
  "enabled": true,
  "execution_state": "ready",
  "timestamp": "2025-11-17T11:30:30Z"
}
```

**Error Cases**:
- `404 NotFound`: Case or task not found
- `400 BadRequest`: Guard condition prevents enabling
- `409 Conflict`: Task already enabled

---

### Complete Task

**Endpoint**: `POST /cases/{case_id}/tasks/{task_id}/complete`

**Purpose**: Mark a task as complete

**URL Parameters**:
- `case_id` (required): Case ID
- `task_id` (required): Task ID

**Request**:
```json
{
  "result": "approved",
  "output_data": {
    "approved_by": "john@acme.com",
    "approval_date": "2025-11-17T11:35:00Z"
  }
}
```

**Response** (200 OK):
```json
{
  "case_id": "case_2025_001",
  "task_id": "approval",
  "completed": true,
  "enabled_tasks": ["process-order", "notify-customer"],
  "timestamp": "2025-11-17T11:35:30Z"
}
```

**Error Cases**:
- `404 NotFound`: Case or task not found
- `400 BadRequest`: Task not enabled
- `409 Conflict`: Task already completed

---

### Cancel Case

**Endpoint**: `POST /cases/{case_id}/cancel`

**Purpose**: Cancel an active workflow case

**URL Parameters**:
- `case_id` (required): Case ID

**Request**:
```json
{
  "reason": "Order cancelled by customer",
  "notes": "Customer preference - switching vendors"
}
```

**Response** (200 OK):
```json
{
  "case_id": "case_2025_001",
  "state": "cancelled",
  "cancelled_at": "2025-11-17T11:40:00Z",
  "completed_tasks": 3,
  "total_tasks": 12
}
```

**Error Cases**:
- `404 NotFound`: Case not found
- `409 Conflict`: Case already completed/failed

---

### Suspend Case

**Endpoint**: `POST /cases/{case_id}/suspend`

**Purpose**: Temporarily suspend case execution

**Request**:
```json
{
  "reason": "Awaiting external approval"
}
```

**Response** (200 OK):
```json
{
  "case_id": "case_2025_001",
  "state": "suspended",
  "suspended_at": "2025-11-17T11:45:00Z"
}
```

---

### Resume Case

**Endpoint**: `POST /cases/{case_id}/resume`

**Purpose**: Resume a suspended case

**Response** (200 OK):
```json
{
  "case_id": "case_2025_001",
  "state": "active",
  "resumed_at": "2025-11-17T12:00:00Z"
}
```

---

## Monitoring & Observability

### Get Case Execution History

**Endpoint**: `GET /cases/{case_id}/history`

**Purpose**: Get detailed execution trace of a case

**Query Parameters**:
- `task_id` (optional): Filter to specific task
- `include_data` (optional): Include data payloads
- `include_telemetry` (optional): Include OTEL spans

**Response** (200 OK):
```json
{
  "case_id": "case_2025_001",
  "events": [
    {
      "timestamp": "2025-11-17T11:00:00Z",
      "type": "case_created",
      "task_id": null
    },
    {
      "timestamp": "2025-11-17T11:00:05Z",
      "type": "task_enabled",
      "task_id": "receive-order"
    },
    {
      "timestamp": "2025-11-17T11:00:30Z",
      "type": "task_completed",
      "task_id": "receive-order",
      "duration_ms": 25
    }
  ]
}
```

---

### Get Workflow Metrics

**Endpoint**: `GET /workflows/{id}/metrics`

**Purpose**: Get performance metrics for a workflow

**Query Parameters**:
- `time_range` (optional): 1h/24h/7d/30d (default: 24h)

**Response** (200 OK):
```json
{
  "workflow_id": "wf_7kJ9nM2xQ8pR",
  "time_range": "24h",
  "metrics": {
    "cases_created": 847,
    "cases_completed": 823,
    "cases_failed": 12,
    "cases_active": 12,
    "avg_duration_ms": 2847,
    "p95_duration_ms": 5200,
    "p99_duration_ms": 8100
  }
}
```

---

## Validation & Analysis

### Validate Workflow

**Endpoint**: `POST /workflows/validate`

**Purpose**: Validate a Turtle workflow before registration

**Request**:
```json
{
  "spec": "workflow:Process @prefix...",
  "check_deadlock": true,
  "check_soundness": true,
  "run_formal_verification": true
}
```

**Response** (200 OK):
```json
{
  "valid": true,
  "warnings": [],
  "errors": [],
  "validation_time_ms": 245,
  "checks": {
    "syntax": "passed",
    "deadlock": "no_deadlock_detected",
    "soundness": "sound",
    "fitness": 0.98
  }
}
```

**Response** (400 BadRequest - validation failed):
```json
{
  "valid": false,
  "errors": [
    {
      "type": "deadlock_detected",
      "message": "Sync join without matching split at line 42",
      "location": "task:approve-order"
    }
  ],
  "warnings": [
    {
      "type": "unused_task",
      "message": "Task 'legacy-approval' is never reachable"
    }
  ]
}
```

---

### Check Deadlock

**Endpoint**: `POST /workflows/{id}/check-deadlock`

**Purpose**: Run deadlock detection on a registered workflow

**Response** (200 OK):
```json
{
  "workflow_id": "wf_7kJ9nM2xQ8pR",
  "has_deadlock": false,
  "check_time_ms": 180,
  "message": "No deadlock patterns detected"
}
```

---

## Status Codes & Errors

### HTTP Status Codes

| Code | Meaning | Common Cause |
|------|---------|--------------|
| **200** | OK | Request succeeded |
| **201** | Created | Resource created successfully |
| **204** | No Content | Successful but no response body |
| **400** | Bad Request | Invalid input or malformed JSON |
| **401** | Unauthorized | Missing/invalid authentication |
| **403** | Forbidden | Insufficient permissions |
| **404** | Not Found | Resource not found |
| **409** | Conflict | Resource already exists or state conflict |
| **413** | Payload Too Large | Request exceeds size limit |
| **500** | Server Error | Internal server error |
| **503** | Service Unavailable | Server overloaded or maintenance |

### Error Response Format

All error responses follow this format:

```json
{
  "error": "BadRequest",
  "code": "INVALID_WORKFLOW_SPEC",
  "message": "Turtle syntax error at line 42: unexpected token",
  "details": {
    "line": 42,
    "column": 15,
    "token": "invalid"
  },
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736"
}
```

### Common Error Codes

| Code | HTTP | Meaning |
|------|------|---------|
| `INVALID_WORKFLOW_SPEC` | 400 | Malformed Turtle/RDF |
| `WORKFLOW_NOT_FOUND` | 404 | Workflow ID doesn't exist |
| `CASE_NOT_FOUND` | 404 | Case ID doesn't exist |
| `TASK_NOT_FOUND` | 404 | Task not in case |
| `GUARD_VIOLATION` | 400 | Guard condition not met |
| `DEADLOCK_DETECTED` | 400 | Cannot register - deadlock found |
| `CASE_IN_WRONG_STATE` | 409 | Cannot perform action in current state |
| `UNAUTHORIZED` | 401 | Not authenticated |
| `PERMISSION_DENIED` | 403 | Authenticated but not authorized |
| `RESOURCE_EXHAUSTED` | 429 | Rate limit exceeded |
| `INTERNAL_ERROR` | 500 | Server error |

---

## Rate Limiting

- **Default**: 1000 requests per minute per API key
- **Headers**:
  - `X-RateLimit-Limit`: Max requests in window
  - `X-RateLimit-Remaining`: Requests remaining
  - `X-RateLimit-Reset`: Unix timestamp when limit resets

---

## Authentication

### Bearer Token

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
  http://localhost:8080/api/v1/workflows
```

### Optional (can be disabled)

See configuration guide for auth settings.

---

## Related Documentation

- [Configuration Guide](../reference/configuration.md) - API server settings
- [Error Codes Reference](../reference/error-codes.md) - Detailed error information
- [How-To: Troubleshooting](../how-to/troubleshooting.md) - Common issues and solutions
- [Tutorial: Your First Workflow](../tutorials/first-workflow.md) - Hands-on example
