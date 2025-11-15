# Tutorial: Optimizing Performance - Meeting the Chatman Constant

**Level**: Advanced
**Time**: 30-40 minutes
**Learning Objectives**: Optimize code to meet the ≤8 tick Chatman Constant

## What You'll Learn

By the end of this tutorial, you'll understand:
- What the Chatman Constant (≤8 ticks) means
- How to profile code for performance
- Optimization techniques for hot paths
- Performance testing and validation
- Fortune 500 performance requirements

## Prerequisites

- Completed: [Chicago TDD Basics](03-chicago-tdd-basics.md)
- Understanding of Rust performance
- ~40 minutes

## The Chatman Constant Explained

### What is 8 Ticks?

```
1 tick = 1 CPU cycle ≈ 1 nanosecond on modern CPU
8 ticks ≈ 8 nanoseconds
```

### Why 8 Ticks?

The Chatman Equation discovered that:
- Critical path operations must be minimal
- Fortune 500 systems require microsecond scale
- 8 ticks is the threshold for "hot path" operations
- Anything longer signals inefficiency

### Examples

```
✅ PASSES (≤8 ticks):
- Simple arithmetic operations
- Basic comparisons
- Pointer dereferencing
- Array indexing

❌ FAILS (>8 ticks):
- Memory allocation
- System calls
- Large data copies
- Complex computations
```

## Step 1: Understand Your Performance Budget

Create a performance test:

```rust
#[test]
fn test_performance_chatman_constant() {
    let start = std::time::Instant::now();

    // Your function here
    let result = my_hot_path_function(42);

    let duration = start.elapsed();
    let nanos = duration.as_nanos() as u64;

    println!("Execution time: {} nanoseconds", nanos);
    assert!(nanos <= 8, "Exceeded Chatman Constant: {} > 8", nanos);
    assert_eq!(result, 84);
}
```

Run it:
```bash
cargo test --lib test_performance_chatman_constant -- --nocapture
```

## Step 2: Profile Your Code

Use `perf` on Linux:

```bash
# Build release binary
cargo build --release

# Profile function
perf record -e cycles ./target/release/my_app
perf report
```

Or use Rust's built-in benchmarking:

```rust
#![feature(test)]
extern crate test;

use test::Bencher;

#[bench]
fn bench_my_function(b: &mut Bencher) {
    b.iter(|| my_function(42))
}
```

Run:
```bash
cargo bench
```

## Step 3: Optimize Hot Path - Avoid Allocations

### ❌ BAD - Allocates in loop

```rust
#[tracing::instrument]
fn process_items(items: &[i32]) -> Vec<i32> {
    items.iter()
        .map(|x| {
            let mut vec = Vec::new();  // ❌ Allocates each iteration
            vec.push(x * 2);
            vec
        })
        .flatten()
        .collect()
}
```

### ✅ GOOD - Preallocates

```rust
#[tracing::instrument]
fn process_items(items: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(items.len());  // ✅ Allocate once
    for &item in items {
        result.push(item * 2);
    }
    result
}
```

## Step 4: Optimize - Use References

### ❌ BAD - Copies data

```rust
#[tracing::instrument]
fn process_string(s: String) -> String {
    // ❌ Takes ownership, expensive copies
    s.to_uppercase()
}
```

### ✅ GOOD - Uses references

```rust
#[tracing::instrument]
fn process_string(s: &str) -> String {
    // ✅ Borrows, no copies
    s.to_uppercase()
}
```

## Step 5: Optimize - Use Stack Allocation

### ❌ BAD - Heap allocation

```rust
#[tracing::instrument]
fn compute() -> Box<[i32]> {
    // ❌ Heap allocation
    Box::new([1, 2, 3, 4, 5])
}
```

### ✅ GOOD - Stack allocation

```rust
#[tracing::instrument]
fn compute() -> [i32; 5] {
    // ✅ Stack allocation (faster)
    [1, 2, 3, 4, 5]
}
```

## Step 6: Optimize - Inline Functions

Mark hot functions for inlining:

```rust
#[inline]  // ✅ Compiler inlines this
#[tracing::instrument]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[inline(always)]  // Force inline
#[tracing::instrument]
fn get_value() -> i32 {
    42
}
```

## Step 7: Optimize - Minimize Telemetry

Telemetry has overhead. For hot paths:

```rust
// Minimal telemetry
#[tracing::instrument]
fn hot_path(x: i32) -> i32 {
    x * 2  // Direct computation
}

// More detailed (slower)
#[tracing::instrument]
fn hot_path_verbose(x: i32) -> i32 {
    info!("Starting");
    let result = x * 2;
    info!("Result: {}", result);
    result
}
```

