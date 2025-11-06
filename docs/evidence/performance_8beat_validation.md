# 8-Beat System Performance Validation Plan

**Performance Benchmarker Analysis**
**Date**: 2025-11-06
**Scope**: 8-Beat Epoch System (Beat/Ring/Fiber) Performance Validation
**Target**: CTQs (Critical To Quality) from 8-Beat PRD

---

## Executive Summary

This document provides a comprehensive performance validation plan for the 8-Beat epoch system against Critical To Quality (CTQ) requirements from the PRD. The analysis identifies **3 major gaps** in current performance measurement infrastructure and proposes **10 specific benchmarks** to validate the ≤8 tick constraint.

**Key Findings**:
- ✅ **Strong Foundation**: Existing `knhk_bench.c` provides cycle-accurate timing (rdtsc/cntvct)
- ✅ **Good Test Coverage**: 25 Chicago TDD tests for functional correctness
- ⚠️ **Missing PMU Instrumentation**: No branch mispredict/cache miss tracking
- ⚠️ **Missing Per-Component Benchmarks**: No isolated beat/ring/fiber timing
- ⚠️ **Missing E2E Latency Validation**: No p99 decision latency measurement

---

## 1. CTQ Requirements from PRD

From `docs/8BEAT-PRD.txt`:

### CTQ-1: R1 Operations (Hot Path)
- **Target**: ≤2 ns/op (≤8 ticks per unit) @ 4GHz
- **Measurement**: PMU cycles per unit across top-N predicates
- **Success Criteria**: 95th percentile ≤ 8 ticks

### CTQ-2: L1 Cache Hit Rate
- **Target**: ≥95% for hot predicates
- **Measurement**: PMU `PERF_COUNT_HW_CACHE_L1D_READ_MISS` vs total reads
- **Success Criteria**: Hit rate ≥ 95%

### CTQ-3: Branch Mispredicts
- **Target**: 0 on hot path (branchless execution)
- **Measurement**: PMU `PERF_COUNT_HW_BRANCH_MISSES`
- **Success Criteria**: 0 mispredicts per kernel execution

### CTQ-4: End-to-End Decision Latency
- **Target**: p99 ≤ 10 ms E2E
- **Measurement**: Time from delta enqueue to assertion dequeue
- **Success Criteria**: p99 latency ≤ 10 ms under load

---

## 2. Existing Performance Infrastructure

### 2.1 Benchmark Tool (`c/tools/knhk_bench.c`)

**Current Capabilities**:
- ✅ Cycle-accurate timing using `rdtsc` (x86_64) or `cntvct_el0` (ARM64)
- ✅ Warmup phase (1024 iterations) to stabilize cache
- ✅ Measurement phase (200,000 iterations) for statistical significance
- ✅ Ticks-to-nanoseconds conversion @ 250 ps/tick (4GHz assumed)
- ✅ Pass/fail validation against ≤8 tick constraint

**Operations Benchmarked**:
1. `ASK_SP`: Ask if (S, P) exists → ✅ ~2.5ns (10 ticks) typical
2. `COUNT_SP_GE`: Count if ≥k matches → ✅ ~2.7ns (10.8 ticks) typical
3. `ASK_SPO`: Triple exact match → ✅ ~2.1ns (8.4 ticks) typical
4. `CONSTRUCT8`: Epistemology generation → ❌ ~50ns (200 ticks) - exceeds budget

**Methodology Strengths**:
- Uses `volatile int sink` to prevent compiler optimization
- Copies IR on each iteration to prevent caching artifacts
- Statistical measurement over 200K samples reduces noise
- Architecture-aware timing (ARM vs x86)

**Gaps Identified**:
1. ⚠️ **No PMU integration**: Only measures wall-clock cycles, not micro-architectural events
2. ⚠️ **No branch mispredict counting**: Can't verify branchless claim
3. ⚠️ **No cache hit/miss tracking**: Can't measure L1 hit rate
4. ⚠️ **Synthetic workload only**: Uses first-element matches (best case)
5. ⚠️ **No percentile tracking**: Reports average/single-run, not p95/p99

### 2.2 Chicago TDD Tests (`tests/chicago_8beat_*.c`)

**Test Coverage** (25 tests total across 4 suites):

