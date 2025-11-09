# KNHK Performance Benchmark Guide

**Quick Start**: Run `./scripts/run_performance_benchmarks.sh`

## Overview

This guide explains how to run, interpret, and optimize based on KNHK's comprehensive Fortune 5 performance benchmark suite.

## Quick Reference

| Command | Purpose |
|---------|---------|
| `./scripts/run_performance_benchmarks.sh` | Run all benchmarks with SLO compliance checks |
| `cargo bench --bench fortune5_performance` | Run all benchmarks (raw) |
| `cargo bench --bench fortune5_performance hot_path` | Run only hot path benchmarks |
| `cargo bench --bench fortune5_performance -- --save-baseline main` | Save baseline for comparison |
| `cargo bench --bench fortune5_performance -- --baseline main` | Compare against baseline |

## Benchmark Suite Structure

### 1. Hot Path Benchmarks (CRITICAL)

**Location**: `benches/fortune5_performance.rs::hot_path_benches`

**Validates**: Chatman Constant (≤8 ticks)

**Benchmarks**:
- `pattern_execution`: Pattern matching overhead
- `state_transition`: State machine performance
- `condition_evaluation`: Decision logic
- `task_lookup_hot_read`: Data structure access (≤2ns)

**Pass Criteria**: ALL operations ≤8 CPU ticks

**Example Output**:
```
hot_path/pattern_execution
                        time:   [2.1567 ns 2.1631 ns 2.1704 ns]
                        ticks: ~5 (PASS: ≤8)
```

### 2. End-to-End Benchmarks

**Location**: `benches/fortune5_performance.rs::e2e_benches`

**Validates**: Real-world workflow SLAs

**Benchmarks**:
- `atm_withdrawal`: ATM transaction flow (<3s)
- `swift_payment`: International payment with parallel compliance (<5s)

**Pass Criteria**:
- ATM: <3 seconds end-to-end
- SWIFT: <5 seconds end-to-end

**Example Output**:
```
e2e_workflows/atm_withdrawal
                        time:   [2.4521 s 2.4893 s 2.5312 s]
                        PASS: <3s target
```

### 3. Scalability Benchmarks

**Location**: `benches/fortune5_performance.rs::scalability_benches`

**Validates**: Performance under load

**Benchmarks**:
- `payroll_scalability`: Multi-instance pattern (10-1000 employees)
- `concurrent_case_creation`: Stress test (10-200 concurrent cases)

**Pass Criteria**:
- 1000 employee payroll: <60 seconds
- Linear scaling (no exponential degradation)

**Example Output**:
```
scalability/payroll_employees/1000
                        time:   [43.521 s 44.123 s 44.892 s]
                        throughput: 22.7 employees/s
                        PASS: <60s target
```

### 4. Telemetry Overhead Benchmarks

**Location**: `benches/fortune5_performance.rs::telemetry_benches`

**Validates**: OTEL instrumentation cost

**Benchmarks**:
- `telemetry_overhead`: Baseline vs instrumented comparison

**Pass Criteria**: <5% overhead

**Example Output**:
```
=== TELEMETRY OVERHEAD ANALYSIS ===
Baseline (no telemetry): 1.234 µs
With telemetry: 1.271 µs
Overhead: 37ns (3.0%)
Target: <5% overhead - ✓ PASS
```

### 5. Resource Allocation Benchmarks

**Location**: `benches/fortune5_performance.rs::resource_benches`

**Validates**: Resource manager efficiency

**Benchmarks**:
- `allocate_resources`: Resource allocation latency

**Pass Criteria**: <1ms p99

## Interpreting Results

### Understanding Criterion Output

```
hot_path/pattern_execution
                        time:   [2.1567 ns 2.1631 ns 2.1704 ns]
                                 ▲         ▲         ▲
                                 │         │         └─ Upper bound (p95)
                                 │         └─────────── Median
                                 └───────────────────── Lower bound (p5)
                        change: [-2.3% -1.8% -1.3%] (p = 0.00 < 0.05)
                                 Performance improved!
```

