# ADR-001: Interface B Work Item Lifecycle Implementation

**Status**: Proposed
**Date**: 2025-11-08
**Deciders**: System Architect, Code Analyzer
**Priority**: 🔴 CRITICAL (Tier 1 - Production Blocker)

---

## Context and Problem Statement

KNHK workflow engine currently lacks a complete work item lifecycle implementation. YAWL Interface B defines 14+ operations for work item management (checkout, checkin, delegate, suspend, etc.), but knhk only implements basic work item creation. This prevents human task interaction and makes the engine unusable for 80% of enterprise workflows.

**Current State**:
- ✅ WorkItemService struct exists
- ✅ Basic work item creation works
- ❌ NO lifecycle operations (checkout, checkin, delegate)
- ❌ NO launch modes (offered, allocated, start-by-system)
- ❌ NO bulk query operations
- ❌ NO privilege management

**Impact**: Without Interface B, users cannot:
- Claim work items from their queue
- Start executing tasks
- Delegate tasks to other users
- Suspend/resume tasks
- Monitor work item status

---

## Decision Drivers

1. **YAWL Compatibility**: Must match YAWL Interface B operations for migration compatibility
2. **Performance**: Hot path operations must be ≤8 ticks (Chatman Constant)
3. **Safety**: Work item state transitions must be atomic and validated
4. **Observability**: All lifecycle events must emit OTEL spans
5. **Scalability**: Must support 1,000+ concurrent work items
6. **Compliance**: Must support audit trails for SOX/GDPR

---

## Considered Options

### Option 1: Monolithic WorkItemService (Current Approach)
```rust
pub struct WorkItemService {
    items: HashMap<WorkItemId, WorkItem>,
    state_machine: StateMachine,
}

impl WorkItemService {
    pub async fn checkout(&self, item_id: WorkItemId, user_id: UserId) -> Result<()> {
        // All logic in one place
    }
}
```

**Pros**:
- Simple architecture
- Fast (no cross-service calls)
- Easy to maintain

**Cons**:
- Poor separation of concerns
- Hard to test individual operations
- No modularity for partial implementations

---

### Option 2: Component-Based Architecture (Recommended)
```rust
pub struct WorkItemService {
    lifecycle_manager: Arc<LifecycleManager>,
    checkout_handler: Arc<CheckoutHandler>,
    delegation_handler: Arc<DelegationHandler>,
    offer_handler: Arc<OfferHandler>,
    allocation_handler: Arc<AllocationHandler>,
    pile_manager: Arc<PileManager>,
    privilege_checker: Arc<PrivilegeChecker>,
    repository: Arc<WorkItemRepository>,
}
```

**Pros**:
- ✅ Clear separation of concerns
- ✅ Each component independently testable
- ✅ Can implement incrementally (Sprint 1: checkout, Sprint 2: delegation)
- ✅ Aligns with C4 component diagram
- ✅ Easy to add new operations

**Cons**:
- More complex architecture
- Slightly more boilerplate

---

### Option 3: Actor-Based Architecture
```rust
pub struct WorkItemActor {
    mailbox: Receiver<WorkItemMessage>,
}

pub enum WorkItemMessage {
    Checkout(WorkItemId, UserId, oneshot::Sender<Result<()>>),
    Checkin(WorkItemId, UserId, oneshot::Sender<Result<()>>),
}
```

**Pros**:
- Natural concurrency model
- Good for high-contention scenarios

**Cons**:
- Overkill for current requirements
- Adds message passing overhead
- Harder to debug

---

## Decision Outcome

**Chosen Option**: **Option 2 - Component-Based Architecture**

### Rationale

1. **Incremental Implementation**: Can deliver Sprint 1 (checkout/checkin) without waiting for all operations
2. **Clear Ownership**: Each component has a single responsibility (SRP)
3. **Testability**: Unit tests per component + integration tests for workflows
4. **YAWL Alignment**: Maps cleanly to YAWL's interface operations
5. **Performance**: Components share data via Arc, no serialization overhead

### Architecture

