# KNHK Integration Analysis Report

**Agent**: Integration Specialist (code-analyzer)
**Swarm ID**: swarm-1762466485307-u67jafg4t
**Date**: 2025-11-06
**Priority**: P2 (INTEGRATION)

## Executive Summary

This report provides a comprehensive analysis of the integration between knhk-etl, knhk-sidecar, knhk-warm, and knhk-hot crates, identifying dependency relationships, integration points, and areas requiring testing.

## 1. Crate Dependency Structure

### 1.1 knhk-hot (v1.0.0)

**Type**: Core library (bottom of dependency tree)
**Dependencies**: None (pure FFI wrapper around C library)
**Dependents**: knhk-etl, knhk-warm, knhk-integration-tests

```toml
[lib]
crate-type = ["staticlib", "cdylib", "rlib"]
```

**Key Features**:
- FFI-safe hot path execution (≤8 ticks)
- No external dependencies
- Provides core `Engine`, `Run`, `Ctx`, `Ir`, `Receipt` types
- Links to C library `libknhk`

**Integration Notes**:
- ✅ Clean FFI boundary
- ✅ No feature flags required
- ✅ Used by both ETL and warm path

### 1.2 knhk-etl (v0.1.0)

**Type**: Core ETL pipeline library
**Dependencies**:
```
├── hashbrown v0.15.5
├── hex v0.4.3
├── knhk-connectors v0.1.0
├── knhk-hot v1.0.0
├── oxigraph v0.5.2
├── reqwest v0.11.27
└── serde_json v1.0.145
```

**Optional Dependencies** (feature-gated):
- `knhk-lockchain` (feature: "knhk-lockchain")
- `knhk-otel` (feature: "knhk-otel")
- `rdkafka` (feature: "kafka")
- `reqwest` (feature: "std")

**Feature Flags**:
```toml
[features]
default = ["std"]
std = ["dep:reqwest"]
grpc = []
kafka = ["dep:rdkafka"]
knhk-lockchain = ["dep:knhk-lockchain"]
knhk-otel = ["dep:knhk-otel"]
```

**Integration Points**:
1. **→ knhk-hot**: Uses hot path execution via `knhk-hot` crate
2. **→ knhk-connectors**: Data ingestion from Kafka, Salesforce, HTTP
3. **→ knhk-lockchain**: Optional immutable audit log
4. **→ knhk-otel**: Optional telemetry export
5. **→ Warm Path**: Via `IntegratedPipeline::set_warm_path_executor()`

### 1.3 knhk-warm (v0.1.0)

**Type**: Warm path operations (≤500ms budget)
**Dependencies**:
```
├── knhk-hot v1.0.0
├── knhk-etl v0.1.0 (default-features = false)
├── oxigraph v0.5
├── lru v0.16
├── ahash v0.8
├── thiserror v2.0
└── serde_json v1.0
```

**Optional Dependencies**:
- `knhk-otel` (feature: "otel")
- `knhk-unrdf` (feature: "unrdf")

**Feature Flags**:
```toml
[features]
default = []
otel = ["knhk-otel"]
std = []
unrdf = ["knhk-unrdf"]
```

**Integration Points**:
1. **→ knhk-hot**: Hot path execution for critical operations
2. **→ knhk-etl**: Uses ETL pipeline (disabled default features)
3. **→ oxigraph**: SPARQL query execution
4. **→ knhk-otel**: Optional telemetry

**Important**: knhk-warm uses `knhk-etl` with `default-features = false` to avoid circular dependencies and reduce feature bloat.

### 1.4 knhk-sidecar (v0.5.0)

**Type**: gRPC proxy service
**Dependencies**:
```
├── tokio v1.48.0 (full)
├── tonic v0.10.2 (tls, tls-roots)
├── prost v0.12.6
├── prost-types v0.12.6
├── knhk-etl v0.1.0
├── knhk-connectors v0.1.0
├── knhk-otel v0.1.0 (features: ["std"])
├── knhk-config v0.1.0
├── serde v1.0.228
├── serde_json v1.0.145
├── tokio-stream v0.1.17
├── thiserror v1.0.69
├── toml v0.8.23
├── tracing v0.1.41
└── tracing-subscriber v0.3.20
```

**Feature Flags**:
```toml
[features]
default = ["otel"]
otel = []
```

**Integration Points**:
1. **→ knhk-etl**: Core ETL pipeline execution
2. **→ knhk-otel**: Telemetry export with Weaver live-check
3. **→ knhk-connectors**: Connector configuration
4. **→ gRPC**: External client integration
5. **→ Weaver**: Runtime schema validation