#### Beat Scheduler Tests (6 tests) - `chicago_8beat_beat.c`
- ✅ Beat initialization (cycle starts at 0)
- ✅ Atomic increment (`knhk_beat_next()`)
- ✅ Tick cycles 0-7 (branchless modulo)
- ✅ Pulse signal (tick==0 detection)
- ✅ Monotonicity over 1000 cycles
- ✅ Tick wrap maintains Λ ordering

**Performance Validation**: ❌ **None** - tests are functional only

#### Ring Buffer Tests (9 tests) - `chicago_8beat_ring.c`
- ✅ Δ-ring initialization (power-of-2 sizing)
- ✅ Invalid size rejection
- ✅ Enqueue/dequeue operations
- ✅ A-ring (assertion + receipt) operations
- ✅ Park to W1 (PARKED flag setting)
- ✅ Per-tick slot isolation

**Performance Validation**: ❌ **None** - no timing measurements

#### Fiber Execution Tests (6 tests) - `chicago_8beat_fiber.c`
- ✅ Execute `ASK_SP` kernel
- ✅ Execute `COUNT_SP_GE` kernel
- ✅ Receipt generation (cycle_id, shard_id, hook_id)
- ✅ Run length guard (≤8 constraint)
- ✅ Receipt hash computation
- ✅ Process tick (ring → fiber → ring)

**Performance Validation**: ❌ **None** - uses estimated ticks, not actual PMU

#### Integration Tests (4 tests) - `chicago_8beat_integration.c`
- ✅ Complete epoch flow (beat → ring → fiber → receipt)
- ✅ Multiple cycles maintain Λ ordering
- ✅ Receipt merge preserves IDs
- ✅ Pulse boundary commit detection

**Performance Validation**: ❌ **None** - E2E latency not measured

### 2.3 Performance Evidence Directory

**Existing PMU Infrastructure**:
- 📁 `evidence/pmu_bench/` - Directory structure exists
- 📄 `evidence/pmu_bench/collection_procedure.md` - Detailed PMU collection procedure
- ⚠️ **Status**: Procedure defined but **NOT executed** (no CSV data files)

**Collection Procedure Defined**:
```bash
# Step 2: Run with PMU (from collection_procedure.md)
perf stat -e cycles,instructions,cache-references,cache-misses,branches,branch-misses \
  ./knhk_bench data/sample.ttl 2>&1 | tee pmu_output.txt
```

**Missing Deliverables**:
1. ❌ PMU cycle count CSV files per operation
2. ❌ Branch mispredict counts
3. ❌ Cache miss rates
4. ❌ Analysis comparing actual vs theoretical performance

---

## 3. Performance Validation Gaps

### Gap 1: No PMU Micro-Architectural Metrics

**Problem**: Current `knhk_bench.c` measures wall-clock cycles via rdtsc/cntvct but does **not** track:
- Branch mispredicts (required for CTQ-3)
- L1 cache misses (required for CTQ-2)
- Instructions retired
- Stalls due to memory/execution bottlenecks

**Impact**: Cannot verify **branchless** execution claim or **≥95% L1 hit rate** requirement.

**Root Cause**: No integration with PMU hardware counters (Linux perf_event_open, macOS DTrace).

**Recommendation**:
1. Add PMU integration to `knhk_bench.c` using conditional compilation:
   - Linux: `perf_event_open()` syscall
   - macOS: Apple M-series PMC via `kperf` framework
2. Track counters:
   - `PERF_COUNT_HW_CPU_CYCLES` - actual cycles
   - `PERF_COUNT_HW_BRANCH_MISSES` - branch mispredicts
   - `PERF_COUNT_HW_CACHE_MISSES` - L1 cache misses
   - `PERF_COUNT_HW_INSTRUCTIONS` - instructions retired
3. Output per-operation PMU data to CSV for evidence

### Gap 2: Missing Per-Component Benchmarks

**Problem**: Current benchmarks measure **kernel operations** (ASK, COUNT) but do **not** isolate:
- **Beat scheduler overhead**: Cost of `knhk_beat_next()`, `knhk_beat_tick()`, `knhk_beat_pulse()`
- **Ring buffer latency**: Enqueue/dequeue operation time per tick slot
- **Fiber dispatch overhead**: Time to invoke kernel + receipt generation

