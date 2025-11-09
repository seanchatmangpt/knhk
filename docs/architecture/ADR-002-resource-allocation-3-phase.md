# ADR-002: 3-Phase Resource Allocation Implementation

**Status**: Proposed
**Date**: 2025-11-08
**Deciders**: System Architect, Code Analyzer
**Priority**: 🔴 CRITICAL (Tier 1 - Enterprise Essential)

---

## Context and Problem Statement

Current knhk resource allocation is single-phase (basic role matching). YAWL implements sophisticated 3-phase allocation (Offer → Allocate → Start) with 10+ filters and 8+ constraints for enterprise resource planning. Without this, knhk cannot support:
- Separation of Duties (SOD) compliance
- 4-Eyes Principle (dual authorization)
- Workload balancing (shortest queue, least busy)
- Skill-based routing (capability matching)
- Organizational hierarchy constraints

**Current State**:
- ✅ `ResourceAllocator` exists with basic policies (RoundRobin, Random, Priority)
- ✅ `Role`, `Capability`, `Position` types defined
- ❌ NO 3-phase allocation framework
- ❌ NO filters (capability, position, org group, experience)
- ❌ NO constraints (SOD, 4-eyes, retain familiar)
- ❌ NO calendar-based availability

---

## Decision Drivers

1. **Enterprise Compliance**: SOD and 4-eyes are **mandatory** for regulated industries
2. **YAWL Compatibility**: Must support YAWL allocation patterns for migration
3. **Flexibility**: Support custom filters and constraints (plugin architecture)
4. **Performance**: Allocation must complete in <50ms for responsive UI
5. **Scalability**: Must handle 10,000+ participants, 1,000+ concurrent allocations

---

## Considered Options

### Option 1: Extend Current Single-Phase Allocator
```rust
pub enum AllocationPolicy {
    RoundRobin,
    Random,
    Priority,
    // Add more policies here...
    ShortestQueue,
    LeastBusy,
    CapabilityMatch,
}
```

**Pros**:
- Minimal code changes
- Fast to implement

**Cons**:
- ❌ Doesn't support YAWL's 3-phase model
- ❌ Hard to compose filters (how to combine capability + role + queue?)
- ❌ No separation between offer, allocate, start phases
- ❌ Can't express complex constraints (SOD, 4-eyes)

---

### Option 2: 3-Phase Allocation Framework (Recommended)
```rust
pub struct ResourceAllocator {
    offer_phase: Arc<OfferPhase>,
    allocate_phase: Arc<AllocatePhase>,
    start_phase: Arc<StartPhase>,
    filter_engine: Arc<FilterEngine>,
    constraint_engine: Arc<ConstraintEngine>,
    calendar_service: Arc<CalendarService>,
}
```

**Pros**:
- ✅ Direct YAWL compatibility
- ✅ Clear separation of concerns (3 phases)
- ✅ Composable filters and constraints
- ✅ Supports incremental implementation (Phase 1 first, then 2, then 3)
- ✅ Aligns with C4 component diagram

**Cons**:
- More complex architecture
- Requires significant refactoring

---

### Option 3: Rule-Based Allocation Engine
```rust
pub struct AllocationRule {
    condition: Box<dyn Fn(&Task, &Participant) -> bool>,
    action: AllocationAction,
}
```

**Pros**:
- Highly flexible
- Can express complex logic

**Cons**:
- Too flexible (hard to reason about)
- Performance unpredictable
- Doesn't match YAWL model

---

## Decision Outcome

**Chosen Option**: **Option 2 - 3-Phase Allocation Framework**

### Rationale

1. **Enterprise Compliance**: Only option that supports SOD and 4-eyes constraints
2. **YAWL Compatibility**: Direct mapping to YAWL's proven architecture
3. **Incremental Implementation**: Can deliver Sprint 1 (basic offer phase) without full framework
4. **Testability**: Each phase independently testable
5. **Extensibility**: Easy to add new filters and constraints

---

## Architecture

### Phase 1: Offer (Select Eligible Participants)

