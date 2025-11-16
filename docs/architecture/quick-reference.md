# KNHK Workflow Engine 2027 - Quick Reference Guide

**Version:** 1.0.0
**Last Updated:** 2025-11-16

This quick reference provides at-a-glance information for developers working on the KNHK workflow engine.

---

## Extended SPARC Phases Overview

| Phase | Name | Duration | Key Deliverable |
|-------|------|----------|-----------------|
| 0 | Async/Await Mastery | 4 weeks | Work-stealing scheduler |
| 1 | Type-System Mastery | 4 weeks | GAT pattern traits |
| 2 | Memory Optimization | 3 weeks | ≤8 tick hot path |
| 3 | MI Execution | 4 weeks | Patterns 12-15 complete |
| 4 | Connector Framework | 3 weeks | Plugin system |
| 5 | Specification | 2 weeks | Requirements docs |
| 6 | Pseudocode | 1 week | Algorithm design |
| 7 | Architecture | 2 weeks | System design |
| 8 | Refinement/TDD | 6 weeks | All 43 patterns |
| 9 | Completion | 2 weeks | Production ready |
| 10 | Error Handling | 4 weeks | Advanced errors |

**Total Timeline:** 22 weeks (5.5 months)

---

## Performance Targets

| Metric | Target | Validation Method |
|--------|--------|-------------------|
| Hot Path Execution | ≤8 ticks | Benchmark suite |
| CPU Utilization | >95% | Load testing |
| Task Spawn Latency | <100ns | Criterion benchmarks |
| Memory (10K cases) | <100MB | Memory profiler |
| SIMD Speedup | 4-8x | SIMD benchmarks |
| Allocations (hot path) | 0 | Allocation profiler |

---

## Key Rust Features by Phase

### Phase 0: Async/Await
```rust
// Async trait methods
trait AsyncPattern {
    async fn execute(&self, ctx: &Context) -> Result<Output>;
}

// Cancellation tokens
let token = CancellationToken::new();
select! {
    result = work() => result,
    _ = token.cancelled() => Err(Cancelled),
}

// Work-stealing
let scheduler = WorkStealingScheduler::new(num_cpus());
scheduler.spawn(async { /* task */ });
```

### Phase 1: Type-System
```rust
// GATs (Generic Associated Types)
trait Pattern {
    type ExecuteFuture<'a>: Future<Output = Result<Output>> + 'a
    where Self: 'a;

    fn execute<'a>(&'a self, ctx: &'a Context) -> Self::ExecuteFuture<'a>;
}

// HRTBs (Higher-Ranked Trait Bounds)
type Executor = Box<dyn for<'a> Fn(&'a Context) -> BoxFuture<'a, Result<Output>>>;

// Type-state pattern
struct Workflow<State> {
    state: PhantomData<State>,
}

impl Workflow<Created> {
    fn validate(self) -> Workflow<Validated> { /* ... */ }
}

// Newtype pattern
#[repr(transparent)]
struct CaseId(Uuid);
```

### Phase 2: Memory Optimization
```rust
// Custom allocator
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Arena allocation
let arena = Bump::new();
let data = arena.alloc_slice_fill_default(1000);

// Memory mapping
let mmap = unsafe { Mmap::map(&file)? };

// SIMD
#[cfg(target_arch = "x86_64")]
unsafe {
    let data = _mm256_loadu_si256(ptr);
    let result = _mm256_add_epi32(data, increment);
}

// Cache-line alignment
#[repr(align(64))]
struct HotData {
    counter: AtomicU64,
    _padding: [u8; 56],
}

// Lazy initialization
static REGISTRY: OnceLock<PatternRegistry> = OnceLock::new();
```

### Phase 3: MI Execution
```rust
// Work-stealing MI
let handles: Vec<_> = (0..count)
    .map(|i| scheduler.spawn_cpu_bound(instance(i)))
    .collect();

futures::future::join_all(handles).await

// Rayon data parallelism
instances.par_iter().map(|i| validate(i)).collect()

// Correlation tracking
let tracker = CorrelationTracker::new();
tracker.record_parent_child(parent_id, child_id);
```