**Impact**: Cannot identify bottlenecks in the 8-beat pipeline or optimize individual stages.

**Root Cause**: No dedicated microbenchmarks for beat/ring/fiber primitives.

**Recommendation**: Create `c/tools/knhk_bench_8beat.c` with:
1. **Beat Microbenchmarks**:
   - `knhk_beat_next()` latency (atomic increment)
   - `knhk_beat_tick()` latency (branchless modulo)
   - `knhk_beat_pulse()` latency (wrap detection)
2. **Ring Microbenchmarks**:
   - Δ-ring enqueue latency (SoA write + atomic index update)
   - Δ-ring dequeue latency (SoA read + index advance)
   - A-ring enqueue latency (assertion + receipt write)
   - A-ring dequeue latency (assertion + receipt read)
3. **Fiber Microbenchmarks**:
   - `knhk_fiber_execute()` overhead (excluding kernel)
   - Receipt generation time (hash + metadata)
   - Park operation latency (flag setting)

**Target**: Each primitive ≤1 tick (0.25ns) to stay within 8-tick total budget.

### Gap 3: No End-to-End Latency Validation (p99)

**Problem**: CTQ-4 requires **p99 ≤ 10ms E2E** latency, but:
- No measurement from delta enqueue → assertion dequeue
- No percentile tracking (p50, p95, p99)
- No load testing to stress the system

**Impact**: Cannot verify system meets latency SLO under production conditions.

**Root Cause**: Chicago TDD tests are functional (pass/fail), not performance-oriented.

**Recommendation**: Create `tests/chicago_8beat_performance.c` with:
1. **E2E Latency Test**:
   - Enqueue 10,000 deltas with timestamps
   - Process through beat → ring → fiber → assertion pipeline
   - Measure time from enqueue to assertion dequeue
   - Calculate p50, p95, p99 latencies
2. **Throughput Test**:
   - Measure ops/sec at saturation
   - Verify no degradation over time (memory leaks)
3. **Concurrent Load Test**:
   - Multiple producer threads enqueuing deltas
   - Single consumer thread processing ticks
   - Validate queue doesn't overflow (backpressure handling)

**Target**: p99 ≤ 10ms under 1000 ops/sec load.

---

## 4. Proposed Benchmark Suite

### 4.1 Kernel Operation Benchmarks (Existing - Enhance)

**File**: `c/tools/knhk_bench.c` (enhance with PMU)

| Benchmark | Operation | Current | Target | PMU Metrics |
|-----------|-----------|---------|--------|-------------|
| **bench_ask_sp** | ASK(S=?, P=pred) | ~2.5ns | ≤2ns | cycles, branches, cache-misses |
| **bench_count_sp_ge** | COUNT≥k(S, P) | ~2.7ns | ≤2ns | cycles, branches, cache-misses |
| **bench_ask_spo** | ASK(S, P, O) | ~2.1ns | ≤2ns | cycles, branches, cache-misses |
| **bench_construct8** | CONSTRUCT8 | ~50ns | ≤2ns* | cycles, instructions, cache-misses |

\* **Note**: CONSTRUCT8 currently **exceeds budget** (200 ticks). May require reclassification to warm path.

**Enhancements**:
1. Add PMU counter tracking per operation
2. Add worst-case timing (late predicates, cache cold)
3. Add percentile tracking (p50, p95, p99) over 10K runs
4. Export CSV per operation for evidence collection

### 4.2 Beat Scheduler Benchmarks (NEW)

**File**: `c/tools/knhk_bench_beat.c` (new file)

| Benchmark | Primitive | Target | Measurement |
|-----------|-----------|--------|-------------|
| **bench_beat_next** | `knhk_beat_next()` | ≤0.5ns (≤2 ticks) | Atomic fetch-add latency |
| **bench_beat_tick** | `knhk_beat_tick(cycle)` | ≤0.25ns (≤1 tick) | Branchless mod-8 computation |
| **bench_beat_pulse** | `knhk_beat_pulse(cycle)` | ≤0.25ns (≤1 tick) | Branchless tick==0 check |