```rust
pub struct OfferPhase {
    filter_engine: Arc<FilterEngine>,
    constraint_engine: Arc<ConstraintEngine>,
    calendar_service: Arc<CalendarService>,
    resource_repository: Arc<ResourceRepository>,
}

impl OfferPhase {
    #[instrument(skip(self))]
    pub async fn select_eligible_participants(
        &self,
        task: &Task,
        case_context: &CaseContext,
    ) -> WorkflowResult<Vec<ParticipantId>> {
        // 1. Get all participants
        let all_participants = self.resource_repository.get_all_participants().await?;

        // 2. Apply filters (capability, role, position, org group, experience)
        let filtered = self.filter_engine.apply_filters(
            all_participants,
            task,
            case_context,
        ).await?;

        // 3. Check calendar availability
        let available = self.calendar_service.filter_available(
            filtered,
            Utc::now(),
        ).await?;

        // 4. Check constraints (SOD, 4-eyes, retain familiar)
        let eligible = self.constraint_engine.check_constraints(
            available,
            task,
            case_context,
        ).await?;

        Ok(eligible)
    }
}
```

### Phase 2: Allocate (Select One Participant)

```rust
pub struct AllocatePhase {
    allocation_strategies: HashMap<AllocationStrategy, Box<dyn AllocationPolicy>>,
    resource_repository: Arc<ResourceRepository>,
}

pub enum AllocationStrategy {
    RoundRobin,
    Random,
    ShortestQueue,
    LeastBusy,
    FastestCompletion,
    Custom(String),
}

impl AllocatePhase {
    #[instrument(skip(self))]
    pub async fn select_participant(
        &self,
        eligible_participants: Vec<ParticipantId>,
        strategy: AllocationStrategy,
    ) -> WorkflowResult<ParticipantId> {
        let policy = self.allocation_strategies
            .get(&strategy)
            .ok_or(WorkflowError::InvalidAllocationStrategy)?;

        // Get participant workload/history for decision
        let participants_with_metrics = self.get_participant_metrics(
            &eligible_participants
        ).await?;

        // Apply strategy to select one participant
        policy.select(participants_with_metrics).await
    }

    async fn get_participant_metrics(
        &self,
        participants: &[ParticipantId],
    ) -> WorkflowResult<Vec<ParticipantWithMetrics>> {
        let mut result = Vec::new();
        for participant_id in participants {
            let metrics = self.resource_repository.get_metrics(participant_id).await?;
            result.push(ParticipantWithMetrics {
                id: *participant_id,
                queue_depth: metrics.queue_depth,
                active_tasks: metrics.active_tasks,
                avg_completion_time: metrics.avg_completion_time,
            });
        }
        Ok(result)
    }
}
```

### Phase 3: Start (Determine When to Start)

```rust
pub struct StartPhase {
    calendar_service: Arc<CalendarService>,
}

pub enum StartMode {
    UserInitiated,   // User claims from worklist
    SystemInitiated, // Auto-start when allocated
    Concurrent,      // Multiple users, first to complete wins
}

impl StartPhase {
    #[instrument(skip(self))]
    pub async fn determine_start_behavior(
        &self,
        task: &Task,
        participant: ParticipantId,
    ) -> WorkflowResult<StartMode> {
        // Read from task metadata
        let start_mode = task.metadata
            .get("start_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("user_initiated");

        match start_mode {
            "system_initiated" => {
                // Auto-start if participant is available now
                if self.calendar_service.is_available_now(participant).await? {
                    Ok(StartMode::SystemInitiated)
                } else {
                    // Schedule for later
                    Ok(StartMode::UserInitiated)
                }
            }
            "concurrent" => Ok(StartMode::Concurrent),
            _ => Ok(StartMode::UserInitiated),
        }
    }
}
```

---

## Filter Implementation

### Filter Engine Architecture

```rust
pub trait ResourceFilter: Send + Sync {
    fn name(&self) -> &str;
    async fn apply(
        &self,
        participants: Vec<ParticipantId>,
        task: &Task,
        context: &CaseContext,
    ) -> WorkflowResult<Vec<ParticipantId>>;
}

pub struct FilterEngine {
    filters: Vec<Box<dyn ResourceFilter>>,
    resource_repository: Arc<ResourceRepository>,
}

impl FilterEngine {
    pub async fn apply_filters(
        &self,
        mut participants: Vec<ParticipantId>,
        task: &Task,
        context: &CaseContext,
    ) -> WorkflowResult<Vec<ParticipantId>> {
        // Apply each filter sequentially (AND logic)
        for filter in &self.filters {
            participants = filter.apply(participants, task, context).await?;
            if participants.is_empty() {
                tracing::warn!(
                    filter = filter.name(),
                    "Filter eliminated all participants"
                );
                break;
            }
        }
        Ok(participants)
    }
}
```

