# KNHK v1.0 Gap Analysis & Priority Matrix
## Comprehensive Requirements Analysis & Action Plan

**Version**: 1.0.0
**Date**: 2025-11-09
**Authority**: Hive Mind Review Agent
**Status**: OPERATIONAL GUIDE

---

## Executive Summary

**Current State**: KNHK is **24.2% compliant** with Definition of Done (8/33 criteria met)

**Gap Summary**:
- **25 criteria not met** (75.8% gaps)
- **4 critical blockers** preventing any progress
- **3 high-priority gaps** in validation infrastructure
- **18 medium-priority gaps** in testing and metrics

**Timeline Impact**:
- **Critical blockers**: 2-3 days to resolve
- **High-priority gaps**: 1-2 weeks to close
- **All gaps closed**: 4-8 weeks total

---

## 1. Critical Blockers (STOP-SHIP ISSUES)

**These issues MUST be fixed before ANY other work can proceed.**

### BLOCKER #1: Clippy Compilation Errors

**Impact**: Cannot compile with production settings (`-D warnings`)
**Affected Gate**: Gate 0 (Build & Quality)
**Current Status**: ❌ BLOCKING (15+ violations)

**Root Causes**:
1. Unused imports (multiple files)
2. Unexpected `cfg(profiling)` conditions (11 occurrences)
3. Documentation formatting issues
4. Dead code not marked with `#[allow(dead_code)]`

**Evidence**:
```rust
error: unused import: `crate::error::PipelineError`
  --> knhk-etl/src/pipeline.rs:9:5

error: unexpected `cfg` condition value: `profiling`
  (11 occurrences in codebase)

error: empty line after doc comment
error: field `max_capacity` is never read
error: doc list item without indentation
```

**Action Plan**:
1. Run `cargo clippy --workspace --fix` (automated fixes)
2. Manually review `cfg(profiling)` usage
3. Either enable feature in Cargo.toml or use `#[cfg_attr]`
4. Fix documentation formatting
5. Remove or `#[allow(dead_code)]` unused fields

**Estimated Effort**: 2-4 hours
**Owner**: Backend Developer Agent
**Priority**: 🔴 CRITICAL (Week 1, Day 1)

---

### BLOCKER #2: Chicago TDD Test Crash

**Impact**: Cannot validate core functionality
**Affected Gate**: Gate 3 (Traditional Testing)
**Current Status**: ❌ BLOCKING (Abort trap: 6)

**Root Cause**: Memory safety violation in lockchain receipt handling

**Evidence**:
```
[TEST] Lockchain Receipt Write
  ✓ Receipt written to lockchain
[TEST] Lockchain Receipt Read
[TEST] Lockchain Receipt Write
make[1]: *** [test-chicago-v04] Abort trap: 6
```

**Analysis**:
- Tests start successfully (CLI, Config pass 15/15)
- Crash occurs during Lockchain integration tests
- Likely segfault or use-after-free
- **This is the exact false positive we're designed to prevent!**

**Action Plan**:
1. Run with `RUST_BACKTRACE=full` to capture stack trace
2. Use AddressSanitizer: `RUSTFLAGS="-Z sanitizer=address" cargo test`
3. Use LeakSanitizer: `RUSTFLAGS="-Z sanitizer=leak" cargo test`
4. Debug memory safety in lockchain receipt handling
5. Fix root cause (likely FFI boundary issue or buffer overflow)
6. Verify all tests pass after fix

**Estimated Effort**: 4-8 hours
**Owner**: Code Analyzer Agent + Backend Developer Agent
**Priority**: 🔴 CRITICAL (Week 1, Day 1-2)

---

### BLOCKER #3: Integration Test Compilation Failures

**Impact**: Cannot validate cross-crate functionality
**Affected Gate**: Gate 3 (Traditional Testing)
**Current Status**: ❌ BLOCKING (compilation errors)

**Root Cause**: Tests out of sync with implementation after API refactoring

**Evidence**:
```rust
error[E0599]: no method named `execute_hooks_parallel` found for struct `Pipeline`
error[E0599]: no method named `execute_hooks_conditional` found for struct `Pipeline`
error[E0599]: no method named `execute_hooks_with_retry` found for struct `Pipeline`
```

