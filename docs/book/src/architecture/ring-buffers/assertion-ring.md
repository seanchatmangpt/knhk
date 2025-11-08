# Assertion Ring

Assertion ring provides output buffer for assertions (results) with per-tick isolation.

## Overview

The assertion ring (A-ring) stores output assertions:
- Per-tick slot isolation
- SoA layout for SIMD
- Receipt generation
- Atomic commit

## Structure

```c
typedef struct {
    uint64_t cycle_id;
    knhk_receipt_t receipt;
    knhk_soa_t assertion;
} knhk_assertion_slot_t;

typedef struct {
    knhk_assertion_slot_t slots[8];  // One slot per tick
    atomic_uint_fast64_t write_index;
    atomic_uint_fast64_t read_index;
} knhk_assertion_ring_t;
```

## Operations

### Commit

```c
int knhk_ring_commit_assertion(knhk_assertion_ring_t *ring, uint64_t tick, knhk_assertion_t *assertion) {
    // Generate receipt
    // Commit assertion to tick slot
    // Atomic index update
}
```

### Read

```c
int knhk_ring_read_assertion(knhk_assertion_ring_t *ring, uint64_t tick, knhk_assertion_t *assertion) {
    // Read assertion from tick slot
    // Atomic index read
    // SoA to RawTriple conversion
}
```

## Receipt Generation

Receipts are generated for all assertions:

```rust
let receipt = generate_receipt(assertion)?;
ring.commit_assertion(assertion, receipt, tick)?;
```

## Related Documentation

- [Ring Buffers](../ring-buffers.md) - Overview
- [SoA Layout](soa-layout.md) - Memory layout
- [Delta Ring](delta-ring.md) - Input buffer
- [Lockchain](../../integration/lockchain.md) - Receipt storage
