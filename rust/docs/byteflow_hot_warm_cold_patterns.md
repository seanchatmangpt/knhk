# ByteFlow Hot/Warm/Cold Path Architecture

## Overview

ByteFlow implements a sophisticated **multi-tier performance architecture** where work is classified and routed based on latency requirements and execution budgets. Understanding these patterns is critical for integrating KNHK with ByteFlow's orchestration layer.

---

## Performance Tiers

### 🔥 Hot Path (≤8 ticks)

**Target Latency**: ≤8 CPU ticks (~2-4 nanoseconds on modern hardware)

**Characteristics**:
- **Zero heap allocation** - Stack-only execution
- **Cache-resident** - 64-byte aligned, L1 cache optimized
- **Lock-free** - SPSC ring buffers for communication
- **Branch-free** - Branchless dispatch and validation
- **SIMD-optimized** - Vectorized operations where possible

**What Runs on Hot Path**:
- ✅ Boolean reflexes (`AskSp`, `AskOp`)
- ✅ Count operations (`CountSpEq`, `CountOpLe`)
- ✅ Fixed-template emit (`Construct8`, ≤8 triples)
- ✅ Crystal sealing (BLAKE3 ≤1 tick)
- ✅ Dispatch table lookup (O(1) MPHF)
- ✅ Beat tick advancement (atomic increment)

**Hot Path Constraints** (Doctrine of 8):
```erlang
% From bf_types.hrl
-define(MAX_HOT_PATH_TICKS, 8).
-define(MAX_WORKFLOW_HOPS, 8).
-define(MAX_RUN_LENGTH, 8).    % SPARQL BGP run length
```

**Hot Path Entry Requirements**:
1. **Budget > 0** - Credit/rate limit check
2. **Admitted** - Passed Θ gate
3. **Cache-resident** - Data in L1/L2
4. **Tick budget available** - Current tick ≤ 7

---

### 🌡️ Warm Path (≤500ms)

**Target Latency**: ≤500 milliseconds

**Characteristics**:
- **Heap allocation allowed** - Complex data structures OK
- **Async operations** - Non-blocking I/O, futures
- **Query execution** - SPARQL via Oxigraph
- **Pattern matching** - Graph traversal and rewriting
- **Moderate concurrency** - Lightweight processes (Erlang) or threads (Rust)

**What Runs on Warm Path**:
- ✅ SPARQL query execution (complex BGPs)
- ✅ Pattern library lookups (43 canonical patterns)
- ✅ Graph validation (SHACL, PB-congruence)
- ✅ Workflow orchestration logic
- ✅ State machine transitions
- ✅ Moderate data transformations

**Warm Path Entry**:
- Work **parked** from hot path if:
  - Tick count exceeds 8
  - Budget exhausted
  - Requires I/O or complex computation
  - Pattern complexity > basic threshold

**Example from `bf_orchestration_server.erl`**:
```erlang
%% Workflow submission - warm path
submit_workflow(WorkflowCrystal) ->
    gen_server:call(?SERVER, {submit_workflow, WorkflowCrystal}).

%% Workflow execution in 16-worker pool
-define(EXECUTOR_POOL_SIZE, 16).
```

---

### 🧊 Cold Path (Batched/Async)

**Target Latency**: Seconds to minutes (no real-time constraint)

**Characteristics**:
- **Batch processing** - Amortize overhead across many items
- **Heavy I/O** - Database writes, network calls, disk operations
- **Complex analytics** - Machine learning, optimization
- **Background jobs** - Cleanup, compaction, archival
- **External integrations** - API calls, webhooks

**What Runs on Cold Path**:
- ✅ Receipt chain Merkle proof generation
- ✅ Long-term storage (persistence)
- ✅ Analytics and reporting
- ✅ External system notifications
- ✅ Backup and archival
- ✅ Capacity planning calculations

**Cold Path Entry**:
- Work **deferred** to background if:
  - No deadline constraint
  - Can be batched with similar operations
  - Requires external resource coordination
  - Result not immediately needed

---

## Admission Pipeline (Θ Predicate)