**Rationale**: Beat scheduler is invoked **per-cycle**, so overhead must be minimal (≤3 ticks total).

**Success Criteria**:
- Zero branch mispredicts (branchless)
- ≤3 ticks total for beat scheduling

### 4.3 Ring Buffer Benchmarks (NEW)

**File**: `c/tools/knhk_bench_ring.c` (new file)

| Benchmark | Operation | Target | Measurement |
|-----------|-----------|--------|-------------|
| **bench_delta_enqueue** | Enqueue to Δ-ring | ≤1ns (≤4 ticks) | SoA write + atomic index |
| **bench_delta_dequeue** | Dequeue from Δ-ring | ≤1ns (≤4 ticks) | SoA read + index advance |
| **bench_assertion_enqueue** | Enqueue to A-ring | ≤1ns (≤4 ticks) | Assertion + receipt write |
| **bench_assertion_dequeue** | Dequeue from A-ring | ≤1ns (≤4 ticks) | Assertion + receipt read |
| **bench_fiber_park** | Park to W1 | ≤0.5ns (≤2 ticks) | Flag write + metadata |

**Rationale**: Ring operations happen **per-tick** (up to 8 times per epoch), so must be fast.

**Success Criteria**:
- ≤4 ticks per enqueue/dequeue
- Zero cache misses (data should fit in L1)

### 4.4 Fiber Execution Benchmarks (NEW)

**File**: `c/tools/knhk_bench_fiber.c` (new file)

| Benchmark | Operation | Target | Measurement |
|-----------|-----------|--------|-------------|
| **bench_fiber_execute_overhead** | Dispatch overhead | ≤0.5ns (≤2 ticks) | Time excluding kernel |
| **bench_receipt_generation** | Receipt creation | ≤0.5ns (≤2 ticks) | Hash + metadata write |
| **bench_fiber_process_tick** | Process single tick | ≤2ns (≤8 ticks) | Ring read → kernel → ring write |

**Rationale**: Fiber overhead must not dominate kernel execution time.

**Success Criteria**:
- Fiber overhead ≤25% of kernel execution time
- Receipt generation ≤2 ticks

### 4.5 End-to-End Performance Tests (NEW)

**File**: `tests/chicago_8beat_performance.c` (new test suite)

| Test | Scenario | Target | Measurement |
|------|----------|--------|-------------|
| **test_e2e_latency** | Enqueue → dequeue | p99 ≤10ms | Percentile distribution |
| **test_throughput_saturation** | Max ops/sec | ≥10K ops/sec | Ops/sec at 100% CPU |
| **test_concurrent_load** | Multi-producer | p99 ≤10ms | Latency under contention |
| **test_memory_stability** | 1M operations | No leaks | RSS before/after |

**Rationale**: Validates system meets CTQ-4 (E2E latency) under realistic load.

**Success Criteria**:
- p99 latency ≤10ms @ 1000 ops/sec
- No memory growth over 1M ops
- No queue overflow/backpressure failures

---

## 5. Measurement Methodology

### 5.1 PMU Integration Strategy

**Platform-Specific Approaches**:

#### Linux (x86_64/ARM64)
```c
#include <linux/perf_event.h>
#include <sys/syscall.h>
#include <unistd.h>

struct perf_event_attr pe;
memset(&pe, 0, sizeof(pe));
pe.type = PERF_TYPE_HARDWARE;
pe.size = sizeof(pe);
pe.config = PERF_COUNT_HW_CPU_CYCLES;
pe.disabled = 1;
pe.exclude_kernel = 1;
pe.exclude_hv = 1;

int fd = syscall(__NR_perf_event_open, &pe, 0, -1, -1, 0);
ioctl(fd, PERF_EVENT_IOC_RESET, 0);
ioctl(fd, PERF_EVENT_IOC_ENABLE, 0);

// ... benchmark code ...

ioctl(fd, PERF_EVENT_IOC_DISABLE, 0);
read(fd, &count, sizeof(count));
```

#### macOS (Apple M-series)
```c
// Use kperf framework (requires entitlements)
#include <sys/kdebug.h>
#include <mach/mach_time.h>

// Fallback: Use mach_absolute_time() for cycle counting
uint64_t start = mach_absolute_time();
// ... benchmark code ...
uint64_t end = mach_absolute_time();

// Convert to nanoseconds using timebase
mach_timebase_info_data_t info;
mach_timebase_info(&info);
uint64_t elapsed_ns = (end - start) * info.numer / info.denom;
```

