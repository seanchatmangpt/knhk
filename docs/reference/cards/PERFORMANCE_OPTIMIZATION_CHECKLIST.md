# Performance Optimization Checklist

**Purpose**: One-page guide for optimizing KNHK performance
**Target**: Hot path ≤8 ticks (Chatman Constant), Warm path ≤500µs
**Validation**: `make test-performance-v04` must pass

---

## 1. Hot Path Constraints (≤8 Ticks / ≤2ns)

**Chatman Constant compliance for critical operations:**

- [ ] ASK queries: ≤1.5 ns measured
- [ ] COUNT queries: ≤1.5 ns measured
- [ ] COMPARE operations: ≤1.5 ns measured
- [ ] VALIDATE operations: ≤2.0 ns measured
- [ ] All hot path operations verified with `make test-performance-v04`
- [ ] No heap allocations in hot path (stack-only)
- [ ] Branchless C engine implementation verified
- [ ] SIMD operations for 8-lane parallel processing

**Pass Criteria**: All 8 items checked ✅

**Expected Improvement**: 10,000-100,000x faster than traditional SPARQL engines

---

## 2. Warm Path Budget (≤500µs)

**CONSTRUCT8 and emit operations:**

- [ ] CONSTRUCT8 operations: ≤500 µs measured
- [ ] Workflow steps: ≤500 µs per step (hot workflow)
- [ ] Cache lookups: ≤100 µs (MPHF cache)
- [ ] Schema validation: ≤200 µs (Weaver check)
- [ ] Graph traversal: ≤500 µs (limited depth)
- [ ] No unnecessary allocations (use object pools)
- [ ] Batch operations where possible (reduce overhead)
- [ ] Async I/O for network calls (non-blocking)

**Pass Criteria**: All 8 items checked ✅

**Expected Improvement**: 100-1000x faster than cold path

---

## 3. Memory Optimization

**Minimize allocations and memory footprint:**

- [ ] Hot path uses SoA (Structure of Arrays) for cache locality
- [ ] 64-byte alignment for SIMD operations (AVX-512)
- [ ] Pre-allocated buffers for known sizes (8-lane arrays)
- [ ] Object pooling for frequently allocated objects
- [ ] Arena allocators for temporary data
- [ ] Memory-mapped files for large datasets
- [ ] Compact data structures (bit-packing, compression)
- [ ] Memory profiling completed (Valgrind, Heaptrack)

**Pass Criteria**: All 8 items checked ✅

**Expected Improvement**: 50-90% memory reduction

---

## 4. CPU Optimization

**Maximize CPU efficiency:**

- [ ] Branchless code in hot path (no if/else)
- [ ] SIMD intrinsics for parallel operations (AVX-512)
- [ ] Loop unrolling for predictable iterations
- [ ] Prefetching for sequential memory access
- [ ] Cache-friendly data layouts (SoA, alignment)
- [ ] CPU affinity for critical threads (isolate cores)
- [ ] Lock-free data structures where possible
- [ ] Profile-guided optimization (PGO) enabled

**Pass Criteria**: All 8 items checked ✅

**Expected Improvement**: 2-5x CPU efficiency

---

## 5. I/O Optimization

**Minimize I/O latency:**

- [ ] Async I/O for all network operations (Tokio)
- [ ] Connection pooling for databases (deadpool, r2d2)
- [ ] Batch reads/writes to reduce syscalls
- [ ] Zero-copy I/O where possible (sendfile, splice)
- [ ] Buffered I/O for sequential access
- [ ] Direct I/O for random access (bypass page cache)
- [ ] I/O profiling completed (iotop, iostat)
- [ ] Network TCP tuning (TCP_NODELAY, buffer sizes)

**Pass Criteria**: All 8 items checked ✅

**Expected Improvement**: 10-100x I/O throughput

---

## 6. Caching Strategy

**Reduce redundant computation:**

- [ ] MPHF (Minimal Perfect Hash Function) cache for hot queries
- [ ] LRU cache for warm queries
- [ ] TTL-based cache invalidation
- [ ] Cache hit rate ≥90% measured
- [ ] Cache warming on startup (pre-populate)
- [ ] Cache eviction policy tuned (size, TTL)
- [ ] Cache coherence for distributed systems
- [ ] Cache metrics monitored (hits, misses, evictions)

**Pass Criteria**: All 8 items checked ✅

**Expected Improvement**: 100-1000x for cache hits

---

## 7. Profiling & Measurement

**Identify bottlenecks scientifically:**

- [ ] CPU profiling with `perf` (flamegraphs)
- [ ] Memory profiling with Valgrind/Heaptrack
- [ ] I/O profiling with `iotop`/`iostat`
- [ ] Latency histograms generated (p50, p99, p99.9)
- [ ] Performance regression tests in CI
- [ ] Tick measurement for hot path (`rdtsc`)
- [ ] Continuous benchmarking (track over time)
- [ ] Production metrics monitored (Prometheus, Grafana)

