# Hot Path

The hot path provides ≤2ns latency (8 ticks) for critical operations.

## Overview

The hot path is the C layer implementation optimized for minimal latency. All operations must complete within 8 ticks (Chatman Constant).

## Performance Targets

- **Single Hook Execution**: <2ns (8 ticks) per admitted unit
- **Ring Buffer Operations**: Branchless enqueue/dequeue
- **Fiber Execution**: ≤8 ticks per tick slot
- **Memory Layout**: Zero-copy, SIMD-aware, 64-byte alignment

## Key Operations

### Beat Scheduler

- **Cycle Counter**: Atomic increment
- **Tick Extraction**: `cycle & 0x7` (branchless)
- **Pulse Detection**: Branchless pulse signal (1 when tick==0)

### Ring Buffers

- **Enqueue**: Branchless atomic index update
- **Dequeue**: Branchless atomic index read
- **Isolation**: Per-tick slot isolation

### Fiber Execution

- **Execute**: Process ≤8 items at tick slot
- **Park**: Automatic park on over-budget
- **Escalate**: W1 escalation for warm path

### Eval Dispatch

Branchless kernel dispatch for:
- ASK operations
- COUNT operations
- COMPARE operations
- VALIDATE operations
- SELECT operations
- UNIQUE operations

## Branchless Operations

All hot path operations are branchless for constant-time execution:

```c
// Branchless tick extraction
uint64_t tick = cycle & 0x7;

// Branchless pulse detection
uint64_t pulse = (tick == 0) ? 1 : 0;
// Optimized to: pulse = !(tick & 0x7);
```

## Memory Layout

### SoA Layout

Structure of Arrays (SoA) for SIMD optimization:
- 64-byte alignment for cache lines
- Power-of-2 sizing for efficient indexing
- Zero-copy operations where possible

### Cache Optimization

- 64-byte alignment prevents false sharing
- SoA layout optimizes cache line usage
- Prefetch-friendly access patterns

## Performance Validation

### PMU Benchmarks

Performance monitoring unit (PMU) benchmarks verify ≤8 tick budget:

```bash
make test-performance-v04
```

### Chatman Constant

All hot path operations must complete within 8 ticks (Chatman Constant):
- Validated through PMU benchmarks
- Enforced through guard constraints
- Monitored through SLO tracking

## Related Documentation

- [8-Beat System](8beat-system.md) - Epoch system overview
- [Branchless C Engine](branchless-engine.md) - C implementation
- [Fiber Execution](fiber-execution.md) - Execution units