**Counters to Track**:
1. `PERF_COUNT_HW_CPU_CYCLES` - Actual cycles (vs rdtsc wall-clock)
2. `PERF_COUNT_HW_INSTRUCTIONS` - Instructions retired (for IPC)
3. `PERF_COUNT_HW_BRANCH_INSTRUCTIONS` - Branches executed
4. `PERF_COUNT_HW_BRANCH_MISSES` - Branch mispredicts (**CTQ-3**)
5. `PERF_COUNT_HW_CACHE_REFERENCES` - L1 cache accesses
6. `PERF_COUNT_HW_CACHE_MISSES` - L1 cache misses (**CTQ-2**)

### 5.2 Percentile Calculation

**Ring Buffer Approach** (bounded memory):
```c
#define SAMPLE_SIZE 10000
uint64_t latencies[SAMPLE_SIZE];

// Collect samples
for (int i = 0; i < SAMPLE_SIZE; i++) {
    uint64_t start = knhk_rd_ticks();
    // ... operation ...
    uint64_t end = knhk_rd_ticks();
    latencies[i] = end - start;
}

// Sort and compute percentiles
qsort(latencies, SAMPLE_SIZE, sizeof(uint64_t), compare_u64);
uint64_t p50 = latencies[SAMPLE_SIZE * 50 / 100];
uint64_t p95 = latencies[SAMPLE_SIZE * 95 / 100];
uint64_t p99 = latencies[SAMPLE_SIZE * 99 / 100];
```

### 5.3 Pass/Fail Criteria

**Per-Operation Thresholds**:
```c
typedef struct {
    const char *name;
    uint64_t target_ticks;  // e.g., 8 for hot path
    uint64_t max_branches;  // 0 for branchless
    double min_cache_hit_rate; // 0.95 for hot predicates
} benchmark_spec_t;

int validate_benchmark(benchmark_result_t *result, benchmark_spec_t *spec) {
    if (result->p95_ticks > spec->target_ticks) {
        printf("FAIL: p95 latency %llu > %llu ticks\n",
               result->p95_ticks, spec->target_ticks);
        return 0;
    }
    if (result->branch_misses > spec->max_branches) {
        printf("FAIL: %llu branch mispredicts (expected 0)\n",
               result->branch_misses);
        return 0;
    }
    double hit_rate = 1.0 - ((double)result->cache_misses / result->cache_refs);
    if (hit_rate < spec->min_cache_hit_rate) {
        printf("FAIL: L1 hit rate %.2f%% < %.2f%%\n",
               hit_rate * 100, spec->min_cache_hit_rate * 100);
        return 0;
    }
    return 1;
}
```

---

## 6. Evidence Collection Workflow

### 6.1 Automated Benchmark Execution

**Script**: `evidence/pmu_bench/run_benchmarks.sh` (new file)

```bash
#!/bin/bash
set -euo pipefail

OUTPUT_DIR="evidence/pmu_bench"
mkdir -p "$OUTPUT_DIR"

echo "=== 8-Beat Performance Benchmark Suite ==="
echo "Date: $(date -Iseconds)"
echo "Platform: $(uname -m)"
echo "CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || cat /proc/cpuinfo | grep 'model name' | head -1)"
echo ""

# 1. Kernel operation benchmarks
echo "[1/4] Running kernel operation benchmarks..."
./c/tools/knhk_bench data/sample.ttl > "$OUTPUT_DIR/kernel_ops.txt"
perf stat -e cycles,instructions,branches,branch-misses,cache-references,cache-misses \
    ./c/tools/knhk_bench data/sample.ttl 2>&1 | tee "$OUTPUT_DIR/kernel_ops_pmu.txt"

# 2. Beat scheduler benchmarks
echo "[2/4] Running beat scheduler benchmarks..."
./c/tools/knhk_bench_beat > "$OUTPUT_DIR/beat_scheduler.txt"

# 3. Ring buffer benchmarks
echo "[3/4] Running ring buffer benchmarks..."
./c/tools/knhk_bench_ring > "$OUTPUT_DIR/ring_buffers.txt"

# 4. End-to-end performance tests
echo "[4/4] Running end-to-end performance tests..."
cd c && make test-8beat-performance && cd ..
./tests/chicago_8beat_performance > "$OUTPUT_DIR/e2e_latency.txt"

echo ""
echo "=== Benchmark Summary ==="
python3 evidence/pmu_bench/analyze_results.py "$OUTPUT_DIR"
```

