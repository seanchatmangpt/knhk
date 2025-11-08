# Tick-Based Rotation

Tick-based rotation routes work to fibers based on the current beat cycle.

## Overview

Fibers rotate through tick slots (0-7) based on the current beat cycle:
- Each tick slot has an assigned fiber
- Work is routed to the fiber for the current tick
- Fibers execute work at their assigned tick slot

## Rotation Pattern

```
Cycle 0: Fiber 0 executes
Cycle 1: Fiber 1 executes
Cycle 2: Fiber 2 executes
...
Cycle 7: Fiber 7 executes
Cycle 8: Fiber 0 executes (next epoch)
```

## Implementation

### Fiber Assignment

```rust
let tick = cycle & 0x7;
let fiber = &mut fibers[tick];
```

### Work Routing

```rust
scheduler.enqueue_delta(triples, tick)?;
let fiber = scheduler.get_fiber(tick);
fiber.execute_tick(tick)?;
```

## Benefits

1. **Load Distribution**: Work distributed across 8 fibers
2. **Parallelism**: Multiple fibers can execute simultaneously
3. **Determinism**: Same cycle → same fiber → same routing
4. **Isolation**: Per-tick slots prevent data races

## Related Documentation

- [Fiber Execution](../fiber-execution.md) - Overview
- [Lifecycle](lifecycle.md) - Fiber lifecycle
- [Park and Escalate](park-escalate.md) - Failure handling
