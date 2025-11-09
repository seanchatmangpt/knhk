# Component Blueprint: Work Item Service

**Component**: Work Item Service (Interface B)
**Priority**: 🔴 CRITICAL (Tier 1 - Production Blocker)
**Estimated Effort**: 4-6 weeks
**Dependencies**: Resource Manager, State Store, OTEL Integration

---

## Executive Summary

The Work Item Service implements YAWL Interface B, providing complete work item lifecycle management for human task interaction. This is the **most critical missing feature** in knhk, as it enables:
- Work item checkout/checkin (acquire/release exclusive locks)
- Task delegation and reallocation
- Multiple launch modes (offered, allocated, system-initiated)
- Pile-based work sharing (shared queues)
- Privilege management (suspend-case, skip, delegate, etc.)

**Without this component**: knhk can only execute automated workflows. Human participants cannot claim or execute tasks.

---

## Business Value

### Value Proposition

**Tier 1 Enterprise Features Enabled**:
1. **Human Task Management** - Users can claim and execute tasks from worklist
2. **Workload Distribution** - System distributes work fairly across participants
3. **Delegation Support** - Tasks can be reassigned when users are unavailable
4. **Audit Trails** - Complete history of who did what and when
5. **Compliance** - Work item locking prevents double-execution

**ROI**:
- **80% of enterprise workflows** require human task interaction
- **Unblocks YAWL migration** - Interface B is mandatory for compatibility
- **Competitive parity** - All commercial BPM systems provide this functionality

---

## Functional Requirements

### FR-1: Work Item Lifecycle Operations

**14 Required Operations** (from YAWL Interface B):

1. **checkEligibleToStart(itemID, userID)** → `bool`
   - Validates if user can start work item
   - Checks resource eligibility + privileges
   - Returns true/false

2. **checkoutWorkItem(itemID, userID)** → `Result<(), Error>`
   - Acquires exclusive lock on work item
   - Transitions state: `offered` → `executing`
   - Prevents other users from claiming
   - Emits `work_item.checked_out` event

3. **checkinWorkItem(itemID, userID, data)** → `Result<(), Error>`
   - Releases lock, saves intermediate data
   - Transitions state: `executing` → `offered`
   - Allows other users to claim
   - Emits `work_item.checked_in` event

4. **startWorkItem(itemID, userID)** → `Result<(), Error>`
   - Begins task execution
   - Transitions state: `allocated` → `executing`
   - Validates user is assigned to item
   - Emits `work_item.started` event

5. **completeWorkItem(itemID, userID, data)** → `Result<(), Error>`
   - Finishes task, commits final result
   - Transitions state: `executing` → `completed`
   - Updates case data with output
   - Emits `work_item.completed` event

6. **suspendWorkItem(itemID, userID)** → `Result<(), Error>`
   - Pauses task execution
   - Transitions state: `executing` → `suspended`
   - Requires `suspend` privilege
   - Emits `work_item.suspended` event

7. **unsuspendWorkItem(itemID, userID)** → `Result<(), Error>`
   - Resumes task execution
   - Transitions state: `suspended` → `executing`
   - Emits `work_item.resumed` event

8. **delegateWorkItem(itemID, fromUser, toUser)** → `Result<(), Error>`
   - Transfers ownership to another user
   - Transitions state: `executing` → `offered` → `allocated`
   - Requires `delegate` privilege
   - Validates toUser eligibility
   - Emits `work_item.delegated` event

9. **offerWorkItem(itemID, userID)** → `Result<(), Error>`
   - Adds item to user's worklist
   - Transitions state: `enabled` → `offered`
   - User can accept or decline
   - Emits `work_item.offered` event

10. **reoffer(itemID, userIDs)** → `Result<(), Error>`
    - Redistributes to different users
    - Removes from current users' worklists
    - Adds to new users' worklists
    - Emits `work_item.reoffered` event

11. **deallocate(itemID, userID)** → `Result<(), Error>`
    - Removes user allocation
    - Transitions state: `allocated` → `offered`
    - Makes available for others to claim
    - Emits `work_item.deallocated` event

