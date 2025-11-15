# Tutorial: Advanced Debugging and Profiling

**Level**: Expert
**Time**: 60-75 minutes
**Learning Objectives**: Master advanced debugging, profiling, and performance analysis

## What You'll Learn

By the end of this tutorial, you'll understand:
- Advanced debugging techniques
- Performance profiling with flamegraph
- Memory profiling and leak detection
- Distributed tracing analysis
- Bottleneck identification
- Real-time performance monitoring
- Production debugging strategies

## Prerequisites

- Completed: [Optimizing Performance](04-optimizing-performance.md)
- Production experience preferred
- ~75 minutes

## Part 1: Advanced Debugging Techniques

### Debugging with MIRI (Undefined Behavior Detection)

```bash
# Install MIRI
cargo +nightly miri setup

# Run tests with MIRI
cargo +nightly miri test --lib

# Detects:
# - Memory safety violations
# - Uninitialized memory reads
# - Data races
# - Use-after-free
```

### Debugging with Address Sanitizer

```bash
# Enable AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo test --lib my_test

# Detects:
# - Memory leaks
# - Buffer overflows
# - Use-after-free
# - Double-free
```

### Debugging with Memory Sanitizer

```bash
# Enable MemorySanitizer
RUSTFLAGS="-Z sanitizer=memory" cargo test --lib

# Detects:
# - Use of uninitialized memory
# - Memory leaks
# - Data races
```

## Part 2: Performance Profiling Deep Dive

### Flamegraph Analysis

```bash
# Install flamegraph
cargo install flamegraph

# Profile application
cargo flamegraph --bin my_app

# Profile tests
cargo flamegraph --test my_test

# Analyze SVG with browser
open flamegraph.svg
```

**Reading Flamegraphs**:
- Width = time spent
- Height = call stack depth
- Widest boxes = hottest code
- Tall stacks = deep recursion

