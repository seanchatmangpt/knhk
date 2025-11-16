# KNHK Performance Validation Suite

## Quick Start

```bash
# Run all performance tests
./scripts/run-comprehensive-performance-tests.sh

# Run with detailed benchmarks
RUN_BENCHMARKS=1 ./scripts/run-comprehensive-performance-tests.sh

# Or use Make
make test-performance
```

## What This Suite Validates

The KNHK Performance Validation Suite enforces the **Chatman constant** (τ ≤ 8 ticks) and validates all critical performance SLOs:

| SLO | Target | Status |
|-----|--------|--------|
| Hot Path Decision Latency | ≤ 8 ticks | ✅ Enforced |
| Warm Path Adaptation | ≤ 1 second (small Δ) | ✅ Enforced |
| Receipt Availability | ≤ 50ms | ✅ Enforced |
| Σ Pointer Updates | Atomic, monotonic | ✅ Enforced |

## Test Suites

### 1. Hot Path Latency Tests (`tests/performance/hot_path_latency_test.rs`)

Validates that ALL hot path operations complete within the Chatman constant:

```rust
#[test]
fn test_hot_path_ask_sp_latency() {
    // Tests ASK(S,P) queries with 10,000 iterations
    // Enforces: max_ticks ≤ 8, violations = 0
}
```

**Coverage**: ASK, COUNT, VALIDATE, composite operations, under-load scenarios

### 2. Warm Path Adaptation Tests (`tests/performance/warm_path_adaptation_test.rs`)

Validates MAPE-K autonomic cycle performance:

```rust
#[test]
fn test_small_delta_adaptation_latency() {
    // Tests Monitor-Analyze-Plan-Execute-Knowledge cycle
    // Enforces: duration ≤ 1000ms for small workload changes
}
```

**Coverage**: Monitor, Analyze, Plan, Execute phases, varying workload sizes

### 3. Receipt Generation Tests (`tests/performance/receipt_generation_test.rs`)

Validates receipt availability and Σ pointer atomicity:

```rust
#[test]
fn test_sigma_pointer_update_atomicity() {
    // Tests 100 concurrent receipt generations
    // Enforces: monotonic Σ pointer, latency ≤ 50ms
}
```

**Coverage**: Single/batch generation, high load, atomicity, hash performance

### 4. Chatman Constant Enforcement (`tests/performance/chatman_constant_enforcement_test.rs`)

Comprehensive validation of τ ≤ 8 across ALL primitives:

```rust
#[test]
fn test_chatman_constant_comprehensive() {
    // Tests: boolean checks, array lookups, hash lookups,
    //        bit manipulation, range checks, conditionals
    // Enforces: ZERO violations across 6,000+ operations
}
```

**Coverage**: All hot path primitives, parking decisions, budget tracking

## Benchmarks

### Hook Execution (`benches/performance/hook_execution_bench.rs`)
- Pre/post task hooks
- Pre/post edit hooks
- Metadata scaling
- Total coordination overhead

### Pattern Library (`benches/performance/pattern_library_bench.rs`)
- Pattern lookup by tag
- Pattern matching with guards
- Batch matching
- Index scanning (10-1000 patterns)

### Guard Evaluation (`benches/performance/guard_evaluation_bench.rs`)
- Tick budget guards
- Data size guards
- Composite guards (nested)
- Batch evaluation (1-50 guards)

### MAPE-K Cycle (`benches/performance/mape_k_cycle_bench.rs`)
- Individual phase benchmarks
- Complete cycle performance
- Scaling behavior (10-1000 metrics)

## Documentation

- **[Performance Validation Guide](performance-validation.md)** - Complete guide to testing and benchmarking
- **[Implementation Summary](IMPLEMENTATION_SUMMARY.md)** - Detailed implementation notes
- **[Test Reports](../../reports/performance/)** - Auto-generated test results

## Architecture

### Tick Measurement Infrastructure

The suite uses hardware performance counters for cycle-accurate measurements:

```rust
// Platform-specific tick counting
#[cfg(target_arch = "x86_64")]
pub fn rdtsc() -> u64 {
    unsafe { _rdtsc() }
}

#[cfg(target_arch = "aarch64")]
pub fn rdtsc() -> u64 {
    let val: u64;
    unsafe { std::arch::asm!("mrs {}, cntvct_el0", out(reg) val); }
    val
}
```

**Precision**: Sub-nanosecond using CPU cycle counters
**Overhead**: ~2-3 CPU cycles (negligible)
**Platforms**: x86_64, aarch64, with fallback for others

### Enforcement Model

1. **Measure**: Execute operation and capture tick count
2. **Analyze**: Calculate statistics (min, max, p50, p95, p99)
3. **Enforce**: Assert zero violations of Chatman constant
4. **Report**: Generate detailed performance reports

## Interpreting Results

### Successful Test

```
=== Hot Path ASK(S,P) Latency Test ===
ASK(S,P): min=2 p50=3 p95=5 p99=6 max=7 violations=0/10000 - ✅ SLO MET

test test_hot_path_ask_sp_latency ... ok
```

- **max ≤ 8**: Maximum latency within budget ✅
- **violations = 0**: Zero SLO violations ✅
- **p99 ≤ 8**: Tail latency acceptable ✅

### Failed Test

```
ASK(S,P): min=2 p50=4 p95=9 p99=12 max=15 violations=42/10000 - ❌ SLO VIOLATED

thread 'test_hot_path_ask_sp_latency' panicked at:
ASK(S,P) violated Chatman constant: max=15 ticks, violations=42/10000
```

- **max > 8**: Maximum latency exceeds budget ❌
- **violations > 0**: SLO violated 42 times ❌
- **Action Required**: Optimize hot path implementation

## CI/CD Integration

The test suite is designed for continuous validation:

```yaml
# .github/workflows/performance.yml
- name: Run Performance Tests
  run: ./scripts/run-comprehensive-performance-tests.sh

- name: Upload Performance Report
  uses: actions/upload-artifact@v3
  with:
    name: performance-report
    path: reports/performance/
```

**Exit Codes**:
- `0` - All tests passed
- `1` - One or more tests failed

## Performance Budgets

Hot path tick allocation for τ = 8:

| Operation | Budget | Notes |
|-----------|--------|-------|
| Guard evaluation | 2 ticks | Check if operation allowed |
| Data lookup | 3 ticks | L1 cache access |
| Decision logic | 2 ticks | Apply decision rules |
| Receipt stub | 1 tick | Mark for async generation |
| **Total** | **8 ticks** | **Maximum allowed** |

## Troubleshooting

### Common Issues

**Issue**: Tests fail with "max > 8 ticks"
**Cause**: Hot path implementation exceeds latency budget
**Fix**: Profile code with `perf` and optimize critical path

**Issue**: High p99 latency but low mean
**Cause**: Tail latency issues (cache misses, branch misprediction)
**Fix**: Optimize worst-case scenarios, add prefetching

**Issue**: Violations occur randomly
**Cause**: System noise (interrupts, CPU throttling)
**Fix**: Run tests with CPU isolation, disable frequency scaling

### Debug Mode

For detailed analysis, enable verbose output:

```bash
RUST_LOG=debug ./scripts/run-comprehensive-performance-tests.sh
```

## Future Enhancements

- [ ] Real-time performance dashboards
- [ ] Regression detection with automatic alerts
- [ ] Hardware-specific auto-tuning
- [ ] Integrated flamegraph generation
- [ ] ML-based performance prediction

## References

- **Law**: μ ⊂ τ ; τ ≤ 8 ticks (Chatman Constant)
- **Test Directory**: `/home/user/knhk/tests/performance/`
- **Benchmark Directory**: `/home/user/knhk/benches/performance/`
- **Reports Directory**: `/home/user/knhk/reports/performance/`
- **Documentation**: This directory

---

**Last Updated**: 2025-11-16
**Maintainer**: KNHK Performance Team
**Status**: ✅ Production Ready
