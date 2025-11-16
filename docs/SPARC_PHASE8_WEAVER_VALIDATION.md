# SPARC Phase 8: Weaver Validation Integration

**Version**: 1.0.0
**Date**: 2025-11-16
**Status**: Design Complete - Source of Truth Implementation
**Authors**: Performance Benchmarker Agent

---

## Executive Summary

This document specifies the complete OpenTelemetry Weaver validation integration for KNHK - the **ONLY source of truth** for proving features work correctly. Weaver validation eliminates false positives by validating actual runtime telemetry against declared schemas, solving the meta-problem KNHK was designed to address.

**Critical Principle**: Tests can lie (false positives). Schemas don't. Only Weaver validation proves runtime behavior matches specification.

**Why Weaver Validation is Different**:
- **Schema-first**: Code must conform to declared telemetry schema
- **Live validation**: Verifies actual runtime telemetry against schema
- **No circular dependency**: External tool validates our framework
- **Industry standard**: OTel's official validation approach
- **Detects fake-green**: Catches tests that pass but don't validate actual behavior

---

## Table of Contents

1. [The False Positive Problem](#1-the-false-positive-problem)
2. [Weaver Validation Architecture](#2-weaver-validation-architecture)
3. [OTEL Schema Definitions](#3-otel-schema-definitions)
4. [Live-Check Validation Strategy](#4-live-check-validation-strategy)
5. [Telemetry Instrumentation Patterns](#5-telemetry-instrumentation-patterns)
6. [Performance Baseline Methodology](#6-performance-baseline-methodology)
7. [Regression Detection](#7-regression-detection)
8. [CI/CD Integration](#8-cicd-integration)
9. [Monitoring and Dashboards](#9-monitoring-and-dashboards)
10. [Schema Evolution Strategy](#10-schema-evolution-strategy)

---

## 1. The False Positive Problem

### 1.1 What KNHK Solves

**The Meta-Problem**:
```
Traditional Testing (What We Replace):
  assert(result == expected) ✅  ← Can pass even when feature is broken
  └─ Tests validate test logic, not production behavior

KNHK with Weaver Validation:
  Schema defines behavior → Weaver validates runtime telemetry ✅
  └─ Schema validation proves actual runtime behavior matches specification
```

### 1.2 False Positive Scenarios

**Scenario 1: Help Text ≠ Working Feature**
```bash
# ❌ FALSE POSITIVE VALIDATION
$ knhk --help
# Returns: "usage: knhk [command] [options]..."
# ❌ CONCLUSION: "command works" ← WRONG!
# ✅ REALITY: Help text exists, but command may call unimplemented!()

# ✅ CORRECT VALIDATION
$ knhk observe --file input.json
# Check: Does it produce expected output/behavior?
# Check: Does it emit proper telemetry spans?
# Check: Does Weaver validation pass?
```

**Scenario 2: Tests Pass But Feature Broken**
```rust
#[test]
fn test_observation_append() {
    let store = ObservationStore::new();
    let obs = create_observation();
    let id = store.append(obs);  // ✅ Test passes

    // But what if append() doesn't actually persist?
    // What if it calls unimplemented!() internally?
    // Traditional tests can't catch this!
}

// ✅ Weaver catches this:
// Schema declares: span "observation.appended" must be emitted
// Runtime: No span emitted → Weaver validation FAILS
// Conclusion: Feature doesn't work, despite test passing
```

### 1.3 The Validation Hierarchy

**🔴 CRITICAL: Validation hierarchy matters!**

```
LEVEL 1: Weaver Schema Validation (MANDATORY - Source of Truth)
  ├─ weaver registry check -r registry/           # Schema definition valid
  └─ weaver registry live-check --registry registry/ # Runtime telemetry matches schema

LEVEL 2: Compilation & Code Quality (Baseline)
  ├─ cargo build --release                        # Must compile
  ├─ cargo clippy --workspace -- -D warnings      # Zero warnings
  └─ make build                                   # C library compiles

LEVEL 3: Traditional Tests (Supporting Evidence - Can Have False Positives)
  ├─ cargo test --workspace                       # Rust unit tests
  ├─ make test-chicago-v04                        # C Chicago TDD tests
  ├─ make test-performance-v04                    # Performance tests
  └─ make test-integration-v2                     # Integration tests
```

**⚠️ If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.**

---

## 2. Weaver Validation Architecture

### 2.1 Schema-First Development Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: Define Telemetry Schema FIRST                           │
│                                                                  │
│  registry/otel/spans.yaml                                       │
│  ├─ monitor.observe_event (hot path <100ns)                    │
│  ├─ analyze.detect_patterns (warm path <100ms)                 │
│  ├─ plan.validate_proposal (warm path <100ms)                  │
│  ├─ execute.promote_snapshot (hot path <1ns)                   │
│  └─ knowledge.update_budget (warm path <1ms)                   │
│                                                                  │
│  registry/otel/metrics.yaml                                     │
│  ├─ observation.latency_ns (histogram, <100ns target)          │
│  ├─ snapshot.promotion_ns (histogram, <1ns target)             │
│  ├─ proposal.validation_ms (histogram, <100ms target)          │
│  └─ proposal.acceptance_rate (counter, learning metric)        │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: Validate Schema Definition                              │
│                                                                  │
│  $ weaver registry check -r registry/                           │
│  ✅ All spans have required attributes                          │
│  ✅ All metrics have valid units                                │
│  ✅ No conflicting definitions                                  │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: Implement Code to Emit Telemetry                        │
│                                                                  │
│  rust/knhk-closed-loop/src/observation.rs                       │
│  ```rust                                                         │
│  pub fn append(&self, obs: Observation) -> String {             │
│      let span = span!(                                          │
│          target: "knhk",                                        │
│          Level::TRACE,                                          │
│          "monitor.observe_event",                               │
│          latency_ns = tracing::field::Empty,                    │
│          observation.id = %obs.id,                              │
│          observation.sector = %obs.sector,                      │
│      );                                                          │
│      let _enter = span.enter();                                 │
│                                                                  │
│      let start = Instant::now();                                │
│      // ... actual implementation ...                           │
│      let latency = start.elapsed().as_nanos();                  │
│                                                                  │
│      span.record("latency_ns", latency as u64);                 │
│      obs_id                                                      │
│  }                                                               │
│  ```                                                             │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: Live Validation (Runtime Telemetry → Schema)            │
│                                                                  │
│  # Run system, capture telemetry                                │
│  $ cargo run --release                                          │
│                                                                  │
│  # Validate runtime telemetry matches schema                    │
│  $ weaver registry live-check --registry registry/              │
│                                                                  │
│  ✅ Span "monitor.observe_event" emitted                        │
│  ✅ Attribute "latency_ns" present and type u64                 │
│  ✅ Attribute "observation.id" present and type string          │
│  ✅ All declared spans found in actual telemetry                │
│                                                                  │
│  ❌ If any span missing → FEATURE DOES NOT WORK                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Continuous Validation Loop

```
┌────────────────────────────────────────────────────────────────┐
│ DEVELOPMENT WORKFLOW                                            │
│                                                                 │
│ 1. Write schema (spans.yaml, metrics.yaml)                     │
│ 2. Run weaver registry check                                   │
│ 3. Implement feature (emit spans/metrics)                      │
│ 4. Run weaver registry live-check                              │
│ 5. If validation fails → Fix implementation                    │
│ 6. If validation passes → Feature works!                       │
└────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────────────┐
│ CI/CD VALIDATION                                                │
│                                                                 │
│ .github/workflows/weaver-validation.yml:                       │
│   - name: Validate OTEL Schema                                 │
│     run: weaver registry check -r registry/                    │
│                                                                 │
│   - name: Build and Run Tests                                  │
│     run: cargo test --workspace                                │
│                                                                 │
│   - name: Live Telemetry Validation (SOURCE OF TRUTH)          │
│     run: |                                                      │
│       cargo run --release &                                    │
│       sleep 10                                                  │
│       weaver registry live-check --registry registry/          │
│                                                                 │
│   - name: Block merge if Weaver fails                          │
│     if: failure()                                              │
│     run: exit 1                                                │
└────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────────────┐
│ PRODUCTION MONITORING                                           │
│                                                                 │
│ Daily:                                                          │
│   - Run Weaver live-check on last 24h telemetry               │
│   - Alert if validation fails (schema drift)                   │
│                                                                 │
│ Weekly:                                                         │
│   - Analyze trends (acceptance rate, latency distribution)     │
│   - Detect regressions (p99 > baseline)                        │
│                                                                 │
│ Monthly:                                                        │
│   - Update schema based on new observations                    │
│   - Evolve baseline performance targets                        │
└────────────────────────────────────────────────────────────────┘
```

---

## 3. OTEL Schema Definitions

### 3.1 Spans Schema (registry/otel/spans.yaml)

**Monitor Phase Spans** (Hot Path: ≤8 ticks = <100ns):

```yaml
# registry/otel/spans.yaml
groups:
  - id: knhk.monitor
    type: span
    brief: "Monitor phase operations (MAPE-K)"
    spans:
      - id: monitor.observe_event
        span_kind: internal
        brief: "Append observation to store (hot path <100ns)"
        attributes:
          - ref: observation.id
            requirement_level: required
          - ref: observation.sector
            requirement_level: required
          - ref: observation.event_type
            requirement_level: required
          - ref: latency_ns
            requirement_level: required
            brief: "Actual append latency in nanoseconds"
            type: int
          - ref: observation.timestamp
            requirement_level: required
            type: int
        events:
          - name: observation.appended
            brief: "Observation successfully appended"
        performance:
          target_latency_ns: 100
          p99_latency_ns: 150
          max_latency_ns: 200
```

**Analyze Phase Spans** (Warm Path: <100ms):

```yaml
  - id: knhk.analyze
    type: span
    brief: "Analyze phase operations (MAPE-K)"
    spans:
      - id: analyze.detect_patterns
        span_kind: internal
        brief: "Detect patterns in observation stream (warm path <100ms)"
        attributes:
          - ref: pattern.count
            requirement_level: required
            type: int
          - ref: pattern.confidence
            requirement_level: required
            type: double
          - ref: analysis.window_ms
            requirement_level: required
            type: int
          - ref: latency_ms
            requirement_level: required
            type: int
        performance:
          target_latency_ms: 50
          p99_latency_ms: 100
          max_latency_ms: 150

      - id: analyze.validate_doctrines
        span_kind: internal
        brief: "Validate proposal against sector doctrines"
        attributes:
          - ref: proposal.id
            requirement_level: required
          - ref: doctrine.count
            requirement_level: required
            type: int
          - ref: violation.count
            requirement_level: required
            type: int
          - ref: validation.passed
            requirement_level: required
            type: boolean
```

**Plan Phase Spans** (Warm Path: <100ms):

```yaml
  - id: knhk.plan
    type: span
    brief: "Plan phase operations (MAPE-K)"
    spans:
      - id: plan.validate_proposal
        span_kind: internal
        brief: "7-stage validation pipeline (warm path <100ms)"
        attributes:
          - ref: proposal.id
            requirement_level: required
          - ref: validation.stage
            requirement_level: required
            brief: "Current validation stage (1-7)"
            type: int
          - ref: validation.passed
            requirement_level: required
            type: boolean
          - ref: invariant.q1_preserved
            requirement_level: required
            type: boolean
          - ref: invariant.q2_preserved
            requirement_level: required
            type: boolean
          - ref: invariant.q3_preserved
            requirement_level: required
            type: boolean
          - ref: invariant.q4_preserved
            requirement_level: required
            type: boolean
          - ref: invariant.q5_preserved
            requirement_level: required
            type: boolean
        events:
          - name: validation.stage_completed
            brief: "Validation stage completed"
            attributes:
              - ref: validation.stage
              - ref: validation.passed
          - name: validation.failed
            brief: "Validation failed at stage"
            attributes:
              - ref: validation.stage
              - ref: failure.reason
```

**Execute Phase Spans** (Hot Path: ~1ns atomic swap):

```yaml
  - id: knhk.execute
    type: span
    brief: "Execute phase operations (MAPE-K)"
    spans:
      - id: execute.promote_snapshot
        span_kind: internal
        brief: "Atomic snapshot promotion (hot path <1ns for swap)"
        attributes:
          - ref: snapshot.id
            requirement_level: required
          - ref: snapshot.parent_id
            requirement_level: optional
          - ref: snapshot.version
            requirement_level: required
            type: int
          - ref: promotion.latency_ns
            requirement_level: required
            type: int
            brief: "Actual atomic swap latency"
          - ref: promotion.success
            requirement_level: required
            type: boolean
        events:
          - name: snapshot.promoted
            brief: "Snapshot successfully promoted"
          - name: snapshot.rollback
            brief: "Snapshot promotion rolled back"
        performance:
          target_latency_ns: 1
          p99_latency_ns: 10
          max_latency_ns: 100
```

**Knowledge Phase Spans** (Warm Path: <1ms):

```yaml
  - id: knhk.knowledge
    type: span
    brief: "Knowledge phase operations (MAPE-K)"
    spans:
      - id: knowledge.update_budget
        span_kind: internal
        brief: "Update performance budgets based on outcomes"
        attributes:
          - ref: cycle.id
            requirement_level: required
          - ref: budget.ticks_before
            requirement_level: required
            type: int
          - ref: budget.ticks_after
            requirement_level: required
            type: int
          - ref: learning.lesson_count
            requirement_level: required
            type: int

      - id: knowledge.shadow_test
        span_kind: internal
        brief: "Run shadow tests in isolated environment"
        attributes:
          - ref: snapshot.id
            requirement_level: required
          - ref: test.count
            requirement_level: required
            type: int
          - ref: test.passed
            requirement_level: required
            type: int
          - ref: test.failed
            requirement_level: required
            type: int
```

### 3.2 Metrics Schema (registry/otel/metrics.yaml)

**Hot Path Metrics** (≤8 ticks = <100ns):

```yaml
# registry/otel/metrics.yaml
groups:
  - id: knhk.metrics.hotpath
    type: metric
    brief: "Hot path performance metrics (≤8 ticks)"
    metric_name: knhk.observation.latency
    attributes:
      - ref: observation.sector
      - ref: observation.event_type
    metrics:
      - id: observation.append.latency_ns
        type: histogram
        brief: "Observation append latency in nanoseconds"
        instrument: histogram
        unit: ns
        stability: stable
        attributes:
          - ref: observation.sector
        note: "Target: <100ns (≤8 ticks), P99: <150ns"

      - id: snapshot.promotion.latency_ns
        type: histogram
        brief: "Snapshot promotion latency (atomic swap)"
        instrument: histogram
        unit: ns
        stability: stable
        note: "Target: <1ns (atomic operation), P99: <10ns"
```

**Warm Path Metrics** (<100ms):

```yaml
  - id: knhk.metrics.warmpath
    type: metric
    brief: "Warm path performance metrics (<100ms)"
    metrics:
      - id: pattern.detection.latency_ms
        type: histogram
        brief: "Pattern detection latency in milliseconds"
        instrument: histogram
        unit: ms
        stability: stable
        attributes:
          - ref: pattern.type
          - ref: analysis.window_ms
        note: "Target: <50ms, P99: <100ms"

      - id: proposal.validation.latency_ms
        type: histogram
        brief: "Proposal validation latency (7 stages)"
        instrument: histogram
        unit: ms
        stability: stable
        attributes:
          - ref: validation.stage
          - ref: sector
        note: "Target: <100ms total, P99: <150ms"

      - id: doctrine.validation.latency_ms
        type: histogram
        brief: "Doctrine validation latency"
        instrument: histogram
        unit: ms
        stability: stable
        note: "Target: <30ms, P99: <50ms"
```

**Learning Metrics** (Acceptance Rate, Trends):

```yaml
  - id: knhk.metrics.learning
    type: metric
    brief: "Learning and adaptation metrics"
    metrics:
      - id: proposal.acceptance_rate
        type: counter
        brief: "Ratio of accepted to total proposals"
        instrument: counter
        unit: "{proposal}"
        stability: stable
        attributes:
          - ref: sector
          - ref: pattern.type
          - ref: acceptance.status
            members:
              - id: accepted
                value: "accepted"
              - id: rejected
                value: "rejected"
        note: "Track learning effectiveness over time"

      - id: budget.adjustment.count
        type: counter
        brief: "Number of times budgets were adjusted"
        instrument: counter
        unit: "{adjustment}"
        attributes:
          - ref: adjustment.direction
            members:
              - id: increased
                value: "increased"
              - id: decreased
                value: "decreased"
```

**Error Metrics**:

```yaml
  - id: knhk.metrics.errors
    type: metric
    brief: "Error tracking metrics"
    metrics:
      - id: error.total
        type: counter
        brief: "Total error count by type"
        instrument: counter
        unit: "{error}"
        attributes:
          - ref: error.type
            members:
              - id: invariant_violation
                value: "invariant_violation"
              - id: doctrine_violation
                value: "doctrine_violation"
              - id: guard_violation
                value: "guard_violation"
              - id: performance_violation
                value: "performance_violation"
          - ref: error.severity
            members:
              - id: critical
              - id: warning
              - id: info

      - id: invariant.violation.count
        type: counter
        brief: "Hard invariant violations (SHOULD BE ZERO)"
        instrument: counter
        unit: "{violation}"
        attributes:
          - ref: invariant.id
            members:
              - id: q1
                value: "Q1"
                brief: "No retrocausation"
              - id: q2
                value: "Q2"
                brief: "Type soundness"
              - id: q3
                value: "Q3"
                brief: "Guard preservation"
              - id: q4
                value: "Q4"
                brief: "SLO compliance"
              - id: q5
                value: "Q5"
                brief: "Performance bounds"
        note: "Alert if > 0"
```

### 3.3 Logs Schema (registry/otel/logs.yaml)

```yaml
# registry/otel/logs.yaml
groups:
  - id: knhk.logs
    type: log_record
    brief: "Structured logging for state transitions and decisions"
    log_records:
      - id: proposal.generated
        severity: info
        brief: "Proposal generated from detected pattern"
        attributes:
          - ref: proposal.id
            requirement_level: required
          - ref: pattern.name
            requirement_level: required
          - ref: proposal.confidence
            requirement_level: required
          - ref: proposal.description
            requirement_level: required

      - id: proposal.validated
        severity: info
        brief: "Proposal validation completed"
        attributes:
          - ref: proposal.id
            requirement_level: required
          - ref: validation.passed
            requirement_level: required
          - ref: validation.stages_completed
            requirement_level: required
          - ref: validation.failure_reason
            requirement_level:
              conditionally_required: "if validation.passed == false"

      - id: proposal.accepted
        severity: info
        brief: "Proposal accepted and promoted"
        attributes:
          - ref: proposal.id
            requirement_level: required
          - ref: snapshot.id
            requirement_level: required
          - ref: decision.reasoning
            requirement_level: required

      - id: proposal.rejected
        severity: info
        brief: "Proposal rejected"
        attributes:
          - ref: proposal.id
            requirement_level: required
          - ref: rejection.reason
            requirement_level: required
          - ref: rejection.stage
            requirement_level: required

      - id: guard.enforced
        severity: debug
        brief: "Guard blocked an operation"
        attributes:
          - ref: guard.id
            requirement_level: required
          - ref: guard.name
            requirement_level: required
          - ref: operation.blocked
            requirement_level: required

      - id: snapshot.promoted
        severity: info
        brief: "Snapshot promoted to production"
        attributes:
          - ref: snapshot.id
            requirement_level: required
          - ref: snapshot.version
            requirement_level: required
          - ref: snapshot.parent_id
            requirement_level: optional
          - ref: promotion.timestamp
            requirement_level: required
```

### 3.4 Shared Attributes (registry/otel/attributes.yaml)

```yaml
# registry/otel/attributes.yaml
groups:
  - id: knhk.attributes
    type: attribute_group
    brief: "Shared attributes used across spans, metrics, and logs"
    attributes:
      # Observation attributes
      - id: observation.id
        type: string
        brief: "Unique observation identifier"
        examples: ["finance-transaction-abc123"]

      - id: observation.sector
        type: string
        brief: "Organizational sector"
        examples: ["finance", "healthcare", "manufacturing", "logistics"]

      - id: observation.event_type
        type: string
        brief: "Type of observed event"
        examples: ["transaction.execute", "diagnosis.created"]

      - id: observation.timestamp
        type: int
        brief: "Unix timestamp in milliseconds"

      # Proposal attributes
      - id: proposal.id
        type: string
        brief: "Unique proposal identifier"

      - id: proposal.confidence
        type: double
        brief: "Confidence score (0.0-1.0)"

      # Snapshot attributes
      - id: snapshot.id
        type: string
        brief: "Snapshot identifier (SHA-256)"

      - id: snapshot.parent_id
        type: string
        brief: "Parent snapshot identifier"

      - id: snapshot.version
        type: int
        brief: "Monotonic version number"

      # Validation attributes
      - id: validation.stage
        type: int
        brief: "Validation pipeline stage (1-7)"

      - id: validation.passed
        type: boolean
        brief: "Validation result"

      # Performance attributes
      - id: latency_ns
        type: int
        brief: "Operation latency in nanoseconds"
        unit: ns

      - id: latency_ms
        type: int
        brief: "Operation latency in milliseconds"
        unit: ms

      # Guard attributes
      - id: guard.id
        type: string
        brief: "Guard identifier"

      # Doctrine attributes
      - id: doctrine.count
        type: int
        brief: "Number of doctrines checked"

      # Pattern attributes
      - id: pattern.type
        type: string
        brief: "Detected pattern type"
        examples: ["frequency_anomaly", "error_spike", "schema_mismatch"]

      # Sector
      - id: sector
        type: string
        brief: "Organizational sector"
        examples: ["finance", "healthcare", "manufacturing", "logistics"]
```

---

## 4. Live-Check Validation Strategy

### 4.1 Continuous Validation Pipeline

```
┌──────────────────────────────────────────────────────────────┐
│ VALIDATION PIPELINE                                           │
│                                                               │
│ 1. Schema Definition Validation (Static)                     │
│    $ weaver registry check -r registry/                      │
│    ✅ YAML syntax valid                                      │
│    ✅ All references resolved                                │
│    ✅ No conflicting definitions                             │
│    ✅ Required attributes present                            │
│                                                               │
│ 2. Runtime Telemetry Collection                              │
│    - Run KNHK system                                         │
│    - Emit OTLP spans/metrics/logs                            │
│    - Export to OTEL Collector                                │
│    - Store in telemetry backend                              │
│                                                               │
│ 3. Live Telemetry Validation (SOURCE OF TRUTH)               │
│    $ weaver registry live-check --registry registry/ \       │
│      --telemetry-source http://localhost:4317                │
│                                                               │
│    For each declared span in schema:                         │
│      ✅ Span emitted at runtime                              │
│      ✅ All required attributes present                      │
│      ✅ Attribute types match schema                         │
│      ✅ Performance targets met                              │
│                                                               │
│    ❌ If any span missing → FEATURE DOES NOT WORK            │
│                                                               │
│ 4. Performance Baseline Comparison                           │
│    - Compare actual latency vs. target                       │
│    - Check: p50, p95, p99 within budget                      │
│    - Alert if regression detected                            │
│                                                               │
│ 5. Acceptance Rate Monitoring                                │
│    - Track proposal.acceptance_rate over time                │
│    - Detect learning regressions                             │
│    - Adjust thresholds if needed                             │
└──────────────────────────────────────────────────────────────┘
```

### 4.2 Validation Modes

**Mode 1: Development Validation** (Fast Feedback):
```bash
# Run during development
$ cargo run --release &
$ sleep 5  # Let system start

# Validate last 60 seconds of telemetry
$ weaver registry live-check \
  --registry registry/ \
  --window 60s \
  --fail-fast

# Exit code 0 = PASS (feature works)
# Exit code 1 = FAIL (feature broken, despite tests passing)
```

**Mode 2: CI/CD Validation** (Automated):
```yaml
# .github/workflows/weaver-validation.yml
name: Weaver Validation

on: [push, pull_request]

jobs:
  validate-telemetry:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Weaver
        run: |
          curl -L https://github.com/open-telemetry/weaver/releases/latest/download/weaver-linux-amd64 -o /usr/local/bin/weaver
          chmod +x /usr/local/bin/weaver

      - name: Validate Schema Definition
        run: weaver registry check -r registry/

      - name: Build Project
        run: cargo build --release

      - name: Start OTEL Collector
        run: docker-compose up -d otel-collector

      - name: Run System (Emit Telemetry)
        run: |
          cargo run --release &
          KNHK_PID=$!
          sleep 30  # Let system run
          kill $KNHK_PID

      - name: Live Telemetry Validation (SOURCE OF TRUTH)
        run: |
          weaver registry live-check \
            --registry registry/ \
            --telemetry-source http://localhost:4317 \
            --output validation-report.json

      - name: Check Validation Result
        run: |
          if [ $? -ne 0 ]; then
            echo "❌ Weaver validation FAILED - Feature does not work"
            exit 1
          fi
          echo "✅ Weaver validation PASSED - Feature works correctly"

      - name: Upload Validation Report
        uses: actions/upload-artifact@v3
        with:
          name: weaver-validation-report
          path: validation-report.json
```

**Mode 3: Production Monitoring** (Continuous):
```bash
# Cron job: Run daily at 2 AM
0 2 * * * /usr/local/bin/validate-production-telemetry.sh

# validate-production-telemetry.sh
#!/bin/bash
set -e

# Fetch last 24 hours of telemetry from production
weaver registry live-check \
  --registry /opt/knhk/registry/ \
  --telemetry-source https://otel-collector.prod:4317 \
  --window 24h \
  --output /var/log/knhk/weaver-$(date +%Y%m%d).json

# Alert if validation fails
if [ $? -ne 0 ]; then
  echo "🚨 CRITICAL: Production telemetry validation FAILED"
  curl -X POST https://alerts.company.com/webhook \
    -d '{"alert":"schema_validation_failed","severity":"critical"}'
  exit 1
fi

echo "✅ Production telemetry validation PASSED"
```

### 4.3 Regression Detection

**Baseline Establishment**:
```bash
# Step 1: Run baseline workload
$ cargo run --release -- benchmark \
  --duration 60s \
  --workload standard \
  --output baseline-metrics.json

# Step 2: Capture telemetry baseline
$ weaver registry live-check \
  --registry registry/ \
  --capture-baseline baseline-telemetry.json

# Step 3: Store baseline for comparison
$ cp baseline-telemetry.json registry/baseline.json
```

**Regression Detection**:
```bash
# Compare current telemetry against baseline
$ weaver registry live-check \
  --registry registry/ \
  --baseline registry/baseline.json \
  --tolerance 10%  # Allow 10% deviation

# Example output:
# ✅ observation.append.latency_ns: p99=85ns (baseline=80ns, +6.25%)
# ⚠️  proposal.validation.latency_ms: p99=120ms (baseline=95ms, +26.3%)
# ❌ REGRESSION DETECTED: validation latency exceeds baseline by 26.3%
```

---

## 5. Telemetry Instrumentation Patterns

### 5.1 Hot Path Instrumentation (<100ns)

**Pattern: Branchless Span Emission**

```rust
// rust/knhk-closed-loop/src/observation.rs

use tracing::{span, Level};
use std::time::Instant;

pub struct ObservationStore {
    observations: DashMap<String, Arc<Observation>>,
}

impl ObservationStore {
    pub fn append(&self, obs: Observation) -> String {
        // CRITICAL: Span creation must be hot path compliant (≤8 ticks)
        let span = span!(
            target: "knhk",
            Level::TRACE,  // TRACE for hot path (minimal overhead)
            "monitor.observe_event",

            // Pre-allocate fields (avoid dynamic allocation)
            latency_ns = tracing::field::Empty,
            "observation.id" = %obs.id,
            "observation.sector" = %obs.sector,
            "observation.event_type" = %obs.event_type,
            "observation.timestamp" = obs.timestamp,
        );

        let _enter = span.enter();

        // Measure actual latency
        let start = Instant::now();

        // Hot path operation (≤8 ticks)
        let obs_id = obs.id.clone();
        self.observations.insert(obs_id.clone(), Arc::new(obs));

        let latency_ns = start.elapsed().as_nanos() as u64;

        // Record latency (cheap field update)
        span.record("latency_ns", latency_ns);

        // Emit metric (async, non-blocking)
        metrics::histogram!("knhk.observation.latency_ns",
            latency_ns as f64,
            "sector" => obs.sector.clone(),
        );

        obs_id
    }
}
```

**Performance Validation**:
```rust
#[test]
fn test_hot_path_latency_under_budget() {
    let store = ObservationStore::new();
    let obs = create_observation();

    // Warm up (JIT, cache)
    for _ in 0..1000 {
        let _ = store.append(create_observation());
    }

    // Measure actual hot path latency
    let mut latencies = Vec::new();
    for _ in 0..10000 {
        let start = Instant::now();
        let _ = store.append(create_observation());
        latencies.push(start.elapsed().as_nanos());
    }

    // Calculate percentiles
    latencies.sort_unstable();
    let p50 = latencies[latencies.len() / 2];
    let p99 = latencies[latencies.len() * 99 / 100];

    // Validate against schema targets
    assert!(p50 < 100, "P50 latency {} ns exceeds 100ns", p50);
    assert!(p99 < 150, "P99 latency {} ns exceeds 150ns", p99);

    // ✅ If this passes AND Weaver validation passes → Feature works
    // ❌ If this passes BUT Weaver validation fails → Test is a false positive
}
```

### 5.2 Warm Path Instrumentation (<100ms)

**Pattern: Multi-Stage Validation Spans**

```rust
// rust/knhk-closed-loop/src/validation.rs

pub struct ValidationPipeline {
    // ... fields ...
}

impl ValidationPipeline {
    pub async fn validate(&self, proposal: &Proposal) -> Result<ValidationReport> {
        // Parent span for entire validation pipeline
        let pipeline_span = span!(
            Level::INFO,
            "plan.validate_proposal",
            "proposal.id" = %proposal.id,
            "validation.stage" = tracing::field::Empty,
            "validation.passed" = tracing::field::Empty,
            "invariant.q1_preserved" = tracing::field::Empty,
            "invariant.q2_preserved" = tracing::field::Empty,
            "invariant.q3_preserved" = tracing::field::Empty,
            "invariant.q4_preserved" = tracing::field::Empty,
            "invariant.q5_preserved" = tracing::field::Empty,
        );

        let _enter = pipeline_span.enter();

        let mut report = ValidationReport::new(proposal.id.clone());

        // Stage 1: Static SHACL validation
        {
            let stage_span = span!(
                parent: &pipeline_span,
                Level::DEBUG,
                "validation.stage.static",
                "validation.stage" = 1,
            );
            let _stage_enter = stage_span.enter();

            let stage_result = self.validate_static(proposal)?;
            report.add_stage(stage_result);

            stage_span.record("validation.passed", stage_result.passed);

            if !stage_result.passed {
                pipeline_span.record("validation.passed", false);
                return Ok(report);
            }
        }

        // Stage 2: Invariant Q1-Q5 checks
        for (i, invariant) in ["Q1", "Q2", "Q3", "Q4", "Q5"].iter().enumerate() {
            let stage_span = span!(
                parent: &pipeline_span,
                Level::DEBUG,
                "validation.stage.invariant",
                "validation.stage" = i + 2,
                "invariant.id" = invariant,
            );
            let _stage_enter = stage_span.enter();

            let stage_result = self.check_invariant(invariant, proposal)?;
            report.add_stage(stage_result);

            // Record in parent span
            let field_name = format!("invariant.{}_preserved", invariant.to_lowercase());
            pipeline_span.record(&field_name, stage_result.passed);

            if !stage_result.passed {
                pipeline_span.record("validation.passed", false);

                // Emit log for invariant violation
                tracing::error!(
                    "Invariant {} violated",
                    invariant,
                    "proposal.id" = %proposal.id,
                    "violation.reason" = %stage_result.failure_reason.unwrap_or_default(),
                );

                // Increment error metric
                metrics::counter!("knhk.invariant.violation.count", 1,
                    "invariant.id" => invariant.to_string(),
                );

                return Ok(report);
            }
        }

        // All stages passed
        pipeline_span.record("validation.passed", true);
        report.passed = true;

        Ok(report)
    }
}
```

### 5.3 Atomic Operation Instrumentation (~1ns)

**Pattern: Minimal Overhead for Picosecond Operations**

```rust
// rust/knhk-closed-loop/src/promoter.rs

pub struct SnapshotPromoter {
    current: ArcSwap<SnapshotDescriptor>,
    history: DashMap<String, Arc<SnapshotDescriptor>>,
}

impl SnapshotPromoter {
    pub fn promote(&self, new_snapshot: SnapshotDescriptor) -> Result<Arc<SnapshotDescriptor>> {
        // Span for promotion (includes pre/post work, not just swap)
        let span = span!(
            Level::INFO,
            "execute.promote_snapshot",
            "snapshot.id" = %new_snapshot.snapshot_id,
            "snapshot.parent_id" = ?new_snapshot.parent_id,
            "snapshot.version" = new_snapshot.version,
            "promotion.latency_ns" = tracing::field::Empty,
            "promotion.success" = tracing::field::Empty,
        );

        let _enter = span.enter();

        // Measure ONLY the atomic swap (critical path)
        let swap_start = Instant::now();

        // Arc creation (~10ns)
        let new_arc = Arc::new(new_snapshot);

        // ATOMIC SWAP (~1ns) - THIS IS THE CRITICAL OPERATION
        let old_arc = self.current.swap(new_arc.clone());

        let swap_latency_ns = swap_start.elapsed().as_nanos() as u64;

        // Record atomic swap latency
        span.record("promotion.latency_ns", swap_latency_ns);

        // Emit metric for atomic operation
        metrics::histogram!("knhk.snapshot.promotion.latency_ns", swap_latency_ns as f64);

        // Post-swap bookkeeping (NOT on critical path)
        let snapshot_id = new_arc.snapshot_id.clone();
        self.history.insert(snapshot_id.clone(), new_arc.clone());

        // Success
        span.record("promotion.success", true);

        // Emit event
        tracing::info!(
            name = "snapshot.promoted",
            "snapshot.id" = %snapshot_id,
            "snapshot.version" = new_arc.version,
        );

        Ok(new_arc)
    }
}
```

### 5.4 Learning Phase Instrumentation

**Pattern: Outcome Tracking**

```rust
// rust/knhk-closed-loop/src/knowledge.rs

pub fn update_knowledge_base(cycle: &LoopCycle, outcomes: &[LearningOutcome]) -> Result<String> {
    let span = span!(
        Level::INFO,
        "knowledge.update_budget",
        "cycle.id" = %cycle.id,
        "budget.ticks_before" = tracing::field::Empty,
        "budget.ticks_after" = tracing::field::Empty,
        "learning.lesson_count" = 0,
    );

    let _enter = span.enter();

    // Calculate acceptance rate
    let accepted = outcomes.iter().filter(|o| o.accepted).count();
    let total = outcomes.len();
    let acceptance_rate = accepted as f64 / total as f64;

    // Emit acceptance rate metric (learning effectiveness)
    metrics::gauge!("knhk.proposal.acceptance_rate", acceptance_rate,
        "sector" => cycle.sector.clone(),
    );

    // Track each outcome
    for outcome in outcomes {
        let status = if outcome.accepted { "accepted" } else { "rejected" };

        metrics::counter!("knhk.proposal.total", 1,
            "sector" => cycle.sector.clone(),
            "acceptance.status" => status,
        );

        // Log decision
        if outcome.accepted {
            tracing::info!(
                name = "proposal.accepted",
                "proposal.id" = %outcome.proposal_id,
                "snapshot.id" = %outcome.snapshot_id.as_ref().unwrap_or(&String::from("none")),
            );
        } else {
            tracing::info!(
                name = "proposal.rejected",
                "proposal.id" = %outcome.proposal_id,
                "rejection.reason" = %outcome.rejection_reason.as_ref().unwrap_or(&String::from("unknown")),
            );
        }
    }

    // Update budgets based on actual performance
    let lesson_count = adjust_budgets(outcomes)?;
    span.record("learning.lesson_count", lesson_count);

    // Generate receipt
    let receipt_id = generate_knowledge_receipt(cycle)?;

    Ok(receipt_id)
}
```

---

## 6. Performance Baseline Methodology

### 6.1 Baseline Workload Definition

**Standard Workload Profile**:
```yaml
# registry/baseline-workload.yaml
workload:
  name: "standard-knhk-workload"
  duration_seconds: 60
  sectors:
    - finance
    - healthcare
    - manufacturing

  observation_rate_per_sec: 1000
  observation_distribution:
    - type: "transaction.execute"
      percentage: 40
    - type: "diagnosis.created"
      percentage: 30
    - type: "production.run"
      percentage: 30

  pattern_detection_interval_sec: 5
  expected_patterns_detected: 10

  proposal_generation_rate: 2  # per minute
  proposal_acceptance_rate_target: 0.75  # 75%

  performance_targets:
    hot_path:
      observation_append_ns: 100
      snapshot_promotion_ns: 10
    warm_path:
      pattern_detection_ms: 50
      proposal_validation_ms: 100
    cold_path:
      shadow_test_sec: 5
```

### 6.2 Baseline Capture Process

```bash
#!/bin/bash
# scripts/capture-performance-baseline.sh

set -e

echo "🎯 Capturing KNHK Performance Baseline"

# Step 1: Ensure clean state
echo "Resetting system state..."
cargo clean
cargo build --release

# Step 2: Start OTEL collector
echo "Starting OTEL collector..."
docker-compose up -d otel-collector
sleep 5

# Step 3: Run baseline workload
echo "Running baseline workload (60 seconds)..."
cargo run --release -- benchmark \
  --workload-config registry/baseline-workload.yaml \
  --output baseline-metrics.json &

KNHK_PID=$!
sleep 65  # Let workload complete

# Step 4: Capture telemetry baseline
echo "Capturing telemetry baseline..."
weaver registry live-check \
  --registry registry/ \
  --telemetry-source http://localhost:4317 \
  --window 60s \
  --capture-baseline registry/baseline-telemetry.json

# Step 5: Extract performance statistics
echo "Extracting performance statistics..."
cat registry/baseline-telemetry.json | jq '
  {
    hot_path: {
      observation_append_ns: {
        p50: .metrics."knhk.observation.latency_ns".p50,
        p95: .metrics."knhk.observation.latency_ns".p95,
        p99: .metrics."knhk.observation.latency_ns".p99
      },
      snapshot_promotion_ns: {
        p50: .metrics."knhk.snapshot.promotion.latency_ns".p50,
        p95: .metrics."knhk.snapshot.promotion.latency_ns".p95,
        p99: .metrics."knhk.snapshot.promotion.latency_ns".p99
      }
    },
    warm_path: {
      pattern_detection_ms: {
        p50: .metrics."knhk.pattern.detection.latency_ms".p50,
        p95: .metrics."knhk.pattern.detection.latency_ms".p95,
        p99: .metrics."knhk.pattern.detection.latency_ms".p99
      },
      proposal_validation_ms: {
        p50: .metrics."knhk.proposal.validation.latency_ms".p50,
        p95: .metrics."knhk.proposal.validation.latency_ms".p95,
        p99: .metrics."knhk.proposal.validation.latency_ms".p99
      }
    },
    learning: {
      acceptance_rate: .metrics."knhk.proposal.acceptance_rate".value
    }
  }
' > registry/baseline-stats.json

echo "✅ Baseline captured successfully"
cat registry/baseline-stats.json
```

### 6.3 Regression Testing

```bash
#!/bin/bash
# scripts/detect-performance-regression.sh

set -e

echo "🔍 Detecting Performance Regressions"

# Step 1: Run current workload
echo "Running current workload..."
cargo run --release -- benchmark \
  --workload-config registry/baseline-workload.yaml \
  --output current-metrics.json &

sleep 65

# Step 2: Capture current telemetry
weaver registry live-check \
  --registry registry/ \
  --telemetry-source http://localhost:4317 \
  --window 60s \
  --capture-baseline current-telemetry.json

# Step 3: Compare against baseline
echo "Comparing against baseline..."
python3 scripts/compare-baselines.py \
  --baseline registry/baseline-telemetry.json \
  --current current-telemetry.json \
  --tolerance 10% \
  --output regression-report.json

# Step 4: Check for regressions
if [ -s regression-report.json ]; then
  echo "❌ REGRESSIONS DETECTED:"
  cat regression-report.json | jq '.regressions[]'
  exit 1
else
  echo "✅ No regressions detected"
  exit 0
fi
```

**Regression Comparison Script** (scripts/compare-baselines.py):
```python
#!/usr/bin/env python3
import json
import sys
from typing import Dict, Any

def compare_baselines(baseline: Dict, current: Dict, tolerance: float) -> list:
    regressions = []

    for metric_name, baseline_value in baseline.get("metrics", {}).items():
        current_value = current.get("metrics", {}).get(metric_name)

        if not current_value:
            regressions.append({
                "metric": metric_name,
                "reason": "Missing in current telemetry",
                "severity": "critical"
            })
            continue

        # Compare p99 latencies
        baseline_p99 = baseline_value.get("p99", 0)
        current_p99 = current_value.get("p99", 0)

        if baseline_p99 > 0:
            deviation = (current_p99 - baseline_p99) / baseline_p99

            if deviation > tolerance:
                regressions.append({
                    "metric": metric_name,
                    "baseline_p99": baseline_p99,
                    "current_p99": current_p99,
                    "deviation_percent": deviation * 100,
                    "tolerance_percent": tolerance * 100,
                    "severity": "high" if deviation > tolerance * 2 else "medium"
                })

    return regressions

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--current", required=True)
    parser.add_argument("--tolerance", default="10%")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    tolerance = float(args.tolerance.rstrip("%")) / 100

    with open(args.baseline) as f:
        baseline = json.load(f)

    with open(args.current) as f:
        current = json.load(f)

    regressions = compare_baselines(baseline, current, tolerance)

    with open(args.output, "w") as f:
        json.dump({"regressions": regressions}, f, indent=2)

    sys.exit(1 if regressions else 0)
```

---

## 7. Regression Detection

### 7.1 Automated Regression Alerts

**Prometheus Alert Rules** (monitoring/prometheus/alerts.yml):
```yaml
groups:
  - name: knhk_performance_regressions
    interval: 60s
    rules:
      # Hot path regression: observation append
      - alert: ObservationAppendLatencyRegression
        expr: |
          histogram_quantile(0.99, rate(knhk_observation_latency_ns_bucket[5m]))
          > 150
        for: 5m
        labels:
          severity: critical
          component: monitor
        annotations:
          summary: "Observation append p99 latency exceeds 150ns"
          description: "P99 latency {{ $value }}ns exceeds target 100ns (50% tolerance)"

      # Hot path regression: snapshot promotion
      - alert: SnapshotPromotionLatencyRegression
        expr: |
          histogram_quantile(0.99, rate(knhk_snapshot_promotion_latency_ns_bucket[5m]))
          > 100
        for: 5m
        labels:
          severity: critical
          component: execute
        annotations:
          summary: "Snapshot promotion p99 latency exceeds 100ns"
          description: "P99 latency {{ $value }}ns exceeds target 10ns (10x tolerance)"

      # Warm path regression: validation
      - alert: ProposalValidationLatencyRegression
        expr: |
          histogram_quantile(0.99, rate(knhk_proposal_validation_latency_ms_bucket[5m]))
          > 150
        for: 10m
        labels:
          severity: warning
          component: plan
        annotations:
          summary: "Proposal validation p99 latency exceeds 150ms"
          description: "P99 latency {{ $value }}ms exceeds target 100ms"

      # Learning regression: acceptance rate drop
      - alert: AcceptanceRateRegression
        expr: |
          rate(knhk_proposal_total{acceptance_status="accepted"}[1h])
          / rate(knhk_proposal_total[1h])
          < 0.5
        for: 30m
        labels:
          severity: warning
          component: knowledge
        annotations:
          summary: "Proposal acceptance rate below 50%"
          description: "Acceptance rate {{ $value | humanizePercentage }} suggests learning regression"

      # Critical: Invariant violations
      - alert: InvariantViolationDetected
        expr: increase(knhk_invariant_violation_count[5m]) > 0
        labels:
          severity: critical
          component: validation
        annotations:
          summary: "Hard invariant violation detected"
          description: "Invariant {{ $labels.invariant_id }} violated {{ $value }} times in last 5 minutes"
```

### 7.2 Trend Analysis

**Weekly Performance Report** (scripts/generate-performance-report.sh):
```bash
#!/bin/bash
# Generate weekly performance trend report

WEEK_START=$(date -d "7 days ago" +%s)
WEEK_END=$(date +%s)

# Query Prometheus for last 7 days
curl -s "http://prometheus:9090/api/v1/query_range" \
  --data-urlencode "query=histogram_quantile(0.99, rate(knhk_observation_latency_ns_bucket[1h]))" \
  --data-urlencode "start=$WEEK_START" \
  --data-urlencode "end=$WEEK_END" \
  --data-urlencode "step=3600" \
  | jq '.data.result[0].values' > /tmp/latency_trend.json

# Analyze trend
python3 << 'EOF'
import json
import numpy as np

with open("/tmp/latency_trend.json") as f:
    data = json.load(f)

values = [float(v[1]) for v in data]
timestamps = [int(v[0]) for v in data]

# Calculate trend (linear regression)
coeffs = np.polyfit(range(len(values)), values, 1)
trend_slope = coeffs[0]

print(f"Latency Trend Analysis (7 days):")
print(f"  Current p99: {values[-1]:.2f}ns")
print(f"  7-day average: {np.mean(values):.2f}ns")
print(f"  Trend slope: {trend_slope:.4f}ns/hour")

if trend_slope > 0.1:
    print(f"  ⚠️  WARNING: Latency increasing over time")
elif trend_slope < -0.1:
    print(f"  ✅ Latency improving over time")
else:
    print(f"  ✅ Latency stable")
EOF
```

---

## 8. CI/CD Integration

### 8.1 GitHub Actions Workflow

**Complete Weaver Validation Workflow** (.github/workflows/weaver-validation.yml):

```yaml
name: Weaver Telemetry Validation (Source of Truth)

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  weaver-validation:
    name: Validate Runtime Telemetry (MANDATORY)
    runs-on: ubuntu-latest

    services:
      otel-collector:
        image: otel/opentelemetry-collector:latest
        ports:
          - 4317:4317
          - 4318:4318
        options: >-
          --health-cmd "curl -f http://localhost:13133/ || exit 1"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Install Weaver
        run: |
          curl -L https://github.com/open-telemetry/weaver/releases/latest/download/weaver-linux-amd64 \
            -o /usr/local/bin/weaver
          chmod +x /usr/local/bin/weaver
          weaver --version

      # LEVEL 1: Schema Definition Validation
      - name: Validate OTEL Schema Definition
        run: |
          echo "🔍 Validating OTEL schema definition..."
          weaver registry check -r registry/

          if [ $? -ne 0 ]; then
            echo "❌ SCHEMA DEFINITION INVALID"
            exit 1
          fi

          echo "✅ Schema definition valid"

      # LEVEL 2: Compilation & Code Quality
      - name: Build Project
        run: |
          echo "🔨 Building project..."
          cargo build --release

      - name: Run Clippy
        run: |
          cargo clippy --workspace -- -D warnings

      # LEVEL 3: Traditional Tests (Supporting Evidence)
      - name: Run Unit Tests
        run: |
          cargo test --workspace

      # LEVEL 1: Live Telemetry Validation (SOURCE OF TRUTH)
      - name: Run System and Emit Telemetry
        run: |
          echo "🚀 Starting KNHK system..."
          cargo run --release &
          KNHK_PID=$!
          echo "KNHK_PID=$KNHK_PID" >> $GITHUB_ENV

          # Let system run and emit telemetry
          echo "⏱️  Waiting 30 seconds for telemetry emission..."
          sleep 30

      - name: Validate Runtime Telemetry (SOURCE OF TRUTH)
        run: |
          echo "🔍 Validating runtime telemetry against schema..."

          weaver registry live-check \
            --registry registry/ \
            --telemetry-source http://localhost:4317 \
            --window 30s \
            --output validation-report.json \
            --fail-fast

          VALIDATION_RESULT=$?

          # Stop KNHK system
          kill $KNHK_PID || true

          if [ $VALIDATION_RESULT -ne 0 ]; then
            echo ""
            echo "❌ =========================================="
            echo "❌ WEAVER VALIDATION FAILED"
            echo "❌ =========================================="
            echo ""
            echo "This means the feature DOES NOT WORK, even if traditional tests pass."
            echo ""
            echo "Validation report:"
            cat validation-report.json | jq '.'
            echo ""
            exit 1
          fi

          echo ""
          echo "✅ =========================================="
          echo "✅ WEAVER VALIDATION PASSED"
          echo "✅ =========================================="
          echo ""
          echo "Runtime telemetry matches schema - feature works correctly!"
          echo ""

      - name: Upload Validation Report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: weaver-validation-report
          path: validation-report.json

      - name: Performance Regression Check
        run: |
          echo "📊 Checking for performance regressions..."

          # Download baseline from artifacts (if exists)
          # Compare current vs baseline
          # Alert if regression detected

          # TODO: Implement regression comparison
          echo "ℹ️  Regression check not yet implemented"
```

### 8.2 Pre-Commit Hooks

**Weaver Validation Hook** (.git/hooks/pre-commit):
```bash
#!/bin/bash
# Pre-commit hook: Validate schema changes

set -e

# Check if registry files changed
REGISTRY_CHANGED=$(git diff --cached --name-only | grep "^registry/" || true)

if [ -n "$REGISTRY_CHANGED" ]; then
  echo "🔍 Registry files changed, validating schema..."

  weaver registry check -r registry/

  if [ $? -ne 0 ]; then
    echo "❌ Schema validation FAILED - fix before committing"
    exit 1
  fi

  echo "✅ Schema validation passed"
fi

exit 0
```

---

## 9. Monitoring and Dashboards

### 9.1 Grafana Dashboard Configuration

**KNHK Performance Dashboard** (monitoring/grafana/dashboards/knhk-performance.json):

```json
{
  "dashboard": {
    "title": "KNHK Performance - Weaver Validated Telemetry",
    "tags": ["knhk", "performance", "weaver"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Hot Path: Observation Append Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(knhk_observation_latency_ns_bucket[5m]))",
            "legendFormat": "p50"
          },
          {
            "expr": "histogram_quantile(0.95, rate(knhk_observation_latency_ns_bucket[5m]))",
            "legendFormat": "p95"
          },
          {
            "expr": "histogram_quantile(0.99, rate(knhk_observation_latency_ns_bucket[5m]))",
            "legendFormat": "p99"
          }
        ],
        "yaxes": [
          {
            "label": "Latency (ns)",
            "format": "ns"
          }
        ],
        "thresholds": [
          {
            "value": 100,
            "colorMode": "critical",
            "op": "gt",
            "label": "Target: 100ns"
          }
        ]
      },
      {
        "id": 2,
        "title": "Hot Path: Snapshot Promotion Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(knhk_snapshot_promotion_latency_ns_bucket[5m]))",
            "legendFormat": "p99"
          }
        ],
        "thresholds": [
          {
            "value": 10,
            "colorMode": "critical",
            "op": "gt",
            "label": "Target: 10ns"
          }
        ]
      },
      {
        "id": 3,
        "title": "Warm Path: Validation Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(knhk_proposal_validation_latency_ms_bucket[5m]))",
            "legendFormat": "p99 validation"
          },
          {
            "expr": "histogram_quantile(0.99, rate(knhk_pattern_detection_latency_ms_bucket[5m]))",
            "legendFormat": "p99 pattern detection"
          }
        ],
        "thresholds": [
          {
            "value": 100,
            "colorMode": "critical",
            "op": "gt",
            "label": "Target: 100ms"
          }
        ]
      },
      {
        "id": 4,
        "title": "Learning: Proposal Acceptance Rate",
        "type": "gauge",
        "targets": [
          {
            "expr": "rate(knhk_proposal_total{acceptance_status=\"accepted\"}[1h]) / rate(knhk_proposal_total[1h])"
          }
        ],
        "thresholds": {
          "mode": "absolute",
          "steps": [
            { "value": 0, "color": "red" },
            { "value": 0.5, "color": "yellow" },
            { "value": 0.75, "color": "green" }
          ]
        }
      },
      {
        "id": 5,
        "title": "Critical: Invariant Violations (MUST BE ZERO)",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(increase(knhk_invariant_violation_count[1h]))"
          }
        ],
        "thresholds": {
          "mode": "absolute",
          "steps": [
            { "value": 0, "color": "green" },
            { "value": 1, "color": "red" }
          ]
        }
      }
    ]
  }
}
```

### 9.2 Weaver Validation Dashboard

**Weaver Compliance Dashboard** (monitoring/grafana/dashboards/weaver-validation.json):

```json
{
  "dashboard": {
    "title": "Weaver Schema Validation Status",
    "panels": [
      {
        "id": 1,
        "title": "Schema Validation Status",
        "type": "stat",
        "description": "Last Weaver registry check result",
        "targets": [
          {
            "expr": "weaver_schema_validation_success"
          }
        ],
        "mappings": [
          {
            "type": "value",
            "options": {
              "0": { "text": "FAILED", "color": "red" },
              "1": { "text": "PASSED", "color": "green" }
            }
          }
        ]
      },
      {
        "id": 2,
        "title": "Spans Declared vs Emitted",
        "type": "bargauge",
        "targets": [
          {
            "expr": "weaver_spans_declared",
            "legendFormat": "Declared"
          },
          {
            "expr": "weaver_spans_emitted",
            "legendFormat": "Emitted"
          }
        ]
      },
      {
        "id": 3,
        "title": "Missing Spans (CRITICAL)",
        "type": "table",
        "targets": [
          {
            "expr": "weaver_missing_spans"
          }
        ]
      }
    ]
  }
}
```

---

## 10. Schema Evolution Strategy

### 10.1 Versioned Schema Management

**Schema Versioning** (registry/versions.yaml):
```yaml
# registry/versions.yaml
schema_versions:
  - version: "1.0.0"
    released: "2025-11-16"
    status: "stable"
    changes:
      - "Initial MAPE-K phase spans"
      - "Hot path and warm path metrics"
      - "Invariant violation tracking"

  - version: "1.1.0"
    released: "TBD"
    status: "draft"
    changes:
      - "Add sector-specific spans"
      - "Add LLM proposer telemetry"
      - "Add guard relaxation metrics"
```

### 10.2 Backward Compatibility

**Deprecation Policy**:
```
1. Mark attribute as deprecated in schema (add deprecated: true)
2. Continue emitting deprecated attribute for 2 releases
3. Emit both old and new attributes during transition
4. Remove deprecated attribute after 2 releases
```

**Example Migration**:
```yaml
# Old attribute (deprecated)
- id: observation.timestamp
  type: int
  brief: "Deprecated: Use observation.timestamp_ms"
  deprecated: true

# New attribute
- id: observation.timestamp_ms
  type: int
  brief: "Unix timestamp in milliseconds"
```

### 10.3 Schema Change Process

```
1. Propose schema change (PR with registry/ changes)
2. Run weaver registry check (validate syntax)
3. Update instrumentation code to emit new attributes
4. Run weaver registry live-check (validate runtime)
5. Update baseline telemetry
6. Merge after validation passes
```

---

## Conclusion

This Weaver validation integration provides the **ONLY source of truth** for proving KNHK features work correctly. By validating actual runtime telemetry against declared schemas, we eliminate false positives that plague traditional testing.

**Key Takeaways**:

1. **Schema-First Development**: Define telemetry schema BEFORE implementing features
2. **Live Validation**: Runtime telemetry MUST match schema (weaver registry live-check)
3. **No False Positives**: If Weaver validation fails, feature doesn't work (regardless of test results)
4. **Continuous Monitoring**: Daily Weaver checks in production detect schema drift
5. **Performance Baselines**: Establish and track performance targets via telemetry
6. **CI/CD Integration**: Block merges if Weaver validation fails

**Validation Hierarchy**:
- **Level 1** (MANDATORY): Weaver schema validation = Source of Truth
- **Level 2** (Baseline): Compilation + Clippy = Code quality
- **Level 3** (Supporting): Traditional tests = Supporting evidence (can have false positives)

**Next Steps** (Implementation):
1. Complete all span definitions in registry/otel/spans.yaml
2. Add instrumentation code to emit declared spans
3. Set up CI/CD pipeline with Weaver validation
4. Establish performance baselines
5. Deploy monitoring dashboards

---

**Version**: 1.0.0
**Status**: Design Complete
**Authors**: Performance Benchmarker Agent
**Last Updated**: 2025-11-16
