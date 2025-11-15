# Performance Troubleshooting Guide

**Purpose**: Diagnose and fix performance issues in KNHK
**Target**: Hot path ≤8 ticks (≤2ns), Warm path ≤500µs
**Validation**: `make test-performance-v04` must pass

---

## Understanding KNHK Performance Model

**Three performance tiers:**

1. **Hot Path**: ≤8 ticks (≤2ns) - ASK, COUNT, COMPARE, VALIDATE operations
2. **Warm Path**: ≤500µs - CONSTRUCT8, simple workflows
3. **Cold Path**: ≤5s - Complex workflows, batch operations

**If operations don't meet their tier, diagnose with this guide.**

---

## Issue 1: Hot Path Operations Exceed 8 Ticks

### Symptom
```bash
$ make test-performance-v04
FAIL: ASK query took 15 ticks (expected ≤8)
```

### Diagnosis

**Step 1: Measure actual ticks:**
```rust
use std::arch::x86_64::_rdtsc;

let start = unsafe { _rdtsc() };
let result = execute_ask(query);
let end = unsafe { _rdtsc() };
let ticks = end - start;

println!("Operation took {} ticks", ticks);
```

**Step 2: Profile hot path:**
```bash
# Run with perf
perf record -g cargo test test_hot_path_ask_query_within_budget
perf report --stdio | head -50

# Look for:
# - Heap allocations (malloc, free)
# - Branch mispredicts
# - Cache misses
# - Function calls (should be inlined)
```

### Common Causes & Solutions

**Cause 1: Heap Allocations**
```rust
// ❌ Wrong: Heap allocation in hot path
fn execute_ask(query: &str) -> bool {
    let triples = Vec::new();  // Heap allocation!
    // ...
}

// ✅ Correct: Stack-only, pre-allocated
fn execute_ask(query: &str) -> bool {
    let mut triples: [Triple; 8] = [Triple::default(); 8];  // Stack
    // ...
}
```

**Cause 2: Branching**
```rust
// ❌ Wrong: Branching in hot path
fn compare(a: u64, b: u64) -> bool {
    if a == b {  // Branch misprediction possible
        true
    } else {
        false
    }
}

// ✅ Correct: Branchless
fn compare(a: u64, b: u64) -> bool {
    a == b  // Compiles to CMOV (no branch)
}

// ✅ Better: Explicit branchless
fn compare(a: u64, b: u64) -> u64 {
    ((a ^ b) == 0) as u64  // Bitwise, no branch
}
```

**Cause 3: Cache Misses**
```rust
// ❌ Wrong: AoS (Array of Structures) - poor cache locality
struct Triple {
    s: u64,
    p: u64,
    o: u64,
}
let triples: [Triple; 8] = [...];

// Access pattern: S₀ P₀ O₀ | S₁ P₁ O₁ | ... (jumps around)

// ✅ Correct: SoA (Structure of Arrays) - cache-friendly
#[repr(C, align(64))]
struct TripleStore {
    s: [u64; 8],
    p: [u64; 8],
    o: [u64; 8],
}

// Access pattern: S₀ S₁ S₂ ... | P₀ P₁ P₂ ... (sequential)
```

**Cause 4: Non-inlined Functions**
```rust
// ❌ Wrong: Function call overhead
fn hash_iri(iri: &str) -> u64 {
    // ...
}

// ✅ Correct: Force inlining
#[inline(always)]
fn hash_iri(iri: &str) -> u64 {
    // ...
}
```

### Measurement Tools

```bash
# CPU profiling
perf stat -e cycles,instructions,cache-misses,branch-misses cargo bench

# Tick measurement
cargo test --test chicago_tdd_hot_path_complete -- --nocapture

# SIMD usage verification
objdump -d target/release/knhk-hot | grep vpxor  # Should see AVX-512
```

### Expected Performance
- ASK: 1.0-1.5 ns (4-6 ticks @ 4GHz)
- COUNT: 1.0-1.5 ns
- COMPARE: 0.9-1.2 ns
- VALIDATE: 1.5-2.0 ns

---

## Issue 2: Warm Path Exceeds 500µs

