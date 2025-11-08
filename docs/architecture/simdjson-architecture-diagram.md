# simdjson Implementation: Architecture Diagrams

**Version:** 1.0.0
**Created:** 2025-11-08
**Architect:** System Architect (Hive Mind Agent)

---

## Diagram 1: Current vs Proposed Architecture

### Current KNHK Hot Path (8 ticks)

```
┌───────────────────────────────────────────────────────────────────┐
│                    CURRENT HOT PATH (8 ticks)                     │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 0-1: Load Triples from Δ-Ring                               │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ knhk_ring_dequeue_delta(ring, tick, S, P, O, capacity)      │  │
│  │ → Loads up to 8 triples sequentially                        │  │
│  │ → No SIMD optimization                                      │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 2-3: Sequential Predicate Matching                          │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ for (i = 0; i < count; i++) {                               │  │
│  │     if (P[i] == target_predicate) { match_found = true; }   │  │
│  │ }                                                            │  │
│  │ → 8 sequential comparisons                                  │  │
│  │ → Branch misprediction overhead                             │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 4: Guard Validation                                         │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ if (run_length > max_run_len) { return ERROR; }             │  │
│  │ → Simple branch check (fast)                                │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 5: Pattern Dispatch                                         │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ PATTERN_DISPATCH_TABLE[pattern_type](ctx, data, size)       │  │
│  │ → Branchless function pointer lookup (optimized)            │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 6: Execute Workflow Pattern                                 │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ knhk_pattern_sequence() / parallel_split() / etc.           │  │
│  │ → Pattern-specific logic                                    │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 7: Write Results to A-Ring                                  │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ knhk_ring_enqueue_assertion(ring, tick, S, P, O, receipt)   │  │
│  │ → Sequential write                                          │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘

TOTAL: 8 ticks
```

---

### Proposed Two-Stage Architecture (6 ticks)

```
┌───────────────────────────────────────────────────────────────────┐
│              STAGE 1: HOT PATH (2 ticks)                          │
│         Fast Structural Validation (SIMD-optimized)               │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 0-0.5: SIMD Predicate Matching                              │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ ARM64 NEON (4 predicates in parallel):                      │  │
│  │   uint64x2_t p_vec = vld1q_u64(predicates);                 │  │
│  │   uint64x2_t target_vec = vdupq_n_u64(target);              │  │
│  │   uint64x2_t cmp = vceqq_u64(p_vec, target_vec);            │  │
│  │                                                              │  │
│  │ → 8 comparisons in 2 SIMD ops (4x faster)                   │  │
│  │ → No branches, fully vectorized                             │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 0.5-1.0: Branchless Guard Validation                        │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ valid = (run_length <= max_run_len) ? 1 : 0;                │  │
│  │ → Branchless (compiler generates cmov)                      │  │
│  │ → SIMD-parallel guard checks for multiple runs              │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 1.0-1.5: Quick SoA Bounds Check                             │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ bounds_ok = (index < size) & (index >= 0);                  │  │
│  │ → Branchless arithmetic check                               │  │
│  │ → Cache-aligned bounds verification                         │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 1.5-2.0: Pattern Dispatch                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ PATTERN_DISPATCH_TABLE[pattern_type](ctx, data, size)       │  │
│  │ → Branchless function pointer lookup                        │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  OUTPUT: Structural Markers                                       │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ struct StructuralMarkers {                                  │  │
│  │     matched_predicates: Vec<usize>,  // Indices of matches  │  │
│  │     guard_valid: bool,                // Guard check result │  │
│  │     bounds_ok: bool,                  // Bounds check       │  │
│  │     pattern_id: u8,                   // Pattern to execute │  │
│  │ }                                                            │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘

                                 ║
                                 ║ Pass structural markers
                                 ║ to Stage 2
                                 ▼

┌───────────────────────────────────────────────────────────────────┐
│              STAGE 2: WARM PATH (4 ticks)                         │
│           Semantic Query Execution (Full SPARQL)                  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 0: Execute Workflow Pattern                                 │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ knhk_pattern_sequence() / parallel_split() / etc.           │  │
│  │ → Uses structural markers from Stage 1                      │  │
│  │ → No re-validation needed                                   │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 1: Complex Joins                                            │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ Graph pattern matching                                      │  │
│  │ SPARQL join operations                                      │  │
│  │ Variable binding                                            │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 2: Result Materialization                                   │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ CONSTRUCT / SELECT result building                          │  │
│  │ Triple materialization                                      │  │
│  │ Receipt generation                                          │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│  TICK 3: Write to A-Ring                                          │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ knhk_ring_enqueue_assertion(ring, tick, S, P, O, receipt)   │  │
│  │ → Batched write (reuses hot buffers)                        │  │
│  └─────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘

TOTAL: 2 + 4 = 6 ticks (25% improvement)
```