**CRITICAL**: Work MUST pass admission before entering hot path.

### 4-Stage Pipeline (from `bf_admission_gate.erl`)

```
MissionCrystal
    ↓
┌─────────────────────────────────┐
│ Stage 1: SHACL Validation       │  <10ms
│ - Shape constraint checking     │
│ - Required fields validation    │
│ - Type conformance              │
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│ Stage 2: PB-Congruence          │  <15ms
│ - Pattern byte validation (1-43)│
│ - Graph signature matching      │
│ - Hop count check (≤8 hops)     │
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│ Stage 3: PQC Verification       │  <20ms
│ - Post-quantum crypto proof     │
│ - Signature validation          │
│ - Certificate chain check       │
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│ Stage 4: Θ Decision             │  <5ms
│ - Binary admit/reject           │
│ - Budget allocation             │
│ - Priority assignment           │
└─────────────────────────────────┘
    ↓
  Θ ∈ {0, 1}
    ↓
[Admitted] → Hot Path
[Rejected] → Zero-tick rejection (<1μs)
```

**Performance Targets**:
```erlang
-define(MAX_ADMISSION_TIME_MS, 50).           % Total pipeline
-define(ZERO_TICK_REJECTION_TIME_US, 1).     % Obvious rejects
```

**Zero-Tick Rejection Criteria** (immediate, no stages):
- ❌ Budget = 0 (no credits)
- ❌ Pattern byte invalid (not in 1-43 range)
- ❌ Malformed structure (missing fields)
- ❌ Known blacklisted source
- ❌ Rate limit exceeded

**Fast Path Optimization** (from `bf_admission_gate.erl`):
```erlang
%% Create caching tables for high-throughput processing
ets:new(rejection_cache, [named_table, public, set]),
ets:new(validation_cache, [named_table, public, set]),
```

---

## Hot Path Optimization Patterns

### Pattern 1: Cache-Line Alignment

**From ByteCore ABIs**:
```c
// envelope.h - 64-byte ingress envelope
typedef struct __attribute__((packed,aligned(64))) {
  uint16_t magic;      // 0xBE64
  uint8_t  ver;        // ABI version
  uint8_t  pb;         // Pattern Byte (1-43)
  uint16_t budget;     // Credits
  uint8_t  flags;      // Processing flags
  uint8_t  priority;   // 0-7
  // ... total 64 bytes
} env64_t;
```

**Why 64 bytes?**
- Matches CPU cache line size (x86_64, ARM64)
- Single cache fetch for entire envelope
- Prevents false sharing in concurrent access
- Enables atomic updates without locks

**Rust Implementation**:
```rust
#[repr(C, align(64))]
pub struct Envelope64 {
    magic: u16,
    version: u8,
    pattern_byte: u8,  // 1-43 canonical patterns
    budget: u16,       // Credits/rate limit
    flags: u8,
    priority: u8,      // 0-7
    // ... 56 more bytes
}
```

### Pattern 2: SPSC Ring Buffers

**Lock-Free Communication** (ByteFlow ↔ ByteActor):

```c
// ring_spsc.h
typedef struct __attribute__((aligned(64))) {
    _Atomic uint64_t head;  // Producer index (Erlang)
    uint8_t _pad1[56];      // Prevent false sharing
    _Atomic uint64_t tail;  // Consumer index (C core)
    uint8_t _pad2[56];
    void* buffer[RING_SIZE];
} ring_spsc_t;
```

**Key Properties**:
- **Single producer, single consumer** - No locks needed
- **Atomic operations** - `Acquire/Release` memory ordering
- **Padded indices** - Prevent false sharing (64-byte separation)
- **Power-of-2 size** - Fast modulo via bit mask
- **Sub-microsecond latency** - No syscalls, no contention

**Usage in ByteFlow** (from research report):
```
Erlang (Orchestrator)          C Core (ByteActor)
       ↓                              ↑
[Envelope] → SPSC Ingress Ring → [Kernel Execution]
       ↑                              ↓
[Receipt] ← SPSC Egress Ring ← [Result Crystal]
```

