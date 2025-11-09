# Technical Debt Inventory

Comprehensive catalog of technical debt across knhk-workflow-engine, categorized by severity and effort to fix.

---

## Overview

**Total Debt Items**: 47
**Critical (Blocks Production)**: 12
**High (Degrades Quality)**: 18
**Medium (Cleanup Needed)**: 11
**Low (Nice-to-Have)**: 6

**Estimated Total Debt**: ~15 weeks of work

---

## Critical Technical Debt (Blocks Production)

### 1. Automated Task Execution Not Implemented

**Location**: `src/executor/task.rs:156-163`

**Description**:
```rust
} else {
    // Automated task: Execute via connector integration
    // FUTURE: Add connector integration for automated atomic tasks
    return Err(WorkflowError::TaskExecutionFailed(
        format!("Automated atomic task execution requires connector integration...")
    ));
}
```

**Impact**: **BLOCKER** - All service tasks fail immediately

**Root Cause**: Connector integration planned but not implemented

**Effort**: 1-2 weeks

**Suggested Fix**:
```rust
} else {
    // Automated task: Execute via connector integration
    let connector_config = task.connector_config.as_ref()
        .ok_or(WorkflowError::TaskExecutionFailed("Missing connector config"))?;

    let connector = engine.connector_registry
        .get(&connector_config.connector_type)
        .ok_or(WorkflowError::TaskExecutionFailed("Connector not found"))?;

    let input = prepare_input(&case.data, &connector_config.input_mapping)?;
    let output = connector.execute(input).await?;
    merge_output(&mut case.data, output, &connector_config.output_mapping)?;
}
```

**Priority**: P0

---

### 2. Composite Task Execution Not Implemented

**Location**: `src/executor/task.rs:165-172`

**Description**:
```rust
crate::parser::TaskType::Composite => {
    // Composite task: Execute as sub-workflow
    // NOTE: Sub-workflow spec should be stored in task metadata or loaded separately
    return Err(WorkflowError::TaskExecutionFailed(
        format!("Composite task {} requires sub-workflow specification...", task.id)
    ));
}
```

**Impact**: **BLOCKER** - All hierarchical workflows fail

**Effort**: 3-5 days

**Suggested Fix**: Create sub-case, execute, merge results

**Priority**: P0

---

### 3. Multiple Instance Execution Skipped

**Location**: `src/executor/task.rs:173-206`

**Description**:
```rust
crate::parser::TaskType::MultipleInstance => {
    // ... validates instance count ...
    // For now, we just validate the instance count but don't execute
    tracing::debug!(
        "Multiple instance task {} requires {} instances (execution skipped - requires task spawning)",
        task.id, instance_count
    );
}
```

**Impact**: **BLOCKER** - All parallel workflows fail

**Effort**: 1-2 weeks

**Priority**: P0

---

### 4. No Pattern Dispatch After Task Completion

**Location**: `src/executor/task.rs:116-155`

**Description**: Polling loop detects work item completion, but doesn't trigger next task

**Impact**: **BLOCKER** - Workflows stall after first task

**Root Cause**: No event system for task completion, no dispatcher

**Effort**: 1 week

**Priority**: P0

---

### 5. Production Code Has 90 .unwrap() Calls

**Location**: Multiple files, primarily `src/cluster/balancer.rs`

**Description**:
```rust
// cluster/balancer.rs:57, 63, 116, 130, 138 (13 total)
let mut backends = self.backends.lock().unwrap();  // ← PANIC RISK!
let mut index = self.current_index.lock().unwrap();  // ← PANIC RISK!
```

**Impact**: **HIGH** - Mutex poisoning can crash entire application

**Effort**: 2-3 days to audit and fix all