---

## Diagram 2: Memory Reuse Pattern

### Current (Allocate on Every Operation)

```
┌───────────────────────────────────────────────────────────────────┐
│                    Operation Flow (Current)                       │
└───────────────────────────────────────────────────────────────────┘

Operation 1:
  Allocate DeltaRing (heap allocation, COLD)  ────► Execute ────► Free
  Allocate Receipt (heap allocation, COLD)    ────► Execute ────► Free
                                                    2-3 ticks

Operation 2:
  Allocate DeltaRing (heap allocation, COLD)  ────► Execute ────► Free
  Allocate Receipt (heap allocation, COLD)    ────► Execute ────► Free
                                                    2-3 ticks

Operation 3:
  Allocate DeltaRing (heap allocation, COLD)  ────► Execute ────► Free
  Allocate Receipt (heap allocation, COLD)    ────► Execute ────► Free
                                                    2-3 ticks

Problem:
  - Every operation allocates from heap (COLD memory, slow)
  - Cache thrashing (new allocations not in L1/L2)
  - Allocation overhead: ~0.5-1 tick per operation
```

---

### Proposed (Global Buffer Pool)

```
┌───────────────────────────────────────────────────────────────────┐
│                  Global Ring Pool (Always HOT)                    │
├───────────────────────────────────────────────────────────────────┤
│                                                                   │
│  DeltaRing Pool (4 pre-allocated buffers):                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐             │
│  │  Ring 0 │  │  Ring 1 │  │  Ring 2 │  │  Ring 3 │             │
│  │  (HOT)  │  │  (HOT)  │  │  (WARM) │  │  (COLD) │             │
│  │  L1     │  │  L1     │  │  L2     │  │  L3     │             │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘             │
│                                                                   │
│  Receipt Pool (1024 pre-allocated receipts):                     │
│  [Receipt 0, Receipt 1, Receipt 2, ..., Receipt 1023]            │
│   ↑ HOT (L1 cache) ↑                                             │
│                                                                   │
│  AssertionRing Pool (4 pre-allocated buffers):                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐             │
│  │  Ring 0 │  │  Ring 1 │  │  Ring 2 │  │  Ring 3 │             │
│  │  (HOT)  │  │  (HOT)  │  │  (WARM) │  │  (COLD) │             │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘             │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘

Operation Flow:

Operation 1:
  Get Ring 0 (HOT, L1 cache, 0 allocations)  ────► Execute ────► Return to pool
  Get Receipt 0 (HOT, L1 cache)              ────► Execute ────► Return to pool
                                                   <1 tick

Operation 2:
  Get Ring 1 (HOT, L1 cache, 0 allocations)  ────► Execute ────► Return to pool
  Get Receipt 1 (HOT, L1 cache)              ────► Execute ────► Return to pool
                                                   <1 tick

Operation 3:
  Get Ring 2 (WARM, L2 cache, 0 allocations) ────► Execute ────► Return to pool
  Get Receipt 2 (HOT, L1 cache)              ────► Execute ────► Return to pool
                                                   <1 tick

Benefits:
  ✅ Zero allocations in hot path
  ✅ Buffers stay hot in L1/L2 cache
  ✅ Allocation overhead eliminated (~1 tick saved)
  ✅ Predictable memory usage (bounded to pool size)
```

---

## Diagram 3: Free Padding Optimization

### Current (Bounds Checking Required)

```
┌───────────────────────────────────────────────────────────────────┐
│                 SoA Array (Current, No Padding)                   │
└───────────────────────────────────────────────────────────────────┘

S: [u64; 8]  = [s0, s1, s2, s3, s4, s5, s6, s7]
                ↑                           ↑
                Start                       End
                                            ↓
                                  Can't safely read beyond!

SIMD Load (4 predicates at once):
  for i in 0..4..8:
    if i + 4 <= len:  ← BRANCH (slow!)
      vec = vld1q_u64(&P[i])
    else:
      scalar_load()   ← Fallback for last elements

Problem:
  - Branches required for bounds checking
  - Can't use SIMD for last elements
  - Slower than necessary
```

---

### Proposed (64-byte Padding)

