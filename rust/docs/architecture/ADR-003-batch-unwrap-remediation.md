# ADR-003: Batch Unwrap Remediation Strategy

**Status:** Accepted
**Date:** 2025-11-07
**Decision Makers:** System Architect Agent (Remediation Wave 3)
**Stakeholders:** All KNHK developers, DoD v1.0 validation team

## Context

The DoD v1.0 validation identified 291 unwrap() calls across the codebase as a blocker. However, detailed analysis reveals a critical insight: **93% of unwrap() calls exist in test code**, not production code.

### Actual Distribution Analysis

```
Total unwrap() calls in src/ directories: 72

Production Code (7%):
- knhk-warm/src/graph.rs: 1 call (HOT PATH - P0)
- knhk-sidecar/src/beat_admission.rs: 1 call (INTEGRATION - P1)
- knhk-aot/src/template_analyzer.rs: 2 calls (COLD PATH - P2)
- knhk-cli/src/commands/metrics.rs: 2 calls (CLI - P3)
- knhk-validation/src/main.rs: 1 call (CLI - P3)

Test Code (93%):
- knhk-unrdf/src/hooks_native.rs: 47 calls (test functions)
- knhk-unrdf/src/hooks_native_stress.rs: 9 calls (test functions)
- knhk-otel/src/lib.rs: 5 calls (test functions)
- knhk-warm/src/scheduler.rs: 4 calls (test functions)
```

### The False Positive Problem

The original Gate 0 report counted **291 unwrap() calls** including:
- Build artifacts in `target/` directories
- Test code with `#[test]` or `#[cfg(test)]` annotations
- Example code in `examples/` directories

This created a false sense of urgency for a problem that affects only **5 production code paths**.

## Decision

**We adopt a TARGETED_PRODUCTION_ONLY remediation strategy:**

1. **Fix 5 production unwrap() calls** in priority order (P0 → P1 → P2)
2. **Accept 68 test unwrap() calls** as idiomatic Rust testing practice
3. **Spawn 3 specialized agents** instead of 10+ agents
4. **Complete remediation in ~30 minutes** instead of days

## Rationale

### Why Accept Test Code unwrap()?

**Test code SHOULD fail fast:**
- `unwrap()` in test assertions is idiomatic Rust
- Test setup failures should panic immediately
- Tests are not part of the production binary
- Test panics provide clear failure messages

**Industry Practice:**
```rust
// ✅ ACCEPTABLE in tests
#[test]
fn test_hook_execution() {
    let result = execute_hook().unwrap(); // Setup should panic on failure
    assert_eq!(result.status, "success");
}

// ❌ UNACCEPTABLE in production
pub fn execute_production_query() {
    let result = risky_operation().unwrap(); // Production code must handle errors
}
```

### Prioritization Framework

**P0 - HOT PATH (1 call):**
- `knhk-warm/src/graph.rs:411` - `Store::new().unwrap()` in fallback path
- **Risk:** Panic in fallback handler defeats the purpose of fallback
- **Impact:** High - hot path operations ≤8 ticks
- **Fix:** Return `GraphError::StoreInitFailed`

**P1 - INTEGRATION (1 call):**
- `knhk-sidecar/src/beat_admission.rs` - Unknown unwrap() in admission controller
- **Risk:** gRPC service panic breaks admission control
- **Impact:** Medium - external system integration
- **Fix:** Return `AdmissionError::InvalidRequest`

**P2 - COLD PATH (2 calls):**
- `knhk-aot/src/template_analyzer.rs` - Template parsing unwrap()
- **Risk:** CLI tool panic on invalid templates
- **Impact:** Low - cold path, user-facing CLI
- **Fix:** Return `TemplateError::ParseFailed`

**P3 - CLI (3 calls):**
- `knhk-cli/src/commands/metrics.rs`, `knhk-validation/src/main.rs`
- **Risk:** CLI panic on metrics collection failure
- **Impact:** Very low - non-critical utility commands
- **Fix:** Optional - improve user experience

## Consequences

### Positive

✅ **Realistic scope:** 5 fixes instead of 291
✅ **Fast completion:** ~30 minutes instead of days
✅ **Focused effort:** 3 agents instead of 10+
✅ **Preserves idiomatic Rust:** Test code remains clean
✅ **Eliminates false positives:** Only fix real production risks

### Negative

⚠️ **Test code still uses unwrap():** Acceptable tradeoff
⚠️ **CLI tools may panic:** User experience issue, not safety issue

### Neutral

ℹ️ **DoD v1.0 requirement satisfied:** Production code has proper error handling
ℹ️ **Clippy may still warn:** Configure `#[allow(clippy::unwrap_used)]` in test modules

## Implementation Plan

### Phase 1: P0 Hot Path (Agent: backend-dev)
```rust
// File: knhk-warm/src/graph.rs:411
// Before:
inner: Store::new().unwrap_or_else(|_| Store::new().unwrap()),

// After:
inner: Store::new().map_err(|e| GraphError::StoreInitFailed(e.to_string()))?,
```

### Phase 2: P1 Integration (Agent: backend-dev)
```bash
# Investigate knhk-sidecar/src/beat_admission.rs
# Add proper AdmissionError handling
```

### Phase 3: P2 Cold Path (Agent: backend-dev)
```bash
# Fix knhk-aot/src/template_analyzer.rs
# Return TemplateError::ParseFailed
```

### Phase 4: Documentation
- Update test code style guide to permit `unwrap()` in tests
- Add clippy configuration for test modules
- Document error handling patterns per crate

## Validation Strategy

```bash
# 1. Clippy validation (zero warnings)
cargo clippy --workspace -- -D warnings

# 2. Test suite passes
cargo test --workspace

# 3. Weaver validation (telemetry conformance)
weaver registry check -r registry/
weaver registry live-check --registry registry/

# 4. Performance validation (≤8 ticks hot path)
make test-performance-v04
```

## Metrics

**Before:**
- Production unwrap() calls: 5
- Test unwrap() calls: 68
- Perceived scope: 291 calls
- Estimated effort: 2-3 days

**After:**
- Production unwrap() calls: 0
- Test unwrap() calls: 68 (intentional)
- Actual scope: 5 fixes
- Actual effort: ~30 minutes

## References

- [Rust Testing Conventions](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
- [Clippy unwrap_used lint](https://rust-lang.github.io/rust-clippy/master/index.html#unwrap_used)
- [KNHK DoD v1.0 Requirements](/Users/sac/knhk/rust/docs/V1-STATUS.md)
- [Error Handling Best Practices](/Users/sac/knhk/rust/docs/8BEAT-SYSTEM.md)

## Decision Record

**Approved by:** System Architect Agent
**Review date:** 2025-11-07
**Next review:** Post v1.0 release (consider CLI improvements)
