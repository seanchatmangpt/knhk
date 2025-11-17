# KNHK Production Readiness - Fix Roadmap

## Immediate Actions (Next 30 Minutes)

### Quick Win #1: Enable oxigraph feature in knhk-cli
```bash
# Test if feature-gating is the only issue
cargo build -p knhk-cli --all-features 2>&1 | head -20

# If successful, update default features
```

**File to Edit**: `rust/knhk-cli/Cargo.toml`
```toml
# Line 72-80: Change from
[features]
default = ["std", "otel"]
# ... to:
[features]
default = ["std", "otel", "rdf"]  # Add rdf to default
rdf = ["oxigraph"]  # Add feature flag
```

---

### Quick Win #2: Fix clippy errors (non-blocking)
```bash
# Auto-fix simple issues
cargo fix --lib -p knhk-latex --allow-dirty
cargo fix --lib -p knhk-marketplace --allow-dirty
cargo fix --lib -p knhk-neural --allow-dirty
cargo fix --lib -p knhk-test-cache --allow-dirty
cargo fix --lib -p knhk-dflss --allow-dirty
```

---

## Phase 1: Critical Path Fixes (4-6 hours)

### Task 1.1: knhk-cli - Fix oxigraph dependency (30 min)

**Option A**: Make oxigraph non-optional
```toml
# rust/knhk-cli/Cargo.toml, line 42
oxigraph = { workspace = true }  # Remove: optional = true
```

**Option B**: Enable RDF feature by default
```toml
# rust/knhk-cli/Cargo.toml
[features]
default = ["std", "otel", "rdf"]
rdf = ["oxigraph"]
```

**Validation**:
```bash
cargo build -p knhk-cli
# Should succeed with 0 errors
```

---

### Task 1.2: knhk - Fix rocksdb conflict (2 hours)

**Investigation Required**:
1. Check if oxigraph actually needs rocksdb backend
2. Test with sled or in-memory backend for oxigraph

**File**: `Cargo.toml` (root)
```toml
# Line 23-26: Currently commented
# Option 1: Conditionally enable rocksdb
[dependencies]
rocksdb = { version = "0.21", optional = true }

[features]
storage-rocksdb = ["rocksdb"]
storage-sled = ["sled"]  # Already available
default = ["storage-sled"]  # Use sled by default
```

**Alternative**: Update oxigraph to use separate rocksdb build
```toml
[patch.crates-io]
oxigraph = { git = "https://github.com/oxigraph/oxigraph", branch = "main" }
# Check if newer version has rocksdb conflict resolution
```

**Files to Update**:
- `src/storage/mod.rs` - Conditional compilation
- `src/storage/rocksdb.rs` - Feature-gated
- `src/storage/sled.rs` - Default backend

**Validation**:
```bash
cargo build -p knhk --no-default-features --features storage-sled
```

---

### Task 1.3: knhk - Fix OpenTelemetry 0.21 API (1 hour)

**Error 1**: `SdkMeterProvider` path changed
```rust
// Find files with: grep -r "SdkMeterProvider" src/

// Old (broken):
use opentelemetry::sdkmetrics::SdkMeterProvider;

// New (fixed):
use opentelemetry_sdk::metrics::SdkMeterProvider;
```

**Error 2**: `Span` trait not dyn-safe
```rust
// File: src/observability/*.rs

// Remove async from trait or use async_trait:
use async_trait::async_trait;

#[async_trait]
pub trait SpanExt {
    async fn custom_method(&self) -> Result<()>;
}
```

**Files to Fix**:
1. `src/observability/telemetry.rs`
2. `src/observability/tracing.rs`
3. Any file importing `opentelemetry::trace::Span`

**Validation**:
```bash
cargo build -p knhk 2>&1 | grep -c "SdkMeterProvider\|Span.*dyn"
# Should be 0
```

---

### Task 1.4: knhk - Fix async trait violations (1 hour)

**Covenant Violation**: DOCTRINE_COVENANT #7
> Never use async trait methods (breaks dyn compatibility)

**Files**:
- `src/receipt/store.rs`
- `src/receipt/trait.rs`

**Current (broken)**:
```rust
pub trait ReceiptStore {
    async fn store_receipt(&self, ...) -> Result<()>;  // ❌ Not dyn-safe
}
```

