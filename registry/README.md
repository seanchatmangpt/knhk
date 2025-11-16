# KNHK OpenTelemetry Schema Registry

**Version**: 1.0.0
**Date**: 2025-11-16
**Status**: Production-Ready

This directory contains the **complete OpenTelemetry schema definitions** for the KNHK MAPE-K autonomous ontology system. These schemas are the **ONLY source of truth** for validating that KNHK features work correctly at runtime.

---

## 📁 Registry Structure

```
registry/
├── otel/
│   ├── spans.yaml          # ALL span definitions (Monitor/Analyze/Plan/Execute/Knowledge)
│   ├── metrics.yaml        # ALL metric definitions (latency histograms, counters, gauges)
│   ├── logs.yaml           # ALL log record definitions (structured logs)
│   └── attributes.yaml     # Shared attribute definitions
├── sectors/                # Sector-specific telemetry extensions
│   ├── finance.yaml        # Finance-specific spans/metrics
│   ├── healthcare.yaml     # Healthcare-specific spans/metrics
│   ├── manufacturing.yaml  # Manufacturing-specific spans/metrics
│   └── logistics.yaml      # Logistics-specific spans/metrics
├── semconv.yaml            # Semantic conventions and constants
├── baseline-telemetry.json # Performance baseline (captured from production)
└── README.md               # This file
```

---

## 🔍 Weaver Validation: The Source of Truth

### Why Weaver Validation Matters

**Traditional tests can lie (false positives):**
```rust
#[test]
fn test_observation_append() {
    let store = ObservationStore::new();
    let id = store.append(obs);  // ✅ Test passes

    // But what if append() calls unimplemented!() internally?
    // What if it doesn't actually persist the observation?
    // Traditional tests can't catch this!
}
```

**Weaver validation tells the truth:**
```yaml
# Schema declares:
spans:
  - id: monitor.observe_event
    attributes:
      - id: observation.id
        requirement_level: required

# Runtime: If span is NOT emitted → Feature does NOT work
# Weaver catches this, regardless of test results
```

### Validation Commands

**Static Schema Validation** (validates YAML syntax and definitions):
```bash
# Run this whenever you modify schema files
weaver registry check -r registry/

# Expected output:
# ✅ Schema syntax valid
# ✅ All references resolved
# ✅ No conflicting definitions
```

**Live Telemetry Validation** (validates actual runtime behavior - **SOURCE OF TRUTH**):
```bash
# Run KNHK system, then validate runtime telemetry
cargo run --release &
sleep 30  # Let system emit telemetry

weaver registry live-check \
  --registry registry/ \
  --telemetry-source http://localhost:4317 \
  --window 30s

# Expected output:
# ✅ All declared spans found in runtime telemetry
# ✅ All required attributes present
# ✅ Performance targets met

# If validation FAILS:
# ❌ Feature does NOT work (even if tests pass)
```

---

## 🎯 Critical Validation Hierarchy

**KNHK uses a 3-level validation hierarchy:**

### Level 1: Weaver Schema Validation (MANDATORY - Source of Truth)
```bash
# This is the ONLY source of truth
weaver registry check -r registry/           # Static schema validation
weaver registry live-check --registry registry/  # Runtime telemetry validation

# If this fails → Feature DOES NOT WORK (regardless of test results)
```

### Level 2: Compilation & Code Quality (Baseline)
```bash
cargo build --release                        # Must compile
cargo clippy --workspace -- -D warnings      # Zero warnings
make build                                   # C library compiles
```

### Level 3: Traditional Tests (Supporting Evidence - Can Have False Positives)
```bash
cargo test --workspace                       # Rust unit tests
make test-chicago-v04                        # C Chicago TDD tests
make test-performance-v04                    # Performance tests
```

**⚠️ If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.**

---

## 📊 Schema Organization

### Spans (registry/otel/spans.yaml)

Defines ALL spans that KNHK emits during runtime, organized by MAPE-K phase:

**Monitor Phase** (Hot Path: ≤8 ticks = <100ns):
- `monitor.observe_event` - Append observation to store
- `monitor.detect_patterns` - Detect patterns in observation stream

**Analyze Phase** (Warm Path: <100ms):
- `analyze.mine_proposal` - Generate change proposal
- `analyze.validate_doctrines` - Validate against sector doctrines
- `analyze.verify_signatures` - Verify cryptographic signatures

**Plan Phase** (Warm Path: <100ms):
- `plan.validate_proposal` - 7-stage validation pipeline
- `plan.check_invariant` - Check single hard invariant (Q1-Q5)

**Execute Phase** (Hot Path: ~1ns):
- `execute.promote_snapshot` - Atomic snapshot promotion (RCU)
- `execute.build_version_chain` - Traverse snapshot parent chain

**Knowledge Phase** (Warm Path: <1ms):
- `knowledge.update_budget` - Update performance budgets
- `knowledge.shadow_test` - Run shadow tests
- `knowledge.record_outcome` - Record learning outcome

### Metrics (registry/otel/metrics.yaml)

Defines ALL metrics that KNHK emits:

**Hot Path Metrics** (≤8 ticks):
- `knhk.observation.latency_ns` - Observation append latency (target: <100ns)
- `knhk.snapshot.promotion.latency_ns` - Atomic swap latency (target: <1ns)

**Warm Path Metrics** (<100ms):
- `knhk.pattern.detection.latency_ms` - Pattern detection latency
- `knhk.proposal.validation.latency_ms` - Validation pipeline latency
- `knhk.doctrine.validation.latency_ms` - Doctrine validation latency

