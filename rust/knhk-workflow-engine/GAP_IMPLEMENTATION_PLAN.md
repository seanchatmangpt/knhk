# Gap Implementation Plan

Based on the gap analysis, this document outlines the implementation plan for closing all identified gaps in the workflow engine.

## 1. Recursive Pattern Execution (Decomposition Nets)

**Gap**: YAWL supports nested subnets (decomposition nets), but current design has flat pattern lookup.

**Implementation**:
- Add `execute_pattern_recursive()` method to `WorkflowEngine`
- Support scoped `PatternExecutionContext` for nested execution
- Handle `next_patterns` in `PatternExecutionResult` recursively
- Maintain execution stack for proper error propagation

**Files**:
- `src/executor/pattern.rs` - Add recursive execution method
- `src/patterns/mod.rs` - Ensure pattern registry supports nested execution

## 2. Enterprise-Scale Concurrency

**Gap**: Current `tokio::RwLock` and `HashMap` are sufficient for low to mid concurrency, not for thousands of parallel cases.

**Implementation**:
- Replace `HashMap` with `DashMap` for concurrent maps (lock-free reads/writes)
- Shard large registries (cases, specs) by consistent hash on ID
- Use `RwLock` only at per-case granularity

**Files**:
- `src/executor/engine.rs` - Replace `HashMap` with `DashMap` for cases/specs
- `src/executor/construction.rs` - Initialize sharded structures
- `src/state/store.rs` - Add sharding support for large datasets

## 3. Pre-binding in ResourceAllocator

**Gap**: Current allocator is reactive, not declarative. Enterprise YAWL uses roles, capabilities, and dynamic binding.

**Implementation**:
- Add `allocate_prebound()` method to `ResourceAllocator`
- Support organizational ontology lookup for pre-binding
- Enable hot-path allocation before case activation

**Files**:
- `src/resource/allocation/allocator.rs` - Add pre-binding methods
- `src/resource/allocation/types.rs` - Add pre-binding types

## 4. Lockchain Receipt Integration

**Gap**: Reflex side (2 ns hot path) and enterprise side (ms–s timescale) lack cross-consistency assurance.

**Implementation**:
- Add commit receipt to lockchain on every state mutation
- Link case receipts with provenance tracker
- Ensure `hash(A) = hash(μ(O))` for audit equivalence

**Files**:
- `src/executor/pattern.rs` - Already partially implemented, complete integration
- `src/executor/provenance.rs` - Link receipts with provenance
- `src/compliance/provenance.rs` - Ensure receipt linking

## 5. Timer Wheel + Durable Buckets

**Gap**: TimerService exists but needs durable buckets for crash safety.

**Implementation**:
- Add durable timer buckets in state store
- Implement timer recovery on restart
- Add secondary index by `next_fire_bucket` for O(1) wheel refill

**Files**:
- `src/services/timer.rs` - Add durable bucket support
- `src/state/store.rs` - Add timer bucket storage
- `src/executor/construction.rs` - Recover timers on startup

## 6. Work Item Lifecycle States and Inbox APIs

**Gap**: WorkItemService exists but needs inbox APIs for human task management.

**Implementation**:
- Add inbox API methods (`get_inbox()`, `claim_work_item()`, `submit_work_item()`)
- Support work item state transitions (Created → Assigned → Claimed → InProgress → Completed)
- Add work item filtering by role/capability

**Files**:
- `src/services/work_items.rs` - Add inbox API methods
- `src/api/rest/server.rs` - Add REST endpoints for inbox (when router is fixed)

## 7. Pattern Dispatch Wiring

**Gap**: Patterns 16/30/31/19/20/27/33/34 need proper dispatch wiring for timers and human tasks.

**Implementation**:
- Wire timer events to pattern 16 (Deferred Choice), 30 (Transient Trigger), 31 (Persistent Trigger)
- Wire work item events to pattern 4 (Exclusive Choice), 6 (Multi-Choice), 33/34 (Partial Joins), 27 (Cancelling Discriminator)
- Wire cancellation events to pattern 19 (Cancel Activity), 20 (Cancel Case)

**Files**:
- `src/executor/events.rs` - Add event dispatch wiring
- `src/executor/task.rs` - Wire task completion to patterns
- `src/services/timer.rs` - Wire timer events to patterns

## 8. Tick-Budget Accounting

**Gap**: No tick-budget accounting for Chatman Constant compliance (≤8 ticks).

**Implementation**:
- Add `rdtsc()`-based cycle counting for hot path operations
- Emit cycle count in metrics
- Assert tick budget in hot path operations
- Store average ticks in metrics for persistence layer validation

**Files**:
- `src/performance/tick_budget.rs` - New module for tick accounting
- `src/executor/pattern.rs` - Add tick counting to pattern execution
- `src/observability/metrics.rs` - Record tick metrics

## 9. Compaction Boundary

**Gap**: Sled compacts lazily; need compaction at fixed tick epochs for reflex compliance.

**Implementation**:
- Add `compact()` method to `StateStore`
- Run compaction at fixed tick epochs
- Ensure no drift (argmin drift(A))

**Files**:
- `src/state/store.rs` - Add compaction methods
- `src/executor/construction.rs` - Schedule periodic compaction

## 10. Dual-Clock Projection

**Gap**: Warm persistence must mirror nanosecond commits to legacy time.

**Implementation**:
- Add background task that replays completed cases to external observers
- Project nanosecond commits to millisecond legacy time
- Bridge nanosecond and human domains

**Files**:
- `src/executor/events.rs` - Add dual-clock projection task
- `src/executor/construction.rs` - Start projection task on engine initialization

## Implementation Order

1. **Phase 1: Core Infrastructure** (Gaps 2, 9, 10)
   - Enterprise-scale concurrency (DashMap, sharding)
   - Compaction boundary
   - Dual-clock projection

2. **Phase 2: Pattern Execution** (Gaps 1, 7, 8)
   - Recursive pattern execution
   - Pattern dispatch wiring
   - Tick-budget accounting

3. **Phase 3: Resource Management** (Gaps 3, 6)
   - Pre-binding in ResourceAllocator
   - Work item lifecycle and inbox APIs

4. **Phase 4: Integration** (Gaps 4, 5)
   - Lockchain receipt integration
   - Timer wheel + durable buckets

## Success Criteria

- All 43 YAWL patterns fully supported
- Enterprise-scale concurrency (thousands of parallel cases)
- Hot path operations ≤8 ticks (Chatman Constant)
- Full audit equivalence (`hash(A) = hash(μ(O))`)
- Crash-safe timer and work item recovery
- Complete pattern dispatch wiring





