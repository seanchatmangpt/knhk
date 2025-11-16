# KNHK Performance Validation Suite

## Overview

The KNHK performance validation suite provides comprehensive testing and benchmarking to enforce the **Chatman constant** (τ ≤ 8 ticks) and other performance SLOs critical to the system's operation.

## Chatman Constant (τ ≤ 8)

The Chatman constant defines the maximum latency allowed for hot path operations:

```
Law: μ ⊂ τ ; τ ≤ 8 ticks
```

Where:
- **μ** = actual decision latency
- **τ** = maximum allowed latency (8 ticks)
- **1 tick** = 1 nanosecond @ 1GHz reference clock

### Why 8 Ticks?

The 8-tick limit ensures that hot path decisions complete within the L1 cache access time on modern CPUs, guaranteeing:
- Predictable, deterministic latency
- Zero context switches
- Maximum throughput
- Minimal resource contention

## Performance SLOs

| Component | SLO | Test Coverage |
|-----------|-----|---------------|
| **Hot Path Latency** | ≤ 8 ticks | `hot_path_latency_test.rs` |
| **Warm Path Adaptation** | ≤ 1 second (small Δ) | `warm_path_adaptation_test.rs` |
| **Receipt Generation** | ≤ 50ms | `receipt_generation_test.rs` |
| **Σ Pointer Updates** | Atomic, auditable | `receipt_generation_test.rs` |
| **MAPE-K Cycle** | Sub-second | `warm_path_adaptation_test.rs` |

## Test Structure

### Core Tests (`tests/performance/`)

#### 1. Tick Measurement Infrastructure (`tick_measurement.rs`)

Provides precise tick counting using hardware performance counters:

```rust
use tick_measurement::{measure_ticks, TickStatistics};

let (result, measurement) = measure_ticks(|| {
    // Your hot path code here
});

assert!(!measurement.exceeds_budget()); // Must be ≤ 8 ticks
```

**Key Components:**
- `rdtsc()` - Read hardware cycle counter (x86_64/aarch64)
- `cycles_to_ticks()` - Convert CPU cycles to KNHK ticks
- `TickMeasurement` - Single measurement with violation detection
- `TickStatistics` - Aggregate statistics (min, max, p50, p95, p99, violations)

#### 2. Hot Path Latency Tests (`hot_path_latency_test.rs`)

Validates all hot path operations meet τ ≤ 8:

```rust
#[test]
fn test_hot_path_ask_sp_latency() {
    // Measures ASK(S,P) query latency
    // Enforces: max_ticks ≤ 8
}
```

**Coverage:**
- ASK(S,P) queries
- COUNT(S,P) queries
- VALIDATE(S,P) checks
- ASK(S,P,O) exact triple matching
- Composite operations
- Under-load scenarios

#### 3. Warm Path Adaptation Tests (`warm_path_adaptation_test.rs`)

Validates MAPE-K cycle performance:

```rust
#[test]
fn test_small_delta_adaptation_latency() {
    // Measures adaptation time for small workload changes
    // Enforces: duration ≤ 1000ms
}
```

**Coverage:**
- Monitor-Analyze-Plan-Execute-Knowledge cycle
- Small delta adaptations (Δ < 100 triples)
- Medium delta adaptations (Δ < 500 triples)
- No-change scenarios
- Repeated adaptations

#### 4. Receipt Generation Tests (`receipt_generation_test.rs`)

Validates receipt availability SLO:

```rust
#[test]
fn test_single_receipt_generation() {
    // Measures receipt generation latency
    // Enforces: duration ≤ 50ms
}
```

**Coverage:**
- Single receipt generation
- Batch receipt generation
- High-load scenarios
- Hash computation performance
- Σ pointer atomicity

#### 5. Chatman Constant Enforcement (`chatman_constant_enforcement_test.rs`)

Comprehensive validation of τ ≤ 8 across all operations:

```rust
#[test]
fn test_chatman_constant_comprehensive() {
    // Tests all primitive operations
    // Enforces: ZERO violations allowed
}
```

**Coverage:**
- Boolean checks
- Array lookups
- Hash lookups
- Bit manipulation
- Range checks
- Conditional assignments
- Parking decision overhead
- Tick budget tracking

### Benchmarks (`benches/performance/`)

#### 1. Hook Execution Benchmarks (`hook_execution_bench.rs`)

Measures coordination overhead:
- Pre-task hooks
- Post-task hooks
- Pre-edit hooks
- Post-edit hooks
- Metadata scaling
- Total hook overhead

#### 2. Pattern Library Benchmarks (`pattern_library_bench.rs`)

Measures pattern matching performance:
- Pattern lookup by tag
- Pattern matching
- Pattern addition
- Batch matching
- Index scanning

#### 3. Guard Evaluation Benchmarks (`guard_evaluation_bench.rs`)

Measures guard evaluation performance:
- Tick budget guards
- Data size guards
- Query complexity guards
- Composite guards
- Batch evaluation
- Nested guards

#### 4. MAPE-K Cycle Benchmarks (`mape_k_cycle_bench.rs`)

Measures autonomic loop performance:
- Monitor phase
- Analyze phase
- Plan phase
- Execute phase
- Complete cycle
- Scaling behavior

## Running Tests

### Quick Start

```bash
# Run all performance tests
make test-performance

# Or use the comprehensive test runner
./scripts/run-comprehensive-performance-tests.sh
```