**Key Metrics**:
- **time**: Median execution time
- **change**: Comparison to baseline (if available)
- **p**: Statistical significance (p < 0.05 = significant)
- **R²**: Goodness of fit (higher = more consistent)

### HTML Reports

After running benchmarks, detailed HTML reports are generated:

```bash
open rust/knhk-workflow-engine/target/criterion/report/index.html
```

**Features**:
- Interactive charts
- Percentile distributions
- Iteration-by-iteration plots
- Comparison graphs

## Running Benchmarks

### Option 1: Automated Script (Recommended)

```bash
./scripts/run_performance_benchmarks.sh
```

**Outputs**:
- Console summary with SLO compliance
- Detailed text reports in `docs/performance/reports/`
- HTML reports in `target/criterion/`

### Option 2: Manual Cargo Bench

```bash
cd rust/knhk-workflow-engine

# Run all benchmarks
cargo bench --bench fortune5_performance

# Run specific group
cargo bench --bench fortune5_performance hot_path
cargo bench --bench fortune5_performance e2e
cargo bench --bench fortune5_performance scalability
cargo bench --bench fortune5_performance telemetry
cargo bench --bench fortune5_performance resource

# Save baseline for future comparison
cargo bench --bench fortune5_performance -- --save-baseline v1.0.0

# Compare against baseline
cargo bench --bench fortune5_performance -- --baseline v1.0.0

# Filter by specific benchmark
cargo bench --bench fortune5_performance -- pattern_execution
```

## Performance Profiling

### CPU Profiling with Flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cd rust/knhk-workflow-engine
cargo flamegraph --bench fortune5_performance -- --bench

# Open flamegraph
open flamegraph.svg
```

### Memory Profiling with Heaptrack

```bash
# Install heaptrack (macOS)
brew install heaptrack

# Run with heaptrack
heaptrack cargo bench --bench fortune5_performance

# Analyze results
heaptrack_gui heaptrack.cargo.*.zst
```

### Lock Contention Analysis

```bash
# Build with debug symbols
RUSTFLAGS="-C debuginfo=2" cargo bench --bench fortune5_performance --no-run