12. **reallocateStateless(itemID, userID)** → `Result<(), Error>`
    - Reassigns without state loss
    - Transitions state: `allocated` → `allocated` (new user)
    - Preserves all data
    - Emits `work_item.reallocated` event

13. **reallocateStateful(itemID, userID, data)** → `Result<(), Error>`
    - Reassigns with state update
    - Allows data modification during reallocation
    - Emits `work_item.reallocated` event

14. **cancelWorkItem(itemID, userID)** → `Result<(), Error>`
    - Aborts work item
    - Transitions state: ANY → `cancelled`
    - Requires `cancel` privilege
    - Emits `work_item.cancelled` event

---

### FR-2: Bulk Query Operations

**Required Queries**:

1. **getWorkItemsForUser(userID, state?)** → `Vec<WorkItem>`
   - Returns all work items for user (worklist)
   - Optional state filter (offered, allocated, executing, suspended)
   - Sorted by priority, enabled_at timestamp

2. **getWorkItemsForCase(caseID)** → `Vec<WorkItem>`
   - Returns all work items for case
   - Used for case monitoring
   - Includes completed and cancelled items

3. **getWorkItemsForSpec(specID, state?)** → `Vec<WorkItem>`
   - Returns all work items for specification
   - Used for workload monitoring
   - Optional state filter

4. **getEnabledWorkItems()** → `Vec<WorkItem>`
   - Returns all enabled items across system
   - Used for admin monitoring

5. **getExecutingWorkItems()** → `Vec<WorkItem>`
   - Returns all executing items
   - Used for active workload monitoring

6. **getSuspendedWorkItems()** → `Vec<WorkItem>`
   - Returns all suspended items
   - Used for exception handling

---

### FR-3: Launch Modes

**5 Launch Modes** (from YAWL):

1. **User-Initiated (Pull)**:
   - User browses worklist
   - Clicks "Claim" button
   - Work item transitions: `enabled` → `offered` → `executing`
   - Most common mode for human tasks

2. **Offered (Push)**:
   - System pushes to user's queue
   - User sees in "Offered Items" section
   - User can accept (→ executing) or decline (→ reoffer)
   - Used for assigned tasks

3. **Allocated (Assigned)**:
   - System assigns to specific user
   - User must execute (no decline option)
   - Transitions: `enabled` → `allocated` → `executing`
   - Used for mandatory tasks (e.g., assigned by manager)

4. **Start-by-System (Automatic)**:
   - Auto-starts when enabled
   - No user interaction
   - Used for automated tasks (connectors, codelets)
   - Transitions: `enabled` → `executing` (immediate)

5. **Concurrent (Parallel)**:
   - Multiple users work on same item
   - First to complete wins
   - Others cancelled automatically
   - Used for competitive scenarios (e.g., "first reviewer approves")

---

### FR-4: Privilege Management

**7 Privileges** (from YAWL):

1. **suspend-case**: Can suspend/resume entire case
2. **skip**: Can skip task without execution
3. **pile**: Can access pile-based shared queues
4. **reorder**: Can change work item order in queue
5. **view-other**: Can view other users' work items
6. **chain**: Can auto-start next item after completion
7. **manage-resourcing**: Can reallocate work items

**Privilege Checking**:
```rust
pub struct PrivilegeChecker {
    user_privileges: Arc<UserRepository>,
}

impl PrivilegeChecker {
    pub async fn has_privilege(&self, user_id: UserId, privilege: Privilege) -> WorkflowResult<bool> {
        let user = self.user_privileges.get(user_id).await?;
        Ok(user.privileges.contains(&privilege))
    }

    pub async fn require_privilege(&self, user_id: UserId, privilege: Privilege) -> WorkflowResult<()> {
        if !self.has_privilege(user_id, privilege).await? {
            return Err(WorkflowError::PrivilegeViolation {
                user_id,
                privilege,
            });
        }
        Ok(())
    }
}
```

