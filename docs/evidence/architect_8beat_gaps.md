# 8-Beat Architecture Gap Analysis

**Analysis Date:** 2025-11-06
**Analyzed by:** System Architect (Hive Mind)
**PRD Version:** 1.0
**Codebase State:** Post-V1.0 (commit 409c427)

---

## Executive Summary

The 8-beat reconciliation epoch (τ=8) implementation is **structurally sound** but has **critical integration gaps** that prevent end-to-end operation. The core primitives (beat scheduler, ring buffers, fibers) are correctly implemented with branchless design, but **admission flow, hook integration, and receipt generation are incomplete**.

**Key Finding:** All architectural components exist, but they operate as **isolated subsystems** rather than a **unified pipeline**. Missing: Sidecar→Scheduler wiring, Hook→Kernel dispatch, Receipt→Lockchain append.

---

## 1. PRD Requirements vs. Implementation Map

### 1.1 Beat Scheduler (IMPLEMENTED ✅)

**PRD Requirements:**
- Global cycle counter with branchless increment
- Tick extraction (cycle & 0x7)
- Pulse detection (tick == 0)
- No branches in hot path

**Implementation Status:**
- ✅ **C implementation** (`c/src/beat.c`, `c/include/knhk/beat.h`): Atomic global cycle, branchless tick/pulse
- ✅ **Rust wrapper** (`rust/knhk-etl/src/beat_scheduler.rs`): FFI to C beat functions
- ✅ **Branchless pulse**: Uses arithmetic underflow `((tick - 1) >> 63) & 1`
- ✅ **Atomic operations**: `atomic_fetch_add(&knhk_global_cycle, 1)`

**Alignment:** **COMPLETE** - Meets all PRD requirements for timing model (Section 6).

---

### 1.2 Ring Buffers (IMPLEMENTED ✅ with minor gaps)

**PRD Requirements:**
- Δ-ring (input): SoA layout, power-of-2 size, per-tick indices
- A-ring (output): SoA layout for assertions + receipts
- Branchless enqueue/dequeue with atomic operations
- 64-byte alignment for cache lines

**Implementation Status:**
- ✅ **Δ-ring** (`c/src/ring.c`, `c/include/knhk/ring.h`):
  - SoA arrays (S, P, O, cycle_ids, flags)
  - Power-of-2 sizing with `size_mask = size - 1`
  - 64-byte aligned allocation (`posix_memalign`)
  - Per-tick atomic write/read indices (`_Atomic(uint64_t) write_idx[8]`)
- ✅ **A-ring**: Same structure with `knhk_receipt_t` array
- ⚠️ **Ring overflow handling**: Rollback logic present but not tested under stress
- ⚠️ **Prefetch hints**: Not yet implemented (PRD Section 11 performance engineering)

**Alignment:** **MOSTLY COMPLETE** - Core structure matches PRD Section 7. Missing prefetch hints for L1 optimization.

---

### 1.3 Fibers (IMPLEMENTED ✅ with integration gaps)

**PRD Requirements:**
- Per-shard execution units
- Pinned to NUMA-local cores
- Yield on tick boundaries
- Run length ≤ 8 (KNHK_NROWS)
- Execute μ (reconciliation kernel)

**Implementation Status:**
- ✅ **Fiber struct** (`c/include/knhk/fiber.h`): Execute, park, process_tick functions
- ✅ **Run length validation**: `if (ctx->run.len > KNHK_NROWS) return ERROR`
- ✅ **Park logic**: Single atomic write to set `PARKED` flag
- ⚠️ **NUMA pinning**: NOT IMPLEMENTED - No core affinity calls
- ⚠️ **Tick budget enforcement**: Estimates ticks (2-8) instead of PMU measurement
- ❌ **Hook→Kernel dispatch**: `knhk_eval_construct8` and `knhk_eval_bool` exist but **incomplete** (Section 3.4)