### Phase 4: Connectors
```rust
// GAT connector trait
trait Connector {
    type Input;
    type Output;
    type Future<'a>: Future<Output = Result<Output>> + 'a where Self: 'a;

    fn execute<'a>(&'a self, input: &'a Input) -> Self::Future<'a>;
}

// Plugin loading
let lib = unsafe { Library::new("connector.so")? };
let constructor: Symbol<fn() -> Box<dyn Connector>> =
    unsafe { lib.get(b"_plugin_create")? };

// Circuit breaker
let breaker = CircuitBreaker::new(threshold, timeout);
breaker.call(|| connector.execute(input)).await?
```

### Phase 10: Error Handling
```rust
// thiserror errors
#[derive(thiserror::Error, Debug)]
enum WorkflowError {
    #[error("Pattern {pattern_id} failed")]
    PatternFailed {
        pattern_id: PatternId,
        #[source]
        source: Box<dyn Error>,
        #[from]
        backtrace: Backtrace,
    },
}

// Error context
error.add_context("case_id", case_id)
     .add_context("workflow_spec", spec_id)

// Recovery strategies
trait ErrorRecovery {
    async fn recover(&self, error: Error) -> Result<RecoveryAction>;
}
```

---

## Concurrency Model Cheat Sheet

| Task Type | Executor | Use Case |
|-----------|----------|----------|
| CPU-bound | Work-stealing | Pattern execution, validation |
| I/O-bound | Tokio | Connectors, database, network |
| Data parallel | Rayon | Batch processing, SIMD |
| Event-driven | Tokio channels | State transitions, triggers |

**Decision Tree:**
```
Is the task CPU-intensive?
├─ Yes → Does it need async?
│  ├─ Yes → Work-stealing scheduler
│  └─ No → Rayon
└─ No → Tokio runtime
```

---

## Pattern Implementation Template

```rust
use knhk_workflow_engine::patterns::Pattern;

pub struct MyPattern {
    config: MyConfig,
}

#[derive(Debug, Deserialize)]
pub struct MyConfig {
    // Configuration fields
}

pub struct MyState {
    // Runtime state
}

pub struct MyOutput {
    // Execution result
}

#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Something failed")]
    Failed(#[from] Box<dyn Error>),
}

impl Pattern for MyPattern {
    type Config = MyConfig;
    type State = MyState;
    type Output = MyOutput;
    type Error = MyError;

    type ExecuteFuture<'a> = impl Future<Output = Result<Self::Output, Self::Error>> + Send + 'a
    where
        Self: 'a;

    fn execute<'a>(&'a self, state: &'a Self::State) -> Self::ExecuteFuture<'a> {
        async move {
            // Implementation
            Ok(MyOutput {})
        }
    }

    fn validate_config(config: &Self::Config) -> Result<(), Self::Error> {
        // Validation logic
        Ok(())
    }
}

// Chicago TDD test
#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::*;

    #[tokio::test]
    async fn test_my_pattern_success() {
        // Arrange
        let config = MyConfig { /* ... */ };
        let pattern = MyPattern { config };
        let state = MyState { /* ... */ };

        // Act
        let result = pattern.execute(&state).await;

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.field, expected_value);
    }

    #[tokio::test]
    async fn test_my_pattern_failure() {
        // Test error conditions
    }
}
```

---

## Build Commands

```bash
# Full build with all features
cargo build --workspace --all-features --release

# Build with specific feature set
cargo build --features "async-core,type-safety,memory-opt"

# Run tests
cargo test --workspace
cargo test --test chicago_tdd_all_patterns

# Performance tests
cargo test --test performance_hotpath -- --ignored
make test-performance-v04

# Clippy (strict)
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Weaver validation
weaver registry check -r registry/
weaver registry live-check --registry registry/

# Benchmarks
cargo bench --bench fortune5_performance
```

---

## Feature Flags Quick Reference

```toml
# Minimal build
cargo build --no-default-features --features "minimal"

# Performance build
cargo build --features "performance"

# 2027 full feature set
cargo build --features "full-2027"

# Development (all features + debug)
cargo build --all-features

# Production (optimized)
cargo build --release --features "default"
```

