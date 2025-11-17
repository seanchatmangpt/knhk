# Production Readiness Fix Checklist
**Target**: Production Ready Status
**Timeline**: 3-5 business days (12-20 hours)
**Start Date**: TBD (after decision)

---

## 🚨 BLOCKING DECISION REQUIRED

### Decision: Unsafe Code Policy

**Current State**: Release builds FORBID unsafe code, but 8-tick guarantee REQUIRES unsafe code

**Options:**

- [ ] **Option A: Allow Unsafe (Recommended)**
  - Change `#![forbid(unsafe_code)]` to `#![warn(unsafe_code)]`
  - Require safety proofs for all unsafe blocks
  - Document safety invariants
  - **Effort**: 1-2 hours
  - **Trade-off**: Accept safety risk for performance

- [ ] **Option B: Remove Performance Requirement**
  - Remove 8-tick guarantee
  - Defeats KNHK's core purpose
  - **Effort**: N/A (not recommended)
  - **Trade-off**: System cannot meet its promise

- [ ] **Option C: Debug-Only Performance**
  - Production slower than development
  - Violates "test what you ship"
  - **Effort**: N/A (not recommended)
  - **Trade-off**: Wrong architectural pattern

**Decision Made**: _____________ (fill in after meeting)
**Date Decided**: _____________
**Approved By**: _____________

---

## PHASE 1: MAKE IT COMPILE (Day 1-2, 8-12 hours)

### Priority 1A: knhk-kernel Unsafe Code Policy (2-3 hours)

**File**: `rust/knhk-kernel/src/lib.rs`

- [ ] **Step 1**: Decision on unsafe policy (see above)
- [ ] **Step 2**: Update policy in `lib.rs:6`
- [ ] **Step 3**: Document decision in DOCTRINE_COVENANT.md
- [ ] **Step 4**: Add safety proofs to each unsafe block:
  - [ ] `descriptor.rs:269` - Box::from_raw safety proof
  - [ ] `descriptor.rs:287` - Pointer dereference safety proof
  - [ ] `descriptor.rs:307` - Memory cleanup safety proof
  - [ ] `executor.rs:110` - Transmute safety proof
  - [ ] `executor.rs:117` - State conversion safety proof
  - [ ] `executor.rs:134` - Raw pointer mutation safety proof
  - [ ] `pattern.rs:156` - get_unchecked bounds proof
  - [ ] `lib.rs:127` - PatternType transmute proof
- [ ] **Step 5**: Verify kernel compiles: `cargo build -p knhk-kernel --release`
- [ ] **Step 6**: Run kernel tests: `cargo test -p knhk-kernel`

**Definition of Done:**
- [ ] knhk-kernel compiles in release mode
- [ ] All unsafe blocks have safety proofs
- [ ] Zero unsafe-related warnings
- [ ] All kernel tests pass

---

### Priority 1B: knhk-consensus Type System Errors (2-3 hours)

**Files**: `rust/knhk-consensus/src/{pbft,hotstuff,state,network}.rs`

#### Error 1: Field Access on Reference
- [ ] **File**: `pbft.rs:290`
- [ ] **Fix**: Change `.map(|entry| (entry.key().0, ...)` to proper tuple access
- [ ] **Test**: Verify prepare message handling
- [ ] **Status**: ⬜ Not started | 🔄 In progress | ✅ Complete

#### Error 2: String/Vec<u8> Type Mismatch
- [ ] **File**: `state.rs:263-264`
- [ ] **Fix**: Convert hash to String or change expected type to Vec<u8>
- [ ] **Options**:
  - [ ] Convert to hex string: `hex::encode(snapshot.hash)`
  - [ ] Change expected type to `Vec<u8>`
- [ ] **Test**: Verify snapshot validation
- [ ] **Status**: ⬜ Not started | 🔄 In progress | ✅ Complete