**Fix Option A**: Use `async_trait` macro
```rust
use async_trait::async_trait;

#[async_trait]
pub trait ReceiptStore {
    async fn store_receipt(&self, ...) -> Result<()>;  // ✅ Dyn-safe with macro
}

#[async_trait]
impl ReceiptStore for ConcreteStore {
    async fn store_receipt(&self, ...) -> Result<()> { ... }
}
```

**Fix Option B**: Return `Pin<Box<dyn Future>>`
```rust
use std::pin::Pin;
use std::future::Future;

pub trait ReceiptStore {
    fn store_receipt<'a>(&'a self, ...) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}
```

**Recommendation**: Use Option A (`async_trait` macro) - simpler and more maintainable

**Validation**:
```bash
cargo build -p knhk 2>&1 | grep -c "lifetime.*not match.*trait"
# Should be 0
```

---

### Task 1.5: knhk-workflow-engine - Fix module exports (30 min)

**File**: `rust/knhk-workflow-engine/src/lib.rs`

**Add missing module declarations**:
```rust
// Add at top of file (after existing pub mods)
pub mod hooks;
pub mod registry;

// In hooks.rs, export:
pub struct HookContext { ... }
pub type HookResult<T> = Result<T, HookError>;
```

**Validation**:
```bash
cargo build -p knhk-workflow-engine 2>&1 | grep -c "unresolved.*HookContext"
# Should be 0
```

---

### Task 1.6: knhk-workflow-engine - Fix generic scope issues (2 hours)

**Problem**: Nested closures trying to use outer generic parameters

**Pattern to Find**:
```bash
grep -rn "can't use generic parameters" target/debug/build/
```

**Example Fix**:
```rust
// ❌ BROKEN:
impl<T> Processor<T> {
    fn process(&self) {
        let closure = || {
            let x: T = ...; // ERROR: Can't access T
        };
    }
}

// ✅ FIXED Option 1: Move logic to method
impl<T> Processor<T> {
    fn process(&self) {
        self.process_internal();
    }
    
    fn process_internal(&self) -> T { ... }
}

// ✅ FIXED Option 2: Explicit capture
impl<T: Clone> Processor<T> {
    fn process(&self, template: T) {
        let closure = move || {
            let x: T = template.clone(); // OK: Captured explicitly
        };
    }
}
```

**Files to Fix**: Scan with
```bash
cargo build -p knhk-workflow-engine 2>&1 | grep "E0401" | awk '{print $2}' | sort -u
```

---

### Task 1.7: knhk-workflow-engine - Fix Context name conflicts (15 min)

**Find conflicts**:
```bash
grep -rn "use.*Context" rust/knhk-workflow-engine/src/
```

**Fix**:
```rust
// ❌ CONFLICT:
use std::task::Context;
use tower::Context;  // ERROR: name defined multiple times

// ✅ FIXED:
use std::task::Context as TaskContext;
use tower::Context as TowerContext;
```

---

### Task 1.8: knhk-workflow-engine - Add missing dependencies (15 min)

**File**: `rust/knhk-workflow-engine/Cargo.toml`

**Add**:
```toml
[dependencies]
rand = "0.8"  # For random selection in patterns
tower = { workspace = true, features = ["util"] }  # For tower::util
```

**Validation**:
```bash
cargo build -p knhk-workflow-engine 2>&1 | grep -c "unresolved import"
# Should be 0
```

---

## Phase 2: Weaver Validation (2 hours)

### Task 2.1: Schema Check
```bash
# Ensure registry/ directory exists with valid schemas
cd /Users/sac/knhk
weaver registry check -r registry/

# Expected: All schemas valid
# If errors: Fix schema definitions
```

### Task 2.2: Live Validation
```bash
# Start OTEL collector (if needed)
# Run application with telemetry
# Validate runtime telemetry

weaver registry live-check --registry registry/

# Expected: Runtime telemetry matches schema
# If errors: Fix instrumentation code
```

---

## Phase 3: Performance Validation (1 hour)

### Task 3.1: Chicago TDD Tests
```bash
cd /Users/sac/knhk
make test-chicago-v04

# Expected: All tests pass, hot path ≤8 ticks
```