```rust
// Core lifecycle state machine
pub struct LifecycleManager {
    allowed_transitions: HashMap<WorkItemState, Vec<WorkItemState>>,
}

impl LifecycleManager {
    pub fn can_transition(&self, from: WorkItemState, to: WorkItemState) -> bool {
        self.allowed_transitions
            .get(&from)
            .map(|allowed| allowed.contains(&to))
            .unwrap_or(false)
    }
}

// Checkout handler
pub struct CheckoutHandler {
    lifecycle_manager: Arc<LifecycleManager>,
    privilege_checker: Arc<PrivilegeChecker>,
    resource_manager: Arc<ResourceManager>,
    repository: Arc<WorkItemRepository>,
    otel: Arc<OtelIntegration>,
}

impl CheckoutHandler {
    #[instrument(skip(self))]
    pub async fn checkout(&self, item_id: WorkItemId, user_id: UserId) -> WorkflowResult<()> {
        // 1. Validate state transition (offered → executing)
        let item = self.repository.get(item_id).await?;
        if !self.lifecycle_manager.can_transition(item.state, WorkItemState::Executing) {
            return Err(WorkflowError::InvalidStateTransition);
        }

        // 2. Check user eligibility
        if !self.resource_manager.is_eligible(user_id, item.task_id).await? {
            return Err(WorkflowError::UserNotEligible);
        }

        // 3. Check privileges (if required)
        // Some workflows require "view-other" privilege to see item

        // 4. Acquire lock (optimistic concurrency control)
        self.repository.update_state(
            item_id,
            WorkItemState::Executing,
            Some(user_id),
            item.version, // CAS: Compare-And-Swap
        ).await?;

        // 5. Emit event
        self.otel.emit_event("work_item.checked_out", item_id, user_id);

        Ok(())
    }
}
```

---

## Implementation Plan

### Sprint 1 (Week 1-2): Core Operations

**Week 1**:
1. Implement `LifecycleManager` with state machine (2 days)
2. Implement `CheckoutHandler` with tests (2 days)
3. Implement `OfferHandler` for offered items (1 day)

**Week 2**:
4. Implement `AllocationHandler` for allocated items (2 days)
5. Implement `DelegationHandler` (2 days)
6. Integration tests for Sprint 1 operations (1 day)

**Deliverables**:
- ✅ Checkout/checkin operations functional
- ✅ Offered and allocated launch modes working
- ✅ Delegation working
- ✅ 80%+ test coverage
- ✅ OTEL instrumentation complete

---

### Sprint 2 (Week 3-4): Advanced Operations

**Week 3**:
1. Implement `SuspendHandler` for suspend/resume (2 days)
2. Implement `PrivilegeChecker` for all privileges (2 days)
3. Implement `PileManager` for pile-based work sharing (1 day)

**Week 4**:
4. Implement bulk query operations (2 days)
5. Implement chain execution (1 day)
6. Performance testing and optimization (2 days)

**Deliverables**:
- ✅ ALL 14 work item lifecycle operations
- ✅ ALL 5 launch modes
- ✅ Privilege management working
- ✅ Pile-based sharing working
- ✅ Performance benchmark: <200ms p99 latency

---

## Work Item State Machine

```
┌─────────┐
│ Enabled │ (Task becomes executable)
└────┬────┘
     │
     ├──> [User-initiated] ──> User claims ──> Executing
     │
     ├──> [Offered] ──> Offered ──> User accepts ──> Executing
     │                    │
     │                    └──> Decline ──> Reoffer
     │
     ├──> [Allocated] ──> Allocated ──> Auto-start ──> Executing
     │
     └──> [Start-by-System] ──> Executing (immediate)

Executing ──> Suspend ──> Suspended ──> Resume ──> Executing
Executing ──> Complete ──> Completed
Executing ──> Delegate ──> Offered (reassign)
Executing ──> Cancel ──> Cancelled
```

---

## API Design

### REST API Endpoints

```bash
# Checkout work item (acquire exclusive lock)
POST /api/v1/work-items/:id/checkout
{
  "user_id": "user123"
}
Response: 200 OK or 409 Conflict

# Checkin work item (release lock, save data)
POST /api/v1/work-items/:id/checkin
{
  "user_id": "user123",
  "data": { "form_field": "value" }
}
Response: 200 OK

# Start work item (begin execution)
POST /api/v1/work-items/:id/start
{
  "user_id": "user123"
}
Response: 200 OK

# Complete work item (finish task)
POST /api/v1/work-items/:id/complete
{
  "user_id": "user123",
  "data": { "result": "approved" }
}
Response: 200 OK

# Delegate work item (reassign to another user)
POST /api/v1/work-items/:id/delegate
{
  "from_user": "user123",
  "to_user": "user456"
}
Response: 200 OK or 403 Forbidden (no privilege)

# Suspend work item
POST /api/v1/work-items/:id/suspend
{
  "user_id": "user123"
}
Response: 200 OK

# Resume work item
POST /api/v1/work-items/:id/resume
{
  "user_id": "user123"
}
Response: 200 OK

# Get work items for user (worklist)
GET /api/v1/work-items?user_id=user123&state=offered
Response: 200 OK
[
  {
    "id": "wi-001",
    "task_name": "Approve Loan",
    "case_id": "case-123",
    "state": "offered",
    "enabled_at": "2025-11-08T10:00:00Z"
  }
]

# Get work items for case
GET /api/v1/cases/:case_id/work-items
Response: 200 OK

# Reoffer work item (redistribute to different users)
POST /api/v1/work-items/:id/reoffer
{
  "user_ids": ["user456", "user789"]
}
Response: 200 OK
```