### Pattern 3: Constant-Time Dispatch

**O(1) Kernel Lookup** via MPHF (Minimal Perfect Hash Function):

```c
// dispatch.h
typedef uint32_t (*kernel_fn)(ctx_t* ctx, ir_t* ir);

// MPHF-based dispatch (≤5 cycles)
static inline kernel_fn dispatch(uint32_t op) {
    uint32_t idx = mphf_hash(op);  // Perfect hash
    return kernel_table[idx];       // Direct lookup
}
```

**No branches, no conditionals** - Single indirect call.

### Pattern 4: Beat Scheduling (8-Tick Epoch)

**From `bf_hot_path_optimizer.erl` and knhk-hot**:

```rust
// 8-tick epoch system
pub const TICK_BUDGET: u32 = 8;

// Beat cycle coordination
pub fn knhk_beat_next() -> u64;        // Advance cycle
pub fn knhk_beat_tick(cycle: u64) -> u64;  // Extract tick (0..7)
pub fn knhk_beat_pulse(cycle: u64) -> u64; // Pulse on tick==0
```

**Deterministic Scheduling**:
- Work partitioned into 8-tick epochs
- Each tick slot has budget for operations
- Pulse (tick 0) triggers synchronization
- No time-based scheduling (deterministic)

---

## Task Escalation Patterns

### Pattern A: Budget Exhaustion → Warm Path

**Trigger**: Tick count exceeds 8 OR budget depleted

**From `bf_orchestration_server.erl`**:
```erlang
retry_budget => 3,  % Allow 3 retries before escalation
```

**Escalation Flow**:
```
Hot Path (tick 8/8, budget low)
    ↓
[Park Work] → Warm Path Queue
    ↓
Async Executor Pool (16 workers)
    ↓
SPARQL Query Engine / Pattern Matcher
    ↓
Result → Resume Hot Path OR
    ↓       Complete Async
Cold Path (if further delay acceptable)
```

**Code Pattern**:
```erlang
case check_tick_budget(CurrentTick) of
    ok ->
        execute_hot_path(Work);
    {error, budget_exceeded} ->
        park_to_warm_path(Work),  % Async queue
        {parked, warm_queue}
end
```

### Pattern B: Complexity → Warm Path

**Trigger**: Work requires operations not available on hot path

**Examples**:
- Complex SPARQL BGPs (>8 triple patterns)
- Graph rewriting (pattern transformation)
- External data fetching
- State machine with >8 transitions

**Detection**:
```erlang
case analyze_work_complexity(Work) of
    {simple, HotPathOps} ->
        execute_hot(HotPathOps);
    {complex, RequiresWarmPath} ->
        submit_to_warm_path(RequiresWarmPath)
end
```

### Pattern C: Error Recovery → Escalation

**From `bf_pqc_error_recovery.erl`**:

```erlang
escalate_critical_error(ErrorEvent, EscalationContext) ->
    gen_server:call(?SERVER, {escalate_critical_error, ...}).
```

**Escalation Tiers**:
1. **Warning** → Retry with delay (600s), escalate to critical
2. **Critical** → Retry with shorter delay (300s), escalate to emergency
3. **Emergency** → No further escalation, manual intervention

**Code Pattern**:
```erlang
case Severity of
    warning ->
        timer:apply_after(600_000, ?MODULE, retry, [Work]),
        maybe_escalate_to_critical(Work);
    critical ->
        timer:apply_after(300_000, ?MODULE, retry, [Work]),
        escalate_to_emergency(Work);
    emergency ->
        alert_operator(Work),
        halt_system_if_needed()
end
```

### Pattern D: Capacity → Backpressure

**From `bf_orchestration_server.erl`**:

```erlang
{ok, MaxConcurrent} = application:get_env(byteflow, max_concurrent_workflows, 1000),

case can_accept_workflow(State) of
    true ->
        handle_workflow_submission(WorkflowCrystal, State);
    false ->
        {reply, {error, capacity_exceeded}, State}  % Reject
end
```