---

## Technical Design

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                  Work Item Service                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌────────────────┐  ┌──────────────────┐             │
│  │ Lifecycle      │  │ Checkout         │             │
│  │ Manager        │◄─┤ Handler          │             │
│  └────────────────┘  └──────────────────┘             │
│         ▲                                              │
│         │            ┌──────────────────┐             │
│         ├───────────►│ Delegation       │             │
│         │            │ Handler          │             │
│         │            └──────────────────┘             │
│         │                                              │
│         │            ┌──────────────────┐             │
│         ├───────────►│ Offer            │             │
│         │            │ Handler          │             │
│         │            └──────────────────┘             │
│         │                                              │
│         │            ┌──────────────────┐             │
│         └───────────►│ Allocation       │             │
│                      │ Handler          │             │
│                      └──────────────────┘             │
│                                                         │
│  ┌────────────────┐  ┌──────────────────┐             │
│  │ Pile           │  │ Privilege        │             │
│  │ Manager        │  │ Checker          │             │
│  └────────────────┘  └──────────────────┘             │
│                                                         │
│  ┌─────────────────────────────────────────┐           │
│  │ Work Item Repository                    │           │
│  │ (Persistent storage)                    │           │
│  └─────────────────────────────────────────┘           │
│                                                         │
└─────────────────────────────────────────────────────────┘
          │                    │                    │
          ▼                    ▼                    ▼
   ┌──────────┐       ┌──────────────┐      ┌──────────┐
   │ State    │       │ Resource     │      │ OTEL     │
   │ Store    │       │ Manager      │      │ Exporter │
   └──────────┘       └──────────────┘      └──────────┘
```

---

### Data Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: WorkItemId,
    pub case_id: CaseId,
    pub task_id: TaskId,
    pub task_name: String,
    pub state: WorkItemState,
    pub assigned_user_id: Option<UserId>,
    pub enabled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub data: serde_json::Value, // Task data (input/output)
    pub version: u32, // For optimistic locking (CAS)
    pub launch_mode: LaunchMode,
    pub priority: u8, // 0-255, higher = more important
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkItemState {
    Enabled,    // Task can be executed
    Offered,    // Offered to user(s)
    Allocated,  // Assigned to specific user
    Executing,  // User is working on it
    Suspended,  // Paused
    Completed,  // Finished successfully
    Cancelled,  // Aborted
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaunchMode {
    UserInitiated,
    Offered,
    Allocated,
    StartBySystem,
    Concurrent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemEvent {
    pub id: u64,
    pub work_item_id: WorkItemId,
    pub event_type: WorkItemEventType,
    pub user_id: Option<UserId>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkItemEventType {
    Enabled,
    Offered,
    CheckedOut,
    CheckedIn,
    Started,
    Completed,
    Suspended,
    Resumed,
    Delegated,
    Cancelled,
    Reoffered,
    Deallocated,
    Reallocated,
}
```

---

### State Machine

```
┌─────────┐
│ Enabled │ (Task becomes executable)
└────┬────┘
     │
     ├──> [User-Initiated] ──> User claims ──> Executing
     │
     ├──> [Offered] ──> Offered ──> Accept ──> Executing
     │                    │
     │                    └──> Decline ──> Reoffer ──> Offered
     │
     ├──> [Allocated] ──> Allocated ──> Start ──> Executing
     │
     ├──> [Start-by-System] ──> Executing (immediate)
     │
     └──> [Concurrent] ──> Executing (multiple users)

┌───────────┐
│ Executing │
└─────┬─────┘
      │
      ├──> Complete ──> Completed
      │
      ├──> Suspend ──> Suspended ──> Resume ──> Executing
      │
      ├──> Delegate ──> Offered (reassign)
      │
      ├──> Checkin ──> Offered (save progress)
      │
      └──> Cancel ──> Cancelled
```