**Pass Criteria**: All 8 items checked ✅

**Tools**:
```bash
# CPU profiling
perf record -g cargo bench
perf report

# Memory profiling
valgrind --tool=massif cargo test

# Tick measurement
make test-performance-v04
```

---

## 8. Benchmarking Best Practices

**Accurate performance measurement:**

- [ ] Warm-up iterations before measurement (exclude cold start)
- [ ] Multiple iterations for statistical significance (100+)
- [ ] Outlier removal (remove top/bottom 5%)
- [ ] Consistent test environment (isolated cores, no turbo boost)
- [ ] Benchmark harness uses `criterion` or similar
- [ ] Baseline comparison (before/after optimization)
- [ ] Real-world workloads tested (not synthetic)
- [ ] CI/CD includes benchmark suite

**Pass Criteria**: All 8 items checked ✅

**Example**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_hot_path_ask(c: &mut Criterion) {
    c.bench_function("hot_path_ask", |b| {
        b.iter(|| {
            // Warm-up excluded automatically by criterion
            let result = execute_ask(black_box(query));
            black_box(result)
        });
    });
}

criterion_group!(benches, benchmark_hot_path_ask);
criterion_main!(benches);
```

---

## 9. Optimization Techniques (Ordered by Impact)

**Apply in this order for maximum ROI:**

1. **Algorithmic improvements** (10-1000x improvement)
   - [ ] Replace O(n²) with O(n log n) algorithm
   - [ ] Use specialized data structures (Bloom filters, tries)

2. **Caching** (10-1000x for hits)
   - [ ] Add MPHF cache for hot path queries
   - [ ] LRU cache for warm path operations

3. **Branchless code** (2-10x in hot path)
   - [ ] Replace if/else with bitwise operations
   - [ ] Use CMOV instructions

4. **SIMD parallelism** (2-8x for data-parallel ops)
   - [ ] Use AVX-512 for 8-lane operations
   - [ ] Vectorize loops

5. **Memory layout** (2-5x for cache efficiency)
   - [ ] Convert AoS to SoA
   - [ ] Align to 64-byte cache lines

6. **I/O batching** (2-100x for I/O-bound)
   - [ ] Batch database queries
   - [ ] Batch network requests

7. **Async I/O** (2-10x for concurrent I/O)
   - [ ] Use Tokio for async operations
   - [ ] Non-blocking network calls

8. **Lock-free algorithms** (2-5x for contention)
   - [ ] Use atomic operations
   - [ ] Lock-free queues

**Pass Criteria**: Top 3 techniques applied ✅

---

## 10. Performance Regression Prevention

**Maintain performance over time:**

- [ ] Benchmark suite in CI/CD
- [ ] Performance budget enforced (fail CI if exceeded)
- [ ] Flamegraph comparison in PRs
- [ ] Performance alerts configured (Grafana)
- [ ] Continuous performance monitoring (production)
- [ ] Performance changelog maintained
- [ ] Optimization guide for contributors
- [ ] Code reviews include performance analysis

**Pass Criteria**: All 8 items checked ✅

---

## Quick Validation Commands

```bash
# Run performance test suite
make test-performance-v04

# Benchmark hot path
cd /home/user/knhk/rust/knhk-warm && cargo bench

# CPU profiling
perf record -g cargo bench
perf report --stdio | head -50

# Memory profiling
valgrind --tool=massif --massif-out-file=massif.out cargo test
ms_print massif.out

# Tick measurement
cargo test --test chicago_tdd_hot_path_complete -- --nocapture
```

---

## Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| ASK query | ≤1.5 ns | `make test-performance-v04` |
| COUNT query | ≤1.5 ns | `make test-performance-v04` |
| COMPARE | ≤1.5 ns | `make test-performance-v04` |
| VALIDATE | ≤2.0 ns | `make test-performance-v04` |
| CONSTRUCT8 | ≤500 µs | Warm path benchmarks |
| Cache lookup | ≤100 µs | MPHF benchmarks |
| Workflow step | ≤500 µs | Workflow engine tests |

---

## Final Sign-Off

- [ ] **All 10 sections completed** (80 total checks)
- [ ] **Hot path ≤8 ticks** (verified with `make test-performance-v04`)
- [ ] **Warm path ≤500µs** (verified with benchmarks)
- [ ] **Top 3 optimization techniques applied**
- [ ] **Performance regression tests in CI**
- [ ] **Production monitoring configured**

**Optimization Approved By**: ________________
**Date**: ________________
**Performance Improvement**: ______x faster

---

**See Also**:
- [Performance Guide](/home/user/knhk/docs/PERFORMANCE.md)
- [Production Readiness Checklist](/home/user/knhk/docs/reference/cards/PRODUCTION_READINESS_CHECKLIST.md)
- [Hot Path Implementation](/home/user/knhk/c/README.md)