**Suggested Fix**:
```rust
// Option 1: Expect with context
let mut backends = self.backends.lock()
    .expect("Backend mutex poisoned - cannot recover");

// Option 2: Handle poisoning
let backends = match self.backends.lock() {
    Ok(guard) => guard,
    Err(poisoned) => {
        tracing::error!("Backend mutex poisoned, recovering");
        poisoned.into_inner()  // Recover data
    }
};
```

**Priority**: P0

---

### 6. No HTTP/REST Connector

**Location**: `../knhk-connectors/src/` (missing file)

**Description**: Only Kafka and Salesforce connectors exist

**Impact**: **BLOCKER** - Cannot call external REST APIs (90% of integrations)

**Effort**: 1 week

**Priority**: P0

---

### 7. LockchainStorage Not Sync

**Location**: Mentioned in context (file not read)

**Description**: LockchainStorage doesn't implement Sync trait

**Impact**: Blocks usage in async contexts, prevents REST API integration

**Effort**: Medium (depends on lockchain implementation)

**Priority**: P0

---

### 8. No Authentication on REST API

**Location**: `src/api/rest/handlers.rs` (all endpoints)

**Description**: Zero authentication or authorization checks

**Impact**: **SECURITY BLOCKER** - Anyone can access/modify workflows

**Effort**: 1 week for JWT auth

**Priority**: P0

---

### 9. Work Items Not Persisted

**Location**: `src/services/work_items.rs:60-72`

**Description**:
```rust
pub struct WorkItemService {
    /// Work items by ID (IN-MEMORY ONLY!)
    work_items: Arc<RwLock<HashMap<String, WorkItem>>>,
    case_items: Arc<RwLock<HashMap<String, Vec<String>>>>,
}
```

**Impact**: **HIGH** - All work item data lost on restart

**Effort**: 3-5 days to add Sled persistence

**Priority**: P0

---

### 10. Worklet Circular Dependency

**Location**: `src/worklets/mod.rs:351`

**Description**:
```rust
pub async fn execute_worklet(
    &self,
    worklet_id: WorkletId,
    context: PatternExecutionContext,
    engine: &crate::executor::WorkflowEngine,  // ← Circular!
) -> WorkflowResult<PatternExecutionResult> {
```

**Impact**: **MEDIUM** - Blocks modular testing, increases coupling

**Effort**: 3-5 days to refactor with trait abstraction

**Priority**: P1

---

### 11. Case History Returns Placeholder

**Location**: `src/api/rest/handlers.rs:92-124`

**Description**:
```rust
pub async fn get_case_history(...) -> Result<Json<CaseHistoryResponse>, StatusCode> {
    // NOTE: Case history retrieval requires StateManager integration
    // For now, return a placeholder response indicating the feature exists
    let entries: Vec<CaseHistoryEntry> = vec![
        CaseHistoryEntry {
            timestamp: chrono::Utc::now(),
            event_type: "case_created".to_string(),
            data: serde_json::json!({
                "note": "Case history requires StateManager integration"
            }),
        }
    ];
    Ok(Json(CaseHistoryResponse { entries }))
}
```

**Impact**: **MEDIUM** - Cannot view audit trail

**Effort**: 2-3 days to integrate StateManager

**Priority**: P1

---

### 12. Polling Loop in Async Task

**Location**: `src/executor/task.rs:116-155`

**Description**:
```rust
loop {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;  // ← Inefficient!
    if let Some(work_item) = engine.work_item_service.get_work_item(&work_item_id).await {
        match work_item.state {
            WorkItemState::Completed => { break; }
            ...
        }
    }
}
```

**Impact**: **MEDIUM** - Wastes CPU, adds latency (100ms delay)

**Effort**: 1 day to replace with tokio::sync::Notify

**Priority**: P1

---

## High Priority Technical Debt (Degrades Quality)

### 13. No Work Item Lifecycle Operations

**Location**: `src/services/work_items.rs`