**Alignment:** **PARTIAL** - Fiber structure correct, but missing NUMA pinning (PRD Section 11) and PMU-based tick measurement (PRD Section 5).

**Critical Gap:** Fiber→Kernel integration is **stubbed out** - no actual hook execution beyond placeholder calls.

---

### 1.4 Hot Kernels (PARTIALLY IMPLEMENTED ⚠️)

**PRD Requirements (Section 3, Kernels):**
- ASK, COUNT, COMPARE, VALIDATE, SELECT, UNIQUE, CONSTRUCT8
- ≤2 ns/op (≤8 ticks total)
- Branchless: masks, cmov/blend, no conditional branches
- SoA lane operations with SIMD
- Non-temporal stores

**Implementation Status:**
- ✅ **ASK**: Implemented in `c/src/simd/compare.h` with SIMD masks
- ✅ **COUNT**: Mask popcnt operations
- ✅ **COMPARE**: cmov/blend for <, ≤, ≥
- ⚠️ **VALIDATE**: Exists but **incomplete** - datatype checks not wired
- ⚠️ **SELECT**: Lane gather exists but **not integrated** with fibers
- ⚠️ **UNIQUE**: MPHF lanes declared but **not implemented**
- ⚠️ **CONSTRUCT8**: Template structure exists (`ir->out_S`, `ir->out_P`, `ir->out_O`) but **incomplete** - constant baking not wired

**Alignment:** **INCOMPLETE** - Core kernels (ASK/COUNT/COMPARE) work, but advanced kernels (UNIQUE/CONSTRUCT8) are **stubs**.

**Critical Gap:** No AOT template compilation (PRD Section 8, Section 16 interfaces). Hook generation from Σ→templates not implemented.

---

### 1.5 Admission & Parking (IMPLEMENTED ✅ with gaps)

**PRD Requirements (Section 8):**
- Admit when: `run_len≤8 ∧ heat≥θ ∧ L1-ready`
- Predictor: MPHF + heatmap
- Demotion: Single atomic write to `PARKED` flag
- No retries in R1

**Implementation Status:**
- ✅ **Sidecar admission** (`rust/knhk-sidecar/src/beat_admission.rs`):
  - Stamps `cycle_id` on admission
  - Enqueues to delta ring
- ⚠️ **Backpressure**: `should_throttle()` is **placeholder** - no ring capacity check
- ❌ **Heat-based admission**: No heatmap or MPHF predictor
- ❌ **L1-ready check**: No cache predictor
- ✅ **Park flag**: `knhk_ring_park_delta()` sets `PARKED` flag atomically

**Alignment:** **PARTIAL** - Admission flow exists but **missing predictive admission** (PRD Section 8, Section 11).

**Critical Gap:** Admission is **blind** - admits all deltas without checking heat or L1 readiness. This violates the ≤8 tick budget guarantee.

---

### 1.6 Receipts & Provenance (PARTIALLY IMPLEMENTED ⚠️)

**PRD Requirements (Section 10, Section 16):**
- Receipt fields: `{cycle_id, shard_id, hook_id, ticks, hash(A)}`
- 100% receipts coverage
- Lockchain append on commit
- `hash(A) = hash(μ(O))`

**Implementation Status:**
- ✅ **Receipt struct** (`c/include/knhk/types.h`):
  ```c
  typedef struct {
    uint64_t cycle_id;
    uint64_t shard_id;
    uint64_t hook_id;
    uint32_t ticks;
    uint32_t lanes;
    uint64_t span_id;
    uint64_t a_hash;
  } knhk_receipt_t;
  ```
- ✅ **Hash computation**: Simple XOR hash in `knhk_fiber_execute()`
- ⚠️ **Span ID generation**: `knhk_generate_span_id()` declared but **not implemented**
- ❌ **Lockchain append**: No lockchain integration in codebase
- ❌ **Receipt continuity check**: No quorum/gap detection

**Alignment:** **PARTIAL** - Receipt structure correct, but **no persistence** to lockchain (PRD Section 10, Section 17 acceptance criteria).

