# Autonomous Ontology System - Validation Strategy

**Version:** 1.0
**Status:** Planning Phase
**Last Updated:** 2025-11-16

## Executive Summary

This document defines the comprehensive validation strategy for the autonomous ontology system, aligned with KNHK's core principle: **Weaver validation is the source of truth**. Traditional tests are supporting evidence; only OTel Weaver schema validation proves features work correctly.

## Table of Contents

1. [Validation Principles](#validation-principles)
2. [Testing Hierarchy](#testing-hierarchy)
3. [Weaver Schema Validation Strategy](#weaver-schema-validation-strategy)
4. [Per-Phase Validation Requirements](#per-phase-validation-requirements)
5. [Performance Validation](#performance-validation)
6. [Integration Testing](#integration-testing)
7. [Production Readiness Checklist](#production-readiness-checklist)
8. [CI/CD Validation Gates](#cicd-validation-gates)

---

## 1. Validation Principles

### 1.1 Core Principle: Schema-First Validation

**KNHK exists to eliminate false positives in testing.**

```
Traditional Testing (What KNHK Replaces):
  assert(result == expected) ✅  ← Can pass even when feature is broken
  └─ Tests validate test logic, not production behavior

KNHK Solution:
  Schema defines behavior → Weaver validates runtime telemetry ✅
  └─ Schema validation proves actual runtime behavior matches specification
```

### 1.2 The False Positive Paradox

**We CANNOT validate KNHK using methods that produce false positives.**

| Validation Method | Can Produce False Positives? | Use in KNHK? |
|-------------------|------------------------------|--------------|
| **Weaver schema validation** | ❌ No (external tool, schema-first) | ✅ Source of Truth |
| **Cargo test** | ✅ Yes (tests can test wrong thing) | 🟡 Supporting Evidence |
| **Cargo clippy** | ❌ No (static analysis) | ✅ Code Quality Baseline |
| **Cargo build** | ❌ No (compilation check) | ✅ Required |
| **--help text** | ✅ Yes (help exists ≠ feature works) | ❌ Never trust alone |
| **README validation** | ✅ Yes (docs can claim broken features work) | ❌ Never trust alone |
| **Agent validation** | ✅ Yes (agents can hallucinate) | ❌ Never trust alone |

### 1.3 Validation Hierarchy (CRITICAL)

```
┌─────────────────────────────────────────────────────────┐
│  LEVEL 1: Weaver Schema Validation (MANDATORY)         │
│  ✅ Source of Truth - Proves features work              │
│  - weaver registry check -r registry/                   │
│  - weaver registry live-check --registry registry/      │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │ Validates
                          │
┌─────────────────────────────────────────────────────────┐
│  LEVEL 2: Compilation & Code Quality (Baseline)        │
│  ✅ Proves code is valid Rust/C                         │
│  - cargo build --release                                │
│  - cargo clippy --workspace -- -D warnings              │
│  - make build                                           │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │ Supports
                          │
┌─────────────────────────────────────────────────────────┐
│  LEVEL 3: Traditional Tests (Supporting Evidence)      │
│  🟡 Can have false positives - NOT source of truth      │
│  - cargo test --workspace                               │
│  - make test-chicago-v04                                │
│  - make test-performance-v04                            │
└─────────────────────────────────────────────────────────┘
```

**If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.**

---

## 2. Testing Hierarchy

### 2.1 Level 1: Weaver Schema Validation (Mandatory)

**Purpose**: Prove that runtime behavior matches declared telemetry schema.

**Commands**:
```bash
# Schema validation (static check)
weaver registry check -r registry/

# Live validation (runtime check)
weaver registry live-check --registry registry/
```

**What This Proves**:
- ✅ All claimed OTEL spans/metrics/logs are defined in schema
- ✅ Runtime telemetry conforms to schema declarations
- ✅ No undeclared telemetry (schema drift detection)
- ✅ Attribute types match declarations

**What This Does NOT Prove**:
- ❌ Correctness of business logic (that's application-specific)
- ❌ Performance characteristics (use benchmarks)
- ❌ Error handling completeness (use property tests)

### 2.2 Level 2: Compilation & Code Quality (Baseline)

**Purpose**: Ensure code compiles and meets quality standards.

**Commands**:
```bash
# Build (release mode)
cargo build --workspace --release

# Linting (zero warnings)
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --all -- --check

# C library
make build
```

**What This Proves**:
- ✅ Code is syntactically valid
- ✅ No common code smells (clippy)
- ✅ Consistent formatting
- ✅ Dependencies resolve correctly

### 2.3 Level 3: Traditional Tests (Supporting Evidence)

**Purpose**: Provide additional confidence in specific logic.

**Commands**:
```bash
# Unit tests
cargo test --workspace --lib

# Integration tests
cargo test --workspace --test '*'

# Chicago TDD tests
make test-chicago-v04

# Performance tests
make test-performance-v04
```

**What This Proves**:
- 🟡 Specific logic paths work (if tests are correct)
- 🟡 Edge cases handled (if tests cover them)
- 🟡 Performance targets met (if tests are accurate)

**What This Does NOT Prove**:
- ❌ That tests validate the right behavior
- ❌ That production code path is tested
- ❌ That telemetry is emitted correctly

---

## 3. Weaver Schema Validation Strategy

### 3.1 Schema Definition (registry/knhk-ontology.yaml)

**All Four Planes Must Have Schemas**:

#### Plane O (Observation)

```yaml
groups:
  # Pattern detection span
  - id: knhk.observation.pattern.detect
    type: span
    span_kind: internal
    stability: experimental
    brief: "Detect workflow pattern from telemetry"
    attributes:
      - ref: knhk.observation.pattern_id
      - ref: knhk.observation.confidence
      - ref: knhk.observation.evidence_span_count

  # Anomaly detection span
  - id: knhk.observation.anomaly.detect
    type: span
    span_kind: internal
    stability: experimental
    brief: "Detect conformance violation or anomaly"
    attributes:
      - ref: knhk.observation.anomaly_type
      - ref: knhk.observation.severity

  # Pattern detection metric
  - id: metric.knhk.observation.patterns_detected
    type: metric
    metric_name: knhk.observation.patterns_detected
    stability: experimental
    brief: "Total patterns detected"
    instrument: counter
    unit: "{patterns}"
```

#### Plane Σ (Ontology)

```yaml
groups:
  # Snapshot creation span
  - id: knhk.ontology.snapshot.create
    type: span
    span_kind: internal
    stability: experimental
    brief: "Create immutable ontology snapshot"
    attributes:
      - ref: knhk.ontology.snapshot_id
      - ref: knhk.ontology.parent_snapshot_id
      - ref: knhk.ontology.triple_count
      - ref: knhk.operation.latency_ms

  # Overlay operations span
  - id: knhk.ontology.overlay.create
    type: span
    span_kind: internal
    stability: experimental
    brief: "Create copy-on-write overlay"
    attributes:
      - ref: knhk.ontology.overlay_id
      - ref: knhk.ontology.base_snapshot_id
      - ref: knhk.ontology.delta_size

  # Query span
  - id: knhk.ontology.query
    type: span
    span_kind: internal
    stability: experimental
    brief: "Query ontology with SPARQL"
    attributes:
      - ref: knhk.ontology.query_type
      - ref: knhk.ontology.result_count
      - ref: knhk.operation.latency_ms
```

#### Plane ΔΣ (Change Engine)

```yaml
groups:
  # Change proposal span
  - id: knhk.change.proposal.create
    type: span
    span_kind: internal
    stability: experimental
    brief: "Create ontology change proposal"
    attributes:
      - ref: knhk.change.proposal_id
      - ref: knhk.change.source
      - ref: knhk.change.priority

  # Validation span
  - id: knhk.change.proposal.validate
    type: span
    span_kind: internal
    stability: experimental
    brief: "Validate change proposal against meta-ontology"
    attributes:
      - ref: knhk.change.proposal_id
      - ref: knhk.change.validation_result
      - ref: knhk.change.validator_count
      - ref: knhk.change.consensus_score

  # Queue metric
  - id: metric.knhk.change.queue_depth
    type: metric
    metric_name: knhk.change.queue_depth
    stability: experimental
    brief: "Number of proposals in queue"
    instrument: gauge
    unit: "{proposals}"
```

#### Plane μ, Π, Λ (Projection)

```yaml
groups:
  # Model projection span
  - id: knhk.projection.model.project
    type: span
    span_kind: internal
    stability: experimental
    brief: "Project ontology snapshot to ggen template"
    attributes:
      - ref: knhk.projection.snapshot_id
      - ref: knhk.projection.template_size_bytes
      - ref: knhk.operation.latency_ms

  # Compilation span
  - id: knhk.projection.pipeline.compile
    type: span
    span_kind: internal
    stability: experimental
    brief: "Compile ggen template to executable artifact"
    attributes:
      - ref: knhk.projection.compilation_time_ms
      - ref: knhk.projection.artifact_size_bytes
      - ref: knhk.projection.incremental

  # Hot-reload span
  - id: knhk.projection.hotreload
    type: span
    span_kind: internal
    stability: experimental
    brief: "Hot-reload compiled artifact into runtime"
    attributes:
      - ref: knhk.projection.reload_success
      - ref: knhk.projection.rollback_triggered
      - ref: knhk.operation.latency_ms
```

### 3.2 Schema Validation Workflow

```bash
# Step 1: Validate schema syntax
weaver registry check -r registry/

# Step 2: Run application with OTEL exporter
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
cargo run --release

# Step 3: Perform operations that emit telemetry
knhk ontology inspect
knhk ontology query "SELECT ?p WHERE { ?p a knhk:Pattern }"

# Step 4: Validate runtime telemetry matches schema
weaver registry live-check --registry registry/

# Expected output:
# ✅ All spans conform to schema
# ✅ All metrics conform to schema
# ✅ No undeclared telemetry detected
```

### 3.3 Schema Evolution Strategy

**When Ontology Changes Require New Telemetry**:

1. **Propose Ontology Change** (ΔΣ)
2. **Validator Checks**: Does this require new telemetry?
3. **If Yes**:
   - Update `registry/knhk-ontology.yaml` with new spans/metrics
   - Run `weaver registry check` to validate schema
   - Only then apply ontology change
4. **If No**: Proceed with change

**Prevents**:
- Ontology evolution breaking telemetry contracts
- Unobservable changes sneaking into production
- Schema drift (runtime telemetry diverging from declared schema)

---

## 4. Per-Phase Validation Requirements

### Phase 1: Meta-Ontology & Runtime

#### Unit Tests (Supporting Evidence)

```bash
# Snapshot Manager tests
cargo test -p knhk-ontology-runtime snapshot

# Test cases:
# - Snapshot creation from RDF graph
# - Snapshot persistence and loading
# - Snapshot query interface
# - Snapshot diff computation
# - Receipt generation and verification

# Overlay Engine tests
cargo test -p knhk-ontology-runtime overlay

# Test cases:
# - Overlay creation on snapshot
# - Add/remove/modify RDF triples
# - Overlay query (merged view)
# - Overlay commit (new snapshot)
# - Overlay discard (no changes)

# Meta-ontology tests
cargo test -p knhk-ontology-meta

# Test cases:
# - Constraint validation
# - Evolution rules enforcement
# - Conflict detection
```

#### Weaver Validation (Source of Truth)

```bash
# 1. Add ontology operations to schema
cat >> registry/knhk-ontology.yaml <<EOF
groups:
  - id: knhk.ontology.snapshot.create
    type: span
    ...
  - id: knhk.ontology.overlay.create
    type: span
    ...
  - id: knhk.ontology.query
    type: span
    ...
EOF

# 2. Validate schema syntax
weaver registry check -r registry/

# 3. Run test application that creates snapshots
cargo run --example snapshot_demo

# 4. Validate runtime telemetry
weaver registry live-check --registry registry/

# Expected:
# ✅ knhk.ontology.snapshot.create span emitted
# ✅ knhk.ontology.overlay.create span emitted
# ✅ knhk.ontology.query span emitted
# ✅ All attributes match schema
```

#### Performance Benchmarks

```bash
# Benchmark snapshot operations
cargo bench -p knhk-ontology-runtime

# Targets:
# - Snapshot creation: <10ms
# - Snapshot query: <1ms
# - Overlay operations: <5ms
# - Receipt generation: <2ms
```

#### Definition of Done (Phase 1)

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test -p knhk-ontology-meta -p knhk-ontology-runtime` passes
- [ ] **`weaver registry check -r registry/` passes** ✅ MANDATORY
- [ ] **`weaver registry live-check` passes** ✅ MANDATORY
- [ ] Snapshot creation <10ms (benchmark)
- [ ] Code coverage >80%
- [ ] Rustdoc documentation complete

---

### Phase 2: Change Engine

#### Unit Tests (Supporting Evidence)

```bash
# Proposer tests
cargo test -p knhk-change-engine proposer

# Test cases:
# - Proposal creation from pattern candidate
# - Priority assignment
# - Justification formatting

# Validator tests
cargo test -p knhk-change-engine validator

# Test cases:
# - Constraint satisfaction checking
# - Conflict detection
# - Consensus mechanism
# - Validation result formatting

# Queue tests
cargo test -p knhk-change-engine queue

# Test cases:
# - Enqueue proposal
# - Dequeue by priority
# - Queue full backpressure
# - Concurrent access
```

#### Integration Tests (Supporting Evidence)

```bash
# End-to-end change application
cargo test -p knhk-change-engine --test integration

# Test scenario:
# 1. Create pattern candidate (observation)
# 2. Proposer creates change proposal
# 3. Enqueue proposal
# 4. Validator validates proposal
# 5. Apply validated change to ontology
# 6. Verify snapshot updated

# Expected:
# ✅ Proposal created
# ✅ Validation passes
# ✅ New snapshot exists
# ✅ Ontology reflects change
```

#### Weaver Validation (Source of Truth)

```bash
# 1. Update schema with change operations
cat >> registry/knhk-ontology.yaml <<EOF
groups:
  - id: knhk.change.proposal.create
    type: span
    ...
  - id: knhk.change.proposal.validate
    type: span
    ...
  - id: metric.knhk.change.queue_depth
    type: metric
    ...
EOF

# 2. Validate schema
weaver registry check -r registry/

# 3. Run change engine with mock proposals
cargo run --example change_engine_demo

# 4. Validate telemetry
weaver registry live-check --registry registry/

# Expected:
# ✅ knhk.change.proposal.create span emitted
# ✅ knhk.change.proposal.validate span emitted
# ✅ knhk.change.queue_depth metric emitted
# ✅ Consensus score attribute present
```

#### Definition of Done (Phase 2)

- [ ] `cargo test -p knhk-change-engine` passes
- [ ] Integration test passes
- [ ] **`weaver registry live-check` validates change operations** ✅ MANDATORY
- [ ] Change validation <50ms (benchmark)
- [ ] Queue throughput >100 proposals/sec
- [ ] Conflict detection functional
- [ ] Consensus mechanism working

---

### Phase 3: Observation Plane

#### Unit Tests (Supporting Evidence)

```bash
# Pattern recognizer tests
cargo test -p knhk-observation pattern

# Test cases:
# - Van der Aalst pattern 1 (Sequence) detection
# - Van der Aalst pattern 2 (Parallel Split) detection
# - Pattern confidence scoring
# - Multi-span pattern matching

# Anomaly detector tests
cargo test -p knhk-observation anomaly

# Test cases:
# - Conformance violation detection
# - Bottleneck detection
# - Resource utilization tracking
```

#### Integration Tests (Supporting Evidence)

```bash
# Telemetry → Pattern → Proposal flow
cargo test -p knhk-observation --test pattern_detection

# Test scenario:
# 1. Inject OTLP spans (simulated telemetry)
# 2. Pattern miner detects patterns
# 3. Pattern candidates emitted
# 4. Change engine receives proposals

# Expected:
# ✅ Pattern detected
# ✅ Confidence score calculated
# ✅ Proposal created
```

#### Weaver Validation (Source of Truth)

```bash
# 1. Update schema with observation operations
cat >> registry/knhk-ontology.yaml <<EOF
groups:
  - id: knhk.observation.pattern.detect
    type: span
    ...
  - id: knhk.observation.anomaly.detect
    type: span
    ...
  - id: metric.knhk.observation.patterns_detected
    type: metric
    ...
EOF

# 2. Validate schema
weaver registry check -r registry/

# 3. Run observation with real telemetry
cargo run --example observation_demo

# 4. Validate telemetry
weaver registry live-check --registry registry/

# Expected:
# ✅ knhk.observation.pattern.detect span emitted
# ✅ Pattern ID attribute present
# ✅ Confidence score attribute present
# ✅ Patterns detected metric incremented
```

#### Performance Benchmarks

```bash
# Benchmark pattern detection throughput
cargo bench -p knhk-observation

# Targets:
# - Pattern detection: >10K spans/sec
# - Anomaly detection: >5K spans/sec
# - Memory usage: <100MB for 1M spans
```

#### Definition of Done (Phase 3)

- [ ] `cargo test -p knhk-observation` passes
- [ ] Pattern detection for Van der Aalst 1-10 validated
- [ ] **`weaver registry live-check` validates observation telemetry** ✅ MANDATORY
- [ ] Pattern detection throughput >10K spans/sec
- [ ] Integration with change engine working
- [ ] Anomaly detection functional

---

### Phase 4: Projection Pipeline

#### Unit Tests (Supporting Evidence)

```bash
# Model projector tests
cargo test -p knhk-projection model

# Test cases:
# - Snapshot → ggen template projection
# - Incremental projection (delta-based)
# - Template validation

# Compiler tests
cargo test -p knhk-projection compiler

# Test cases:
# - ggen → Rust code compilation
# - Incremental compilation
# - Artifact validation

# Hot-reloader tests
cargo test -p knhk-projection hotreload

# Test cases:
# - Library loading
# - Symbol resolution
# - Rollback on failure
```

#### Integration Tests (Supporting Evidence)

```bash
# End-to-end projection flow
cargo test -p knhk-projection --test hot_reload

# Test scenario:
# 1. Create ontology snapshot
# 2. Project to ggen template
# 3. Compile template to .so
# 4. Hot-reload into runtime
# 5. Verify new behavior active
# 6. Rollback and verify old behavior

# Expected:
# ✅ Projection successful
# ✅ Compilation successful
# ✅ Hot-reload successful
# ✅ Rollback successful
```

#### Weaver Validation (Source of Truth)

```bash
# 1. Update schema with projection operations
cat >> registry/knhk-ontology.yaml <<EOF
groups:
  - id: knhk.projection.model.project
    type: span
    ...
  - id: knhk.projection.pipeline.compile
    type: span
    ...
  - id: knhk.projection.hotreload
    type: span
    ...
EOF

# 2. Validate schema
weaver registry check -r registry/

# 3. Run projection pipeline
cargo run --example projection_demo

# 4. Validate telemetry
weaver registry live-check --registry registry/

# Expected:
# ✅ knhk.projection.model.project span emitted
# ✅ knhk.projection.pipeline.compile span emitted
# ✅ knhk.projection.hotreload span emitted
# ✅ Compilation time attribute present
# ✅ Rollback triggered attribute (if rollback)
```

#### Hot Path Validation (CRITICAL)

```bash
# CRITICAL: Ensure hot path performance unaffected
make test-performance-v04

# Expected:
# ✅ Hot path latency ≤8 ticks (Chatman Constant)
# ✅ No regression from baseline

# If fails: Hot-reload has introduced overhead
# Action: Refactor hot-reload mechanism
```

#### Definition of Done (Phase 4)

- [ ] `cargo test -p knhk-projection` passes
- [ ] Hot-reload integration test passes
- [ ] **`weaver registry live-check` validates projection telemetry** ✅ MANDATORY
- [ ] **`make test-performance-v04` passes** ✅ CRITICAL (hot path ≤8 ticks)
- [ ] Compilation latency <1s (incremental)
- [ ] Rollback mechanism validated
- [ ] knhk-warm integration working

---

### Phase 5: Closed-Loop Control

#### Integration Tests (Supporting Evidence)

```bash
# Full closed-loop test
cargo test -p knhk-ontology-cli --test closed_loop

# Test scenario:
# 1. Emit telemetry (simulated workflow execution)
# 2. Observation detects pattern
# 3. Change engine proposes ontology update
# 4. Validator approves change
# 5. Ontology updated
# 6. Projection compiles new template
# 7. Hot-reload applies new behavior
# 8. Verify new pattern recognized in subsequent telemetry

# Expected:
# ✅ Pattern detected
# ✅ Proposal created
# ✅ Validation passed
# ✅ Ontology updated
# ✅ Compilation successful
# ✅ Hot-reload successful
# ✅ Closed-loop complete
```

#### Rollback Validation

```bash
# Automatic rollback test
cargo test -p knhk-ontology-cli --test rollback

# Test scenario:
# 1. Apply ontology change
# 2. Projection compiles new template
# 3. Hot-reload applies change
# 4. Inject telemetry showing performance degradation (latency >10ms)
# 5. Monitor detects degradation
# 6. Automatic rollback triggered
# 7. Verify old behavior restored

# Expected:
# ✅ Performance degradation detected
# ✅ Rollback triggered
# ✅ Old snapshot restored
# ✅ Performance restored
```

#### CLI Validation (NOT JUST --help!)

```bash
# ❌ WRONG: Only test --help
knhk ontology --help

# ✅ CORRECT: Actually execute commands
knhk ontology inspect
# Expected: Current snapshot displayed

knhk ontology query "SELECT ?p WHERE { ?p a knhk:Pattern }"
# Expected: Query results in JSON

knhk ontology snapshots
# Expected: List of snapshots with IDs

knhk ontology diff <snapshot1> <snapshot2>
# Expected: RDF diff between snapshots

knhk ontology export --format turtle > ontology.ttl
# Expected: Valid Turtle RDF file
```

#### Weaver Validation (Source of Truth - Final)

```bash
# 1. Ensure all plane operations in schema
weaver registry check -r registry/

# 2. Run full system with all planes active
cargo run --release

# 3. Trigger closed-loop cycle
# (emit telemetry → pattern detection → change → projection → reload)

# 4. Validate all telemetry
weaver registry live-check --registry registry/

# Expected:
# ✅ Observation telemetry valid
# ✅ Change engine telemetry valid
# ✅ Ontology runtime telemetry valid
# ✅ Projection telemetry valid
# ✅ End-to-end flow observable
# ✅ No undeclared telemetry
```

#### Definition of Done (Phase 5 - Final)

- [ ] All CLI commands functional (tested with REAL operations, not just --help)
- [ ] Closed-loop integration test passes
- [ ] Rollback integration test passes
- [ ] **`weaver registry live-check` validates ALL planes** ✅ MANDATORY
- [ ] **`make test-performance-v04` passes** ✅ CRITICAL
- [ ] `make validate-production-ready` passes
- [ ] Documentation complete (architecture, CLI reference, deployment)
- [ ] No `.unwrap()` or `.expect()` in production code
- [ ] Error handling comprehensive
- [ ] Logging uses `tracing` macros (not `println!`)

---

## 5. Performance Validation

### 5.1 Hot Path Constraint

**Chatman Constant: ≤8 ticks**

```bash
# Validate hot path unaffected by ontology system
make test-performance-v04

# Expected:
# ✅ JSON parsing: <8 ticks
# ✅ Span emission: <8 ticks
# ✅ No regression from baseline
```

**If Test Fails**:
- ❌ Ontology system has introduced hot path overhead
- **Action**: Refactor to eliminate hot path queries
- **Verify**: No FFI calls in hot path except init/shutdown

### 5.2 Warm Path Targets

| Operation | Target Latency | Benchmark Command |
|-----------|----------------|-------------------|
| Snapshot creation | <10ms | `cargo bench -p knhk-ontology-runtime snapshot_create` |
| Snapshot query | <1ms | `cargo bench -p knhk-ontology-runtime snapshot_query` |
| Overlay operations | <5ms | `cargo bench -p knhk-ontology-runtime overlay` |
| Change validation | <50ms | `cargo bench -p knhk-change-engine validate` |
| Pattern detection | >10K spans/sec | `cargo bench -p knhk-observation throughput` |
| Incremental compilation | <1s | `cargo bench -p knhk-projection incremental_compile` |
| Full compilation | <10s | `cargo bench -p knhk-projection full_compile` |
| Hot-reload | <100ms | `cargo bench -p knhk-projection hotreload` |

### 5.3 Scalability Benchmarks

```bash
# Ontology size scalability
cargo bench -p knhk-ontology-runtime --bench scale

# Test scenarios:
# - 1K triples: <1ms query
# - 10K triples: <5ms query
# - 100K triples: <50ms query
# - 1M triples: <500ms query

# Pattern detection throughput
cargo bench -p knhk-observation --bench throughput

# Test scenarios:
# - 1K spans/sec (single thread)
# - 10K spans/sec (4 threads)
# - 100K spans/sec (16 threads)
```

---

## 6. Integration Testing

### 6.1 Plane Interaction Tests

**O → Σ**: Pattern candidate stored in ontology
```bash
cargo test --test observation_to_ontology

# Scenario:
# 1. Observation detects pattern
# 2. Pattern candidate created
# 3. Query ontology to verify pattern stored

# Expected:
# ✅ Pattern in ontology
```

**Σ → ΔΣ**: Ontology change proposed
```bash
cargo test --test ontology_to_change

# Scenario:
# 1. Pattern detected
# 2. Change proposed
# 3. Validate proposal references correct ontology snapshot

# Expected:
# ✅ Proposal valid
```

**ΔΣ → Σ**: Change applied to ontology
```bash
cargo test --test change_to_ontology

# Scenario:
# 1. Proposal validated
# 2. Change applied
# 3. New snapshot created
# 4. Query ontology to verify change

# Expected:
# ✅ New snapshot exists
# ✅ Change reflected in ontology
```

**Σ → μ**: Ontology projected to template
```bash
cargo test --test ontology_to_projection

# Scenario:
# 1. Ontology snapshot exists
# 2. Project to ggen template
# 3. Validate template correctness

# Expected:
# ✅ Template generated
# ✅ Template valid
```

**μ → Π → Λ**: Template compiled and hot-reloaded
```bash
cargo test --test projection_to_runtime

# Scenario:
# 1. ggen template exists
# 2. Compile to .so
# 3. Hot-reload into runtime
# 4. Verify new behavior

# Expected:
# ✅ Compilation successful
# ✅ Hot-reload successful
# ✅ New behavior active
```

### 6.2 End-to-End Scenarios

**Scenario 1: New Pattern Discovery**
```bash
cargo test --test scenario_new_pattern

# Flow:
# Runtime → Telemetry → Observation → Pattern Detected → Change Proposed
#   → Validated → Ontology Updated → Projection → Hot-Reload → Runtime

# Validation:
# ✅ Pattern in ontology
# ✅ Weaver validates all telemetry
# ✅ Hot path latency ≤8 ticks
```

**Scenario 2: Conformance Violation**
```bash
cargo test --test scenario_conformance_violation

# Flow:
# Runtime → Telemetry → Observation → Anomaly Detected → Alert Logged

# Validation:
# ✅ Anomaly detected
# ✅ Alert logged
# ✅ No ontology change (anomaly doesn't change ontology)
```

**Scenario 3: Automatic Rollback**
```bash
cargo test --test scenario_rollback

# Flow:
# Change Applied → Projection → Hot-Reload → Performance Degradation
#   → Rollback Triggered → Previous Snapshot Restored

# Validation:
# ✅ Degradation detected
# ✅ Rollback successful
# ✅ Performance restored
```

---

## 7. Production Readiness Checklist

### 7.1 Compilation & Code Quality

- [ ] `cargo build --workspace --release` succeeds
- [ ] `cargo clippy --workspace -- -D warnings` passes (zero warnings)
- [ ] `cargo fmt --all -- --check` passes
- [ ] `make build` succeeds (C library)
- [ ] No `.unwrap()` in production code paths
- [ ] No `.expect()` in production code paths
- [ ] All traits `dyn` compatible (no async trait methods)
- [ ] Proper `Result<T, E>` error handling
- [ ] No `println!` in production code (use `tracing` macros)
- [ ] No fake `Ok(())` returns

### 7.2 Weaver Validation (MANDATORY)

- [ ] **`weaver registry check -r registry/` passes**
- [ ] **`weaver registry live-check --registry registry/` passes**
- [ ] All planes emit telemetry (O, Σ, ΔΣ, μ/Π/Λ)
- [ ] Schema documents exact telemetry behavior
- [ ] Live telemetry matches schema declarations
- [ ] No undeclared telemetry emitted

### 7.3 Functional Validation (MANDATORY)

- [ ] **All CLI commands executed with REAL arguments** (not just --help)
- [ ] **Commands produce expected output/behavior**
- [ ] **Commands emit proper telemetry** (validated by Weaver)
- [ ] **End-to-end workflow tested** (not just unit tests)
- [ ] **Performance constraints met** (≤8 ticks hot path)

### 7.4 Traditional Testing (Supporting Evidence)

- [ ] `cargo test --workspace` passes completely
- [ ] `make test-chicago-v04` passes
- [ ] `make test-performance-v04` passes (≤8 ticks)
- [ ] `make test-integration-v2` passes
- [ ] Tests follow AAA pattern (Arrange, Act, Assert)
- [ ] Descriptive test names

### 7.5 Performance & Scalability

- [ ] Hot path latency ≤8 ticks (Chatman Constant)
- [ ] Snapshot creation <10ms
- [ ] Snapshot query <1ms
- [ ] Change validation <50ms
- [ ] Pattern detection >10K spans/sec
- [ ] Incremental compilation <1s
- [ ] Hot-reload <100ms
- [ ] Memory usage reasonable (<1GB for typical workload)

### 7.6 Documentation

- [ ] Architecture document complete
- [ ] API documentation (rustdoc) generated
- [ ] CLI reference guide complete
- [ ] Deployment guide written
- [ ] Schema documentation (registry/)
- [ ] Integration guide for developers

### 7.7 Security & Reliability

- [ ] No SQL injection vectors (parameterized queries)
- [ ] No command injection vectors
- [ ] Input validation comprehensive
- [ ] Error messages don't leak sensitive info
- [ ] Rollback mechanism tested
- [ ] Graceful degradation on failures

---

## 8. CI/CD Validation Gates

### 8.1 Pre-Commit Checks (Local)

```bash
# Run before committing
make pre-commit

# Runs:
# 1. cargo fmt --all
# 2. cargo clippy --workspace -- -D warnings
# 3. cargo check --workspace
```

### 8.2 Pull Request Checks (CI)

```yaml
# .github/workflows/validate.yml

name: Validation

on: [pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Build
      - name: Build workspace
        run: cargo build --workspace --release

      # Code quality
      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

      # Unit tests
      - name: Unit tests
        run: cargo test --workspace --lib

      # Weaver validation (MANDATORY)
      - name: Weaver schema check
        run: weaver registry check -r registry/

      # Performance tests
      - name: Performance tests
        run: make test-performance-v04

      # Integration tests
      - name: Integration tests
        run: cargo test --workspace --test '*'
```

### 8.3 Release Validation (Pre-Deploy)

```bash
# Final validation before release
make validate-production-ready

# Runs:
# 1. cargo build --workspace --release
# 2. cargo clippy --workspace -- -D warnings
# 3. cargo test --workspace
# 4. make test-performance-v04
# 5. weaver registry check -r registry/
# 6. weaver registry live-check --registry registry/
# 7. Deployment smoke tests
```

---

## 9. Validation Anti-Patterns (What NOT to Do)

### ❌ Anti-Pattern 1: Trust --help Text

```bash
# ❌ WRONG
knhk ontology --help
# Output: Command exists
# Conclusion: "Feature works" ← FALSE!

# ✅ CORRECT
knhk ontology inspect
# Verify: Actual output produced, telemetry emitted, Weaver validates
```

### ❌ Anti-Pattern 2: Trust Tests Alone

```bash
# ❌ WRONG
cargo test --workspace
# All tests pass
# Conclusion: "Features work" ← FALSE!

# ✅ CORRECT
cargo test --workspace && weaver registry live-check
# Tests pass AND Weaver validates telemetry
```

### ❌ Anti-Pattern 3: Skip Weaver Validation

```bash
# ❌ WRONG
# "Tests pass, that's good enough"

# ✅ CORRECT
weaver registry check -r registry/
weaver registry live-check --registry registry/
# Schema validation is MANDATORY
```

### ❌ Anti-Pattern 4: Mock Everything

```bash
# ❌ WRONG
# Test with 100% mocks
# Never exercise real code paths

# ✅ CORRECT
# Integration tests with real dependencies
# Weaver validates actual runtime telemetry
```

---

## Appendix A: Validation Checklist Template

```markdown
## Phase X Validation

### Compilation & Code Quality
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] No warnings

### Unit Tests (Supporting Evidence)
- [ ] `cargo test -p <crate>` passes
- [ ] Code coverage >80%
- [ ] All critical paths tested

### Weaver Validation (MANDATORY)
- [ ] Schema updated in `registry/knhk-ontology.yaml`
- [ ] `weaver registry check -r registry/` passes
- [ ] `weaver registry live-check --registry registry/` passes
- [ ] All spans/metrics conform to schema

### Performance (If Applicable)
- [ ] Benchmarks meet targets
- [ ] No hot path regression
- [ ] Scalability validated

### Integration (If Applicable)
- [ ] Integration tests pass
- [ ] Plane interactions validated
- [ ] End-to-end flow working

### Documentation
- [ ] Rustdoc updated
- [ ] Architecture docs updated
- [ ] Examples working
```

---

**Document Status**: Final v1.0 - Ready for Use
**Next Steps**: Apply this strategy during implementation, validate rigorously
**Owners**: KNHK Core Team, QA Team, Autonomous Ontology Working Group