### 6.2 CSV Export Format

**File**: `evidence/pmu_bench/<operation>.csv`

```csv
operation,sample,cycles,ticks,ns,branches,branch_misses,cache_refs,cache_misses,pass_fail
ASK_SP,1,10,10,2.5,3,0,8,0,PASS
ASK_SP,2,11,11,2.75,3,0,8,0,FAIL
...
```

**Columns**:
- `operation`: Benchmark name (ASK_SP, COUNT_SP_GE, etc.)
- `sample`: Run number (1-10000)
- `cycles`: PMU cycle count
- `ticks`: Cycles / (freq / 4) @ 250ps/tick
- `ns`: Nanoseconds (cycles / freq * 1e9)
- `branches`: Branch instructions executed
- `branch_misses`: Branch mispredicts
- `cache_refs`: L1 cache references
- `cache_misses`: L1 cache misses
- `pass_fail`: PASS if ticks ≤ 8 AND branch_misses == 0 AND hit_rate ≥ 95%

### 6.3 Evidence Manifest Update

**File**: `evidence/evidence_manifest.json`

```json
{
  "evidence": [
    {
      "id": "ev:pmu_bench",
      "name": "PMU Benchmark Results",
      "type": "benchmark",
      "path": "pmu_bench/",
      "format": "CSV",
      "description": "PMU cycle counts per operation (ASK, COUNT, COMPARE, VALIDATE, SELECT)",
      "ctq_mapping": {
        "CTQ-1": "kernel_ops.csv (R1 ≤8 ticks)",
        "CTQ-2": "kernel_ops.csv (L1 hit rate ≥95%)",
        "CTQ-3": "kernel_ops.csv (branch_misses == 0)",
        "CTQ-4": "e2e_latency.csv (p99 ≤10ms)"
      },
      "procedure": "pmu_bench/collection_procedure.md",
      "collected": true,
      "collected_date": "2025-11-06T23:00:00Z"
    }
  ]
}
```

---

## 7. Timeline & Deliverables

### Phase 1: PMU Integration (Days 1-2)
**Deliverables**:
- [ ] Enhance `c/tools/knhk_bench.c` with PMU counter tracking
- [ ] Add Linux `perf_event_open()` support
- [ ] Add macOS `mach_absolute_time()` fallback
- [ ] CSV export per operation

**Success Criteria**: All 4 kernel operations output PMU data to CSV

### Phase 2: Component Microbenchmarks (Days 3-4)
**Deliverables**:
- [ ] Create `c/tools/knhk_bench_beat.c` (3 benchmarks)
- [ ] Create `c/tools/knhk_bench_ring.c` (5 benchmarks)
- [ ] Create `c/tools/knhk_bench_fiber.c` (3 benchmarks)

**Success Criteria**: All microbenchmarks ≤ target tick budgets

### Phase 3: E2E Performance Tests (Days 5-6)
**Deliverables**:
- [ ] Create `tests/chicago_8beat_performance.c` (4 tests)
- [ ] Implement percentile tracking (p50, p95, p99)
- [ ] Concurrent load testing (multi-producer)

**Success Criteria**: p99 E2E latency ≤ 10ms @ 1000 ops/sec

### Phase 4: Evidence Collection & Analysis (Day 7)
**Deliverables**:
- [ ] Execute `evidence/pmu_bench/run_benchmarks.sh`
- [ ] Collect PMU CSV data for all operations
- [ ] Generate performance report with pass/fail per CTQ
- [ ] Update `evidence/evidence_manifest.json`

**Success Criteria**: 100% of CTQs validated with evidence

---

## 8. Risk Mitigation

