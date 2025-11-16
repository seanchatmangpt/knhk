# KNHK Performance Benchmarking Guide

## Overview

This guide covers the comprehensive performance profiling suite for the KNHK closed-loop system. All benchmarks validate against the **Chatman Constant** (≤8 ticks for hot path operations).

## Running Benchmarks

### Basic Execution

```bash
# Run all benchmarks
cargo bench --package knhk-closed-loop

# Run specific benchmark group
cargo bench --package knhk-closed-loop -- snapshot_promotion
cargo bench --package knhk-closed-loop -- doctrine_validation
cargo bench --package knhk-closed-loop -- pattern_detection

# Run with increased sample size
cargo bench --package knhk-closed-loop -- --sample-size 10000
```

### Generate HTML Reports

```bash
# Benchmarks automatically generate HTML reports
cargo bench --package knhk-closed-loop

# View reports at:
open target/criterion/report/index.html
```

## Benchmark Categories

### 1. Atomic Snapshot Promotion (Target: <10ns)

**Critical Path:** This is the hottest of hot paths - atomic pointer swap using RCU semantics.

```bash
cargo bench --package knhk-closed-loop -- snapshot_promotion
```

**Expected Performance:**
- `atomic_swap`: 5-15 ns (raw pointer swap)
- `atomic_swap_with_stats`: 20-50 ns (with atomic counter updates)
- `concurrent_promotion_10_threads`: Should scale linearly, no lock contention

**Pass Criteria:**
- ✅ PASS: Promotion latency < 100 ns (well under 1 tick)
- ✅ PASS: No lock contention visible in concurrent test
- ✅ PASS: Stats tracking overhead < 40 ns

**Failure Indicators:**
- ❌ Latency > 1 µs indicates memory allocation or synchronization issue
- ❌ Non-linear scaling in concurrent test indicates lock contention

### 2. Doctrine Validation (Target: <100ns per rule, <8 ticks total)

**Critical Path:** Q invariant checking must be extremely fast.

```bash
cargo bench --package knhk-closed-loop -- doctrine_validation
```

**Expected Performance:**
- `validate_single_doctrine_pass`: 50-150 ns
- `validate_single_doctrine_fail`: 50-150 ns (early exit on failure)
- `validate_5_doctrines`: 250-750 ns (5 rules × ~150ns = 750ns max)
- `validate_n_doctrines`: Linear scaling up to ~100 rules

**Pass Criteria:**
- ✅ PASS: Single doctrine < 200 ns
- ✅ PASS: 5 doctrines < 1 µs (≤8 ticks equivalent)
- ✅ PASS: Linear scaling O(n) with rule count
- ✅ PASS: Pass and fail paths have similar latency (no branch prediction bias)

**Scalability Analysis:**
- 1 rule: ~100 ns
- 5 rules: ~500 ns
- 10 rules: ~1 µs
- 50 rules: ~5 µs (acceptable for warm path)
- 100 rules: ~10 µs (warm path only)

**Failure Indicators:**
- ❌ Single rule > 500 ns: Check for unnecessary allocations
- ❌ Non-linear scaling: Look for O(n²) algorithms
- ❌ Large difference between pass/fail: Branch prediction issues

### 3. Pattern Detection (Target: <1µs for hot path)

**Critical Path:** Analyzing observations to detect patterns.

```bash
cargo bench --package knhk-closed-loop -- pattern_detection
```

**Expected Performance:**
- 10 observations: <500 ns
- 100 observations: <5 µs
- 1000 observations: <50 µs
- 10000 observations: <500 µs (warm path acceptable)

**Pass Criteria:**
- ✅ PASS: O(n) or O(n log n) scaling
- ✅ PASS: No memory allocation spikes
- ✅ PASS: Hot path (recent observations) < 1 µs

**Optimization Opportunities:**
- Use windowing (analyze only last N observations)
- Cache pattern detection results
- Use approximate algorithms for large datasets

### 4. Receipt Signing & Verification (Target: <1ms signing, <500µs verify)

**Cryptographic Operations:** Ed25519 signature generation and verification.

```bash
cargo bench --package knhk-closed-loop -- receipt_operations
```