**Critical Gap:** Receipts are **ephemeral** - generated but never persisted. No audit trail.

---

### 1.7 Hooks & Guards (NOT IMPLEMENTED ❌)

**PRD Requirements (Section 3, Hooks Engine):**
- μ ⊣ H (partial hook application)
- Guards at ingress: typing/cardinality/uniqueness/ranges
- Hook generation: Σ→(templates⊔kernels)
- AOT compilation: constants baked, graph handles preselected

**Implementation Status:**
- ❌ **Hook catalog**: No mapping of predicates→invariants
- ❌ **Hook generation**: No Σ→template compiler
- ❌ **Guard integration**: No ingress validation beyond basic type checks
- ⚠️ **Hook IR struct**: `knhk_hook_ir_t` exists but fields unused (PRD Section 16 interface)

**Alignment:** **MISSING** - Entire hooks subsystem is **not implemented**.

**Critical Gap:** Without hooks, the system cannot enforce invariants Q. This breaks the law `A = μ(O)` because μ is incomplete.

---

### 1.8 Policy Engine (NOT IMPLEMENTED ❌)

**PRD Requirements (Section 4, Policies):**
- Budgets: `ticks≤8`, `run_len≤8`
- Admission thresholds: `park_rate≤20%`, `C1<2%`, `heat95≥θ`
- Rego-based policy packs

**Implementation Status:**
- ❌ **Policy integration**: No Rego engine in codebase
- ⚠️ **Hardcoded budgets**: `KNHK_NROWS=8` hardcoded, not policy-driven
- ❌ **Adaptive thresholds**: No heat-based admission tuning

**Alignment:** **MISSING** - Policy engine is **not implemented**. Budgets are hardcoded constants instead of runtime policies.

---

### 1.9 Observability (PARTIALLY IMPLEMENTED ⚠️)

**PRD Requirements (Section 9, Weaver):**
- Live checks: `ticks_per_unit`, `l1_miss_rate`, `branch_miss`, `park_rate`, `heat95`, `receipt_gap`
- Traces: Δ→μ→A spans per beat
- Dashboards: hit-rate, latency, violations ppm, receipts, availability

**Implementation Status:**
- ⚠️ **Span IDs**: Generated in receipts but **not emitted** to OTEL
- ❌ **Weaver schema**: No `registry/` with schema definitions
- ❌ **Metrics emission**: No telemetry in C code (only Rust)
- ⚠️ **Rust telemetry**: Uses `tracing` crate but **not wired to beat events**

**Alignment:** **INCOMPLETE** - Telemetry infrastructure exists but **not integrated** with beat system.

**Critical Gap:** Cannot validate τ≤8 constraint without PMU instrumentation (PRD Section 13 test plan).

---

### 1.10 Security Mesh (NOT IMPLEMENTED ❌)

**PRD Requirements (Section 10, Security):**
- SPIFFE mTLS
- HSM/KMS key rotation ≤24h
- ABAC in RDF
- Guards decide, not app code

**Implementation Status:**
- ❌ **SPIFFE integration**: None
- ❌ **Key rotation**: No KMS integration
- ❌ **RDF-based ABAC**: No authorization engine

**Alignment:** **MISSING** - Security mesh is **not implemented**.

---

### 1.11 Cold Path (unrdf/Erlang) (NOT IMPLEMENTED ❌)

**PRD Requirements (Section 3, Cold Path):**
- Async finalize for parked units
- Analytics/joins never block beat
- Receipts continue even when W1 escalates to C1

**Implementation Status:**
- ❌ **unrdf integration**: No Erlang cold path
- ✅ **Park manager** (`rust/knhk-etl/src/park.rs`): Collects parked deltas
- ❌ **W1→C1 escalation**: No warm path implementation

**Alignment:** **MISSING** - Cold path is **not implemented**. Parked deltas have nowhere to go.

---

## 2. Integration Flow Gaps