### Symptom
```bash
$ cargo test test_warm_path_construct8
FAIL: CONSTRUCT8 took 850µs (expected ≤500µs)
```

### Diagnosis

**Step 1: Measure latency:**
```rust
let start = std::time::Instant::now();
let result = execute_construct8(ctx, ir);
let duration = start.elapsed();
println!("CONSTRUCT8 took {:?}", duration);
```

**Step 2: Profile with flamegraph:**
```bash
# Install flamegraph
cargo install flamegraph

# Profile warm path
cargo flamegraph --test warm_path_test -- test_construct8_performance

# Open flamegraph.svg to see bottlenecks
```

### Common Causes & Solutions

**Cause 1: Unnecessary Allocations**
```rust
// ❌ Wrong: Allocation per call
fn emit_triples(triples: &[Triple]) -> Vec<u8> {
    let mut buffer = Vec::new();  // Allocation!
    for triple in triples {
        buffer.extend_from_slice(&triple.serialize());
    }
    buffer
}

// ✅ Correct: Pre-allocated buffer
fn emit_triples(triples: &[Triple], buffer: &mut Vec<u8>) {
    buffer.clear();
    buffer.reserve(triples.len() * 24);  // 3 * 8 bytes per triple
    for triple in triples {
        buffer.extend_from_slice(&triple.serialize());
    }
}
```

**Cause 2: Synchronous I/O**
```rust
// ❌ Wrong: Blocking I/O
fn query_graph(sparql: &str) -> Result<Vec<Triple>, Error> {
    let response = reqwest::blocking::get(format!("http://graph-db/query?q={}", sparql))?;
    // Blocks for network I/O (~10ms)
    response.json()
}

// ✅ Correct: Async I/O
async fn query_graph(sparql: &str) -> Result<Vec<Triple>, Error> {
    let response = reqwest::get(format!("http://graph-db/query?q={}", sparql)).await?;
    // Non-blocking, allows other work
    response.json().await
}
```

**Cause 3: No Caching**
```rust
// ❌ Wrong: Recompute every time
fn execute_query(sparql: &str) -> Result<Vec<Triple>, Error> {
    let ast = parse_sparql(sparql)?;  // Parse every time (~100µs)
    execute_ast(&ast)
}

// ✅ Correct: Cache parsed queries
use lru::LruCache;

static QUERY_CACHE: Mutex<LruCache<String, Ast>> = Mutex::new(LruCache::new(100));

fn execute_query(sparql: &str) -> Result<Vec<Triple>, Error> {
    let mut cache = QUERY_CACHE.lock().unwrap();
    let ast = match cache.get(sparql) {
        Some(ast) => ast.clone(),
        None => {
            let ast = parse_sparql(sparql)?;
            cache.put(sparql.to_string(), ast.clone());
            ast
        }
    };
    execute_ast(&ast)
}
```

**Cause 4: Lock Contention**
```rust
// ❌ Wrong: Mutex on hot path
use std::sync::Mutex;

static COUNTER: Mutex<u64> = Mutex::new(0);

fn record_metric() {
    let mut count = COUNTER.lock().unwrap();  // Lock contention!
    *count += 1;
}

// ✅ Correct: Lock-free atomic
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn record_metric() {
    COUNTER.fetch_add(1, Ordering::Relaxed);  // Lock-free!
}
```

### Measurement Tools

```bash
# Latency histogram
cargo bench --bench warm_path_bench

# Memory profiling
valgrind --tool=massif cargo test test_construct8

# Lock contention
perf record -e sched:sched_switch cargo test
perf report
```

### Expected Performance
- CONSTRUCT8: 100-300µs (typical), ≤500µs (max)
- Cache hit: <10µs
- Cache miss: 100-500µs

---

## Issue 3: Memory Usage Growing Over Time

### Symptom
```bash
$ ps aux | grep knhk
knhk  1234  50.0  25.0  2.5GB ...  # Growing over time
```

### Diagnosis

**Step 1: Check for memory leaks:**
```bash
# Valgrind leak check
valgrind --leak-check=full cargo test

# Heaptrack
heaptrack cargo run --bin knhk-cli
heaptrack_gui heaptrack.knhk-cli.*.gz
```