**Allowed Transitions** (enforced by `LifecycleManager`):
- `Enabled` → `Offered`, `Allocated`, `Executing`
- `Offered` → `Executing`, `Allocated`, `Cancelled`
- `Allocated` → `Executing`, `Offered`, `Cancelled`
- `Executing` → `Completed`, `Suspended`, `Offered` (via checkin/delegate), `Cancelled`
- `Suspended` → `Executing`, `Cancelled`
- `Completed` → (terminal state)
- `Cancelled` → (terminal state)

---

## Implementation Guide

### Step 1: Lifecycle Manager (2 days)

**File**: `rust/knhk-workflow-engine/src/services/work_item/lifecycle.rs`

```rust
pub struct LifecycleManager {
    allowed_transitions: HashMap<WorkItemState, Vec<WorkItemState>>,
}

impl LifecycleManager {
    pub fn new() -> Self {
        let mut allowed_transitions = HashMap::new();
        allowed_transitions.insert(
            WorkItemState::Enabled,
            vec![WorkItemState::Offered, WorkItemState::Allocated, WorkItemState::Executing],
        );
        allowed_transitions.insert(
            WorkItemState::Offered,
            vec![WorkItemState::Executing, WorkItemState::Allocated, WorkItemState::Cancelled],
        );
        // ... (continue for all states)

        Self { allowed_transitions }
    }

    pub fn can_transition(&self, from: WorkItemState, to: WorkItemState) -> bool {
        self.allowed_transitions
            .get(&from)
            .map(|allowed| allowed.contains(&to))
            .unwrap_or(false)
    }

    pub fn validate_transition(&self, from: WorkItemState, to: WorkItemState) -> WorkflowResult<()> {
        if !self.can_transition(from, to) {
            return Err(WorkflowError::InvalidStateTransition { from, to });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        let manager = LifecycleManager::new();
        assert!(manager.can_transition(WorkItemState::Enabled, WorkItemState::Offered));
        assert!(manager.can_transition(WorkItemState::Offered, WorkItemState::Executing));
        assert!(manager.can_transition(WorkItemState::Executing, WorkItemState::Completed));
    }

    #[test]
    fn test_invalid_transitions() {
        let manager = LifecycleManager::new();
        assert!(!manager.can_transition(WorkItemState::Completed, WorkItemState::Executing));
        assert!(!manager.can_transition(WorkItemState::Enabled, WorkItemState::Completed));
    }
}
```

---

### Step 2: Checkout Handler (2 days)

**File**: `rust/knhk-workflow-engine/src/services/work_item/checkout.rs`

```rust
pub struct CheckoutHandler {
    lifecycle_manager: Arc<LifecycleManager>,
    privilege_checker: Arc<PrivilegeChecker>,
    resource_manager: Arc<ResourceManager>,
    repository: Arc<WorkItemRepository>,
    otel: Arc<OtelIntegration>,
}

impl CheckoutHandler {
    #[instrument(skip(self), fields(item_id = %item_id, user_id = %user_id))]
    pub async fn checkout(&self, item_id: WorkItemId, user_id: UserId) -> WorkflowResult<()> {
        // 1. Get work item
        let item = self.repository.get(item_id).await?;

        // 2. Validate state transition
        self.lifecycle_manager.validate_transition(item.state, WorkItemState::Executing)?;

        // 3. Check user eligibility
        if !self.resource_manager.is_eligible(user_id, item.task_id).await? {
            return Err(WorkflowError::UserNotEligible { user_id, task_id: item.task_id });
        }

        // 4. Acquire lock (optimistic concurrency control)
        let success = self.repository.update_state_cas(
            item_id,
            WorkItemState::Executing,
            Some(user_id),
            item.version, // Compare-And-Swap: only update if version matches
        ).await?;

        if !success {
            return Err(WorkflowError::WorkItemAlreadyExecuting { item_id });
        }

        // 5. Emit event
        self.otel.emit_event("work_item.checked_out", &[
            ("item_id", item_id.to_string()),
            ("user_id", user_id.to_string()),
            ("task_name", item.task_name.clone()),
        ]);

        tracing::info!(
            item_id = %item_id,
            user_id = %user_id,
            "Work item checked out"
        );

        Ok(())
    }

    #[instrument(skip(self), fields(item_id = %item_id, user_id = %user_id))]
    pub async fn checkin(
        &self,
        item_id: WorkItemId,
        user_id: UserId,
        data: serde_json::Value,
    ) -> WorkflowResult<()> {
        // 1. Get work item
        let item = self.repository.get(item_id).await?;

        // 2. Validate user owns the item
        if item.assigned_user_id != Some(user_id) {
            return Err(WorkflowError::UserNotAssigned { item_id, user_id });
        }

        // 3. Validate state transition
        self.lifecycle_manager.validate_transition(item.state, WorkItemState::Offered)?;

        // 4. Save data and release lock
        self.repository.update_state_and_data(
            item_id,
            WorkItemState::Offered,
            None, // Release user assignment
            data,
        ).await?;

        // 5. Emit event
        self.otel.emit_event("work_item.checked_in", &[
            ("item_id", item_id.to_string()),
            ("user_id", user_id.to_string()),
        ]);

        Ok(())
    }
}
```