### Criterion Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| {
            // Prevent optimization
            let n = black_box(20);
            fibonacci(n)
        })
    });

    // Compare with baseline
    c.bench_function("fib 25", |b| {
        b.iter(|| {
            let n = black_box(25);
            fibonacci(n)
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

Run:
```bash
cargo bench
```

## Part 3: Memory Profiling

### Heaptrack (Heap Profiler)

```bash
# Install heaptrack
sudo apt-get install heaptrack

# Profile application
heaptrack ./target/release/my_app

# Analyze results
heaptrack_gui heaptrack.my_app.123456.gz
```

Shows:
- Memory allocations
- Peak memory usage
- Allocation sites
- Memory leaks

### Valgrind

```bash
# Install Valgrind
sudo apt-get install valgrind

# Run with Memcheck (default)
valgrind --leak-check=full ./target/debug/my_app

# Detailed leak detection
valgrind --leak-check=full --show-leak-kinds=all ./target/debug/my_app
```

## Part 4: Distributed Tracing Analysis

### Jaeger Setup

```yaml
# docker-compose.yml
version: '3'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "6831:6831/udp"  # Jaeger agent
      - "16686:16686"    # UI
```

### Instrumenting for Jaeger

```rust
use opentelemetry::global;
use opentelemetry_jaeger as jaeger;
use tracing::instrument;

// Initialize Jaeger
let _tracer = jaeger::new_agent_pipeline()
    .install_simple()
    .unwrap();

#[instrument]
async fn my_function(param: &str) -> Result<String> {
    // Automatically sends spans to Jaeger
    Ok(param.to_uppercase())
}
```

Visit http://localhost:16686 to analyze traces.

## Part 5: Bottleneck Identification

### 1. CPU Bottlenecks

```bash
# Profile CPU usage
perf record -F 99 -p $PID -g -- sleep 30
perf report

# Or use flamegraph
cargo flamegraph --freq 99
```

Identify:
- Hot functions
- Tight loops
- Expensive operations

### 2. Memory Bottlenecks

```bash
# Monitor memory usage
watch -n 1 'ps aux | grep my_app | grep -v grep'

# Detailed memory profiling
cargo flamegraph --bench memory_test
```

Identify:
- Memory leaks
- Large allocations
- Unnecessary copies

### 3. I/O Bottlenecks

```bash
# Monitor I/O operations
iostat -x 1

# Profile system calls
strace -c ./target/release/my_app

# Network profiling
netstat -i
```

Identify:
- File I/O hotspots
- Network latency
- System call overhead

## Part 6: Production Debugging

### Log Analysis

```bash
# Search for errors
grep "ERROR" /var/log/my_app.log | head -100

# Count error types
grep "ERROR" /var/log/my_app.log | cut -d: -f2 | sort | uniq -c

# Timeline analysis
grep "ERROR\|WARN" /var/log/my_app.log | tail -1000
```

### Metrics Analysis

```prometheus
# Query high error rates
rate(errors_total[5m]) > 0.01

# Find slow endpoints
histogram_quantile(0.99, http_request_duration_seconds) > 1

# Memory pressure
process_resident_memory_bytes > 500000000
```

### Distributed Tracing

```bash
# Find slow requests in Jaeger UI
# 1. Service: select service
# 2. Operation: select operation
# 3. Tags: add error=false for success only
# 4. Min Duration: 1s for slow requests
# 5. Sort by duration
```

## Part 7: Production Profiling (Safe)

### CPU Profiling in Production

```rust
// Safe, low-overhead profiling
use perf_event::Builder;

let counter = Builder::new()
    .build()
    .expect("Failed to create counter");

counter.reset().unwrap();
counter.enable().unwrap();

// Code to profile
let _ = expensive_operation();

counter.disable().unwrap();
println!("CPU cycles: {}", counter.read().unwrap());
```

### Continuous Profiling

```rust
// Lightweight continuous monitoring
use std::thread;
use std::time::Duration;

thread::spawn(|| {
    loop {
        let metrics = get_runtime_metrics();
        send_to_prometheus(metrics);
        thread::sleep(Duration::from_secs(10));
    }
});
```

## Part 8: Advanced Debugging Patterns

### Crash Dump Analysis

```bash
# Enable core dumps
ulimit -c unlimited

# Run application
./target/release/my_app

# Analyze crash dump
gdb ./target/release/my_app core

# In GDB:
# (gdb) bt          # Backtrace
# (gdb) print var   # Print variable
# (gdb) frame N     # Select frame
```

### Race Condition Detection

```bash
# ThreadSanitizer
RUSTFLAGS="-Z sanitizer=thread" cargo test --lib

# Detects:
# - Data races
# - Deadlocks
# - Memory ordering issues
```

### Panic Analysis

```rust
// Set panic hook for detailed info
std::panic::set_hook(Box::new(|panic_info| {
    eprintln!("Panic occurred!");

    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        eprintln!("Message: {}", s);
    }

    if let Some(location) = panic_info.location() {
        eprintln!("Location: {}:{}", location.file(), location.line());
    }

    // Get backtrace
    let backtrace = std::backtrace::Backtrace::capture();
    eprintln!("Backtrace:\n{}", backtrace);
}));
```

## Part 9: Debugging Checklist

When a production issue occurs:

1. **Gather Information**
   - [ ] Error logs
   - [ ] Metrics at time of issue
   - [ ] Traces (if available)
   - [ ] System state (CPU, memory, I/O)

2. **Analyze Logs**
   - [ ] Filter for errors/warnings
   - [ ] Find error timeline
   - [ ] Identify patterns
   - [ ] Check correlations

3. **Profile Metrics**
   - [ ] Check error rate
   - [ ] Review latency percentiles
   - [ ] Monitor resource usage
   - [ ] Look for anomalies

4. **Examine Traces**
   - [ ] Find slow requests
   - [ ] Identify bottlenecks
   - [ ] Check service dependencies
   - [ ] Analyze failure paths

5. **Reproduce Locally**
   - [ ] Create test case
   - [ ] Run with MIRI
   - [ ] Use AddressSanitizer
   - [ ] Profile with flamegraph

6. **Fix and Verify**
   - [ ] Apply fix
   - [ ] Test thoroughly
   - [ ] Profile improvement
   - [ ] Deploy carefully

## Complete Example: Finding Memory Leak

```bash
# 1. Detect issue (memory growing)
watch 'ps aux | grep my_app | grep -v grep | awk "{print \$6}"'

# 2. Profile with heaptrack
heaptrack ./target/release/my_app

# 3. Run for a while
sleep 60

# 4. Analyze
heaptrack_gui heaptrack.my_app.*.gz
# Look for growing allocations

# 5. Debug code
# Add logging to allocation sites
RUST_LOG=trace cargo test

# 6. Find leak
// Code analysis shows:
// Vec::new() created without drop - LEAK!

# 7. Fix
drop(vec);  // or use scope

# 8. Verify
cargo test --lib
heaptrack ./target/release/my_app
# Verify memory stable
```

## What You've Learned

Congratulations! You now understand:

1. **Advanced Debugging** - MIRI, sanitizers, crash dumps
2. **Performance Profiling** - Flamegraph, Criterion, perf
3. **Memory Analysis** - Heaptrack, Valgrind, leak detection
4. **Distributed Tracing** - Jaeger setup and analysis
5. **Bottleneck Identification** - CPU, memory, I/O
6. **Production Debugging** - Logs, metrics, traces
7. **Continuous Profiling** - Safe production monitoring
8. **Debugging Patterns** - Race detection, panic handling

## Next Steps

- **Debug your app**: Apply these techniques
- **Set up profiling**: Add continuous monitoring
- **Analyze production**: Use Jaeger/metrics
- **Optimize**: Use flamegraph to find hotspots

---

**You are here**: Tutorial (Learning-oriented)
**Framework**: Diátaxis
**Tutorial Duration**: ~75 minutes
**Difficulty**: Expert
**Prerequisites**: Performance Optimization
