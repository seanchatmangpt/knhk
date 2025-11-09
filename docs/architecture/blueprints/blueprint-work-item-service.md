# Implementation Blueprint: Work Item Service

**Component:** Work Item Service (Interface B)
**Priority:** P0 (MVP Blocker)
**Estimated Effort:** 3-4 weeks (1 engineer)
**Dependencies:** Core Engine, State Store (Sled)

---

## Purpose and Scope

The Work Item Service manages the complete lifecycle of human tasks in workflows. It implements YAWL's Interface B work item operations, enabling resources (users) to see, claim, execute, and complete work items.

**In Scope:**
- 14 lifecycle operations (create, offer, allocate, start, suspend, resume, complete, fail, rollback, skip, pile, unpile, cancel, reallocate)
- Work item state machine with transition guards
- Queue management (offered, allocated, executing, suspended)
- 5 launch modes (user, auto, external, timed, chained)
- Privilege checking integration
- Sub-tick latency for hot path operations

**Out of Scope (defer to v2.0):**
- Bulk operations (batch allocate, batch suspend)
- Advanced launch modes (external, timed, chained)
- Work item templates
- Custom state machine extensions

---

## Functional Requirements

### FR-1: Work Item Creation
**Actor:** Workflow Engine
**Description:** Create work item from task definition
**Preconditions:** Task exists in workflow specification
**Postconditions:** Work item in Created state

**Flow:**
1. Engine identifies task requiring human execution
2. Call `work_item_service.create(task, case_id, data)`
3. Service creates WorkItem struct with unique ID
4. Service persists to Sled state store
5. Service publishes `WorkItemCreated` event
6. Return WorkItemId to engine

**Edge Cases:**
- Task has no resourcing spec → Use default (system user)
- Case already completed → Reject creation
- Data missing required fields → Validate schema

**Test Scenarios:**
- [x] Create work item with valid task
- [x] Create with missing data (expect validation error)
- [x] Create for completed case (expect rejection)

---

### FR-2: Work Item Offer
**Actor:** Resource Allocator
**Description:** Offer work item to qualified resources
**Preconditions:** Work item in Created state
**Postconditions:** Work item in Offered state, resources notified

**Flow:**
1. Resource Allocator identifies qualified resources
2. Call `work_item_service.offer(item_id, resources)`
3. Service validates state transition (Created → Offered)
4. Service adds work item to each resource's offered queue
5. Service publishes `WorkItemOffered` event (for each resource)
6. Service sends notification to resources (optional)

**Edge Cases:**
- Offer to 0 resources → Auto-allocate to system user
- Offer to 1000+ resources → Enforce limit (max 100)
- Resource no longer available → Remove from offer list

**Test Scenarios:**
- [x] Offer to single resource
- [x] Offer to multiple resources
- [x] Offer to unavailable resource (expect filtered out)
- [x] Offer with 0 resources (expect auto-allocation)

---

### FR-3-14: [Similar structure for remaining 12 operations]

*(Abbreviated for space - full blueprint would detail all 14 operations)*

---

## Non-Functional Requirements

### NFR-1: Performance
- **Sub-Tick Latency:** Create, offer, allocate operations MUST complete in <8 ticks (0.125ms)
- **Throughput:** Support 10,000 concurrent work items
- **Queue Operations:** O(log N) complexity for sorted queues

### NFR-2: Reliability
- **State Consistency:** ACID transactions for state changes
- **Crash Recovery:** Recover to last consistent state from Sled WAL
- **Idempotency:** Repeated calls to same operation are safe

### NFR-3: Security
- **Authorization:** Check resource privileges before state transitions
- **Audit:** Log all state changes to PostgreSQL audit store
- **Data Isolation:** Work items isolated by case (no cross-contamination)

---

## API Specification

### Work Item Service Interface