```
┌───────────────────────────────────────────────────────────────────┐
│              SoA Array with Padding (Proposed)                    │
└───────────────────────────────────────────────────────────────────┘

S: [u64; 8+8] = [s0, s1, s2, s3, s4, s5, s6, s7, pad, pad, pad, pad, pad, pad, pad, pad]
                 ↑                           ↑   ↑─────────────────────────────────────↑
                 Start                       End  Padding (64 bytes, same page)

Page Boundary Check:
  ┌──────────────────────────────────────────┬────────────────┐
  │        Valid Data (64 bytes)             │ Padding (64B)  │
  └──────────────────────────────────────────┴────────────────┘
   ↑                                                           ↑
   Page Start                                     Still within 4KB page!

SIMD Load (No Bounds Checking):
  for i in 0..4..8:
    vec = vld1q_u64(&P[i])  ← ALWAYS SAFE (no branch!)

  Even when i=6:
    vec = vld1q_u64(&P[6])  → Loads P[6], P[7], pad[0], pad[1]
                              → Safe because padding is within page
                              → Use only P[6], P[7] (discard padding)

Benefits:
  ✅ Zero branches in SIMD code
  ✅ Can always load 4 elements at once
  ✅ No segfaults (padding within page boundary)
  ✅ Faster SIMD loops (~0.5 tick saved)
```

---

## Diagram 4: SIMD Predicate Matching

### Sequential (Current, 2 ticks)

```
┌───────────────────────────────────────────────────────────────────┐
│         Sequential Predicate Matching (Current)                   │
└───────────────────────────────────────────────────────────────────┘

Predicates: [P0, P1, P2, P3, P4, P5, P6, P7]
Target: 0xABCD

Sequential Loop:
  TICK 0:   if P0 == 0xABCD → false
  TICK 0.25: if P1 == 0xABCD → false
  TICK 0.5: if P2 == 0xABCD → false
  TICK 0.75: if P3 == 0xABCD → true  ← MATCH FOUND
  TICK 1:   if P4 == 0xABCD → false
  TICK 1.25: if P5 == 0xABCD → false
  TICK 1.5: if P6 == 0xABCD → false
  TICK 1.75: if P7 == 0xABCD → false

Total: 2 ticks (8 sequential comparisons)

Problem:
  - Sequential processing (1 comparison per cycle)
  - Branch misprediction overhead
  - No parallelism
```

---

### SIMD (Proposed, 0.5 ticks)

```
┌───────────────────────────────────────────────────────────────────┐
│       ARM64 NEON Vectorized Predicate Matching (Proposed)         │
└───────────────────────────────────────────────────────────────────┘

Predicates: [P0, P1, P2, P3, P4, P5, P6, P7]
Target: 0xABCD

SIMD Processing (4 predicates in parallel):

  TICK 0-0.25:
    Load Vec0: [P0, P1] (2 × u64 NEON register)
    Load Vec1: [P2, P3]
    Load Target: [0xABCD, 0xABCD]

    Compare Vec0 == Target → [false, false]
    Compare Vec1 == Target → [false, true]  ← MATCH FOUND in parallel!

  TICK 0.25-0.5:
    Load Vec2: [P4, P5]
    Load Vec3: [P6, P7]

    Compare Vec2 == Target → [false, false]
    Compare Vec3 == Target → [false, false]

Total: 0.5 ticks (8 comparisons in 2 SIMD ops)

ARM64 NEON Code:
  uint64x2_t p_vec0 = vld1q_u64(&predicates[0]);  // Load P0, P1
  uint64x2_t p_vec1 = vld1q_u64(&predicates[2]);  // Load P2, P3
  uint64x2_t target_vec = vdupq_n_u64(target);    // [target, target]

  uint64x2_t cmp0 = vceqq_u64(p_vec0, target_vec); // Compare in parallel
  uint64x2_t cmp1 = vceqq_u64(p_vec1, target_vec);

  uint64_t match = vgetq_lane_u64(cmp0, 0) |      // Extract results
                   vgetq_lane_u64(cmp0, 1) |
                   vgetq_lane_u64(cmp1, 0) |
                   vgetq_lane_u64(cmp1, 1);

Benefits:
  ✅ 4x speedup (0.5 ticks vs 2 ticks)
  ✅ No branches (fully branchless)
  ✅ Parallel comparison of 4 predicates
  ✅ Perfect for KNHK's ≤8 predicates constraint
```

---

## Diagram 5: Complete Data Flow

### Full Pipeline (Proposed Two-Stage Architecture)

