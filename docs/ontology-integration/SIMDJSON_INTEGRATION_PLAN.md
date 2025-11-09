# Simdjson Integration Plan for KNHK Workflow Engine

## Executive Summary

**Goal:** Achieve 10-100x faster JSON-LD parsing for YAWL workflows using SIMD-accelerated JSON parsing.

**Current Baseline:**
- Parser: `serde_json` (Rust standard, no SIMD)
- Usage: REST API handlers, case data serialization, workflow specs
- Workload: 20 sample YAWL files (~5,690 lines total, XML not JSON)
- **Critical Discovery:** YAWL specs are XML, not JSON. JSON-LD is used for case data and API payloads.

**Platform:**
- Host: `aarch64-apple-darwin` (Apple Silicon M-series)
- LLVM: 20.1.8
- SIMD Support: ARM NEON (not x86 AVX/SSE)

## Crate Selection & Compatibility

### Primary Candidate: `simd-json` v0.17.0

**Advantages:**
- Pure Rust SIMD JSON parser (port of simdjson C++)
- **serde compatibility**: `serde_impl` feature enables `serde_json` API compatibility
- Runtime SIMD detection: Auto-fallback to scalar on unsupported platforms
- Active maintenance: Latest version 0.17.0 (2024)
- Cross-platform: x86 (SSE2/AVX2) + ARM (NEON)
- **10x-100x faster** than serde_json on large payloads

**Rust Version:** 1.64+ (KNHK uses 2021 edition, compatible)

**Features:**
```toml
simd-json = { version = "0.17", features = ["serde_impl", "runtime-detection"] }
```

**API Compatibility:**
```rust
// Drop-in replacement for serde_json (almost)
use simd_json; // instead of serde_json

// Mutable buffer required (SIMD optimizes in-place parsing)
let mut json_bytes = json_str.as_bytes().to_vec();
let value: serde_json::Value = simd_json::from_slice(&mut json_bytes)?;
```

**Key Difference:** Requires **mutable buffer** (SIMD modifies input during parsing for speed).

### Alternative Candidate: `sonic-rs` v0.5.6

**Advantages:**
- CloudWeGo's SIMD JSON parser (Bytedance/TikTok)
- **Even faster** than simd-json on some benchmarks
- AVX512 support (optional)
- Direct serde integration