**Step 2: Profile memory allocations:**
```bash
# Memory profiling with Massif
valgrind --tool=massif --massif-out-file=massif.out cargo test
ms_print massif.out | less

# Look for growing allocations
```

### Common Causes & Solutions

**Cause 1: Unbounded Cache**
```rust
// ❌ Wrong: No size limit
use std::collections::HashMap;

static CACHE: Mutex<HashMap<String, Data>> = Mutex::new(HashMap::new());

fn get_cached(key: &str) -> Option<Data> {
    CACHE.lock().unwrap().get(key).cloned()  // Grows forever!
}

// ✅ Correct: LRU cache with size limit
use lru::LruCache;

static CACHE: Mutex<LruCache<String, Data>> = Mutex::new(LruCache::new(1000));

fn get_cached(key: &str) -> Option<Data> {
    CACHE.lock().unwrap().get(key).cloned()  // Max 1000 entries
}
```

**Cause 2: Leaked Spans**
```rust
// ❌ Wrong: Spans not ended
fn execute_operation() {
    let span = tracer.start_span("operation".to_string(), None);
    // ... operation ...
    // Span never ended! Leaked.
}

// ✅ Correct: Always end spans
fn execute_operation() {
    let span = tracer.start_span("operation".to_string(), None);
    let result = do_work();
    tracer.end_span(span, SpanStatus::Ok);  // Always end
    result
}

// ✅ Better: RAII guard
struct SpanGuard<'a> {
    tracer: &'a mut Tracer,
    span: SpanContext,
}

impl Drop for SpanGuard<'_> {
    fn drop(&mut self) {
        self.tracer.end_span(self.span.clone(), SpanStatus::Ok);
    }
}
```

**Cause 3: Retained Test Data**
```rust
// ❌ Wrong: Static test data never freed
#[cfg(test)]
mod tests {
    static TEST_DATA: Mutex<Vec<LargeStruct>> = Mutex::new(Vec::new());

    #[test]
    fn test_something() {
        TEST_DATA.lock().unwrap().push(large_data());  // Never cleared!
    }
}

// ✅ Correct: Use thread-local or clean up
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        let test_data = vec![large_data()];  // Freed when test ends
        // ...
    }
}
```

### Measurement Tools

```bash
# Heap profiling
valgrind --tool=massif --pages-as-heap=yes cargo run
ms_print massif.out

# Track allocations
RUST_LOG=trace cargo run 2>&1 | grep -i alloc

# Memory usage over time
while true; do ps aux | grep knhk; sleep 10; done
```

### Expected Memory Usage
- Hot path: 0 allocations
- Warm path: <1MB per operation
- Steady state: <100MB RSS

---

## Issue 4: High CPU Usage

### Symptom
```bash
$ top
PID  COMMAND  %CPU
1234 knhk     250%  # Way too high!
```

### Diagnosis

**Step 1: CPU profiling:**
```bash
# Sample CPU usage
perf record -F 99 -p $(pgrep knhk) -g -- sleep 30
perf report --stdio | head -100

# Flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > cpu.svg
```

**Step 2: Identify hot functions:**
```bash
# Find top CPU consumers
perf top -p $(pgrep knhk)
```

### Common Causes & Solutions

**Cause 1: Busy-Wait Loop**
```rust
// ❌ Wrong: Busy waiting
loop {
    if queue.is_ready() {  // Spins at 100% CPU!
        break;
    }
}

// ✅ Correct: Sleep or async wait
loop {
    if queue.is_ready() {
        break;
    }
    std::thread::sleep(Duration::from_millis(10));  // Release CPU
}

// ✅ Better: Async wait
queue.wait_ready().await;  // Yields to scheduler
```

**Cause 2: Inefficient Algorithm**
```rust
// ❌ Wrong: O(n²) algorithm
fn find_duplicates(items: &[String]) -> Vec<String> {
    let mut dups = Vec::new();
    for i in 0..items.len() {
        for j in i+1..items.len() {
            if items[i] == items[j] {  // O(n²)
                dups.push(items[i].clone());
            }
        }
    }
    dups
}

// ✅ Correct: O(n) algorithm
use std::collections::HashSet;

fn find_duplicates(items: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut dups = Vec::new();
    for item in items {
        if !seen.insert(item.clone()) {  // O(n)
            dups.push(item.clone());
        }
    }
    dups
}
```

