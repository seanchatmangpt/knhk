# FFI Integration

FFI (Foreign Function Interface) integration bridges the C hot path and Rust ETL layers.

## Overview

The FFI layer provides:
- C function bindings
- Type conversions
- Memory management
- Error handling

## Key Components

### Beat Scheduler FFI

```rust
use knhk_hot::beat_ffi;

let cycle = beat_ffi::knhk_beat_next();
let tick = beat_ffi::knhk_beat_tick(cycle);
let pulse = beat_ffi::knhk_beat_pulse(cycle);
```

### Ring Buffer FFI

```rust
use knhk_hot::ring_ffi;

ring_ffi::knhk_ring_enqueue_delta(&mut ring, tick, &delta)?;
let delta = ring_ffi::knhk_ring_dequeue_delta(&ring, tick)?;
```

### Fiber Execution FFI

```rust
use knhk_hot::fiber_ffi;

let result = fiber_ffi::knhk_fiber_execute(&mut fiber, tick)?;
fiber_ffi::knhk_fiber_park(&mut fiber, &work)?;
```

## Type Conversions

### Receipt Conversion

```rust
use knhk_hot::receipt_convert;

let c_receipt = receipt_convert::rust_to_c_receipt(rust_receipt)?;
let rust_receipt = receipt_convert::c_to_rust_receipt(c_receipt)?;
```

### SoA Conversion

```rust
use knhk_etl::ring_conversion;

let soa = ring_conversion::triples_to_soa(triples)?;
let triples = ring_conversion::soa_to_triples(&soa)?;
```

## Memory Management

### Safety

- All FFI calls are marked `unsafe`
- Memory is managed by Rust (RAII)
- C pointers are validated before use

### Error Handling

- C error codes converted to Rust `Result<T, E>`
- Proper error propagation
- Resource cleanup on errors

## Related Documentation

- [C API](c-api.md) - C hot path implementation
- [Rust API](rust-api.md) - Rust FFI bindings
- [8-Beat Integration](../../8BEAT-INTEGRATION-COMPLETE.md) - Integration details
