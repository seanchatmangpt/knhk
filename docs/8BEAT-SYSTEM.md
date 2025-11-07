# 8-Beat Reconciliation Epoch System

**Version**: 1.0  
**Status**: Active  
**Last Updated**: January 2025

## Overview

The 8-Beat Reconciliation Epoch system provides a fixed-cadence, branchless timing model for deterministic reconciliation operations. Every delta (Δ) is admitted on beat `k`, reconciled by μ within ≤8 ticks, emitted with receipt, or parked to W1.

**Key Principles**:
- **Determinism**: Global beat defines order Λ across pods and shards
- **Bounded time**: R1 completion ≤8 ticks per admitted unit
- **Branchless cadence**: Zero branch mispredicts on hot path
- **Isolation**: Over-budget work parks to W1 automatically
- **Provenance**: Every beat yields receipts with `hash(A)=hash(μ(O))`

## Architecture

### Components

1. **C Beat Scheduler** (`c/include/knhk/beat.h`, `c/src/beat.c`)
   - Branchless cycle/tick/pulse generation
   - Global atomic cycle counter
   - Tick extraction: `tick = cycle & 0x7` (0-7)
   - Pulse detection: `pulse = (tick == 0)` (branchless)

2. **C Fiber Execution** (`c/include/knhk/fiber.h`, `c/src/fiber.c`)
   - Per-shard, per-hook execution units
   - Executes μ on ≤8 items (run_len≤8)
   - Parks to W1 if ticks>8 or L1 miss predicted

3. **C Ring Buffers** (`c/include/knhk/ring.h`, `c/src/ring.c`)
   - Δ-ring (input): SoA layout for deltas
   - A-ring (output): SoA layout for assertions + receipts
   - Power-of-2 size for mod-8 indexing
   - Branchless enqueue/dequeue with atomic operations

4. **Rust Integration** (`rust/knhk-etl/src/beat_scheduler.rs`)
   - Uses C beat scheduler for cycle/tick/pulse
   - Manages delta rings and assertion rings
   - Coordinates fiber execution
   - Handles lockchain append on pulse boundary

5. **Rust Fiber** (`rust/knhk-etl/src/fiber.rs`)
   - Calls C fiber executor for hot path execution
   - Handles parking and receipt generation
   - Manages tick budget enforcement

## Functional Requirements

1. **Beat generation**: Global `cycle` increments; `tick = cycle & 0x7`
2. **Admission**: Sidecar stamps Δ with `cycle_id`; enqueue to Δ-ring[tick]
3. **Execution**: Fibers consume slot=tick, run μ on ≤8 items (run_len≤8)
4. **Park rule**: If kernel predicts L1 miss or ticks>8 → park Δ→W1 with receipt
5. **Emit**: Write A + receipt to out-ring[tick]; lockchain append
6. **Order Λ**: Commit happens at tick wrap (pulse)
7. **Drift control**: μ∘μ=μ holds across wraps; no cross-beat mutation
8. **Backpressure**: When Δ volume overflows, admission throttles by policy

## Performance Requirements

- **Latency (R1)**: ≤2ns/op (≤8 ticks per unit)
- **Hit-rate**: L1 ≥95% for hot predicates
- **Branch mispredicts**: 0 on hot path
- **Receipts coverage**: 100%
- **Availability**: ≥99.99% R1 service

## Implementation Status

### ✅ Completed

- **C Beat Scheduler**: Branchless cycle/tick/pulse generation
- **C Fiber Execution**: Hot path execution with tick budget enforcement
- **C Ring Buffers**: Lock-free ring buffers with SoA layout
- **Rust FFI Bindings**: Complete FFI bindings for C components
- **Rust Integration**: Beat scheduler and fiber integration
- **Chicago TDD Tests**: 22 tests covering beat scheduler, fiber, ring conversion

### Integration Points