**Available Features:**
- `async-core` - Async runtime and cancellation
- `work-stealing` - Custom scheduler
- `type-safety` - GAT patterns
- `memory-opt` - Custom allocators
- `simd` - SIMD optimizations
- `mi-patterns` - Multiple instance patterns
- `connectors` - Plugin framework
- `otel` - Observability
- `weaver-validation` - Weaver integration
- `grpc-api` - gRPC server
- `rest-api` - REST server

---

## Common Tasks

### Add a New Pattern

1. **Create pattern module** (`src/patterns/my_pattern.rs`)
2. **Implement `Pattern` trait** (see template above)
3. **Write Chicago TDD tests** (`tests/patterns/test_my_pattern.rs`)
4. **Register in registry** (`src/patterns/mod.rs`)
5. **Add Weaver schema** (`registry/patterns/my_pattern.yaml`)
6. **Validate with Weaver** (`weaver registry check`)

### Add a New Connector

1. **Implement `Connector` trait**
2. **Create plugin export** (`#[no_mangle] pub fn _plugin_create()`)
3. **Build as dynamic library** (`crate-type = ["cdylib"]`)
4. **Test loading** (`load_connector("target/release/libmy_connector.so")`)
5. **Add to registry**

### Debug Performance Issues

1. **Profile hot path**
   ```bash
   cargo flamegraph --bin knhk-workflow
   ```

2. **Check tick count**
   ```bash
   cargo test --test performance_hotpath -- --nocapture
   ```

3. **Memory profiling**
   ```bash
   valgrind --tool=massif target/release/knhk-workflow
   ```

4. **SIMD verification**
   ```bash
   cargo asm knhk_workflow_engine::validate_instances_simd
   ```

---

## Troubleshooting

### Compilation Errors

**Error:** `associated type bounds are unstable`
```toml
# Cargo.toml - ensure Rust 1.65+
[package]
rust-version = "1.65"
```

**Error:** `impl Trait` not allowed in type aliases
```rust
// Use explicit Future bounds instead
type Fut<'a> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
```

### Runtime Issues

**Issue:** Low CPU utilization
```rust
// Check task classification
debug_assert!(task.is_cpu_bound());

// Verify work-stealing is used
scheduler.stats().cpu_bound_tasks > 0
```

**Issue:** High allocation rate
```rust
// Use arena allocator for batch operations
let arena = Bump::new();
let batch = arena.alloc_slice_fill_default(count);
```

### Weaver Validation Failures

**Issue:** Span not found in schema
```yaml
# Add to registry/workflow-engine.yaml
spans:
  - id: my.new.span
    brief: Description
    attributes:
      - ref: workflow.id
```

**Issue:** Attribute type mismatch
```rust
// Ensure OTEL attribute types match schema
tracing::info!(
    workflow.id = %id,  // String (%formatting)
    instance.count = count,  // Int (no formatting)
);
```

---

## Resources

### Documentation
- **Architecture:** `/docs/architecture/2027-workflow-engine-architecture.md`
- **ADRs:** `/docs/architecture/ADR-*.md`
- **API Docs:** `cargo doc --open`

### Examples
- **Pattern Implementation:** `examples/custom_pattern.rs`
- **Connector Plugin:** `examples/http_connector/`
- **Weaver Validation:** `examples/workflow_weaver_livecheck.rs`

### External References
- **Rust GATs:** https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
- **Tokio Docs:** https://docs.rs/tokio/
- **Rayon Docs:** https://docs.rs/rayon/
- **Weaver:** https://github.com/open-telemetry/weaver

---

## Contact

**Architecture Questions:** System Architect
**Implementation Questions:** Tech Lead
**Performance Questions:** Performance Engineer
**Weaver Questions:** Observability Team

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-16 | Initial architecture |

---

## Checklist for New Developers

- [ ] Read main architecture document
- [ ] Review ADR-001 (Work-Stealing)
- [ ] Review ADR-002 (GAT Patterns)
- [ ] Set up development environment
- [ ] Run full test suite
- [ ] Implement a basic pattern (training)
- [ ] Review existing pattern implementations
- [ ] Understand Weaver validation workflow
- [ ] Pair with senior developer on first task

Welcome to KNHK Workflow Engine! 🚀
