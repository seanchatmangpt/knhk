# Delta Ring

Delta ring provides input buffer for deltas (changes) with per-tick isolation.

## Overview

The delta ring (Δ-ring) stores incoming deltas:
- Per-tick slot isolation
- SoA layout for SIMD
- Cycle ID stamping
- Atomic enqueue/dequeue

## Structure

```c
typedef struct {
    uint64_t cycle_id;
    knhk_soa_t delta;
} knhk_delta_slot_t;

typedef struct {
    knhk_delta_slot_t slots[8];  // One slot per tick
    atomic_uint_fast64_t write_index;
    atomic_uint_fast64_t read_index;
} knhk_delta_ring_t;
```

## Operations

### Enqueue

```c
int knhk_ring_enqueue_delta(knhk_delta_ring_t *ring, uint64_t tick, knhk_delta_t *delta) {
    // Enqueue delta to tick slot
    // Atomic index update
    // SoA conversion
}
```

### Dequeue

```c
int knhk_ring_dequeue_delta(knhk_delta_ring_t *ring, uint64_t tick, knhk_delta_t *delta) {
    // Dequeue delta from tick slot
    // Atomic index read
    // SoA to RawTriple conversion
}
```

## Per-Tick Isolation

Each tick slot (0-7) has isolated buffer space:
- Prevents data races
- Enables parallel processing
- Simplifies memory management

## Related Documentation

- [Ring Buffers](../ring-buffers.md) - Overview
- [SoA Layout](soa-layout.md) - Memory layout
- [Assertion Ring](assertion-ring.md) - Output buffer
