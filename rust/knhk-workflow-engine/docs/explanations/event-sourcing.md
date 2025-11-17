# Explanation: Event Sourcing & State Management in KNHK

**Understanding How KNHK Maintains Complete Workflow History**

---

## Introduction

KNHK uses **Event Sourcing** to manage workflow state. This means:

> Instead of storing just the current state, KNHK stores **every event that happened**, and reconstructs state by replaying events.

**Traditional Approach**:
```
Database: {case_001: {status: "approved", amount: 500}}
               ↑
         (current state only)
         (history lost)
```

**Event Sourcing Approach**:
```
Event Log:
  1. CaseCreated (amount: 500)
  2. ValidateStarted
  3. ValidateCompleted
  4. ApprovalStarted
  5. ApprovalCompleted
  6. CaseCompleted
       ↓
 (replay events to reconstruct state at any point in time)
 (complete audit trail)
 (can recover from failures)
```

This document explains why event sourcing matters and how it works.

---

## Why Event Sourcing?

### Problem with Traditional State Storage

Traditional approach stores only current state:

```
case_001 = {
  status: "completed",
  amount: 500.00,
  approved_by: "manager_001",
  approved_at: "2025-11-17T10:00:00Z"
}
```

**Problems**:

1. **No History**
   - Can't see how we got here
   - Can't debug what went wrong
   - Can't answer "when was it approved?"

2. **Overwrite Loss**
   - Update state → old state lost forever
   - Can't rollback
   - Can't verify data integrity

3. **Concurrency Issues**
   - Multiple systems update same state → conflicts
   - Last-write-wins loses data
   - No clear audit trail of changes

4. **Compliance**
   - "What happened to this case?" ← can't answer
   - "Prove who approved it" ← no proof
   - "Show the audit trail" ← missing

### Solution: Event Sourcing

Instead of storing state, store **the events that created it**:

```
Event Log (immutable):
  Event 1: CaseCreated { case_id: 001, amount: 500 }
  Event 2: ValidateCompleted { validated: true }
  Event 3: ApprovalStarted { task: "approval" }
  Event 4: ApprovalCompleted { approved_by: "manager_001" }
  Event 5: CaseCompleted { status: "completed" }

Current State (derived):
  Replay all events → {
    status: "completed",
    amount: 500.00,
    approved_by: "manager_001"
  }
```

**Benefits**:

1. **Complete History**
   - Can replay events to any point in time
   - Can see exact sequence of what happened
   - Can answer "when" and "why" questions

2. **No Data Loss**
   - Events are immutable (append-only)
   - Can't accidentally lose data
   - Every decision is recorded

3. **Natural Concurrency**
   - Events are ordered (timestamp + event ID)
   - No conflicts → events compose naturally
   - Clear ordering of who did what

4. **Compliance & Audit**
   - Complete audit trail
   - Timestamps on every action
   - Can prove every decision
   - Regulatory compliance built-in

---

## How Event Sourcing Works in KNHK

### The Event Stream

Every workflow case produces an **event stream**:

```
Case: expense_approval_001
Time: 2025-11-17T10:00:00Z onwards

Events (in order):
┌──────────────────────────────────────────┐
│ Event 1: CaseCreated                     │
│ Timestamp: 2025-11-17T10:00:00.000Z     │
│ Data: {                                  │
│   case_id: "expense_approval_001",      │
│   workflow_id: "wf_expense",            │
│   data: {amount: 450.00}                │
│ }                                        │
└──────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────┐
│ Event 2: TaskEnabled                     │
│ Timestamp: 2025-11-17T10:00:00.050Z     │
│ Data: {                                  │
│   task_id: "submit_expense",            │
│   case_id: "expense_approval_001"       │
│ }                                        │
└──────────────────────────────────────────┘
                    ↓
┌──────────────────────────────────────────┐
│ Event 3: TaskCompleted                   │
│ Timestamp: 2025-11-17T10:00:05.200Z     │
│ Data: {                                  │
│   task_id: "submit_expense",            │
│   case_id: "expense_approval_001",      │
│   result: "submitted",                  │
│   duration_ms: 5150                     │
│ }                                        │
└──────────────────────────────────────────┘
                    ↓
         (events continue...)
```