**Missing Operations**:
- `start_work_item()` - mark as started
- `suspend_work_item()` - pause work
- `resume_work_item()` - resume paused work
- `reoffer_work_item()` - return to queue
- `reallocate_work_item()` - reassign
- `delegate_work_item()` - delegate to another user
- `skip_work_item()` - skip optional task
- `pile_work_item()` - add to pile

**Impact**: Limited YAWL compatibility, poor UX

**Effort**: 1 week for all operations

**Priority**: P1

---

### 14. No 3-Phase Work Distribution

**Location**: `src/services/work_items.rs`

**Description**: Only 2 states (Created/Assigned), missing Offered/Allocated/Started

**Impact**: Cannot offer work to multiple users, no user choice

**Effort**: 1-2 weeks

**Priority**: P1

---

### 15. No Filter Engine

**Location**: `src/resource/allocation/` (missing implementation)

**Description**: Cannot filter resources by availability, capabilities, workload

**Impact**: Poor resource allocation, inefficient distribution

**Effort**: 1-2 weeks

**Priority**: P1

---

### 16. No Privilege System

**Location**: `src/security/` (missing entirely)

**Description**: No privilege checking (can user claim, can user complete, etc.)

**Impact**: No authorization, security risk

**Effort**: 1 week

**Priority**: P1

---

### 17. Generic Error Responses from API

**Location**: `src/api/rest/handlers.rs` (all endpoints)

**Description**:
```rust
.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;  // ← Loses context!
```

**Impact**: Hard to debug, poor developer experience

**Effort**: 3-5 days to add detailed error responses

**Priority**: P1

---

### 18. No Work Item REST API Endpoints

**Location**: `src/api/rest/` (missing entirely)

**Missing Endpoints**:
- `GET /api/v1/work-items/{id}`
- `GET /api/v1/work-items?resource_id=X`
- `POST /api/v1/work-items/{id}/claim`
- `POST /api/v1/work-items/{id}/start`
- `POST /api/v1/work-items/{id}/complete`
- etc.

**Impact**: Cannot build user interfaces

**Effort**: 3-5 days

**Priority**: P0 (reclassified from P1)

---

### 19. No YAWL XML Parser

**Location**: `src/parser/` (only Turtle parser)

**Description**: Most YAWL workflows are in XML format, cannot import them

**Impact**: Cannot migrate existing YAWL workflows

**Effort**: 1-2 weeks

**Priority**: P1

---

### 20. No Integration Tests for Patterns

**Location**: `src/patterns/` (tests exist per-pattern, but no full workflows)

**Description**: Patterns tested in isolation, not in actual workflow execution

**Impact**: Pattern execution in real workflows not validated

**Effort**: 1 week

**Priority**: P1

---

### 21. No Circuit Breakers for External Calls

**Location**: `../knhk-connectors/` (missing)

**Description**: Connector failures cascade to entire system

**Impact**: One slow/failing API brings down whole system

**Effort**: 1 week to add circuit breaker pattern

**Priority**: P1

---

### 22. No Rate Limiting

**Location**: `src/api/rest/` (missing middleware)

**Description**: API has no rate limiting

**Impact**: DoS vulnerability

**Effort**: 1 day to add rate limiting middleware

**Priority**: P1

---

### 23. No Backup/Recovery

**Location**: `src/state/` (missing backup.rs)

**Description**: No automated backups, no recovery procedures

**Impact**: Data loss risk

**Effort**: 3-5 days

**Priority**: P1

---

### 24. No Exlet Support (External Worklets)

**Location**: `src/worklets/` (missing)

**Description**: Cannot call external services as worklets

**Impact**: Limited exception handling flexibility

**Effort**: 1-2 weeks

**Priority**: P2

---

### 25. No Timeout for Human Tasks

**Location**: `src/executor/task.rs` (timer not integrated)

**Description**: Human tasks never timeout, workflows can stall indefinitely

**Impact**: Workflows can hang forever

**Effort**: 3-5 days to integrate timer service

**Priority**: P1

