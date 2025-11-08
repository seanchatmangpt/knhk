# Rust API

The Rust API provides high-level interfaces for KNHK operations.

## Overview

The Rust layer provides:
- ETL pipeline integration
- Beat scheduler management
- Fiber execution coordination
- Ring buffer conversion

## Core Modules

### Beat Scheduler

```rust
use knhk_etl::BeatScheduler;

let mut scheduler = BeatScheduler::new();
let (tick, pulse) = scheduler.advance_beat();
scheduler.enqueue_delta(triples, tick)?;
let assertions = scheduler.commit_cycle()?;
```

### Fiber Management

```rust
use knhk_etl::Fiber;

let fiber = Fiber::new(shard_id);
fiber.execute_tick(tick)?;
fiber.park(work)?;
```

### Ring Conversion

```rust
use knhk_etl::ring_conversion;

let soa = ring_conversion::triples_to_soa(triples)?;
let triples = ring_conversion::soa_to_triples(&soa)?;
```

## Crate Structure

### knhk-etl

ETL pipeline with 8-beat epoch integration:
- Beat scheduler
- Fiber management
- Ring conversion
- Park manager

### knhk-hot

C FFI bindings for hot path:
- Beat scheduler FFI
- Ring buffer FFI
- Fiber execution FFI
- Receipt conversion

### knhk-unrdf

Rust-native hooks engine:
- Hooks execution
- SPARQL queries
- RDF canonicalization
- Query caching

### knhk-sidecar

gRPC proxy service:
- Beat admission control
- Request batching
- Circuit breaker
- Retry logic

## Related Documentation

- [C API](c-api.md) - C hot path implementation
- [FFI Integration](ffi-integration.md) - FFI bridge
- [Integration](../integration/overview.md) - Integration guide