#### Error 3: Borrow Checker - Immutable Count
- [ ] **File**: `hotstuff.rs:289`
- [ ] **Fix**: Change `or_insert(0)` to `or_insert_with(|| 0)`
- [ ] **Code**: `let count = self.vote_count.entry(view).or_insert_with(|| 0);`
- [ ] **Test**: Verify vote counting
- [ ] **Status**: ⬜ Not started | 🔄 In progress | ✅ Complete

- [ ] **File**: `pbft.rs:236`
- [ ] **Fix**: Same as hotstuff.rs:289
- [ ] **Test**: Verify prepare counting
- [ ] **Status**: ⬜ Not started | 🔄 In progress | ✅ Complete

#### Error 4: Moved Value (msg)
- [ ] **File**: `network.rs:260`
- [ ] **Fix**: Clone msg before inserting: `self.received.insert(msg.sequence, msg.clone());`
- [ ] **Test**: Verify message handling
- [ ] **Status**: ⬜ Not started | 🔄 In progress | ✅ Complete

**Definition of Done:**
- [ ] knhk-consensus compiles without errors
- [ ] All type mismatches resolved
- [ ] All borrow checker errors fixed
- [ ] Consensus tests pass

---

### Priority 1C: Root Package Dependencies (1-2 hours)

**Files**: `src/production/{persistence,observability,monitoring,cost_tracking}.rs`

#### Error 1: Missing rocksdb Dependency
- [ ] **File**: `Cargo.toml` (root)
- [ ] **Fix**: Add `rocksdb = "0.21"` to dependencies
- [ ] **Alternative**: Remove rocksdb usage if not needed
- [ ] **Status**: ⬜ Not started | 🔄 In progress | ✅ Complete

#### Error 2: OpenTelemetry SDK API
- [ ] **File**: `src/production/observability.rs:189`
- [ ] **Current**: `sdkmetrics::SdkMeterProvider::builder()`
- [ ] **Fix**: Change to `sdkmetrics::MeterProvider::builder()`
- [ ] **Also fix**: Line 63 type declaration
- [ ] **Status**: ⬜ Not started | 🔄 In progress | ✅ Complete

#### Error 3: Private Struct Access
- [ ] **File**: `src/production/platform.rs:836`
- [ ] **Fix**: Change `struct SystemHealth` to `pub struct SystemHealth`
- [ ] **Status**: ⬜ Not started | 🔄 In progress | ✅ Complete

#### Error 4: Missing Macro Import
- [ ] **File**: `src/production/cost_tracking.rs:5`
- [ ] **Fix**: Add `use tracing::warn;`
- [ ] **Status**: ⬜ Not started | 🔄 In progress | ✅ Complete

**Definition of Done:**
- [ ] All dependencies added to Cargo.toml
- [ ] All API mismatches resolved
- [ ] All visibility issues fixed
- [ ] Root package compiles

---

### Priority 1D: Cleanup Warnings (1 hour)

**Optional but recommended**

- [ ] **Unused imports**: Fix 20+ unused import warnings
- [ ] **Unused variables**: Fix 10+ unused variable warnings
- [ ] **Unused crate dependencies**: Remove 18+ unused deps from knhk-consensus
- [ ] **Dead code**: Address 5+ dead code warnings
- [ ] **Run**: `cargo clippy --workspace --fix --allow-dirty`

**Definition of Done:**
- [ ] Zero clippy warnings
- [ ] Clean compilation output

---

### PHASE 1 CHECKPOINT

**Validation:**
```bash
cargo build --workspace --release 2>&1 | tee build-fixed.log
```

**Expected Output:**
```
Finished release [optimized] target(s) in X.XXs
```

**Pass Criteria:**
- [ ] Zero compilation errors
- [ ] Build completes successfully
- [ ] All packages compile
- [ ] Binary artifacts generated

**If Failed:**
- [ ] Review build-fixed.log
- [ ] Identify remaining errors
- [ ] Update this checklist
- [ ] Continue fixes

---

## PHASE 2: MAKE IT PASS TESTS (Day 2, 2-4 hours)

### Step 2.1: Run Full Test Suite (1 hour)

```bash
cargo test --workspace --lib 2>&1 | tee test-results.log
```

