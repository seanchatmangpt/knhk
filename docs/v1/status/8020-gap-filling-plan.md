# 80/20 Gap Filling Plan

**Date**: 2025-11-08  
**Principle**: Focus on the critical 20% of gaps that provide 80% of value  
**Total Effort**: ~31-48 hours (vs 70-107 hours for all gaps)

---

## Executive Summary

**80/20 Strategy**: Fix the 4 critical blockers + 3 high-priority validation gaps = **80% of production readiness value**

**Gaps Included** (7 total):
- ✅ 4 Critical Blockers (must fix before anything else works)
- ✅ 3 High-Priority Validation Gaps (prove the system actually works)

**Gaps Deferred** (18 total):
- ⏸️ Medium-priority testing gaps (can be done incrementally)
- ⏸️ Medium-priority code quality gaps (non-blocking)
- ⏸️ Six Sigma metrics (optional for v1.0)

**Value Delivered**:
- ✅ System compiles and runs
- ✅ Core functionality validated (Weaver)
- ✅ Performance verified (≤8 ticks)
- ✅ Production-ready error handling (hot path)

---

## Phase 1: Critical Blockers (23-34 hours)

**Goal**: Unblock all other work

### Day 1: Build & Compilation (3-6 hours)

#### BLOCKER #1: Clippy Compilation Errors (2-4h)
**Impact**: Cannot compile with production settings  
**Effort**: 2-4 hours  
**Priority**: 🔴 CRITICAL

**Actions**:
1. Run `cargo clippy --workspace --fix` (automated fixes)
2. Fix `cfg(profiling)` conditions (11 occurrences)
   - Either enable feature in Cargo.toml or use `#[cfg_attr]`
3. Fix documentation formatting issues
4. Remove or `#[allow(dead_code)]` unused fields

**Success Criteria**: `cargo clippy --workspace -D warnings` passes

#### BLOCKER #3: Integration Test Compilation (1-2h)
**Impact**: Cannot validate cross-crate functionality  
**Effort**: 1-2 hours  
**Priority**: 🔴 CRITICAL

**Actions**:
1. Review current `Pipeline` API in `knhk-etl/src/pipeline.rs`
2. Update tests to use current API (methods renamed/removed)
3. If methods intentionally removed, update tests to reflect new behavior

**Success Criteria**: Integration tests compile and run

---

### Day 2-3: Memory Safety & Error Handling (12-16 hours)

#### BLOCKER #2: Chicago TDD Test Crash (4-8h)
**Impact**: Cannot validate core functionality  
**Effort**: 4-8 hours  
**Priority**: 🔴 CRITICAL

**Actions**:
1. Run with `RUST_BACKTRACE=full` to capture stack trace
2. Use AddressSanitizer: `RUSTFLAGS="-Z sanitizer=address" cargo test`
3. Use LeakSanitizer: `RUSTFLAGS="-Z sanitizer=leak" cargo test`
4. Debug memory safety in lockchain receipt handling
5. Fix root cause (likely FFI boundary issue or buffer overflow)
6. Verify all tests pass after fix

**Success Criteria**: All Chicago TDD tests pass without crashes

#### BLOCKER #4 Phase 1: Hot Path `.unwrap()` (8h)
**Impact**: Hot path can panic in production  
**Effort**: 8 hours  
**Priority**: 🔴 CRITICAL

**Focus Files**:
- `knhk-etl/src/hot_path_engine.rs` ← CRITICAL
- `knhk-etl/src/pipeline.rs` ← CRITICAL

**Actions**:
1. Audit hot path files for `.unwrap()` and `.expect()`
2. Replace with proper `Result<T, E>` error handling
3. Add error types where needed
4. Use `?` operator for error propagation

**Code Pattern**:
```rust
// ❌ WRONG
let result = operation.execute().unwrap();

// ✅ CORRECT
let result = operation.execute()
    .map_err(|e| HotPathError::ExecutionFailed(e))?;
```

**Success Criteria**: Hot path has zero `.unwrap()` or `.expect()` calls

---

## Phase 2: High-Priority Validation (8-14 hours)

**Goal**: Prove the system actually works (source of truth)

### Day 4-5: Weaver Validation (6-10 hours)

#### GAP #5: Live Weaver Validation (2-4h)
**Impact**: Cannot prove features actually work  
**Effort**: 2-4 hours  
**Priority**: 🟡 HIGH (MANDATORY)

**Why Critical**: This is the ONLY validation that proves features work. All other tests are supporting evidence.

**Actions**:
1. Fix all blockers first (prerequisite)
2. Build application successfully
3. Set up OTLP collector:
   ```bash
   docker run -p 4317:4317 -p 4318:4318 \
     otel/opentelemetry-collector:latest
   ```
4. Start application with telemetry:
   ```bash
   export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
   export RUST_LOG=trace
   cargo run --bin knhk
   ```
