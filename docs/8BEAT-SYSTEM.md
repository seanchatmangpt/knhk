# 8-Beat Reconciliation Epoch System

**Version**: 1.0  
**Status**: Active  
**Last Updated**: January 2025

## Overview

The 8-Beat Reconciliation Epoch system provides a fixed-cadence, branchless timing model for deterministic reconciliation operations. Every delta (Δ) is admitted on beat `k`, reconciled by μ within ≤8 ticks, emitted with receipt, or parked to W1.

**Key Principles**:
- **Determinism**: Global beat defines order Λ across pods and shards
- **Bounded time**: R1 completion ≤8 ticks per admitted unit
- **Branchless cadence**: Zero branch mispredicts on hot path
- **Isolation**: Over-budget work parks to W1 automatically
- **Provenance**: Every beat yields receipts with `hash(A)=hash(μ(O))`

## Architecture

### Components

1. **C Beat Scheduler** (`c/include/knhk/beat.h`, `c/src/beat.c`)
   - Branchless cycle/tick/pulse generation
   - Global atomic cycle counter
   - Tick extraction: `tick = cycle & 0x7` (0-7)
   - Pulse detection: `pulse = (tick == 0)` (branchless)

2. **C Fiber Execution** (`c/include/knhk/fiber.h`, `c/src/fiber.c`)
   - Per-shard, per-hook execution units
   - Executes μ on ≤8 items (run_len≤8)
   - Parks to W1 if ticks>8 or L1 miss predicted

3. **C Ring Buffers** (`c/include/knhk/ring.h`, `c/src/ring.c`)
   - Δ-ring (input): SoA layout for deltas
   - A-ring (output): SoA layout for assertions + receipts
   - Power-of-2 size for mod-8 indexing
   - Branchless enqueue/dequeue with atomic operations

4. **Rust Integration** (`rust/knhk-etl/src/beat_scheduler.rs`)
   - Uses C beat scheduler for cycle/tick/pulse
   - Manages delta rings and assertion rings
   - Coordinates fiber execution
   - Handles lockchain append on pulse boundary

5. **Rust Fiber** (`rust/knhk-etl/src/fiber.rs`)
   - Calls C fiber executor for hot path execution
   - Handles parking and receipt generation
   - Manages tick budget enforcement

## Functional Requirements

1. **Beat generation**: Global `cycle` increments; `tick = cycle & 0x7`
2. **Admission**: Sidecar stamps Δ with `cycle_id`; enqueue to Δ-ring[tick]
3. **Execution**: Fibers consume slot=tick, run μ on ≤8 items (run_len≤8)
4. **Park rule**: If kernel predicts L1 miss or ticks>8 → park Δ→W1 with receipt
5. **Emit**: Write A + receipt to out-ring[tick]; lockchain append
6. **Order Λ**: Commit happens at tick wrap (pulse)
7. **Drift control**: μ∘μ=μ holds across wraps; no cross-beat mutation
8. **Backpressure**: When Δ volume overflows, admission throttles by policy

## Performance Requirements

- **Latency (R1)**: ≤2ns/op (≤8 ticks per unit)
- **Hit-rate**: L1 ≥95% for hot predicates
- **Branch mispredicts**: 0 on hot path
- **Receipts coverage**: 100%
- **Availability**: ≥99.99% R1 service

## Implementation Status

### ✅ Completed

- **C Beat Scheduler**: Branchless cycle/tick/pulse generation
- **C Fiber Execution**: Hot path execution with tick budget enforcement
- **C Ring Buffers**: Lock-free ring buffers with SoA layout
- **Rust FFI Bindings**: Complete FFI bindings for C components
- **Rust Integration**: Beat scheduler and fiber integration
- **Chicago TDD Tests**: 22 tests covering beat scheduler, fiber, ring conversion

### Integration Points

**C → Rust FFI** (`rust/knhk-hot/src/`):
- `beat_ffi.rs`: Beat scheduler functions
- `ring_ffi.rs`: Ring buffer operations
- `fiber_ffi.rs`: Fiber execution

**Rust ETL Integration** (`rust/knhk-etl/src/`):
- `beat_scheduler.rs`: Uses C beat scheduler, manages rings
- `fiber.rs`: Calls C fiber executor, handles parking

## Usage

### Initialization

```rust
use knhk_etl::beat_scheduler::BeatScheduler;

// Create beat scheduler (4 shards, 2 domains, ring capacity 8)
let mut scheduler = BeatScheduler::new(4, 2, 8)?;

// Initialize C beat scheduler (call once at startup)
knhk_hot::BeatScheduler::init();
```

### Beat Execution

```rust
// Advance to next beat
let (tick, pulse) = scheduler.advance_beat();

// Execute fibers for current tick
// (handled internally by advance_beat())

// Commit on pulse boundary (every 8 ticks)
if pulse {
    // Lockchain append happens automatically
}
```

### Enqueue Delta

```rust
// Enqueue delta to delta ring
scheduler.enqueue_delta(
    domain_id,
    delta_triples,
    scheduler.current_cycle(),
)?;
```

## Related Documentation

- **[8BEAT-PRD.txt](8BEAT-PRD.txt)** - Complete Product Requirements Document
- **[BRANCHLESS_C_ENGINE_IMPLEMENTATION.md](BRANCHLESS_C_ENGINE_IMPLEMENTATION.md)** - Branchless C engine details
- **[INTEGRATION_SUMMARY.md](INTEGRATION_SUMMARY.md)** - C/Rust integration summary

## Testing

Chicago TDD tests cover:
- Beat scheduler creation and advancement
- Tick rotation and pulse detection
- Fiber execution within tick budget
- Ring conversion (SoA ↔ RawTriple)
- Receipt generation and merging

Run tests:
```bash
cargo test --package knhk-etl --test chicago_tdd_beat_scheduler
cargo test --package knhk-etl --test chicago_tdd_pipeline
```


