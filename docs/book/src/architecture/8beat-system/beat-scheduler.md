# Beat Scheduler

The beat scheduler manages the 8-beat epoch system with atomic cycle advancement and branchless tick/pulse generation.

## Overview

The beat scheduler is the core of the 8-beat epoch system, providing:
- Atomic cycle counter advancement
- Branchless tick extraction
- Branchless pulse detection
- Cycle ID generation

## C Layer Implementation

### Core Functions

```c
// Advance cycle atomically
uint64_t knhk_beat_next(void);

// Extract tick (0-7) via cycle & 0x7
uint64_t knhk_beat_tick(uint64_t cycle);

// Compute pulse signal (1 when tick==0) branchlessly
uint64_t knhk_beat_pulse(uint64_t cycle);
```

### Implementation Details

**Cycle Counter**:
- Atomic increment using `__sync_fetch_and_add` or `stdatomic.h`
- Thread-safe across all fibers
- Global ordering across pods/shards

**Tick Extraction**:
```c
static inline uint64_t knhk_beat_tick(uint64_t cycle) {
    return cycle & 0x7;  // Branchless: extract bits 0-2
}
```

**Pulse Detection**:
```c
static inline uint64_t knhk_beat_pulse(uint64_t cycle) {
    return !(cycle & 0x7);  // Branchless: 1 when tick==0, 0 otherwise
}
```

## Rust Integration

### BeatScheduler

```rust
pub struct BeatScheduler {
    cycle: AtomicU64,
    // ... other fields
}

impl BeatScheduler {
    pub fn advance_beat(&mut self) -> (u64, bool) {
        let cycle = self.cycle.fetch_add(1, Ordering::SeqCst);
        let tick = cycle & 0x7;
        let pulse = tick == 0;
        (tick, pulse)
    }
}
```

### Integration Points

- **ETL Pipeline**: Beat advancement triggers pipeline stages
- **Fiber Rotation**: Tick determines which fiber executes
- **Ring Buffers**: Tick determines which slot to use
- **Commit Boundary**: Pulse triggers receipt commit

## Performance

- **Cycle Advancement**: Atomic operation, constant time
- **Tick Extraction**: Bitwise AND, constant time
- **Pulse Detection**: Bitwise operation, constant time
- **Total Cost**: <1 tick per beat advancement

## Related Documentation

- [Tick Rotation](tick-rotation.md) - Tick-based work routing
- [Pulse Detection](pulse-detection.md) - Pulse signal generation
- [Epoch Containment](epoch-containment.md) - Time-bounded execution