**C → Rust FFI** (`rust/knhk-hot/src/`):
- `beat_ffi.rs`: Beat scheduler functions
- `ring_ffi.rs`: Ring buffer operations
- `fiber_ffi.rs`: Fiber execution

**Rust ETL Integration** (`rust/knhk-etl/src/`):
- `beat_scheduler.rs`: Uses C beat scheduler, manages rings
- `fiber.rs`: Calls C fiber executor, handles parking

## Usage

### Initialization

```rust
use knhk_etl::beat_scheduler::BeatScheduler;

// Create beat scheduler (4 shards, 2 domains, ring capacity 8)
let mut scheduler = BeatScheduler::new(4, 2, 8)?;

// Initialize C beat scheduler (call once at startup)
knhk_hot::BeatScheduler::init();
```

### Beat Execution

```rust
// Advance to next beat
let (tick, pulse) = scheduler.advance_beat();

// Execute fibers for current tick
// (handled internally by advance_beat())

// Commit on pulse boundary (every 8 ticks)
if pulse {
    // Lockchain append happens automatically
}
```

### Enqueue Delta

```rust
// Enqueue delta to delta ring
scheduler.enqueue_delta(
    domain_id,
    delta_triples,
    scheduler.current_cycle(),
)?;
```

## Related Documentation

- **[8BEAT-PRD.txt](8BEAT-PRD.txt)** - Complete Product Requirements Document
- **[BRANCHLESS_C_ENGINE_IMPLEMENTATION.md](BRANCHLESS_C_ENGINE_IMPLEMENTATION.md)** - Branchless C engine details
- **[INTEGRATION_SUMMARY.md](INTEGRATION_SUMMARY.md)** - C/Rust integration summary

## Testing

Chicago TDD tests cover:
- Beat scheduler creation and advancement
- Tick rotation and pulse detection
- Fiber execution within tick budget
- Ring conversion (SoA ↔ RawTriple)
- Receipt generation and merging

Run tests:
```bash
cargo test --package knhk-etl --test chicago_tdd_beat_scheduler
cargo test --package knhk-etl --test chicago_tdd_pipeline
```

---

## Enterprise Architecture: R1 Embedded in W1 and C1

### C4 Container Diagram (Target Architecture)

The following diagram shows the **target Reflex Enterprise architecture** where W1 (warm path) and C1 (cold path) can delegate deterministic subtasks to R1 (hot path) via μ_spawn().

```plantuml
@startuml
title Reflex Enterprise — C4 Container View: R1 Embedded in W1 and C1

!define RECTANGLE class

RECTANGLE EnterpriseSystem {
  RECTANGLE R1_HotPath as R1 {
    component "Beat Scheduler\n(8-tick loop)" as Beat
    component "Fiber Executor\n(μ execution ≤8 items)" as Fiber
    component "Ring Buffers\n(Δ-ring, A-ring)" as Rings
    component "Guard Layer H₁\n(schema, range, invariants)" as H1
  }

  RECTANGLE W1_WarmPath as W1 {
    component "Aggregator / Transform\n(non-critical ETL)" as ETL
    component "Guard Layer H₂\n(statistical, consistency)" as H2
    component "μ_spawn() calls\nfor deterministic subops" as MuSpawn
  }

  RECTANGLE C1_ColdPath as C1 {
    component "Analytics / ML / Training" as Analytics
    component "Guard Layer H₃\n(causal, provenance)" as H3
    component "Subtask Router\n(dynamic routing to R1)" as Router
  }

  RECTANGLE ControlPlane as CP {
    component "Beat Orchestrator\n(epoch & tick allocator)" as Orchestrator
    component "Provenance Validator\n(hash(A)=hash(μ(O)))" as Validator
    component "Metrics Engine\n(R1 utilization ratio R≥0.75)" as Metrics
  }
}

Beat --> Fiber : "dispatches μ subtasks"
Fiber --> Rings : "read/write Δ, A"
H1 --> Beat : "guards pre/post μ"

W1 --> MuSpawn : "delegates deterministic subtasks"
MuSpawn --> R1 : "invoke μ(O⊔Δ)"
ETL --> MuSpawn : "partition heavy ETL into 8-tick subtasks"
H2 --> MuSpawn : "enforce invariant Q"

C1 --> Router : "route subtask if deterministic"
Router --> R1 : "call μ via FFI"
H3 --> Validator : "verify provenance"

R1 --> CP : "emit receipts + performance data"
CP --> Metrics : "compute R1 utilization"
CP --> Validator : "validate drift(A)=0"

@enduml
```

