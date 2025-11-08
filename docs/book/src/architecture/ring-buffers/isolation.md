# Per-Tick Isolation

Per-tick isolation prevents data races and enables parallel processing.

## Overview

Each tick slot (0-7) has isolated buffer space:
- **Delta Ring**: Isolated input slots
- **Assertion Ring**: Isolated output slots
- **Fiber Execution**: Isolated execution context

## Isolation Mechanism

### Slot Assignment

```rust
let tick = cycle & 0x7;
let delta_slot = &delta_ring.slots[tick];
let assertion_slot = &assertion_ring.slots[tick];
```

### Memory Isolation

Each slot has separate memory:
- No shared state between slots
- Prevents data races
- Enables parallel processing

## Benefits

1. **Data Race Prevention**: No shared state between ticks
2. **Parallelism**: Multiple ticks can process simultaneously
3. **Determinism**: Same tick → same slot → same behavior
4. **Simplified Memory Management**: Per-slot cleanup

## Related Documentation

- [Ring Buffers](../ring-buffers.md) - Overview
- [Delta Ring](delta-ring.md) - Input buffer
- [Assertion Ring](assertion-ring.md) - Output buffer
