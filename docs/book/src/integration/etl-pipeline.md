# ETL Pipeline

The ETL pipeline provides data ingestion, transformation, and loading with 8-beat epoch integration.

## Overview

The ETL pipeline stages:
1. **Ingest**: Parse RDF/Turtle input
2. **Transform**: IRI hashing, schema validation
3. **Load**: SoA arrays, predicate runs
4. **Reflex**: Hook execution, receipt generation
5. **Emit**: Lockchain integration, downstream APIs

## Pipeline Stages

### Ingest Stage

```rust
use knhk_etl::ingest::Ingester;

let mut ingester = Ingester::new();
let triples = ingester.ingest(input)?;
```

### Transform Stage

```rust
use knhk_etl::transform::Transformer;

let transformer = Transformer::new();
let transformed = transformer.transform(triples)?;
```

### Load Stage

```rust
use knhk_etl::load::Loader;

let loader = Loader::new();
let soa = loader.load(transformed)?;
```

### Reflex Stage

```rust
use knhk_etl::reflex::Reflex;

let reflex = Reflex::new();
let receipts = reflex.execute(soa)?;
```

### Emit Stage

```rust
use knhk_etl::emit::Emitter;

let emitter = Emitter::new();
emitter.emit(receipts)?;
```

## Beat Integration

### Beat Scheduler

```rust
use knhk_etl::BeatScheduler;

let mut scheduler = BeatScheduler::new();
let (tick, pulse) = scheduler.advance_beat();
```

### Tick-Based Routing

Work is routed to tick slots based on the current beat cycle:
- Tick 0-7: Rotating slots
- Pulse detection: Every 8 ticks
- Commit boundary: On pulse

## Runtime Classes

### R1 (Hot Path)

- ≤8 ticks per operation
- Branchless operations
- SIMD-optimized

### W1 (Warm Path)

- More time budget
- Can break into smaller chunks
- Retry with exponential backoff

### C1 (Cold Path)

- Async operations
- Complex queries
- Batch processing

## Related Documentation

- [8-Beat System](../architecture/8beat-system.md) - Epoch system
- [Connectors](connectors.md) - Data source connectors
- [Lockchain](lockchain.md) - Provenance system