---

### Step 3: Work Item Repository (2 days)

**File**: `rust/knhk-workflow-engine/src/services/work_item/repository.rs`

```rust
pub struct WorkItemRepository {
    state_store: Arc<StateStore>,
}

impl WorkItemRepository {
    pub async fn get(&self, item_id: WorkItemId) -> WorkflowResult<WorkItem> {
        let key = format!("work_item:{}", item_id);
        let bytes = self.state_store.get(&key).await?
            .ok_or(WorkflowError::WorkItemNotFound { item_id })?;
        let item: WorkItem = bincode::deserialize(&bytes)?;
        Ok(item)
    }

    pub async fn save(&self, item: &WorkItem) -> WorkflowResult<()> {
        let key = format!("work_item:{}", item.id);
        let bytes = bincode::serialize(item)?;
        self.state_store.insert(&key, bytes).await?;
        Ok(())
    }

    /// Compare-And-Swap: Only update if version matches (optimistic locking)
    pub async fn update_state_cas(
        &self,
        item_id: WorkItemId,
        new_state: WorkItemState,
        new_user: Option<UserId>,
        expected_version: u32,
    ) -> WorkflowResult<bool> {
        let mut item = self.get(item_id).await?;

        // Check version (CAS)
        if item.version != expected_version {
            return Ok(false); // Version mismatch, someone else modified
        }

        // Update state
        item.state = new_state;
        item.assigned_user_id = new_user;
        item.version += 1;

        if new_state == WorkItemState::Executing && item.started_at.is_none() {
            item.started_at = Some(Utc::now());
        }
        if new_state == WorkItemState::Completed {
            item.completed_at = Some(Utc::now());
        }

        self.save(&item).await?;
        Ok(true)
    }

    pub async fn get_for_user(&self, user_id: UserId, state: Option<WorkItemState>) -> WorkflowResult<Vec<WorkItem>> {
        // Scan all work items (inefficient for large scale, use index in production)
        let prefix = "work_item:";
        let mut items = Vec::new();

        for (key, value) in self.state_store.scan_prefix(prefix).await? {
            let item: WorkItem = bincode::deserialize(&value)?;

            // Filter by user and state
            let matches_user = item.assigned_user_id == Some(user_id)
                || item.state == WorkItemState::Offered; // Offered items visible to all eligible
            let matches_state = state.map(|s| item.state == s).unwrap_or(true);

            if matches_user && matches_state {
                items.push(item);
            }
        }

        // Sort by priority (descending), then enabled_at (ascending)
        items.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then(a.enabled_at.cmp(&b.enabled_at))
        });

        Ok(items)
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_checkout_success() {
        let handler = setup_checkout_handler().await;
        let item_id = create_test_work_item(WorkItemState::Offered).await;
        let user_id = UserId::new();

        // Act
        let result = handler.checkout(item_id, user_id).await;

        // Assert
        assert!(result.is_ok());
        let item = handler.repository.get(item_id).await.unwrap();
        assert_eq!(item.state, WorkItemState::Executing);
        assert_eq!(item.assigned_user_id, Some(user_id));
    }

    #[tokio::test]
    async fn test_checkout_already_executing() {
        let handler = setup_checkout_handler().await;
        let item_id = create_test_work_item(WorkItemState::Executing).await;
        let user_id = UserId::new();

        // Act
        let result = handler.checkout(item_id, user_id).await;

        // Assert
        assert!(matches!(result, Err(WorkflowError::InvalidStateTransition { .. })));
    }

    #[tokio::test]
    async fn test_checkout_concurrent_conflict() {
        let handler = setup_checkout_handler().await;
        let item_id = create_test_work_item(WorkItemState::Offered).await;
        let user1 = UserId::new();
        let user2 = UserId::new();

        // Act: Both users try to checkout simultaneously
        let (result1, result2) = tokio::join!(
            handler.checkout(item_id, user1),
            handler.checkout(item_id, user2),
        );

        // Assert: One succeeds, one fails
        assert!(result1.is_ok() ^ result2.is_ok()); // XOR: exactly one success
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_work_item_lifecycle() {
    let engine = setup_test_engine().await;
    let spec_id = load_test_workflow(&engine).await;
    let case_id = engine.create_case(spec_id, json!({})).await.unwrap();

    engine.start_case(case_id).await.unwrap();

    // Wait for work item to be enabled
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Get work items for user
    let user_id = UserId::new();
    let work_items = engine.get_work_items_for_user(user_id).await.unwrap();
    assert_eq!(work_items.len(), 1);
    let item_id = work_items[0].id;

    // Checkout
    engine.checkout_work_item(item_id, user_id).await.unwrap();

    // Complete
    engine.complete_work_item(item_id, user_id, json!({ "result": "approved" })).await.unwrap();

    // Verify completed
    let item = engine.get_work_item(item_id).await.unwrap();
    assert_eq!(item.state, WorkItemState::Completed);
}
```

