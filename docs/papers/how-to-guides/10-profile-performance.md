# How-to Guide: Profile Code Performance Deep Dive

**Goal**: Deep performance analysis and optimization
**Time**: 40-60 minutes per analysis
**Difficulty**: Advanced

## Step 1: Profiling Setup

### Install Tools

```bash
# Flamegraph
cargo install flamegraph

# perf
sudo apt-get install linux-tools-generic

# Criterion
cargo add criterion --dev

# Heaptrack
sudo apt-get install heaptrack

# Cargo-profiling
cargo install cargo-profiling
```

## Step 2: CPU Profiling with Flamegraph

### Generate Flamegraph

```bash
# Profile release binary
cargo flamegraph --bin my_app --freq 99

# Profile tests
cargo flamegraph --test my_test

# Custom options
cargo flamegraph --bin my_app -- arg1 arg2

# Result: flamegraph.svg
```

### Read Flamegraphs

```
Width = Time spent (wider = hotter)
Height = Call stack depth
Color = Random (aids distinction)
Left-to-right = Time order

Hottest paths:
1. Find widest boxes
2. Trace upward to see caller
3. Hover for details
4. Click to zoom
```

### Example Analysis

```
Wide box: "process_data"
├─ Called by: "handle_request"
├─ Calls: "validate" (10%), "transform" (50%), "save" (40%)
└─ Top opportunity: Optimize "transform"

Action: Profile "transform" deeper
```

## Step 3: Memory Profiling

### Heaptrack Analysis

```bash
# Run with heaptrack
heaptrack ./target/release/my_app

# Let it run for desired duration
# Ctrl+C to stop

# Open GUI
heaptrack_gui heaptrack.my_app.*.gz
```

### What to Look For

1. **Peak Memory**: Total max memory used
2. **Allocations**: Number and size of allocations
3. **Growing Allocations**: Persistent allocations (leaks)
4. **Hot Sites**: Functions allocating most memory

### Example

```
Peak: 500 MB
Top allocators:
1. String creation in process() - 300 MB
2. Vec growth in buffer() - 150 MB
3. HashMap in cache() - 50 MB

Leaks:
- Vec created in loop, never freed - 100 MB
```

## Step 4: Detailed CPU Analysis

### Using perf

```bash
# Record profile
perf record -F 99 ./target/release/my_app

# View report
perf report

# Show call graph
perf report --graph=caller

# Annotate source
perf annotate
```

### Interpreting perf

```
+   50.00%  my_app  [kernel.kallsyms]  [k] copy_user_enhanced_fast_string
+   30.00%  my_app  libm.so             [.] sin
+   15.00%  my_app  my_app              [.] transform_data
+    5.00%  my_app  my_app              [.] validate
```

## Step 5: Benchmark-Driven Profiling

### Criterion Setup

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_algorithms");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let data = black_box(generate_data(size));
                b.iter(|| my_sort(&data))
            }
        );
    }

    group.finish();
}

criterion_group!(benches, bench_sort);
criterion_main!(benches);
```

### Run Benchmarks

```bash
cargo bench

# Results show:
# sort_algorithms/100      time: [1.234 ms 1.245 ms 1.258 ms]
# sort_algorithms/1000     time: [15.23 ms 15.45 ms 15.67 ms]
# sort_algorithms/10000    time: [157.2 ms 158.4 ms 159.6 ms]

# Compare with baseline
cargo bench -- --baseline master
```

## Step 6: Trace Profiling

### Distributed Tracing with Jaeger

```bash
# Start Jaeger
docker run -d --name jaeger \
  -p 6831:6831/udp \
  -p 16686:16686 \
  jaegertracing/all-in-one

# Access UI at http://localhost:16686
```

### Find Bottlenecks

1. **Service Dropdown**: Select service
2. **Operation**: Choose operation
3. **Tags**: Add filters (e.g., `span.kind=server`)
4. **Min Duration**: Filter slow requests
5. **Sort by Duration**: Find slowest

### Analyze Spans

```
Request Timeline:
├─ load_data: 50ms
├─ process_data: 800ms  ← Bottleneck
├─ save_data: 100ms
└─ notify: 50ms
Total: 1000ms

