# Tick Rotation

Tick rotation provides deterministic work routing based on the current beat cycle.

## Overview

Work is routed to tick slots (0-7) based on the current beat cycle:
- **Tick 0**: First slot in epoch
- **Tick 1-6**: Intermediate slots
- **Tick 7**: Last slot before pulse
- **Pulse**: Commit boundary (tick wraps to 0)

## Tick Extraction

```c
// Branchless tick extraction
uint64_t tick = cycle & 0x7;
```

This extracts bits 0-2 from the cycle counter, giving values 0-7.

## Work Routing

### Delta Admission

Deltas are admitted to the tick slot corresponding to the current cycle:

```rust
let tick = cycle & 0x7;
scheduler.enqueue_delta(triples, tick)?;
```

### Fiber Execution

Fibers execute work at their assigned tick slot:

```rust
let fiber = scheduler.get_fiber(tick);
fiber.execute_tick(tick)?;
```

### Ring Buffer Slots

Ring buffers use tick slots for isolation:

```rust
let slot = &ring.slots[tick];
slot.enqueue(delta)?;
```

## Rotation Pattern

The rotation pattern repeats every 8 beats:

```
Cycle 0: Tick 0 → Pulse
Cycle 1: Tick 1
Cycle 2: Tick 2
...
Cycle 7: Tick 7
Cycle 8: Tick 0 → Pulse (next epoch)
```

## Benefits

1. **Deterministic**: Same cycle → same tick → same routing
2. **Isolation**: Per-tick slots prevent data races
3. **Parallelism**: Multiple ticks can execute in parallel
4. **Ordering**: Global beat defines order across pods/shards

## Related Documentation

- [Beat Scheduler](beat-scheduler.md) - Core scheduler
- [Pulse Detection](pulse-detection.md) - Commit boundaries
- [Fiber Execution](../fiber-execution.md) - Execution units

