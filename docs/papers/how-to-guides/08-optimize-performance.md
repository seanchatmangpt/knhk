# How-to Guide 8: Optimize Performance

## Goal

Identify, measure, and optimize code to meet KNHK's ≤8 tick performance constraint (Chatman Constant) while maintaining proper instrumentation.

**Time Estimate**: 2-3 hours
**Prerequisites**: [Run Tests Efficiently](02-run-tests-efficiently.md), [Emit Proper Telemetry](07-emit-proper-telemetry.md)
**Difficulty**: Advanced
**Outcomes**: Production code that provably meets performance constraints

---

## The Chatman Constant: ≤8 Ticks

### What is a "Tick"?

A **tick** is KNHK's unit of computational work. Understanding it is critical:

```
1 Tick ≈ 1 millisecond of typical CPU work
        (On a modern processor at standard speed)

≤8 Ticks = ≤8ms = Operation must complete very quickly

Why 8? The Chatman Equation determines:
  - Fortune 500 requirements
  - Modern user expectations
  - Network latency budgets
  - Distributed system requirements
```

### Where the Constraint Applies

```
API Request
  ├─ Network latency: 1-2 ticks (unavoidable)
  ├─ Deserialization: 0.5 tick
  ├─ Business logic: ≤4 ticks ← YOUR CODE HERE
  ├─ Database query: ≤2 ticks
  └─ Serialization: 0.5 tick
─────────────────────────
  Total: ≤10 ticks acceptable
  Comfortable: ≤8 ticks target
```

---

## Part 1: Measuring Performance

### Method 1: Built-in Test Validation

```bash
# Run performance tests
make test-performance-v04

# Output shows:
# operation_name: 5 ticks ✓ (under limit)
# expensive_op: 12 ticks ✗ (exceeds limit)
```

### Method 2: Manual Timing

```rust
#[test]
fn test_operation_performance() {
    let start = std::time::Instant::now();

    // Operation to measure
    let result = expensive_operation();

    let elapsed = start.elapsed();
    let ticks = elapsed.as_millis();

    println!("Operation took: {} ticks", ticks);
    assert!(ticks <= 8, "Operation exceeds ≤8 tick limit");
}
```

### Method 3: Flamegraph (Detailed Profiling)

```bash
# Install flamegraph
cargo install flamegraph

# Profile your code
cargo flamegraph --bin my_binary

# Generate interactive visualization
# Shows where CPU time is spent
```

### Method 4: Criterion.rs (Benchmarking)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_operation(c: &mut Criterion) {
    c.bench_function("operation", |b| {
        b.iter(|| expensive_operation(black_box(100)))
    });
}

criterion_group!(benches, benchmark_operation);
criterion_main!(benches);
```

Run with: `cargo bench`

---

## Part 2: Identifying Bottlenecks

### Step 1: Profile First

```bash
# Determine where time is actually spent
cargo flamegraph --bin my_binary -- --test slow_test

# Visual output shows:
# - 60% time in function_a
# - 30% time in function_b
# - 10% time in function_c
```

### Step 2: Measure Each Component

```rust
#[instrument]  // Automatically measures!
fn operation() {
    step_one();   // ~2 ticks
    step_two();   // ~3 ticks
    step_three(); // ~2 ticks
}
// Total: ~7 ticks (from span duration)
```

### Step 3: Find the Culprits

**Common performance issues**:

```rust
// ❌ SLOW: N² algorithm
fn is_duplicate(items: &[Item]) -> bool {
    for i in 0..items.len() {
        for j in i+1..items.len() {
            if items[i].id == items[j].id {
                return true;  // O(n²)
            }
        }
    }
    false
}

// ✅ FAST: Set lookup
fn is_duplicate(items: &[Item]) -> bool {
    let mut seen = std::collections::HashSet::new();
    for item in items {
        if !seen.insert(item.id) {
            return true;  // O(n)
        }
    }
    false
}
```

---

## Part 3: Optimization Strategies

### Strategy 1: Algorithm Optimization

**Common complexity improvements**:

```
O(n²) → O(n log n)   : Switch to sorted algorithm
O(n)  → O(log n)     : Use binary search
O(n²) → O(n)         : Use hash table instead of linear search
O(2ⁿ) → O(n)         : Use dynamic programming
```

**Example**:
```rust
// ❌ O(n²) - 50ms for 1000 items
fn find_matching_pairs(items: &[Item]) -> Vec<(Item, Item)> {
    let mut pairs = Vec::new();
    for i in 0..items.len() {
        for j in i+1..items.len() {
            if matches(&items[i], &items[j]) {
                pairs.push((items[i].clone(), items[j].clone()));
            }
        }
    }
    pairs
}