---

### 26. Error Handling Loses Context

**Location**: Multiple locations

**Description**: Errors propagated without context (which case? which task?)

**Impact**: Hard to debug production issues

**Effort**: 1 week to add context to all errors

**Priority**: P1

---

### 27. No Event Sourcing Replay

**Location**: `src/state/manager.rs` (missing implementation)

**Description**: Cannot reconstruct state from events

**Impact**: Cannot debug state issues, cannot do point-in-time recovery

**Effort**: 3-5 days

**Priority**: P1

---

### 28. No Caching for Hot Cases

**Location**: `src/state/` (missing cache.rs)

**Description**: Every case access hits Sled database

**Impact**: Slow performance for active workflows

**Effort**: 2-3 days

**Priority**: P1

---

### 29. No BPMN Parser

**Location**: `src/parser/` (missing)

**Description**: Cannot import BPMN workflows (common in enterprise)

**Impact**: Limited migration path for BPMN users

**Effort**: 2-3 weeks (BPMN is complex)

**Priority**: P2

---

### 30. No Complex Queries on Cases

**Location**: `src/state/` (missing query engine)

**Description**: Cannot query cases by state, date range, workflow ID efficiently

**Impact**: Poor reporting and monitoring

**Effort**: 1 week

**Priority**: P2

---

## Medium Priority Technical Debt (Cleanup Needed)

### 31. Inconsistent Error Handling

**Location**: Throughout codebase

**Description**: Some functions use `?`, some use `map_err`, some return errors directly

**Impact**: Code inconsistency

**Effort**: 2-3 days to standardize

**Priority**: P2

---

### 32. Sparse Documentation

**Location**: Many function docs missing

**Description**: Only ~60% of public functions documented

**Impact**: Onboarding difficulty

**Effort**: 1 week to document all public APIs

**Priority**: P2

---

### 33. No Performance Regression Tests

**Location**: `tests/` (missing)

**Description**: No tests to detect performance degradation

**Impact**: Performance can regress unnoticed

**Effort**: 3-5 days

**Priority**: P2

---

### 34. No Load Tests

**Location**: `tests/` (missing)

**Description**: Haven't tested with 100+ concurrent workflows

**Impact**: Unknown scalability limits

**Effort**: 3-5 days

**Priority**: P2

---

### 35. Mutex Unwraps Throughout Codebase

**Location**: `src/cluster/balancer.rs` primarily, also in tests

**Description**: 90 `.unwrap()` calls, mostly on mutexes