---

## Database Schema

```sql
CREATE TABLE work_items (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    state TEXT NOT NULL, -- enabled, offered, allocated, executing, suspended, completed, cancelled
    assigned_user_id TEXT, -- NULL if not assigned
    enabled_at TIMESTAMP NOT NULL,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    data JSONB, -- Task data (input/output)
    version INTEGER NOT NULL DEFAULT 1, -- For optimistic locking
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    FOREIGN KEY (case_id) REFERENCES cases(id)
);

CREATE INDEX idx_work_items_case ON work_items(case_id);
CREATE INDEX idx_work_items_user ON work_items(assigned_user_id);
CREATE INDEX idx_work_items_state ON work_items(state);

CREATE TABLE work_item_events (
    id SERIAL PRIMARY KEY,
    work_item_id TEXT NOT NULL,
    event_type TEXT NOT NULL, -- checkout, checkin, start, complete, delegate, suspend, resume
    user_id TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT NOW(),
    data JSONB, -- Event-specific data
    FOREIGN KEY (work_item_id) REFERENCES work_items(id)
);

CREATE INDEX idx_work_item_events_item ON work_item_events(work_item_id);
CREATE INDEX idx_work_item_events_timestamp ON work_item_events(timestamp);
```

---

## Acceptance Criteria

### Must Have (Sprint 1)
- [ ] Checkout operation prevents double-booking
- [ ] Checkin operation saves intermediate data
- [ ] Start operation validates user eligibility
- [ ] Complete operation persists final result
- [ ] Delegate operation validates "delegate" privilege
- [ ] All operations emit OTEL spans
- [ ] All operations are atomic (CAS for concurrency)
- [ ] REST API endpoints return correct status codes
- [ ] Work item state machine enforces valid transitions

### Should Have (Sprint 2)
- [ ] Suspend/resume operations work correctly
- [ ] Pile-based work sharing functional
- [ ] Chain execution (auto-start next item)
- [ ] Bulk query operations (get all user's items)
- [ ] Privilege checker validates all 7 privileges
- [ ] Performance: <200ms p99 latency for checkout
- [ ] Performance: <8 ticks for state transitions (hot path)

### Could Have (Sprint 3+)
- [ ] Concurrent execution mode (multiple users, first wins)
- [ ] Piled execution (batch processing)
- [ ] Secondary resource allocation (equipment, facilities)
- [ ] Custom privilege plugins

---

## Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| State machine complexity | Medium | High | Comprehensive unit tests for all transitions |
| Concurrency bugs (double-booking) | High | Critical | Optimistic locking (CAS), integration tests |
| Performance degradation | Medium | High | Benchmark each operation, use connection pooling |
| Privilege escalation | Low | Critical | Security audit, principle of least privilege |
| Integration with ResourceManager | Medium | High | Design clear interface early, mock for testing |

---

## Success Metrics

### Performance
- Checkout operation: <8 ticks (hot path)
- REST API latency: <200ms p99
- Support 1,000 concurrent work items
- Support 100 concurrent users

### Quality
- 90%+ test coverage (unit + integration)
- Zero clippy warnings
- Zero `unwrap()`/`expect()` in production paths
- All error handling uses `Result<T, E>`

### Completeness
- 100% of 14 YAWL Interface B operations implemented
- 100% of 5 launch modes implemented
- 100% of 7 privileges implemented
- OTEL instrumentation for all operations

---

## References

- YAWL Interface B Specification: `vendors/yawl/src/org/yawlfoundation/yawl/engine/interfce/interfaceB/InterfaceB_EngineBasedClient.java`
- KNHK Work Item Service: `rust/knhk-workflow-engine/src/services/work_item.rs`
- YAWL Missing Features Report: `docs/YAWL_MISSING_FEATURES.md`
- C4 Component Diagram: `docs/architecture/c4-component-work-item-service.puml`

---

**Decision**: ✅ Approved for implementation
**Next Steps**: Begin Sprint 1 implementation (LifecycleManager, CheckoutHandler)