// ✅ O(n log n) - 1ms for 1000 items
fn find_matching_pairs(items: &[Item]) -> Vec<(Item, Item)> {
    let mut sorted = items.to_vec();
    sorted.sort_by_key(|item| item.category);

    let mut pairs = Vec::new();
    for window in sorted.windows(2) {
        if window[0].category == window[1].category {
            pairs.push((window[0].clone(), window[1].clone()));
        }
    }
    pairs
}
```

### Strategy 2: Data Structure Selection

```rust
// ❌ SLOW: Vec for lookups - O(n) per lookup
let mut users = Vec::new();  // Linear search needed

// ✅ FAST: HashMap for lookups - O(1) per lookup
let mut users = std::collections::HashMap::new();
users.insert(id, user);
let user = users.get(&id);  // Instant!

// ✅ FAST: BTreeMap when order matters - O(log n)
let mut users = std::collections::BTreeMap::new();
users.insert(id, user);
```

### Strategy 3: Avoid Allocations

```rust
// ❌ SLOW: Allocate strings in loop - 1000 allocations
for item in items {
    let formatted = format!("Item: {}", item.id);  // Allocate
    process(&formatted);
}

// ✅ FAST: Reuse buffer
let mut buffer = String::new();
for item in items {
    buffer.clear();
    write!(buffer, "Item: {}", item.id)?;  // No allocation
    process(&buffer);
}
```

### Strategy 4: Batch Operations

```rust
// ❌ SLOW: Individual database calls
for user_id in user_ids {
    let user = db.get_user(user_id).await?;  // 1000 DB calls
    process(user);
}

// ✅ FAST: Batch load
let users = db.get_users(&user_ids).await?;  // 1 DB call
for user in users {
    process(user);
}
```

### Strategy 5: Parallelization

```rust
// ❌ SLOW: Sequential processing
let results: Vec<_> = items
    .iter()
    .map(|item| expensive_compute(item))
    .collect();

// ✅ FAST: Parallel processing
use rayon::prelude::*;
let results: Vec<_> = items
    .par_iter()
    .map(|item| expensive_compute(item))
    .collect();
```

### Strategy 6: Caching

```rust
// ❌ SLOW: Recompute every time
fn get_user_permissions(user_id: u64) -> Vec<Permission> {
    let user = db.get_user(user_id)?;
    compute_permissions(&user)  // Recomputed every time
}

// ✅ FAST: Cache computed results
let mut permission_cache = std::collections::HashMap::new();

fn get_user_permissions(user_id: u64) -> &Vec<Permission> {
    permission_cache.entry(user_id)
        .or_insert_with(|| {
            let user = db.get_user(user_id).unwrap();
            compute_permissions(&user)
        })
}
```

---

## Part 4: Telemetry-Aware Optimization

### Measure Optimization Impact

```rust
#[instrument]  // Automatically measures duration
fn operation_before_optimization() {
    // Slow algorithm: 12 ticks
    for i in 0..items.len() {
        for j in i+1..items.len() {
            check_match(items[i], items[j]);
        }
    }
}

// After optimization...

#[instrument]  // Still automatically measured
fn operation_after_optimization() {
    // Fast algorithm: 3 ticks
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for item in items {
        if !seen.insert(item.id) {
            record_match(item);
        }
    }
}
```

### Optimize Instrumentation Overhead

```rust
// ❌ WRONG: Expensive operations in instrumentation
#[instrument(fields(items = format!("{:?}", all_items)))]  // Expensive!
fn process(items: &[Item]) { }

// ✅ RIGHT: Cheap operations only
#[instrument(fields(count = items.len()))]
fn process(items: &[Item]) { }
```

---

## Part 5: Step-by-Step Optimization Process

### Step 1: Establish Baseline

```bash
# Measure current performance
cargo test slow_test
# Result: operation takes 12 ticks (exceeds 8 tick limit)
```

### Step 2: Profile

```bash
# Identify bottleneck
cargo flamegraph --bin test_binary

# Output shows:
# - 70% in function_a (bottleneck!)
# - 20% in function_b
# - 10% other
```

### Step 3: Optimize Bottleneck

```rust
// Original: 70% of time, O(n²)
fn function_a_slow() {
    for i in 0..1000 {
        for j in 0..1000 {
            expensive_check(i, j);
        }
    }
}

