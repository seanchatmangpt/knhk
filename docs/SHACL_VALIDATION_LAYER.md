# SHACL Validation Layer: Covenant 2 Implementation

**Status**: ✅ COMPLETE | **Version**: 1.0.0 | **Date**: 2025-11-16

---

## Executive Summary

This document describes the complete SHACL (Shapes Constraint Language) validation layer that enforces **Covenant 2: Invariants Are Law** from DOCTRINE_2027. This validation layer ensures that all workflow definitions satisfy hard quality constraints (Q invariants) before deployment.

**Key Achievement**: Quality constraints are now **automatically enforced**, not advisory. Violations block deployment, not humans reviewing code.

---

## DOCTRINE ALIGNMENT

### Principle: Q (Hard Invariants)

**From DOCTRINE_2027**:
> "Total Quality Leadership" and "conscientious program managers" become something much sharper: invariants the system cannot cross.

**From DOCTRINE_COVENANT (Covenant 2)**:
> Q invariants are not suggestions; they are enforceable constraints. Every design decision must satisfy all Q conditions. Violations are not warnings; they are errors that block promotion.

### What This Means

Quality is not optional—it is **law**. This SHACL validation layer is the **enforcement mechanism** that makes Q constraints executable at machine speed.

**The Meta-Shift**:
- **Before**: Quality checked by humans (slow, lossy, inconsistent)
- **After**: Quality checked by SHACL validators (fast, complete, automatic)

---

## Q INVARIANTS ENFORCED

The SHACL validation layer enforces all five Q invariants from Covenant 2:

### Q1: No Retrocausation (Immutability)

**Invariant**: Observation snapshots and state transitions form an immutable DAG. Time only moves forward.

**SHACL Enforcement**:
- ✅ Workflows must have immutable version numbers
- ✅ Backward flows (cycles) must have iteration control ≤ 8
- ✅ Execution state transitions must be monotonic
- ✅ Observations must have immutable timestamps

**Validation**: `q:NoCyclesViolatingImmutability`, `q:WorkflowVersionImmutability`, `q:MonotonicStateTransitions`

**Example Violation**:
```turtle
:BadTask yawl:BackwardFlow :BadTask .
# ❌ Missing: yawl:MaxIterations 8
```

---

### Q2: Type Soundness (O ⊨ Σ)

**Invariant**: All observations must satisfy the current ontology. All split-join combinations must be in the permutation matrix.

**SHACL Enforcement**:
- ✅ Split-join combinations validated against permutation matrix
- ✅ Data inputs/outputs must declare explicit types
- ✅ Execution modes must be valid (Sync/Async/Queued/Parallel)
- ✅ Resource assignments must match capability requirements
- ✅ Event types must be declared (Message/Timer/Error/Signal)
- ✅ Metric types must be valid categories

**Validation**: `q:ValidSplitJoinCombination`, `q:DataTypeSoundness`, `q:ValidExecutionMode`

**Example Violation**:
```turtle
:BadTask yawl:hasSplitType yawl:AND ;
         yawl:hasJoinType yawl:Discriminator .
# ❌ AND-Discriminator not in permutation matrix
```

---

### Q3: Bounded Recursion (max_run_length ≤ 8)

**Invariant**: The Chatman constant enforces complexity bounds. All critical path operations ≤ 8 ticks.

**SHACL Enforcement**:
- ✅ Iteration bounds must be ≤ 8 (Chatman constant)
- ✅ All cycles must have declared MaxIterations
- ✅ Recursion depth must be limited
- ✅ Cycle detection mode required (DepthLimited or CounterBased)
- ✅ MAPE-K loop frequency must be bounded

**Validation**: `q:ChatmanConstantIterationBound`, `q:RecursionDepthLimit`, `q:CycleDetectionRequired`

**Example Violation**:
```turtle
:BadTask yawl:MaxIterations 10 .
# ❌ Exceeds Chatman constant (max 8)
```

---

### Q4: Latency SLOs (hot path ≤ 8 ticks, warm ≤ 100ms)

**Invariant**: Performance is not aspirational—it is contractual.

**SHACL Enforcement**:
- ✅ Hot path tasks must declare expectedDuration ≤ 8 ticks
- ✅ Async tasks must have timeout policies
- ✅ Milestone tasks must declare timeout
- ✅ MAPE-K Execute phase respects Chatman constant

**Validation**: `q:HotPathDurationDeclaration`, `q:AsyncTaskTimeoutPolicy`, `q:MilestoneTimeoutRequired`