**Backpressure Mechanisms**:
- **Reject** - Return error to client immediately
- **Queue** - Buffer in memory up to limit
- **Defer** - Schedule for later execution
- **Shed** - Drop low-priority work

---

## Integration Recommendations for KNHK

### 1. **Adopt 3-Tier Architecture**

```rust
// Hot path: knhk-hot (current)
pub const TICK_BUDGET: u32 = 8;

// Warm path: NEW - knhk-warm
pub const WARM_BUDGET_MS: u32 = 500;

// Cold path: NEW - knhk-cold (or knhk-batch)
// No time constraint, batched execution
```

### 2. **Implement Admission Gate (Θ)**

```rust
// NEW: rust/knhk-admission/src/lib.rs

pub struct AdmissionGate {
    shacl_validator: ShaclValidator,
    pb_checker: PatternByteChecker,
    pqc_verifier: PqcVerifier,
}

pub struct AdmissionResult {
    decision: Theta,  // 0 = reject, 1 = admit
    budget: u16,      // Allocated credits
    priority: u8,     // 0-7
    latency_ms: f64,  // Pipeline time
}

impl AdmissionGate {
    pub fn admit(&self, crystal: &MissionCrystal) -> AdmissionResult {
        // 4-stage pipeline: SHACL → PB → PQC → Θ
        // Target: <50ms total, <1μs for obvious rejects
    }

    pub fn zero_tick_reject(&self, crystal: &MissionCrystal) -> bool {
        // Fast path: check obvious failures
        crystal.budget == 0 ||
        crystal.pattern_byte > 43 ||
        self.is_rate_limited(&crystal.source)
    }
}
```

### 3. **Add Work Parking for Warm Path**

```rust
// NEW: rust/knhk-etl/src/park.rs (ALREADY EXISTS!)

pub struct ParkingLot {
    warm_queue: VecDeque<ParkedWork>,
    cold_queue: VecDeque<DeferredWork>,
}

pub enum ParkReason {
    BudgetExceeded,
    ComplexityTooHigh,
    ErrorRecovery,
    CapacityExceeded,
}

impl ParkingLot {
    pub fn park(&mut self, work: Work, reason: ParkReason) {
        match reason {
            ParkReason::BudgetExceeded | ParkReason::ComplexityTooHigh =>
                self.warm_queue.push_back(work),
            ParkReason::ErrorRecovery | ParkReason::CapacityExceeded =>
                self.cold_queue.push_back(work),
        }
    }
}
```

### 4. **Implement Budget Tracking**

```rust
// Add to knhk-hot/src/ffi.rs

#[repr(C)]
pub struct BudgetTracker {
    initial_budget: u16,
    remaining_budget: u16,
    ticks_consumed: u8,
    escalation_threshold: u8,
}

impl BudgetTracker {
    pub fn consume_tick(&mut self) -> Result<(), TickBudgetExceeded> {
        self.ticks_consumed += 1;
        if self.ticks_consumed > TICK_BUDGET {
            Err(TickBudgetExceeded)
        } else {
            Ok(())
        }
    }

    pub fn should_escalate(&self) -> bool {
        self.ticks_consumed >= self.escalation_threshold
    }
}
```

### 5. **Create Warm Path Executor**

```rust
// NEW: rust/knhk-warm/src/executor.rs (ALREADY EXISTS!)

pub struct WarmPathExecutor {
    thread_pool: rayon::ThreadPool,  // Or tokio runtime
    timeout: Duration,                // 500ms default
}

impl WarmPathExecutor {
    pub async fn execute(&self, work: ParkedWork) -> Result<Crystal, Error> {
        // Execute with timeout
        tokio::time::timeout(self.timeout, async {
            self.run_sparql_query(work.query).await
        }).await?
    }
}
```

---

## Performance Monitoring

### Metrics to Track

**Hot Path**:
- `hot_path_ticks_avg` - Average tick consumption
- `hot_path_budget_exhaustion_rate` - % of work exceeding budget
- `hot_path_cache_hit_rate` - L1/L2 cache efficiency
- `hot_path_dispatch_latency_ns` - Kernel dispatch time