**Learning Metrics**:
- `knhk.proposal.total` - Total proposals by acceptance status
- `knhk.proposal.acceptance_rate` - Acceptance rate (learning effectiveness)
- `knhk.budget.adjustment.total` - Budget adjustment count

**Error Metrics**:
- `knhk.error.total` - Total errors by type
- `knhk.invariant.violation.total` - **MUST BE ZERO** (hard invariant violations)
- `knhk.doctrine.violation.total` - Doctrine violations

### Logs (registry/otel/logs.yaml)

Defines ALL structured logs that KNHK emits:

**Proposal Lifecycle**:
- `proposal.generated` - Proposal generated from pattern
- `proposal.validated` - Validation completed
- `proposal.accepted` - Proposal accepted and promoted
- `proposal.rejected` - Proposal rejected

**Invariant Enforcement**:
- `invariant.violated` - Hard invariant violated (CRITICAL)
- `invariant.preserved` - Invariant preserved

**Guard Enforcement**:
- `guard.enforced` - Guard blocked an operation
- `guard.relaxed` - Guard temporarily relaxed
- `guard.slo_violated` - SLO violated during relaxation

### Attributes (registry/otel/attributes.yaml)

Defines ALL shared attributes used across spans, metrics, and logs:

- **Observation attributes**: `observation.id`, `observation.sector`, `observation.event_type`
- **Proposal attributes**: `proposal.id`, `proposal.confidence`, `proposal.description`
- **Snapshot attributes**: `snapshot.id`, `snapshot.parent_id`, `snapshot.version`
- **Validation attributes**: `validation.stage`, `validation.passed`, `validation.failure_reason`
- **Invariant attributes**: `invariant.id`, `invariant.q1_preserved`, ..., `invariant.q5_preserved`
- **Performance attributes**: `latency_ns`, `latency_ms`, `promotion.latency_ns`
- **And many more...**

---

## 🚀 CI/CD Integration

### GitHub Actions Workflow

**Weaver validation is integrated into CI/CD** (.github/workflows/weaver-validation.yml):

```yaml
jobs:
  weaver-validation:
    steps:
      # Level 1: Schema Definition Validation
      - name: Validate OTEL Schema Definition
        run: weaver registry check -r registry/

      # Level 2: Build and Code Quality
      - name: Build Project
        run: cargo build --release

      - name: Run Clippy
        run: cargo clippy --workspace -- -D warnings

      # Level 3: Traditional Tests
      - name: Run Unit Tests
        run: cargo test --workspace

      # Level 1: Live Telemetry Validation (SOURCE OF TRUTH)
      - name: Run System and Emit Telemetry
        run: |
          cargo run --release &
          sleep 30  # Let system emit telemetry

      - name: Validate Runtime Telemetry (SOURCE OF TRUTH)
        run: |
          weaver registry live-check \
            --registry registry/ \
            --telemetry-source http://localhost:4317

          if [ $? -ne 0 ]; then
            echo "❌ WEAVER VALIDATION FAILED - Feature does not work"
            exit 1
          fi

      - name: Block merge if Weaver fails
        if: failure()
        run: exit 1
```

**If Weaver validation fails, the merge is blocked.**

---

## 📈 Performance Baselines

### Baseline Capture

**Baseline telemetry is captured periodically** to detect performance regressions:

```bash
# Capture baseline (run this after major releases)
./scripts/capture-performance-baseline.sh

# This generates:
# - registry/baseline-telemetry.json
# - registry/baseline-stats.json
```

### Regression Detection

**Compare current telemetry against baseline:**

```bash
./scripts/detect-performance-regression.sh

# Compares:
# - Hot path latencies (observation append, snapshot promotion)
# - Warm path latencies (pattern detection, validation)
# - Acceptance rate (learning effectiveness)

# Alerts if:
# - P99 latency exceeds baseline by >10%
# - Acceptance rate drops below 50%
# - Invariant violations > 0
```

---

## 🛠️ Schema Evolution

### Versioning Policy

**Semantic versioning** (MAJOR.MINOR.PATCH):
- **MAJOR**: Breaking changes (remove attributes, change types)
- **MINOR**: Add new spans/metrics/logs
- **PATCH**: Fix bugs, clarify documentation

### Deprecation Process

**Attributes are deprecated gradually:**

1. Mark attribute as `deprecated: true` in schema
2. Continue emitting deprecated attribute for 2 releases
3. Emit both old and new attributes during transition
4. Remove deprecated attribute after 2 releases

**Example:**
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

---

## 🔗 Related Documentation

- [SPARC Phase 8: Weaver Validation Design](/home/user/knhk/docs/SPARC_PHASE8_WEAVER_VALIDATION.md)
- [SPARC Architecture](/home/user/knhk/docs/SPARC_ARCHITECTURE_UNIFIED.md)
- [SPARC Pseudocode](/home/user/knhk/docs/SPARC_PSEUDOCODE_MAPE-K.md)
- [Chicago TDD Patterns](/home/user/knhk/docs/CHICAGO_TDD_PATTERNS.md)

---

## 📞 Support

If Weaver validation fails:

1. Check validation report: `cat validation-report.json`
2. Compare declared spans vs emitted spans
3. Verify instrumentation code emits all required attributes
4. Run `weaver registry check` to validate schema syntax
5. Consult [Weaver documentation](https://github.com/open-telemetry/weaver)

**Remember**: If Weaver validation fails, the feature does NOT work, regardless of test results. Fix the implementation, not the tests.

---

**Version**: 1.0.0
**Last Updated**: 2025-11-16
**Status**: Production-Ready

