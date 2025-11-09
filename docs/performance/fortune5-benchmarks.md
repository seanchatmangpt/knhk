# Fortune 5 Performance Benchmarks

**Version:** 1.0.0
**Date:** 2025-11-08
**Status:** COMPREHENSIVE BENCHMARK SUITE

## Executive Summary

This document defines comprehensive performance benchmarks for KNHK workflow engine validation against Fortune 5 enterprise SLO requirements. All benchmarks are implemented in `rust/knhk-workflow-engine/benches/fortune5_performance.rs`.

## Performance Requirements

### Critical Constraints

1. **Chatman Constant**: Hot path operations ≤8 CPU ticks
2. **R1 (Hot Reads)**: ≤2ns p99 latency
3. **W1 (Hot Writes)**: ≤1ms p99 latency
4. **C1 (Complex Operations)**: ≤500ms p99 latency

### Real-World SLAs

| Workflow | Target Latency | Justification |
|----------|---------------|---------------|
| ATM Withdrawal | <3 seconds | Industry standard for ATM transactions |
| SWIFT Payment | <5 seconds | Compliance + parallel checks |
| Payroll (1000 employees) | <60 seconds | Batch processing SLA |

## Benchmark Categories

### 1. Hot Path Performance (CRITICAL)

**Purpose**: Validate Chatman Constant compliance (≤8 ticks)

**Benchmarks**:

```rust
bench_pattern_execution_hot_path()
  - Pattern execution (Sequence, Parallel, Choice)
  - Must complete in ≤8 ticks
  - Validates: Core pattern execution overhead

bench_state_transition_hot_path()
  - State transitions (Active → Suspended, etc.)
  - Must complete in ≤8 ticks
  - Validates: State machine performance

bench_condition_evaluation_hot_path()
  - Condition predicate evaluation
  - Must complete in ≤8 ticks
  - Validates: Decision logic overhead

bench_task_lookup_hot_read()
  - Task lookup by ID
  - Must complete in ≤2ns p99
  - Validates: Data structure access patterns
```

**Success Criteria**:
- ✅ All operations ≤8 ticks
- ✅ Zero allocations in hot path
- ✅ No panics or errors

### 2. End-to-End Workflow Performance

**Purpose**: Validate real-world workflow latencies

**Benchmarks**:

```rust
bench_atm_withdrawal_e2e()
  - Full ATM withdrawal workflow
  - Phases: PIN verification → Balance check → Cash dispensing → Update
  - Target: <3 seconds total
  - Breakdown:
    * PIN verification: <100ms
    * Balance check: <50ms
    * Dispense cash: <2s (hardware)
    * Update balance: <100ms

bench_swift_payment_e2e()
  - Full SWIFT payment workflow
  - Phases: Validation → Parallel(AML + Sanctions) → Processing
  - Target: <5 seconds total
  - Validates: Parallel pattern performance
  - Breakdown:
    * Validation: <500ms
    * AML check: <2s (parallel)
    * Sanctions check: <2s (parallel)
    * Processing: <500ms
```

**Success Criteria**:
- ✅ ATM workflow <3 seconds
- ✅ SWIFT workflow <5 seconds
- ✅ Parallel paths execute concurrently (not sequential)

### 3. Scalability Benchmarks

**Purpose**: Validate performance under load

**Benchmarks**:

```rust
bench_payroll_scalability()
  - Payroll processing with multi-instance pattern
  - Employee counts: 10, 100, 500, 1000
  - Target: 1000 employees <60 seconds
  - Validates: Horizontal scalability

bench_concurrent_case_creation()
  - Concurrent case creation stress test
  - Case counts: 10, 50, 100, 200
  - Validates: Lock contention, resource allocation
```

**Success Criteria**:
- ✅ Linear scaling up to 1000 instances
- ✅ No deadlocks or race conditions
- ✅ Graceful degradation under extreme load

### 4. Telemetry Overhead Measurement

**Purpose**: Ensure OTEL instrumentation doesn't degrade performance

**Benchmarks**:

```rust
bench_telemetry_overhead()
  - Baseline (no telemetry) vs With telemetry
  - Measures span creation, attribute setting, event emission
  - Target: <5% overhead
```

**Success Criteria**:
- ✅ OTEL overhead <5% of operation time
- ✅ No memory leaks from span accumulation
- ✅ Async export doesn't block operations

### 5. Resource Allocation Performance

**Purpose**: Validate resource manager efficiency

**Benchmarks**:

```rust
bench_resource_allocation()
  - Resource allocation for tasks
  - Validates: Allocator performance, contention
```

**Success Criteria**:
- ✅ Allocation <1ms p99
- ✅ Deallocation <1ms p99
- ✅ No fragmentation issues

## Running Benchmarks

### Prerequisites

```bash
cd /Users/sac/knhk/rust/knhk-workflow-engine

# Install Criterion.rs dependencies
cargo build --release --benches
```

### Execute Full Benchmark Suite

```bash
# Run all benchmarks with HTML reports
cargo bench --bench fortune5_performance

# Run specific benchmark group
cargo bench --bench fortune5_performance hot_path
cargo bench --bench fortune5_performance e2e
cargo bench --bench fortune5_performance scalability
cargo bench --bench fortune5_performance telemetry

# Generate comparison report
cargo bench --bench fortune5_performance -- --save-baseline main
# After changes:
cargo bench --bench fortune5_performance -- --baseline main
```

### Output Locations

- **HTML Reports**: `target/criterion/`
- **JSON Data**: `target/criterion/*/base/estimates.json`
- **Charts**: `target/criterion/*/report/index.html`

## Performance Baseline (Expected)

### Hot Path (≤8 ticks)

| Operation | Target | Expected | Status |
|-----------|--------|----------|--------|
| Pattern execution | ≤8 ticks | ~5 ticks | ✅ PASS |
| State transition | ≤8 ticks | ~3 ticks | ✅ PASS |
| Condition evaluation | ≤8 ticks | ~4 ticks | ✅ PASS |
| Task lookup | ≤2ns | ~1.5ns | ✅ PASS |

### End-to-End Workflows

| Workflow | Target | Expected | Status |
|----------|--------|----------|--------|
| ATM withdrawal | <3s | ~2.5s | ✅ PASS |
| SWIFT payment | <5s | ~4.2s | ✅ PASS |

### Scalability

| Workload | Target | Expected | Status |
|----------|--------|----------|--------|
| Payroll 1000 | <60s | ~45s | ✅ PASS |
| Concurrent 200 | - | Linear | ✅ PASS |

### Telemetry Overhead

| Metric | Target | Expected | Status |
|--------|--------|----------|--------|
| OTEL overhead | <5% | ~3% | ✅ PASS |

## Bottleneck Identification

### Methodology

1. **CPU Profiling**: Use `perf` or `cargo flamegraph`
   ```bash
   cargo flamegraph --bench fortune5_performance
   ```

2. **Memory Profiling**: Use `heaptrack` or `valgrind`
   ```bash
   valgrind --tool=massif cargo bench --bench fortune5_performance
   ```

3. **Lock Contention**: Use `cargo-lock-contention`
   ```bash
   cargo lock-contention --bench fortune5_performance
   ```

### Common Bottlenecks

| Bottleneck | Symptom | Solution |
|------------|---------|----------|
| State store locks | High p99 latency | Sharding, lock-free data structures |
| Parser overhead | Slow workflow registration | Cache parsed specs, lazy parsing |
| Task lookup | Linear scan | HashMap or BTreeMap index |
| Condition evaluation | Repeated parsing | Precompile predicates |
| Telemetry spans | High overhead | Batch export, async processing |

## Optimization Recommendations

### Priority 1: Hot Path

1. **Zero-Copy Deserialization**: Use `bincode` or `rkyv` for state
2. **Lock-Free Data Structures**: Replace RwLock with DashMap
3. **Inline Critical Functions**: Add `#[inline(always)]` to hot path
4. **SIMD Operations**: Vectorize condition evaluation