// Optimized: Use hash table, O(n)
fn function_a_fast() {
    use std::collections::HashSet;
    let results = HashSet::new();
    for i in 0..1000 {
        if should_include(i) {
            results.insert(i);
        }
    }
}
```

### Step 4: Measure Impact

```bash
# Test again
cargo test slow_test
# Result: operation now takes 5 ticks (under 8 tick limit!)
```

### Step 5: Validate

```bash
# Full performance suite
make test-performance-v04

# Should pass:
# ✓ All operations ≤8 ticks
# ✓ No regressions
```

---

## Part 6: Common Performance Problems

### Problem 1: Unnecessary Cloning

**❌ Slow**:
```rust
fn process(items: Vec<Item>) {
    let copy = items.clone();  // Expensive clone!
    work_with(&copy);
}
```

**✅ Fast**:
```rust
fn process(items: &[Item]) {
    work_with(items);  // No clone
}
```

### Problem 2: Unbounded String Formatting

**❌ Slow**:
```rust
for item in items {
    let desc = format!("{:?}", item);  // Large allocation
}
```

**✅ Fast**:
```rust
for item in items {
    // Skip formatting or use write! to buffer
}
```

### Problem 3: Repeated Database Calls

**❌ Slow**:
```rust
for user_id in user_ids {
    let user = db.get_user(user_id).await?;  // N DB calls
}
```

**✅ Fast**:
```rust
let users = db.get_users(&user_ids).await?;  // 1 DB call
```

### Problem 4: Lock Contention

**❌ Slow**:
```rust
let shared = Arc::Mutex::new(data);
for i in 0..1000 {
    let mut guard = shared.lock().unwrap();  // Lock 1000 times
    process_one(&mut guard);
}
```

**✅ Fast**:
```rust
let mut local = data.clone();
for i in 0..1000 {
    process_one(&mut local);  // No locking
}
local.sync_back();
```

---

## Part 7: Performance Testing

### Write Performance Tests

```rust
#[test]
fn test_operation_meets_performance_constraint() {
    let start = Instant::now();

    for _ in 0..1000 {
        let result = operation();
        assert!(result.is_ok());
    }

    let elapsed = start.elapsed();
    let avg_ticks = elapsed.as_millis() / 1000;

    println!("Average: {} ticks/operation", avg_ticks);
    assert!(avg_ticks <= 8, "Exceeds ≤8 tick limit");
}
```

### Create Benchmark Comparisons

```bash
# Before optimization
cargo bench operation_before
# Result: operation_before time:   [12.050 ms]

# After optimization
cargo bench operation_after
# Result: operation_after time:    [3.240 ms]

# Improvement: 73% faster!
```

---

## Part 8: Troubleshooting Performance

### Issue: Still Exceeds 8 Ticks After Optimization

**Diagnosis**:
```bash
# Profile again to find remaining bottleneck
cargo flamegraph --bin test_binary --release

# Check what's still slow
```

**Solutions**:
1. Revisit algorithm choice
2. Consider parallelization
3. Reduce data size
4. Cache more aggressively
5. Consider distributed approach

### Issue: Optimization Broke Tests

**Solution**:
```bash
# Verify tests still pass
cargo test --all

# Ensure no semantic changes
# Just performance improvements
```

### Issue: Optimization Introduced Bugs

**Solution**:
```bash
# Always run full test suite
cargo test --all

# Run Weaver validation
weaver registry live-check --registry registry/

# If tests pass but telemetry wrong, revisit
```

---

## Quick Reference: Optimization Checklist

- [ ] Profile with flamegraph
- [ ] Identify bottleneck (70%+ of time)
- [ ] Choose optimization strategy
- [ ] Implement optimization
- [ ] Measure improvement
- [ ] Run full test suite
- [ ] Validate with Weaver
- [ ] Check no performance regressions

---

## Summary

### The Optimization Process

1. **Measure**: Current performance (baseline)
2. **Profile**: Find bottlenecks (flamegraph)
3. **Analyze**: Understand why it's slow
4. **Optimize**: Apply appropriate strategy
5. **Validate**: Ensure still correct
6. **Measure**: Verify improvement

### Key Principle

**Don't optimize prematurely. Profile first, optimize what matters.**

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Advanced
**Prerequisite for**: Passing make test-performance-v04