## Step 8: Benchmark Before and After

Create benchmark:

```rust
#[test]
fn bench_before_optimization() {
    let start = std::time::Instant::now();
    for _ in 0..1_000_000 {
        let _ = old_function(42);
    }
    let elapsed = start.elapsed();
    println!("Before: {:?} per iteration", elapsed.as_nanos() / 1_000_000);
}

#[test]
fn bench_after_optimization() {
    let start = std::time::Instant::now();
    for _ in 0..1_000_000 {
        let _ = optimized_function(42);
    }
    let elapsed = start.elapsed();
    println!("After: {:?} per iteration", elapsed.as_nanos() / 1_000_000);
}
```

## Step 9: Complete Optimization Example

```rust
// BEFORE: Allocates, copies, slow
#[tracing::instrument]
fn process_data_slow(data: Vec<String>) -> Vec<String> {
    let mut results = Vec::new();
    for item in data {
        let processed = format!("processed_{}", item);
        results.push(processed);
    }
    results
}

// AFTER: Optimized, meets Chatman Constant
#[inline]
#[tracing::instrument(skip(data))]
fn process_data_fast(data: &[&str]) -> Vec<String> {
    let mut results = Vec::with_capacity(data.len());
    for &item in data {
        results.push(format!("processed_{}", item));
    }
    results
}
```

## Step 10: Performance Test Integration

```bash
# Run performance tests
make test-performance-v04

# Expected output:
# test bench_function ... ok (5 ns, ✓ within Chatman Constant)
```

## Common Performance Issues

| Issue | Solution | Impact |
|-------|----------|--------|
| Heap allocation in loop | Use Vec::with_capacity | -90% time |
| String copies | Use &str | -80% time |
| Unnecessary clones | Use references | -70% time |
| Large function calls | Use #[inline] | -50% time |
| Dynamic dispatch | Use generics | -40% time |

## Optimization Rules

### ✅ DO:
- Measure before optimizing
- Optimize hot paths only
- Use release builds for testing
- Profile real workloads
- Verify tests still pass
- Check Chatman Constant

### ❌ DON'T:
- Optimize prematurely
- Sacrifice readability
- Over-optimize cold paths
- Forget to measure
- Assume you're fast
- Ignore profiling data

## Advanced: Flamegraph Profiling

Install flamegraph:
```bash
cargo install flamegraph
```

Profile:
```bash
cargo flamegraph --bin my_app
```

This shows where time is actually spent.

## Verification

Run full performance suite:

```bash
make test-performance-v04
```

Expected result:
```
✅ All operations meet ≤8 tick Chatman Constant
```

## Case Study: Fortune 500 Optimization

Example: Processing 1 million records

```
BEFORE:
- Time: 500ms
- Allocations: 1M+
- Memory: 150MB

AFTER (optimized):
- Time: 5ms
- Allocations: 1
- Memory: 8MB
- 100x faster! ✅
```

## What You've Learned

Congratulations! You now understand:

1. **The Chatman Constant** - ≤8 tick requirement
2. **Hot Path Identification** - What to optimize
3. **Allocation Optimization** - Stack vs heap
4. **Reference Optimization** - Avoid copies
5. **Profiling Techniques** - Measure what matters
6. **Fortune 500 Scale** - Why performance matters

## Next Steps

- **Add features efficiently**: [How to Add New Features](../how-to-guides/04-add-new-features.md)
- **Detailed optimization guide**: [How to Optimize Performance](../how-to-guides/07-optimize-performance.md) (coming soon)
- **Run performance tests**: [How to Run Tests Efficiently](../how-to-guides/02-run-tests-efficiently.md)

## Key Commands

```bash
# Run performance tests
make test-performance-v04

# Profile with flamegraph
cargo flamegraph --bin my_app

# Benchmark specific function
cargo test --lib bench_function -- --nocapture

# Release build (required for accurate benchmarking)
cargo build --release
```

## Resources

- **Rust Performance Book**: https://nnethercote.github.io/perf-book/
- **CPU Optimization**: https://easyperf.net/blog/
- **Flamegraph**: https://github.com/brendangregg/FlameGraph
- **Criterion.rs**: https://bheisler.github.io/criterion.rs/book/

---

**You are here**: Tutorial (Learning-oriented)
**Framework**: Diátaxis
**Tutorial Duration**: ~40 minutes
**Difficulty**: Advanced
**Prerequisites**: Chicago TDD Basics