**Expected Performance:**
- `receipt_signing`: 50-200 µs (Ed25519 is fast)
- `receipt_verification`: 100-300 µs (signature verification)
- `receipt_batch_sign_100`: 5-20 ms (100 signatures)

**Pass Criteria:**
- ✅ PASS: Single signature < 1 ms
- ✅ PASS: Verification < 500 µs
- ✅ PASS: Batch operations show linear scaling

**Note:** Cryptographic operations are inherently slower but are NOT on the hot path for observation ingestion.

### 5. Guard Enforcement Check (Target: <50ns)

**Critical Path:** Checking if invariants hold.

```bash
cargo bench --package knhk-closed-loop -- guard_enforcement
```

**Expected Performance:**
- `guard_check_all_pass`: 5-20 ns (just boolean checks)
- `guard_check_one_fail`: 5-20 ns (early exit)
- `guard_identify_violations`: 20-50 ns (collect violations)

**Pass Criteria:**
- ✅ PASS: All checks < 50 ns
- ✅ PASS: Branch-free or highly predictable branches
- ✅ PASS: No allocations for common cases

### 6. Complete MAPE-K Cycle (Target: within latency budget)

**Full Autonomic Loop:** Monitor → Analyze → Plan → Execute → Knowledge

```bash
cargo bench --package knhk-closed-loop -- mapek_cycle
```

**Expected Performance:**
- `cycle_empty`: 100-500 µs (no patterns detected)
- `cycle_with_patterns`: 500 µs - 5 ms (with pattern detection + validation)

**Pass Criteria:**
- ✅ PASS: Empty cycle < 1 ms
- ✅ PASS: Pattern cycle < 10 ms
- ✅ PASS: All receipts generated correctly
- ✅ PASS: No panics or errors

**Breakdown by Phase:**
- Monitor: <100 µs
- Analyze: 200 µs - 2 ms (pattern detection)
- Plan: 100-500 µs (proposal generation)
- Execute: 500 µs - 2 ms (validation + promotion)
- Knowledge: <100 µs (receipt creation)

### 7. Real-World Scenarios

**Industry-Specific Workloads:**

```bash
cargo bench --package knhk-closed-loop -- real_world_scenarios
```

#### Finance: 1000 Transactions/Second Validation

**Scenario:** High-frequency trading system validating 1000 txn/sec against 5 doctrines.

**Expected Performance:**
- Total: <10 ms for 1000 validations
- Per transaction: <10 µs

**Pass Criteria:**
- ✅ PASS: Can sustain 1000 txn/sec (1 ms per txn budget)
- ✅ PASS: P99 latency < 50 µs
- ✅ PASS: No GC pauses or allocation spikes

#### Healthcare: Compliance Checking

**Scenario:** HIPAA-compliant access control for 100 patient records.

**Expected Performance:**
- Total: <50 ms for 100 checks
- Per check: <500 µs

**Pass Criteria:**
- ✅ PASS: Time-window validation < 200 µs per check
- ✅ PASS: No false negatives (compliance failures)
- ✅ PASS: Deterministic results (same input → same output)

#### Manufacturing: Sensor Pattern Detection

**Scenario:** 1000 sensor readings across 100 sensors, detecting anomalies.

**Expected Performance:**
- Total: <100 ms for 1000 readings
- Per sensor: <1 ms

**Pass Criteria:**
- ✅ PASS: Pattern detection scales O(n) with sensor count
- ✅ PASS: Real-time detection (< 10 ms from ingestion to pattern)
- ✅ PASS: Memory bounded (no unbounded growth)

#### Logistics: Route Optimization Validation

**Scenario:** 1000 shipments validated against SLA constraints.

**Expected Performance:**
- Total: <50 ms for 1000 validations
- Per shipment: <50 µs

**Pass Criteria:**
- ✅ PASS: Resource limit checking < 100 µs
- ✅ PASS: Deterministic validation
- ✅ PASS: No false positives (incorrect rejections)

### 8. Latency Percentiles Analysis

**Statistical Distribution:** P50, P75, P90, P95, P99, P99.9 latencies.

```bash
cargo bench --package knhk-closed-loop -- latency_percentiles
```