**Impact**: Panic risk (already covered in Critical #5)

**Effort**: 2-3 days

**Priority**: P0 (moved to Critical)

---

### 36. No Correlation IDs in Logs

**Location**: Logging throughout codebase

**Description**: Cannot trace request across logs

**Impact**: Hard to debug distributed workflows

**Effort**: 2-3 days to add correlation ID to all logs

**Priority**: P2

---

### 37. No Metrics Endpoint

**Location**: `src/api/rest/` (missing /metrics)

**Description**: No Prometheus metrics exposed

**Impact**: Cannot monitor system

**Effort**: 1 day

**Priority**: P1 (reclassified)

---

### 38. Large Function: execute_task_with_allocation()

**Location**: `src/executor/task.rs:38-236` (197 lines!)

**Description**: One function handles all task types, very complex

**Impact**: Hard to maintain, high cyclomatic complexity

**Effort**: 2-3 days to refactor into smaller functions

**Priority**: P2

---

### 39. Complex Rule Evaluation

**Location**: `src/worklets/mod.rs:186-300` (115 lines)

**Description**: evaluate_rule() handles many expression types in one function

**Impact**: Hard to extend, high complexity

**Effort**: 2-3 days to refactor

**Priority**: P2

---

### 40. No Property-Based Testing

**Location**: `tests/` (missing)

**Description**: No QuickCheck/proptest for fuzzing

**Impact**: Edge cases not tested

**Effort**: 1 week

**Priority**: P3

---

### 41. No Chaos Testing

**Location**: `tests/` (missing)

**Description**: No failure injection tests (kill DB, kill process, network partition)

**Impact**: Recovery mechanisms not validated

**Effort**: 1 week

**Priority**: P2

---

## Low Priority Technical Debt (Nice-to-Have)

### 42. Inconsistent Module Structure

**Location**: Throughout codebase

**Description**: Some modules use `mod.rs` with re-exports, others don't

**Impact**: Minor inconsistency

**Effort**: 1 day

**Priority**: P3

---

### 43. Comment Style Varies

**Location**: Throughout codebase

**Description**: Mix of `//!`, `///`, `//` comments

**Impact**: Cosmetic

**Effort**: 1 day

**Priority**: P3

---

### 44. Some Tests in Production Code

**Location**: `src/worklets/mod.rs:411-502`

**Description**: Tests at end of file instead of separate tests/ module

**Impact**: Minor organization issue

**Effort**: 1 hour

**Priority**: P3

---

### 45. Outdated Dependencies

**Location**: `Cargo.toml`

**Description**: tokio 1.35 (latest is 1.40+), tonic 0.10 (latest 0.12+)

**Impact**: Missing bug fixes and features

**Effort**: 1-2 days (test for regressions)

**Priority**: P2

---

### 46. No Examples Directory

**Location**: Missing `/examples`

**Description**: No example usage code

**Impact**: Onboarding difficulty

**Effort**: 2-3 days to create examples

**Priority**: P3

---

### 47. No Architecture Decision Records (ADRs)

**Location**: Missing `/docs/architecture`

**Description**: No record of major design decisions

**Impact**: Context lost over time

**Effort**: 1 week to document past decisions

**Priority**: P3

---

## Debt Paydown Strategy

### Phase 1: Critical Debt (4-6 weeks)

Pay down all P0 items before production deployment:

1. Automated task execution (1-2 weeks)
2. Composite task execution (3-5 days)
3. MI execution (1-2 weeks)
4. Pattern dispatch (1 week)
5. Work item APIs (3-5 days)
6. Fix .unwrap() calls (2-3 days)
7. HTTP connector (1 week)
8. JWT authentication (1 week)
9. Persist work items (3-5 days)

**Total**: ~6 weeks for 2 developers (parallel work possible)

### Phase 2: High Priority Debt (4-6 weeks)

Pay down P1 items for enterprise readiness:

1. Work item lifecycle (1 week)
2. 3-phase distribution (1-2 weeks)
3. Filter engine (1-2 weeks)
4. Circuit breakers (1 week)
5. Backup/recovery (3-5 days)
6. Timeout handling (3-5 days)
7. Error context (1 week)
8. Metrics endpoint (1 day)
9. YAWL XML parser (1-2 weeks)

**Total**: ~6 weeks

### Phase 3: Medium/Low Priority Debt (Ongoing)

Pay down incrementally over time:

- Documentation improvements
- Code consistency
- Performance tests
- Load tests
- Examples
- Refactoring

**Total**: ~3 weeks (can be spread over months)

---

## Debt Metrics

**Current Debt Load**: ~15 weeks of work

**Interest Rate** (cost of carrying debt):
- **Critical**: 100% (blocks production)
- **High**: 50% (degrades quality)
- **Medium**: 10% (minor impact)
- **Low**: 1% (cosmetic)

**Paydown Priority**:
1. Pay critical debt first (highest ROI)
2. Pay high priority debt next (quality improvements)
3. Pay medium/low debt opportunistically (when touching code)

**Prevent New Debt**:
- Code review checklist (no .unwrap(), proper error handling)
- Pre-commit hooks (run clippy, fmt)
- Test coverage requirements (80% minimum)
- Documentation requirements (all public APIs)
- Architecture review for major changes