**Example Violation**:
```turtle
:AsyncTask yawl-exec:executionMode yawl-exec:Asynchronous .
# ❌ Missing: yawl-exec:timeoutPolicy
```

---

### Q5: Resource Bounds (explicit CPU, memory, throughput)

**Invariant**: All resource consumption must be explicitly bounded and measurable.

**SHACL Enforcement**:
- ✅ Parallel tasks must declare MaxConcurrency
- ✅ Critical sections must declare semaphore capacity
- ✅ Resource metrics must be monitored
- ✅ Discriminator joins must declare quorum threshold
- ✅ Workflows should declare resource constraints

**Validation**: `q:ConcurrencyLimitRequired`, `q:CriticalSectionSemaphore`, `q:DiscriminatorQuorumRequired`

**Example Violation**:
```turtle
:ParallelTask yawl-exec:executionMode yawl-exec:Parallel .
# ❌ Missing: yawl-exec:MaxConcurrency
```

---

## DELIVERABLES

### 1. Q Invariants SHACL Shapes (`ontology/shacl/q-invariants.ttl`)

**Size**: 521 lines
**Purpose**: Enforce all Q constraints (Q1-Q5)
**Validations**: 25+ SHACL shapes covering all Q invariants

**Key Features**:
- No warnings—only violations (hard constraints)
- Pattern permutation matrix validation
- Chatman constant enforcement
- Resource bounds checking
- Type soundness verification

**Example Shape** (Q3: Bounded Recursion):
```turtle
q:ChatmanConstantIterationBound a sh:NodeShape ;
    sh:targetClass yawl:Task ;
    sh:sparql [
        sh:message "Q3-VIOLATION: Task has iteration bound > 8 ticks (Chatman constant)" ;
        sh:severity sh:Violation ;
        sh:select """
            SELECT $this ?maxIter WHERE {
                $this yawl:MaxIterations ?maxIter .
                FILTER (?maxIter > 8)
            }
        """ ;
    ] .
```

---

### 2. Workflow Soundness SHACL Shapes (`ontology/shacl/workflow-soundness.ttl`)

**Size**: 415 lines
**Purpose**: Structural correctness validation (complements Q invariants)
**Validations**: 20+ SHACL shapes for workflow structure

**Coverage**:
- ✅ **Control Flow**: Proper start/end, connectivity, reachability
- ✅ **Data Flow**: Variables initialized, types consistent, transformations valid
- ✅ **Event Flow**: Events declared, handlers present, recovery defined
- ✅ **Structural**: No orphaned elements, valid references, complete workflows
- ✅ **MAPE-K**: All components present, rules valid, knowledge integrated
- ✅ **Composition**: Interfaces defined, mappings complete

**Relationship to Q Invariants**:
- `workflow-soundness.ttl` → **CAN** it execute? (structural)
- `q-invariants.ttl` → **SHOULD** it execute? (quality)

---

### 3. Validation Script (`ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh`)

**Size**: 350 lines
**Purpose**: Executable validation script with clear error reporting
**Dependencies**: `rapper` (Turtle syntax), `pyshacl` (SHACL validation)

**Features**:
- ✅ Validates Turtle syntax before SHACL validation
- ✅ Clear violation/warning/info categorization
- ✅ Color-coded output (red=error, yellow=warning, green=success)
- ✅ Detailed error messages with covenant references
- ✅ Exit codes: 0=pass, 1=violations, 2=warnings, 3=script error
- ✅ Verbose mode for detailed diagnostics

**Usage**:
```bash
# Validate workflow with all shapes
./validate-shapes.sh examples/workflow.ttl

# Verbose output
./validate-shapes.sh workflow.ttl --verbose

# Custom shapes only
./validate-shapes.sh workflow.ttl --shapes custom.ttl

# Skip Q invariants
./validate-shapes.sh workflow.ttl --no-q-invariants
```

**Output Example**:
```
═══════════════════════════════════════════════════════════════════
  SHACL VALIDATION: simple-workflow.ttl
═══════════════════════════════════════════════════════════════════

INFO: Validating Turtle syntax: simple-workflow.ttl
SUCCESS: Turtle syntax valid

INFO: Running SHACL validation: q-invariants
SUCCESS: q-invariants: No violations

INFO: Running SHACL validation: workflow-soundness
SUCCESS: workflow-soundness: No violations

═══════════════════════════════════════════════════════════════════
  VALIDATION SUMMARY
═══════════════════════════════════════════════════════════════════

SUCCESS: All validations passed! ✓

Workflow satisfies all Q invariants and soundness constraints.
Ready for deployment.
```

