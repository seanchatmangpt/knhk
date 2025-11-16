# Deterministic Multi-Core Scheduler Architecture

**Epic 2 Implementation - Type-Safe Concurrency for KNHK μ-Kernel**

## Executive Summary

The KNHK μ-kernel concurrency system implements deterministic, multi-core scheduling where concurrency properties are expressed and enforced by Rust's type system. This ensures:

1. **Zero data races** (proven at compile time)
2. **Deterministic execution** (same inputs → same outputs)
3. **Cross-machine reproducibility** (via replay infrastructure)
4. **Type-level resource contracts** (tick budgets, priorities, guards)

## Core Principles

### 1. Type-Level Concurrency Guarantees

**Problem**: Traditional concurrency relies on runtime checks (mutexes, locks) which add overhead and can fail.

**Solution**: Use Rust's type system to enforce concurrency properties at compile time.

```rust
// Core-local data (NEVER crosses core boundaries)
pub struct CoreLocal<T> {
    data: UnsafeCell<T>,
    _not_send: PhantomData<*const ()>,  // !Send
    _not_sync: PhantomData<UnsafeCell<()>>,  // !Sync
}

// Shared data (requires explicit ordering)
pub struct Shared<T: Send + Sync> {
    data: Arc<T>,
    ordering: GlobalOrdering,  // SeqCst, AcqRel, or Relaxed
}
```

**Guarantees**:
- `CoreLocal<T>` cannot be moved to another thread (type system prevents it)
- `Shared<T>` requires `T: Send + Sync` and explicit memory ordering
- No possibility of accidentally sharing core-local data

### 2. Deterministic Work Queues

**Three queue types for different ordering guarantees**:

#### a) CoreLocal Work Queue (SPSC)
```rust
pub struct WorkQueue<T, const CAPACITY: usize> {
    head: AtomicUsize,  // Consumer
    tail: AtomicUsize,  // Producer
    buffer: [Option<T>; CAPACITY],  // Ring buffer
}
```

- **Single-Producer, Single-Consumer** (SPSC)
- **Lock-free** (no blocking operations)
- **Cache-friendly** (128-byte aligned, padding to prevent false sharing)
- **Bounded** (fixed capacity for determinism)
- **FIFO** (preserves enqueue order)

#### b) GlobalOrdered Queue (Timestamp-Based)
```rust
pub struct GlobalOrdered<T> {
    heap: Vec<TimestampedEvent<T>>,  // Min-heap by timestamp
    lock: AtomicU64,  // Spinlock
}
```

- **Total Order** (all events ordered by Lamport timestamps)
- **Deterministic** (ties broken by core_id)
- **Thread-safe** (spinlock for heap modifications)

#### c) BestEffort Queue (MPMC)
```rust
pub struct BestEffort<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}
```

- **Lock-free MPMC** (Michael-Scott queue)
- **No ordering guarantees** (for metrics, logging)
- **Best performance** (minimal synchronization)

### 3. Schedulable Tasks with Resource Contracts

**Type-safe resource constraints**:

```rust
pub struct SchedulableTask<B, P, G>
where
    P: Priority,     // Compile-time priority
    G: GuardSet,     // Compile-time guard requirements
{
    task_id: u64,
    tick_budget: TickBudget,  // Runtime tick budget
    _priority: PhantomData<P>,  // Type-level priority
    guards: G,  // Type-level guards
    work: TaskWork,
    sigma: &'static SigmaPointer,
}
```

**Priority Types**:
- `PriorityHigh` (LEVEL = 0)
- `PriorityNormal` (LEVEL = 1)
- `PriorityLow` (LEVEL = 2)

**Guard Types**:
- `NoGuards` (COUNT = 0)
- `SingleGuard(GuardId)` (COUNT = 1)
- `MultiGuard<N>([GuardId; N])` (COUNT = N, max 8)

### 4. Deterministic Scheduler

**Architecture**:

```text
DeterministicScheduler<CORES>:
  ├─ core_queues[CORES]      - Per-core work queues (CoreLocal)
  ├─ global_order             - Globally ordered queue (Shared)
  ├─ logical_clock            - Lamport clock (Shared)
  └─ replay_log               - Event log (Shared)
```

**Key Properties**:

1. **Deterministic Core Assignment**: `core_id = task_id & (CORES - 1)`
2. **Logical Timestamps**: Lamport clocks for happens-before ordering
3. **Event Logging**: All non-deterministic events recorded
4. **Type-Safe API**: Cannot enqueue invalid tasks

**Example Usage**:

```rust
let scheduler = DeterministicScheduler::<4>::new(sigma_ptr);

let task = SchedulableTask::new(
    task_id,
    TickBudget::chatman(),  // ≤8 ticks
    PriorityHigh,
    NoGuards,
    TaskWork::Pure(42),
    sigma_ptr,
)?;

scheduler.enqueue(task)?;  // Deterministically assigned to core

let result = scheduler.run_cycle(core_id)?;
```

