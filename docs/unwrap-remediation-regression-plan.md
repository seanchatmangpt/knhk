# Unwrap() Remediation Regression Testing Plan
**TDD London School Approach**

## Executive Summary

**Objective**: Verify that eliminating 149 unwrap() calls across knhk-etl doesn't introduce regressions.

**Baseline Metrics** (2025-11-07):
- **Clippy Errors**: 41 (must remain at 0 after fixes)
- **Unwrap Calls**: 56 in src/, 18 in tests
- **Test Status**: Currently failing due to existing clippy errors

**Critical Success Criteria**:
1. Zero clippy warnings after fixes (`cargo clippy -- -D warnings`)
2. All existing tests continue to pass
3. No performance degradation (≤8 ticks hot path)
4. New error paths have test coverage

## Baseline Analysis

### Current Clippy Errors (41 total)

**Categories**:
1. **Redundant Closures** (8 errors):
   - `beat_scheduler.rs`: Lines 119, 123, 435, 443
   - `reconcile.rs`: Lines 152, 256

2. **Needless Range Loops** (2 errors):
   - `reconcile.rs`: Lines 164, 271

3. **Non-Snake-Case Variables** (31 errors):
   - `ring_conversion.rs`: S, P, O variable names (conforming to RDF triple naming)
   - These are intentional (Subject/Predicate/Object) but trigger warnings

### Unwrap() Distribution

**Production Code** (`src/`): 56 unwrap() calls
- `reconcile.rs`: 1 unwrap
- `ring_buffer.rs`: 7 unwrap
- `lib.rs`: 11 unwrap
- `failure_actions.rs`: 4 unwrap
- `hook_registry.rs`: 9 unwrap
- `reflex_map.rs`: 3 unwrap
- `runtime_class.rs`: 12 unwrap

**Test Code**: 18 expect() calls
- Chicago TDD tests use expect() for cleaner failure messages

## Regression Testing Strategy

### 1. Continuous Monitoring

**Script**: `/Users/sac/knhk/rust/knhk-etl/tests/monitor_unwrap_remediation.sh`

**Capabilities**:
- Count unwrap()/expect() usage
- Run `cargo clippy -- -D warnings`
- Run `cargo test`
- Check Chicago TDD test files
- Log metrics to Claude Flow memory
- Detect regressions immediately

**Usage**:
```bash
# Single validation pass
./tests/monitor_unwrap_remediation.sh

# Continuous monitoring (every 30 seconds)
./tests/monitor_unwrap_remediation.sh watch
```

### 2. Regression Test Suite

**File**: `/Users/sac/knhk/rust/knhk-etl/tests/regression_unwrap_fixes.rs`

**Test Categories** (15 test cases):

#### A. BeatScheduler Error Path Coverage (7 tests)
- `test_beat_scheduler_invalid_shard_count_error`
- `test_beat_scheduler_invalid_domain_count_error`
- `test_fiber_error_propagation`
- `test_ring_buffer_full_error`
- `test_quorum_failed_error_propagation` (lockchain feature)
- `test_storage_failed_error_propagation` (lockchain feature)

#### B. Pipeline Error Handling (2 tests)
- `test_pipeline_creation_error_handling`
- `test_pipeline_error_propagation`

#### C. HookRegistry Error Handling (2 tests)
- `test_hook_registry_duplicate_error`
- `test_hook_registry_not_found_error`

#### D. FFI Error Conversion (2 tests)
- `test_rust_error_to_c_error_code`
- `test_null_pointer_safety`

#### E. Telemetry Context Preservation (1 test)
- `test_error_preserves_span_context`

#### F. Performance Verification (2 tests)
- `test_no_heap_allocation_in_hot_path`
- `test_error_path_allocation_acceptable`

#### G. Lock Poisoning Recovery (1 test)
- `test_mutex_poison_recovery`

### 3. Verification Protocol

For **each file** fixed by backend-dev agent:

1. **Before Fix**:
   - Record unwrap() count
   - Record clippy error count
   - Run existing tests

2. **After Fix**:
   - Verify unwrap() count decreased
   - Run `cargo clippy -p knhk-etl -- -D warnings`
   - Run `cargo test -p knhk-etl`
   - Check Chicago TDD tests still pass
   - Verify no performance regression

3. **If Regression Detected**:
   - Document exact failure
   - Identify root cause
   - Suggest fix to backend-dev agent
   - Re-verify after fix

## Coordination with Backend-Dev Agent

### Memory Keys