### Capability Filter Example

```rust
pub struct CapabilityFilter {
    resource_repository: Arc<ResourceRepository>,
}

#[async_trait]
impl ResourceFilter for CapabilityFilter {
    fn name(&self) -> &str {
        "CapabilityFilter"
    }

    async fn apply(
        &self,
        participants: Vec<ParticipantId>,
        task: &Task,
        _context: &CaseContext,
    ) -> WorkflowResult<Vec<ParticipantId>> {
        // Get required capabilities from task
        let required_capabilities: Vec<String> = task.metadata
            .get("required_capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        if required_capabilities.is_empty() {
            return Ok(participants); // No filtering needed
        }

        // Filter participants who have all required capabilities
        let mut result = Vec::new();
        for participant_id in participants {
            let participant = self.resource_repository.get_participant(participant_id).await?;
            let has_all_capabilities = required_capabilities.iter().all(|cap| {
                participant.capabilities.iter().any(|c| c.name == *cap)
            });
            if has_all_capabilities {
                result.push(participant_id);
            }
        }

        Ok(result)
    }
}
```

---

## Constraint Implementation

### Constraint Engine Architecture

```rust
pub trait ResourceConstraint: Send + Sync {
    fn name(&self) -> &str;
    async fn check(
        &self,
        participants: Vec<ParticipantId>,
        task: &Task,
        context: &CaseContext,
    ) -> WorkflowResult<Vec<ParticipantId>>;
}

pub struct ConstraintEngine {
    constraints: Vec<Box<dyn ResourceConstraint>>,
}

impl ConstraintEngine {
    pub async fn check_constraints(
        &self,
        mut participants: Vec<ParticipantId>,
        task: &Task,
        context: &CaseContext,
    ) -> WorkflowResult<Vec<ParticipantId>> {
        // Apply each constraint sequentially (AND logic)
        for constraint in &self.constraints {
            participants = constraint.check(participants, task, context).await?;
            if participants.is_empty() {
                tracing::warn!(
                    constraint = constraint.name(),
                    "Constraint eliminated all participants"
                );
                break;
            }
        }
        Ok(participants)
    }
}
```

### Separation of Duties Constraint Example

```rust
pub struct SeparationOfDutiesConstraint {
    case_repository: Arc<CaseRepository>,
}

#[async_trait]
impl ResourceConstraint for SeparationOfDutiesConstraint {
    fn name(&self) -> &str {
        "SeparationOfDutiesConstraint"
    }

    async fn check(
        &self,
        participants: Vec<ParticipantId>,
        task: &Task,
        context: &CaseContext,
    ) -> WorkflowResult<Vec<ParticipantId>> {
        // Get SOD configuration from task
        let sod_tasks: Vec<String> = task.metadata
            .get("separation_of_duties")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        if sod_tasks.is_empty() {
            return Ok(participants); // No SOD constraint
        }

        // Get case history to see who executed conflicting tasks
        let case_history = self.case_repository.get_history(context.case_id).await?;
        let excluded_users: HashSet<ParticipantId> = case_history
            .iter()
            .filter(|event| {
                matches!(event, StateEvent::TaskCompleted { task_name, user_id, .. }
                    if sod_tasks.contains(task_name))
            })
            .filter_map(|event| {
                if let StateEvent::TaskCompleted { user_id, .. } = event {
                    *user_id
                } else {
                    None
                }
            })
            .collect();

        // Filter out excluded users
        let result = participants.into_iter()
            .filter(|p| !excluded_users.contains(p))
            .collect();

        Ok(result)
    }
}
```

---

## Implementation Plan

### Sprint 1 (Week 1-2): Foundation