**Analysis**:
- Tests reference methods that were removed or renamed
- Pipeline API was refactored (likely renamed to `execute_to_load`)
- Tests were not updated after refactoring
- **Classic false positive:** tests claim to validate API that no longer exists

**Action Plan**:
1. Review current `Pipeline` API in `knhk-etl/src/pipeline.rs`
2. Identify renamed/removed methods
3. Update tests to use current API
4. If methods were intentionally removed, update tests to reflect new behavior
5. Add regression test to prevent API/test mismatches

**Estimated Effort**: 1-2 hours
**Owner**: Code Analyzer Agent
**Priority**: 🔴 CRITICAL (Week 1, Day 2)

---

### BLOCKER #4: .unwrap()/.expect() in Production Code

**Impact**: Production code can panic instead of returning errors
**Affected Gate**: Gate 0 (Build & Quality)
**Current Status**: ❌ BLOCKING (71 files with violations)

**Root Cause**: Improper error handling patterns throughout codebase

**High-Risk Files** (HOT PATH - CRITICAL):
```
knhk-etl/src/hot_path_engine.rs  ← CRITICAL (hot path can panic)
knhk-etl/src/pipeline.rs          ← CRITICAL (core pipeline can panic)
knhk-otel/src/lib.rs              ← HIGH (telemetry infrastructure)
+ 68 other files
```

**Analysis**:
- KNHK exists to eliminate false positives
- `.unwrap()` creates potential panics that can mask errors
- Hot path with `.unwrap()` violates ≤8 tick performance guarantee
- Production code must use proper `Result<T, E>` error handling

**Action Plan** (Phased Approach):

**Phase 1: Hot Path (CRITICAL - 8 hours)**
1. Audit `knhk-etl/src/hot_path_engine.rs`
2. Audit `knhk-etl/src/pipeline.rs`
3. Replace all `.unwrap()` with proper error handling
4. Add error types where needed
5. Use `?` operator for error propagation

**Phase 2: Infrastructure (HIGH - 4 hours)**
1. Audit `knhk-otel/src/lib.rs`
2. Audit sidecar, warm path, CLI
3. Replace `.unwrap()` with error handling
4. Document acceptable `.expect()` patterns

**Phase 3: Remaining Files (MEDIUM - 8 hours)**
1. Audit remaining 65 files
2. Automated tool-assisted fixes where possible
3. Manual review of complex cases
4. Add `#![deny(clippy::unwrap_used)]` to all production crates

**Code Pattern**:
```rust
// ❌ WRONG - production code can panic
let result = operation.execute().unwrap();

// ✅ CORRECT - returns error to caller
let result = operation.execute()
    .map_err(|e| HotPathError::ExecutionFailed(e))?;
```

**Estimated Effort**: 16-20 hours (phased over Week 1-2)
**Owner**: System Architect Agent (design error types) + Backend Developer Agent (implement)
**Priority**: 🔴 CRITICAL (Week 1-2, phased)

---

## 2. High-Priority Gaps (MANDATORY FOR v1.0)

**These gaps MUST be closed for v1.0 release but can proceed after blockers are fixed.**

### GAP #5: Live Weaver Validation Not Run

**Impact**: Cannot prove features actually work (SOURCE OF TRUTH missing)
**Affected Gate**: Gate 1 (Weaver Validation - MANDATORY)
**Current Status**: ⚠️ NOT RUN (blocked by compilation errors)