**Warm Path**:
- `warm_path_queue_depth` - Pending work count
- `warm_path_latency_p50/p95/p99` - Latency percentiles
- `warm_path_timeout_rate` - % exceeding 500ms
- `warm_path_executor_utilization` - Thread pool usage

**Cold Path**:
- `cold_path_batch_size` - Items per batch
- `cold_path_throughput` - Items/second
- `cold_path_completion_time` - End-to-end latency

**Admission**:
- `admission_accept_rate` - % admitted
- `admission_zero_tick_reject_rate` - % fast-rejected
- `admission_pipeline_latency_ms` - 4-stage time

### Alert Thresholds

```erlang
%% From byteflow_alert_system.erl
warning => #{delay_ms => 600000, escalate_to => critical},
critical => #{delay_ms => 300000, escalate_to => emergency},
emergency => #{delay_ms => 0, escalate_to => emergency}
```

**Recommended Alerts**:
- ⚠️ **Hot path average >6 ticks** - Optimization needed
- 🔴 **Warm path queue >100** - Backpressure building
- 🚨 **Admission reject rate >20%** - Capacity or validation issue
- 🚨 **Cold path batch delay >60s** - Resource starvation

---

## Summary: Key Takeaways

### ✅ What ByteFlow Does Well

1. **Clear Performance Tiers** - Hot/Warm/Cold with explicit budgets
2. **Admission Pipeline** - 4-stage validation before hot path entry
3. **Zero-Tick Rejection** - <1μs for obvious failures
4. **Cache-Optimized** - 64-byte alignment, SPSC rings
5. **Escalation Paths** - Automatic promotion when budget exceeded
6. **Batch Processing** - Cold path amortizes overhead
7. **Comprehensive Monitoring** - Metrics at every tier

### 🎯 Patterns to Adopt in KNHK

1. **Admission Gate** - Validate before hot path (reuse ByteCore ABIs)
2. **Budget Tracking** - Track tick consumption, escalate at threshold
3. **Work Parking** - Queue system for warm/cold escalation (already exists in knhk-etl/src/park.rs!)
4. **Performance Tiers** - Explicit hot (≤8 ticks), warm (≤500ms), cold (batched)
5. **Zero-Tick Fast Path** - Reject invalid work immediately
6. **Cache Alignment** - 64-byte structures for hot path data
7. **Alert System** - Tiered escalation (warning → critical → emergency)

### 📚 Related KNHK Components

**Already Implemented**:
- ✅ `knhk-hot` - Hot path FFI (≤8 ticks)
- ✅ `knhk-warm` - Warm path query execution
- ✅ `knhk-etl/src/park.rs` - Work parking
- ✅ `knhk-validation` - Policy engine

**To Implement**:
- ⏳ `knhk-admission` - Θ gate (SHACL, PB, PQC, Θ)
- ⏳ Budget tracking in receipts
- ⏳ Tiered alerting system
- ⏳ Zero-tick rejection fast path

---

## References

- **ByteFlow Source**: `/Users/sac/bytestar/byteflow/src/`
  - `bf_admission_gate.erl` - Θ pipeline
  - `bf_hot_path_optimizer.erl` - Hot path optimization
  - `bf_orchestration_server.erl` - Workflow management
  - `bf_pqc_error_recovery.erl` - Error escalation
  - `byteflow_alert_system.erl` - Monitoring

- **ByteCore ABIs**: `/Users/sac/bytestar/bytecore/abi/`
  - `envelope.h` - 64-byte ingress
  - `crystal.h` - Result crystals
  - `ring_spsc.h` - Lock-free rings
  - `admission_gate.h` - Θ structures

- **KNHK Implementation**: `/Users/sac/knhk/rust/`
  - `knhk-hot` - Hot path FFI
  - `knhk-warm` - Warm path executor
  - `knhk-etl/src/park.rs` - Parking lot

---

**Document Version**: 1.0
**Last Updated**: 2025-11-07
**Author**: Hive Mind Collective Intelligence Analysis