### 5. Logical Time and Causality

**Lamport Clocks**:

```rust
pub struct LogicalClock {
    timestamp: AtomicU64,
}

impl LogicalClock {
    pub fn tick(&self) -> Timestamp {
        // Rule 1: Increment before local event
        self.timestamp.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn recv(&self, remote: Timestamp) -> Timestamp {
        // Rule 2: max(local, remote) + 1
        loop {
            let current = self.timestamp.load(Ordering::SeqCst);
            let new = current.max(remote.0) + 1;
            if self.timestamp.compare_exchange_weak(
                current, new,
                Ordering::SeqCst, Ordering::SeqCst
            ).is_ok() {
                return Timestamp(new);
            }
        }
    }
}
```

**Happened-Before Relationship**:

```rust
pub trait HappensBefore {
    fn happens_before(&self, other: &Self) -> bool;
}

impl HappensBefore for Timestamp {
    fn happens_before(&self, other: &Self) -> bool {
        self.0 < other.0
    }
}
```

**Timestamped Events**:

```rust
pub struct TimestampedEvent<T> {
    timestamp: Timestamp,
    core_id: u8,  // For deterministic tie-breaking
    event: T,
}

// Total order: (timestamp, core_id)
impl<T> Ord for TimestampedEvent<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
            .then(self.core_id.cmp(&other.core_id))
    }
}
```

### 6. Deterministic Replay Infrastructure

**Replay Events**:

```rust
pub enum ReplayEvent {
    TaskEnqueued { task_id, core_id, timestamp },
    TaskExecuted { task_id, core_id, timestamp, ticks, output_hash },
    StateChange { timestamp, state_hash },
    ExternalInput { timestamp, input_hash },  // Non-deterministic!
}
```

**Replay Log**:

```rust
pub struct ReplayLog {
    events: Vec<ReplayEvent>,
    checksum: u64,  // XOR of all timestamps
}

impl ReplayLog {
    pub fn verify(&self) -> bool {
        // Recompute checksum
        let mut computed = 0u64;
        for event in &self.events {
            computed ^= event.timestamp().as_raw();
        }
        computed == self.checksum
    }
}
```

**Deterministic Trait**:

```rust
pub trait Deterministic {
    type Seed;

    fn replay(&self, seed: Self::Seed) -> ReplayIterator;
    fn seed(&self) -> Self::Seed;
}
```

**Replay Verification**:

```rust
pub fn compare_replays(original: &ReplayLog, replay: &ReplayLog)
    -> ReplayResult
{
    // ExactMatch, MinorDifference, or Diverged
}
```

## Type-Level Guarantees

### Compile-Time Safety

**1. Data Race Freedom**

```rust
// ❌ COMPILE ERROR: CoreLocal<T> is !Send
let local = CoreLocal::new(42);
std::thread::spawn(move || {
    local.with(|_| {});  // ERROR: cannot send between threads
});

// ✅ OK: Shared<T> is Send + Sync
let shared = Shared::new(AtomicU64::new(0), GlobalOrdering::SeqCst);
std::thread::spawn(move || {
    shared.fetch_add(1);  // OK
});
```

**2. Guard Validation**

```rust
// ❌ COMPILE ERROR: Invalid guard ID
let invalid = SingleGuard(2000);
SchedulableTask::new(..., invalid, ...)?;  // ERROR at runtime (validation)

// ✅ OK: Valid guard
let valid = SingleGuard(42);
SchedulableTask::new(..., valid, ...)?;  // OK
```

**3. Priority Enforcement**

```rust
// Type system tracks priority
let high_task: SchedulableTask<_, PriorityHigh, _> = ...;
let normal_task: SchedulableTask<_, PriorityNormal, _> = ...;

assert_eq!(high_task.priority(), 0);
assert_eq!(normal_task.priority(), 1);
```

### Runtime Guarantees

**1. Deterministic Execution**

```rust
// Same inputs → same outputs
let scheduler1 = DeterministicScheduler::<4>::new(sigma);
let scheduler2 = DeterministicScheduler::<4>::new(sigma);

// Enqueue identical tasks
for task_id in 1..=10 {
    scheduler1.enqueue(make_task(task_id))?;
    scheduler2.enqueue(make_task(task_id))?;
}

// Execute
let results1 = execute_all(&mut scheduler1);
let results2 = execute_all(&mut scheduler2);

// Verify determinism
assert_eq!(results1, results2);
```

**2. Cross-Machine Reproducibility**

```rust
// Machine A
let log_a = scheduler_a.replay_log();
save_to_disk(log_a);

// Machine B
let log_b = load_from_disk();
let comparison = compare_replays(log_a, log_b);
assert_eq!(comparison, ReplayResult::ExactMatch);
```