1. Implement `OfferPhase` with basic role filter (2 days)
2. Implement `AllocatePhase` with RoundRobin, Random (2 days)
3. Implement `StartPhase` with UserInitiated, SystemInitiated (1 day)

**Deliverables**:
- ✅ 3-phase allocation framework operational
- ✅ Basic role-based filtering working
- ✅ Round-robin and random allocation working
- ✅ 70%+ test coverage

---

### Sprint 2 (Week 3-4): Filters

1. Implement `CapabilityFilter` (1 day)
2. Implement `PositionFilter` (organizational hierarchy) (1 day)
3. Implement `OrgGroupFilter` (team membership) (1 day)
4. Implement `LeastQueuedFilter` (workload-based) (1 day)
5. Implement `AvailabilityFilter` (calendar integration) (2 days)

**Deliverables**:
- ✅ 5 essential filters implemented
- ✅ Filters composable (AND logic)
- ✅ 80%+ test coverage

---

### Sprint 3 (Week 5-6): Constraints

1. Implement `SeparationOfDutiesConstraint` (2 days)
2. Implement `FourEyesPrincipleConstraint` (1 day)
3. Implement `RetainFamiliarConstraint` (1 day)
4. Implement `HistoryConstraint` (1 day)
5. Performance testing and optimization (2 days)

**Deliverables**:
- ✅ 4 compliance constraints implemented
- ✅ SOD and 4-eyes functional (critical for compliance)
- ✅ Performance: <50ms allocation latency
- ✅ 90%+ test coverage

---

## Configuration Example

```toml
# Workflow specification metadata for resource allocation
[task.approve_loan]
# Phase 1: Offer - Select eligible participants
required_roles = ["loan_officer", "senior_loan_officer"]
required_capabilities = ["credit_analysis", "risk_assessment"]
min_experience_years = 5
org_group = "retail_banking"

# Phase 2: Allocate - Select one participant
allocation_strategy = "shortest_queue"

# Phase 3: Start - Determine when to start
start_mode = "user_initiated"

# Constraints
separation_of_duties = ["create_loan_application"] # Can't approve own application
four_eyes_principle = true # Requires approval from 2 users
retain_familiar = "case_completion" # Same user for all tasks in case

[task.verify_documents]
required_roles = ["compliance_officer"]
allocation_strategy = "least_busy"
start_mode = "system_initiated" # Auto-start when allocated
```

---

## Acceptance Criteria

### Must Have
- [ ] OfferPhase selects eligible participants based on role
- [ ] AllocatePhase selects one participant using configured strategy
- [ ] StartPhase determines user-initiated vs system-initiated
- [ ] CapabilityFilter validates participant capabilities
- [ ] SeparationOfDutiesConstraint prevents same user from conflicting tasks
- [ ] FourEyesPrincipleConstraint requires dual authorization
- [ ] All filters and constraints emit OTEL spans
- [ ] Performance: <50ms allocation latency (p99)

### Should Have
- [ ] 10+ filter types implemented
- [ ] 8+ constraint types implemented
- [ ] Custom filter plugin support
- [ ] Custom constraint plugin support
- [ ] Calendar integration for availability
- [ ] Allocation strategy: ShortestQueue, LeastBusy, FastestCompletion

### Could Have
- [ ] Machine learning-based allocation (predict best participant)
- [ ] Cost-based allocation (minimize labor costs)
- [ ] Geographic allocation (prefer local participants)

---

## Success Metrics

- ✅ SOX compliance: Separation of Duties enforced
- ✅ PCI-DSS compliance: 4-Eyes Principle enforced
- ✅ Allocation fairness: Workload balanced within 10% across participants
- ✅ Performance: <50ms allocation latency (p99)
- ✅ Scalability: Support 10,000 participants, 1,000 concurrent allocations

---

## References

- YAWL Resource Patterns: `vendors/yawl/src/org/yawlfoundation/yawl/resourcing/`
- KNHK Resource Module: `rust/knhk-workflow-engine/src/resource/`
- C4 Component Diagram: `docs/architecture/c4-component-resource-manager.puml`
- Implementation Gaps: `docs/implementation-gaps.md`

---

**Decision**: ✅ Approved for implementation
**Next Steps**: Begin Sprint 1 (OfferPhase, AllocatePhase, StartPhase)
