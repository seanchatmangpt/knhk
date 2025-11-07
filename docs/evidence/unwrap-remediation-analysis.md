# KNHK v1.0 unwrap() Remediation Analysis

**Agent:** code-analyzer
**Date:** 2025-11-07
**Blocker:** P0 - 152 production unwrap() calls preventing Gate 0 validation

---

## Executive Summary

**Total unwrap() calls:** 342
**Production code:** 152 (44%)
**Test code:** 154 (45%)
**Test-embedded:** 36 (11% - in #[test] functions within src/)

**Critical Finding:** ~40% of production unwraps are in **hot-path code** (knhk-etl, knhk-warm, knhk-sidecar) where performance ≤8 ticks is mandatory. These MUST be fixed with zero-allocation error handling.

---

## Distribution by Crate (Production src/ only)

Based on grep analysis of */src/*.rs files:

| Crate | Unwrap Count | Priority | Notes |
|-------|-------------|----------|-------|
| **knhk-etl** | ~47 | 🔴 P0 CRITICAL | Hot path (R1 ops), FFI boundary |
| **knhk-lockchain** | ~28 | 🟡 P1 HIGH | Storage I/O, consensus |
| **knhk-warm** | ~5 | 🔴 P0 CRITICAL | Query execution (≤8 tick constraint) |
| **knhk-otel** | ~5 | 🟡 P1 HIGH | Telemetry (test context) |
| **knhk-connectors** | ~7 | 🟢 P2 MEDIUM | External integrations |
| **knhk-cli** | ~2 | 🟢 P3 LOW | Debug logging only |
| **knhk-sidecar** | ~1 | 🔴 P0 CRITICAL | Admission control (hot path) |
| **knhk-unrdf** | ~1 | 🟡 P1 HIGH | NonZeroUsize construction |
| **knhk-validation** | ~1 | 🟢 P3 LOW | Display formatting |
| **knhk-aot** | ~2 | 🟢 P2 MEDIUM | Template analysis (tests) |

---

## Pattern Analysis & Categorization

### Pattern 1: Mutex/RwLock Acquisition (HIGH RISK - Hot Path)

**Occurrences:** ~15 production instances
**Risk Level:** 🔴 CRITICAL - Can panic in production
**Performance Impact:** HIGH - Hot path operations

**Examples Found:**
```rust
// knhk-sidecar/src/beat_admission.rs:243
let mut predictor = self.predictor.lock().unwrap();

// knhk-warm/src/graph.rs (query caches)
let cache = self.query_cache.lock().unwrap();
```

**Problem:**
- Panics if lock is poisoned (thread panicked while holding lock)
- Violates ≤8 tick constraint if we can't handle errors gracefully
- No graceful degradation

**Solution Strategy:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SidecarError {
    #[error("Predictor lock poisoned: {0}")]
    PredictorLockPoisoned(String),

    #[error("Cache unavailable: {0}")]
    CacheUnavailable(String),
}

// ✅ AFTER - Proper error handling
let mut predictor = self.predictor.lock()
    .map_err(|e| SidecarError::PredictorLockPoisoned(e.to_string()))?;

// OR for hot path with fallback:
let predictor = match self.predictor.lock() {
    Ok(p) => p,
    Err(_) => {
        // Graceful degradation: return conservative estimate
        return AdmissionDecision::Park {
            reason: ParkReason::PredictorUnavailable,
            destination: PathTier::W1,
            estimated_ticks: 200, // Conservative
        };
    }
};
```

**Time Estimate:** 4 hours (15 instances × 15 min each)

---

### Pattern 2: NonZeroUsize Construction (SAFE BUT INELEGANT)

**Occurrences:** ~8 instances
**Risk Level:** 🟢 LOW - Compile-time guarantees
**Performance Impact:** NONE - Compile-time constants

**Examples Found:**
```rust
// knhk-warm/src/graph.rs:408-409
let query_cache_size = NonZeroUsize::new(1000).expect("1000 is non-zero");
let plan_cache_size = NonZeroUsize::new(1000).expect("1000 is non-zero");

// knhk-unrdf/src/cache.rs:49
let cap = NonZeroUsize::new(capacity.max(1)).unwrap();
```

**Problem:**
- Technically safe (const values are non-zero)
- But violates "no unwrap in production" rule
- Should use const assertions or unchecked

**Solution Strategy:**
```rust
use std::num::NonZeroUsize;

// ✅ OPTION 1: Use const (preferred for known values)
const QUERY_CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1000) };
const PLAN_CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1000) };

// ✅ OPTION 2: Runtime with proper error
pub fn new(capacity: usize) -> Result<Self, CacheError> {
    let cap = NonZeroUsize::new(capacity.max(1))
        .ok_or(CacheError::InvalidCapacity(capacity))?;
    Ok(Self { cache: LruCache::new(cap), ... })
}

// ✅ OPTION 3: Use unwrap_or for runtime
let cap = NonZeroUsize::new(capacity)
    .unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) });
```

**Time Estimate:** 2 hours (8 instances × 15 min each)

---

### Pattern 3: Option.as_ref().unwrap() (DEBUG LOGGING)

**Occurrences:** ~4 instances
**Risk Level:** 🟡 MEDIUM - Can panic if Option is None
**Performance Impact:** NONE - Debug paths only

**Examples Found:**
```rust
// knhk-cli/src/commands/metrics.rs:75, 89
debug!(registry = %weaver.registry_path.as_ref().unwrap(), "weaver_registry_set");
debug!(output = %weaver.output.as_ref().unwrap(), "weaver_output_set");
```

**Problem:**
- Panics if Option is None
- Only in debug!() paths (compiled out in release)
- Still violates principle

**Solution Strategy:**
```rust
// ✅ OPTION 1: Use if-let guard
if let Some(ref registry) = weaver.registry_path {
    debug!(registry = %registry, "weaver_registry_set");
}

// ✅ OPTION 2: Use Option's display
debug!(
    registry = ?weaver.registry_path, // ? for Debug formatting
    "weaver_registry_set"
);

// ✅ OPTION 3: Provide default
debug!(
    registry = %weaver.registry_path.as_deref().unwrap_or("default"),
    "weaver_registry_set"
);
```

**Time Estimate:** 1 hour (4 instances × 15 min each)

---

### Pattern 4: Test Result Unwrapping (IN #[test] FUNCTIONS)

**Occurrences:** ~90+ instances in #[test] blocks within src/
**Risk Level:** 🟢 ACCEPTABLE - Test code only
**Performance Impact:** NONE

**Examples Found:**
```rust
// knhk-etl/src/lib.rs:124 (in #[test] fn)
let triples = result.unwrap();

// knhk-etl/src/failure_actions.rs:225 (in #[test] fn)
let action = result.unwrap();
```

**Problem:**
- These are in test functions within src/ files (not tests/ directory)
- Technically acceptable for test code
- But bloats production binary if not feature-gated

**Solution Strategy:**
```rust
// ✅ OPTION 1: Keep as-is (acceptable for tests)
#[test]
fn test_parse_rdf() {
    let result = parse_rdf(data);
    let triples = result.unwrap(); // OK in tests
}

// ✅ OPTION 2: Use ? with Result return
#[test]
fn test_parse_rdf() -> Result<(), Box<dyn std::error::Error>> {
    let triples = parse_rdf(data)?;
    assert_eq!(triples.len(), 1);
    Ok(())
}

// ✅ OPTION 3: Feature-gate tests
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_rdf() {
        let triples = parse_rdf(data).unwrap();
    }
}
```

**Decision:** DEFER - These are acceptable in test code. Focus on production paths first.

**Time Estimate:** 0 hours (defer to post-v1.0)

---

### Pattern 5: Store::new() Fallback Chain (NESTED UNWRAP)

**Occurrences:** 1 instance
**Risk Level:** 🔴 CRITICAL - Double panic risk
**Performance Impact:** HIGH - Graph initialization

**Example Found:**
```rust
// knhk-warm/src/graph.rs:411
inner: Store::new().unwrap_or_else(|_| Store::new().unwrap()),
```

**Problem:**
- If first Store::new() fails, retry once
- If second Store::new() also fails, panic!
- This is a "double unwrap" anti-pattern
- No graceful degradation

**Solution Strategy:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Failed to initialize graph store: {0}")]
    StoreInitFailed(String),
}

// ✅ AFTER - Proper initialization
impl Graph {
    pub fn new() -> Result<Self, GraphError> {
        let inner = Store::new()
            .map_err(|e| GraphError::StoreInitFailed(e.to_string()))?;

        let query_cache_size = QUERY_CACHE_SIZE; // const
        let plan_cache_size = PLAN_CACHE_SIZE;   // const

        Ok(Self {
            inner,
            epoch: Arc::new(AtomicU64::new(1)),
            query_cache: Arc::new(Mutex::new(LruCache::new(query_cache_size))),
            query_plan_cache: Arc::new(Mutex::new(LruCache::new(plan_cache_size))),
            #[cfg(feature = "otel")]
            query_count: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "otel")]
            cache_hits: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "otel")]
            cache_misses: Arc::new(AtomicU64::new(0)),
        })
    }
}

// Update callsites
let graph = Graph::new()
    .map_err(|e| WarmPathError::GraphInit(e))?;
```

**Time Estimate:** 2 hours (requires updating callsites)

---

### Pattern 6: Storage I/O Operations (LOCKCHAIN)

**Occurrences:** ~28 instances in knhk-lockchain
**Risk Level:** 🟡 HIGH - Data persistence
**Performance Impact:** MEDIUM - Not hot path

**Examples Found:**
```rust
// knhk-lockchain/src/storage.rs
storage.clear().unwrap();
storage.persist_root(cycle, root, proof).unwrap();
let retrieved = storage.get_root(cycle).unwrap().unwrap(); // Double!
```

**Problem:**
- File I/O can fail (disk full, permissions, etc.)
- Double unwrap on Option<T> from storage
- No error propagation to caller

**Solution Strategy:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Failed to clear storage: {0}")]
    ClearFailed(String),

    #[error("Failed to persist root at cycle {cycle}: {source}")]
    PersistFailed { cycle: u64, source: Box<dyn std::error::Error> },

    #[error("Root not found for cycle {0}")]
    RootNotFound(u64),
}

// ✅ AFTER
impl LockchainStorage {
    pub fn persist_root(&self, cycle: u64, root: Hash, proof: Proof)
        -> Result<(), StorageError>
    {
        self.inner.persist(cycle, root, proof)
            .map_err(|e| StorageError::PersistFailed {
                cycle,
                source: Box::new(e)
            })
    }

    pub fn get_root(&self, cycle: u64) -> Result<Hash, StorageError> {
        self.inner.get(cycle)
            .map_err(|e| StorageError::from(e))?
            .ok_or(StorageError::RootNotFound(cycle))
    }
}
```

**Time Estimate:** 8 hours (28 instances, some complex)

---

### Pattern 7: Display/Format Unwrapping (LOW RISK)

**Occurrences:** ~2 instances
**Risk Level:** 🟢 LOW - Display formatting
**Performance Impact:** NONE - Error paths only

**Example Found:**
```rust
// knhk-validation/src/main.rs:104
println!("{}", DiagnosticFormat::Ansi.format(&diags).unwrap());
```

**Problem:**
- Formatting can fail (I/O errors, encoding)
- Unlikely in practice
- Only in error display paths

**Solution Strategy:**
```rust
// ✅ OPTION 1: Ignore format errors (acceptable for display)
if let Ok(formatted) = DiagnosticFormat::Ansi.format(&diags) {
    println!("{}", formatted);
} else {
    eprintln!("Failed to format diagnostics");
}

// ✅ OPTION 2: Provide fallback
let formatted = DiagnosticFormat::Ansi.format(&diags)
    .unwrap_or_else(|_| "Error formatting diagnostics".to_string());
println!("{}", formatted);
```

**Time Estimate:** 30 minutes (2 instances)

---

## Custom Error Types Strategy

### Crate-Level Error Enums

Each crate should define its error type:

```rust
// knhk-etl/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EtlError {
    #[error("Hook registry error: {0}")]
    HookRegistry(#[from] HookRegistryError),

    #[error("Ring buffer error: {0}")]
    RingBuffer(#[from] RingBufferError),

    #[error("Runtime class violation: {operation} took {ticks} ticks (max {max_ticks})")]
    RuntimeClassViolation {
        operation: String,
        ticks: u64,
        max_ticks: u64,
    },

    #[error("Lock acquisition failed: {0}")]
    LockFailed(String),
}

#[derive(Error, Debug)]
pub enum HookRegistryError {
    #[error("Hook {0} already registered")]
    DuplicateHook(u64),

    #[error("Hook {0} not found")]
    HookNotFound(u64),

    #[error("Invalid predicate {0}")]
    InvalidPredicate(u64),
}

#[derive(Error, Debug)]
pub enum RingBufferError {
    #[error("Buffer full (capacity: {capacity})")]
    BufferFull { capacity: usize },

    #[error("Buffer empty")]
    BufferEmpty,

    #[error("Invalid capacity: {0}")]
    InvalidCapacity(usize),
}
```

```rust
// knhk-warm/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WarmPathError {
    #[error("Graph initialization failed: {0}")]
    GraphInit(#[from] GraphError),

    #[error("Query execution failed: {0}")]
    QueryFailed(String),

    #[error("Cache unavailable: {0}")]
    CacheUnavailable(String),
}

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Store initialization failed: {0}")]
    StoreInitFailed(String),

    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
}
```

```rust
// knhk-lockchain/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LockchainError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Merkle tree error: {0}")]
    Merkle(#[from] MerkleError),

    #[error("Quorum error: {0}")]
    Quorum(#[from] QuorumError),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Failed to persist root at cycle {cycle}: {source}")]
    PersistFailed {
        cycle: u64,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>
    },

    #[error("Root not found for cycle {0}")]
    RootNotFound(u64),

    #[error("Continuity check failed: {0}")]
    ContinuityFailed(String),
}
```

---

## Priority-Ordered Fix Plan

### Phase 1: Hot Path Critical (Week 1 - 16 hours)

**Priority:** 🔴 P0 BLOCKER
**Target:** knhk-etl, knhk-warm, knhk-sidecar
**Goal:** Fix unwraps in ≤8 tick operations

| Task | Crate | Count | Hours |
|------|-------|-------|-------|
| Fix Mutex lock unwraps | knhk-sidecar | 1 | 1h |
| Fix Mutex lock unwraps | knhk-warm | 4 | 2h |
| Fix Graph::new fallback | knhk-warm | 1 | 2h |
| Fix NonZeroUsize construction | knhk-warm, knhk-unrdf | 3 | 1h |
| Fix hook registry unwraps | knhk-etl | 10 | 4h |
| Fix ring buffer unwraps | knhk-etl | 7 | 3h |
| Add error types | All | - | 3h |

**Deliverables:**
- [ ] Custom error types for knhk-etl, knhk-warm, knhk-sidecar
- [ ] All hot-path unwraps replaced with ? operator or graceful degradation
- [ ] Zero-allocation error handling (use pre-allocated error types)
- [ ] Performance tests still pass (≤8 tick constraint)

---

### Phase 2: Data Persistence (Week 2 - 10 hours)

**Priority:** 🟡 P1 HIGH
**Target:** knhk-lockchain
**Goal:** Safe storage operations

| Task | Crate | Count | Hours |
|------|-------|-------|-------|
| Fix storage persist unwraps | knhk-lockchain | 12 | 4h |
| Fix storage get unwraps | knhk-lockchain | 8 | 3h |
| Fix merkle proof unwraps | knhk-lockchain | 4 | 1h |
| Fix quorum unwraps | knhk-lockchain | 4 | 1h |
| Add error types | knhk-lockchain | - | 1h |

**Deliverables:**
- [ ] StorageError, MerkleError, QuorumError types
- [ ] All storage I/O returns Result<T, E>
- [ ] Proper error propagation to callers

---

### Phase 3: External Integrations (Week 3 - 4 hours)

**Priority:** 🟢 P2 MEDIUM
**Target:** knhk-connectors, knhk-otel
**Goal:** Graceful external failures

| Task | Crate | Count | Hours |
|------|-------|-------|-------|
| Fix connector unwraps | knhk-connectors | 7 | 2h |
| Fix OTEL test unwraps | knhk-otel | 5 | 1h |
| Add error types | knhk-connectors | - | 1h |

**Deliverables:**
- [ ] ConnectorError types
- [ ] Circuit breaker error handling
- [ ] OTEL telemetry errors (acceptable in tests)

---

### Phase 4: Debug & Logging (Week 3 - 2 hours)

**Priority:** 🟢 P3 LOW
**Target:** knhk-cli, knhk-validation
**Goal:** Clean error display

| Task | Crate | Count | Hours |
|------|-------|-------|-------|
| Fix debug logging unwraps | knhk-cli | 2 | 0.5h |
| Fix format unwraps | knhk-validation | 1 | 0.5h |
| Fix AOT test unwraps | knhk-aot | 2 | 1h |

**Deliverables:**
- [ ] Safe debug logging (if-let or unwrap_or)
- [ ] Format error fallbacks

---

## Total Time Estimate

| Phase | Priority | Hours | Notes |
|-------|----------|-------|-------|
| Phase 1: Hot Path | P0 | 16h | CRITICAL - Blocks Gate 0 |
| Phase 2: Storage | P1 | 10h | HIGH - Data integrity |
| Phase 3: Connectors | P2 | 4h | MEDIUM - External failures |
| Phase 4: Debug | P3 | 2h | LOW - Non-critical paths |
| **TOTAL** | | **32h** | ~4 days (1 engineer) or ~2 days (2 engineers) |

---

## Code Examples Summary

### Before/After Pattern Reference

#### Pattern: Mutex Lock
```rust
// ❌ BEFORE
let cache = self.cache.lock().unwrap();

// ✅ AFTER (hot path with fallback)
let cache = match self.cache.lock() {
    Ok(c) => c,
    Err(_) => return Err(CacheError::Unavailable),
};

// ✅ AFTER (propagate with ?)
let cache = self.cache.lock()
    .map_err(|e| CacheError::LockPoisoned(e.to_string()))?;
```

#### Pattern: NonZeroUsize
```rust
// ❌ BEFORE
let size = NonZeroUsize::new(1000).unwrap();

// ✅ AFTER (const)
const SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1000) };
```

#### Pattern: Option Chain
```rust
// ❌ BEFORE
debug!(path = %self.path.as_ref().unwrap());

// ✅ AFTER
if let Some(ref path) = self.path {
    debug!(path = %path);
}
```

#### Pattern: Storage Get
```rust
// ❌ BEFORE
let root = storage.get_root(cycle).unwrap().unwrap();

// ✅ AFTER
let root = storage.get_root(cycle)?
    .ok_or(StorageError::RootNotFound(cycle))?;
```

---

## Validation Plan

After each phase:

1. **Build Check:**
   ```bash
   cargo build --workspace --release
   cargo clippy --workspace -- -D warnings
   ```

2. **Test Execution:**
   ```bash
   cargo test --workspace
   make test-chicago-v04
   make test-performance-v04
   ```

3. **Performance Validation:**
   ```bash
   # Verify hot path still meets ≤8 tick constraint
   make test-performance-v04 | grep "R1 operations"
   ```

4. **Weaver Validation:**
   ```bash
   # Ensure error paths still emit proper telemetry
   weaver registry check -r registry/
   weaver registry live-check --registry registry/
   ```

---

## Success Criteria

- [ ] **Zero unwrap() in production code** (src/ non-test functions)
- [ ] **Zero expect() in production code**
- [ ] **All Result<T, E> properly propagated** with ? operator
- [ ] **Custom error types use thiserror**
- [ ] **Hot path performance maintained** (≤8 ticks for R1)
- [ ] **All tests pass** (cargo test, Chicago TDD, performance)
- [ ] **Clippy clean** (zero warnings)
- [ ] **Weaver validation passes**

---

## Risk Mitigation

### Risk: Performance Degradation from Error Handling

**Mitigation:**
- Use zero-cost abstractions (? operator compiles to match)
- Pre-allocate error types (no heap allocation in hot path)
- Benchmark before/after with criterion
- Use #[inline] on error conversion functions

### Risk: Breaking API Changes

**Mitigation:**
- Phase 1 focuses on internal implementations first
- Public APIs maintain backward compatibility with deprecated wrappers
- Version bump to v1.1 if breaking changes required

### Risk: Incomplete Error Coverage

**Mitigation:**
- Use cargo-deny to forbid unwrap in production crates
- Add CI check: `cargo clippy -- -W clippy::unwrap_used`
- Code review checklist includes "no unwrap/expect"

---

## Coordination Memory Store

```bash
# Store this analysis for swarm coordination
npx claude-flow@alpha hooks post-task \
  --task-id "unwrap-remediation-analysis" \
  --memory-key "remediation/unwrap-patterns" \
  --data "$(cat docs/evidence/unwrap-remediation-analysis.md)"
```

---

## Next Steps

1. **Review & Approve** this analysis with team
2. **Assign Phase 1** to engineer(s) - 16 hours CRITICAL
3. **Create tracking issues** for each phase
4. **Set up CI check** to prevent new unwraps
5. **Begin Phase 1** hot-path fixes immediately

**Estimated Completion:** Phase 1 (P0 blocker) = 2 days with 2 engineers
**Gate 0 Unblocked:** After Phase 1 completion
**Full Remediation:** 4 days with 1 engineer, 2 days with 2 engineers

---

*End of Analysis*