```rust
pub struct WorkItemService {
    store: Arc<Sled>,
    queues: Arc<QueueManager>,
    state_machine: StateMachine,
    resource_service: Arc<ResourceService>,
    event_bus: Arc<EventBus>,
}

impl WorkItemService {
    /// Create work item from task
    ///
    /// # Arguments
    /// * `task` - Task definition from workflow spec
    /// * `case_id` - Case this work item belongs to
    /// * `data` - Input data for work item
    ///
    /// # Returns
    /// * `WorkItemId` - Unique identifier for created work item
    ///
    /// # Errors
    /// * `InvalidTask` - Task definition invalid
    /// * `CaseNotFound` - Case does not exist
    /// * `DataValidationError` - Input data schema validation failed
    ///
    /// # Performance
    /// * Latency: <8 ticks (sub-tick requirement)
    /// * Complexity: O(1)
    pub async fn create(
        &self,
        task: Task,
        case_id: CaseId,
        data: HashMap<String, String>,
    ) -> Result<WorkItemId>;

    /// Offer work item to resources
    ///
    /// # Arguments
    /// * `item_id` - Work item to offer
    /// * `resources` - List of qualified resources
    ///
    /// # Returns
    /// * `()` - Success
    ///
    /// # Errors
    /// * `WorkItemNotFound` - Work item does not exist
    /// * `InvalidStateTransition` - Work item not in Created state
    /// * `EmptyResourceList` - No resources provided (auto-allocates to system)
    ///
    /// # Performance
    /// * Latency: <8 ticks per resource
    /// * Complexity: O(R) where R = number of resources
    pub async fn offer(
        &self,
        item_id: WorkItemId,
        resources: Vec<ResourceId>,
    ) -> Result<()>;

    /// Allocate work item to resource
    ///
    /// # Arguments
    /// * `item_id` - Work item to allocate
    /// * `resource_id` - Resource claiming work item
    ///
    /// # Returns
    /// * `()` - Success
    ///
    /// # Errors
    /// * `WorkItemNotFound` - Work item does not exist
    /// * `InvalidStateTransition` - Work item not Offered to this resource
    /// * `AlreadyAllocated` - Another resource already claimed (race condition)
    /// * `ResourceUnavailable` - Resource no longer available
    ///
    /// # Performance
    /// * Latency: <8 ticks (sub-tick requirement)
    /// * Complexity: O(1) with optimistic locking
    pub async fn allocate(
        &self,
        item_id: WorkItemId,
        resource_id: ResourceId,
    ) -> Result<()>;

    /// Start work item execution
    ///
    /// # Arguments
    /// * `item_id` - Work item to start
    ///
    /// # Returns
    /// * `WorkItemData` - Current data for execution
    ///
    /// # Errors
    /// * `WorkItemNotFound` - Work item does not exist
    /// * `InvalidStateTransition` - Work item not in Allocated state
    /// * `ResourceNotAuthorized` - Resource lacks can-start privilege
    ///
    /// # Performance
    /// * Latency: <8 ticks
    /// * Complexity: O(1)
    pub async fn start(&self, item_id: WorkItemId) -> Result<WorkItemData>;

    // ... remaining 10 operations (suspend, resume, complete, etc.)
}
```

---

## Data Models

### Work Item

