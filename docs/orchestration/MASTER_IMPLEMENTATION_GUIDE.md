# Master Implementation Guide
## KNHK Workflow Engine Enterprise Migration

**Date**: 2025-11-08
**Orchestrator**: task-orchestrator agent
**Status**: Production-Ready Documentation
**Target**: Code implementation swarms

---

## Executive Summary

This master guide consolidates findings from:
- **system-architect**: 6 C4 diagrams, 2 comprehensive ADRs, 1 component blueprint
- **code-analyzer**: Implementation status matrix, code quality analysis
- **researcher**: 80/20 feature selection, compliance requirements

**Mission**: Enable code swarms to implement **15 critical features** delivering **80% of enterprise value** in **12 weeks**.

**Current State**: **62% implementation complete**
**Target State**: **85% implementation complete** (enterprise-ready)

---

## Table of Contents

1. [80/20 Feature Selection](#8020-feature-selection)
2. [Architecture Overview](#architecture-overview)
3. [Implementation Status](#implementation-status)
4. [Sprint Roadmap](#sprint-roadmap)
5. [Acceptance Criteria](#acceptance-criteria)
6. [Risk Mitigation](#risk-mitigation)
7. [Success Metrics](#success-metrics)

---

## 80/20 Feature Selection

**Principle**: Implement the **20% of features** that deliver **80% of enterprise value**.

### The Critical 15 Features

| Feature | Usage % | Compliance | Priority | Sprint |
|---------|---------|------------|----------|--------|
| **Work Item Lifecycle** | 88% | SOX, PCI-DSS | 🔴 P0 | 1-2 |
| **3-Phase Allocation** | 82% | SOX, PCI-DSS | 🔴 P0 | 2-3 |
| **Separation of Duties** | 47% | SOX, PCI-DSS | 🔴 P0 | 3 |
| **4-Eyes Principle** | 41% | PCI-DSS | 🔴 P0 | 3 |
| **Parallel Split & Sync** | 76% | - | ✅ Complete | - |
| **Exclusive Choice (XOR)** | 94% | - | ✅ Complete | - |
| **Task Delegation** | 71% | - | 🟡 P1 | 2 |
| **Exception Handling** | 53% | SOX | 🟡 P1 | 4 |
| **Data Mappings** | 88% | - | ✅ Complete | - |
| **Time-Based Triggers** | 65% | - | ✅ Complete | - |
| **Offered Launch Mode** | 59% | - | 🟡 P1 | 2 |
| **XQuery Transformations** | 29% | - | 🟢 P2 | 5 |
| **Resource Calendars** | 24% | - | 🟢 P2 | 6 |
| **Multiple Instance** | 35% | - | 🟢 P2 | 1 |
| **Allocated Launch Mode** | 47% | - | 🟡 P1 | 2 |

**Value Delivery**:
- **Sprints 1-2** (4 weeks): 40% value (work item lifecycle)
- **Sprint 3** (2 weeks): +30% value (compliance: SOD, 4-eyes) = **70% total**
- **Sprint 4** (2 weeks): +10% value (exception handling) = **80% total**
- **Sprints 5-6** (4 weeks): +5% value (XQuery, calendars) = **85% total**

---

## Architecture Overview

### C4 Context Diagram

**External Systems**:
- Identity Provider (SPIFFE/SPIRE) - Zero-trust authentication
- Observability Platform (Grafana/Prometheus) - Monitoring
- Process Mining Tools (ProM, Celonis) - Audit analysis
- External Services (Salesforce, SAP) - Business integration

**Actors**:
- Workflow Author - Designs and deploys specifications
- Workflow Participant - Executes assigned work items
- System Administrator - Monitors health and performance
- Compliance Officer - Audits execution for regulatory compliance

**See**: `/docs/architecture/c4-context-workflow-engine.puml`

---

### C4 Container Diagram

**Core Containers**:
1. **REST API** (Axum) - HTTP/JSON interface
2. **gRPC API** (Tonic) - High-performance RPC
3. **Workflow Engine** (Rust) - Core execution engine
4. **Resource Manager** (Rust) - Allocation with filters/constraints
5. **Work Item Service** (Rust) - Interface B implementation
6. **Exception Service** (Rust) - Worklet/exlet framework
7. **Data Service** (Rust) - XQuery and transformations
8. **State Store** (Sled) - Persistent state
9. **Lockchain** (Git2) - Immutable audit trail
10. **OTEL Exporter** (opentelemetry-rust) - Distributed tracing

**See**: `/docs/architecture/c4-container-workflow-engine.puml`

---

### Critical Components

#### 1. Work Item Service

**Purpose**: Interface B - Human task interaction

**Sub-Components**:
- Lifecycle Manager - State machine (enabled → offered → allocated → executing → completed)
- Checkout Handler - Acquire exclusive locks
- Delegation Handler - Reassign tasks
- Offer Handler - Push to user queues
- Allocation Handler - System assignment
- Pile Manager - Shared work queues
- Privilege Checker - Authorization (7 privileges)

**State Machine**:
```
Enabled → Offered → Executing → Completed
        ↓          ↘
    Allocated   → Executing → Suspended → Executing
                     ↓
                  Cancelled
```

**See**:
- C4 Diagram: `/docs/architecture/c4-component-work-item-service.puml`
- ADR: `/docs/architecture/ADR-001-interface-b-work-item-lifecycle.md`
- Blueprint: `/docs/blueprints/blueprint-work-item-service.md`

---

#### 2. Resource Manager

**Purpose**: 3-Phase resource allocation with filters and constraints

**Sub-Components**:
- Allocation Orchestrator - Coordinates 3 phases
- Offer Phase - Selects eligible participants (filters)
- Allocate Phase - Selects one participant (strategies)
- Start Phase - Determines when to start (modes)
- Filter Engine - 10+ filter types
- Constraint Engine - 8+ constraint types
- Calendar Service - Time-based availability

**3-Phase Flow**:
```
Phase 1 (Offer): Apply filters → Check constraints → Get eligible participants
Phase 2 (Allocate): Apply strategy → Select one participant
Phase 3 (Start): Determine mode → User-initiated or system-initiated
```

**Filters**:
1. CapabilityFilter - Match skills
2. RoleFilter - Job role
3. PositionFilter - Hierarchy level
4. OrgGroupFilter - Team membership
5. LeastQueuedFilter - Workload balancing
6. AvailabilityFilter - Calendar-based
7. (+ 4 more)

**Constraints**:
1. **SeparationOfDutiesConstraint** - No same user for conflicting tasks (SOX mandatory)
2. **FourEyesPrincipleConstraint** - Dual authorization (PCI-DSS mandatory)
3. RetainFamiliarConstraint - Same user for related tasks
4. (+ 5 more)

**See**:
- C4 Diagram: `/docs/architecture/c4-component-resource-manager.puml`
- ADR: `/docs/architecture/ADR-002-resource-allocation-3-phase.md`

---

#### 3. Exception Service

**Purpose**: Worklet and exlet-based exception handling

**Sub-Components**:
- Exception Detector - Identifies anticipated/unanticipated exceptions
- RDR Engine - Ripple Down Rules for worklet selection
- Worklet Selector - Selects appropriate recovery worklet
- Worklet Executor - Executes sub-process
- Exlet Handler - Exception processes
- Compensation Handler - Rollback/compensate strategies

**Exception Types**:
- **Anticipated**: Constraint violations, timeouts, external exceptions
- **Unanticipated**: Resource failures, service unavailability, data corruption

**Compensation Strategies**:
- Compensate (undo)
- Force-complete
- Force-fail
- Restart
- Rollback
- Suspend
- Skip
- Invoke Worklet

**See**: `/docs/architecture/c4-component-exception-service.puml`

---

#### 4. Data Service

**Purpose**: XQuery transformations and data gateway

**Sub-Components**:
- XQuery Processor (Saxon-rs) - Complex transformations
- XPath Evaluator - Expression evaluation
- Data Mapper - Input/output mappings
- Data Gateway - External data sources
- SQL Connector - Database integration
- REST Connector - API integration
- Schema Validator - XML Schema validation

**See**: `/docs/architecture/c4-component-data-service.puml`

---

## Implementation Status

**Overall**: **62% Complete**

### Category Breakdown

| Category | Complete | Partial | Missing | Status |
|----------|----------|---------|---------|--------|
| Core Engine | 15/15 | 0/15 | 0/15 | ✅ 100% |
| Workflow Patterns | 41/43 | 1/43 | 1/43 | ⚠️ 98% |
| API Layer | 12/25 | 8/25 | 5/25 | ⚠️ 48% |
| Resource Management | 8/20 | 6/20 | 6/20 | ⚠️ 40% |
| Work Item Service | 4/20 | 2/20 | 14/20 | 🔴 20% |
| Exception Handling | 4/15 | 2/15 | 9/15 | 🔴 27% |
| Data Service | 7/12 | 2/12 | 3/12 | ⚠️ 58% |
| Integration | 8/18 | 4/18 | 6/18 | ⚠️ 44% |
| Observability | 9/10 | 1/10 | 0/10 | ✅ 90% |

### Critical Gaps (Production Blockers)

#### 1. Work Item Service (20% complete)

**Missing**:
- ALL 14 lifecycle operations (checkout, checkin, delegate, suspend, etc.)
- ALL 5 launch modes (only scaffolded)
- Privilege management (0/7 privileges)
- Pile-based work sharing
- Bulk query operations

**File**: `rust/knhk-workflow-engine/src/services/work_item.rs`
**Issue**: Service exists but operations not implemented

---

#### 2. Resource Management (40% complete)

**Missing**:
- 3-phase allocation framework (only single-phase exists)
- Filters: 0/10 implemented (only basic role check)
- Constraints: 0/8 implemented (critical: SOD, 4-eyes missing)
- Calendar service (no time-based availability)
- Resource repository (in-memory only, needs database)

**File**: `rust/knhk-workflow-engine/src/resource/allocation/`
**Issue**: Basic allocation policies exist, but enterprise features missing

---

#### 3. REST API (48% complete)

**Issue**: Router returns empty `Router::new()` due to LockchainStorage Sync issue

**File**: `rust/knhk-workflow-engine/src/api/rest/server.rs:67`
```rust
fn create_router(engine: Arc<WorkflowEngine>) -> Router {
    // FUTURE: Re-enable routes once LockchainIntegration is Sync
    Router::new()
}
```

**Fix**: Wrap LockchainIntegration in `Arc<RwLock<>>` (1 day)

---

#### 4. Connector Integration (Broken)

**Issue**: Automated tasks fail with connector not found error

**File**: `rust/knhk-workflow-engine/src/executor/task.rs:158`
```rust
TaskType::Automated(connector_name) => {
    // FUTURE: Integrate with connector framework
    return Err(WorkflowError::ConnectorNotFound { name: connector_name });
}
```

**Fix**: Integrate ConnectorIntegration with WorkflowEngine (3 days)

---

#### 5. Multiple Instance Execution (Partial)

**Issue**: MI patterns skip execution with debug message

**File**: `rust/knhk-workflow-engine/src/executor/task.rs:196`
```rust
MultipleInstanceWithoutSynchronization => {
    tracing::debug!("Skipping MI Without Sync pattern execution");
    PatternExecutionResult::CompletedImmediately
}
```

**Fix**: Implement task spawning infrastructure (3 days)

---

### High-Priority Gaps

1. **gRPC API** (0% complete): No tonic service implementation
2. **XQuery Support** (0% complete): No Saxon/XQilla integration
3. **RDR Rule Engine** (0% complete): No rule evaluation
4. **Worklet Execution** (blocked): Circular dependency
5. **Health Checks** (0% complete): All `unimplemented!()`

**See**: `/docs/code-analysis/implementation-status-matrix.md` for complete analysis

---

## Sprint Roadmap

### Sprint 1 (Week 1-2): Work Item Foundation

**Goal**: Enable basic human task interaction

**Features**:
1. Implement LifecycleManager with state machine (2 days)
2. Implement CheckoutHandler (checkout, checkin) (2 days)
3. Implement OfferHandler for offered items (1 day)
4. Implement AllocationHandler for allocated items (2 days)
5. Implement DelegationHandler (2 days)
6. Fix Multiple Instance execution (3 days)

**Deliverables**:
- ✅ Checkout/checkin operations functional
- ✅ Offered and allocated launch modes working
- ✅ Delegation working
- ✅ MI patterns (12-15) fully functional
- ✅ 80%+ test coverage
- ✅ REST API endpoints for work items

**Acceptance Criteria**:
- [ ] Users can checkout work items (exclusive lock acquired)
- [ ] Users can checkin work items (lock released, progress saved)
- [ ] Users can delegate tasks to other users
- [ ] System can offer tasks to eligible users
- [ ] System can allocate tasks to specific users
- [ ] All work item state transitions emit OTEL spans
- [ ] Optimistic locking prevents double-booking

**Value Delivered**: **25% of enterprise value**

---

### Sprint 2 (Week 3-4): Work Item Completion

**Goal**: Complete Interface B implementation

**Features**:
1. Implement SuspendHandler (suspend, resume) (2 days)
2. Implement PrivilegeChecker (all 7 privileges) (2 days)
3. Implement PileManager (pile-based work sharing) (1 day)
4. Implement bulk query operations (2 days)
5. REST API: 10 more work item endpoints (2 days)
6. gRPC API: Implement all service methods (2 days)

**Deliverables**:
- ✅ ALL 14 work item lifecycle operations
- ✅ ALL 5 launch modes
- ✅ Privilege management (7 privileges)
- ✅ Pile-based work sharing
- ✅ Bulk queries (getWorkItemsForUser, etc.)
- ✅ REST and gRPC APIs complete
- ✅ 90%+ test coverage

**Acceptance Criteria**:
- [ ] Users can suspend/resume work items
- [ ] Privilege checks enforced (delegate, suspend-case, skip, etc.)
- [ ] Pile-based queues allow work sharing
- [ ] Bulk queries return all user's work items
- [ ] Chain execution works (auto-start next item)
- [ ] Performance: <200ms p99 latency for all operations
- [ ] Performance: <8 ticks for state transitions (hot path)

**Value Delivered**: **+15% = 40% cumulative**

---

### Sprint 3 (Week 5-6): Resource Allocation & Compliance

**Goal**: Implement 3-phase allocation with SOD and 4-eyes

**Features**:
1. Implement OfferPhase with basic filters (2 days)
2. Implement AllocatePhase with strategies (2 days)
3. Implement StartPhase with modes (1 day)
4. Implement CapabilityFilter (1 day)
5. Implement SeparationOfDutiesConstraint (2 days)
6. Implement FourEyesPrincipleConstraint (1 day)
7. Integration testing (2 days)

**Deliverables**:
- ✅ 3-phase allocation framework operational
- ✅ 5 essential filters (role, capability, least queued, position, org group)
- ✅ 2 compliance constraints (SOD, 4-eyes)
- ✅ Allocation strategies (RoundRobin, Random, ShortestQueue, LeastBusy)
- ✅ 80%+ test coverage
- ✅ SOX/PCI-DSS compliance audit passes

**Acceptance Criteria**:
- [ ] OfferPhase selects eligible participants based on filters
- [ ] AllocatePhase selects one participant using configured strategy
- [ ] StartPhase determines user-initiated vs system-initiated
- [ ] CapabilityFilter validates participant capabilities
- [ ] SeparationOfDutiesConstraint prevents same user from conflicting tasks
- [ ] FourEyesPrincipleConstraint requires dual authorization
- [ ] Performance: <50ms allocation latency (p99)

**Value Delivered**: **+30% = 70% cumulative** (SOX/PCI-DSS compliance achieved)

---

### Sprint 4 (Week 7-8): Exception Handling

**Goal**: Implement worklet-based exception recovery

**Features**:
1. Fix worklet execution (circular dependency) (1 day)
2. Implement ExceptionDetector (2 days)
3. Implement CompensationHandler (2 days)
4. Implement basic RDR rule engine (3 days)
5. Integrate with WorkflowEngine (2 days)

**Deliverables**:
- ✅ Worklet execution functional
- ✅ Exception detection (anticipated + unanticipated)
- ✅ 4 compensation strategies (compensate, force-complete, rollback, suspend)
- ✅ Basic RDR rule evaluation
- ✅ 70%+ test coverage

**Acceptance Criteria**:
- [ ] Worklets can be invoked when exceptions occur
- [ ] Compensation strategies execute correctly
- [ ] RDR rules select appropriate worklet
- [ ] Exception events emit OTEL spans
- [ ] Worklet execution isolated (no impact on parent case on failure)

**Value Delivered**: **+10% = 80% cumulative**

---

### Sprint 5 (Week 9-10): Data Service

**Goal**: XQuery transformations and data gateway

**Features**:
1. Integrate Saxon-rs or XQilla-rs (3 days)
2. Implement XQuery processor (2 days)
3. Implement entity unescaping (1 day)
4. Implement SQL connector (2 days)
5. Implement data gateway (2 days)

**Deliverables**:
- ✅ XQuery transformations functional
- ✅ Entity unescaping (2-level)
- ✅ SQL connector (PostgreSQL, MySQL)
- ✅ REST connector (HTTP/HTTPS)
- ✅ Data gateway with connection pooling
- ✅ 80%+ test coverage

**Acceptance Criteria**:
- [ ] XQuery expressions execute correctly
- [ ] Complex data transformations work
- [ ] SQL queries execute via data gateway
- [ ] REST API calls work via connector
- [ ] Connection pooling limits concurrent connections
- [ ] Performance: <100ms p99 for data transformations

**Value Delivered**: **+3% = 83% cumulative**

---

### Sprint 6 (Week 11-12): Resource Calendars & OpenXES

**Goal**: Time-based availability and process mining

**Features**:
1. Implement CalendarService (shift patterns) (2 days)
2. Implement availability checking (1 day)
3. Integrate with ResourceAllocator (1 day)
4. Implement OpenXES export (2 days)
5. Implement event subscription API (1 day)
6. Performance testing and optimization (3 days)

**Deliverables**:
- ✅ Resource calendars (shifts, holidays)
- ✅ Availability validation in allocation
- ✅ OpenXES XML export
- ✅ Event subscription for external monitoring
- ✅ Performance benchmarks met
- ✅ 90%+ test coverage

**Acceptance Criteria**:
- [ ] Calendar defines working hours (daily/weekly patterns)
- [ ] Holidays and exceptions handled
- [ ] Allocation validates resource availability
- [ ] OpenXES export produces valid XES XML
- [ ] Event listeners receive workflow events
- [ ] Performance: All benchmarks met

**Value Delivered**: **+2% = 85% cumulative**

---

## Acceptance Criteria

### Work Item Lifecycle

**Feature**: checkout()

```gherkin
Scenario: Successful checkout
  Given a work item in "offered" state
  And I am eligible for this work item
  When I checkout the work item
  Then the work item state changes to "executing"
  And the work item is locked to me
  And other users cannot checkout this item
  And the checkout event is logged

Scenario: Checkout already executing item
  Given a work item in "executing" state
  And the work item is locked to another user
  When I attempt to checkout the work item
  Then I receive a "WorkItemAlreadyExecuting" error
  And the work item state remains "executing"
  And no event is logged

Scenario: Checkout ineligible item
  Given a work item in "offered" state
  And I am NOT eligible for this work item
  When I attempt to checkout the work item
  Then I receive a "UserNotEligible" error
  And the work item state remains "offered"
  And no event is logged
```

**Feature**: complete()

```gherkin
Scenario: Successful completion
  Given a work item in "executing" state
  And I am the assigned user
  When I complete the work item with valid data
  Then the work item state changes to "completed"
  And the case data is updated with output
  And the completion event is logged
  And the next task in the workflow is enabled

Scenario: Complete without assignment
  Given a work item in "executing" state
  And I am NOT the assigned user
  When I attempt to complete the work item
  Then I receive a "UserNotAssigned" error
  And the work item state remains "executing"

Scenario: Complete with invalid data
  Given a work item in "executing" state
  And I am the assigned user
  When I complete the work item with invalid data
  Then I receive a "DataValidationError"
  And the work item state remains "executing"
```

**See**: `/docs/orchestration/ACCEPTANCE_CRITERIA_CATALOG.md` for 50+ scenarios

---

### Resource Allocation

**Feature**: Separation of Duties

```gherkin
Scenario: SOD prevents same user
  Given a case with task "create_loan" completed by User A
  And task "approve_loan" has SOD constraint on "create_loan"
  When the system allocates "approve_loan"
  Then User A is NOT in the eligible participants
  And only other users are offered the task

Scenario: SOD allows different user
  Given a case with task "create_loan" completed by User A
  And task "approve_loan" has SOD constraint on "create_loan"
  When User B is allocated to "approve_loan"
  Then the allocation succeeds
  And User B can execute "approve_loan"
```

**Feature**: 4-Eyes Principle

```gherkin
Scenario: 4-Eyes requires 2 approvals
  Given a task with 4-eyes principle enabled
  And User A has approved
  When User A attempts to approve again
  Then the approval is rejected
  And the system requires a different user

Scenario: 4-Eyes completion after 2 approvals
  Given a task with 4-eyes principle enabled
  And User A has approved
  And User B has approved
  When the system checks completion criteria
  Then the task is completed
  And both approvals are logged
```

---

## Risk Mitigation

### Risk Register

| Risk ID | Risk | Probability | Impact | Mitigation | Owner |
|---------|------|-------------|--------|------------|-------|
| R-001 | Circular dependency blocker (worklet executor) | High | High | Refactor to use `Weak<WorkflowEngine>` | backend-dev |
| R-002 | XQuery integration complexity | Medium | High | Prototype with Saxon-rs early, allocate buffer time | backend-dev |
| R-003 | Performance regression | Low | High | Benchmark each feature, validate ≤8 ticks | performance-benchmarker |
| R-004 | Security gaps in authorization | Medium | High | Security audit Sprint 2, implement privilege checking | security-manager |
| R-005 | LockchainStorage Sync issue | High | Medium | Wrap in `Arc<RwLock<>>`, test thread safety | backend-dev |
| R-006 | Test coverage gaps | Medium | Medium | Mandate 80%+ coverage per sprint, CI gates | tdd-london-swarm |
| R-007 | SOD/4-eyes implementation errors | Low | Critical | Comprehensive testing, compliance audit | production-validator |
| R-008 | gRPC API design issues | Medium | Medium | Review YAWL Interface B spec, design review early | system-architect |

**See**: `/docs/orchestration/RISK_REGISTER.md` for complete analysis

---

## Success Metrics

### Feature Completeness

- [ ] 15/15 critical features implemented (80% value)
- [ ] 100% of P0 features implemented
- [ ] 80% of P1 features implemented
- [ ] All 43 workflow patterns fully functional

### Code Quality

- [ ] Zero .unwrap()/.expect() in production paths
- [ ] 80%+ test coverage (minimum)
- [ ] Zero clippy warnings
- [ ] All ADRs documented (architecture decisions)
- [ ] All error handling uses `Result<T, E>`

### Performance

- [ ] Hot path ≤8 ticks (Chatman Constant maintained)
- [ ] API latency <200ms (p99)
- [ ] Allocation latency <50ms (p99)
- [ ] Support 10,000 cases/day throughput
- [ ] Support 1,000 concurrent users

### Enterprise Readiness

- [ ] SOX compliance checklist 100%
- [ ] PCI-DSS compliance checklist 100%
- [ ] GDPR compliance checklist 100%
- [ ] Security audit passed
- [ ] Load testing passed (1,000 concurrent users)
- [ ] Migration guide validated (YAWL → knhk)

### Observability

- [ ] OTEL instrumentation for all operations
- [ ] OpenXES export functional (process mining)
- [ ] Health checks for all integrations
- [ ] Performance dashboards deployed
- [ ] Alert rules configured

**See**: `/docs/orchestration/SUCCESS_METRICS.md` for complete metrics

---

## Documentation Index

### Architecture
- C4 Context Diagram: `/docs/architecture/c4-context-workflow-engine.puml`
- C4 Container Diagram: `/docs/architecture/c4-container-workflow-engine.puml`
- C4 Component (Work Item): `/docs/architecture/c4-component-work-item-service.puml`
- C4 Component (Resource): `/docs/architecture/c4-component-resource-manager.puml`
- C4 Component (Exception): `/docs/architecture/c4-component-exception-service.puml`
- C4 Component (Data): `/docs/architecture/c4-component-data-service.puml`
- ADR-001 (Interface B): `/docs/architecture/ADR-001-interface-b-work-item-lifecycle.md`
- ADR-002 (3-Phase Allocation): `/docs/architecture/ADR-002-resource-allocation-3-phase.md`

### Code Analysis
- Implementation Status Matrix: `/docs/code-analysis/implementation-status-matrix.md`
- Code Quality Report: `/docs/code-analysis/code-quality-report.md`
- Critical Path Gaps: `/docs/code-analysis/critical-path-gaps.md`
- Enterprise Readiness Checklist: `/docs/code-analysis/enterprise-readiness-checklist.md`

### Research
- 80/20 Feature Selection: `/docs/research/80-20-enterprise-feature-selection.md`
- Compliance Requirements: `/docs/research/compliance-requirements.md`
- Industry Use Cases: `/docs/research/industry-use-cases.md`
- Performance Requirements: `/docs/research/performance-requirements.md`

### Blueprints
- Work Item Service: `/docs/blueprints/blueprint-work-item-service.md`

### Orchestration
- Master Implementation Guide: `/docs/orchestration/MASTER_IMPLEMENTATION_GUIDE.md` (this file)
- Sprint Planning Guide: `/docs/orchestration/SPRINT_PLANNING_GUIDE.md`
- Acceptance Criteria Catalog: `/docs/orchestration/ACCEPTANCE_CRITERIA_CATALOG.md`
- Risk Register: `/docs/orchestration/RISK_REGISTER.md`
- Success Metrics: `/docs/orchestration/SUCCESS_METRICS.md`
- Documentation Index: `/docs/orchestration/DOCUMENTATION_INDEX.md`
- Agent Coordination Summary: `/docs/orchestration/AGENT_COORDINATION_SUMMARY.md`

### Existing Documentation
- YAWL Missing Features: `/docs/YAWL_MISSING_FEATURES.md`
- YAWL Parity Report: `/docs/YAWL_PARITY_FINAL_REPORT.md`
- Week 1 Implementation Plan: `/docs/WEEK1_IMPLEMENTATION_PLAN.md`
- Implementation Gaps: `/docs/implementation-gaps.md`

---

## Next Steps for Code Swarms

### Immediate Actions (Day 1)

1. **Read this master guide completely**
2. **Read relevant ADRs and blueprints for assigned sprint**
3. **Set up coordination hooks**:
   ```bash
   npx claude-flow@alpha hooks pre-task --description "Sprint N implementation"
   npx claude-flow@alpha hooks session-restore --session-id "swarm-enterprise-migration"
   ```

### During Implementation

1. **Follow ADR architecture decisions** (non-negotiable)
2. **Reference component blueprints** for implementation details
3. **Use acceptance criteria** for validation
4. **Emit OTEL spans** for all operations
5. **Maintain ≤8 tick hot path** constraint
6. **Achieve 80%+ test coverage** minimum
7. **Store progress in memory**:
   ```bash
   npx claude-flow@alpha hooks post-edit --file "path/to/file.rs" --memory-key "swarm/sprint-N/feature-X"
   npx claude-flow@alpha hooks notify --message "Completed feature X"
   ```

### After Sprint Completion

1. **Run all tests** (`cargo test --workspace`)
2. **Run clippy** (`cargo clippy --workspace -- -D warnings`)
3. **Run benchmarks** (verify performance requirements)
4. **Update acceptance criteria** (mark completed scenarios)
5. **Document any deviations** from ADRs
6. **Store sprint results**:
   ```bash
   npx claude-flow@alpha hooks post-task --task-id "sprint-N"
   npx claude-flow@alpha hooks session-end --export-metrics true
   ```

---

## Critical Success Factors

### 1. Follow the 80/20 Principle

**DO** focus on:
- Work Item Lifecycle (88% usage)
- 3-Phase Allocation (82% usage)
- SOD & 4-Eyes (compliance mandatory)

**DON'T** waste time on:
- Cost Service (<5% usage)
- Custom Forms (frontend concern)
- Document Store (external system)
- Proclet Service (<10% usage)

### 2. Maintain Performance

**Every feature must**:
- Hot path operations ≤8 ticks
- API operations <200ms p99 latency
- Allocation <50ms p99 latency

**Use hot path validator**:
```rust
#[cfg(test)]
mod tests {
    use knhk_workflow_engine::performance::hot_path::validate_ticks;

    #[test]
    fn test_checkout_hot_path() {
        let ticks = measure_ticks(|| {
            checkout_handler.checkout(item_id, user_id).await.unwrap();
        });
        assert!(ticks <= 8, "Hot path violation: {} ticks", ticks);
    }
}
```

### 3. Ensure Compliance

**SOX/PCI-DSS requirements are non-negotiable**:
- Separation of Duties constraint MUST work correctly
- 4-Eyes Principle constraint MUST work correctly
- Audit trails MUST capture all state transitions
- Work item locking MUST prevent double-execution

**Test compliance rigorously**:
```rust
#[tokio::test]
async fn test_sod_prevents_same_user() {
    // User A creates loan
    complete_task("create_loan", user_a).await;

    // User A attempts to approve (should fail)
    let result = allocate_task("approve_loan").await;

    // Assert User A is excluded
    assert!(!result.eligible_users.contains(&user_a));
}
```

### 4. Prioritize Code Quality

**Production code requirements**:
- ✅ All errors use `Result<T, E>`
- ✅ No `.unwrap()` or `.expect()` (use `?` operator)
- ✅ OTEL spans for all operations
- ✅ Comprehensive error messages
- ✅ Zero clippy warnings

**Example**:
```rust
// ❌ WRONG
pub async fn checkout(&self, item_id: WorkItemId, user_id: UserId) {
    let item = self.repository.get(item_id).await.unwrap(); // NEVER unwrap!
    ...
}

// ✅ CORRECT
pub async fn checkout(&self, item_id: WorkItemId, user_id: UserId) -> WorkflowResult<()> {
    let item = self.repository.get(item_id).await?; // Use ? operator
    ...
    Ok(())
}
```

---

## Conclusion

This master guide provides **everything code swarms need** to implement enterprise-grade workflow features:

- ✅ **Architecture** - C4 diagrams and ADRs for design decisions
- ✅ **Implementation Status** - Detailed gap analysis (62% → 85%)
- ✅ **Sprint Roadmap** - 6-sprint plan to 80% value in 12 weeks
- ✅ **Acceptance Criteria** - 50+ Gherkin scenarios for validation
- ✅ **Risk Mitigation** - 8 critical risks with mitigation strategies
- ✅ **Success Metrics** - Measurable goals for feature completeness, quality, performance

**Next Action**: Code swarms should begin **Sprint 1 implementation** (Work Item Lifecycle) using this guide as the single source of truth.

---

**Generated by**: task-orchestrator agent
**Date**: 2025-11-08
**Status**: Production-Ready
**Confidence**: HIGH (consolidates 3 specialized agent outputs)
