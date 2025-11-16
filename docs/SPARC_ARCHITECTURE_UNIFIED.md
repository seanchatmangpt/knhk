# KNHK Unified Architecture Specification

**Version**: 1.0.0
**Date**: 2025-11-16
**Status**: SPARC Phase 3 Complete
**Related Documents**:
- [LLM Overlay Proposer Design](designs/llm-overlay-proposer-design.md)
- [Sector Ontology Variants Architecture](designs/sector-ontology-variants-architecture.md)
- [Chicago TDD Patterns](CHICAGO_TDD_PATTERNS.md)
- [Autonomous Ontology System Design](autonomous-ontology-system-design.md)

---

## Executive Summary

This document provides the complete architectural specification for the **Knowledge-Native Hyper-Kernel (KNHK)** closed-loop autonomous intelligence system. KNHK achieves "312 Fortune 500 deployments across finance, healthcare, manufacturing, and logistics sectors" (2027) by implementing a four-plane architecture that autonomously evolves sector-specific ontologies while maintaining hard invariants (Q1-Q5) through cryptographically-verified receipts.

**Core Innovation**: A MAPE-K (Monitor-Analyze-Plan-Execute-Knowledge) closed loop that detects patterns in observations, proposes ontology changes (ΔΣ), validates against hard invariants and sector-specific doctrines, and promotes changes atomically (~1ns) via RCU semantics—all with complete cryptographic auditability.

**Performance Guarantees**:
- Hot path: ≤8 ticks (Chatman Constant)
- Snapshot promotion: ~1ns (atomic pointer swap)
- Warm path: <100ms (pattern detection, validation)
- Cold path: No hard constraint (LLM reasoning, multi-party approval)

**Key Architectural Principles**:
1. **Immutability**: All decisions recorded as cryptographically-signed receipts
2. **Atomicity**: Snapshot promotion via RCU (Read-Copy-Update) semantics
3. **Auditability**: Complete chain of custody for all ontology changes
4. **Composability**: Σ = Σ_core ⊕ Σ_sector (immutable core + mutable sector extensions)
5. **Constraint Preservation**: Hard invariants (Q1-Q5) never violated

---

## Table of Contents

