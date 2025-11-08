# KNHK v1.0.0 - Critical Blockers Remediation Plan

**Status**: ❌ **NO-GO FOR RELEASE**
**Estimated Remediation**: 8-15 hours (1-2 days)
**Date**: 2025-11-07

---

## 5 Critical (P0) Blockers

### 1. ❌ Clippy Errors (5 issues) - 1-2 hours

**Location**: `rust/knhk-etl/`

**Errors**:
```rust
// Error 1-2: unexpected `cfg` condition value: `tokio-runtime`
// Fix: Add to knhk-etl/Cargo.toml:
[features]
tokio-runtime = []

// Error 3: complex return type in ring_conversion.rs:12
// Fix: Create type alias
type SoaTriples = Result<(Vec<u64>, Vec<u64>, Vec<u64>), String>;
pub fn raw_triples_to_soa(triples: &[RawTriple]) -> SoaTriples { ... }

// Error 4-5: needless_range_loop in reconcile.rs
// Fix: Use iterator enumerate
// Line 164:
for (i, item) in delta.iter().enumerate() {
    // Use item instead of delta[i]
}
// Line 286: Same fix
```

**Validation**:
```bash
cd rust/knhk-etl
# Apply fixes above
cargo clippy -- -D warnings  # Must pass with zero errors
```

---

### 2. ❌ Code Formatting - 15 minutes

**Command**:
```bash
cd /Users/sac/knhk/rust
cargo fmt --all
cargo fmt --all -- --check  # Verify zero violations
```

**Impact**: Fixes all 1,075,957 formatting violations automatically.

---

### 3. ❌ Version Inconsistency - 1 hour

**Current State**:
- Workspace: `1.0.0`
- knhk-hot: `1.0.0` ✅
- knhk-sidecar: `0.5.0` ❌
- All others: `0.1.0` ❌

**Fix**: Update all crate `Cargo.toml` files:

```bash
# Update these files to version = "1.0.0":
rust/knhk-aot/Cargo.toml
rust/knhk-cli/Cargo.toml
rust/knhk-config/Cargo.toml
rust/knhk-connectors/Cargo.toml
rust/knhk-etl/Cargo.toml
rust/knhk-integration-tests/Cargo.toml
rust/knhk-lockchain/Cargo.toml
rust/knhk-otel/Cargo.toml
rust/knhk-unrdf/Cargo.toml
rust/knhk-validation/Cargo.toml
rust/knhk-warm/Cargo.toml

# Special case: knhk-sidecar (excluded from workspace)
# Either: Add to workspace or keep at 0.5.0 and document as "preview"
```

**Verification**:
```bash
grep -h "^version" rust/*/Cargo.toml | sort | uniq -c
# Should show: "13 version = "1.0.0"" (or 12 if sidecar excluded)
```

---

### 4. ❌ Weaver Live Validation - 2-4 hours

**Current Status**: ❌ BLOCKED by port 4318 conflict

**Per V1-STATUS.md**: "Gate 1: Weaver live-check blocked (port 4318 conflict)"

**Fix Steps**:

1. **Identify port conflict**:
```bash
lsof -i :4318
# Kill conflicting process or choose different port
```

2. **Start OTLP collector**:
```bash
# Option A: Use Docker
docker run -p 4318:4318 otel/opentelemetry-collector-contrib:latest

# Option B: Use local collector
# Configure collector.yaml with OTLP receiver on 4318
```

3. **Run KNHK with telemetry**:
```bash
# Execute hot path operations to generate telemetry
cd rust/knhk-cli
cargo run -- boot --sigma schema.ttl --q invariants.shacl
cargo run -- admit --delta delta.ttl
# Etc. for all commands
```

4. **Execute live validation**:
```bash
weaver registry live-check --registry registry/
# Must show: ✔ All telemetry conforms to schema
```

**CRITICAL**: This is the SOURCE OF TRUTH. Without this, we cannot prove features work.

---

### 5. ❌ Test Suite Compilation - 4-8 hours

**Failures**:
- `knhk-config`: 13 errors
- `knhk-validation`: 2 errors
- `knhk-integration-tests`: 50+ errors

**Root Causes**:

1. **knhk-config errors** (13 errors):
```rust
// Unresolved imports
// Fix: Update knhk-config/src/lib.rs
pub use error::ConfigError;
pub fn load_default_config() -> Result<Config, ConfigError> { ... }

// Missing fields in Config struct
// Fix: Restore removed fields or update test expectations
```

