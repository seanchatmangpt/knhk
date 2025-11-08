# System Completion Architecture Design

**Date**: January 2025
**Agent**: System Architect #5
**Status**: Architecture Design Complete

## Executive Summary

This document designs the completion strategy for the KNHK system based on the 3-tier validation hierarchy: Weaver → Build → Tests. The architecture identifies critical dependencies, execution order, and strategies for achieving full system validation.

## Current State Analysis

### Level 1: Weaver Schema Validation ✅ PASSING
```bash
weaver registry check -r registry/
✔ `knhk` semconv registry `registry/` loaded (5 files)
✔ No `before_resolution` policy violation
✔ `knhk` semconv registry resolved
✔ No `after_resolution` policy violation
```

**Status**: SOURCE OF TRUTH VALIDATED
- 5 schema files loaded and validated
- No policy violations
- Registry fully resolved
- **This is the foundation - telemetry schema is correct**

### Level 2: Compilation Status ⚠️ BLOCKED (3 errors)
```rust
error[E0463]: can't find crate for `knhk_otel`
error[E0463]: can't find crate for `knhk_lockchain`
error[E0432]: unresolved import `knhk_otel`
```

**Root Cause**: Dependency resolution issue in `knhk-etl` crate
- `knhk-etl` depends on `knhk-otel` and `knhk-lockchain`
- Crates exist but are not resolvable in build graph
- **CRITICAL**: Blocks all downstream testing

**Warnings** (non-blocking):
- 7 warnings in `knhk-connectors` (unused imports, mutability)
- 6 warnings in `knhk-hot` (naming conventions)
- 1 warning in `knhk-otel` (unused import)

### Level 3: Test Execution Status 🔴 BLOCKED BY LEVEL 2
```makefile
# C Tests (Chicago TDD) - Ready but need Makefile path fix
make test-chicago-v04     # Missing tests/chicago_v04_test.c (path issue)
make test-performance-v04 # Blocked by above
make test-integration-v2  # Blocked by above

# Rust Tests - Blocked by compilation errors
cargo test --workspace    # Blocked by knhk-etl compilation
```

**C Test Issues**:
- Test files exist in `/Users/sac/knhk/tests/` (not `c/tests/`)
- Makefile expects `tests/chicago_v04_test.c` but file is `../tests/chicago_v04_test.c`
- **Simple path fix required**

## Dependency Graph

```
Level 1 (Weaver Schema)
    ✅ registry/*.yaml
    │
    ├─> PASSES: Schema validated
    │
    └─> ENABLES: Level 2 validation

Level 2 (Compilation)
    ⚠️ Rust Crates
    │   ├─> knhk-otel ✅ (compiles)
    │   ├─> knhk-lockchain ✅ (compiles)
    │   ├─> knhk-connectors ✅ (compiles with warnings)
    │   ├─> knhk-hot ✅ (compiles with warnings)
    │   └─> knhk-etl 🔴 (BLOCKED: can't find knhk_otel, knhk_lockchain)
    │
    ⚠️ C Library
    │   ├─> libknhk.a ✅ (compiles)
    │   └─> Test binaries 🔴 (BLOCKED: path mismatch)
    │
    └─> BLOCKS: Level 3 execution

Level 3 (Test Execution)
    🔴 Chicago TDD Tests (C)
    │   ├─> test-chicago-v04 🔴 (BLOCKED: Makefile path)
    │   ├─> test-performance-v04 🔴 (BLOCKED: Makefile path)
    │   └─> test-integration-v2 🔴 (BLOCKED: Makefile path)
    │
    🔴 Rust Tests
    │   ├─> cargo test --workspace 🔴 (BLOCKED: knhk-etl compilation)
    │   └─> Individual crate tests ⚠️ (partial: some crates pass)
    │
    └─> VALIDATES: System behavior
```

## Critical Path Analysis

### Blocking Issues (Must Fix First)

#### 1. Rust Dependency Resolution (CRITICAL - P0)
**Problem**: `knhk-etl` cannot find `knhk_otel` and `knhk_lockchain` crates

