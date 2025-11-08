# Ring Buffers

Ring buffers provide SoA-optimized input (Δ-ring) and output (A-ring) with per-tick isolation.

## Overview

KNHK uses two ring buffers:
- **Δ-ring**: Input buffer for deltas (changes)
- **A-ring**: Output buffer for assertions (results)

Both buffers use Structure of Arrays (SoA) layout for SIMD optimization.

## Design Principles

### SoA Layout

Structure of Arrays (SoA) layout optimizes for SIMD operations:

```rust
struct SoAArrays {
    s: [u64; 8],  // Subjects
    p: [u64; 8],  // Predicates
    o: [u64; 8],  // Objects
}
```

### Per-Tick Isolation

Each tick slot (0-7) has isolated buffer space:
- Prevents data races
- Enables parallel processing
- Simplifies memory management

### 64-Byte Alignment

Buffers are aligned to 64-byte cache lines:
- Optimizes cache performance
- Enables SIMD operations
- Reduces false sharing

## C Layer Implementation

### Ring Buffer Structure

```c
typedef struct {
    uint64_t cycle_id;
    uint64_t s[8];
    uint64_t p[8];
    uint64_t o[8];
} knhk_ring_slot_t;
```

### Operations

- **Enqueue**: Add delta to Δ-ring at tick slot
- **Dequeue**: Read delta from Δ-ring
- **Commit**: Write assertion to A-ring
- **Isolation**: Per-tick slot isolation

## Rust Integration

### Ring Conversion

```rust
// Convert RawTriple to SoA
let soa = ring_conversion::triples_to_soa(triples)?;

// Convert SoA to RawTriple
let triples = ring_conversion::soa_to_triples(&soa)?;
```

### Beat Scheduler Integration

```rust
let scheduler = BeatScheduler::new();
scheduler.enqueue_delta(triples, tick)?;
let assertions = scheduler.dequeue_assertions(tick)?;
```

## Performance

- **Zero-Copy**: SoA layout minimizes copies
- **SIMD-Optimized**: Aligned for vector operations
- **Cache-Friendly**: 64-byte alignment

## Related Documentation

- [8-Beat System](8beat-system.md) - Epoch system overview
- [Fiber Execution](fiber-execution.md) - Execution units
- [Hot Path](hot-path.md) - Hot path operations

