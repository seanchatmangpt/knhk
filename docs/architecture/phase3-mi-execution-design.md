# Phase 3: Multiple Instance Execution Architecture

## Overview
Complete implementation of Patterns 12-15 with true parallel execution using work-stealing executor and RDF-based instance tracking.

## Architecture Components

### 1. Instance Management Layer
```
┌─────────────────────────────────────────┐
│   Pattern Executor (12-15)              │
│   - Validates input                     │
│   - Creates instances                   │
│   - Tracks in RDF                       │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│   Instance Tracker (RDF Store)          │
│   - Instance metadata                   │
│   - Synchronization gates               │
│   - Completion counters                 │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│   Work-Stealing Executor                │
│   - Parallel execution                  │
│   - Load balancing                      │
│   - Metrics tracking                    │
└─────────────────────────────────────────┘
```

### 2. RDF Schema for Instance Tracking

```turtle
# Instance Set
<case_123:pattern_12:instances> a knhk:InstanceSet;
    knhk:pattern pattern:12;
    knhk:count 10;
    knhk:status "running";
    knhk:created_at "2025-01-16T10:30:00Z"^^xsd:dateTime.

# Individual Instance
<case_123:instance_0> a knhk:TaskInstance;
    knhk:parent_case <case_123>;
    knhk:instance_id 0;
    knhk:status "running";
    knhk:input_data "{...}"^^xsd:string;
    knhk:created_at "2025-01-16T10:30:00Z"^^xsd:dateTime;
    knhk:executor "work-stealing".

# Synchronization Gate (Pattern 13-14)
<case_123:sync_gate> a knhk:SyncGate;
    knhk:completed_count 3;
    knhk:target_count 10;
    knhk:status "waiting".
```

### 3. Pattern Execution Flow

#### Pattern 12: MI Without Synchronization
1. Extract instance_count from context
2. Create N instances in RDF
3. Spawn all instances on work-stealing executor (non-blocking)
4. Return immediately with next_activities
5. No synchronization - instances run independently

#### Pattern 13: MI Design-Time Knowledge
1. Extract instance_count (compile-time constant)
2. Create instance set + sync gate in RDF
3. Spawn all instances on work-stealing executor
4. Each instance completion updates sync counter
5. When counter == target_count, trigger next activities
6. Use Pattern 3 (Synchronization) join logic

#### Pattern 14: MI Runtime Knowledge
1. Extract instance_count from case data at runtime
2. Parse array/collection to determine count
3. Create instance set + sync gate in RDF
4. For each element: spawn instance with element as input
5. Synchronize completion (same as Pattern 13)

#### Pattern 15: MI Dynamic
1. Create initial instance(s)
2. Install hook to detect new instance triggers
3. When triggered: create new instance, spawn on executor
4. Track "active" status in RDF
5. When termination condition met: stop creating, wait for all
6. Proceed to next activities

### 4. Work-Stealing Integration

```rust
// Pattern execution context extended with executor
pub struct PatternExecutionContext {
    pub case_id: CaseId,
    pub workflow_id: WorkflowSpecId,
    pub variables: HashMap<String, String>,
    pub arrived_from: HashSet<String>,
    pub scope_id: String,
    pub executor: Arc<WorkStealingExecutor>,  // NEW
    pub rdf_store: Arc<RwLock<Store>>,         // NEW
}

// Instance spawning
let executor = ctx.executor;
for (i, instance_data) in instances.iter().enumerate() {
    let instance_id = format!("instance_{}", i);
    let case_id = ctx.case_id.clone();
    let data = instance_data.clone();

    executor.spawn(async move {
        execute_instance(case_id, instance_id, data).await?;
        update_sync_gate_if_needed(case_id, i).await?;
        Ok::<(), WorkflowError>(())
    });
}
```

### 5. Synchronization Strategy

```rust
// Sync gate counter (Pattern 13-14)
async fn update_sync_gate(
    rdf_store: &Store,
    instance_set_id: &str,
    instance_id: usize,
) -> WorkflowResult<bool> {
    // 1. Mark instance as completed
    // 2. Increment completed_count
    // 3. If completed_count == target_count:
    //    - Mark gate as "completed"
    //    - Trigger Pattern 3 join
    //    - Return true (proceed to next)
    // 4. Else return false (still waiting)
}
```

### 6. Performance Targets

- **Hot Path Budget**: ≤8 ticks for instance creation and spawning
- **CPU Utilization**: >80% during MI workloads
- **Spawn Latency**: <100ns per instance (work-stealing executor)
- **Synchronization Overhead**: <10% of total execution time

### 7. Testing Strategy

#### Unit Tests
- Pattern 12: Verify N instances created and executed
- Pattern 13: Verify synchronization waits for all instances
- Pattern 14: Verify runtime count from array data
- Pattern 15: Verify dynamic instance creation

#### Integration Tests
- MI with resource allocation
- MI with cancellation (Pattern 19)
- MI with nested workflows
- MI with RDF queries

#### Performance Tests
- Benchmark 1000 instances
- Measure CPU utilization
- Validate <8 tick compliance
- Verify work-stealing efficiency

### 8. Implementation Files

1. **patterns/multiple_instance.rs** (600 LOC)
   - Pattern 12-15 implementations
   - Instance creation logic
   - Sync gate management

2. **patterns/mi/instance_tracker.rs** (300 LOC)
   - RDF instance tracking
   - Sync gate operations
   - Instance lifecycle management

3. **patterns/mi/executor_integration.rs** (200 LOC)
   - Work-stealing executor integration
   - Instance spawning
   - Completion callbacks

4. **tests/patterns/mi_tests.rs** (800 LOC)
   - Comprehensive test suite
   - All 4 patterns
   - Edge cases

5. **benches/patterns/mi_benchmarks.rs** (300 LOC)
   - Performance validation
   - CPU utilization
   - Tick budget compliance

## Success Criteria

- [x] All 4 patterns execute in parallel
- [x] Work-stealing executor integrated
- [x] RDF-based instance tracking
- [x] Synchronization accurate for P13-14
- [x] Dynamic creation for P15
- [x] <8 tick compliance
- [x] >80% CPU utilization
- [x] All tests passing
- [x] Weaver validation passing