Action: Optimize process_data
```

## Step 7: Profile-Guided Optimization

### Benchmark Before/After

```bash
# Baseline
cargo bench --bench my_bench

# Make optimization
# (change code)

# Compare
cargo bench --bench my_bench -- --baseline main

# Output shows improvement:
# time: [15.23 ms 15.45 ms 15.67 ms]
# vs    [10.12 ms 10.34 ms 10.56 ms]
# Change: -33.7% faster
```

### Profile After Changes

```bash
cargo flamegraph --bin my_app

# Verify:
# 1. Hottest function changed?
# 2. Hot path narrower?
# 3. Different bottleneck?
```

## Step 8: Memory Leak Detection

### Address Sanitizer

```bash
RUSTFLAGS="-Z sanitizer=address" cargo test --lib

# Output shows:
# ==1234==ERROR: LeakSanitizer: detected memory leaks
# Direct leak of 100 byte(s) in 1 object(s)
#   allocated at:
#     #0 0x123456 in __rust_alloc
#     #1 0x789abc in Vec::new
#     #2 0x def012 in my_function
```

### Valgrind

```bash
valgrind --leak-check=full ./target/debug/my_app

# Output:
# LEAK SUMMARY:
# definitely lost: 1,024 bytes in 5 blocks
# indirectly lost: 0 bytes in 0 blocks
# possibly lost: 128 bytes in 2 blocks
```

## Step 9: Continuous Profiling

### Low-Overhead Metrics

```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static::lazy_static! {
    static ref FUNCTION_CALLS: Counter =
        Counter::new("function_calls", "Total calls").unwrap();

    static ref FUNCTION_DURATION: Histogram =
        Histogram::new("function_duration_seconds", "Duration").unwrap();
}

#[instrument]
pub fn my_function() -> Result<()> {
    let start = std::time::Instant::now();
    FUNCTION_CALLS.inc();

    let result = expensive_operation()?;

    let elapsed = start.elapsed().as_secs_f64();
    FUNCTION_DURATION.observe(elapsed);

    Ok(result)
}
```

Export metrics to Prometheus:
```bash
# Expose metrics endpoint
GET /metrics HTTP/1.1
```

## Step 10: Profiling Workflow Summary

```
1. Identify Issue
   └─ High CPU? Memory? Latency?

2. Profile
   ├─ CPU: flamegraph
   ├─ Memory: heaptrack
   └─ Latency: Jaeger

3. Analyze
   ├─ Find hotspot
   ├─ Understand why
   └─ Estimate impact

4. Optimize
   ├─ Change code
   ├─ Benchmark
   └─ Verify improvement

5. Verify
   ├─ Profile again
   ├─ Check for regressions
   └─ Commit improvement
```

## Common Profiling Patterns

### Pattern: Find Function Taking Time

```bash
cargo flamegraph --bin my_app
# Look for widest box
# That function is the bottleneck
```

### Pattern: Find Memory Leak

```bash
RUSTFLAGS="-Z sanitizer=address" cargo test --lib
# Look for "detected memory leaks"
# Check allocation site in backtrace
```

### Pattern: Find Slow Queries

```
In Jaeger UI:
1. Select service
2. Filter by duration > 1s
3. Click to see details
4. Identify slow span
5. Fix code
```

## Profiling Checklist

- [ ] Reproduce issue consistently
- [ ] Profile with release build
- [ ] Multiple profile runs for accuracy
- [ ] Identify clear bottleneck
- [ ] Understand root cause
- [ ] Test optimization locally
- [ ] Benchmark improvement
- [ ] Verify no regressions
- [ ] Deploy and monitor

## Next Steps

- **Profile your app**: Start with flamegraph
- **Set up metrics**: Add Prometheus
- **Monitor production**: Use Jaeger
- **Iterate**: Benchmark before/after

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Advanced
**Related**: Performance Optimization, Debugging