### Event Types

KNHK defines specific event types:

#### Lifecycle Events

```json
{
  "type": "CaseCreated",
  "case_id": "case_001",
  "timestamp": "2025-11-17T10:00:00Z",
  "data": {
    "workflow_id": "wf_001",
    "initial_data": {...}
  }
}

{
  "type": "CaseSuspended",
  "case_id": "case_001",
  "timestamp": "2025-11-17T10:05:00Z",
  "data": {
    "reason": "Awaiting approval"
  }
}

{
  "type": "CaseResumed",
  "case_id": "case_001",
  "timestamp": "2025-11-17T10:10:00Z"
}

{
  "type": "CaseCompleted",
  "case_id": "case_001",
  "timestamp": "2025-11-17T10:30:00Z",
  "data": {
    "duration_ms": 1800000
  }
}
```

#### Task Events

```json
{
  "type": "TaskEnabled",
  "case_id": "case_001",
  "task_id": "task_001",
  "timestamp": "2025-11-17T10:00:05Z"
}

{
  "type": "TaskStarted",
  "case_id": "case_001",
  "task_id": "task_001",
  "timestamp": "2025-11-17T10:00:06Z"
}

{
  "type": "TaskCompleted",
  "case_id": "case_001",
  "task_id": "task_001",
  "timestamp": "2025-11-17T10:00:30Z",
  "data": {
    "result": "approved",
    "duration_ms": 24000,
    "output_data": {...}
  }
}

{
  "type": "TaskFailed",
  "case_id": "case_001",
  "task_id": "task_001",
  "timestamp": "2025-11-17T10:00:35Z",
  "data": {
    "error": "Guard condition not satisfied",
    "reason": "amount exceeds limit"
  }
}
```

#### Data Events

```json
{
  "type": "DataUpdated",
  "case_id": "case_001",
  "timestamp": "2025-11-17T10:00:10Z",
  "data": {
    "changes": {
      "status": "validating",
      "validation_start": "2025-11-17T10:00:10Z"
    }
  }
}
```

### Building State from Events

To get current state, KNHK **replays all events**:

```rust
// Pseudocode: How state is reconstructed

fn get_case_state(case_id: &str) -> CaseState {
    let mut state = CaseState::new();

    // Get all events for this case
    let events = read_events(case_id);

    // Replay each event
    for event in events {
        match event.type {
            "CaseCreated" => {
                state.case_id = event.data.case_id;
                state.workflow_id = event.data.workflow_id;
                state.status = "active";
            }
            "TaskEnabled" => {
                state.enabled_tasks.insert(event.data.task_id);
            }
            "TaskCompleted" => {
                state.enabled_tasks.remove(&event.data.task_id);
                state.completed_tasks.push(event.data.task_id);
            }
            "DataUpdated" => {
                state.data.merge(event.data.changes);
            }
            // ... handle other events
        }
    }

    state
}
```

**Key point**: Current state is always **derived** from events, never stored directly.

### Snapshots for Performance

Replaying all events every time would be slow for old cases. KNHK uses **snapshots**:

```
Events 1-1000: Initial events
Snapshot at event 1000: {case_state: {...}}
Events 1001-2000: More events
(current state = snapshot + replay from 1001)

Instead of:
current state = replay all 2000 events
```

**Snapshot Process**:

```
Event 1000 reached
    ↓
[Snapshot Current State]
    ↓
Save Snapshot + Event 1000 metadata
    ↓
Archive Events 1-1000 (less frequently accessed)
    ↓
Continue with Events 1001+
```

---

## Auditability: The Receipt Chain

### What is a Receipt?

A **receipt** proves that an event happened and is cryptographically linked to previous events.

```
Event 1: CaseCreated
Receipt 1: {
  hash: "blake3:a1b2c3d4...",
  timestamp: "2025-11-17T10:00:00Z"
}
           ↓ [linked]
Event 2: TaskEnabled
Receipt 2: {
  previous_hash: "blake3:a1b2c3d4...",
  hash: "blake3:e5f6g7h8...",
  timestamp: "2025-11-17T10:00:05Z"
}
           ↓ [linked]
Event 3: TaskCompleted
Receipt 3: {
  previous_hash: "blake3:e5f6g7h8...",
  hash: "blake3:i9j0k1l2...",
  timestamp: "2025-11-17T10:00:30Z"
}

Receipt Chain:
a1b2... → e5f6... → i9j0...
(unbroken chain of custody)
```