### Task 3.2: Performance Benchmarks
```bash
make test-performance-v04

# Expected: No regressions, Chatman Constant respected
```

### Task 3.3: Integration Tests
```bash
make test-integration-v2

# Expected: End-to-end workflows execute successfully
```

---

## Phase 4: Final Validation (1 hour)

### Task 4.1: Full Workspace Build
```bash
cargo build --workspace --release
# Expected: 0 errors, 0 warnings
```

### Task 4.2: Full Test Suite
```bash
cargo test --workspace
# Expected: 100% pass rate
```

### Task 4.3: Clippy Validation
```bash
cargo clippy --workspace -- -D warnings
# Expected: 0 warnings (all fixed)
```

### Task 4.4: Production Deployment Check
```bash
# Verify binaries
ls -lh target/release/knhk
ls -lh target/release/knhk-workflow

# Run smoke tests
./target/release/knhk --version
./target/release/knhk-workflow --help

# Execute end-to-end RevOps workflow
cargo run --bin execute_revops -- --config config/production.toml
```

---

## Success Criteria

### Build Quality ✅
- [ ] `cargo build --workspace` succeeds with 0 errors, 0 warnings
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] `cargo test --workspace` 100% pass rate
- [ ] No `.unwrap()` or `.expect()` in production paths
- [ ] All traits dyn-compatible

### Weaver Validation ✅
- [ ] `weaver registry check -r registry/` passes
- [ ] `weaver registry live-check --registry registry/` passes
- [ ] All OTEL spans/metrics match schema

### Performance ✅
- [ ] Hot path operations ≤8 ticks (Chatman Constant)
- [ ] `make test-chicago-v04` passes
- [ ] `make test-performance-v04` passes
- [ ] No performance regressions

### Functional ✅
- [ ] CLI commands execute with real arguments
- [ ] End-to-end workflow tested
- [ ] RevOps scenario completes successfully
- [ ] `make test-integration-v2` passes

### DOCTRINE_COVENANT ✅
- [ ] Covenant 7 (Trait Design) - No async trait methods without `async_trait`
- [ ] Covenant 2 (Invariants) - Q layer validation passes
- [ ] All covenant violations documented and resolved

---

## Risk Mitigation

### If Oxigraph + RocksDB Conflict Cannot Resolve:
**Fallback**: Use separate storage backends
```toml
[features]
rdf-memory = ["oxigraph/sophia"]  # In-memory RDF store
storage-sled = ["sled"]           # Sled for KV storage
default = ["rdf-memory", "storage-sled"]
```

### If OpenTelemetry 0.21 API Too Broken:
**Fallback**: Upgrade to 0.22 or 0.23
```bash
# Check latest version
cargo search opentelemetry
# Update workspace dependencies
```

### If Workflow Engine Generic Issues Too Complex:
**Fallback**: Refactor to use trait objects instead of generics
```rust
// Instead of:
struct Executor<T: Task> { ... }

// Use:
struct Executor {
    task: Box<dyn Task>,
}
```

---

## Timeline Summary

| Phase | Duration | Dependencies | Risk |
|-------|----------|--------------|------|
| Quick Wins | 30 min | None | Low |
| Phase 1 | 4-6 hours | Compilation fixes | Medium |
| Phase 2 | 2 hours | Phase 1 complete | Low |
| Phase 3 | 1 hour | Phase 2 complete | Low |
| Phase 4 | 1 hour | Phase 3 complete | Low |
| **Total** | **8-10 hours** | Sequential | **Medium** |

**Critical Path**: Phase 1 (compilation) blocks everything
**Highest Risk**: RocksDB conflict resolution
**Highest Impact**: oxigraph feature flag (125 errors → 0)

---

## Next Steps

1. **START HERE**: Enable oxigraph feature in knhk-cli (30 min)
2. Test quick wins with `cargo fix` (15 min)
3. Tackle RocksDB conflict (2 hours)
4. Fix OpenTelemetry API issues (1 hour)
5. Resolve workflow engine issues (3 hours)
6. Run Weaver validation (2 hours)
7. Final validation and smoke tests (2 hours)

**Total Estimated Effort**: 10.75 hours to production-ready ✅