5. Run live validation:
   ```bash
   weaver registry live-check --registry registry/
   ```
6. Verify all declared spans/metrics/logs appear
7. Fix any schema mismatches

**Success Criteria**: `weaver registry live-check` passes with zero errors

#### GAP #6: Functional Validation (4-6h)
**Impact**: Commands not proven to work with real arguments  
**Effort**: 4-6 hours  
**Priority**: 🟡 HIGH (MANDATORY)

**The False Positive Paradox**: Help text ≠ working feature

**Actions**:
1. Create test configurations for each command
2. Execute each command with REAL arguments:
   ```bash
   knhk etl run --config test_pipeline.yaml
   knhk hot query --ask "subject predicate object"
   knhk sidecar start --port 50051
   knhk workflow execute --spec workflow.yaml
   knhk config validate --file config.toml
   ```
3. Verify expected output/behavior
4. Verify telemetry emission (Weaver validation)
5. Test end-to-end workflows

**Success Criteria**: All commands execute successfully with real arguments and emit proper telemetry

---

### Day 6: Performance Validation (2-4 hours)

#### GAP #7: Performance Validation (2-4h)
**Impact**: Cannot prove ≤8 tick constraint is met  
**Effort**: 2-4 hours  
**Priority**: 🟡 HIGH (DFLSS requirement)

**CTQ Target**: Hot path operations ≤8 ticks (Chatman Constant)

**Actions**:
1. Run `make test-performance-v04`
2. Measure with PMU:
   ```bash
   perf stat -e cycles,instructions,cache-references,cache-misses \
     ./target/release/knhk hot query --ask "s p o"
   ```
3. Verify all hot path operations ≤8 ticks
4. If violations found, profile and optimize
5. Document actual tick counts
6. Create performance regression tests

**Success Criteria**: All hot path operations measured ≤8 ticks

---

## 80/20 Value Analysis

### What We're Fixing (20% of gaps = 80% of value)

| Gap | Value | Effort | ROI |
|-----|-------|--------|-----|
| Blocker #1 (Clippy) | Unblocks compilation | 2-4h | 🔥 CRITICAL |
| Blocker #3 (Integration) | Unblocks testing | 1-2h | 🔥 CRITICAL |
| Blocker #2 (Crash) | Unblocks validation | 4-8h | 🔥 CRITICAL |
| Blocker #4 (Hot Path) | Production safety | 8h | 🔥 CRITICAL |
| Gap #5 (Weaver) | Source of truth | 2-4h | 🔥 CRITICAL |
| Gap #6 (Functional) | Proves features work | 4-6h | 🔥 CRITICAL |
| Gap #7 (Performance) | DFLSS requirement | 2-4h | 🔥 CRITICAL |
| **TOTAL** | **80% value** | **23-34h** | **🔥🔥🔥** |

### What We're Deferring (80% of gaps = 20% of value)

| Category | Gaps | Effort | Value | Defer Reason |
|----------|------|--------|-------|--------------|
| Testing Infrastructure | #8-12 | 16-25h | Medium | Can be done incrementally |
| DFLSS Metrics | #13-17 | 9-14h | Medium | Nice-to-have, not blocking |
| Code Quality | #18-21 | 11-18h | Medium | Non-blocking improvements |
| Six Sigma | #22-26 | 8-13h | Low | Optional for v1.0 |
| **TOTAL** | **18 gaps** | **44-70h** | **20%** | **Can defer** |

---

## Success Criteria

### Minimum Viable Release (80/20 Complete)

**All Critical Blockers Fixed**:
- ✅ Compilation succeeds (`cargo clippy -D warnings` passes)
- ✅ Integration tests compile and run
- ✅ Chicago TDD tests pass without crashes
- ✅ Hot path has proper error handling (no `.unwrap()`)

**All High-Priority Validation Complete**:
- ✅ Weaver live-check passes (source of truth)
- ✅ Functional validation passes (commands work)
- ✅ Performance ≤8 ticks verified (DFLSS requirement)

**Result**: **Production-ready system** with 80% of DoD value delivered in ~31-48 hours vs 70-107 hours for 100%.

---

## Timeline

**Week 1**: Critical Blockers (23-34 hours)
- Day 1: Build & Compilation (3-6h)
- Day 2-3: Memory Safety & Error Handling (12-16h)
- Day 4-5: Validation (6-10h)
- Day 6: Performance (2-4h)

**Total**: 6 days for 80% value

---

## Next Steps After 80/20 Complete

Once the critical 20% is done, you can:
1. **Ship v1.0** with confidence (core functionality proven)
2. **Incrementally add** medium-priority gaps (testing, metrics)
3. **Iterate** based on real-world usage

**The 80/20 approach gets you to production faster while maintaining quality.**