### How Receipts Work

**BLAKE3 Hashing**:

```
Receipt for Event N:
  hash = blake3(
    previous_receipt.hash +
    event_id +
    timestamp +
    event_data +
    signer_key
  )
```

**Verification**:

```
Given: Event N with receipt R_n
Want: Prove no one tampered with Event N

Check:
1. Recalculate R_n hash from event data
2. Compare with stored R_n hash
3. If equal: Event is unmodified ✓
4. If different: Event was tampered ✗

Also check:
5. Verify R_n.previous_hash == R_(n-1).hash
6. If equal: Chain is unbroken ✓
7. If different: Someone deleted/modified events ✗
```

### Compliance Benefits

Event sourcing with receipts provides:

**Non-repudiation**:
- "Who approved this?" ← Event shows who
- "Can you prove it?" ← Receipt proves it
- Cannot deny later

**Immutability**:
- Events are append-only
- Can't delete or modify events
- Chain breaks if tampered

**Auditability**:
- Complete history available
- Timestamps on everything
- Can replay any point in time

**Compliance**:
- GDPR: Track all data access
- HIPAA: Audit trail for healthcare
- SOC2: Demonstrate controls

---

## Operational Benefits

### Point-in-Time Reconstruction

You can ask "What was the state at 2pm?"

```
Case: expense_approval_001
Time: "2025-11-17T14:00:00Z"

Replay events up to that time:
  Event 1: CaseCreated (10:00)
  Event 2: TaskEnabled (10:00)
  Event 3: TaskCompleted (10:05)
  Event 4: TaskEnabled (10:05) ← up to 14:00
  [skip Event 5 onwards (after 14:00)]

Result: State at 14:00
```

**Useful for**:
- Debugging: "What was wrong?"
- Retrospective: "How did it fail?"
- Analysis: "Where was the bottleneck?"
- Testing: "Replay this scenario"

### Event-Driven Integration

External systems can **subscribe to events**:

```
KNHK Event Stream
    ├─→ Email System (send on TaskCompleted)
    ├─→ Slack (notify on CaseCompleted)
    ├─→ Analytics (track metrics)
    ├─→ Data Warehouse (populate reporting)
    └─→ Compliance System (audit trail)

All from same event source → no sync issues
```

### Failure Recovery

If system crashes, can recover by **replaying events**:

```
System crashes at event 5000

On restart:
  1. Load last snapshot (event 3000)
  2. Replay events 3001-5000
  3. Recover full state
  4. Continue from event 5001

Result: Zero data loss, automatic recovery
```

---

## Event Sourcing Patterns

### Event Types Hierarchy

```
BaseEvent
├─ CaseEvent
│  ├─ CaseCreated
│  ├─ CaseSuspended
│  ├─ CaseResumed
│  └─ CaseCompleted
├─ TaskEvent
│  ├─ TaskEnabled
│  ├─ TaskStarted
│  ├─ TaskCompleted
│  └─ TaskFailed
├─ DataEvent
│  ├─ DataUpdated
│  └─ DataDeleted
└─ ErrorEvent
   ├─ GuardViolation
   ├─ TimeoutOccurred
   └─ ExternalError
```

### Event Versioning

Events can evolve:

```json
// Version 1 (original)
{
  "type": "TaskCompleted",
  "task_id": "task_001",
  "result": "approved"
}

// Version 2 (added fields)
{
  "type": "TaskCompleted",
  "version": 2,
  "task_id": "task_001",
  "result": "approved",
  "duration_ms": 5000,        // New
  "output_data": {...}        // New
}

// Replay logic handles both versions
match event.version {
  1 => { /* parse v1 */ }
  2 => { /* parse v2 with new fields */ }
}
```

### Event Sourcing + CQRS

**Command Query Responsibility Segregation**:

```
           API Request
                ↓
         [Write Side]
    Generate Event
         ↓
    Store Event
         ↓
         [Read Side]
    Rebuild Projection
         ↓
         [Query Side]
    Answer Query
         ↓
      Response
```

**Benefit**: Reads don't slow down writes

---

## Trade-offs

### Advantages

✅ **Complete History**
- Can answer any historical question
- Perfect for auditing

✅ **Natural Concurrency**
- Events compose without conflicts
- Multiple systems can consume same stream

✅ **Resilience**
- Recover from failures automatically
- No state corruption possible

✅ **Compliance**
- Immutable audit trail
- Timestamps on everything

### Disadvantages

⚠️ **Storage**
- Stores all events (more data than current state)
- But with snapshots, this is manageable

⚠️ **Complexity**
- Different mental model from CRUD
- Requires thinking in events

⚠️ **Eventual Consistency**
- State is derived (takes time to rebuild)
- But KNHK uses snapshots to minimize this

⚠️ **Debugging**
- Need to understand event flow
- But helps with some debugging scenarios

---

## Real-World Example

### Order Approval Workflow

```
Traditional Approach:
  order_001 = {status: "approved", amount: 500}

  Q: "When was it approved?"
  A: Unknown (no timestamp)

  Q: "Who approved it?"
  A: Unknown (not recorded)

  Q: "Show the audit trail"
  A: Doesn't exist

Event Sourcing Approach:
  Events:
    OrderCreated (10:00) - amount: 500
    OrderValidated (10:01) - status: valid
    ApprovalRequested (10:05) - supervisor: john_doe
    ApprovalCompleted (10:10) - approved_by: john_doe, result: approved

  Q: "When was it approved?"
  A: 10:10 (from ApprovalCompleted event)

  Q: "Who approved it?"
  A: john_doe (from ApprovalCompleted event)

  Q: "Show the audit trail"
  A: (all events above, timestamped and receipted)
```

### Recovery Scenario

```
Case: expense_001
Status: Running for 2 hours

System crashes at 12:00

Before recovery:
  - Last snapshot: Event 500 (event 1000)
  - Events 501-600: Lost in crash

Recovery process:
  1. Load snapshot (state at event 1000)
  2. Read remaining events (1001-1230)
  3. Replay to rebuild state
  4. Verify receipts (unbroken chain)
  5. Continue from event 1231

Result: Case continues as if nothing happened
```

---

## Implementation in KNHK

### Storage Backend

KNHK uses **Sled** (embedded database) by default:

```toml
[storage]
backend = "sled"
db_path = "./data/knhk"

# Sled is event-sourcing friendly:
# - Append-only writes
# - Immutable data
# - ACID transactions
# - Built-in compression
```

Optionally uses **PostgreSQL**:

```toml
[storage]
backend = "postgres"

# PostgreSQL stores:
# - event_log table (immutable events)
# - snapshots table (periodic snapshots)
# - receipts table (cryptographic proofs)
```

### Querying Events

```bash
# Get all events for a case
curl http://localhost:8080/api/v1/cases/case_001/history

# Response:
{
  "case_id": "case_001",
  "events": [
    {
      "event_id": 1,
      "timestamp": "2025-11-17T10:00:00Z",
      "type": "CaseCreated",
      "data": {...}
    },
    ...
  ]
}
```

---

## Key Takeaways

| Concept | Meaning | Benefit |
|---------|---------|---------|
| **Event** | Something that happened | Complete record |
| **Event Stream** | All events for a case | Full history |
| **Event Sourcing** | Store events, derive state | No data loss |
| **Snapshot** | Cached state at a point | Fast queries |
| **Receipt** | Cryptographic proof | Audit trail |
| **Immutable** | Can't be changed | Compliance |
| **Auditability** | Can prove what happened | Regulatory |

---

## Related Documentation

- [Explanation: Σ-Π-μ-O-MAPE-K Layers](./sigma-pi-mu-mape-k.md) - O layer creates events
- [How-To: Troubleshooting](../how-to/troubleshooting.md) - Use history for debugging
- [Configuration Guide](../reference/configuration.md) - Storage backends
- [API Reference](../reference/api-endpoints.md) - History endpoint