**Expected**: `test result: ok`

- [ ] **All tests pass**: Yes / No
- [ ] **Failed tests**: _____ (count)
- [ ] **Flaky tests**: _____ (count)

**If tests fail, categorize:**

#### Unit Test Failures
- [ ] List failing tests here:
  - [ ] Test 1: __________________
  - [ ] Test 2: __________________
  - [ ] Test 3: __________________

#### Integration Test Failures
- [ ] List failing tests here:
  - [ ] Test 1: __________________
  - [ ] Test 2: __________________

---

### Step 2.2: Chicago TDD Performance (1 hour)

```bash
make test-chicago-v04 2>&1 | tee chicago-results.log
```

**Expected**: All performance checks pass, hot path ≤8 ticks

- [ ] **Hot path latency**: _____ ticks (must be ≤8)
- [ ] **Cold path latency**: _____ ticks
- [ ] **Warm path latency**: _____ ticks
- [ ] **All benchmarks pass**: Yes / No

**If failed:**
- [ ] Identify slow code paths
- [ ] Profile with `perf` or `flamegraph`
- [ ] Optimize hot path
- [ ] Re-run benchmarks

---

### Step 2.3: Integration Tests (30 minutes)

```bash
make test-integration-v2 2>&1 | tee integration-results.log
```

**Expected**: All integration tests pass

- [ ] **Tests passed**: _____ / _____
- [ ] **All components integrated**: Yes / No

**If failed:**
- [ ] List failing integrations:
  - [ ] Integration 1: __________________
  - [ ] Integration 2: __________________

---

### PHASE 2 CHECKPOINT

**Pass Criteria:**
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Hot path ≤8 ticks (Chicago TDD)
- [ ] Zero test failures

**If Failed:**
- [ ] Categorize test failures
- [ ] Fix critical failures first
- [ ] Accept known flaky tests (document)
- [ ] Re-run tests

---

## PHASE 3: MAKE IT PRODUCTION READY (Day 3, 2-4 hours)

### Step 3.1: Weaver Schema Validation (1-2 hours)

#### Schema Definition Check
```bash
weaver registry check -r registry/ 2>&1 | tee weaver-schema.log
```

**Expected**: Schema validation passes

- [ ] **Schema valid**: Yes / No
- [ ] **Errors found**: _____ (count)

**If failed:**
- [ ] List schema errors:
  - [ ] Error 1: __________________
  - [ ] Error 2: __________________

#### Runtime Telemetry Check
```bash
weaver registry live-check --registry registry/ 2>&1 | tee weaver-live.log
```

**Expected**: Runtime telemetry matches schema

- [ ] **Telemetry valid**: Yes / No
- [ ] **Span mismatches**: _____ (count)
- [ ] **Metric mismatches**: _____ (count)

**If failed:**
- [ ] Fix schema definitions
- [ ] Fix telemetry emission code
- [ ] Re-run validation

---

### Step 3.2: E2E RevOps Workflow (1 hour)

**Test Workflow**: Revenue Recognition Spike Detection

#### Step 1: Observation
- [ ] Simulate revenue spike: `knhk observe --event revenue_spike --amount 1000000`
- [ ] Expected: Event logged
- [ ] Actual: __________________

#### Step 2: Pattern Detection
- [ ] Check pattern match: `knhk patterns detect --event revenue_spike`
- [ ] Expected: "Revenue Recognition" pattern matched
- [ ] Actual: __________________

#### Step 3: Proposal
- [ ] Generate proposal: `knhk propose --pattern revenue_recognition`
- [ ] Expected: "Execute approval workflow" proposed
- [ ] Actual: __________________

#### Step 4: Execution (Hot Path)
- [ ] Execute workflow: `knhk execute --proposal <id>`
- [ ] Expected: Completed in ≤8 ticks
- [ ] Actual latency: _____ ticks

#### Step 5: Verification
- [ ] Check receipt: `knhk receipt verify --execution <id>`
- [ ] Expected: Valid cryptographic receipt
- [ ] Actual: __________________