### Individual Test Suites

```bash
# Hot path latency tests
cargo test --test hot_path_latency_test --release -- --nocapture

# Warm path adaptation tests
cargo test --test warm_path_adaptation_test --release -- --nocapture

# Receipt generation tests
cargo test --test receipt_generation_test --release -- --nocapture

# Chatman constant enforcement
cargo test --test chatman_constant_enforcement_test --release -- --nocapture
```

### Run Benchmarks

```bash
# Run all benchmarks
RUN_BENCHMARKS=1 ./scripts/run-comprehensive-performance-tests.sh

# Individual benchmarks
cargo bench --bench hook_execution_bench
cargo bench --bench pattern_library_bench
cargo bench --bench guard_evaluation_bench
cargo bench --bench mape_k_cycle_bench
```

## Interpreting Results

### Test Output Format

```
=== Hot Path ASK(S,P) Latency Test ===
ASK(S,P): min=2 p50=3 p95=5 p99=6 max=7 mean=3.45 violations=0/10000 - ✅ SLO MET

test test_hot_path_ask_sp_latency ... ok
```

**Key Metrics:**
- **min**: Minimum observed ticks
- **p50**: Median (50th percentile)
- **p95**: 95th percentile
- **p99**: 99th percentile
- **max**: Maximum observed ticks (**MUST be ≤ 8**)
- **violations**: Count of measurements exceeding 8 ticks (**MUST be 0**)

### SLO Compliance

✅ **PASS**: `max ≤ 8` AND `violations = 0`
❌ **FAIL**: `max > 8` OR `violations > 0`

### Performance Report

After running the comprehensive test suite, a detailed report is generated:

```
reports/performance/summary_YYYYMMDD_HHMMSS.md
```

This includes:
- Overall pass/fail status
- Individual test results
- SLO compliance matrix
- Next steps and recommendations

## Continuous Integration

Performance tests are integrated into CI/CD:

```yaml
# .github/workflows/performance.yml
- name: Run Performance Tests
  run: make test-performance

- name: Upload Performance Report
  uses: actions/upload-artifact@v3
  with:
    name: performance-report
    path: reports/performance/
```

## Debugging Performance Issues

### Identifying Violations

If a test fails with violations:

```
ASK(S,P): min=2 p50=4 p95=8 p99=12 max=15 violations=42/10000 - ❌ SLO VIOLATED
```

1. **Check p99**: If p99 > 8 but max is much higher, you have tail latency issues
2. **Analyze violation pattern**: Are violations random or consistent?
3. **Profile the hot path**: Use `perf` or `flamegraph` to identify bottlenecks

### Common Causes

1. **Branch misprediction**: Too many conditional branches in hot path
2. **Cache misses**: Data not in L1 cache
3. **Memory allocation**: Hot path must be allocation-free
4. **Syscalls**: Any syscall will violate the budget
5. **Lock contention**: Hot path must be lock-free

### Optimization Strategies

1. **Branchless code**: Use bit manipulation instead of conditionals
2. **Cache-friendly data structures**: Structure-of-arrays (SoA) instead of array-of-structures
3. **Prefetching**: Explicit prefetch instructions for predictable access patterns
4. **SIMD**: Use vector instructions for parallel operations
5. **Inlining**: Mark critical functions with `#[inline(always)]`

## Architecture Integration

### C Integration (PMU)

The C performance monitoring unit provides hardware-level tick counting:

```c
#include "knhk/pmu.h"

knhk_pmu_measurement_t m = knhk_pmu_start();
// ... hot path code ...
knhk_pmu_end(&m);

assert(m.elapsed_ticks <= 8);
```

### Rust Integration

Rust tests use the `tick_measurement` module:

```rust
mod tick_measurement;

use tick_measurement::measure_and_assert_budget;

measure_and_assert_budget("operation_name", || {
    // ... hot path code ...
});
```

## Performance Budgets

### Tick Budget Breakdown

For a complete hot path decision with τ = 8:

| Operation | Budget | Notes |
|-----------|--------|-------|
| Guard evaluation | 2 ticks | Check if operation is allowed |
| Data lookup | 3 ticks | Find relevant data in cache |
| Decision logic | 2 ticks | Apply decision rules |
| Receipt generation | 1 tick | Generate decision receipt |
| **Total** | **8 ticks** | **Maximum allowed** |

### Exceeding the Budget

When an operation would exceed 8 ticks:
1. **Park the request**: Move to warm/cold path
2. **Generate parking receipt**: Document why it was parked
3. **Continue processing**: Warm/cold path handles the request

## Future Enhancements

1. **Real-time monitoring**: Live performance dashboards
2. **Regression detection**: Automatic alerts when SLOs are violated
3. **Hardware-specific tuning**: Auto-detect CPU frequency and adjust tick calculations
4. **Performance profiling**: Integrated flamegraphs and perf analysis
5. **Machine learning**: Predict performance issues before they occur

## References

- `/home/user/knhk/tests/performance/` - Test implementations
- `/home/user/knhk/benches/performance/` - Benchmark implementations
- `/home/user/knhk/c/include/knhk/pmu.h` - C PMU header
- `/home/user/knhk/scripts/run-comprehensive-performance-tests.sh` - Test runner
- [Chatman Equation Specification](../../chatman-equation/README.md)

---

**Last Updated**: 2025-11-16
**Maintainer**: KNHK Performance Team