**Expected Distribution (Snapshot Promotion):**
- P50: 5-10 ns
- P75: 8-15 ns
- P90: 10-20 ns
- P95: 15-30 ns
- P99: 20-50 ns
- P99.9: 50-200 ns

**Pass Criteria:**
- ✅ PASS: P99 < 1 µs (no outliers)
- ✅ PASS: P99.9 < 10 µs (rare GC pauses acceptable)
- ✅ PASS: Tight distribution (low variance)

**Outlier Detection:**
- Look for bimodal distributions (indicates GC or other interruptions)
- Check for long tail (indicates lock contention or allocation)
- Verify P99/P50 ratio < 10 (low variance)

### 9. Memory Allocation Patterns

**Allocation Profiling:** Detect unnecessary allocations on hot paths.

```bash
cargo bench --package knhk-closed-loop -- memory_allocation
```

**Expected Behavior:**
- Observation append: ~200 bytes per observation (minimal)
- Doctrine store: ~500 bytes per rule (reasonable)
- No allocations on read-only operations

**Pass Criteria:**
- ✅ PASS: Read operations have zero allocations
- ✅ PASS: Write operations have bounded allocations
- ✅ PASS: No unbounded growth (memory leaks)

## Interpreting Results

### Chatman Constant Compliance

**The 8-Tick Rule:**
```
1 tick ≈ 100-200 ns (modern CPU @ 3-5 GHz)
8 ticks ≈ 800-1600 ns ≈ 1.5 µs

Hot path operations MUST complete within 8 ticks.
```

**Validation:**
- ✅ Snapshot promotion: ~10 ns → **0.05 ticks** ✓
- ✅ Guard check: ~20 ns → **0.1 ticks** ✓
- ✅ Single doctrine: ~100 ns → **0.5 ticks** ✓
- ✅ 5 doctrines: ~500 ns → **2.5 ticks** ✓
- ✅ Pattern detection (100 obs): ~5 µs → **25 ticks** (warm path OK)

### Performance Regression Detection

**Continuous Benchmarking:**

```bash
# Baseline (before changes)
cargo bench --package knhk-closed-loop -- --save-baseline main

# After changes
cargo bench --package knhk-closed-loop -- --baseline main

# Criterion will show % change for each benchmark
```

**Red Flags:**
- ❌ Any hot path operation > 10% slower
- ❌ Allocations increased
- ❌ Variance increased (unstable performance)
- ❌ P99 latency increased > 20%

### Bottleneck Identification

**Common Issues:**

1. **High P99 Latency:**
   - Cause: GC pauses, lock contention, allocation
   - Fix: Reduce allocations, use lock-free structures

2. **Non-Linear Scaling:**
   - Cause: O(n²) algorithms, repeated work
   - Fix: Use caching, better data structures

3. **High Variance:**
   - Cause: Branch prediction misses, cache misses
   - Fix: Improve locality, reduce branching

4. **Memory Growth:**
   - Cause: Unbounded data structures, leaks
   - Fix: Implement bounds, use bounded queues

## Optimization Guidelines

### Hot Path Optimization

**Priority 1: Zero Allocations**
```rust
// ❌ BAD: Allocates on hot path
fn get_value(&self) -> String {
    format!("value: {}", self.inner)
}

// ✅ GOOD: Returns reference or Copy type
fn get_value(&self) -> &str {
    &self.inner
}
```

**Priority 2: Lock-Free When Possible**
```rust
// ❌ BAD: Uses mutex on hot path
let value = self.data.lock().unwrap().clone();

// ✅ GOOD: Uses atomic or lock-free structure
let value = self.data.load(Ordering::Acquire);
```

**Priority 3: Branch Prediction Friendly**
```rust
// ❌ BAD: Unpredictable branch
if random_condition() {
    hot_path();
}

// ✅ GOOD: Predictable branch (common case first)
if likely_true_condition() {
    hot_path();
} else {
    rare_path();
}
```

### Warm Path Optimization

**Acceptable Trade-offs:**
- Small allocations OK (< 1KB)
- Occasional locks OK (< 10% contention)
- O(n log n) algorithms acceptable

