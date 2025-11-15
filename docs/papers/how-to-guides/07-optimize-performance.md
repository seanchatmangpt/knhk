# How-to Guide: Optimize Performance

**Goal**: Improve code performance to meet Chatman Constant requirements
**Time**: 20-30 minutes per optimization
**Difficulty**: Intermediate

## Quick Checklist

Before optimizing:
- [ ] Function is in hot path (frequently called)
- [ ] Current performance is measured
- [ ] Tests exist and pass
- [ ] You have permission to optimize

## Step 1: Identify Hot Paths

Functions to optimize:
- Called millions of times
- Used in critical path
- Directly affects latency
- Already causes problems

Find with profiling:
```bash
cargo flamegraph --bin my_app
# Look for functions using most CPU time
```

## Step 2: Establish Baseline

Measure current performance:

```rust
#[test]
fn benchmark_function_baseline() {
    let start = std::time::Instant::now();

    for _ in 0..1_000_000 {
        my_function(42);
    }

    let elapsed = start.elapsed();
    println!("Time per call: {} ns", elapsed.as_nanos() / 1_000_000);
}
```

Run and record result:
```bash
cargo test --lib benchmark_function_baseline -- --nocapture --test-threads=1
```

Example output:
```
Time per call: 45 ns
Target: ≤8 ns (Chatman Constant)
Optimization needed: ~82%
```

## Step 3: Eliminate Allocations

### Pattern: Reuse Vectors

❌ **BEFORE** - Creates new Vec each call
```rust
#[tracing::instrument]
fn process(items: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();  // ❌ Allocates each time
    for &item in items {
        result.push(item * 2);
    }
    result
}
```

✅ **AFTER** - Preallocate with capacity
```rust
#[tracing::instrument]
fn process(items: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(items.len());  // ✅ Allocate once
    for &item in items {
        result.push(item * 2);
    }
    result
}
```

**Impact**: -40% time

### Pattern: Use References

❌ **BEFORE** - Clones data
```rust
#[tracing::instrument]
fn parse_data(data: Vec<u8>) -> String {
    // ❌ Takes ownership, expensive clone
    String::from_utf8_lossy(&data).to_string()
}
```

✅ **AFTER** - Borrows data
```rust
#[tracing::instrument]
fn parse_data(data: &[u8]) -> Cow<str> {
    // ✅ Borrows, no allocation
    String::from_utf8_lossy(data)
}
```

**Impact**: -60% time

## Step 4: Inline Functions

Mark hot functions for inlining:

```rust
#[inline]  // Compiler decides
#[tracing::instrument]
fn simple_add(a: i32, b: i32) -> i32 {
    a + b
}

#[inline(always)]  // Force inline
fn get_constant() -> i32 {
    42
}

#[inline(never)]  // Never inline
fn cold_function() {
    // For debugging
}
```

**Impact**: -20% time

## Step 5: Use Iterator Chains

❌ **BEFORE** - Intermediate Vec
```rust
#[tracing::instrument]
fn double_evens(nums: &[i32]) -> Vec<i32> {
    let evens = nums.iter()
        .filter(|&x| x % 2 == 0)
        .collect::<Vec<_>>();  // ❌ Intermediate allocation

    evens.iter()
        .map(|&x| x * 2)
        .collect()
}
```

✅ **AFTER** - Single chain
```rust
#[tracing::instrument]
fn double_evens(nums: &[i32]) -> Vec<i32> {
    nums.iter()
        .filter(|&x| x % 2 == 0)
        .map(|&x| x * 2)
        .collect()  // ✅ Single allocation
}
```

**Impact**: -30% time

## Step 6: Reduce Telemetry Overhead

❌ **BEFORE** - Heavy logging in hot path
```rust
#[tracing::instrument]
fn hot_path(x: i32) -> i32 {
    info!("Called with: {}", x);
    debug!("About to compute");
    let result = expensive_compute(x);
    info!("Result: {}", result);
    debug!("Returning");
    result
}
```

✅ **AFTER** - Minimal telemetry
```rust
#[tracing::instrument]
fn hot_path(x: i32) -> i32 {
    expensive_compute(x)
    // Telemetry captured by #[instrument] only
}
```

**Impact**: -50% time

## Step 7: Use Stack Allocation