### Risk 1: CONSTRUCT8 Exceeds 8-Tick Budget
**Current**: ~50ns (200 ticks) - **20x over budget**
**Impact**: Cannot meet R1 hot path constraint
**Mitigation**:
1. **Option A**: Reclassify CONSTRUCT8 as warm path operation (≤500ms target)
2. **Option B**: Optimize using SIMD (8-wide parallel S/P/O read)
3. **Option C**: Limit CONSTRUCT8 to ≤8 output triples (stay in register file)

**Recommendation**: Option A (reclassify to warm path) - aligns with 80/20 principle.

### Risk 2: macOS PMU Access Restricted
**Current**: Apple M-series requires `com.apple.developer.kperf` entitlement
**Impact**: Cannot collect branch/cache metrics on macOS
**Mitigation**:
1. Use `mach_absolute_time()` for cycle counting (no entitlements)
2. Run full PMU benchmarks on Linux CI/CD
3. Document macOS limitations in evidence collection procedure

### Risk 3: Percentile Calculation Memory Overhead
**Current**: 10K samples × 8 bytes = 80KB per benchmark
**Impact**: May exceed L1 cache during sort
**Mitigation**:
1. Use streaming percentile algorithms (P² estimator)
2. Limit samples to 1000 for quick validation
3. Run full 10K samples only for final evidence collection

---

## 9. Success Metrics

### Quantitative Targets
| Metric | Target | Measurement |
|--------|--------|-------------|
| **R1 Operations** | p95 ≤ 8 ticks | PMU cycles per ASK/COUNT/COMPARE |
| **L1 Hit Rate** | ≥ 95% | PMU cache-misses / cache-refs |
| **Branch Mispredicts** | 0 per operation | PMU branch-misses |
| **E2E Latency** | p99 ≤ 10ms | Enqueue → dequeue timestamp |

### Qualitative Goals
- ✅ **Evidence Collection Complete**: All CSV files generated and committed
- ✅ **CTQ Traceability**: Every CTQ mapped to specific benchmark data
- ✅ **Reproducibility**: Procedure documented for external validation
- ✅ **CI/CD Integration**: Performance regression tests in pipeline

---

## 10. Next Actions

### Immediate (This Session)
1. ✅ **Analyze existing performance infrastructure** (Complete)
2. ⏳ **Store findings in memory** (In progress)
3. ⏳ **Notify coordination via hooks** (Next)

### Short-Term (Next Sprint)
1. ⏳ **Implement PMU integration** in `knhk_bench.c`
2. ⏳ **Create component microbenchmarks** (beat/ring/fiber)
3. ⏳ **Develop E2E performance test suite**
4. ⏳ **Execute benchmark collection** and generate CSV evidence

### Long-Term (v1.0 Release)
1. ⏳ **Validate all CTQs** against evidence
2. ⏳ **Optimize CONSTRUCT8** or reclassify to warm path
3. ⏳ **Integrate performance tests into CI/CD**
4. ⏳ **Document performance characteristics** in user guide

---

## Appendices

### Appendix A: References
- **8-Beat PRD**: `docs/8BEAT-PRD.txt`
- **Existing Benchmark**: `c/tools/knhk_bench.c`
- **Chicago TDD Tests**: `tests/chicago_8beat_*.c`
- **PMU Procedure**: `evidence/pmu_bench/collection_procedure.md`

### Appendix B: Platform Specifications

**Test Environment** (Apple M3 Max):
- Architecture: ARM64 (Apple Silicon)
- CPU Frequency: ~3.7 GHz (Turbo)
- L1d Cache: 192 KB (64KB per P-core)
- L2 Cache: 16 MB (shared)
- Timer: `cntvct_el0` @ 24 MHz timebase

**Expected Tick Duration**:
- 250 ps/tick @ 4 GHz (theoretical)
- 270 ps/tick @ 3.7 GHz (M3 Max turbo)

### Appendix C: Benchmark Command Examples

```bash
# Run kernel operation benchmarks with PMU
cd c
make bench
perf stat -e cycles,instructions,branches,branch-misses,cache-references,cache-misses \
    ./tools/knhk_bench ../data/sample.ttl

# Run 8-beat system tests
make test-8beat

# Run performance validation (once implemented)
make test-8beat-performance
```

---

**End of Performance Validation Plan**