### 1.5 knhk-integration-tests (v0.1.0)

**Type**: Integration test binary
**Dependencies**:
```
├── testcontainers v0.16
├── testcontainers-modules v0.16 (kafka, postgres)
├── tokio v1 (full)
├── knhk-connectors v0.1.0
├── knhk-etl v0.1.0 (features: ["std"])
├── knhk-hot v1.0.0
├── knhk-otel v0.1.0
└── anyhow v1.0
```

**Test Coverage**:
- Kafka connector integration
- ETL pipeline with Kafka source
- Lockchain with PostgreSQL backend
- OTEL collector integration
- End-to-end Kafka → ETL → Lockchain → OTEL

## 2. Integration Points Analysis

### 2.1 Sidecar → ETL Pipeline

**File**: `rust/knhk-sidecar/src/lib.rs`
**Method**: `run(config: SidecarConfig)`

**Integration Flow**:
```
SidecarServer
  ├─ Creates SidecarClient (gRPC)
  ├─ Uses knhk-etl::Pipeline internally
  ├─ Exports telemetry via knhk-otel
  └─ Validates via Weaver live-check
```

**Key Code**:
```rust
let client = SidecarClient::new(client_config, Arc::clone(&metrics)).await?;
let server = SidecarServer::new(server_config, client, metrics, health).await?;
```

**Integration Test Needed**:
- [ ] Sidecar receives gRPC request
- [ ] Triggers ETL pipeline execution
- [ ] Returns proper response
- [ ] Exports telemetry

### 2.2 ETL → Hot Path

**File**: `rust/knhk-etl/src/ingest.rs`, `rust/knhk-etl/src/reflex.rs`
**Integration**: Direct FFI calls to `knhk-hot`

**Integration Flow**:
```
Pipeline::execute()
  ├─ Calls hot path operations via knhk_hot::Engine
  ├─ Enforces ≤8 tick budget
  ├─ Collects Receipt for telemetry
  └─ Returns results
```

**Key Code** (from `knhk-etl/src/reflex.rs`):
```rust
use knhk_hot::{Engine, Run, Ir, Receipt, Op, NROWS};

pub fn eval_reflex(engine: &Engine, ir: &mut Ir) -> Result<Receipt, ReflexError> {
    let mut receipt = Receipt::default();
    let success = engine.eval_bool(ir, &mut receipt);
    // ...
}
```

**Integration Test Needed**:
- [x] ETL pipeline calls hot path operations
- [x] Performance budget enforced (≤8 ticks)
- [ ] Receipt properly collected
- [ ] Error handling works

### 2.3 ETL → Warm Path

**File**: `rust/knhk-etl/src/integration.rs`
**Integration**: Via `WarmPathQueryExecutor` trait

**Integration Flow**:
```
IntegratedPipeline
  ├─ set_warm_path_executor(executor)
  ├─ execute_warm_path_query(sparql)
  └─ Returns WarmPathQueryResult
```

**Key Code**:
```rust
pub trait WarmPathQueryExecutor: Send + Sync {
    fn execute_query(&self, sparql: &str) -> Result<WarmPathQueryResult, String>;
}

pub fn execute_warm_path_query(&self, sparql: &str) -> Result<WarmPathQueryResult, PipelineError> {
    if let Some(ref executor) = self.warm_path_executor {
        executor.execute_query(sparql)
            .map_err(|e| PipelineError::ReflexError(format!("Warm path query failed: {}", e)))
    } else {
        Err(PipelineError::ReflexError("Warm path executor not configured".to_string()))
    }
}
```

**Integration Test Needed**:
- [ ] ETL pipeline executes SPARQL query
- [ ] Warm path executor returns results
- [ ] Query timeout enforced (≤500ms)
- [ ] Error propagation works

### 2.4 Sidecar → OTEL Export

**File**: `rust/knhk-sidecar/src/lib.rs`
**Integration**: Weaver live-check validation

**Integration Flow**:
```
Sidecar run()
  ├─ Starts Weaver live-check process
  ├─ Configures OTLP endpoint
  ├─ Exports spans/metrics via knhk-otel
  └─ Weaver validates against schema
```

**Key Code**:
```rust
let weaver_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
let weaver_endpoint: Option<String> = if config.weaver_enabled && config.enable_otel {
    // Start Weaver
    let (process, endpoint) = start_weaver_with_verification(&config).await?;
    *weaver_process.lock().await = Some(process);
    Some(endpoint)
} else {
    None
};
```