```
remediation/baseline          # Initial metrics
remediation/etl-fixes         # List of files fixed
remediation/regression-suite  # Test suite metadata
remediation/regression-check  # Verification results
```

### Hook Integration

**Pre-Fix**:
```bash
npx claude-flow@alpha hooks pre-task --description "Fix unwrap() in <file>"
```

**Post-Fix**:
```bash
npx claude-flow@alpha hooks post-edit --file "<file>" \
  --memory-key "remediation/etl-fixes"
npx claude-flow@alpha hooks notify --message "Fixed unwrap() in <file>"
```

**Post-Verification**:
```bash
npx claude-flow@alpha hooks post-task --task-id "regression-verification" \
  --memory-key "remediation/regression-check"
```

## Critical Regression Scenarios

### 1. Error Propagation Breaks

**Risk**: Changing `.unwrap()` to `?` changes function signature to `Result<T, E>`.

**Detection**:
- Compilation will fail if caller doesn't handle Result
- Tests will fail if error handling missing

**Mitigation**:
- Verify all callers updated
- Add tests for error paths

### 2. Panics Become Silent Failures

**Risk**: Code that previously panicked now returns `Err` silently.

**Detection**:
- Log analysis (errors should be logged)
- Telemetry verification (errors should emit spans)

**Mitigation**:
- Ensure error logging in place
- Verify OTEL context preserved

### 3. Performance Regression

**Risk**: New error handling adds heap allocations.

**Detection**:
- Performance test suite (`test-performance-v04`)
- RUSTFLAGS="-Z print-alloc-stats"

**Mitigation**:
- Error types should be stack-allocated
- Only String::from allocates (acceptable in cold path)

### 4. FFI Boundary Issues

**Risk**: C ↔ Rust error conversion breaks.

**Detection**:
- FFI tests fail
- C integration tests fail

**Mitigation**:
- Test error code round-tripping
- Test null pointer safety

## Success Metrics

### Gate 0: Zero Clippy Warnings
```bash
cargo clippy -p knhk-etl -- -D warnings
# Must exit 0
```

### Gate 1: All Tests Pass
```bash
cargo test -p knhk-etl
# All tests pass
```

### Gate 2: Chicago TDD Pass
```bash
cd /Users/sac/knhk/c
make test-chicago-v04
# All Chicago tests pass
```

### Gate 3: Performance Maintained
```bash
make test-performance-v04
# All operations ≤8 ticks
```

### Gate 4: Weaver Validation
```bash
weaver registry check -r registry/
weaver registry live-check --registry registry/
# Both pass (source of truth)
```

## Deliverables

### 1. Regression Test Report

**Location**: `docs/evidence/unwrap-remediation-regression-report.md`

**Contents**:
- Baseline vs. final metrics
- List of files fixed
- Regression incidents (if any)
- Performance impact analysis
- Final validation results

### 2. New Test Cases

**Location**: `rust/knhk-etl/tests/regression_unwrap_fixes.rs`

**Coverage**:
- Error path testing
- FFI error conversion
- Telemetry context preservation
- Lock poisoning recovery

### 3. Monitoring Script

**Location**: `rust/knhk-etl/tests/monitor_unwrap_remediation.sh`

**Features**:
- Continuous validation
- Metric tracking
- Regression detection
- Claude Flow integration

## Risk Assessment

### High Risk Areas

1. **`beat_scheduler.rs`**: Complex state machine, 4 clippy errors
2. **`reconcile.rs`**: Core reconciliation logic, 4 clippy errors
3. **`ring_conversion.rs`**: Performance-critical hot path, 31 warnings

### Medium Risk Areas

1. **`hook_registry.rs`**: 9 unwrap() calls
2. **`runtime_class.rs`**: 12 unwrap() calls
3. **`lib.rs`**: 11 unwrap() calls in tests

### Low Risk Areas

1. **`reflex_map.rs`**: 3 unwrap() calls
2. **`failure_actions.rs`**: 4 unwrap() calls
3. **`ring_buffer.rs`**: 7 unwrap() calls

## Timeline

1. **Phase 1**: Backend-dev fixes unwrap() calls (in progress)
2. **Phase 2**: TDD London School verifies each fix (this agent)
3. **Phase 3**: Final validation and report generation
4. **Phase 4**: Weaver schema validation (source of truth)

## Notes

- **The Only Source of Truth**: Weaver validation is the final authority
- **Running `--help` proves NOTHING**: Must execute actual commands
- **Tests can have false positives**: Only Weaver detects real behavior
- **Performance must be verified**: ≤8 ticks is the law (Chatman Constant)