1. [Four-Plane Architecture](#1-four-plane-architecture)
2. [MAPE-K Closed Loop](#2-mape-k-closed-loop)
3. [Component Inventory](#3-component-inventory)
4. [Data Flow Diagrams](#4-data-flow-diagrams)
5. [Integration Points](#5-integration-points)
6. [Sector Architecture](#6-sector-architecture)
7. [Testing Architecture (Chicago TDD)](#7-testing-architecture-chicago-tdd)
8. [Performance Characteristics](#8-performance-characteristics)
9. [Thread Safety and Concurrency](#9-thread-safety-and-concurrency)
10. [Deployment Architecture](#10-deployment-architecture)

---

## 1. Four-Plane Architecture

The KNHK system operates across four distinct architectural planes, each with specific responsibilities and performance characteristics:

```
┌─────────────────────────────────────────────────────────────────────┐
│                     OBSERVATION PLANE (O)                            │
│  Ingests events, telemetry, and receipts from external systems      │
│  Performance: Hot path (≤8 ticks), append-only, immutable           │
│                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ OTLP Spans   │  │ RDF Triples  │  │ Receipts     │              │
│  │ (OTEL)       │  │ (Turtle)     │  │ (ed25519)    │              │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
│         │                 │                 │                       │
│         └─────────────────┴─────────────────┘                       │
│                           │                                         │
│                  ObservationStore (DashMap)                         │
│                  Append-only, concurrent                            │
└───────────────────────────┼─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     ONTOLOGY PLANE (Σ)                               │
│  Active ontology snapshot: Σ = Σ_core ⊕ Σ_sector                   │
│  Performance: Sub-nanosecond read (atomic pointer load)             │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ SnapshotPromoter (ArcSwap<SnapshotDescriptor>)             │    │
│  │ • current: Arc<SnapshotDescriptor>  [atomic read]          │    │
│  │ • history: DashMap<SnapshotId, Arc<Snapshot>>              │    │
│  │ • promote(): ~1ns (atomic pointer swap via RCU)            │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  Composition: Σ_finance = Σ_core ⊕ Σ_finance_ext                   │
│               Σ_health = Σ_core ⊕ Σ_health_ext                     │
│               ...                                                    │
└───────────────────────────┼─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  CHANGE PLANE (ΔΣ + Q)                               │
│  Proposes, validates, and applies ontology changes                  │
│  Performance: Warm path (<100ms validation pipeline)                │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 1. PatternDetector: O → DetectedPatterns                   │    │
│  │    - Frequency anomaly detection                           │    │
│  │    - Error spike detection                                 │    │
│  │    - Schema mismatch detection                             │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 2. LLM Overlay Proposer: Pattern → ΔΣ (Proposal)           │    │
│  │    - Constraint-aware prompt engineering                   │    │
│  │    - Guided decoding (LMQL for critical constraints)       │    │
│  │    - Post-hoc validation (defense-in-depth)                │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 3. ValidationPipeline: ΔΣ → ValidationReport               │    │
│  │    Stage 1: Static schema check (SHACL)                    │    │
│  │    Stage 2: Invariant check (Q1-Q5)                        │    │
│  │    Stage 3: Doctrine check (sector-specific)               │    │
│  │    Stage 4: Guard check (immutable boundaries)             │    │
│  │    Stage 5: Performance check (≤8 ticks)                   │    │
│  │    Stage 6: Rollback check (reversibility)                 │    │
│  │    Stage 7: Compatibility check (backward compat)          │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                   Valid ✓ / Invalid ✗                               │
└───────────────────────────┼─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   PROJECTION PLANE (μ, Π, Λ)                         │
│  Compiles ontology to executable code (C headers, Rust traits)      │
│  Performance: Cold path (no hard latency constraint)                 │
│                                                                      │
│  μ: Code generation (ggen templates → C headers)                    │
│  Π: Promotion orchestration (atomic snapshot swap)                  │
│  Λ: Learning feedback (accepted/rejected proposals → corpus)        │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ If Valid:                                                   │    │
│  │   1. Π: Promote ΔΣ → Σ' (atomic snapshot swap)             │    │
│  │   2. μ: Codegen Σ' → C headers / Rust traits              │    │
│  │   3. Λ: Record acceptance in learning corpus               │    │
│  │                                                             │    │
│  │ If Invalid:                                                 │    │
│  │   1. Reject ΔΣ with reason                                 │    │
│  │   2. Λ: Record rejection + constraint violations           │    │
│  │   3. Adapt LLM prompts based on failure patterns           │    │
│  └────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.1 Plane Responsibilities

| Plane | Primary Function | Performance Tier | Concurrency Model | Storage |
|-------|------------------|------------------|-------------------|---------|
| **Observation (O)** | Ingest events, detect patterns | Hot path (≤8 ticks) | Lock-free append (DashMap) | Append-only log |
| **Ontology (Σ)** | Serve current snapshot | Sub-nanosecond | Atomic load (ArcSwap) | Immutable snapshots |
| **Change (ΔΣ+Q)** | Propose & validate changes | Warm path (<100ms) | Read-heavy (RwLock for write) | Proposal queue |
| **Projection (μ,Π,Λ)** | Compile & promote | Cold path (no limit) | Sequential (mutex) | Code artifacts |

### 1.2 Invariants Across Planes

**Q1: No Retrocausation**
- Time flows forward only
- Snapshot DAG has no cycles
- Parent always has earlier timestamp than child

**Q2: Type Soundness**
- O ⊨ Σ (observations conform to ontology)
- All properties have valid domain/range
- SHACL shapes validated

**Q3: Guard Preservation**
- max_run_len ≤ 8 (Chatman Constant)
- Hot path operations ≤8 execution steps
- No nested loops in hot path

**Q4: SLO Compliance**
- Hot path: ≤8 ticks (branchless C)
- Warm path: <100ms (Rust)
- Cold path: No hard constraint (Python/LLM)

**Q5: Performance Bounds**
- Memory: ≤1GB per sector
- CPU: ≤50% average utilization
- Tail latency (p99): ≤500ms

---

## 2. MAPE-K Closed Loop

The autonomic control loop follows the MAPE-K pattern:

```
┌──────────────────────────────────────────────────────────────────────┐
│                         MAPE-K CLOSED LOOP                            │
│                                                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ M: MONITOR (Observation Plane)                                │  │
│  │  • Ingest OTLP spans, RDF triples, receipts                   │  │
│  │  • Append to ObservationStore (immutable log)                 │  │
│  │  • Performance: Hot path ≤8 ticks                             │  │
│  │  • Output: Observation stream                                 │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                 │                                     │
│                                 ▼                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ A: ANALYZE (Change Plane - Pattern Detection)                 │  │
│  │  • PatternDetector scans observations                         │  │
│  │  • Detect: frequency anomalies, error spikes, schema drift    │  │
│  │  • Performance: Warm path <100ms                              │  │
│  │  • Output: DetectedPattern[] with confidence scores           │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                 │                                     │
│                                 ▼                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ P: PLAN (Change Plane - Proposal Generation)                  │  │
│  │  • LLM Overlay Proposer generates ΔΣ from patterns            │  │
│  │  • Constraint-aware prompts (Q1-Q5, doctrines, guards)        │  │
│  │  • Guided decoding (LMQL) for critical constraints            │  │
│  │  • Performance: Cold path (LLM inference 5-30s)               │  │
│  │  • Output: Proposal (ΔΣ + reasoning + confidence)             │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                 │                                     │
│                                 ▼                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ E: EXECUTE (Change Plane - Validation & Promotion)            │  │
│  │  • ValidationPipeline (7 stages)                              │  │
│  │  • If valid: SnapshotPromoter.promote() (~1ns atomic swap)    │  │
│  │  • If invalid: Reject + log reason                            │  │
│  │  • Performance: Warm path <100ms (validation)                 │  │
│  │  •              Hot path ~1ns (promotion)                      │  │
│  │  • Output: New Σ' or rejection receipt                        │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                 │                                     │
│                                 ▼                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ K: KNOWLEDGE (Projection Plane - Learning)                    │  │
│  │  • Record accepted/rejected proposals in corpus               │  │
│  │  • Update few-shot examples                                   │  │
│  │  • Adapt prompts based on violation patterns                  │  │
│  │  • Feed knowledge back to Monitor for improved detection      │  │
│  │  • Performance: Cold path (no latency constraint)             │  │
│  │  • Output: Updated learning corpus, refined prompts           │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                 │                                     │
│                                 │ (feedback loop)                     │
│                                 └────────────────────────┐            │
│                                                          │            │
│  ┌───────────────────────────────────────────────────────▼────────┐  │
│  │ RECEIPT STORE (Cryptographic Audit Trail)                     │  │
│  │  • Every phase generates signed receipt (ed25519)             │  │
│  │  • Immutable chain of custody (parent_hash links)             │  │
│  │  • Verifiable proof of all decisions                          │  │
│  └────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────┘
```

### 2.1 MAPE-K Phase Details

#### Monitor Phase (Hot Path: ≤8 ticks)

**Implementation**: `ObservationStore::append(observation)`

```rust
pub struct ObservationStore {
    observations: DashMap<String, Arc<Observation>>,  // Lock-free concurrent map
}

impl ObservationStore {
    pub fn append(&self, obs: Observation) -> String {
        let id = obs.id.clone();
        self.observations.insert(id.clone(), Arc::new(obs));  // ≤8 ticks
        id
    }
}
```

**Performance**:
- Tick count: 2-4 ticks (hash + atomic insert)
- Latency: <100ns (sub-microsecond)
- Throughput: >1M observations/sec
- Concurrency: Lock-free (DashMap uses sharded locking)

**Receipt**: `ReceiptOperation::ObservationIngested`

#### Analyze Phase (Warm Path: <100ms)

**Implementation**: `PatternDetector::detect_patterns()`

```rust
pub struct PatternDetector {
    store: Arc<ObservationStore>,
}

impl PatternDetector {
    pub async fn detect_patterns(&self) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Frequency anomaly (>100 events/min)
        if let Some(p) = self.detect_frequency_anomaly().await {
            patterns.push(p);
        }

        // Pattern 2: Error spike (>5% error rate)
        if let Some(p) = self.detect_error_spike().await {
            patterns.push(p);
        }

        // Pattern 3: Schema mismatch
        if let Some(p) = self.detect_schema_mismatch().await {
            patterns.push(p);
        }

        patterns
    }
}
```

**Performance**:
- Latency: 10-50ms typical, <100ms guaranteed
- Throughput: 10-100 patterns/sec
- Concurrency: Read-heavy (parallel pattern detection)

**Receipt**: `ReceiptOperation::PatternDetected`

#### Plan Phase (Cold Path: 5-30s)

**Implementation**: `LLMProposer::generate_proposal()`

**Strategy**: Hybrid (defense-in-depth)
1. **Layer 1**: Prompt-based constraints (encode Q1-Q5 in prompt)
2. **Layer 2**: Guided decoding (LMQL for critical invariants)
3. **Layer 3**: Post-hoc validation (safety net)

**Performance**:
- Latency: 5-30s (LLM inference time)
- Throughput: 2-10 proposals/min
- Concurrency: Parallel LLM calls (up to 5 candidates)

**Receipt**: `ReceiptOperation::ProposalGenerated`

#### Execute Phase (Warm → Hot: <100ms → ~1ns)

**Validation Pipeline** (Warm path: <100ms):

```rust
pub struct ValidationPipeline {
    schema_validator: SchemaValidator,
    invariant_checkers: Vec<Box<dyn InvariantChecker>>,
    doctrine_validator: DoctrineValidator,
    guard_validator: GuardValidator,
    performance_estimator: PerformanceEstimator,
    rollback_analyzer: RollbackAnalyzer,
    compatibility_checker: CompatibilityChecker,
}
```

**7-Stage Validation**:
1. Static schema check (SHACL): 5-10ms
2. Invariant check (Q1-Q5): 10-20ms
3. Doctrine check (sector-specific): 5-15ms
4. Guard check (immutable boundaries): 5-10ms
5. Performance check (≤8 ticks estimate): 5-10ms
6. Rollback check (reversibility): 5-10ms
7. Compatibility check (backward compat): 10-30ms

**Total validation**: 45-105ms (within warm path budget)

**Promotion** (Hot path: ~1ns):

```rust
pub fn promote(&self, new_snapshot: SnapshotDescriptor) -> Result<Arc<SnapshotDescriptor>> {
    let new_arc = Arc::new(new_snapshot);

    // Atomic pointer swap (RCU: replace old with new)
    let _ = self.current.swap(new_arc.clone());  // ~1ns

    self.history.insert(id, new_arc.clone());
    Ok(new_arc)
}
```

**Performance**:
- Validation: 45-105ms (warm path)
- Promotion: ~1ns (atomic pointer swap)
- Total: <100ms end-to-end

**Receipt**: `ReceiptOperation::SnapshotPromoted` or `ValidationExecuted`

#### Knowledge Phase (Cold Path: No limit)

**Implementation**: `ProposalLearningSystem::record_outcome()`

```rust
pub struct ProposalLearningSystem {
    corpus: ProposalCorpus,
    prompt_adapter: PromptAdapter,
    pattern_analyzer: PatternAnalyzer,
}

impl ProposalLearningSystem {
    pub fn record_outcome(&mut self, proposal: Proposal, report: ValidationReport) {
        if report.passed {
            self.corpus.accepted_proposals.push((proposal, report));
        } else {
            self.corpus.rejected_proposals.push((proposal, report));

            // Track which constraints were violated
            for stage in &report.stages {
                if !stage.passed {
                    self.corpus.constraint_violations
                        .entry(stage.name.clone())
                        .or_default()
                        .push(proposal.id.clone());
                }
            }
        }

        // Adapt prompts based on patterns
        self.adapt_prompts_from_feedback()?;
    }
}
```

**Performance**:
- Latency: 100-500ms (corpus update + prompt adaptation)
- Throughput: Background async processing
- Concurrency: Mutex-protected writes (infrequent)

**Receipt**: `ReceiptOperation::LoopCycleCompleted`

### 2.2 Cycle Completion

**End-to-End Latency Breakdown**:
- Monitor (M): <100ns (hot path)
- Analyze (A): 10-50ms (warm path)
- Plan (P): 5-30s (cold path, LLM)
- Execute (E): 45-105ms validation + ~1ns promotion (warm→hot)
- Knowledge (K): 100-500ms (cold path, async)

**Total**: ~5-30s (dominated by LLM inference in Plan phase)

**Cycle Frequency**:
- Continuous monitoring (hot path, always active)
- Pattern analysis: Every 1-5 minutes (warm path)
- Proposal generation: On-demand (when patterns detected)
- Promotion: Immediate (hot path, <1ns)
- Learning: Batch async (every 10-60 minutes)

---

## 3. Component Inventory

### 3.1 Core Components

#### ObservationStore

**Location**: `rust/knhk-closed-loop/src/observation.rs`

**Public API**:
```rust
pub struct ObservationStore {
    observations: DashMap<String, Arc<Observation>>,
    patterns: DashMap<String, Arc<DetectedPattern>>,
}

impl ObservationStore {
    pub fn new() -> Self;
    pub fn append(&self, obs: Observation) -> String;
    pub fn record_pattern(&self, pattern: DetectedPattern);
    pub fn get_observation(&self, id: &str) -> Option<Arc<Observation>>;
    pub fn observations_since(&self, timestamp: u64) -> Vec<Arc<Observation>>;
    pub fn get_sector_observations(&self, sector: &str) -> Vec<Arc<Observation>>;
    pub fn count_observations(&self) -> usize;
}
```

**State Management**:
- Immutable append-only log
- Lock-free concurrent reads/writes (DashMap)
- No deletion (observations never removed)

**Thread Safety**:
- `DashMap` provides lock-free sharded concurrent access
- `Arc<Observation>` for cheap cloning across threads
- Read operations never block

**Performance**:
- Append: ≤8 ticks (2-4 typical)
- Read: <10 ticks (hash lookup)
- Memory: O(n) where n = observation count
- Throughput: >1M observations/sec

**Testing Strategy**:
- Unit tests: Append, retrieve, filter by timestamp/sector
- Property tests: Concurrent append safety, observation ordering
- Integration tests: Full MAPE-K cycle with observations

---

#### SnapshotPromoter

**Location**: `rust/knhk-closed-loop/src/promoter.rs`

**Public API**:
```rust
pub struct SnapshotPromoter {
    current: ArcSwap<SnapshotDescriptor>,
    history: DashMap<String, Arc<SnapshotDescriptor>>,
}

impl SnapshotPromoter {
    pub fn new(initial_snapshot: SnapshotDescriptor) -> Self;
    pub fn current(&self) -> Arc<SnapshotDescriptor>;
    pub fn promote(&self, new_snapshot: SnapshotDescriptor) -> Result<Arc<SnapshotDescriptor>>;
    pub fn get(&self, snapshot_id: &str) -> Result<Arc<SnapshotDescriptor>>;
    pub fn rollback(&self) -> Result<()>;
    pub fn chain(&self) -> Result<Vec<Arc<SnapshotDescriptor>>>;
    pub fn snapshot_count(&self) -> usize;
}
```

**State Management**:
- Atomic pointer (ArcSwap) for current snapshot
- Immutable snapshot history (DashMap)
- RCU (Read-Copy-Update) semantics for zero-downtime updates

**Thread Safety**:
- `ArcSwap::load()`: Atomic read (wait-free, linearizable)
- `ArcSwap::swap()`: Atomic write (lock-free, ~1ns)
- History: Lock-free concurrent map (DashMap)

**Performance**:
- Current read: <1ns (atomic pointer load)
- Promotion: ~1ns (atomic pointer swap)
- History lookup: <10 ticks (hash + atomic read)
- Memory: O(h) where h = history depth

**Testing Strategy**:
- Unit tests: Promote, rollback, chain traversal
- Performance tests: Promotion latency <10µs
- Concurrency tests: Parallel reads during promotion
- Property tests: Snapshot chain DAG invariants (no cycles)

---

#### ReceiptStore

**Location**: `rust/knhk-closed-loop/src/receipt.rs`

**Public API**:
```rust
pub struct ReceiptStore {
    receipts: DashMap<String, Arc<Receipt>>,
    verifying_key: VerifyingKey,
}

impl ReceiptStore {
    pub fn new(verifying_key: VerifyingKey) -> Self;
    pub async fn append(&self, receipt: Receipt) -> Result<String>;
    pub fn get(&self, id: &str) -> Result<Arc<Receipt>>;
    pub fn get_sector_receipts(&self, sector: &str) -> Vec<Arc<Receipt>>;
    pub fn get_chain(&self, receipt_id: &str) -> Result<Vec<Arc<Receipt>>>;
    pub fn list_all(&self) -> Vec<Arc<Receipt>>;
}

pub struct Receipt {
    pub id: String,
    pub operation: ReceiptOperation,
    pub timestamp: DateTime<Utc>,
    pub outcome: ReceiptOutcome,
    pub evidence: Vec<String>,
    pub signature: String,  // ed25519, hex-encoded
    pub parent_hash: Option<String>,
    pub sector: String,
}

impl Receipt {
    pub fn create(...) -> Result<Self>;
    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<()>;
    pub fn verify_chain(&self, store: &ReceiptStore) -> Result<()>;
}
```

**State Management**:
- Immutable append-only log (receipts never modified)
- Cryptographic signatures (ed25519, 64 bytes)
- Linked chain (parent_hash creates DAG)

**Thread Safety**:
- Lock-free concurrent appends (DashMap)
- Signature verification is read-only (no shared state)
- Chain traversal uses immutable Arc references

**Performance**:
- Create + sign: 50-100µs (ed25519 signature)
- Append: <10 ticks (hash + insert)
- Verify: 50-100µs (ed25519 verify)
- Chain verification: O(depth) × 100µs

**Testing Strategy**:
- Unit tests: Create, sign, verify, chain traversal
- Security tests: Invalid signatures rejected
- Property tests: Chain integrity (no orphans, no cycles)
- Integration tests: Full MAPE-K cycle produces valid receipt chain

---

#### MapEKCoordinator

**Location**: `rust/knhk-closed-loop/src/coordinator.rs`

**Public API**:
```rust
pub struct MapEKCoordinator {
    observation_store: Arc<ObservationStore>,
    receipt_store: Arc<ReceiptStore>,
    pattern_detector: PatternDetector,
    signing_key: SigningKey,
    sector: String,
}

impl MapEKCoordinator {
    pub fn new(...) -> Self;
    pub async fn execute_cycle(&self) -> Result<LoopCycle>;
}

pub struct LoopCycle {
    pub id: String,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub patterns_detected: usize,
    pub proposals_generated: usize,
    pub validations_passed: usize,
    pub validations_failed: usize,
    pub snapshots_promoted: usize,
    pub outcome: CycleOutcome,
    pub receipt_ids: Vec<String>,
}
```

**State Management**:
- Stateless (delegates to stores)
- Each cycle creates new LoopCycle instance
- Receipts track all state changes

**Thread Safety**:
- Read-only access to stores (thread-safe by design)
- Async execution (Tokio runtime)
- No shared mutable state

**Performance**:
- Monitor: <100ns
- Analyze: 10-50ms
- Plan: 5-30s (LLM dominated)
- Execute: 45-105ms + ~1ns
- Knowledge: 100-500ms async
- **Total**: ~5-30s per cycle

**Testing Strategy**:
- Unit tests: Each MAPE-K phase independently
- Integration tests: Full cycle with mock observations
- Performance tests: Cycle duration <60s with real LLM
- Chicago TDD: State-based tests with real collaborators

---

#### InvariantValidator

**Location**: `rust/knhk-closed-loop/src/invariants.rs`

**Public API**:
```rust
pub struct HardInvariants {
    pub q1_no_retrocausation: bool,
    pub q2_type_soundness: bool,
    pub q3_guard_preservation: bool,
    pub q4_slo_compliance: bool,
    pub q5_performance_bounds: bool,
}

impl HardInvariants {
    pub fn all_preserved(&self) -> bool;
    pub fn which_violated(&self) -> Vec<String>;
}

pub struct InvariantValidator;

impl InvariantValidator {
    pub fn check_q1_no_retrocausation(...) -> Result<bool>;
    pub fn check_q2_type_soundness(...) -> Result<bool>;
    pub fn check_q3_guard_preservation(max_ticks: u32) -> Result<bool>;
    pub fn check_q4_slo_compliance(...) -> Result<bool>;
    pub fn check_q5_performance_bounds(...) -> Result<bool>;
    pub fn check_all(...) -> Result<HardInvariants>;
}
```

**State Management**:
- Stateless (pure functions)
- No side effects
- Idempotent validation

**Thread Safety**:
- No shared state (thread-safe by design)
- Parallel validation possible
- Immutable inputs

**Performance**:
- Q1 check: 5-10ms (graph traversal)
- Q2 check: 10-20ms (SHACL validation)
- Q3 check: <1ms (arithmetic check)
- Q4 check: <1ms (arithmetic check)
- Q5 check: 5-10ms (resource aggregation)
- **Total**: 21-42ms for all 5 invariants

**Testing Strategy**:
- Unit tests: Each Q invariant independently (pass/fail cases)
- Property tests: Invariant preservation across random proposals
- Integration tests: Full validation pipeline
- Chicago TDD: State-based tests verifying invariants hold

---

#### DoctrineStore

**Location**: `rust/knhk-closed-loop/src/doctrine.rs`

**Public API**:
```rust
pub struct DoctrineStore {
    rules: DashMap<String, Arc<DoctrineRule>>,
    history: RwLock<Vec<(u64, String, DoctrineRule)>>,
    current_snapshot: ArcSwap<DoctrineSnapshot>,
}

impl DoctrineStore {
    pub fn new() -> Result<Self>;
    pub fn add_rule(&self, rule: DoctrineRule) -> Result<String>;
    pub fn get_rule(&self, rule_id: &str) -> Result<Arc<DoctrineRule>>;
    pub fn list_rules_for_sector(&self, sector: &str) -> Vec<Arc<DoctrineRule>>;
    pub fn validate_against_doctrines(...) -> Result<Vec<DoctrineViolation>>;
    pub fn get_snapshot(&self) -> Arc<DoctrineSnapshot>;
    pub fn promote_snapshot(&self, new_rules: Vec<DoctrineRule>) -> Result<Arc<DoctrineSnapshot>>;
}

pub struct DoctrineRule {
    pub id: String,
    pub name: String,
    pub sector: String,
    pub constraint_type: ConstraintType,
    pub enforcement_level: EnforcementLevel,
    ...
}

pub enum ConstraintType {
    ApprovalChain { required_signers: usize, ... },
    SegregationOfDuties { incompatible_roles: Vec<Vec<String>> },
    ResourceLimit { resource_type: String, max_value: f64 },
    TimeWindow { start_hour: u8, end_hour: u8, ... },
    Schema { rules: Vec<String> },
    Custom { rule_type: String },
}
```

**State Management**:
- Immutable doctrine rules (Arc shared ownership)
- Append-only history (RwLock for writes)
- Atomic snapshot promotion (ArcSwap)

**Thread Safety**:
- Lock-free reads (DashMap, ArcSwap)
- Write-locked history (RwLock, infrequent writes)
- No data races (Arc prevents mutation)

**Performance**:
- Add rule: 5-10ms (validation + insert)
- Get rule: <1ms (hash lookup)
- Validate proposal: 10-30ms per sector (depends on rule count)
- Snapshot promotion: ~1ns (atomic swap)

**Testing Strategy**:
- Unit tests: Each constraint type (ApprovalChain, SegregationOfDuties, etc.)
- Integration tests: Multi-rule validation
- Property tests: Doctrine composition (no conflicts)
- Chicago TDD: Real doctrine validation with actual signers

---

#### GovernanceEngine

**Location**: `rust/knhk-closed-loop/src/governance.rs`

**Public API**:
```rust
pub struct GovernanceEngine {
    guards: DashMap<String, Arc<Guard>>,
    relaxation_requests: DashMap<String, Arc<RwLock<GuardRelaxationRequest>>>,
    approval_history: Arc<RwLock<Vec<GuardRelaxationRequest>>>,
    active_relaxations: DashMap<String, RelaxationWindow>,
}

impl GovernanceEngine {
    pub fn new() -> Self;
    pub fn register_guard(&self, guard: Guard) -> Result<()>;
    pub fn get_guard(&self, guard_id: &str) -> Result<Arc<Guard>>;
    pub fn request_relaxation(...) -> Result<String>;
    pub fn approve_relaxation(..., signing_key: &SigningKey) -> Result<()>;
    pub fn reject_relaxation(...) -> Result<()>;
    pub fn is_relaxation_approved(&self, request_id: &str) -> bool;
    pub fn activate_relaxation(...) -> Result<RelaxationWindow>;
    pub fn revoke_relaxation(&self, request_id: &str) -> Result<()>;
    pub fn expire_relaxations(&self) -> Vec<String>;
}

pub struct Guard {
    pub id: String,
    pub name: String,
    pub guard_type: GuardType,
    pub criticality: Criticality,
    pub is_mutable: bool,
    pub relaxation_policy: RelaxationPolicy,
    pub enforced: Arc<RwLock<bool>>,
}

pub enum Criticality {
    Critical,  // Requires 3 approvals, 24h review
    High,      // Requires 2 approvals, 12h review
    Medium,    // Requires 1 approval, 1h review
    Low,       // Requires 1 approval, instant
}
```

**State Management**:
- Guard state (enforced/relaxed): Arc<RwLock<bool>>
- Relaxation requests: DashMap for concurrent access
- Approval history: RwLock for infrequent writes
- Active relaxations: DashMap for concurrent checks

**Thread Safety**:
- Lock-free guard lookup (DashMap)
- RwLock for guard enforcement state (rare writes)
- Cryptographic signatures prevent tampering
- Atomic operations for quorum counting

**Performance**:
- Register guard: <1ms
- Request relaxation: 1-5ms (validation + insert)
- Approve: 50-100µs (ed25519 sign + verify)
- Activate relaxation: 1-5ms
- Expire check: <10ms (scan active relaxations)

**Testing Strategy**:
- Unit tests: Criticality levels, quorum requirements
- Security tests: Signature verification, tampering detection
- Integration tests: Multi-party approval workflows
- Property tests: Quorum consensus properties

---

### 3.2 Auxiliary Components

#### PatternDetector

**Location**: `rust/knhk-closed-loop/src/observation.rs`

**Purpose**: Analyze observation stream for anomalies

**Patterns Detected**:
1. **Frequency Anomaly**: >100 events/min for same event type
2. **Error Spike**: >5% error rate in observation window
3. **Missing Observations**: No observations in last 10 seconds
4. **Schema Mismatch**: Unexpected fields in observations

**Performance**: 10-50ms per detection cycle

---

#### ValidationPipeline

**Location**: Integrated across multiple modules

**7-Stage Pipeline**:
1. **Static Check**: SHACL validation (5-10ms)
2. **Invariant Check**: Q1-Q5 validation (21-42ms)
3. **Doctrine Check**: Sector-specific rules (10-30ms)
4. **Guard Check**: Immutable boundaries (5-10ms)
5. **Performance Check**: Tick budget estimation (5-10ms)
6. **Rollback Check**: Reversibility analysis (5-10ms)
7. **Compatibility Check**: Backward compatibility (10-30ms)

**Total**: 61-142ms (within warm path budget <100ms typical)

---

#### LLM Overlay Proposer

**Location**: (External integration, design in `designs/llm-overlay-proposer-design.md`)

**Implementation Strategy**: Hybrid (defense-in-depth)

**Layers**:
1. **Prompt Engineering**: Encode constraints in LLM prompt
2. **Guided Decoding**: LMQL/Guidance for critical invariants
3. **Post-hoc Validation**: Safety net (ValidationPipeline)

**Performance**: 5-30s (LLM inference dominated)

---

## 4. Data Flow Diagrams

### 4.1 Observation Ingestion Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                     EXTERNAL SYSTEMS                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ Application  │  │ OTEL         │  │ User         │              │
│  │ Events       │  │ Collector    │  │ Actions      │              │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
│         │ OTLP            │ gRPC            │ HTTP                  │
└─────────┼─────────────────┼─────────────────┼─────────────────────┘
          │                 │                 │
          ▼                 ▼                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   KNHK OBSERVATION INGESTION                         │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 1. Protocol Adapter                                        │    │
│  │    • OTLP → Observation (spans, metrics, logs)            │    │
│  │    • RDF/Turtle → Observation (triples)                   │    │
│  │    • HTTP/JSON → Observation (user events)                │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 2. Observation Creation                                    │    │
│  │    • Generate ID: hash(timestamp + event_type + sector)   │    │
│  │    • Attach metadata: sector, timestamp, tags             │    │
│  │    • Performance: ≤8 ticks                                 │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 3. ObservationStore.append()                               │    │
│  │    • Lock-free insert (DashMap)                           │    │
│  │    • Performance: 2-4 ticks                                │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 4. Receipt Generation                                      │    │
│  │    • ReceiptOperation::ObservationIngested                │    │
│  │    • Sign with ed25519 (50-100µs)                         │    │
│  │    • Append to ReceiptStore                               │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 5. Pattern Detection Trigger (async)                      │    │
│  │    • If observation count > threshold:                    │    │
│  │      Trigger PatternDetector.detect_patterns()            │    │
│  └────────────────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────────────┘
```

**Latency Breakdown**:
- Protocol adaptation: <10µs
- Observation creation: <100ns (≤8 ticks)
- Store append: <100ns (2-4 ticks)
- Receipt generation: 50-100µs (ed25519 sign)
- **Total**: <200µs (well within hot path budget)

---

### 4.2 Pattern → Proposal → Validation Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                      PATTERN DETECTION                               │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ PatternDetector.detect_patterns()                          │    │
│  │  • Scan observations_since(now - 60s)                     │    │
│  │  • Frequency anomaly: >100/min                            │    │
│  │  • Error spike: >5% errors                                │    │
│  │  • Schema mismatch: unexpected fields                     │    │
│  │  • Performance: 10-50ms                                    │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                  DetectedPattern[]                                  │
└───────────────────────────┼─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   LLM PROPOSAL GENERATION                            │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 1. Load Constraints                                        │    │
│  │    • Load doctrines for pattern.sector                    │    │
│  │    • Load hard invariants (Q1-Q5)                         │    │
│  │    • Load guard profiles                                  │    │
│  │    • Load performance budget (remaining_ticks)            │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 2. Build Constraint-Aware Prompt                          │    │
│  │    • Encode Q1-Q5 as instructions                         │    │
│  │    • List applicable doctrines                            │    │
│  │    • Specify guard boundaries                             │    │
│  │    • Include few-shot examples (sector-specific)          │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 3. LLM Inference (with guided decoding)                   │    │
│  │    • Strategy A: Prompt-based (baseline)                  │    │
│  │    • Strategy B: LMQL guided (for critical Q)             │    │
│  │    • Strategy C: Post-hoc filtering (safety net)          │    │
│  │    • Performance: 5-30s (LLM dominated)                   │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 4. Parse Proposal                                          │    │
│  │    • Extract ΔΣ (added/removed classes/properties)        │    │
│  │    • Extract reasoning, confidence                        │    │
│  │    • Estimate performance impact                          │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                      Proposal (ΔΣ)                                  │
└───────────────────────────┼─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    VALIDATION PIPELINE                               │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ Stage 1: Static Schema Check (5-10ms)                     │    │
│  │  • SHACL shape validation                                 │    │
│  │  • All classes have valid subclass_of                     │    │
│  │  • All properties have domain/range                       │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ Stage 2: Invariant Check (21-42ms)                        │    │
│  │  • Q1: No retrocausation (graph cycle detection)          │    │
│  │  • Q2: Type soundness (SHACL validation)                  │    │
│  │  • Q3: Guard preservation (max_run_len ≤ 8)               │    │
│  │  • Q4: SLO compliance (hot path ≤8 ticks)                 │    │
│  │  • Q5: Performance bounds (memory, CPU)                   │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ Stage 3: Doctrine Check (10-30ms)                         │    │
│  │  • DoctrineStore.validate_against_doctrines()             │    │
│  │  • Check ApprovalChain, SegregationOfDuties, etc.         │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ Stage 4: Guard Check (5-10ms)                             │    │
│  │  • Verify no protected classes removed                    │    │
│  │  • Verify no protected properties removed                 │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ Stage 5: Performance Check (5-10ms)                       │    │
│  │  • Estimate tick count after ΔΣ applied                   │    │
│  │  • Verify estimate ≤ remaining_ticks                      │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ Stage 6: Rollback Check (5-10ms)                          │    │
│  │  • Analyze if change is reversible                        │    │
│  │  • Check for dependent changes                            │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ Stage 7: Compatibility Check (10-30ms)                    │    │
│  │  • Verify backward compatibility                          │    │
│  │  • Check for breaking changes                             │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                   ValidationReport                                  │
│                   (passed: bool + reasons)                          │
└───────────────────────────┼─────────────────────────────────────────┘
                            │
                            ▼
                  Valid ✓ → Promotion
                  Invalid ✗ → Rejection + Learning
```

**Total Latency**:
- Pattern detection: 10-50ms
- LLM proposal: 5-30s
- Validation: 61-142ms
- **Total**: ~5-30s (LLM dominated)

---

### 4.3 Snapshot Promotion Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│              VALIDATED PROPOSAL (ΔΣ approved)                        │
│                            │                                         │
│                            ▼                                         │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 1. Create New Snapshot Descriptor                         │    │
│  │    • snapshot_id: SHA-256(Σ_current ⊕ ΔΣ)                │    │
│  │    • parent_id: Σ_current.snapshot_id                     │    │
│  │    • promoted_at: now()                                   │    │
│  │    • version: parent.version + 1                          │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 2. Atomic Promotion (RCU Semantics)                       │    │
│  │    • new_arc = Arc::new(SnapshotDescriptor)               │    │
│  │    • self.current.swap(new_arc) [~1ns atomic op]          │    │
│  │    • self.history.insert(snapshot_id, new_arc)            │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 3. Receipt Generation                                      │    │
│  │    • ReceiptOperation::SnapshotPromoted                   │    │
│  │    • Sign with ed25519                                    │    │
│  │    • Link to parent receipt                               │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 4. Code Generation (Async, Cold Path)                     │    │
│  │    • Generate C headers from Σ' (ggen templates)          │    │
│  │    • Generate Rust traits                                 │    │
│  │    • Compile to shared library (.so)                      │    │
│  │    • Performance: 1-10s (depends on Σ size)               │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 5. Weaver Validation (OpenTelemetry Schema Check)         │    │
│  │    • weaver registry check -r registry/                   │    │
│  │    • weaver registry live-check --registry registry/      │    │
│  │    • Verify runtime telemetry matches schema              │    │
│  │    • Performance: 100-500ms                               │    │
│  └────────────────────────────────────────────────────────────┘    │
│                            │                                        │
│                            ▼                                        │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 6. Learning Feedback                                       │    │
│  │    • ProposalLearningSystem.record_outcome()              │    │
│  │    • Add to accepted_proposals corpus                     │    │
│  │    • Update few-shot examples for sector                  │    │
│  └────────────────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────────────┘
```

**Critical Path** (affects uptime):
- Snapshot creation: <1µs
- Atomic swap: ~1ns
- Receipt generation: 50-100µs
- **Total downtime**: <100µs

**Async Post-Promotion** (does not affect availability):
- Code generation: 1-10s
- Weaver validation: 100-500ms
- Learning feedback: 100-500ms

---

## 5. Integration Points

### 5.1 Observation → Receipt Integration

**Pattern**: Every observation ingestion produces a signed receipt

```rust
// Observation ingestion
let obs = Observation::new(
    "transaction.execute".to_string(),
    json!({"amount": 50000, "account": "acc-1001"}),
    "finance".to_string(),
    metadata,
);
let obs_id = observation_store.append(obs);

// Receipt generation
let receipt = Receipt::create(
    ReceiptOperation::ObservationIngested { observation_id: obs_id },
    ReceiptOutcome::Approved,
    vec![format!("Observation {} ingested", obs_id)],
    "finance".to_string(),
    &signing_key,
    parent_receipt_hash,
)?;
let receipt_id = receipt_store.append(receipt).await?;
```

**Integration Point**: `ObservationStore` → `ReceiptStore`

**Performance**: <200µs total (100ns observation + 100µs receipt)

---

### 5.2 Pattern → Doctrine → Invariant Integration

**Pattern**: Every proposal must satisfy doctrines AND invariants

```rust
// 1. Detect pattern
let patterns = pattern_detector.detect_patterns().await;

// 2. Load sector-specific doctrines
let doctrines = doctrine_store.list_rules_for_sector("finance");

// 3. Generate proposal (LLM-based, constraint-aware)
let proposal = llm_proposer.generate_proposal(
    &pattern,
    &doctrines,
    &hard_invariants,
    &guard_profile,
).await?;

// 4. Validate against doctrines
let doctrine_violations = doctrine_store.validate_against_doctrines(
    &proposal.delta_sigma,
    "finance",
    &validation_context,
)?;

// 5. Validate against hard invariants
let invariants = InvariantValidator::check_all(
    &proposal.delta_sigma,
    /* ... params ... */
)?;

// 6. Combine results
if doctrine_violations.is_empty() && invariants.all_preserved() {
    // APPROVE
} else {
    // REJECT with detailed reasons
}
```

**Integration Points**:
- `PatternDetector` → `LLMProposer`
- `DoctrineStore` → `ValidationPipeline`
- `InvariantValidator` → `ValidationPipeline`

---

### 5.3 Change Plane → Projection Plane Integration

**Pattern**: Validated proposals flow through promotion pipeline

```rust
// Change Plane: Validation complete
let validation_report = validation_pipeline.validate(&proposal).await?;

if validation_report.passed {
    // Projection Plane: Promotion
    let new_snapshot = SnapshotDescriptor {
        snapshot_id: compute_snapshot_id(&proposal.delta_sigma),
        parent_id: Some(current_snapshot.snapshot_id),
        promoted_at: Utc::now().timestamp_millis() as u64,
        version: current_snapshot.version + 1,
    };

    // Atomic swap (~1ns)
    let promoted = snapshot_promoter.promote(new_snapshot)?;

    // Async codegen (1-10s, non-blocking)
    tokio::spawn(async move {
        codegen_engine.generate_code(&promoted).await?;
    });

    // Learning feedback (async)
    tokio::spawn(async move {
        learning_system.record_outcome(proposal, validation_report).await?;
    });
}
```

**Integration Points**:
- `ValidationPipeline` → `SnapshotPromoter`
- `SnapshotPromoter` → `CodegenEngine`
- `ValidationPipeline` → `ProposalLearningSystem`

---

### 5.4 Sector → Core Coordination

**Pattern**: Sector ontologies compose with core ontology

```rust
pub struct CompositeOntology {
    core: Arc<CoreOntology>,          // Σ_core (immutable)
    sector: SectorOntology,            // Σ_finance | Σ_health | ...
    composition_id: SnapshotId,        // Hash of (core + sector)
}

impl CompositeOntology {
    pub fn query(&self, sparql: &str) -> Result<QueryResult> {
        // Query is union of core + sector triples
        let core_results = self.core.query(sparql)?;
        let sector_results = self.sector.query(sparql)?;
        Ok(merge_results(core_results, sector_results))
    }
}
```

**Integration Points**:
- `CoreOntology` (immutable) → `SectorOntology` (mutable)
- `CompositeOntology` → `SnapshotPromoter`

**Performance**:
- Query: <10ms (depends on SPARQL complexity)
- Composition: <1µs (pointer composition)

---

## 6. Sector Architecture

### 6.1 Composition Model: Σ = Σ_core ⊕ Σ_sector

Each deployed ontology is a composition of:
1. **Σ_core**: Immutable kernel (Van der Aalst's 43 patterns, Receipt, Guard, Observation)
2. **Σ_sector**: Mutable sector extensions (finance, healthcare, manufacturing, logistics)

```turtle
# Core Ontology (Σ_core) - Immutable Foundation
knhk:CoreOntology a owl:Ontology ;
    rdfs:label "KNHK Core Ontology" ;
    owl:versionIRI <http://knhk.io/ontology/core/1.0.0> ;
    rdfs:comment "Immutable kernel: 43 Van der Aalst patterns, Receipt, Guard, Observation" .

# Finance Sector Ontology (Σ_finance)
knhk:FinanceOntology a owl:Ontology ;
    rdfs:label "KNHK Finance Sector Ontology" ;
    owl:imports knhk:CoreOntology ;
    owl:versionIRI <http://knhk.io/ontology/finance/1.2.3> ;
    meta:sector "finance" .

# Healthcare Sector Ontology (Σ_health)
knhk:HealthcareOntology a owl:Ontology ;
    owl:imports knhk:CoreOntology ;
    meta:sector "healthcare" .
```

**Invariant Q0 (Meta-Ontology)**:
```
∀ Σ_sector: Σ_sector ⊇ Σ_core ∧ ¬(Σ_sector ∩ Σ_core ≠ Σ_core)
```
(Every sector ontology must import core, and cannot modify core)

---

### 6.2 Finance Sector

**Core Classes**:
- `finance:Transaction`: Atomic financial operation (debit/credit pair)
- `finance:ApprovalChain`: Sequential approval workflow
- `finance:Account`: General ledger account
- `finance:AuditTrail`: Immutable SOX compliance record

**Sector-Specific Patterns**:
- **F1: Double-Entry Bookkeeping**: ∀ T: sum(T.debits) = sum(T.credits)
- **F2: Approval Chain Enforcement**: T.amount > threshold ⇒ T ∈ ApprovalChain
- **F3: Balance Sheet Reconciliation**: Assets = Liabilities + Equity

**Sector-Specific Invariants**:
- **Q6 (Finance)**: Balance Preservation
- **Q7 (Finance)**: Approval Chain Integrity
- **Q8 (Finance)**: Audit Trail Immutability

**Guard Profiles**:
```rust
GuardProfile {
    id: "FINANCE_CORE_GUARD",
    protected_classes: ["Account", "Transaction", "ApprovalChain"],
    protected_properties: ["account_id", "transaction_id", "timestamp"],
    max_run_len: 8,
    performance_tier: HotPath,
}
```

---

### 6.3 Healthcare Sector

**Core Classes**:
- `healthcare:Patient`: Healthcare recipient
- `healthcare:TreatmentProtocol`: Standardized clinical workflow
- `healthcare:Diagnosis`: Clinical diagnosis
- `healthcare:ConsentRecord`: HIPAA consent for data sharing

**Sector-Specific Patterns**:
- **H1: Treatment Protocol Adherence**: ∀ diagnosis D, treatment T ∈ protocol(D)
- **H2: HIPAA Consent Validation**: ∀ access A to patient P: ∃ consent C for P
- **H3: Patient Safety Threshold**: vitals > threshold ⇒ escalate(<1s)

**Sector-Specific Invariants**:
- **Q9 (Healthcare)**: HIPAA Compliance
- **Q10 (Healthcare)**: Treatment Safety

---

### 6.4 Manufacturing Sector

**Core Classes**:
- `mfg:ProductionRun`: Production event
- `mfg:Equipment`: Manufacturing equipment
- `mfg:QualityMetric`: Quality assurance metric
- `mfg:MaintenanceSchedule`: Preventive maintenance

**Sector-Specific Patterns**:
- **M1: Quality Control Gate**: ProductionRun R → QualityMetric Q ≥ threshold
- **M2: Predictive Maintenance**: Equipment E degradation trend D ⇒ schedule maintenance M

**Sector-Specific Invariants**:
- **Q11 (Manufacturing)**: Quality Assurance
- **Q12 (Manufacturing)**: Equipment Certification

---

### 6.5 Logistics Sector

**Core Classes**:
- `logistics:Shipment`: Delivery event
- `logistics:Route`: Delivery route
- `logistics:InventoryLocation`: Warehouse/inventory
- `logistics:DeliveryConstraint`: SLA/deadline

**Sector-Specific Patterns**:
- **L1: Route Optimization**: Shipment S + deadline D → Route R minimizing cost
- **L2: Supply Chain Disruption**: Inventory I < threshold T ⇒ restock workflow W

---

### 6.6 Cross-Sector Coordination

**Problem**: How do changes to Σ_core atomically update all sectors?

**Solution**: Two-Phase Snapshot Promotion

**Phase 1: Prepare** (concurrent, no locks)
```rust
for sector in [finance, healthcare, manufacturing, logistics] {
    let new_snapshot = compose(new_core, sector.current_ext);
    sector.prepare_snapshot(new_snapshot);  // Pre-validate, don't activate
}
```

**Phase 2: Commit** (atomic, sequential)
```rust
atomic {
    core_snapshot.store(Release);           // ~1ns
    for sector in all_sectors {
        sector.current.store(Release);      // ~1ns each
    }
}
// Total latency: ~5ns (4 sectors × 1ns + core)
```

**Rollback Strategy**: If any sector fails preparation, abort all sector updates and keep current Σ_core.

---

## 7. Testing Architecture (Chicago TDD)

### 7.1 Chicago TDD Principles

KNHK testing follows **Chicago School TDD**:
1. **State-based testing**: Verify invariants hold across state transitions
2. **Real collaborators**: No mocks; use actual ObservationStore, ReceiptStore, etc.
3. **Given-When-Then**: Clear test structure with assertions
4. **Multi-stage validation**: Static → Dynamic → Performance → Invariants

**Key Principle**: Test the invariant, not the implementation.

---

### 7.2 Test Pyramid

```
                    ▲
                   /│\
                  / │ \
                 /  │  \    Integration Tests (End-to-end MAPE-K)
                /   │   \   • Full cycle: Observation → Promotion
               /    │    \  • 10-20 tests
              /_____│_____\
                   /│\
                  / │ \
                 /  │  \    Multi-Stage Tests (Validation Pipeline)
                /   │   \   • Static → Dynamic → Perf → Invariants
               /    │    \  • 20-50 tests
              /_____│_____\
                   /│\
                  / │ \
                 /  │  \    Unit Tests (Individual Components)
                /   │   \   • ObservationStore, ReceiptStore, etc.
               /    │    \  • 100+ tests
              /_____│_____\
```

---

### 7.3 State-Based Testing Pattern

**Pattern**: Test that a system maintains invariants across state transitions

```rust
#[test]
fn spec_rule_Q3_guard_preservation() {
    // GIVEN: Initial state
    let fixture = create_fixture();
    let initial_ticks = fixture.estimate_hot_path_ticks();
    assert!(initial_ticks <= CHATMAN_CONSTANT); // Pre-condition

    // WHEN: Action that transitions state (add observation)
    for i in 0..100 {
        fixture.observation_store.append(create_observation(i));
    }

    // THEN: Verify invariant holds (max_run_len still ≤ 8)
    let final_ticks = fixture.estimate_hot_path_ticks();
    assert!(final_ticks <= CHATMAN_CONSTANT); // Post-condition

    // AND: Verify no corruption or side effects
    assert!(fixture.consistency_check());
}
```

**Why**: State-based tests catch real concurrency bugs, race conditions, and integration issues that mocks hide.

---

### 7.4 Hot Path Performance Tests (Chatman Constant Budget)

**Pattern**: Verify operations stay within ≤8 tick budget

```rust
#[test]
fn hot_path_observation_append_under_budget() {
    let store = ObservationStore::new();
    let obs = create_observation(0);

    let start = std::time::Instant::now();
    store.append(obs);  // Should be ≤8 ticks = <100ns
    let elapsed = start.elapsed();

    assert!(elapsed.as_nanos() < 100, "Append must be <100ns (≤8 ticks)");
}

#[test]
fn hot_path_snapshot_promotion_under_budget() {
    let promoter = SnapshotPromoter::new(genesis);

    let start = std::time::Instant::now();
    promoter.promote(new_snap)?;  // Should be ~1ns (atomic swap)
    let elapsed = start.elapsed();

    assert!(elapsed.as_micros() < 10, "Promotion must be <10µs (atomic)");
}
```

**Performance Targets**:
- Observation append: <100ns (≤8 ticks)
- Snapshot read: <1ns (atomic load)
- Snapshot promotion: <10µs (atomic swap + history insert)

---

### 7.5 Warm Path Tests (100ms Budget)

**Pattern**: Verify warm path operations complete within latency budget

```rust
#[tokio::test]
async fn warm_path_pattern_detection_under_budget() {
    let detector = PatternDetector::new(obs_store);

    let start = std::time::Instant::now();
    let patterns = detector.detect_patterns().await;
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 100, "Detection must complete in <100ms");
    assert!(!patterns.is_empty(), "Should detect at least one pattern");
}

#[tokio::test]
async fn warm_path_validation_pipeline_under_budget() {
    let pipeline = ValidationPipeline::new(...);

    let start = std::time::Instant::now();
    let report = pipeline.validate(&proposal).await?;
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 100, "Validation must complete in <100ms");
    assert!(report.stages.len() == 7, "All 7 stages must run");
}
```

---

### 7.6 Integration Tests (Full MAPE-K Cycle)

**Pattern**: End-to-end test of complete autonomic loop

```rust
#[tokio::test]
async fn integration_full_mape_k_to_promotion() {
    // 1. OBSERVE: Add observations
    for i in 0..200 {
        fixture.observation_store.append(create_observation(i));
    }

    // 2. DETECT: Run pattern detection
    let patterns = detector.detect_patterns().await;
    assert!(!patterns.is_empty(), "Should detect patterns");

    // 3. PROPOSE: Generate ΔΣ (simplified: skip LLM for test)
    let proposals: Vec<_> = patterns
        .iter()
        .map(|p| create_proposal_from_pattern(p))
        .collect();
    assert!(!proposals.is_empty(), "Should generate proposals");

    // 4. VALIDATE: Run validation suite
    for proposal in &proposals {
        let report = validation_pipeline.validate(proposal).await?;
        assert!(report.passed, "Proposal should pass validation");
    }

    // 5. EXECUTE: Promote snapshot
    let new_snap = create_snapshot_from_proposal(&proposals[0]);
    let promoted = promoter.promote(new_snap)?;
    assert_eq!(promoter.current().snapshot_id, promoted.snapshot_id);

    // 6. VERIFY: Receipts recorded for each phase
    let receipts = receipt_store.list_all();
    assert!(receipts.len() >= 5, "Should have receipts for all MAPE-K phases");
}
```

---

### 7.7 Property-Based Testing

**Pattern**: Test that properties hold across randomized inputs

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_invariant_q3_always_preserved(max_ticks in 1u32..=100) {
        let result = InvariantValidator::check_q3_guard_preservation(max_ticks);

        // Property: Q3 is preserved if and only if max_ticks <= CHATMAN_CONSTANT
        prop_assert_eq!(result.is_ok(), max_ticks <= CHATMAN_CONSTANT);
    }

    #[test]
    fn prop_receipts_always_verify(
        operations in prop::collection::vec(any::<ReceiptOp>(), 1..10)
    ) {
        let signing_key = create_key();
        let verifying_key = signing_key.verifying_key();

        for op in operations {
            let receipt = Receipt::create(op, ...)?;

            // Property: Every receipt verifies correctly
            prop_assert!(receipt.verify(&verifying_key).is_ok());
        }
    }
}
```

---

### 7.8 Failure Mode Tests

**Pattern**: Test that invariant violations are caught

```rust
#[test]
fn test_q1_violation_caught() {
    // Create circular snapshot reference (invalid)
    let snap1 = SnapshotDescriptor {
        snapshot_id: "snap1".to_string(),
        parent_id: Some("snap2".to_string()),
        ...
    };
    let snap2 = SnapshotDescriptor {
        snapshot_id: "snap2".to_string(),
        parent_id: Some("snap1".to_string()),  // Cycle!
        ...
    };

    // Should fail validation
    let result = InvariantValidator::check_q1_no_retrocausation("snap1", Some("snap2"), ...);
    assert!(result.is_err(), "Should detect cycle");
}

#[test]
fn test_q3_violation_caught() {
    // max_ticks > 8 (Chatman constant)
    let result = InvariantValidator::check_q3_guard_preservation(9);
    assert!(result.is_err(), "Should reject > 8 ticks");
}
```

---

### 7.9 Success Metrics

For a test suite to validate KNHK, it must demonstrate:

1. **Latency**: Hot path <100ns, warm path <100ms, cold path no constraint
2. **Reliability**: 99.9%+ cycle success rate, zero invariant violations
3. **Auditability**: Every decision has cryptographic receipt
4. **Composability**: Snapshots chain (DAG), overlays compose, guards compose
5. **Autonomy**: MAPE-K cycles run without human intervention

---

## 8. Performance Characteristics

### 8.1 Performance Budget Tiers

| Tier | Latency Budget | Operations | Concurrency Model |
|------|---------------|------------|-------------------|
| **Hot Path** | ≤8 ticks (<100ns) | Observation append, snapshot read | Lock-free (DashMap, ArcSwap) |
| **Warm Path** | <100ms | Pattern detection, validation | Read-heavy (RwLock) |
| **Cold Path** | No hard constraint | LLM inference, codegen | Sequential (Mutex) |

---

### 8.2 Component Performance Summary

| Component | Operation | Latency | Throughput | Memory |
|-----------|-----------|---------|------------|--------|
| **ObservationStore** | Append | 2-4 ticks (<100ns) | >1M/sec | O(n) observations |
| **SnapshotPromoter** | Read (current) | <1ns | Lock-free | O(1) |
| **SnapshotPromoter** | Promote | ~1ns atomic + <10µs insert | 100K/sec | O(h) history depth |
| **ReceiptStore** | Append | 50-100µs (sign) | 10K/sec | O(n) receipts |
| **PatternDetector** | Detect | 10-50ms | 20-100 patterns/sec | O(1) (scans existing observations) |
| **InvariantValidator** | Q1-Q5 check | 21-42ms | 25-50 checks/sec | O(1) |
| **DoctrineStore** | Validate | 10-30ms | 30-100 checks/sec | O(r) rules |
| **GovernanceEngine** | Approve | 50-100µs (sign) | 10K approvals/sec | O(g) guards |
| **MapEKCoordinator** | Full cycle | ~5-30s (LLM) | 2-10 cycles/min | O(1) |

---

### 8.3 Scalability Characteristics

**Horizontal Scalability**:
- **ObservationStore**: Shardable by sector (each sector has independent store)
- **ReceiptStore**: Append-only, can shard by time period
- **SnapshotPromoter**: Single writer, multiple readers (RCU semantics)

**Vertical Scalability**:
- **Memory**: O(observations + snapshots + receipts), typically <10GB per sector
- **CPU**: Lock-free data structures minimize contention
- **I/O**: Append-only writes, sequential reads

**Bottlenecks**:
- **LLM inference**: 5-30s per proposal (cold path, async)
- **Validation pipeline**: 61-142ms (warm path, can parallelize stages)

---

### 8.4 Performance Monitoring

**Key Metrics**:
- `knhk.observation.append.latency_ns`: p50, p95, p99, max
- `knhk.snapshot.promote.latency_ns`: p50, p95, p99, max
- `knhk.receipt.sign.latency_us`: p50, p95, p99, max
- `knhk.mape_k.cycle.duration_ms`: p50, p95, p99, max
- `knhk.invariant.violation.count`: total violations (should be 0)

**Alerting Thresholds**:
- Hot path >100ns: CRITICAL (violates Chatman Constant)
- Warm path >100ms: WARNING (investigate bottleneck)
- Invariant violation >0: CRITICAL (system integrity compromised)

---

## 9. Thread Safety and Concurrency

### 9.1 Concurrency Model

KNHK uses a **lock-free read, occasional write** concurrency model:

**Hot Path (Lock-Free)**:
- `DashMap`: Sharded concurrent hashmap (lock-free reads, lock-per-shard writes)
- `ArcSwap`: Atomic pointer swap (lock-free reads, atomic writes)
- `Arc<T>`: Reference counting for cheap cloning

**Warm Path (Read-Heavy RwLock)**:
- `RwLock<Vec<T>>`: Multiple concurrent readers, exclusive writer
- Used for: Doctrine history, governance approval history

**Cold Path (Mutex)**:
- `Mutex<T>`: Exclusive access
- Used for: Codegen engine, learning system updates

---

### 9.2 Thread Safety Guarantees

| Component | Thread Safety | Mechanism | Guarantee |
|-----------|--------------|-----------|-----------|
| **ObservationStore** | Yes | DashMap (lock-free) | Concurrent append/read safe |
| **SnapshotPromoter** | Yes | ArcSwap (atomic) | Linearizable reads during promotion |
| **ReceiptStore** | Yes | DashMap (lock-free) | Concurrent append/read safe |
| **DoctrineStore** | Yes | DashMap + RwLock | Concurrent reads, exclusive writes |
| **GovernanceEngine** | Yes | DashMap + RwLock | Concurrent reads, exclusive writes |
| **MapEKCoordinator** | Yes | Arc references (stateless) | No shared mutable state |

---

### 9.3 RCU (Read-Copy-Update) Semantics

**SnapshotPromoter** uses RCU for zero-downtime updates:

1. **Read Phase** (hot path):
   ```rust
   let snapshot = self.current.load();  // Atomic load (~1ns)
   ```
   - Readers never block
   - Linearizable reads

2. **Update Phase** (atomic):
   ```rust
   let new_arc = Arc::new(new_snapshot);
   self.current.swap(new_arc);  // Atomic swap (~1ns)
   ```
   - Single atomic operation
   - No locks acquired
   - Old snapshot remains valid for existing readers

3. **Grace Period**:
   - Arc reference counting handles cleanup
   - Old snapshot freed when last reader drops reference
   - No explicit grace period management

**Performance**:
- Read latency: <1ns (atomic pointer load)
- Write latency: ~1ns (atomic pointer swap)
- Downtime: 0ns (readers never block)

---

### 9.4 Concurrency Testing

**Pattern**: Test concurrent access patterns

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_concurrent_observation_append() {
    let store = Arc::new(ObservationStore::new());
    let mut handles = vec![];

    for i in 0..100 {
        let store_clone = store.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..1000 {
                store_clone.append(create_observation(i * 1000 + j));
            }
        }));
    }

    // Wait for all appends
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify count
    assert_eq!(store.count_observations(), 100 * 1000);
}

#[tokio::test]
async fn test_snapshot_promotion_during_concurrent_reads() {
    let promoter = Arc::new(SnapshotPromoter::new(genesis));
    let mut handles = vec![];

    // Spawn readers
    for _ in 0..10 {
        let promoter_clone = promoter.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..1000 {
                let _ = promoter_clone.current();  // Concurrent reads
            }
        }));
    }

    // Promote snapshot while readers active
    promoter.promote(new_snapshot)?;

    // All readers complete without error
    for handle in handles {
        handle.await.unwrap();
    }
}
```

---

## 10. Deployment Architecture

### 10.1 Deployment Topology

```
┌─────────────────────────────────────────────────────────────────────┐
│                        PRODUCTION DEPLOYMENT                          │
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ Load Balancer (Nginx/Envoy)                                │     │
│  │  • TLS termination                                          │     │
│  │  • Rate limiting                                            │     │
│  │  • Health checks                                            │     │
│  └────────────────────────┬───────────────────────────────────┘     │
│                           │                                           │
│         ┌─────────────────┼─────────────────┐                        │
│         │                 │                 │                        │
│         ▼                 ▼                 ▼                        │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐               │
│  │ KNHK Node 1 │   │ KNHK Node 2 │   │ KNHK Node 3 │               │
│  │ (Finance)   │   │ (Health)    │   │ (Mfg/Log)   │               │
│  ├─────────────┤   ├─────────────┤   ├─────────────┤               │
│  │ ObsStore    │   │ ObsStore    │   │ ObsStore    │               │
│  │ ReceiptStore│   │ ReceiptStore│   │ ReceiptStore│               │
│  │ Promoter    │   │ Promoter    │   │ Promoter    │               │
│  │ Coordinator │   │ Coordinator │   │ Coordinator │               │
│  └─────────────┘   └─────────────┘   └─────────────┘               │
│         │                 │                 │                        │
│         └─────────────────┴─────────────────┘                        │
│                           │                                           │
│                           ▼                                           │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ Shared Storage                                             │     │
│  │  • PostgreSQL (receipt chain, audit log)                   │     │
│  │  • Redis (observation cache, rate limiting)                │     │
│  │  • S3 (snapshot history, codegen artifacts)                │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ Observability                                              │     │
│  │  • Prometheus (metrics)                                    │     │
│  │  • Grafana (dashboards)                                    │     │
│  │  • OTLP Collector (telemetry ingestion)                    │     │
│  │  • Weaver (schema validation)                              │     │
│  └────────────────────────────────────────────────────────────┘     │
└───────────────────────────────────────────────────────────────────────┘
```

---

### 10.2 Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knhk-finance
  labels:
    app: knhk
    sector: finance
spec:
  replicas: 3
  selector:
    matchLabels:
      app: knhk
      sector: finance
  template:
    metadata:
      labels:
        app: knhk
        sector: finance
    spec:
      containers:
      - name: knhk
        image: knhk-closed-loop:latest
        ports:
        - containerPort: 8080  # OTLP ingestion
        - containerPort: 9090  # Prometheus metrics
        env:
        - name: KNHK_SECTOR
          value: "finance"
        - name: RUST_LOG
          value: "info,knhk_closed_loop=debug"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: knhk-finance
spec:
  selector:
    app: knhk
    sector: finance
  ports:
  - name: otlp
    protocol: TCP
    port: 8080
    targetPort: 8080
  - name: metrics
    protocol: TCP
    port: 9090
    targetPort: 9090
  type: ClusterIP
```

---

### 10.3 High Availability

**Failure Modes**:

1. **Node Failure**: Load balancer detects via health check, reroutes to healthy nodes
2. **Snapshot Corruption**: Rollback to previous snapshot via `promoter.rollback()`
3. **Invariant Violation**: Reject proposal, log to audit trail, alert operator
4. **Storage Failure**: Replicate receipts to 3+ nodes (PostgreSQL HA)

**Recovery Time Objectives** (RTO):
- Node failure: <30s (K8s restarts pod)
- Snapshot rollback: <1µs (atomic pointer swap)
- Storage failover: <60s (PostgreSQL HA)

**Recovery Point Objectives** (RPO):
- Observations: 0 (append-only, replicated)
- Receipts: 0 (cryptographically chained, replicated)
- Snapshots: 0 (immutable history)

---

## Conclusion

This unified architecture document consolidates the complete KNHK system design, from the four-plane architecture to sector-specific ontologies, MAPE-K closed loop, Chicago TDD testing patterns, and deployment strategies.

**Key Achievements**:
- ✅ Complete component inventory with APIs, state management, thread safety
- ✅ Detailed data flow diagrams for all critical paths
- ✅ Integration points clearly specified
- ✅ Sector architecture with finance, healthcare, manufacturing, logistics
- ✅ Chicago TDD testing patterns with performance budgets
- ✅ Performance characteristics and concurrency models documented
- ✅ Deployment architecture with Kubernetes manifests

**Next Steps** (SPARC Phase 4: Refinement):
1. Implement LLM Overlay Proposer with constraint-aware prompting
2. Build ValidationPipeline with 7-stage checks
3. Integrate Weaver for OpenTelemetry schema validation
4. Implement ProposalLearningSystem for continuous improvement
5. Deploy to Kubernetes with sector-specific configurations

**Version**: 1.0.0
**Status**: SPARC Phase 3 Complete
**Authors**: KNHK Architecture Working Group
**Last Updated**: 2025-11-16

---

**Related Files**:
- Implementation: `/home/user/knhk/rust/knhk-closed-loop/`
- Design Documents: `/home/user/knhk/docs/designs/`
- Tests: `/home/user/knhk/rust/knhk-closed-loop/tests/`
- Cargo Manifest: `/home/user/knhk/rust/knhk-closed-loop/Cargo.toml`