**Analysis**:
- Static schema validation ✅ PASSES (6 schema files valid)
- Live runtime validation ❌ NOT RUN (application won't compile)
- **This is the ONLY validation that proves features work**
- All other tests are supporting evidence, not proof

**Why This is Critical**:
```
Traditional Testing (What We Replace):
  Test passes ✅ → Assumes feature works → FALSE POSITIVE
  └─ Test only validates test code, not production behavior

KNHK with Weaver Validation:
  Weaver validates schema ✅ → Telemetry proves feature works → TRUE POSITIVE
  └─ Schema validation proves actual runtime behavior matches specification
```

**Action Plan**:
1. Fix all blockers first (prerequisite)
2. Build application successfully
3. Set up OTLP collector (OpenTelemetry Collector)
4. Start application with telemetry enabled
5. Run `weaver registry live-check --registry registry/`
6. Verify all declared spans/metrics/logs appear in actual telemetry
7. Document any schema mismatches
8. Fix mismatches and re-validate

**Setup Commands**:
```bash
# Terminal 1: Start OTLP collector
docker run -p 4317:4317 -p 4318:4318 \
  otel/opentelemetry-collector:latest

# Terminal 2: Start application with telemetry
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
export RUST_LOG=trace
cargo run --bin knhk

# Terminal 3: Run live validation
weaver registry live-check --registry registry/
```

**Estimated Effort**: 2-4 hours (after blockers fixed)
**Owner**: Production Validator Agent
**Priority**: 🟡 HIGH (Week 2)
**Prerequisite**: Blockers #1-4 must be fixed

---

### GAP #6: Functional Validation Not Executed

**Impact**: Commands not proven to work with real arguments
**Affected Gate**: Gate 2 (Functional Validation - MANDATORY)
**Current Status**: ❌ NOT RUN (blocked by compilation errors)

**Analysis**:
- **Help text ≠ working feature**
- `--help` only proves command is registered in CLI
- Commands can have help text but call `unimplemented!()`
- MUST execute with real arguments to prove functionality

**The False Positive Paradox**:
```bash
# ❌ FALSE POSITIVE VALIDATION
knhk --help        # Returns help text
# ❌ CONCLUSION: "command works"  ← WRONG!
# ✅ REALITY: Help text exists, but command may call unimplemented!()

# ✅ CORRECT VALIDATION
knhk etl run --config test.yaml  # Actually execute the command
# Check: Does it produce expected output/behavior?
# Check: Does it emit proper telemetry?
# Check: Does Weaver validation pass?
```

**Action Plan**:
1. Fix all blockers first (prerequisite)
2. Create test configurations for each command
3. Execute each command with REAL arguments
4. Verify expected output/behavior
5. Verify telemetry emission (Weaver validation)
6. Test end-to-end workflows
7. Document results

**Commands to Validate**:
```bash
# ETL command
knhk etl run --config test_pipeline.yaml

# Hot path query
knhk hot query --ask "subject predicate object"

# Sidecar
knhk sidecar start --port 50051

# Workflow execution
knhk workflow execute --spec workflow.yaml

# Config management
knhk config validate --file config.toml
```

**Estimated Effort**: 4-6 hours (after blockers fixed)
**Owner**: Production Validator Agent + Code Analyzer Agent
**Priority**: 🟡 HIGH (Week 2)
**Prerequisite**: Blockers #1-4 and Gap #5 must be fixed

---

### GAP #7: Performance Validation Not Run

**Impact**: Cannot prove ≤8 tick constraint is met (DFLSS requirement)
**Affected Gate**: Gate 2 (Functional) + Gate 4 (DFLSS)
**Current Status**: ❌ NOT RUN (blocked by compilation errors)

**Analysis**:
- **Performance is a CTQ (Critical to Quality) metric**
- Hot path operations must be ≤8 ticks (Chatman Constant)
- This is a DFLSS hard requirement, not optional
- Must use PMU (Performance Monitoring Unit) for accurate measurement

**CTQ Target**:
```
Reconciliation time ≤ 8 ticks (measured with RDTSC)
- ASK query: ≤8 ticks
- COUNT query: ≤8 ticks
- Comparison operations: ≤8 ticks
- SIMD predicate matching: <5 ticks (W1 target)
```

**Action Plan**:
1. Fix all blockers first (prerequisite)
2. Run `make test-performance-v04`
3. Measure with PMU: `perf stat -e cycles,instructions ./bench`
4. Verify all hot path operations ≤8 ticks
5. If violations found, profile and optimize
6. Document actual tick counts
7. Create performance regression tests

**Measurement Commands**:
```bash
# Performance tests
make test-performance-v04

# PMU measurement
perf stat -e cycles,instructions,cache-references,cache-misses \
  ./target/release/knhk hot query --ask "s p o"

# Throughput benchmark
cargo bench --bench hot_path_throughput
```

**Estimated Effort**: 2-4 hours (measurement) + 8-16 hours (optimization if needed)
**Owner**: Performance Benchmarker Agent
**Priority**: 🟡 HIGH (Week 2)
**Prerequisite**: Blockers #1-4 must be fixed

---

## 3. Medium-Priority Gaps (Complete DoD Compliance)

**These gaps should be closed for v1.0 but are not blocking critical path.**

### Testing Infrastructure Gaps (5 gaps)

| Gap | Description | Effort | Week |
|-----|-------------|--------|------|
| #8 | Rust unit tests not passing | 2-4h | Week 2 |
| #9 | Test coverage <80% | 4-6h | Week 2 |
| #10 | Tests don't follow AAA pattern consistently | 2-3h | Week 2 |
| #11 | Missing edge case tests | 4-6h | Week 2-3 |
| #12 | Missing integration tests | 4-6h | Week 2-3 |

**Combined Estimated Effort**: 16-25 hours

---

### DFLSS Metrics Gaps (5 gaps)

| Gap | Description | Effort | Week |
|-----|-------------|--------|------|
| #13 | Idempotence not verified (μ∘μ = μ) | 2-3h | Week 3 |
| #14 | Provenance not verified (hash(A) = hash(μ(O))) | 2-3h | Week 3 |
| #15 | Sparsity not tracked (Δ size vs O size) | 2-3h | Week 3 |
| #16 | Drift not monitored (drift(A) → 0) | 2-3h | Week 3 |
| #17 | Throughput not measured (≥10⁷ rec/sec/core) | 1-2h | Week 3 |

**Combined Estimated Effort**: 9-14 hours

---

### Code Quality Gaps (4 gaps)

| Gap | Description | Effort | Week |
|-----|-------------|--------|------|
| #18 | Async trait methods (breaks dyn compatibility) | 2-4h | Week 1-2 |
| #19 | `println!` in production code | 1-2h | Week 1 |
| #20 | Fake `Ok(())` returns | 4-6h | Week 2 |
| #21 | Files >500 lines (modularity) | 4-6h | Week 2-3 |

**Combined Estimated Effort**: 11-18 hours

---

### Six Sigma Gaps (OPTIONAL for v1.0) (5 gaps)

| Gap | Description | Effort | Week |
|-----|-------------|--------|------|
| #22 | Cp not calculated (process capability) | 2-3h | Week 3-4 |
| #23 | Cpk not calculated (process capability index) | 2-3h | Week 3-4 |
| #24 | DPMO not tracked (defects per million) | 2-3h | Week 3-4 |
| #25 | FPY not tracked (first pass yield) | 1-2h | Week 3-4 |
| #26 | PCE not tracked (process cycle efficiency) | 1-2h | Week 3-4 |

**Combined Estimated Effort**: 8-13 hours

**Note**: Six Sigma metrics are OPTIONAL for v1.0, but baseline data should be collected for v1.1.

---

## 4. Overlap Analysis

**Identified Overlaps** (opportunities for efficiency):

### Overlap #1: Error Handling + Code Quality
- Fixing `.unwrap()/.expect()` (Blocker #4) also addresses:
  - Proper `Result<T, E>` error handling (Gap #20)
  - Fake `Ok(())` returns (Gap #20)
- **Efficiency Gain**: Fix once, satisfy 3 criteria

### Overlap #2: Weaver Validation + Functional Validation
- Running live Weaver validation (Gap #5) also validates:
  - Commands emit proper telemetry (Gap #6)
  - End-to-end workflows (Gap #6)
- **Efficiency Gain**: Single validation run, dual proof

### Overlap #3: Performance Tests + DFLSS Metrics
- Running performance tests (Gap #7) collects data for:
  - Reconciliation time (Gap #13)
  - Throughput measurement (Gap #17)
- **Efficiency Gain**: Single test run, multiple metrics

### Overlap #4: Test Infrastructure + Integration Tests
- Fixing integration test compilation (Blocker #3) enables:
  - Integration test coverage (Gap #12)
  - Cross-crate validation (Gap #11)
- **Efficiency Gain**: Fix once, enable entire test suite

**Total Efficiency Gain**: ~15-20 hours saved through overlap optimization

---

## 5. Priority Matrix & Timeline

### Week 1: Critical Blockers (30-40 hours)

**Day 1-2: Build & Compilation**
- [ ] Blocker #1: Fix clippy errors (2-4h) 🔴
- [ ] Blocker #3: Fix integration test compilation (1-2h) 🔴
- [ ] Gap #19: Remove `println!` from production code (1-2h)

**Day 3-4: Memory Safety & Error Handling**
- [ ] Blocker #2: Debug and fix Chicago TDD crash (4-8h) 🔴
- [ ] Blocker #4 Phase 1: Fix hot path `.unwrap()` (8h) 🔴
- [ ] Gap #18: Fix async trait methods (2-4h)

**Day 5: Validation**
- [ ] Run all tests to verify blockers fixed
- [ ] Prepare for Week 2 validation work

**Week 1 Deliverable**: Compilation succeeds, tests pass, critical blockers resolved

---

### Week 2: Validation & Testing (30-40 hours)

**Day 1-2: Weaver Validation (MANDATORY)**
- [ ] Gap #5: Run live Weaver validation (2-4h) 🟡
- [ ] Gap #6: Execute functional validation (4-6h) 🟡

**Day 3-4: Performance & Error Handling**
- [ ] Gap #7: Run performance tests, verify ≤8 ticks (2-4h) 🟡
- [ ] Blocker #4 Phase 2: Fix infrastructure `.unwrap()` (4h) 🔴

**Day 5: Testing Infrastructure**
- [ ] Gap #8: Fix failing Rust unit tests (2-4h)
- [ ] Gap #9: Improve test coverage to 80% (4-6h)
- [ ] Gap #10: Refactor tests to AAA pattern (2-3h)

**Week 2 Deliverable**: All MANDATORY validations complete, test suite passing

---

### Week 3: DFLSS Metrics & Quality (20-30 hours)

**Day 1-2: DFLSS Metrics**
- [ ] Gap #13: Verify idempotence (μ∘μ = μ) (2-3h)
- [ ] Gap #14: Verify provenance (hash equality) (2-3h)
- [ ] Gap #15: Track sparsity (Δ vs O size) (2-3h)
- [ ] Gap #16: Monitor drift (cumulative error) (2-3h)
- [ ] Gap #17: Measure throughput (1-2h)

**Day 3-4: Code Quality**
- [ ] Blocker #4 Phase 3: Fix remaining `.unwrap()` (8h) 🔴
- [ ] Gap #20: Audit fake `Ok(())` returns (4-6h)
- [ ] Gap #21: Refactor large files (4-6h)

**Day 5: Testing Completion**
- [ ] Gap #11: Add edge case tests (4-6h)
- [ ] Gap #12: Complete integration tests (4-6h)

**Week 3 Deliverable**: DFLSS metrics collected, code quality goals met

---

### Week 4: Six Sigma Baseline & Certification (10-20 hours)

**Day 1-2: Six Sigma Metrics (OPTIONAL)**
- [ ] Gap #22: Calculate Cp (2-3h)
- [ ] Gap #23: Calculate Cpk (2-3h)
- [ ] Gap #24: Track DPMO (2-3h)
- [ ] Gap #25: Track FPY (1-2h)
- [ ] Gap #26: Track PCE (1-2h)

**Day 3-4: Final Validation**
- [ ] Re-run all validation gates
- [ ] Collect all evidence artifacts
- [ ] Review for any missed gaps

**Day 5: Certification**
- [ ] Complete release certification checklist
- [ ] Obtain signatures
- [ ] Archive evidence
- [ ] Prepare release artifacts

**Week 4 Deliverable**: v1.0 certified for production release

---

## 6. Risk Mitigation

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Memory safety bugs (Blocker #2) | HIGH | CRITICAL | AddressSanitizer, LeakSanitizer, manual code review |
| Performance regressions | MEDIUM | HIGH | Continuous PMU monitoring, regression tests |
| Weaver validation failures | MEDIUM | CRITICAL | Schema review, telemetry audit, iterative fixes |
| API/test mismatches | LOW | MEDIUM | Automated API change detection, test sync checks |
| .unwrap() audit incomplete | MEDIUM | HIGH | Automated grep, clippy deny rules, phased approach |

### Schedule Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Blocker #2 takes >8h | MEDIUM | HIGH | Allocate buffer time, engage multiple agents |
| Weaver live-check reveals major issues | LOW | CRITICAL | Schema pre-review, incremental validation |
| Performance optimization needed | MEDIUM | HIGH | Profile early, optimize hot path first |
| Six Sigma metrics too complex | LOW | LOW | Optional for v1.0, defer to v1.1 if needed |

---

## 7. Success Criteria

### Minimum Viable Release (v1.0)

**Gates 0-4 MUST be 100% complete:**
- ✅ 28/28 mandatory criteria met
- ✅ Zero critical blockers
- ✅ Weaver live-check passes
- ✅ Performance ≤8 ticks verified
- ✅ All tests passing

**Gate 5 (Six Sigma) OPTIONAL:**
- Baseline data collected
- Analysis deferred to v1.1

### Optimal Release (v1.0+)

**All 33 criteria met:**
- ✅ Gates 0-5 100% complete
- ✅ Six Sigma metrics calculated
- ✅ Control charts established
- ✅ Continuous monitoring in place

---

## 8. Resource Allocation

### Agent Assignments

| Agent | Primary Gaps | Estimated Hours |
|-------|-------------|-----------------|
| Backend Developer | Blockers #1, #2, #4 | 20-30h |
| Code Analyzer | Blocker #3, Gap #20 | 10-15h |
| Production Validator | Gaps #5, #6 | 6-10h |
| Performance Benchmarker | Gap #7, #13-17 | 8-12h |
| System Architect | Blocker #4 (error types), Gap #18 | 8-12h |
| TDD London Swarm | Gaps #8-12 | 12-18h |
| CI/CD Engineer | Gaps #22-26 (automation) | 6-10h |

**Total Estimated Effort**: 70-107 hours
**With Overlaps**: 55-87 hours
**With Parallel Work**: 4-8 weeks calendar time

---

## 9. Actionable Next Steps

### Immediate Actions (This Week)

1. **Assign Blocker #1 to Backend Developer Agent**
   - Fix clippy errors
   - Target: 2-4 hours

2. **Assign Blocker #2 to Code Analyzer + Backend Developer**
   - Debug Chicago TDD crash
   - Use AddressSanitizer
   - Target: 4-8 hours

3. **Assign Blocker #3 to Code Analyzer Agent**
   - Fix integration test compilation
   - Update test API calls
   - Target: 1-2 hours

4. **Assign Blocker #4 Phase 1 to System Architect + Backend Developer**
   - Design error handling strategy
   - Fix hot path `.unwrap()`
   - Target: 8 hours

### Week 2 Preparation

1. **Setup OTLP Collector** (DevOps)
   - Install and configure
   - Test telemetry pipeline
   - Target: 1-2 hours

2. **Create Test Configurations** (Production Validator)
   - Prepare functional test cases
   - Create test data
   - Target: 2-3 hours

3. **Setup Performance Monitoring** (Performance Benchmarker)
   - Configure PMU access
   - Prepare benchmark suite
   - Target: 1-2 hours

---

## 10. Document Control

**Version**: 1.0.0
**Status**: OPERATIONAL
**Location**: `/Users/sac/knhk/docs/V1_GAPS_AND_PRIORITIES.md`
**Related Documents**:
- `/Users/sac/knhk/docs/V1_DEFINITION_OF_DONE.md` (authoritative DoD)
- `/Users/sac/knhk/docs/V1_RELEASE_CERTIFICATION_CHECKLIST.md` (operational checklist)

**Review Cycle**: Weekly during v1.0 development

**Change Log**:
- 2025-11-09: v1.0.0 - Initial gap analysis and priority matrix

---

**This document provides the tactical plan to close all gaps and achieve v1.0 production readiness.**

**Review and update weekly as gaps are closed and priorities shift.**