# Run with perf (Linux)
perf record -g target/release/deps/fortune5_performance-*
perf report
```

## SLO Compliance Checks

The automated script checks these SLOs:

| SLO | Benchmark | Threshold | Auto-Check |
|-----|-----------|-----------|------------|
| Chatman Constant | `hot_path/*` | ≤8 ticks | ✅ Yes |
| Hot reads | `task_lookup_hot_read` | ≤2ns p99 | ✅ Yes |
| Hot writes | `allocate_resources` | ≤1ms p99 | ⚠️ Manual |
| Complex ops | `atm_withdrawal` | <3s | ✅ Yes |
| Telemetry | `telemetry_overhead` | <5% | ✅ Yes |

**Manual SLO Verification**:

```bash
# Extract p99 latency from Criterion JSON
jq '.mean.point_estimate' target/criterion/hot_path/task_lookup_hot_read/base/estimates.json

# Check if ≤2ns (2000000 femtoseconds)
# Criterion uses picoseconds (ps): 2ns = 2000ps
```

## Optimization Workflow

### Step 1: Identify Bottlenecks

```bash
# Run benchmarks
./scripts/run_performance_benchmarks.sh

# Generate flamegraph
cargo flamegraph --bench fortune5_performance -- --bench
```

### Step 2: Profile Hot Path

```bash
# Focus on failing benchmark
cargo bench --bench fortune5_performance -- pattern_execution

# Add instrumentation
# Edit benches/fortune5_performance.rs:
// Add println!() or tracing::info!() to measure phases
```

### Step 3: Apply Optimizations

**Common Optimizations**:

1. **Reduce Allocations**:
   ```rust
   // Before: allocates Vec
   let tasks: Vec<Task> = workflow.tasks.iter().cloned().collect();

   // After: iterate without allocation
   for task in &workflow.tasks { ... }
   ```

2. **Cache Lookups**:
   ```rust
   // Add to WorkflowEngine:
   task_cache: Arc<DashMap<TaskId, Task>>
   ```

3. **Optimize Lock Granularity**:
   ```rust
   // Before: single RwLock
   state: Arc<RwLock<State>>

   // After: fine-grained DashMap
   cases: Arc<DashMap<CaseId, Case>>
   specs: Arc<DashMap<SpecId, Spec>>
   ```

4. **Inline Hot Functions**:
   ```rust
   #[inline(always)]
   pub fn pattern_matches(&self, pattern: Pattern) -> bool {
       // Hot path logic
   }
   ```

### Step 4: Verify Improvement

```bash
# Save current as baseline
cargo bench --bench fortune5_performance -- --save-baseline before

# Apply optimization
# ... make changes ...

# Compare
cargo bench --bench fortune5_performance -- --baseline before

# Look for negative change (improvement):
# change: [-15.3% -12.8% -10.2%] ← 12.8% faster!
```

### Step 5: Regression Testing

```bash
# Add to CI/CD:
#!/bin/bash
cargo bench --bench fortune5_performance -- --save-baseline main

# After merge:
cargo bench --bench fortune5_performance -- --baseline main

# Fail if >10% regression
if [ $(jq '.change.percent > 10' estimates.json) ]; then
    echo "Performance regression detected!"
    exit 1
fi
```

## Continuous Monitoring

### Production Metrics

Add to OTEL collector config:

```yaml
# prometheus.yml
- job_name: 'knhk'
  static_configs:
    - targets: ['localhost:9090']
  relabel_configs:
    - source_labels: [__name__]
      regex: 'knhk_(pattern_execution|workflow)_duration_.*'
      action: keep
```

### Alerting Rules

```yaml
# alerts.yml
groups:
  - name: knhk_performance
    rules:
      - alert: HotPathSlowdown
        expr: histogram_quantile(0.99, rate(knhk_pattern_execution_duration_ns[5m])) > 3.2
        annotations:
          summary: "Hot path exceeds 8 ticks (3.2ns @ 2.5GHz)"

      - alert: WorkflowSLOViolation
        expr: histogram_quantile(0.99, rate(knhk_workflow_duration_seconds{workflow="atm"}[5m])) > 3.0
        annotations:
          summary: "ATM workflow exceeds 3s SLO"
```

## Troubleshooting

### Issue: Benchmarks fail to compile

```bash
# Check Cargo.toml has correct config
[[bench]]
name = "fortune5_performance"
harness = false

# Ensure criterion in dev-dependencies
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

### Issue: Inconsistent results

```bash
# Increase sample size
cargo bench --bench fortune5_performance -- --sample-size 1000

# Increase measurement time
# Edit benches/fortune5_performance.rs:
group.measurement_time(Duration::from_secs(30));
```

### Issue: Out of memory

```bash
# Reduce concurrent workload
# Edit benches/fortune5_performance.rs:
for employee_count in [10, 50, 100].iter() {  // Was [10, 100, 500, 1000]
```

### Issue: Can't find baseline

```bash
# List available baselines
ls -la target/criterion/*/base/

# Baselines stored per benchmark
# Must use exact same benchmark name
cargo bench --bench fortune5_performance -- --baseline exact_name
```

## Best Practices

1. **Run on Consistent Hardware**: Same machine, same load
2. **Close Background Apps**: Minimize noise
3. **Use Release Mode**: Benchmarks always use `--release`
4. **Warmup JIT**: Criterion handles this automatically
5. **Statistical Significance**: p < 0.05 for valid comparison
6. **Multiple Runs**: Criterion averages 100+ iterations
7. **Save Baselines**: Before major changes
8. **Regression Tests**: In CI/CD pipeline
9. **Profile First**: Don't guess, measure
10. **Incremental Optimization**: One change at a time

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph Guide](https://github.com/flamegraph-rs/flamegraph)
- [KNHK Performance SLOs](fortune5-benchmarks.md)

---

**Last Updated**: 2025-11-08
**Maintainer**: Performance Benchmarker Agent