**Root Cause Hypotheses**:
1. Missing workspace-level `Cargo.toml` defining all crates
2. Incorrect crate names in dependencies (underscore vs hyphen)
3. Missing `path` directives in `Cargo.toml` dependencies
4. Circular dependency between crates

**Resolution Strategy**:
```rust
// Option A: Fix knhk-etl/Cargo.toml dependencies
[dependencies]
knhk-otel = { path = "../knhk-otel" }        # Use hyphen
knhk-lockchain = { path = "../knhk-lockchain" }

// Option B: Create workspace Cargo.toml
[workspace]
members = [
    "knhk-etl",
    "knhk-otel",
    "knhk-lockchain",
    "knhk-connectors",
    "knhk-hot",
    "knhk-warm",
    "knhk-sidecar",
    "knhk-cli",
    "knhk-aot",
    "knhk-unrdf",
]
```

**Validation**:
```bash
cd /Users/sac/knhk/rust/knhk-etl
cargo build  # Should complete without errors
```

#### 2. C Test Path Resolution (CRITICAL - P0)
**Problem**: Makefile looks for `tests/chicago_v04_test.c` but files are in `../tests/`

**Resolution Strategy**:
```makefile
# Current (broken)
TEST_V04 = ../tests/chicago_v04_test
$(TEST_V04): tests/chicago_v04_test.c ...  # Wrong path

# Fixed
TEST_V04 = ../tests/chicago_v04_test
$(TEST_V04): ../tests/chicago_v04_test.c ...  # Correct path
```

**Validation**:
```bash
cd /Users/sac/knhk/c
make test-chicago-v04  # Should build and run
```

### Non-Blocking Issues (Fix After Critical)

#### 3. Rust Warnings (P1 - Style)
- 7 warnings in `knhk-connectors` (unused imports, mutability)
- 6 warnings in `knhk-hot` (naming conventions)
- 1 warning in `knhk-otel` (unused import)

**Resolution**: Run `cargo fix` and `cargo clippy --fix`

## Execution Strategy

### Phase 1: Fix Compilation (Level 2)
**Goal**: Achieve `cargo build --workspace` success with zero errors

**Steps**:
1. Analyze `rust/knhk-etl/Cargo.toml` dependencies
2. Identify missing/incorrect crate references
3. Fix dependency declarations (path, name, version)
4. Verify `knhk-otel` and `knhk-lockchain` are buildable individually
5. Create workspace `Cargo.toml` if needed
6. Run `cargo build --workspace` to verify

**Success Criteria**:
```bash
cd /Users/sac/knhk/rust
cargo build --workspace
# Output: "Finished" with 0 errors (warnings acceptable)
```

### Phase 2: Fix C Test Paths (Level 2)
**Goal**: Make `make test-chicago-v04` buildable

**Steps**:
1. Analyze `c/Makefile` test target paths
2. Update paths to reference `../tests/` instead of `tests/`
3. Verify test file locations match expected paths
4. Run `make test-chicago-v04` to verify build

**Success Criteria**:
```bash
cd /Users/sac/knhk/c
make test-chicago-v04
# Output: Test binary created successfully
```

### Phase 3: Execute Tests (Level 3)
**Goal**: Run all test suites and verify passing status

**Order of Execution**:
```bash
# 1. Chicago TDD Tests (fundamental behavior)
cd /Users/sac/knhk/c
make test-chicago-v04        # Comprehensive TDD suite
# Expected: All tests pass (behavior validated)

# 2. Performance Tests (≤8 ticks constraint)
make test-performance-v04    # Hot path performance
# Expected: All operations ≤8 ticks (except CONSTRUCT8, which is documented)

# 3. Integration Tests (cross-component)
make test-integration-v2     # System integration
# Expected: All integration flows work

# 4. Rust Tests (component validation)
cd /Users/sac/knhk/rust
cargo test --workspace       # All Rust components
# Expected: All tests pass (component behavior validated)
```

### Phase 4: Weaver Live Validation (FINAL)
**Goal**: Validate runtime telemetry matches schema