**Focus:**
- Minimize P99 latency
- Predictable performance
- Bounded resource usage

### Cold Path

**No Restrictions:**
- Allocations fine
- Complex algorithms OK
- Focus on correctness over speed

## Continuous Performance Validation

### CI Integration

```yaml
# .github/workflows/performance.yml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench --package knhk-closed-loop -- --save-baseline pr
      - name: Compare with main
        run: cargo bench --package knhk-closed-loop -- --baseline main
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
```

### Performance SLOs

**Service Level Objectives:**

| Operation | Target | Alert Threshold |
|-----------|--------|-----------------|
| Snapshot Promotion | < 50 ns | > 500 ns |
| Single Doctrine | < 200 ns | > 1 µs |
| 5 Doctrines | < 1 µs | > 5 µs |
| Guard Check | < 50 ns | > 200 ns |
| Empty MAPE-K Cycle | < 1 ms | > 5 ms |
| Pattern MAPE-K Cycle | < 10 ms | > 50 ms |

**Alerting:**
- P99 latency exceeds alert threshold
- Performance regression > 20% from baseline
- Memory usage unbounded growth
- Allocation rate on hot path > 0

## Troubleshooting

### Benchmark Variability

**Problem:** Results vary widely between runs.

**Solutions:**
1. Increase sample size: `--sample-size 1000`
2. Increase measurement time: `--measurement-time 30`
3. Disable CPU frequency scaling
4. Close background applications
5. Pin benchmarks to specific CPU cores

### Slow Benchmarks

**Problem:** Benchmarks take too long.

**Solutions:**
1. Reduce sample size for quick feedback
2. Run specific benchmark groups only
3. Use `--quick` mode (less accurate but faster)
4. Parallelize with `--jobs N`

### Misleading Results

**Problem:** Benchmarks show incorrect performance.

**Causes:**
1. Compiler optimizing away code (`black_box` missing)
2. Dead code elimination
3. Constant folding
4. Branch prediction training

**Solution:** Use `black_box()` on all inputs and outputs.

## Reporting

### Generate Performance Report

```bash
# Run all benchmarks
cargo bench --package knhk-closed-loop

# Results are in:
# - Console output: Summary statistics
# - HTML reports: target/criterion/report/index.html
# - JSON data: target/criterion/<benchmark>/base/estimates.json
```

### Sample Report Format

```markdown
# Performance Report - KNHK Closed Loop

**Date:** 2025-11-16
**Commit:** abc123def
**Hardware:** Apple M1 Pro, 32GB RAM

## Summary

| Benchmark | Target | Actual | Status |
|-----------|--------|--------|--------|
| Snapshot Promotion | < 50 ns | 12 ns | ✅ PASS |
| Single Doctrine | < 200 ns | 87 ns | ✅ PASS |
| 5 Doctrines | < 1 µs | 435 ns | ✅ PASS |
| Guard Check | < 50 ns | 18 ns | ✅ PASS |
| MAPE-K Empty | < 1 ms | 234 µs | ✅ PASS |
| MAPE-K Pattern | < 10 ms | 3.2 ms | ✅ PASS |

## Chatman Constant Compliance

All hot path operations complete within 8 ticks (1.5 µs):
- Snapshot promotion: 0.06 ticks ✓
- Guard enforcement: 0.09 ticks ✓
- Doctrine validation: 2.2 ticks ✓

## Recommendations

1. ✅ No optimizations needed - all targets met
2. 📊 Monitor P99 latencies in production
3. 🔄 Re-benchmark after major changes
```

## Next Steps

1. **Run Initial Baseline:**
   ```bash
   cargo bench --package knhk-closed-loop -- --save-baseline initial
   ```

2. **Monitor for Regressions:**
   ```bash
   cargo bench --package knhk-closed-loop -- --baseline initial
   ```

3. **Profile Hotspots:**
   ```bash
   cargo flamegraph --bench performance_profile
   ```

4. **Validate in Production:**
   - Deploy to staging
   - Monitor actual P99 latencies
   - Compare with benchmark predictions
   - Adjust SLOs based on real data

---

**Remember:** Benchmarks predict performance, but production telemetry is the source of truth. Always validate with real workloads.