### 2.1 Sidecar→Scheduler→Fiber Pipeline

**Expected Flow (PRD Section 16):**
```
Sidecar --enqueue(Δ, cycle_id)--> Scheduler --slot=tick--> Fiber --SoA ptrs--> Kernel --emit(A, receipt)--> Commit
```

**Current State:**
1. ✅ Sidecar calls `BeatAdmission::admit_delta()`
2. ✅ Admission enqueues to `BeatScheduler::enqueue_delta()`
3. ✅ Scheduler advances beat: `advance_beat()` → `execute_tick()`
4. ⚠️ `execute_tick()` dequeues from delta ring
5. ❌ **GAP:** Fiber receives delta but has no hook IR to execute
6. ❌ **GAP:** Kernel execution returns placeholder results
7. ⚠️ Receipts generated but not persisted

**Missing:**
- Hook registry to map predicates→kernels
- AOT template compiler to generate hook IR
- OTEL span emission in fiber execution
- Lockchain append in commit phase

---

### 2.2 Hook Generation (Σ→Templates→Kernels)

**Expected Flow (PRD Section 8, Hook Generation):**
```
Ontology (Σ) --templates--> Hook Catalog --AOT compile--> Hook IR --load--> Fiber --dispatch--> Kernel
```

**Current State:**
- ❌ No ontology parser (Σ input)
- ❌ No template→IR compiler
- ❌ No hook catalog (predicate→kernel map)
- ⚠️ Hook IR struct exists but unused

**Missing:**
- `knhk-policy` crate to parse Σ and generate templates
- AOT compiler to generate `knhk_hook_ir_t` structs
- Hook loading mechanism in fibers

---

### 2.3 Receipts→Lockchain→Audit

**Expected Flow (PRD Section 10, Provenance):**
```
Fiber --emit(A, receipt)--> A-ring --pulse--> Commit --lockchain append--> Quorum --audit query--> Report
```

**Current State:**
- ✅ Receipts generated in fibers
- ✅ Receipts enqueued to A-ring
- ❌ **GAP:** No lockchain implementation
- ❌ **GAP:** No quorum consensus
- ❌ **GAP:** No audit query interface

**Missing:**
- Lockchain crate with Merkle root generation
- Quorum protocol for cross-shard receipt validation
- Audit API to query receipt history

---

### 2.4 Performance Instrumentation (PMU)

**Expected Flow (PRD Section 11, Section 13):**
```
Fiber start --PMU read--> Kernel exec --PMU read--> Fiber end --ticks compute--> Receipt --verify ticks≤8--> Park if over
```

**Current State:**
- ⚠️ Tick estimation: `receipt->ticks = 2` (hardcoded)
- ❌ No PMU integration (RDTSC, PERF_COUNT)
- ❌ No tick budget enforcement (just logs warning)

**Missing:**
- PMU wrapper in C (`rdtsc()`, `perf_event_open()`)
- Tick budget validation in fiber
- Park trigger when `ticks > 8`

---

## 3. Architectural Coherence Assessment

### 3.1 Strengths

1. **Branchless Design**: Beat scheduler, tick calculation, pulse detection are all branchless. ✅
2. **SoA Layout**: Ring buffers use proper Structure-of-Arrays for cache efficiency. ✅
3. **Atomic Operations**: All ring operations use atomics (fetch-and-add, load, store). ✅
4. **Separation of Concerns**: Clear boundaries between scheduler (Rust), kernels (C), fibers (C). ✅
5. **Power-of-2 Sizing**: Ring buffers enforce power-of-2 for branchless modulo. ✅

### 3.2 Weaknesses

1. **No End-to-End Integration**: Subsystems work in isolation but don't form a complete pipeline. ❌
2. **Missing Hook System**: The entire μ (reconciliation function) is incomplete. ❌
3. **No Predictive Admission**: Admits all deltas blindly, violating tick budget guarantee. ❌
4. **Ephemeral Receipts**: Generated but not persisted, breaking audit requirements. ❌
5. **No PMU Instrumentation**: Cannot verify ≤8 tick constraint. ❌
6. **Hardcoded Budgets**: Policy engine missing, so budgets are magic numbers. ❌