**Strategy**:
```bash
# 1. Start Weaver live-check
weaver registry live-check \
    --registry registry/ \
    --otlp-grpc-port 4317 \
    --admin-port 8080 \
    --inactivity-timeout 3600

# 2. Run system with OTEL enabled
export KGC_SIDECAR_WEAVER_ENABLED=true
export KGC_SIDECAR_OTEL_ENABLED=true

# 3. Execute operations to generate telemetry
# (Run tests, CLI commands, API calls)

# 4. Verify Weaver reports no violations
# Check: http://localhost:8080/status
# Check: Weaver output logs
```

**Success Criteria**:
- Weaver reports no semantic convention violations
- All spans/metrics/logs match schema definitions
- Runtime telemetry proves features work (not just tests)

## Architectural Patterns

### 1. Dependency Inversion for Testing
```
High-Level Policy (Tests)
    │
    └─> Abstraction (Traits)
          │
          └─> Implementation (Concrete Types)
```

**Benefit**: Tests validate behavior through abstractions, not implementation details

### 2. Schema-First Validation
```
Schema Definition (registry/*.yaml)
    │
    ├─> Code Generation (build.rs)
    │   └─> Telemetry Types
    │
    ├─> Runtime Validation (Weaver)
    │   └─> Telemetry Conformance
    │
    └─> Test Validation (Chicago TDD)
        └─> Behavior Verification
```

**Benefit**: Schema is source of truth, code must conform

### 3. Layered Testing Strategy
```
Layer 1: Unit Tests (Component Behavior)
    │
    └─> Layer 2: Integration Tests (Cross-Component)
          │
          └─> Layer 3: Performance Tests (SLO Validation)
                │
                └─> Layer 4: Weaver Validation (Telemetry Proof)
```

**Benefit**: Each layer validates different aspects, building confidence

## Risk Assessment

### High Risk (Immediate Attention)

#### Risk 1: Circular Dependencies in Rust Crates
**Likelihood**: Medium
**Impact**: High (blocks all progress)

**Mitigation**:
- Analyze dependency graph with `cargo tree`
- Identify circular dependencies
- Break cycles by introducing trait abstractions
- Move shared types to a `knhk-core` crate

#### Risk 2: Missing Workspace Configuration
**Likelihood**: High
**Impact**: High (prevents workspace-level operations)

**Mitigation**:
- Create workspace `Cargo.toml` at `/Users/sac/knhk/rust/Cargo.toml`
- Define all crates as workspace members
- Use workspace dependencies for shared crates

### Medium Risk (Monitor)

#### Risk 3: Test File Organization Mismatch
**Likelihood**: Medium
**Impact**: Medium (confusing, but fixable)

**Mitigation**:
- Document expected test file locations
- Update Makefile to use consistent paths
- Consider moving all tests to `c/tests/` or symlink

#### Risk 4: Weaver Process Instability
**Likelihood**: Low
**Impact**: High (blocks final validation)

**Mitigation**:
- Use `verify-weaver.sh` script before validation
- Implement health check monitoring
- Have restart strategy ready (see WEAVER_INTEGRATION.md)

### Low Risk (Acceptable)

#### Risk 5: Compiler Warnings
**Likelihood**: High (already present)
**Impact**: Low (non-blocking)

**Mitigation**:
- Run `cargo fix` and `cargo clippy --fix`
- Address after critical issues fixed

## Integration Patterns

### Pattern 1: C → Rust FFI
```c
// C calls Rust
extern void knhk_construct8_cold(const char* query, char** result);

// Rust provides FFI
#[no_mangle]
pub extern "C" fn knhk_construct8_cold(query: *const c_char, result: *mut *mut c_char) {
    // Cold path CONSTRUCT8 via unrdf
}
```

**Validation**: Test with `chicago_construct8_pipeline.c`

### Pattern 2: Sidecar gRPC Integration
```
Client → gRPC → Sidecar → KGC Backend
   │              │
   └─> OTEL ──> Weaver (validation)
```

**Validation**: `knhk-sidecar/tests/chicago_tdd_capabilities.rs`

