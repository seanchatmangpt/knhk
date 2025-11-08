# C API

The C API provides the hot path implementation for KNHK.

## Overview

The C layer implements the 8-beat epoch system with branchless operations for minimal latency.

## Core Components

### Beat Scheduler

```c
// Advance cycle atomically
uint64_t knhk_beat_next(void);

// Extract tick (0-7) via cycle & 0x7
uint64_t knhk_beat_tick(uint64_t cycle);

// Compute pulse signal (1 when tick==0) branchlessly
uint64_t knhk_beat_pulse(uint64_t cycle);
```

### Ring Buffers

```c
// Enqueue delta to Δ-ring at tick slot
int knhk_ring_enqueue_delta(knhk_ring_t *ring, uint64_t tick, knhk_delta_t *delta);

// Dequeue delta from Δ-ring
int knhk_ring_dequeue_delta(knhk_ring_t *ring, uint64_t tick, knhk_delta_t *delta);

// Commit assertion to A-ring
int knhk_ring_commit_assertion(knhk_ring_t *ring, uint64_t tick, knhk_assertion_t *assertion);
```

### Fiber Execution

```c
// Execute μ on ≤8 items at tick slot
knhk_fiber_result_t knhk_fiber_execute(knhk_fiber_t *fiber, uint64_t tick);

// Park over-budget work to W1
int knhk_fiber_park(knhk_fiber_t *fiber, knhk_work_t *work);

// Process tick (read → execute → write)
int knhk_fiber_process_tick(knhk_fiber_t *fiber, uint64_t tick);
```

### Eval Dispatch

```c
// Branchless ASK operation
knhk_result_t knhk_eval_ask(knhk_soa_t *soa, knhk_query_t *query);

// Branchless COUNT operation
uint64_t knhk_eval_count(knhk_soa_t *soa, knhk_query_t *query);

// Branchless COMPARE operation
int knhk_eval_compare(knhk_soa_t *soa, knhk_query_t *query);
```

## Header Files

- `c/include/knhk/beat.h` - Beat scheduler
- `c/include/knhk/ring.h` - Ring buffers
- `c/include/knhk/fiber.h` - Fiber execution
- `c/include/knhk/eval_dispatch.h` - Eval dispatch
- `c/include/knhk/types.h` - Core types

## Performance

All C API functions are optimized for:
- **Branchless operations**: Constant-time execution
- **SIMD optimization**: SoA layout for vector operations
- **Cache optimization**: 64-byte alignment

## Related Documentation

- [Rust API](rust-api.md) - Rust FFI bindings
- [FFI Integration](ffi-integration.md) - FFI bridge
- [Architecture](../architecture/system-overview.md) - System design