---

## Performance Requirements

| Operation | Latency (p50) | Latency (p99) | Throughput |
|-----------|---------------|---------------|------------|
| checkout() | <10ms | <50ms | 1,000/sec |
| checkin() | <10ms | <50ms | 1,000/sec |
| complete() | <20ms | <100ms | 500/sec |
| getWorkItemsForUser() | <50ms | <200ms | 100/sec |

**Hot Path Constraint**: State transitions must be ≤8 ticks

---

## Rollout Plan

### Phase 1: MVP (Sprint 1)
- Checkout/checkin operations
- Basic state machine
- Offered and allocated launch modes
- 80% test coverage

### Phase 2: Complete (Sprint 2)
- All 14 lifecycle operations
- All 5 launch modes
- Privilege management
- Pile-based sharing
- 90% test coverage

### Phase 3: Production (Sprint 3)
- Performance optimization
- Load testing (1,000 concurrent users)
- Security audit
- Documentation
- Production deployment

---

## Success Criteria

**Must Have**:
- [ ] All 14 work item lifecycle operations functional
- [ ] All 5 launch modes working
- [ ] Privilege checking enforced
- [ ] Optimistic locking prevents double-booking
- [ ] OTEL instrumentation complete
- [ ] 90%+ test coverage
- [ ] Performance benchmarks met

**Should Have**:
- [ ] Pile-based work sharing
- [ ] Chain execution
- [ ] Concurrent execution mode
- [ ] REST API fully documented
- [ ] gRPC API implemented

**Nice to Have**:
- [ ] GraphQL API
- [ ] WebSocket notifications
- [ ] Work item search/filtering

---

## References

- ADR-001: Interface B Work Item Lifecycle
- C4 Component Diagram: `docs/architecture/c4-component-work-item-service.puml`
- YAWL Interface B: `vendors/yawl/src/org/yawlfoundation/yawl/engine/interfce/interfaceB/`
- Implementation Gaps: `docs/implementation-gaps.md`