### Pattern 3: ETL Pipeline
```
Connector → Ingest → Reflex → Emit → Destination
    │          │        │       │
    └─> OTEL spans prove each stage works
```

**Validation**: `make test-etl` + Weaver live-check

## Completion Checklist

### Level 1: Weaver Schema ✅
- [x] Registry files exist and are valid
- [x] `weaver registry check` passes
- [x] Schema documents all telemetry
- [x] No policy violations

### Level 2: Compilation ⚠️
- [ ] Fix `knhk-etl` dependency resolution
- [ ] All Rust crates compile (`cargo build --workspace`)
- [ ] Fix C test Makefile paths
- [ ] All C tests build (`make test-chicago-v04`)
- [ ] Run `cargo clippy --workspace -- -D warnings` (zero warnings)
- [ ] Run `cargo fmt --all` (consistent formatting)

### Level 3: Test Execution 🔴
- [ ] `make test-chicago-v04` passes (all TDD tests)
- [ ] `make test-performance-v04` passes (≤8 ticks verified)
- [ ] `make test-integration-v2` passes (integration validated)
- [ ] `cargo test --workspace` passes (all Rust tests)
- [ ] All tests achieve 100% pass rate

### Level 4: Weaver Live Validation 🔴
- [ ] Start Weaver live-check successfully
- [ ] Run system with OTEL enabled
- [ ] Generate telemetry from operations
- [ ] Verify no semantic convention violations
- [ ] Confirm runtime telemetry matches schema

## Success Metrics

### Quantitative Metrics
- **Compilation**: 0 errors, ≤10 warnings
- **Test Pass Rate**: 100% (all tests pass)
- **Performance**: Hot path ≤8 ticks (except CONSTRUCT8)
- **Coverage**: ≥80% code coverage (Chicago TDD principles)
- **Weaver Violations**: 0 (all telemetry conforms to schema)

### Qualitative Metrics
- **Documentation Accuracy**: Claims match implementation
- **Error Messages**: Clear and actionable
- **Test Clarity**: Tests document expected behavior
- **Architecture**: Clean separation of concerns

## Recommendations

### Immediate Actions (Next 2 Hours)
1. **Fix Rust dependency resolution** (Agent #1: Investigator)
   - Analyze `knhk-etl/Cargo.toml`
   - Create workspace `Cargo.toml` if needed
   - Verify `cargo build --workspace` succeeds

2. **Fix C Makefile paths** (Agent #2: Builder)
   - Update test target paths to `../tests/`
   - Verify `make test-chicago-v04` builds

### Short-Term Actions (Next 4 Hours)
3. **Run test suites** (Agent #3: Validator)
   - Execute Chicago TDD tests
   - Execute performance tests
   - Execute integration tests
   - Document any failures

4. **Fix test failures** (Agent #4: Debugger)
   - Investigate any test failures
   - Apply minimal fixes (no scope creep)
   - Re-run until 100% pass rate

### Medium-Term Actions (Next 8 Hours)
5. **Weaver live validation** (Agent #5: System Architect)
   - Start Weaver live-check
   - Run full system with OTEL
   - Verify telemetry conformance
   - Document any violations

6. **Final validation** (Agent #6: Production Validator)
   - Run full validation suite
   - Verify all 3 tiers pass
   - Generate completion report
   - Tag release candidate

## Conclusion

The KNHK system is architecturally sound with a validated telemetry schema (Level 1 ✅). The critical blockers are:

1. **Rust dependency resolution** (knhk-etl compilation)
2. **C Makefile path fixes** (test building)

Once these are resolved, the system can progress through the testing hierarchy (Level 3) and final Weaver live validation (Level 4).

The 3-tier validation hierarchy ensures:
- **Schema correctness** (Weaver) = What telemetry SHOULD be emitted
- **Code correctness** (Compilation) = Code is syntactically valid
- **Behavior correctness** (Tests) = Code does what it claims
- **Runtime correctness** (Weaver live-check) = Telemetry proves features work

**Estimated Time to Completion**: 4-8 hours (assuming no major architectural issues)

**Next Agent**: Agent #1 (Dependency Investigator) to analyze and fix Rust crate dependencies.