---

### 4. Validation Examples (`validation-examples/`)

**Structure**:
```
validation-examples/
├── valid/
│   ├── simple-workflow.ttl       # Basic sequential workflow
│   └── parallel-workflow.ttl     # AND split-join with resource bounds
├── invalid/
│   ├── unbounded-recursion.ttl   # Violates Q3
│   ├── type-mismatch.ttl         # Violates Q2
│   └── missing-resource-bounds.ttl # Violates Q5
└── README.md                      # Complete usage guide
```

**Valid Examples**:
- `simple-workflow.ttl`: Demonstrates all Q invariants satisfied (sequential flow)
- `parallel-workflow.ttl`: Demonstrates Q5 resource bounds (parallel tasks with MaxConcurrency)

**Invalid Examples** (demonstrate enforcement):
- `unbounded-recursion.ttl`: Cycle without MaxIterations (Q3 violation)
- `type-mismatch.ttl`: Invalid split-join + type mismatch (Q2 violation)
- `missing-resource-bounds.ttl`: Parallel task without concurrency limit (Q5 violation)

Each invalid example includes:
- Clear violation markers (❌)
- Expected validation results
- Covenant references
- Explanation of what's wrong and how to fix

---

## VALIDATION HIERARCHY

The validation system operates in a strict hierarchy:

### Level 1: SHACL Validation (MANDATORY - Source of Truth)

```bash
# Q invariants validation
weaver registry check -r registry/         # Schema is valid
pyshacl -s q-invariants.ttl workflow.ttl   # Runtime conforms to Q

# If SHACL fails, workflow DOES NOT DEPLOY
```

**Result**: Hard failures (sh:Violation) block all deployment

---

### Level 2: Compilation & Code Quality (Baseline)

```bash
cargo build --release                      # Must compile
cargo clippy --workspace -- -D warnings    # Zero warnings
make build                                 # C library compiles
```

**Result**: Code quality baseline—must pass for build

---

### Level 3: Traditional Tests (Supporting Evidence)

```bash
cargo test --workspace                     # Rust unit tests
make test-chicago-v04                      # Chicago TDD (latency)
make test-performance-v04                  # Performance tests
make test-integration-v2                   # Integration tests
```

**Result**: Supporting evidence—can have false positives

---

**Critical Hierarchy Rule**:

> **If Weaver/SHACL validation fails, the feature DOES NOT WORK, regardless of test results.**

This is the core of Covenant 2: tests can lie (false positives), but schemas cannot.

---

## ANTI-PATTERNS PREVENTED

The SHACL validation layer **automatically prevents** these violations:

### ❌ Unbounded Loops or Recursion (Q3)
```turtle
# WRONG: Unbounded cycle
:Task yawl:BackwardFlow :Task .

# RIGHT: Bounded cycle
:Task yawl:BackwardFlow :Task ;
      yawl:MaxIterations 8 ;
      yawl:CycleDetectionMode yawl:DepthLimited .
```

### ❌ Latency Exceeding SLO (Q4)
```turtle
# WRONG: No duration declared
:HotPathTask yawl-exec:executionMode yawl-exec:Synchronous .

# RIGHT: Duration declared (≤ 8 ticks)
:HotPathTask yawl-exec:executionMode yawl-exec:Synchronous ;
             yawl:expectedDuration "PT0.000000002S"^^xsd:duration ;
             yawl:criticalPath true .
```

### ❌ State Mutations Violating Immutability (Q1)
```turtle
# WRONG: No version number
:Workflow a yawl:WorkflowSpecification .

# RIGHT: Immutable version
:Workflow a yawl:WorkflowSpecification ;
          yawl:versionNumber "1.0.0" .
```

### ❌ Resource Consumption Exceeding Bounds (Q5)
```turtle
# WRONG: Unbounded parallelism
:Task yawl-exec:executionMode yawl-exec:Parallel .

# RIGHT: Bounded concurrency
:Task yawl-exec:executionMode yawl-exec:Parallel ;
      yawl-exec:MaxConcurrency 4 .
```

### ❌ Type Violations (Q2)
```turtle
# WRONG: Invalid split-join combination
:Task yawl:hasSplitType yawl:AND ;
      yawl:hasJoinType yawl:Discriminator .

# RIGHT: Valid combination from matrix
:Task yawl:hasSplitType yawl:AND ;
      yawl:hasJoinType yawl:AND .
```