```rust
/// Work item representing a human task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    /// Unique identifier
    pub id: WorkItemId,

    /// Case this work item belongs to
    pub case_id: CaseId,

    /// Task ID from workflow specification
    pub task_id: String,

    /// Current state in lifecycle
    pub state: WorkItemState,

    /// Input/output data
    pub data: HashMap<String, String>,

    /// Resource allocated to this work item
    pub allocated_resource: Option<ResourceId>,

    /// Resources offered this work item
    pub offered_resources: Vec<ResourceId>,

    /// Timestamps
    pub created_at: SystemTime,
    pub offered_at: Option<SystemTime>,
    pub allocated_at: Option<SystemTime>,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,

    /// Launch mode for this work item
    pub launch_mode: LaunchMode,

    /// Saved state for suspension
    pub saved_state: Option<SavedState>,
}

/// Work item states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkItemState {
    Created,
    Offered,
    Allocated,
    Executing,
    Suspended,
    Completed,
    Failed,
    Cancelled,
}

/// Launch modes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LaunchMode {
    /// Resource manually starts (default)
    UserInitiated,

    /// Auto-start on allocation
    AutoInitiated,

    /// External service triggers (v2.0)
    ExternalInitiated,

    /// Timer triggers (v2.0)
    TimeInitiated,

    /// Previous work item completion triggers (v2.0)
    Chained,
}

/// Saved state for suspension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub checkpoint_data: HashMap<String, String>,
    pub suspended_at: SystemTime,
    pub resume_token: String,  // Security token for resumption
}
```

---

## State Machine

### State Transition Diagram

```
Created ──offer──> Offered ──allocate──> Allocated ──start──> Executing
   │                 │                      │                     │
   │                 │                      │                     ├──suspend──> Suspended
   │                 │                      │                     │                │
   │                 │                      │                     │                └──resume──> Executing
   │                 │                      │                     │
   │                 │                      │                     ├──complete──> Completed
   │                 │                      │                     ├──fail──────> Failed
   │                 │                      │                     │
   └──cancel────────┴──cancel──────────────┴──cancel────────────┴──cancel────> Cancelled
```

### Transition Guards

```rust
pub struct StateMachine {
    transitions: HashMap<(WorkItemState, Operation), WorkItemState>,
}

impl StateMachine {
    /// Validate state transition
    pub fn validate_transition(
        &self,
        current: WorkItemState,
        operation: Operation,
    ) -> Result<WorkItemState> {
        let key = (current, operation);
        self.transitions
            .get(&key)
            .cloned()
            .ok_or(Error::InvalidStateTransition { current, operation })
    }
}

/// Operations that trigger state transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    Offer,
    Allocate,
    Start,
    Suspend,
    Resume,
    Complete,
    Fail,
    Cancel,
    Rollback,
    Skip,
    Pile,
    Unpile,
    Reallocate,
}
```

---

## Error Handling Strategy

### Error Types

```rust
#[derive(Debug, Error)]
pub enum WorkItemError {
    #[error("Work item {0} not found")]
    NotFound(WorkItemId),

    #[error("Invalid state transition: {current:?} --{operation:?}--> ???")]
    InvalidStateTransition {
        current: WorkItemState,
        operation: Operation,
    },

    #[error("Work item {0} already allocated to resource {1}")]
    AlreadyAllocated(WorkItemId, ResourceId),

    #[error("Resource {0} not authorized for operation {1}")]
    NotAuthorized(ResourceId, Operation),

    #[error("Data validation failed: {0}")]
    DataValidationError(String),

    #[error("Case {0} not found or completed")]
    CaseNotFound(CaseId),
}
```

### Error Recovery

- **NotFound:** Return 404 to API caller
- **InvalidStateTransition:** Return 409 Conflict with current state
- **AlreadyAllocated:** Return 409 Conflict (race condition)
- **NotAuthorized:** Return 403 Forbidden
- **DataValidationError:** Return 400 Bad Request with schema errors

---

## Testing Strategy

