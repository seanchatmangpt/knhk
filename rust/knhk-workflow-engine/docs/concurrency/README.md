# Phase 0: Async/Await Mastery - Concurrency Primitives

Foundation for high-performance async workflow execution in KNHK.

## Quick Start

### Enable the Feature

```toml
[dependencies]
knhk-workflow-engine = { version = "1.0", features = ["async-v2"] }
```

### Basic Usage

#### Nursery - Structured Concurrency

```rust
use knhk_workflow_engine::concurrency::Nursery;

async fn example() {
    let mut nursery = Nursery::new();

    // Spawn tasks
    nursery.spawn(async { println!("Task 1"); Ok(()) }).await;
    nursery.spawn(async { println!("Task 2"); Ok(()) }).await;

    // Wait for all to complete
    nursery.wait_all().await?;
}
```

#### CancelToken - Cooperative Cancellation

```rust
use knhk_workflow_engine::concurrency::CancelToken;

async fn example() {
    let token = CancelToken::new();
    let child = token.child_token();

    tokio::spawn({
        let token = child.clone();
        async move {
            tokio::select! {
                _ = token.cancelled() => println!("Cancelled"),
                _ = do_work() => println!("Completed"),
            }
        }
    });

    // Cancel all children
    token.cancel();
}
```

#### Work-Stealing Executor

```rust
use knhk_workflow_engine::concurrency::WorkStealingExecutor;

async fn example() {
    let executor = WorkStealingExecutor::new(4);

    for i in 0..1000 {
        executor.spawn(async move {
            compute(i);
        });
    }

    executor.shutdown().await;
}
```

## Components

| Component | Purpose | Performance |
|-----------|---------|-------------|
| **Nursery** | Structured concurrency | RAII cleanup |
| **CancelToken** | Hierarchical cancellation | Zero-cost abstraction |
| **WorkStealingExecutor** | High-performance scheduling | <100ns spawn latency |
| **PinExt** | Pin/Unpin utilities | Compile-time |

## Testing

```bash
# Run all concurrency tests
cargo test --features async-v2 concurrency

# Run benchmarks
cargo bench --features async-v2 --bench concurrency_benchmarks

# Check compilation
cargo check --features async-v2
```

## Performance Targets

| Metric | Target | Validation |
|--------|--------|------------|
| Task Spawn Latency (P99) | <100ns | Criterion benchmark |
| CPU Utilization | >95% | System monitoring |
| Throughput | >1M tasks/sec | Benchmark suite |

## Documentation

- [Implementation Report](./PHASE_0_IMPLEMENTATION.md) - Complete implementation details
- [Weaver Schema](../../registry/concurrency.yaml) - OTEL telemetry schema
- Tests: `tests/concurrency/` - 72 Chicago TDD tests
- Benchmarks: `benches/concurrency_benchmarks.rs`

## Architecture

```
Phase 0: Async/Await Mastery
│
├── Nursery (Structured Concurrency)
│   ├── Spawn tasks in scope
│   ├── wait_all() - Wait for all tasks
│   ├── wait_any() - Wait for first task
│   └── RAII cleanup with NurseryScope
│
├── CancelToken (Cooperative Cancellation)
│   ├── Hierarchical parent/child tokens
│   ├── Async .cancelled() future
│   └── RAII cleanup with CancelScope
│
├── WorkStealingExecutor (High-Performance Scheduler)
│   ├── Per-worker local queues
│   ├── Global injector queue
│   ├── Random work stealing
│   └── Metrics tracking
│
└── Pin Utilities
    ├── PinExt trait
    ├── Helper functions
    └── Convenience macros
```

## Next Steps

1. **Pattern Migration**: Convert patterns to `AsyncPatternExecutor`
2. **Executor Integration**: Use work-stealing for MI patterns (12-15)
3. **Cancellation Integration**: Use CancelToken for patterns 19-25
4. **Instrumentation**: Add OTEL telemetry matching schema

## Examples

See `tests/concurrency/` for comprehensive examples of all features.

---

**Status**: ✅ Complete
**Tests**: 72 Chicago TDD tests
**Benchmarks**: <100ns spawn latency target
**Documentation**: Full implementation report available