2. **knhk-validation errors** (2 errors):
```rust
// Unresolved imports
use knhk_validation::policy_engine;  // Add policy_engine module
use performance_validation;  // Add performance_validation module
```

3. **knhk-integration-tests errors** (50+ errors):
```rust
// Missing gRPC types (query_request, health_status, etc.)
// Fix: Either:
//   A. Add gRPC feature dependencies back
//   B. Gate tests with #[cfg(feature = "grpc")]
//   C. Implement minimal gRPC stubs for testing
```

**Validation**:
```bash
cd rust
cargo test --workspace --no-fail-fast
# Must show: "test result: ok"

make test-chicago-v04
# Must show: "22/22 tests passed"

make test-performance-v04
# Must show: "All hot path operations ≤8 ticks"
```

---

## Pre-Release Checklist

Execute in order:

```bash
# 1. Format code (15 min)
cd /Users/sac/knhk/rust
cargo fmt --all
git diff  # Review changes

# 2. Fix clippy errors (1-2 hours)
cd rust/knhk-etl
# Apply 5 fixes from Blocker #1
cargo clippy --workspace -- -D warnings

# 3. Update versions (1 hour)
# Edit 11 Cargo.toml files per Blocker #3
grep -h "^version" rust/*/Cargo.toml | sort | uniq -c

# 4. Fix test compilation (4-8 hours)
# Apply fixes from Blocker #5
cargo test --workspace

# 5. Weaver live validation (2-4 hours)
# Follow steps from Blocker #4
weaver registry live-check --registry registry/

# 6. Commit changes
git add .
git commit -m "fix: resolve v1.0.0 critical blockers

- Fix clippy errors in knhk-etl (5 issues)
- Format all code with rustfmt
- Update all crate versions to 1.0.0
- Fix test compilation errors
- Verify Weaver live validation passes

Resolves: P0 blockers for v1.0.0 release"

# 7. Create release tag (ONLY after all above pass)
git tag -a v1.0.0 -m "KNHK v1.0.0 - Production Release"
git push origin main --tags
```

---

## Validation Gates (Must All Pass)

- [ ] **Gate 0: Code Quality**
  - [ ] `cargo clippy --workspace -- -D warnings` (0 errors)
  - [ ] `cargo fmt --all -- --check` (0 violations)
  - [ ] All versions = 1.0.0
  - [ ] Zero unwrap() in production code ✅ (Already done)

- [ ] **Gate 1: Weaver Validation** (SOURCE OF TRUTH)
  - [x] `weaver registry check -r registry/` ✅ (Already passing)
  - [ ] `weaver registry live-check --registry registry/`

- [ ] **Gate 2: Traditional Testing**
  - [ ] `cargo test --workspace` (all pass)
  - [ ] `make test-chicago-v04` (22/22 pass)
  - [ ] `make test-performance-v04` (≤8 ticks)

- [ ] **Gate 3: Release Artifacts**
  - [ ] `make build` (C library compiles)
  - [ ] CHANGELOG.md created
  - [ ] RELEASE.md with v1.0 notes
  - [ ] All changes committed
  - [ ] Tag v1.0.0 created

---

## Success Criteria

**v1.0.0 is READY FOR RELEASE when**:

1. ✅ All 5 P0 blockers resolved
2. ✅ All validation gates pass
3. ✅ DFLSS score recalculated ≥95%
4. ✅ Weaver live validation confirms runtime telemetry
5. ✅ No uncommitted changes
6. ✅ Release documentation complete

**Current Status**: ❌ **0/6 criteria met**

**After P0 Remediation**: ⚠️ **Estimated 4-5/6 criteria met**

**Full Production Ready**: 1-2 weeks per V1-STATUS.md roadmap

---

## Quick Reference

**Most Critical First**:
1. Weaver live validation (proves features work)
2. Test suite (validates functionality)
3. Clippy (code quality)
4. Formatting (style consistency)
5. Versions (release preparation)

**Fastest Wins**:
1. Formatting (15 min)
2. Clippy (1-2 hours)
3. Versions (1 hour)

**Longest Tasks**:
1. Test compilation (4-8 hours)
2. Weaver setup (2-4 hours)

---

**Total Time**: 8-15 hours (optimistic) to 1-2 weeks (realistic with full validation)

**Report Generated**: 2025-11-07
**Next Review**: After P0 blockers resolved
**Owner**: Development team