```
┌───────────────────────────────────────────────────────────────────┐
│                     INPUT: Δ-RING (Delta Ring)                    │
│  Per-tick slots: [Tick 0 | Tick 1 | Tick 2 | ... | Tick 7]        │
│  Each slot: S[], P[], O[], cycle_ids[], flags[]                   │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 │ Dequeue triples
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│                STAGE 1: HOT PATH (≤2 ticks)                       │
│             Fast Structural Validation (SIMD)                     │
├───────────────────────────────────────────────────────────────────┤
│                                                                   │
│  1. SIMD Predicate Matching (0.5 ticks)                          │
│     ┌────────────────────────────────────────────────┐           │
│     │ uint64x2_t p_vec = vld1q_u64(predicates)       │           │
│     │ uint64x2_t cmp = vceqq_u64(p_vec, target)      │           │
│     │ → 4x speedup, fully branchless                 │           │
│     └────────────────────────────────────────────────┘           │
│                                                                   │
│  2. Branchless Guard Validation (0.5 ticks)                      │
│     ┌────────────────────────────────────────────────┐           │
│     │ valid = (run_length <= max_run_len) ? 1 : 0   │           │
│     │ → Compiler generates cmov (no branch)          │           │
│     └────────────────────────────────────────────────┘           │
│                                                                   │
│  3. Quick SoA Bounds Check (0.5 ticks)                           │
│     ┌────────────────────────────────────────────────┐           │
│     │ bounds_ok = (index < size) & (index >= 0)      │           │
│     │ → Branchless arithmetic                        │           │
│     └────────────────────────────────────────────────┘           │
│                                                                   │
│  4. Pattern Dispatch (0.5 ticks)                                 │
│     ┌────────────────────────────────────────────────┐           │
│     │ PATTERN_DISPATCH_TABLE[pattern_type](...)      │           │
│     │ → Branchless function pointer lookup           │           │
│     └────────────────────────────────────────────────┘           │
│                                                                   │
│  OUTPUT: Structural Markers                                      │
│  { matched_predicates, guard_valid, bounds_ok, pattern_id }      │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 │ Pass markers
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│                STAGE 2: WARM PATH (4 ticks)                       │
│           Semantic Query Execution (Full SPARQL)                  │
├───────────────────────────────────────────────────────────────────┤
│                                                                   │
│  1. Execute Workflow Pattern (1 tick)                            │
│     ┌────────────────────────────────────────────────┐           │
│     │ knhk_pattern_sequence() / parallel_split()     │           │
│     │ → Uses structural markers from Stage 1         │           │
│     └────────────────────────────────────────────────┘           │
│                                                                   │
│  2. Complex Joins (1 tick)                                       │
│     ┌────────────────────────────────────────────────┐           │
│     │ Graph pattern matching                         │           │
│     │ SPARQL join operations                         │           │
│     └────────────────────────────────────────────────┘           │
│                                                                   │
│  3. Result Materialization (1 tick)                              │
│     ┌────────────────────────────────────────────────┐           │
│     │ CONSTRUCT / SELECT result building             │           │
│     │ Receipt generation                             │           │
│     └────────────────────────────────────────────────┘           │
│                                                                   │
│  4. Write to A-Ring (1 tick)                                     │
│     ┌────────────────────────────────────────────────┐           │
│     │ knhk_ring_enqueue_assertion(ring, ...)         │           │
│     │ → Batched write (reuses hot buffers)           │           │
│     └────────────────────────────────────────────────┘           │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
                                 │
                                 │ Enqueue results
                                 ▼
┌───────────────────────────────────────────────────────────────────┐
│                  OUTPUT: A-RING (Assertion Ring)                  │
│  Per-tick slots: [Tick 0 | Tick 1 | Tick 2 | ... | Tick 7]        │
│  Each slot: S[], P[], O[], receipts[]                             │
└───────────────────────────────────────────────────────────────────┘

TOTAL: 2 ticks (Stage 1) + 4 ticks (Stage 2) = 6 ticks
       25% improvement from 8-tick baseline
```

---

## Performance Summary

| Optimization | Baseline | Optimized | Improvement |
|--------------|----------|-----------|-------------|
| Predicate Matching | 2 ticks | 0.5 ticks | **4x faster** |
| Allocation Overhead | 1 tick | 0 ticks | **100% eliminated** |
| Guard Validation | 1 tick | 0.5 ticks | **2x faster** |
| Total Hot Path | 8 ticks | 6 ticks | **25% faster** |

---

**Status:** ✅ Architecture Diagrams Complete
**Next:** Production Validator validates implementation approach