❌ **BEFORE** - Heap allocation
```rust
#[tracing::instrument]
fn process() -> Box<[i32; 100]> {
    Box::new([0; 100])  // ❌ Heap
}
```

✅ **AFTER** - Stack allocation
```rust
#[tracing::instrument]
fn process() -> [i32; 100] {
    [0; 100]  // ✅ Stack (faster)
}
```

**Impact**: -70% time

## Step 8: Avoid Dynamic Dispatch

❌ **BEFORE** - Trait object (dynamic)
```rust
#[tracing::instrument]
fn process(processor: &dyn Processor) -> Result<()> {
    processor.process()  // ❌ Dynamic dispatch
}
```

✅ **AFTER** - Generics (static)
```rust
#[tracing::instrument]
fn process<T: Processor>(processor: &T) -> Result<()> {
    processor.process()  // ✅ Static dispatch (inlined)
}
```

**Impact**: -40% time

## Step 9: Cache Computations

```rust
#[tracing::instrument]
fn with_caching(x: i32) -> i32 {
    static CACHE: once_cell::sync::Lazy<std::collections::HashMap<i32, i32>> =
        once_cell::sync::Lazy::new(|| {
            let mut m = std::collections::HashMap::new();
            // Precompute expensive values
            m.insert(0, compute(0));
            m.insert(1, compute(1));
            m
        });

    *CACHE.get(&x).unwrap_or(&compute(x))
}
```

**Impact**: -90% time (for cached values)

## Step 10: Benchmark After Optimization

```bash
cargo test --lib benchmark_function_baseline -- --nocapture --test-threads=1
```

Compare:
```
BEFORE: 45 ns per call
AFTER:  6 ns per call
Improvement: 87% ✅ Meets Chatman Constant!
```

## Complete Example

Complete optimization case:

```rust
// ❌ BEFORE: 150 ns per call
#[tracing::instrument]
fn process_slow(items: Vec<String>) -> Vec<String> {
    info!("Processing {} items", items.len());

    let mut results = Vec::new();
    for item in items {
        debug!("Processing: {}", item);
        let result = format!("processed_{}", item);
        info!("Processed: {}", result);
        results.push(result);
    }

    info!("Done");
    results
}

// ✅ AFTER: 5 ns per call
#[inline]
#[tracing::instrument(skip(items))]
fn process_fast(items: &[&str]) -> Vec<String> {
    let mut results = Vec::with_capacity(items.len());
    for &item in items {
        results.push(format!("processed_{}", item));
    }
    results
}
```

## Performance Testing

Verify optimization with tests:

```bash
# Run performance tests
make test-performance-v04

# Run Chicago TDD tests
make test-chicago-v04

# Both must pass after optimization
```

## Optimization Rules

| Rule | Time Saved | Complexity |
|------|------------|-----------|
| Eliminate allocations | -40% | Low |
| Use references | -60% | Low |
| Inline functions | -20% | Low |
| Iterator chains | -30% | Medium |
| Stack allocation | -70% | Medium |
| Reduce telemetry | -50% | Low |
| Generic dispatch | -40% | Medium |
| Caching | -90% | High |

## Decision Tree

```
Is function in hot path?
├─ NO → Don't optimize (premature optimization)
└─ YES → Measure baseline
    ├─ <8 ns? → Done ✅
    └─ >8 ns? → Eliminate allocations
        ├─ Still >8 ns? → Use references
        ├─ Still >8 ns? → Inline
        ├─ Still >8 ns? → Stack allocation
        └─ Still >8 ns? → Cache/dynamic dispatch
```

## When NOT to Optimize

❌ **DON'T optimize:**
- Functions called rarely (<1000x)
- Non-critical paths
- If it makes code unreadable
- If tests don't prove it works
- Without measurements

✅ **DO optimize:**
- Hot paths (millions of calls)
- Critical operations
- When measurements show need
- When tests verify correctness

## Next Steps

- **Validate optimization**: Run `make test-performance-v04`
- **Learn more**: [Optimizing Performance](../tutorials/04-optimizing-performance.md)
- **Profile code**: Use `cargo flamegraph`

## Key Commands

```bash
# Run performance tests
make test-performance-v04

# Profile with flamegraph
cargo flamegraph --bin my_app

# Benchmark
cargo test --lib bench -- --nocapture

# Release build (for benchmarking)
cargo build --release
```

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Intermediate
**Related**: Chicago TDD, Performance Optimization Tutorial