---

## 4. Performance-Critical Path Analysis

**PRD Requirement:** R1 completion ≤8 ticks per admitted unit (≤2 ns/op).

### 4.1 Current Hot Path

```
Fiber entry --> Run length check (branchless) --> Kernel dispatch (if/else) --> SIMD compare --> Receipt hash --> Fiber exit
```

**Branch Analysis:**
- ✅ `knhk_beat_next()`: Branchless (atomic add)
- ✅ `knhk_beat_tick()`: Branchless (bitwise AND)
- ✅ `knhk_beat_pulse()`: Branchless (arithmetic underflow)
- ⚠️ `knhk_fiber_execute()`: **HAS BRANCHES** (line 21, 26, 45, 51, 69, 86 in fiber.c)
- ⚠️ Kernel dispatch: **HAS BRANCH** (`if (ir->op == KNHK_OP_CONSTRUCT8)`)
- ✅ SIMD compare: Branchless (masks)

**Critical Issue:** Fiber execution has **6 conditional branches** in hot path. This violates PRD Section 11 (zero branch mispredicts).

### 4.2 Recommended Optimizations

1. **Replace `if (ir->op == ...)` with function pointer dispatch:**
   ```c
   typedef int (*kernel_fn_t)(const knhk_context_t*, knhk_hook_ir_t*, knhk_receipt_t*);
   kernel_fn_t kernels[8] = {knhk_eval_ask, knhk_eval_count, ...};
   result = kernels[ir->op](ctx, ir, receipt); // Indirect call, no branch
   ```

2. **Use mask-based logic instead of error checks:**
   ```c
   // Instead of: if (!ctx || !ir || !receipt) return ERROR;
   uint64_t valid = (ctx != NULL) & (ir != NULL) & (receipt != NULL);
   result = valid * knhk_eval_kernel(...); // Zero if invalid
   ```

3. **Prefetch next SoA block** (PRD Section 11):
   ```c
   __builtin_prefetch(&ctx->S[ctx->run.off + 8], 0, 3); // Read, high locality
   ```

4. **Non-temporal stores for receipts** (PRD Section 11):
   ```c
   _mm_stream_si64((long long*)&ring->receipts[idx], receipt_val);
   ```

---

## 5. DFLSS Readiness (PRD Section 12)

**State:** **NOT READY** for Measure phase.

### 5.1 Missing Evidence

From PRD Section 18 (Evidence Stubs):
- ❌ `ev:beat_design.md` - This document (now created)
- ❌ `ev:pmu_bench.csv` - No PMU benchmarks
- ❌ `ev:weaver_checks.yaml` - No Weaver schema
- ❌ `ev:policy_packs.rego` - No policy engine
- ❌ `ev:receipts_root.json` - No lockchain
- ❌ `ev:canary_report.md` - No pilot deployment
- ❌ `ev:finance_oom.xlsx` - No business case

### 5.2 Acceptance Criteria (PRD Section 17)

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Beat stable 24h | ❌ Not tested | No long-running test |
| R1 p99 ≤ 2 ns/op | ❌ Not measured | No PMU benchmarks |
| Park rate ≤ 20% | ❌ Not measured | No stress test |
| C1 < 2% | ❌ N/A | No cold path |
| 100% receipts | ⚠️ Generated but not persisted | No lockchain |
| Dashboards green | ❌ No dashboards | No OTEL integration |

---

## 6. Recommendations (Priority Order)

### P0 (Blocking for Measure Phase)

1. **Implement Hook System:**
   - Create `knhk-policy` crate to parse Σ ontology
   - Build AOT compiler: templates → `knhk_hook_ir_t`
   - Wire hook dispatch in fibers