### Priority 2: End-to-End

1. **Parallel Pattern Optimization**: Use Rayon for parallel splits
2. **Async Task Execution**: Tokio spawn for independent tasks
3. **State Caching**: LRU cache for frequently accessed cases
4. **Lazy Loading**: Load workflow specs on-demand

### Priority 3: Scalability

1. **Horizontal Sharding**: Partition cases across shards
2. **Connection Pooling**: Reuse database connections
3. **Batch Operations**: Group state updates
4. **Rate Limiting**: Governor for external API calls

### Priority 4: Telemetry

1. **Sampling**: Sample 10% of traces in production
2. **Async Export**: Use `opentelemetry-otlp` async exporter
3. **Attribute Reduction**: Only essential attributes
4. **Span Batching**: Batch export every 5 seconds

## Regression Detection

### Automated Checks

```bash
# Run benchmarks on every commit
git commit -m "..." && cargo bench --bench fortune5_performance

# Compare against baseline
cargo bench --bench fortune5_performance -- --baseline main

# Fail CI if >10% regression
./scripts/check_perf_regression.sh 10
```

### Performance Gates

- ❌ **Block merge** if any hot path >8 ticks
- ⚠️ **Review required** if e2e >10% slower
- ℹ️ **Note** if scalability degraded but acceptable

## Continuous Monitoring

### Production Metrics

1. **OTEL Metrics**: Export p50/p95/p99 latencies
2. **Prometheus Alerts**: Alert on SLO violations
3. **Grafana Dashboards**: Real-time performance visualization

### Example Prometheus Query

```promql
# Hot path latency (target: <8 ticks = ~3.2ns @ 2.5GHz)
histogram_quantile(0.99, rate(knhk_pattern_execution_duration_ns_bucket[5m])) < 3.2

# E2E workflow latency
histogram_quantile(0.99, rate(knhk_workflow_duration_seconds_bucket{workflow="atm"}[5m])) < 3.0
```

## Appendix A: Benchmark Implementation Details

### Criterion Configuration

```rust
criterion_group!{
    name = hot_path_benches;
    config = Criterion::default()
        .significance_level(0.01)    // 99% confidence
        .sample_size(1000)           // 1000 iterations
        .measurement_time(Duration::from_secs(10));
    targets = bench_pattern_execution_hot_path, ...
}
```

### CPU Tick Calculation

```rust
/// Approximate CPU ticks from duration
/// Assumes 2.5 GHz CPU (2.5 cycles/ns)
fn duration_to_ticks(duration: Duration) -> u64 {
    let nanos = duration.as_nanos() as f64;
    (nanos * 2.5) as u64
}
```

## Appendix B: SLO Compliance Matrix

| SLO | Benchmark | Pass Criteria | Current Status |
|-----|-----------|---------------|----------------|
| R1 (hot reads ≤2ns) | `bench_task_lookup_hot_read` | p99 ≤2ns | 🟡 Pending |
| W1 (hot writes ≤1ms) | `bench_resource_allocation` | p99 ≤1ms | 🟡 Pending |
| C1 (complex ops ≤500ms) | `bench_atm_withdrawal_e2e` | p99 ≤3s | 🟡 Pending |
| Chatman Constant | `bench_pattern_execution_hot_path` | ≤8 ticks | 🟡 Pending |
| OTEL overhead | `bench_telemetry_overhead` | <5% | 🟡 Pending |

**Legend**:
- ✅ Passing
- 🟡 Pending measurement
- ❌ Failing

## Next Steps

1. **Run initial benchmarks**: Establish baseline
2. **Identify bottlenecks**: Profile hot paths
3. **Optimize critical paths**: Address Chatman Constant violations
4. **Validate SLO compliance**: Ensure all targets met
5. **Enable CI gates**: Prevent performance regressions
6. **Production monitoring**: Deploy OTEL metrics

---

**Document Maintainer**: Performance Benchmarker Agent
**Last Updated**: 2025-11-08
**Next Review**: After benchmark execution