### Architecture Gap Analysis (v1.0)

#### ✅ R1 Hot Path: **87.5% Complete**

| Component | Status | Location | v1.0 Notes |
|-----------|--------|----------|------------|
| Beat Scheduler | ✅ **IMPLEMENTED** | `c/src/beat.c`, `knhk-etl/src/beat_scheduler.rs` | Branchless 8-tick cycle |
| Fiber Executor | ✅ **IMPLEMENTED** | `c/src/fiber.c`, `knhk-etl/src/fiber.rs` | ≤8 item execution |
| Ring Buffers | ✅ **IMPLEMENTED** | `c/src/ring.c`, `knhk-hot/src/ring_ffi.rs` | DeltaRing + AssertionRing |
| Guard Layer H₁ | ⚠️ **PARTIAL** | `knhk-aot/src/lib.rs` | AOT guards exist, not integrated |

**Missing**: H₁ integration into beat loop for pre/post μ validation.

#### ⚠️ W1 Warm Path: **33.3% Complete**

| Component | Status | Location | v1.0 Notes |
|-----------|--------|----------|------------|
| Aggregator/Transform | ✅ **IMPLEMENTED** | `knhk-etl/src/transform.rs` | ETL pipeline |
| μ_spawn() API | ❌ **NOT FOUND** | N/A | **CRITICAL GAP** |
| Guard Layer H₂ | ❌ **NOT FOUND** | N/A | Statistical guards missing |

**Critical Gap**: W1 cannot delegate deterministic subtasks to R1. ETL runs entirely in warm path, missing performance optimization opportunity.

**Proposed μ_spawn() API** (v1.1):
```rust
/// Spawn deterministic subtask in R1 hot path from W1 warm path
pub fn mu_spawn(delta: &[RawTriple], k: usize) -> Result<Receipt, Error> {
    // 1. Validate delta ≤8 ticks via AOT guard (H₁)
    AotGuard::validate_ir(op, delta.len() as u64, k as u64)?;

    // 2. Convert to SoA format
    let soa = raw_triples_to_soa(delta)?;

    // 3. Invoke C fiber executor via FFI
    let receipt = unsafe {
        knhk_hot::fiber_execute(&soa)?
    };

    // 4. Return receipt with A = μ(O⊔Δ)
    Ok(receipt)
}
```

#### ❌ C1 Cold Path: **0% Complete**

| Component | Status | Location | v1.0 Notes |
|-----------|--------|----------|------------|
| Analytics/ML | ❌ **NOT FOUND** | N/A | Not implemented |
| Subtask Router | ❌ **NOT FOUND** | N/A | Not implemented |
| Guard Layer H₃ | ❌ **NOT FOUND** | N/A | Not implemented |

**Note**: C1 cold path is planned for v1.1+. Not required for v1.0 GO decision.

#### ⚠️ Control Plane: **50% Complete**

| Component | Status | Location | v1.0 Notes |
|-----------|--------|----------|------------|
| Beat Orchestrator | ✅ **IMPLEMENTED** | `knhk-etl/src/beat_scheduler.rs` | Epoch & tick allocation |
| Provenance Validator | ❌ **NOT FOUND** | N/A | hash(A)=hash(μ(O)) not validated |
| Metrics Engine | ⚠️ **PARTIAL** | `knhk-otel/src/lib.rs` | OTEL exists, R≥0.75 not enforced |

