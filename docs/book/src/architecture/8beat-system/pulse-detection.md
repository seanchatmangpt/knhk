# Pulse Detection

Pulse detection identifies commit boundaries in the 8-beat epoch system.

## Overview

A pulse occurs every 8 ticks (when tick wraps from 7 to 0), signaling:
- **Commit Boundary**: Time to commit receipts
- **Epoch Completion**: One full epoch completed
- **Receipt Generation**: Generate receipts for completed work
- **Lockchain Update**: Update lockchain with receipts

## Branchless Implementation

### C Layer

```c
static inline uint64_t knhk_beat_pulse(uint64_t cycle) {
    return !(cycle & 0x7);  // 1 when tick==0, 0 otherwise
}
```

### Rust Layer

```rust
let pulse = (tick == 0);
// Optimized to: pulse = !(cycle & 0x7);
```

## Pulse Events

### Commit Cycle

When pulse is detected:

```rust
if pulse {
    let receipts = scheduler.commit_cycle()?;
    lockchain.store_receipts(receipts)?;
}
```

### Receipt Generation

Receipts are generated for all assertions in the completed epoch:

```rust
let receipts: Vec<Receipt> = assertions
    .iter()
    .map(|assertion| generate_receipt(assertion))
    .collect();
```

### Lockchain Update

Receipts are stored in the lockchain:

```rust
for receipt in receipts {
    lockchain.store_receipt(receipt)?;
}
```

## Pulse Frequency

- **Every 8 ticks**: One pulse per epoch
- **Deterministic**: Pulse occurs at cycle % 8 == 0
- **Global**: All pods/shards pulse simultaneously

## Related Documentation

- [Beat Scheduler](beat-scheduler.md) - Core scheduler
- [Tick Rotation](tick-rotation.md) - Work routing
- [Epoch Containment](epoch-containment.md) - Time bounds

