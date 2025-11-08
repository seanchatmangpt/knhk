# knhk-etl

ETL pipeline framework for RDF processing with Chicago TDD methodology.

## Features

- ✅ **Chicago TDD** (London School) for behavior-focused testing
- ✅ **Buffer pooling** for zero-allocation hot path
- ✅ **Branchless validation** from simdjson lessons
- ✅ **Connector integration** (Kafka, Salesforce, custom)
- ✅ **Lockchain verification** for distributed consensus
- ✅ **OpenTelemetry instrumentation** with Weaver validation

## Usage

```rust
use knhk_etl::{Pipeline, BufferPool, PredRun};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pool = BufferPool::new(8192);
    let pipeline = Pipeline::new(&mut pool)?;

    // Process RDF data through ETL pipeline
    pipeline.process(&data)?;

    Ok(())
}
```

## Architecture

### Buffer Pooling

```rust
let mut pool = BufferPool::new(8192);
let soa = pool.get_soa(1024)?;  // Reuses existing buffers
// ... use soa ...
pool.return_soa(soa);  // Return to pool
```

### Branchless Validation

```rust
use knhk_etl::guard_validation::*;

let valid = validate_all_guards_branchless(&run, tick_budget, capacity);
if valid != 0 {
    // Process run
}
```

## Performance

- Hot path latency: ≤8 ticks (Chatman Constant)
- Buffer pool hit rate: >95%
- Zero allocations in hot path
- SIMD-optimized operations

## License

Licensed under MIT license.