**Missing**:
1. **Provenance Validator**: No runtime validation that drift(A) = 0
2. **R1 Utilization SLO**: No enforcement of R ≥ 0.75 rule

---

### Implementation Roadmap (v1.1)

#### Phase 1: μ_spawn() API (Week 4) - **CRITICAL**
- **Priority**: P0 (enables W1→R1 delegation)
- **Effort**: 16 hours
- **Components**:
  1. FFI bridge from Rust W1 to C R1
  2. Tick budget validation via AOT guards (H₁)
  3. SoA conversion helpers
  4. Receipt generation and return
- **Success Criteria**: W1 ETL can delegate ≤8 tick subtasks to R1

#### Phase 2: Guard Layer Integration (Week 5)
- **Priority**: P1 (quality gates)
- **Effort**: 24 hours
- **Components**:
  1. H₁ integration: Pre/post μ validation in beat loop
  2. H₂ implementation: Statistical consistency guards in W1
  3. H₃ implementation: Provenance guards in control plane
- **Success Criteria**: All guard layers operational, violations tracked

#### Phase 3: Control Plane Completion (Week 6)
- **Priority**: P1 (observability)
- **Effort**: 16 hours
- **Components**:
  1. Provenance Validator: Validate hash(A)=hash(μ(O))
  2. Metrics Engine: Compute R1 utilization, enforce R≥0.75
  3. SLO Alerts: Notify on violations
- **Success Criteria**: Full observability, drift detection operational

#### Phase 4: C1 Cold Path (Week 7-8) - **FUTURE**
- **Priority**: P2 (post-v1.0)
- **Effort**: 40 hours
- **Components**:
  1. Subtask Router: Dynamic routing logic
  2. Analytics/ML integration points
  3. H₃ guard layer
- **Success Criteria**: C1→R1 delegation operational

---

### Performance Impact of μ_spawn()

**Without μ_spawn() (v1.0 Current)**:
- All ETL runs in W1 warm path (~50-100µs latency)
- R1 hot path underutilized (R < 0.50)
- No optimization for deterministic subtasks

**With μ_spawn() (v1.1 Target)**:
- Deterministic ETL ops delegated to R1 (≤2ns latency)
- R1 utilization increases (R ≥ 0.75)
- **Expected Performance Gain**: 25,000x for deterministic ops (50µs → 2ns)

**Example Use Case**:
```rust
// W1: Heavy ETL operation
fn transform_batch(deltas: &[RawTriple]) -> Result<Vec<Receipt>, Error> {
    // Partition into ≤8 tick chunks
    for chunk in deltas.chunks(8) {
        if is_deterministic(chunk) {
            // Delegate to R1 hot path (2ns)
            let receipt = mu_spawn(chunk, chunk.len())?;
            receipts.push(receipt);
        } else {
            // Process in W1 warm path (50µs)
            let result = transform_warm(chunk)?;
            receipts.push(result);
        }
    }
    Ok(receipts)
}
```

---

### v1.0 Architecture Compliance

| Layer | Target Components | v1.0 Implemented | v1.0 Missing | Compliance % |
|-------|------------------|------------------|--------------|--------------|
| **R1 (Hot)** | 4 | 3.5 | 0.5 | **87.5%** ✅ |
| **W1 (Warm)** | 3 | 1 | 2 | **33.3%** ⚠️ |
| **C1 (Cold)** | 3 | 0 | 3 | **0%** ❌ |
| **Control Plane** | 3 | 1.5 | 1.5 | **50%** ⚠️ |
| **OVERALL** | 13 | 6 | 7 | **46.2%** ⚠️ |

**v1.0 Conclusion**:
- R1 hot path is **production-ready** (87.5%)
- W1/C1 delegation missing → **architecture incomplete**
- μ_spawn() API is **critical path** to v1.1 GO

---

**Last Updated**: 2025-11-07 (v1.0 architecture gap analysis)
**Status**: Hive Queen validation complete, μ_spawn() API planned for v1.1