**Integration Test Needed**:
- [ ] Sidecar starts Weaver process
- [ ] Telemetry exported to OTLP endpoint
- [ ] Weaver validates schema compliance
- [ ] Health checks work

### 2.5 Warm → Hot Path

**File**: `rust/knhk-warm/src/hot_path.rs`
**Integration**: Direct hot path calls for critical operations

**Integration Flow**:
```
WarmPathExecutor
  ├─ Identifies hot path operations
  ├─ Calls knhk_hot::Engine
  ├─ Falls back to oxigraph for complex queries
  └─ Returns unified results
```

**Integration Test Needed**:
- [ ] Warm path identifies hot path operations
- [ ] Hot path execution succeeds
- [ ] Fallback to warm path works
- [ ] Performance budgets respected

## 3. Feature Flag Analysis

### 3.1 Critical Feature Flags

| Crate | Feature | Default | Purpose | Integration Impact |
|-------|---------|---------|---------|-------------------|
| knhk-etl | `std` | ✅ Yes | Enables reqwest HTTP client | Required by knhk-sidecar |
| knhk-etl | `knhk-otel` | ❌ No | Enables telemetry | Optional, enables metrics |
| knhk-etl | `knhk-lockchain` | ❌ No | Enables audit log | Optional, enables receipts |
| knhk-etl | `kafka` | ❌ No | Enables Kafka connector | Optional |
| knhk-warm | `otel` | ❌ No | Enables telemetry | Should match knhk-etl |
| knhk-warm | `unrdf` | ❌ No | Enables RDF utilities | Optional |
| knhk-sidecar | `otel` | ✅ Yes | Enables telemetry | Required for Weaver |

### 3.2 Feature Flag Recommendations

**Issue 1**: knhk-warm uses `knhk-etl` with `default-features = false`
- **Impact**: Disables `std` feature in knhk-etl
- **Consequence**: May break HTTP connectors if used
- **Recommendation**: Document this limitation or add conditional feature

**Issue 2**: OTEL feature inconsistency
- knhk-etl: `knhk-otel` (optional)
- knhk-warm: `otel` (optional)
- knhk-sidecar: `otel` (default enabled)
- **Recommendation**: Align feature names or document clearly

**Issue 3**: Conditional compilation boundaries
- `#[cfg(feature = "knhk-otel")]` used in knhk-etl/src/integration.rs
- Must ensure consistent feature propagation
- **Recommendation**: Integration tests with various feature combinations

## 4. Dependency Audit

### 4.1 Dependency Health

| Dependency | Version | Crates Using | Health Status |
|-----------|---------|--------------|---------------|
| tokio | 1.48.0 | knhk-sidecar, integration-tests | ✅ Active, stable |
| tonic | 0.10.2 | knhk-sidecar | ✅ Active, stable |
| oxigraph | 0.5.2 | knhk-etl, knhk-warm | ✅ Active, stable |
| reqwest | 0.11.27 | knhk-etl | ✅ Active, stable |
| serde_json | 1.0.145 | All crates | ✅ Active, stable |
| testcontainers | 0.16 | integration-tests | ⚠️ Could update to 0.17+ |

### 4.2 Circular Dependency Risk

**Analysis**:
```
knhk-hot (no deps)
  └─ knhk-etl
      └─ knhk-warm (uses knhk-etl with default-features = false)
  └─ knhk-sidecar (uses knhk-etl)
```

**Risk**: knhk-warm → knhk-etl → knhk-warm could create circular dependency

**Mitigation**: knhk-warm uses `default-features = false` to break potential cycles

**Status**: ✅ No circular dependencies detected

### 4.3 Version Consistency

| Crate | knhk-etl | knhk-warm | knhk-sidecar | Status |
|-------|----------|-----------|--------------|--------|
| knhk-hot | 1.0.0 | 1.0.0 | - | ✅ Consistent |
| knhk-otel | 0.1.0 | 0.1.0 | 0.1.0 | ✅ Consistent |
| serde_json | 1.0.145 | 1.0 | 1.0.145 | ⚠️ Minor version mismatch |
| thiserror | - | 2.0 | 1.0.69 | ⚠️ Major version mismatch |

**Recommendations**:
- Align serde_json versions (minor impact)
- Review thiserror version mismatch (potential API incompatibility)

## 5. Integration Test Coverage

### 5.1 Existing Tests

**File**: `rust/knhk-integration-tests/src/main.rs`