- [ ] Check telemetry: `weaver registry live-check --registry registry/`
- [ ] Expected: All spans emitted correctly
- [ ] Actual: __________________

**E2E Workflow Pass**: Yes / No

---

### Step 3.3: Final Production Checklist (30 minutes)

#### Build & Code Quality
- [ ] `cargo build --workspace --release` - succeeds
- [ ] `cargo clippy --workspace -- -D warnings` - zero warnings
- [ ] `make build` (C library) - succeeds
- [ ] No `.unwrap()` in production code
- [ ] All traits `dyn` compatible
- [ ] Proper `Result<T, E>` usage
- [ ] No `println!` in production code
- [ ] No fake `Ok(())` returns

#### Weaver Validation (SOURCE OF TRUTH)
- [ ] `weaver registry check -r registry/` - passes
- [ ] `weaver registry live-check` - passes
- [ ] All OTEL spans defined in schema
- [ ] Schema documents exact behavior
- [ ] Live telemetry matches schema

#### Functional Validation
- [ ] Commands execute with real arguments
- [ ] Commands produce expected output
- [ ] Commands emit proper telemetry
- [ ] E2E workflow tested successfully
- [ ] Performance ≤8 ticks confirmed

#### Traditional Testing
- [ ] `cargo test --workspace` - all pass
- [ ] `make test-chicago-v04` - all pass
- [ ] `make test-performance-v04` - all pass
- [ ] `make test-integration-v2` - all pass
- [ ] Tests follow AAA pattern

---

### PHASE 3 CHECKPOINT

**Final Validation:**

Create final report:
```bash
# Copy this template and fill in results
cp docs/validation/PRODUCTION_READY_VALIDATION.md \
   docs/validation/PRODUCTION_READY_FINAL.md
```

**Update sections with:**
- [ ] Build status: ✅ SUCCESS
- [ ] Test results: _____ passed / _____ total
- [ ] Performance: Hot path _____ ticks (≤8)
- [ ] Weaver status: ✅ COMPLIANT
- [ ] E2E workflow: ✅ PASSED
- [ ] Readiness verdict: **PRODUCTION READY** (or list blockers)

---

## FINAL SIGN-OFF

### Production Readiness Certification

**Date**: _____________
**Version**: _____________
**Validated By**: _____________

**Checklist:**
- [ ] All compilation errors resolved
- [ ] All tests passing (100%)
- [ ] Performance requirements met (≤8 ticks)
- [ ] Weaver schema validation passes
- [ ] E2E workflow validated
- [ ] Documentation updated
- [ ] Known issues documented
- [ ] Deployment plan created

**Verdict**:
- [ ] ✅ **PRODUCTION READY** - Approved for deployment
- [ ] ⚠️  **PRODUCTION READY WITH NOTES** - Approved with conditions
- [ ] ❌ **NOT PRODUCTION READY** - Blockers remain

**Conditions (if any):**
1. __________________
2. __________________
3. __________________

**Approvers:**
- Technical Lead: _____________ (signature/date)
- Security Review: _____________ (signature/date)
- Product Owner: _____________ (signature/date)

---

## PROGRESS TRACKING

### Daily Standup Template

**Date**: _____________

**Yesterday:**
- Completed: __________________
- Blockers: __________________

**Today:**
- Plan: __________________
- Target: __________________

**Risks:**
- __________________

### Milestone Tracking

| Milestone | Target Date | Actual Date | Status |
|-----------|-------------|-------------|--------|
| Decision on unsafe policy | Day 1 AM | _________ | ⬜ |
| Phase 1 complete (compile) | Day 2 | _________ | ⬜ |
| Phase 2 complete (tests) | Day 2 EOD | _________ | ⬜ |
| Phase 3 complete (production) | Day 3 | _________ | ⬜ |
| Final sign-off | Day 3 EOD | _________ | ⬜ |

---

**Checklist Version**: 1.0
**Created**: 2025-11-17
**Last Updated**: _____________
**Next Review**: After each phase completion
