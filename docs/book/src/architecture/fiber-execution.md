# Fiber Execution

Fiber execution provides per-shard execution units with tick-based rotation and park/escalate capabilities.

## Overview

Fibers are lightweight execution units that process work within the 8-beat epoch system. Each fiber operates on a shard of data and executes within the tick budget.

## Key Concepts

### Fiber Lifecycle

1. **Creation**: Fibers are created per shard
2. **Execution**: Execute work at assigned tick slot
3. **Park**: Park over-budget work to W1
4. **Escalate**: Escalate critical failures to supervisor

### Tick-Based Rotation

Fibers rotate through tick slots (0-7) based on the current beat cycle:

```rust
let tick = cycle & 0x7;  // Extract tick (0-7)
let fiber = fibers[tick]; // Get fiber for this tick
```

## C Layer Implementation

### Functions

- `knhk_fiber_execute()`: Execute μ on ≤8 items at tick slot
- `knhk_fiber_park()`: Park over-budget work to W1
- `knhk_fiber_process_tick()`: Process tick (read → execute → write)

### Memory Layout

- SoA-optimized for SIMD operations
- 64-byte alignment for cache lines
- Power-of-2 sizing for efficient indexing

## Rust Integration

### BeatScheduler Integration

```rust
let scheduler = BeatScheduler::new();
let (tick, pulse) = scheduler.advance_beat();
let fiber = scheduler.get_fiber(tick);
```

### Fiber Management

- Per-shard fibers with tick-based rotation
- Automatic park on over-budget
- W1 escalation for warm path processing

## Performance

- **Hot Path**: ≤8 ticks per tick slot
- **Automatic Park**: Over-budget work escalated to W1
- **Zero-Copy**: SoA layout minimizes memory copies

## Related Documentation

- [8-Beat System](8beat-system.md) - Epoch system overview
- [Ring Buffers](ring-buffers.md) - Input/output buffers
- [Hot Path](hot-path.md) - Hot path operations