### Unit Tests (90% coverage target)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_work_item_success() {
        // Arrange
        let service = WorkItemService::new_test();
        let task = Task::mock_approval_task();
        let case_id = CaseId::new();
        let data = mock_data();

        // Act
        let result = service.create(task, case_id, data).await;

        // Assert
        assert!(result.is_ok());
        let work_item_id = result.unwrap();
        let work_item = service.get(work_item_id).await.unwrap();
        assert_eq!(work_item.state, WorkItemState::Created);
    }

    #[tokio::test]
    async fn test_allocate_race_condition() {
        // Arrange: Two resources try to allocate same work item
        let service = Arc::new(WorkItemService::new_test());
        let work_item_id = create_and_offer_work_item(&service).await;
        let resource1 = ResourceId::new();
        let resource2 = ResourceId::new();

        // Act: Concurrent allocation attempts
        let service1 = service.clone();
        let service2 = service.clone();
        let handle1 = tokio::spawn(async move {
            service1.allocate(work_item_id, resource1).await
        });
        let handle2 = tokio::spawn(async move {
            service2.allocate(work_item_id, resource2).await
        });

        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        // Assert: One succeeds, one fails with AlreadyAllocated
        assert!(result1.is_ok() ^ result2.is_ok());  // XOR: exactly one succeeds
    }

    // ... more unit tests for all 14 operations
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_work_item_lifecycle_end_to_end() {
    // Full lifecycle: create → offer → allocate → start → complete
    let service = WorkItemService::new_test();
    let resource = ResourceId::new();

    // Create
    let work_item_id = service.create(mock_task(), case_id, data).await.unwrap();

    // Offer
    service.offer(work_item_id, vec![resource]).await.unwrap();

    // Verify in queue
    let offered = service.queues().get_offered_for_resource(resource);
    assert!(offered.contains(&work_item_id));

    // Allocate
    service.allocate(work_item_id, resource).await.unwrap();

    // Start
    let data = service.start(work_item_id).await.unwrap();

    // Complete
    let output = HashMap::from([("approved".to_string(), "true".to_string())]);
    service.complete(work_item_id, output).await.unwrap();

    // Verify final state
    let work_item = service.get(work_item_id).await.unwrap();
    assert_eq!(work_item.state, WorkItemState::Completed);
}
```

### Performance Tests

```rust
#[tokio::test]
async fn test_create_sub_tick_latency() {
    let service = WorkItemService::new_test();
    let task = Task::mock_approval_task();

    // Warm up
    for _ in 0..100 {
        service.create(task.clone(), CaseId::new(), HashMap::new()).await.unwrap();
    }

    // Measure
    let start = knhk_hot::tick();
    service.create(task, CaseId::new(), HashMap::new()).await.unwrap();
    let end = knhk_hot::tick();

    let ticks = end - start;
    assert!(ticks < 8, "create() took {ticks} ticks (requirement: <8)");
}
```

---

## Acceptance Criteria

### Functional
- [x] All 14 lifecycle operations implemented
- [x] State machine validates all transitions
- [x] Queue operations (add, remove, list) working
- [ ] Launch modes: User + Auto (external/timed/chained defer to v2.0)
- [ ] Privilege checking integrated with Resource Service

### Performance
- [ ] Create operation: <8 ticks (0.125ms)
- [ ] Offer operation: <8 ticks per resource
- [ ] Allocate operation: <8 ticks
- [ ] Support 10,000 concurrent work items

### Quality
- [ ] 90%+ test coverage
- [ ] Zero Clippy warnings
- [ ] All property tests pass (state machine invariants)
- [ ] Integration tests pass (full lifecycle)

### Integration
- [ ] REST API endpoints working
- [ ] gRPC API endpoints working
- [ ] Event bus integration (publishes state change events)
- [ ] Audit logging to PostgreSQL

---

## Implementation Phases

### Week 1: Core Data Structures & State Machine
- Define WorkItem, WorkItemState, Operation enums
- Implement StateMachine with transition table
- Unit tests for state machine

### Week 2: Lifecycle Operations (Create, Offer, Allocate, Start, Complete)
- Implement 5 core operations
- Sled persistence
- Unit tests for each operation

### Week 3: Advanced Operations & Queues
- Implement suspend, resume, fail, cancel, rollback, skip, pile, unpile, reallocate
- QueueManager for work item queues
- Integration with Resource Service

### Week 4: Testing & Polish
- Integration tests (full lifecycle)
- Performance tests (sub-tick latency)
- API endpoints (REST + gRPC)
- Documentation

---

**End of Blueprint: Work Item Service**