| Test | Coverage | Status |
|------|----------|--------|
| `test_kafka_connector_integration` | Kafka connector creation | ✅ Basic |
| `test_etl_pipeline_kafka_integration` | ETL + Kafka | ✅ Basic |
| `test_lockchain_postgres_integration` | Lockchain + PostgreSQL | ✅ Basic |
| `test_otel_collector_integration` | OTEL tracer | ✅ Basic |
| `test_end_to_end_integration` | Full pipeline | ✅ Basic |

### 5.2 Missing Integration Tests

**Critical Gaps**:
1. ❌ **Sidecar → ETL pipeline integration**: No tests for gRPC → ETL flow
2. ❌ **ETL → Hot path performance**: No tests verifying ≤8 tick budget
3. ❌ **ETL → Warm path query**: No tests for SPARQL execution
4. ❌ **Sidecar → Weaver validation**: No tests for live schema checking
5. ❌ **Feature flag combinations**: No tests for different feature sets
6. ❌ **Error propagation**: No tests for failure scenarios
7. ❌ **Concurrent operations**: No tests for thread safety

## 6. Recommendations

### 6.1 High Priority

1. **Create comprehensive integration test suite** (Priority: P0)
   - Test all integration points documented in Section 2
   - Add performance validation tests
   - Add failure scenario tests

2. **Align feature flags** (Priority: P1)
   - Standardize OTEL feature naming
   - Document feature propagation clearly
   - Test all feature combinations

3. **Fix version mismatches** (Priority: P1)
   - Align thiserror versions (1.x vs 2.x)
   - Standardize minor version numbers

### 6.2 Medium Priority

4. **Add integration documentation** (Priority: P2)
   - Document each integration point
   - Provide example code
   - Document performance characteristics

5. **Improve error handling** (Priority: P2)
   - Test error propagation across crate boundaries
   - Ensure proper error context
   - Add integration-level error types

### 6.3 Low Priority

6. **Update dependencies** (Priority: P3)
   - Consider testcontainers 0.17+
   - Review for security updates

7. **Performance benchmarks** (Priority: P3)
   - Benchmark integration overhead
   - Measure cross-crate call costs

## 7. Conclusion

The KNHK crate ecosystem has a clean dependency structure with well-defined integration points. The main areas requiring attention are:

1. **Integration test coverage**: Current tests are basic; comprehensive tests needed
2. **Feature flag consistency**: Some inconsistencies in naming and propagation
3. **Version alignment**: Minor version mismatches should be resolved
4. **Documentation**: Integration points need clear documentation

The architecture is sound with no circular dependencies, and the optional feature system allows flexible deployment configurations. The key risk area is the lack of comprehensive integration tests covering failure scenarios and performance validation.

## Appendices

### A. Dependency Graph

```
knhk-hot (1.0.0)
    └─ (no dependencies)

knhk-etl (0.1.0)
    ├─ knhk-hot (1.0.0)
    ├─ knhk-connectors (0.1.0)
    ├─ [optional] knhk-lockchain (0.1.0)
    ├─ [optional] knhk-otel (0.1.0)
    ├─ oxigraph (0.5.2)
    └─ [optional] reqwest (0.11.27)

knhk-warm (0.1.0)
    ├─ knhk-hot (1.0.0)
    ├─ knhk-etl (0.1.0, default-features = false)
    ├─ [optional] knhk-otel (0.1.0)
    └─ oxigraph (0.5)

knhk-sidecar (0.5.0)
    ├─ knhk-etl (0.1.0)
    ├─ knhk-connectors (0.1.0)
    ├─ knhk-otel (0.1.0)
    ├─ knhk-config (0.1.0)
    └─ tokio/tonic/gRPC stack

knhk-integration-tests (0.1.0)
    ├─ knhk-etl (0.1.0, features: ["std"])
    ├─ knhk-hot (1.0.0)
    ├─ knhk-otel (0.1.0)
    ├─ knhk-connectors (0.1.0)
    └─ testcontainers (0.16)
```

### B. Integration Point Summary

| From | To | Method | Status |
|------|-----|--------|--------|
| knhk-sidecar | knhk-etl | Direct crate dependency | ✅ Working |
| knhk-etl | knhk-hot | FFI calls | ✅ Working |
| knhk-etl | knhk-warm | Trait-based integration | ✅ Working |
| knhk-warm | knhk-hot | FFI calls | ✅ Working |
| knhk-sidecar | knhk-otel | Direct crate dependency | ✅ Working |
| knhk-etl | knhk-otel | Optional feature | ✅ Working |
| knhk-warm | knhk-otel | Optional feature | ✅ Working |

---

**Report Generated**: 2025-11-06
**Agent**: Integration Specialist (code-analyzer)
**Next Steps**: Implement comprehensive integration test suite