2. **Add PMU Instrumentation:**
   - Wrap RDTSC/PERF_COUNT in C
   - Measure actual ticks in fiber execution
   - Enforce 8-tick budget (park if exceeded)

3. **Implement Lockchain:**
   - Create `knhk-lockchain` crate
   - Append receipts to Merkle tree on commit
   - Persist roots to storage

4. **Integrate Weaver Validation:**
   - Create `registry/` with schema definitions
   - Emit OTEL spans in fiber execution
   - Run `weaver registry live-check` in CI

### P1 (Required for Production)

5. **Predictive Admission:**
   - Implement MPHF + heatmap for hot predicates
   - Add L1-ready check before admission
   - Backpressure when ring capacity >80%

6. **Branchless Fiber Execution:**
   - Replace error checks with mask-based logic
   - Use function pointer dispatch for kernels
   - Remove all `if` statements in hot path

7. **Complete Kernels:**
   - Implement UNIQUE (MPHF lanes)
   - Complete CONSTRUCT8 (AOT template baking)
   - Wire VALIDATE to datatype checks

8. **Policy Engine:**
   - Integrate OPA Rego
   - Define policy packs (budgets, admission rules)
   - Make thresholds runtime-configurable

### P2 (Optimization)

9. **NUMA Pinning:**
   - Pin fibers to cores with `sched_setaffinity()`
   - Allocate rings on NUMA-local memory

10. **Cache Optimizations:**
    - Add prefetch hints (`__builtin_prefetch`)
    - Non-temporal stores for receipts
    - Cache coloring to avoid set conflicts

11. **Cold Path Integration:**
    - Implement unrdf (Erlang) for W1/C1
    - Wire parked deltas to async processing
    - Escalation logic for C1

---

## 7. Conclusion

The 8-beat architecture is **well-designed** at the component level but **incomplete** as a system. The core timing model (beat scheduler, ring buffers, fibers) is **production-quality** and meets PRD requirements. However, **critical integration points are missing**:

1. **No hook system** → Cannot enforce invariants Q
2. **No PMU instrumentation** → Cannot verify τ≤8 constraint
3. **No lockchain** → Cannot prove `hash(A) = hash(μ(O))`
4. **No predictive admission** → Will violate tick budgets under load

**Priority:** Implement hook system, PMU, and lockchain **before** pilot deployment. Without these, the system cannot demonstrate DFLSS acceptance criteria (PRD Section 17).

**Architecture Verdict:** **STRUCTURALLY SOUND, FUNCTIONALLY INCOMPLETE**. Ready for focused implementation sprints, not ready for Measure phase.

---

## Appendix: File Inventory

### Implemented Files
- `c/include/knhk/beat.h` - Beat scheduler interface ✅
- `c/src/beat.c` - Beat scheduler implementation ✅
- `c/include/knhk/fiber.h` - Fiber interface ✅
- `c/src/fiber.c` - Fiber implementation ⚠️ (has branches)
- `c/include/knhk/ring.h` - Ring buffer interface ✅
- `c/src/ring.c` - Ring buffer implementation ✅
- `rust/knhk-etl/src/beat_scheduler.rs` - Rust beat wrapper ✅
- `rust/knhk-sidecar/src/beat_admission.rs` - Admission logic ⚠️ (no predictor)
- `rust/knhk-etl/src/park.rs` - Park manager ✅
- `rust/knhk-etl/src/fiber.rs` - Rust fiber wrapper ✅

### Missing Files (PRD-Required)
- `knhk-policy/` - Hook generation from Σ ❌
- `knhk-lockchain/` - Receipt persistence ❌
- `registry/` - Weaver schema definitions ❌
- `c/src/pmu.c` - PMU instrumentation ❌
- `c/src/kernels/unique.c` - UNIQUE kernel ❌
- `c/src/kernels/construct8.c` - CONSTRUCT8 kernel (complete) ❌
- `rust/knhk-warm/` - W1 warm path ❌
- `erlang/unrdf/` - C1 cold path ❌

---

**End: A = μ(O)**