### ❌ Patterns Not in Permutation Matrix (Q2)
```turtle
# WRONG: Unvalidated pattern
:Task yawl:customPattern <http://example.org/weird-pattern> .

# RIGHT: Use validated W3C patterns
:Task yawl:implementsPattern yawl-pattern:ParallelSplit .
```

---

## ENFORCEMENT PROTOCOL

### Before Any Code Is Written
1. Map to doctrine: Which Q invariant does this implement?
2. Identify invariants: What constraints must this respect?
3. Define validation: Which SHACL shapes will catch violations?
4. Check for violations: Does this introduce anti-patterns?

### During Code Review
1. **Doctrine alignment**: Satisfies stated Q principle?
2. **Covenant validation**: SHACL validation passes?
3. **Violation check**: Any anti-patterns introduced?
4. **Measurement**: Right telemetry assertions in place?

### At Promotion/Release
1. **SHACL validation**: All shapes pass (zero violations)
2. **Chicago TDD**: Latency bounds verified (≤ 8 ticks hot path)
3. **Integration**: Full cycle O → Σ → μ → O' tested
4. **Knowledge**: Has the system learned from this change?

**CRITICAL GATE**: If ANY sh:Violation detected, workflow CANNOT deploy.

---

## INTEGRATION WITH DEVELOPMENT WORKFLOW

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

for file in $(git diff --cached --name-only | grep '\.ttl$'); do
    if [[ $file == ontology/workflows/* ]]; then
        ./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh "$file"
        if [ $? -eq 1 ]; then
            echo "❌ Workflow $file has Q invariant violations"
            echo "Covenant 2: Fix violations before committing"
            exit 1
        fi
    fi
done
```

### CI/CD Pipeline

```yaml
# .github/workflows/validate-workflows.yml
name: Validate YAWL Workflows

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install dependencies
        run: |
          sudo apt-get install -y raptor2-utils
          pip install pyshacl

      - name: Validate all workflows
        run: |
          for file in ontology/workflows/**/*.ttl; do
            ./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh "$file"
            if [ $? -eq 1 ]; then
              echo "❌ BLOCKING: Workflow has Q invariant violations"
              exit 1
            fi
          done
```

### Makefile Integration

```makefile
# Add to project Makefile

.PHONY: validate-shacl
validate-shacl:
	@echo "Validating SHACL constraints..."
	@for file in ontology/workflows/**/*.ttl; do \
		./ggen-marketplace/knhk-yawl-workflows/scripts/validate-shapes.sh $$file || exit 1; \
	done

.PHONY: test
test: validate-shacl test-chicago-v04 test-performance-v04
	@echo "All validations passed"
```

---

## CANONICAL REFERENCES

This SHACL validation layer is the **enforcement bridge** between:

- **DOCTRINE_2027.md** - Foundational Q principles (50 years → 2027)
- **DOCTRINE_COVENANT.md** - Covenant 2: Invariants Are Law
- **ontology/yawl-extended.ttl** - Complete YAWL ontology
- **ontology/yawl-pattern-permutations.ttl** - Valid pattern matrix
- **ontology/mape-k-autonomic.ttl** - MAPE-K feedback loops
- **CHATMAN_EQUATION_SPEC.md** - Formal Q3 definition (max_run_length ≤ 8)

**All code must satisfy all SHACL shapes. No exceptions.**

---

## VERSION HISTORY

| Version | Date | Change |
|---------|------|--------|
| 1.0.0 | 2025-11-16 | Initial SHACL validation layer implementation |

---

## WHAT SUCCESS LOOKS LIKE

### Before (Manual Quality Checking)
- ❌ Humans review code for Q violations (slow, error-prone)
- ❌ Quality issues found in production (too late)
- ❌ Inconsistent enforcement across reviewers
- ❌ No formal proof of compliance

### After (Automatic SHACL Validation)
- ✅ Machine validates Q constraints (instant, complete)
- ✅ Violations caught at definition time (shift left)
- ✅ 100% consistent enforcement (same rules everywhere)
- ✅ Formal proof via SHACL validation reports

---

## COVENANT 2 SUMMARY

**The Meta-Statement**:

For 50 years, conscientious program managers enforced quality manually. In 2027, quality enforcement runs at hardware speed via SHACL validators.

**What Changed**: Not the principles (Q invariants), but the **enforcement mechanism** (manual → automatic).

**The Result**: Quality is no longer optional. It is **law**, enforced by machines, proven by schemas.

---

**All code must satisfy all covenants. No exceptions.**