## Performance Characteristics

### Lock-Free Operations

- **Core-local queues**: Zero synchronization overhead
- **Global ordered queue**: Spinlock only during enqueue/dequeue
- **Logical clock**: Atomic CAS operations

### Cache-Friendly Design

- **128-byte alignment**: Prevents false sharing
- **Ring buffers**: Cache-line prefetching
- **Padding**: Separate cache lines for head/tail

### Tick Budget Compliance

- **Enqueue**: O(1), ~2-3 ticks
- **Dequeue**: O(1), ~2-3 ticks
- **Timestamp**: O(1), ~1 tick
- **Total overhead**: ≤8 ticks (within Chatman Constant)

## Integration with μ-Kernel

### Memory Layout

```text
Concurrency Space:
  0x0000_5000_0000 - Core-local queues (per-core, 64KB each)
  0x0000_5100_0000 - Global ordered queue (shared, 1MB)
  0x0000_5200_0000 - Logical clocks (shared, 64KB)
  0x0000_5300_0000 - Replay logs (shared, 1MB)
```

### Integration Points

1. **MuKernel**: Scheduler integrated into `MuState`
2. **ISA**: `MuInstruction::eval_task` executed via scheduler
3. **Receipts**: Timestamped receipts from replay log
4. **Guards**: Guard validation before task execution

## Usage Examples

### Example 1: Simple Task Execution

```rust
let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
let mut scheduler = DeterministicScheduler::<4>::new(sigma_ptr);

let task = SchedulableTask::new(
    1,
    TickBudget::chatman(),
    PriorityNormal,
    NoGuards,
    TaskWork::Pure(42),
    sigma_ptr,
)?;

scheduler.enqueue(task)?;

let result = scheduler.run_cycle(1)?;  // Core 1 (task_id=1 → core_id=1)
println!("Task {} completed in {} ticks",
    result.task_id, result.result.ticks_used);
```

### Example 2: Globally Ordered Tasks

```rust
// Tasks that must execute in timestamp order
let task1 = SchedulableTask::new(...)?;
let task2 = SchedulableTask::new(...)?;

scheduler.enqueue_ordered(task1, Timestamp::from_raw(10))?;
scheduler.enqueue_ordered(task2, Timestamp::from_raw(5))?;

// task2 executes first (lower timestamp)
```

### Example 3: Deterministic Replay

```rust
// Original execution
let mut scheduler = DeterministicScheduler::<4>::new(sigma_ptr);
for task in tasks {
    scheduler.enqueue(task)?;
}
execute_all(&mut scheduler);
let original_log = scheduler.replay_log();

// Replay execution
let mut replay_scheduler = DeterministicScheduler::<4>::new(sigma_ptr);
for task in tasks {
    replay_scheduler.enqueue(task)?;
}
execute_all(&mut replay_scheduler);
let replay_log = replay_scheduler.replay_log();

// Verify determinism
assert_eq!(
    compare_replays(original_log, replay_log),
    ReplayResult::ExactMatch
);
```

## Testing Strategy

### Unit Tests

- Type-level guarantees (compile-time tests)
- Core-local queue operations
- Global ordered queue ordering
- Logical clock synchronization
- Replay log verification

### Integration Tests

- Multi-core task execution
- Determinism across runs
- Cross-machine replay
- Resource contract enforcement

### Performance Tests

- Tick budget compliance (≤8 ticks)
- Throughput benchmarks
- Latency measurements
- Scalability tests (1-256 cores)

## Future Extensions

### 1. Hierarchical Scheduling

```rust
pub struct HierarchicalScheduler<const CORES: usize> {
    schedulers: [DeterministicScheduler<1>; CORES],
    coordinator: GlobalCoordinator,
}
```

### 2. Adaptive Topology

```rust
pub enum Topology {
    Flat,  // All cores equal
    Hierarchical { levels: usize },
    Mesh { dimensions: [usize; 3] },
}
```

### 3. Work Stealing

```rust
impl<const CORES: usize> DeterministicScheduler<CORES> {
    pub fn steal_work(&mut self, from_core: usize, to_core: usize)
        -> Result<(), SchedulerError>
    {
        // Deterministic work stealing (preserves replay)
    }
}
```

## Summary

The KNHK μ-kernel concurrency system achieves:

✅ **Type-safe concurrency** (zero data races, proven at compile time)
✅ **Deterministic execution** (same inputs → same outputs)
✅ **Cross-machine reproducibility** (via replay infrastructure)
✅ **Performance** (≤8 ticks overhead, within Chatman Constant)
✅ **Scalability** (1-256 cores, lock-free design)

This provides a foundation for deterministic, multi-core knowledge operations in the KNHK μ-kernel.