**Cause 3: Excessive Logging**
```rust
// ❌ Wrong: Logging in tight loop
for i in 0..1000000 {
    tracing::debug!("Processing item {}", i);  // 1M log lines!
}

// ✅ Correct: Sample logging
for i in 0..1000000 {
    if i % 10000 == 0 {
        tracing::debug!("Processed {} items", i);  // 100 log lines
    }
}
```

### Measurement Tools

```bash
# CPU profiling
perf record -g cargo run
perf report

# Find busy threads
ps -eLf | grep knhk | awk '{print $4}' | sort -rn | head -10

# System-wide CPU usage
mpstat -P ALL 1 10
```

### Expected CPU Usage
- Hot path: <1% CPU (single-threaded)
- Warm path: <10% CPU
- Cold path: <50% CPU

---

## Issue 5: Inconsistent Performance (Jitter)

### Symptom
```
Query latency: 1.2ns, 1.3ns, 15.2ns, 1.1ns, 22.5ns  # Spikes!
```

### Diagnosis

**Step 1: Measure latency distribution:**
```rust
use hdrhistogram::Histogram;

let mut hist = Histogram::<u64>::new(3).unwrap();

for _ in 0..10000 {
    let start = std::time::Instant::now();
    execute_query();
    let latency = start.elapsed().as_nanos() as u64;
    hist.record(latency).unwrap();
}

println!("p50: {}ns", hist.value_at_quantile(0.50));
println!("p99: {}ns", hist.value_at_quantile(0.99));
println!("p99.9: {}ns", hist.value_at_quantile(0.999));
println!("max: {}ns", hist.max());
```

### Common Causes & Solutions

**Cause 1: Garbage Collection (not applicable in Rust, but allocations)**
```rust
// Reduce allocations to minimize jitter
// Use object pools instead of allocating/freeing
```

**Cause 2: CPU Frequency Scaling**
```bash
# Disable CPU frequency scaling
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable turbo boost
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo
```

**Cause 3: Interrupt Storms**
```bash
# Check interrupt rate
watch -n 1 'cat /proc/interrupts'

# Pin to isolated cores
taskset -c 4-7 cargo run  # Use cores 4-7 only
```

**Cause 4: Page Faults**
```bash
# Check page faults
perf stat -e page-faults cargo run

# Pre-fault memory
mlock(addr, size);  # Lock pages in RAM
```

### Measurement Tools

```bash
# Latency histogram
cargo bench --bench latency_histogram

# Trace scheduler
perf sched record cargo run
perf sched timehist

# Measure jitter
perf stat -r 100 cargo bench  # Run 100 times, check variance
```

### Expected Jitter
- p50: 1.2ns
- p99: 2.5ns (≤2x p50)
- p99.9: 5.0ns (≤4x p50)
- max: ≤10ns (≤8x p50)

---

## Quick Performance Diagnostic Script

```bash
#!/bin/bash
# performance-check.sh

echo "=== Build & Test ==="
make test-performance-v04 || echo "FAILED: Performance test"

echo "=== CPU Profiling ==="
perf stat -e cycles,instructions,cache-misses,branch-misses \
    cargo test test_hot_path_ask_query_within_budget 2>&1 | grep -E '(cycles|instructions|cache-misses|branch-misses)'

echo "=== Memory Usage ==="
valgrind --tool=massif --massif-out-file=massif.out cargo test 2>&1 | tail -5
ms_print massif.out | grep 'MB'

echo "=== Hot Spots ==="
perf record -g cargo bench
perf report --stdio | head -20

echo "=== Performance Summary ==="
cargo bench | grep -E '(time:|ns/iter)'
```

---

## See Also

- [Performance Guide](/home/user/knhk/docs/PERFORMANCE.md)
- [Performance Optimization Checklist](/home/user/knhk/docs/reference/cards/PERFORMANCE_OPTIMIZATION_CHECKLIST.md)
- [Hot Path Implementation](/home/user/knhk/c/README.md)