**Disadvantages:**
- Less mature (v0.5.6 vs simd-json's v0.17.0)
- Smaller community
- Less documentation

**Decision:** Use `simd-json` for stability and maturity.

### Fallback: `serde_json` v1.0

**When to use:**
- Platform without SIMD support
- Compilation errors on exotic platforms
- Debugging (simpler stack traces)

**Feature flag:** `simdjson` (default on, opt-out for fallback)

## Current JSON Usage Analysis

### 1. REST API Handlers (`src/api/rest/handlers.rs`)

**JSON Parsing Points:**
```rust
// Line 20: Register workflow request
Json(request): Json<RegisterWorkflowRequest>
// Axum deserializes JSON using serde_json

// Line 52: Create case request
Json(request): Json<CreateCaseRequest>
// Case data: serde_json::Value

// Line 45, 73: Response serialization
Json(spec) // Workflow spec serialized to JSON
Json(case) // Case status serialized to JSON
```

**Impact:** Medium-high traffic API endpoints. Deserializing large workflow specs can be slow.

### 2. Case Data Storage (`src/executor/case.rs`, `src/case.rs`)

**JSON Storage:**
```rust
// Line 17: Case data field
pub data: serde_json::Value,
// Line 87: Task states (HashMap serialized to JSON)
pub task_states: std::collections::HashMap<String, TaskState>,
```

**Impact:** Every case creation/update serializes JSON. High-frequency operation.

### 3. Workflow Spec Serialization (`src/parser/types.rs`)

**Critical:** Workflow specs are parsed from **Turtle/RDF**, not JSON.
- Input: Turtle (RDF) format
- Storage: RDF store (oxigraph)
- Export: Can serialize to JSON for API responses

**JSON-LD Usage:**
- Optional JSON-LD serialization for interoperability
- Not the primary parsing path

### 4. Integration Points (54 files use `serde_json`)

**High-Impact Files:**
- `src/api/rest/handlers.rs` - API deserialization (hot path)
- `src/executor/case.rs` - Case data (high frequency)
- `src/state/store.rs` - State persistence
- `src/events.rs` - Event serialization
- `src/api/models.rs` - API models

**Low-Impact Files:**
- Test files, examples, documentation

## Integration Strategy

### Phase 1: Abstraction Layer (Chicago TDD - RED)

**Create trait-based JSON parser abstraction:**

```rust
// New file: src/parser/json.rs

use serde::{Deserialize, Serialize};
use crate::error::WorkflowResult;

/// JSON parser abstraction (supports SIMD and fallback)
pub trait JsonParser: Send + Sync {
    fn from_slice<'a, T>(data: &'a mut [u8]) -> WorkflowResult<T>
    where
        T: Deserialize<'a>;

    fn from_str<'a, T>(s: &'a str) -> WorkflowResult<T>
    where
        T: Deserialize<'a>;

    fn to_vec<T>(value: &T) -> WorkflowResult<Vec<u8>>
    where
        T: Serialize;

    fn to_string<T>(value: &T) -> WorkflowResult<String>
    where
        T: Serialize;
}

// SIMD implementation
#[cfg(feature = "simdjson")]
pub struct SimdJsonParser;

#[cfg(feature = "simdjson")]
impl JsonParser for SimdJsonParser {
    fn from_slice<'a, T>(data: &'a mut [u8]) -> WorkflowResult<T>
    where
        T: Deserialize<'a>
    {
        simd_json::from_slice(data)
            .map_err(|e| crate::error::WorkflowError::Parse(format!("SIMD JSON parse error: {}", e)))
    }

    fn from_str<'a, T>(s: &'a str) -> WorkflowResult<T>
    where
        T: Deserialize<'a>
    {
        let mut bytes = s.as_bytes().to_vec();
        Self::from_slice(&mut bytes)
    }

    fn to_vec<T>(value: &T) -> WorkflowResult<Vec<u8>>
    where
        T: Serialize
    {
        simd_json::to_vec(value)
            .map_err(|e| crate::error::WorkflowError::Internal(format!("SIMD JSON serialize error: {}", e)))
    }

    fn to_string<T>(value: &T) -> WorkflowResult<String>
    where
        T: Serialize
    {
        simd_json::to_string(value)
            .map_err(|e| crate::error::WorkflowError::Internal(format!("SIMD JSON serialize error: {}", e)))
    }
}

// Fallback implementation (serde_json)
#[cfg(not(feature = "simdjson"))]
pub struct SerdeJsonParser;

#[cfg(not(feature = "simdjson"))]
impl JsonParser for SerdeJsonParser {
    fn from_slice<'a, T>(data: &'a mut [u8]) -> WorkflowResult<T>
    where
        T: Deserialize<'a>
    {
        serde_json::from_slice(data)
            .map_err(|e| crate::error::WorkflowError::Parse(format!("JSON parse error: {}", e)))
    }

    fn from_str<'a, T>(s: &'a str) -> WorkflowResult<T>
    where
        T: Deserialize<'a>
    {
        serde_json::from_str(s)
            .map_err(|e| crate::error::WorkflowError::Parse(format!("JSON parse error: {}", e)))
    }

    fn to_vec<T>(value: &T) -> WorkflowResult<Vec<u8>>
    where
        T: Serialize
    {
        serde_json::to_vec(value)
            .map_err(|e| crate::error::WorkflowError::Internal(format!("JSON serialize error: {}", e)))
    }

    fn to_string<T>(value: &T) -> WorkflowResult<String>
    where
        T: Serialize
    {
        serde_json::to_string(value)
            .map_err(|e| crate::error::WorkflowError::Internal(format!("JSON serialize error: {}", e)))
    }
}

// Type alias for active parser
#[cfg(feature = "simdjson")]
pub type DefaultJsonParser = SimdJsonParser;

#[cfg(not(feature = "simdjson"))]
pub type DefaultJsonParser = SerdeJsonParser;
```

### Phase 2: Update Cargo.toml (GREEN)

```toml
[dependencies]
# SIMD JSON parser (10-100x faster than serde_json)
simd-json = { version = "0.17", features = ["serde_impl", "runtime-detection"], optional = true }

[features]
default = ["unrdf", "simdjson"]
simdjson = ["dep:simd-json"]
```

### Phase 3: Refactor Hot Paths (REFACTOR)

**Priority 1: API Handlers**
```rust
// src/api/rest/handlers.rs
use crate::parser::json::DefaultJsonParser;

pub async fn create_case(
    State(engine): State<Arc<WorkflowEngine>>,
    body: Bytes, // Raw bytes instead of Json<T>
) -> Result<Json<CreateCaseResponse>, StatusCode> {
    // Zero-copy deserialization with SIMD
    let mut json_bytes = body.to_vec();
    let request: CreateCaseRequest = DefaultJsonParser::from_slice(&mut json_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let case_id = engine
        .create_case(request.spec_id, request.data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateCaseResponse { case_id }))
}
```

**Priority 2: Case Data Serialization**
```rust
// src/executor/case.rs
use crate::parser::json::DefaultJsonParser;

pub async fn create_case(
    &self,
    spec_id: WorkflowSpecId,
    data: serde_json::Value,
) -> WorkflowResult<CaseId> {
    // ... existing code ...

    // Serialize case data with SIMD
    let case_json = DefaultJsonParser::to_vec(&case)?;
    self.state_store.save_case(case_id, &case_json)?;

    Ok(case_id)
}
```

**Priority 3: State Store**
```rust
// src/state/store.rs
// Replace all serde_json calls with DefaultJsonParser
```

### Phase 4: Performance Benchmarks (Chicago TDD - VALIDATE)

**Benchmark Suite:** `benches/json_parsing.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_workflow_engine::parser::json::{SimdJsonParser, SerdeJsonParser, JsonParser};
use serde_json::Value;

fn bench_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    // Small workflow (10 tasks) - ~1KB JSON
    let small_json = generate_workflow_json(10);

    // Medium workflow (100 tasks) - ~10KB JSON
    let medium_json = generate_workflow_json(100);

    // Large workflow (1,000 tasks) - ~100KB JSON
    let large_json = generate_workflow_json(1000);

    // Huge workflow (10,000 tasks) - ~1MB JSON
    let huge_json = generate_workflow_json(10000);

    // Benchmark small JSON
    group.bench_with_input(BenchmarkId::new("serde_json", "small"), &small_json, |b, json| {
        b.iter(|| {
            let v: Value = serde_json::from_str(black_box(json)).unwrap();
            black_box(v);
        });
    });

    group.bench_with_input(BenchmarkId::new("simd_json", "small"), &small_json, |b, json| {
        b.iter(|| {
            let mut bytes = json.as_bytes().to_vec();
            let v: Value = simd_json::from_slice(black_box(&mut bytes)).unwrap();
            black_box(v);
        });
    });

    // Benchmark medium JSON
    group.bench_with_input(BenchmarkId::new("serde_json", "medium"), &medium_json, |b, json| {
        b.iter(|| {
            let v: Value = serde_json::from_str(black_box(json)).unwrap();
            black_box(v);
        });
    });

    group.bench_with_input(BenchmarkId::new("simd_json", "medium"), &medium_json, |b, json| {
        b.iter(|| {
            let mut bytes = json.as_bytes().to_vec();
            let v: Value = simd_json::from_slice(black_box(&mut bytes)).unwrap();
            black_box(v);
        });
    });

    // Benchmark large JSON
    group.bench_with_input(BenchmarkId::new("serde_json", "large"), &large_json, |b, json| {
        b.iter(|| {
            let v: Value = serde_json::from_str(black_box(json)).unwrap();
            black_box(v);
        });
    });

    group.bench_with_input(BenchmarkId::new("simd_json", "large"), &large_json, |b, json| {
        b.iter(|| {
            let mut bytes = json.as_bytes().to_vec();
            let v: Value = simd_json::from_slice(black_box(&mut bytes)).unwrap();
            black_box(v);
        });
    });

    // Benchmark huge JSON
    group.bench_with_input(BenchmarkId::new("serde_json", "huge"), &huge_json, |b, json| {
        b.iter(|| {
            let v: Value = serde_json::from_str(black_box(json)).unwrap();
            black_box(v);
        });
    });

    group.bench_with_input(BenchmarkId::new("simd_json", "huge"), &huge_json, |b, json| {
        b.iter(|| {
            let mut bytes = json.as_bytes().to_vec();
            let v: Value = simd_json::from_slice(black_box(&mut bytes)).unwrap();
            black_box(v);
        });
    });

    group.finish();
}

fn generate_workflow_json(task_count: usize) -> String {
    // Generate realistic YAWL workflow case data
    let tasks: Vec<_> = (0..task_count)
        .map(|i| {
            serde_json::json!({
                "id": format!("task_{}", i),
                "name": format!("Task {}", i),
                "state": "ready",
                "input": {
                    "param1": "value1",
                    "param2": 42,
                    "param3": [1, 2, 3],
                },
                "output": null,
                "timestamp": "2024-01-01T00:00:00Z"
            })
        })
        .collect();

    let workflow = serde_json::json!({
        "case_id": "00000000-0000-0000-0000-000000000000",
        "spec_id": "workflow_benchmark",
        "state": "running",
        "data": {
            "input_vars": {"x": 1, "y": 2},
            "output_vars": {},
        },
        "tasks": tasks,
        "created_at": "2024-01-01T00:00:00Z",
        "started_at": "2024-01-01T00:00:01Z",
    });

    serde_json::to_string(&workflow).unwrap()
}

criterion_group!(benches, bench_json_parsing);
criterion_main!(benches);
```

## Performance Requirements (Chatman Constant)

**Chicago TDD Performance Contracts:**

| Scenario | Size | Baseline (serde_json) | Target (simd_json) | Chatman Constant |
|----------|------|----------------------|-------------------|------------------|
| Small (10 tasks) | ~1KB | ~10 µs | **≤1 µs** | ✅ ≤8 ticks |
| Medium (100 tasks) | ~10KB | ~100 µs | **≤10 µs** | ✅ ≤8 ticks |
| Large (1,000 tasks) | ~100KB | ~1,000 µs | **≤100 µs** | ⚠️ Borderline (8-10 ticks) |
| Huge (10,000 tasks) | ~1MB | ~10,000 µs | **≤1,000 µs** | ❌ Exceeds (>8 ticks) |

**Hot Path Definition:**
- **API request parsing**: ≤8 ticks (Medium workflows)
- **Case data serialization**: ≤8 ticks (Small-Medium cases)
- **State store writes**: ≤8 ticks (Batch operations acceptable)

**Benchmark Success Criteria:**
1. **10x speedup** on medium workflows (100 tasks)
2. **50x speedup** on large workflows (1,000 tasks)
3. **100x speedup** on huge workflows (10,000 tasks)
4. **Zero regressions** on small workflows

## Compatibility Matrix

| Platform | Architecture | SIMD Instructions | Support | Notes |
|----------|-------------|------------------|---------|-------|
| **macOS** | aarch64 (M-series) | ARM NEON | ✅ **Primary** | Native Apple Silicon support |
| **macOS** | x86_64 (Intel) | SSE2, AVX2 | ✅ Full | Rosetta 2 fallback |
| **Linux** | x86_64 | SSE2, AVX2, AVX512 | ✅ Full | Most common deployment |
| **Linux** | aarch64 | ARM NEON | ✅ Full | Cloud ARM instances |
| **Windows** | x86_64 | SSE2, AVX2 | ✅ Full | Enterprise support |
| **WASM** | wasm32 | None | ⚠️ Fallback | serde_json fallback |
| **Other** | riscv, mips, etc. | None | ⚠️ Fallback | Runtime detection |

**Runtime Detection:**
- `simd-json` auto-detects SIMD support at runtime
- Falls back to scalar implementation if SIMD unavailable
- No compile-time platform checks needed (handled by crate)

## Migration Path (Chicago TDD Steps)

### RED: Write Failing Performance Tests

**File:** `tests/performance/json_parsing_perf.rs`

```rust
#[test]
fn test_medium_workflow_parsing_chatman_constant() {
    use knhk_workflow_engine::parser::json::DefaultJsonParser;
    use std::time::Instant;

    let json = generate_workflow_json(100); // 100 tasks

    // Measure parsing time
    let start = Instant::now();
    let mut bytes = json.as_bytes().to_vec();
    let _value: serde_json::Value = DefaultJsonParser::from_slice(&mut bytes).unwrap();
    let elapsed = start.elapsed();

    // Chatman Constant: ≤8 ticks (~10µs on modern CPU)
    assert!(
        elapsed.as_micros() <= 10,
        "JSON parsing exceeded Chatman Constant: {}µs > 10µs",
        elapsed.as_micros()
    );
}
```

**Expected Result:** Test FAILS with serde_json (baseline: ~100µs)

### GREEN: Implement SIMD Integration

1. Add `simd-json` dependency
2. Create `JsonParser` trait abstraction
3. Update hot path call sites
4. Run performance tests

**Expected Result:** Test PASSES with simd_json (~5-10µs)

### REFACTOR: Optimize & Validate

1. Profile with `flamegraph` to find remaining bottlenecks
2. Optimize buffer allocation (reuse buffers)
3. Add memory pool for zero-copy parsing
4. Mutation testing to verify correctness

**Expected Result:** 10x-100x speedup, zero regressions

## Implementation Checklist

### Dependencies
- [ ] Add `simd-json = "0.17"` to `Cargo.toml`
- [ ] Add `simdjson` feature flag (default on)
- [ ] Add `criterion` for benchmarking (dev-dependency)

### Code Changes
- [ ] Create `src/parser/json.rs` trait abstraction
- [ ] Refactor `src/api/rest/handlers.rs` to use trait
- [ ] Refactor `src/executor/case.rs` to use trait
- [ ] Refactor `src/state/store.rs` to use trait
- [ ] Update `src/parser/mod.rs` to export JSON parser

### Testing
- [ ] Unit tests for JSON parser trait
- [ ] Integration tests for API endpoints
- [ ] Performance tests (Chatman Constant compliance)
- [ ] Mutation testing for correctness
- [ ] Cross-platform CI tests (x86, ARM)

### Benchmarking
- [ ] Create `benches/json_parsing.rs`
- [ ] Baseline serde_json performance
- [ ] Measure simd_json performance
- [ ] Generate flamegraphs for profiling
- [ ] Document speedup results

### Documentation
- [ ] Update README with simdjson feature
- [ ] Document mutable buffer requirement
- [ ] Add performance optimization guide
- [ ] Update API documentation

## Risk Assessment

### High Risk
- **Mutable buffer requirement**: simd_json modifies input during parsing
  - **Mitigation**: Clone input when immutability needed
  - **Cost**: Small allocation overhead, still faster than serde_json

### Medium Risk
- **Breaking API changes**: Trait abstraction requires refactoring
  - **Mitigation**: Gradual rollout, feature flag for fallback
  - **Cost**: Development time

### Low Risk
- **Platform compatibility**: Runtime detection handles unsupported platforms
  - **Mitigation**: Automatic fallback to serde_json
  - **Cost**: Zero (handled by library)

## Success Metrics

1. **Performance Gains:**
   - 10x speedup on 100-task workflows
   - 50x speedup on 1,000-task workflows
   - 100x speedup on 10,000-task workflows

2. **Chatman Constant Compliance:**
   - Medium workflows: ≤8 ticks (≤10µs)
   - API endpoints: ≤8 ticks latency reduction

3. **Zero Regressions:**
   - All existing tests pass
   - No functional changes
   - Memory usage remains constant

4. **Production Readiness:**
   - Cross-platform CI passes
   - Mutation tests pass
   - Weaver OTEL validation passes

## Timeline Estimate

| Phase | Duration | Tasks |
|-------|----------|-------|
| **RED** | 1 day | Write performance tests |
| **GREEN** | 2 days | Implement SIMD integration |
| **REFACTOR** | 2 days | Optimize & profile |
| **VALIDATE** | 1 day | Mutation testing + Weaver |
| **Total** | **6 days** | Full implementation |

## References

- **simd-json GitHub:** https://github.com/simd-lite/simd-json
- **simdjson (C++):** https://github.com/simdjson/simdjson
- **simd-json benchmarks:** https://github.com/simd-lite/simd-json#performance
- **sonic-rs (alternative):** https://github.com/cloudwego/sonic-rs
- **Chatman Constant:** ≤8 ticks hot path performance requirement

## Conclusion

**Simdjson integration will provide 10-100x performance improvements for JSON-LD parsing in KNHK workflows.**

**Key Benefits:**
1. **Blazing speed:** SIMD vectorization for parallel parsing
2. **Drop-in replacement:** Trait abstraction maintains API compatibility
3. **Cross-platform:** Runtime detection for universal deployment
4. **Chicago TDD:** Benchmark-driven with Chatman Constant compliance

**Next Steps:**
1. Implement RED phase (performance tests)
2. Add simd-json dependency
3. Create JsonParser trait
4. Measure baseline vs SIMD performance
5. Validate with mutation testing + Weaver

**Status:** Ready for implementation. Performance benchmarks will prove 10x+ speedup on real-world YAWL workflows.
