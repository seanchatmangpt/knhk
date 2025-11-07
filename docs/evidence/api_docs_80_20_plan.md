# API Documentation 80/20 Plan for v1.0

**Date**: 2025-11-07
**Agent**: API Documentation Specialist
**Objective**: Identify critical 20% of public APIs that provide 80% developer value

---

## Executive Summary

**Current Status:**
- **Total Public Items**: ~250 across 13 crates (estimated from grep analysis)
- **Documented Items**: ~15-20% have basic doc comments
- **Missing Documentation**: 965 items flagged by cargo doc (most are internal/private)
- **Critical Path**: ~50 public APIs represent 80% of developer usage

**v1.0 Strategy:**
- Document **50 MUST-HAVE APIs** (20% providing 80% value)
- Defer **200+ internal/advanced APIs** to v1.1+
- Focus on developer-facing entry points, not implementation details

---

## 📦 Crate Overview: Public API Surface

### Critical Crates (Developer-Facing)

| Crate | Public Items | Developer Usage | v1.0 Priority |
|-------|--------------|-----------------|---------------|
| `knhk-etl` | 77 items | **HIGH** - Main pipeline API | **CRITICAL** |
| `knhk-config` | 19 items | **HIGH** - Configuration entry point | **CRITICAL** |
| `knhk-lockchain` | 11 items | **MEDIUM** - Receipt storage API | **HIGH** |
| `knhk-warm` | 35 items | **MEDIUM** - Query execution API | **MEDIUM** |
| `knhk-hot` | 31 items | **LOW** - FFI wrappers (internal use) | **LOW** |

### Supporting Crates (Internal/Advanced)

| Crate | Public Items | Developer Usage | v1.0 Priority |
|-------|--------------|-----------------|---------------|
| `knhk-cli` | 15 items | **LOW** - CLI implementation | **DEFER** |
| `knhk-otel` | 10 items | **LOW** - Telemetry internals | **DEFER** |
| `knhk-sidecar` | 8 items | **LOW** - gRPC service | **DEFER** |
| `knhk-validation` | 6 items | **LOW** - Internal validation | **DEFER** |
| `knhk-connectors` | 12 items | **LOW** - Data source adapters | **DEFER** |
| `knhk-aot` | 5 items | **LOW** - Ahead-of-time compilation | **DEFER** |
| `knhk-unrdf` | 8 items | **LOW** - Cold path fallback | **DEFER** |
| `knhk-integration-tests` | 0 items | N/A - Test crate | N/A |

---

## 🎯 MUST-DOCUMENT APIs for v1.0 (Top 20%)

### 1. **knhk-etl** (25 critical APIs)

**Pipeline Orchestration** (5 APIs - HIGHEST PRIORITY)
- ✅ `Pipeline::new()` - Main entry point
- ✅ `Pipeline::execute()` - Execute full ETL flow
- ❌ `Pipeline` struct fields - Document public fields
- ❌ `PipelineError` - Error type documentation
- ❌ `PipelineMetrics` - Performance metrics

**Core Stages** (8 APIs)
- ✅ `IngestStage::new()` - Create ingest stage
- ❌ `IngestStage::parse_rdf_turtle()` - RDF parsing
- ✅ `TransformStage::new()` - Create transform stage
- ❌ `TransformStage::transform()` - Type and hash triples
- ❌ `LoadStage::new()` - Create load stage
- ❌ `LoadStage::load()` - Group by predicate
- ❌ `ReflexStage::new()` - Create reflex stage
- ❌ `EmitStage::new()` - Create emit stage

**8-Beat System** (6 APIs - CRITICAL FOR V1.0)
- ❌ `BeatScheduler::new()` - Initialize scheduler
- ❌ `BeatScheduler::advance_beat()` - Advance cycle counter
- ❌ `BeatScheduler::submit_fiber()` - Submit fiber for execution
- ❌ `BeatScheduler::collect_receipts()` - Get execution receipts
- ❌ `BeatSchedulerError` - Error types
- ❌ 8-beat architecture explanation (module-level docs)

**Hook Registry** (4 APIs)
- ❌ `HookRegistry::new()` - Create hook registry
- ❌ `HookRegistry::register()` - Register predicate hooks
- ❌ `HookRegistry::get_kernel()` - Retrieve kernel for predicate
- ❌ `HookMetadata` - Hook metadata structure

**Data Types** (2 APIs)
- ❌ `RawTriple` - Input triple structure
- ❌ `Receipt` - Execution receipt structure

### 2. **knhk-config** (8 critical APIs)

**Configuration Management** (8 APIs)
- ❌ `Config::default()` - Default configuration
- ❌ `load_config()` - Load from TOML file
- ❌ `load_env_config()` - Load from environment
- ❌ `apply_env_overrides()` - Override config with env vars
- ❌ `Config` struct - All configuration fields
- ❌ `Config::validate()` - Validate configuration
- ❌ `KNHK_CONFIG_PATH` - Environment variable documentation
- ❌ Example TOML configuration (module-level docs)

### 3. **knhk-lockchain** (7 critical APIs)

**Receipt Storage** (7 APIs)
- ❌ `MerkleTree::new()` - Create Merkle tree
- ❌ `MerkleTree::add_receipt()` - Add receipt to tree
- ❌ `MerkleTree::compute_root()` - Calculate Merkle root
- ❌ `QuorumManager::new()` - Create quorum manager
- ❌ `QuorumManager::request_votes()` - Byzantine consensus
- ❌ `Receipt::compute_hash()` - URDNA2015 + SHA-256 hashing
- ❌ `LockchainStorage::save()` - Persist receipts

### 4. **knhk-warm** (10 critical APIs)

**Query Execution** (10 APIs)
- ❌ `WarmPathExecutor::new()` - Create executor
- ❌ `WarmPathExecutor::execute()` - Execute SPARQL query
- ❌ `WarmPathGraph::new()` - Create in-memory graph
- ❌ `WarmPathGraph::add_triple()` - Add RDF triple
- ❌ `WarmPathGraph::query()` - Query oxigraph backend
- ❌ `QueryError` - Query error types
- ❌ `SelectResult` - SELECT query results
- ❌ `ConstructResult` - CONSTRUCT query results
- ❌ `EpochScheduler::new()` - Schedule epochs (500ms budget)
- ❌ SPARQL support documentation (module-level docs)

---

## 📊 Documentation Coverage Analysis

### Current Coverage (Estimated)

**Well-Documented** (✅):
- `Pipeline::new()` - Has usage example
- `Pipeline::execute()` - Basic doc comment
- Stage constructors (`IngestStage::new()`, etc.)

**Partially Documented** (⚠️):
- Most structs have single-line comments
- No usage examples for complex workflows
- Missing error handling examples
- No performance constraint documentation

**Undocumented** (❌):
- 8-beat scheduler APIs (CRITICAL GAP)
- Hook registry APIs (CRITICAL GAP)
- Configuration APIs (HIGH PRIORITY GAP)
- Lockchain APIs (MEDIUM PRIORITY GAP)
- 80% of public functions lack examples

### Coverage by Priority

| Priority | APIs to Document | Current % | Target v1.0 % |
|----------|------------------|-----------|---------------|
| CRITICAL | 25 APIs | ~20% | **100%** |
| HIGH | 15 APIs | ~10% | **80%** |
| MEDIUM | 10 APIs | ~5% | **50%** |
| LOW | 200 APIs | ~1% | **0%** (defer to v1.1) |

---

## 🚧 DEFER to v1.1+ (Bottom 80%)

### Internal Implementation APIs (Not Developer-Facing)

**knhk-hot FFI wrappers** (31 items - defer)
- Raw C FFI bindings
- Internal receipt conversion
- Kernel executor internals
- Low-level beat scheduler FFI

**knhk-cli command implementations** (15 items - defer)
- CLI argument parsing
- Command handlers
- Output formatting

**knhk-otel telemetry internals** (10 items - defer)
- Span ID generation
- Trace context propagation
- Metric collectors

**knhk-sidecar gRPC service** (8 items - defer)
- Protocol buffer definitions
- Service implementation
- gRPC handlers

**Advanced ETL internals** (defer)
- `PathSelector` - Internal query routing
- `SoAArrays` - Internal data layout
- `PredRun` - Internal predicate runs
- `Fiber` internals - Implementation details
- `RingBuffer` internals - Lock-free ring details

### Rationale for Deferral

**80/20 Principle:**
- Developers interact with high-level APIs (Pipeline, Config, Query)
- Internal APIs are implementation details
- FFI wrappers are not directly called by users
- CLI commands are self-documenting via `--help`

**Time Constraints:**
- 250 total public items × 30 min each = 125 hours (unfeasible for v1.0)
- 50 critical items × 45 min each = 37.5 hours (feasible)
- 80% time savings by focusing on developer-facing APIs

---

## 📝 Documentation Standards for v1.0

### Required Elements (ALL CRITICAL APIs)

1. **Purpose** - What does this API do?
2. **Usage Example** - Working code snippet
3. **Parameters** - Description of all arguments
4. **Return Value** - What is returned/produced
5. **Errors** - When does it fail? How to handle?
6. **Performance** - Time budgets (8 ticks, 500ms, etc.)
7. **See Also** - Links to related APIs

### Example: Complete API Documentation

```rust
/// 8-beat epoch scheduler for deterministic ETL execution.
///
/// Manages cycle counter, ring buffers, and fiber rotation for ≤8 tick execution.
/// Each beat represents one tick (0-7 ticks per cycle). Pulse detected at tick 0.
///
/// # Architecture
///
/// - **Cycles**: 8-beat cycles (cycle_id increments every 8 ticks)
/// - **Pulse**: Detected when `tick == 0` (commit boundary)
/// - **Fibers**: Cooperative execution units (one per shard)
/// - **Ring Buffers**: Lock-free SPSC for delta/assertion queues
///
/// # Performance Constraints
///
/// - **Hot Path**: ≤8 ticks per operation (Chatman Constant)
/// - **Ring Capacity**: Must be power of 2 (8, 16, 32, etc.)
///
/// # Example
///
/// ```rust
/// use knhk_etl::BeatScheduler;
///
/// // Create scheduler: 4 shards, 1 domain, 8-element ring buffers
/// let mut scheduler = BeatScheduler::new(4, 1, 8)?;
///
/// // Advance beat and check for pulse (cycle boundary)
/// let (tick, pulse) = scheduler.advance_beat();
/// if pulse {
///     println!("Cycle committed at tick {}", tick);
/// }
///
/// // Submit fiber for execution
/// let fiber = Fiber::new(0, vec![/* triples */]);
/// scheduler.submit_fiber(fiber)?;
///
/// // Collect receipts after execution
/// let receipts = scheduler.collect_receipts();
/// for receipt in receipts {
///     assert!(receipt.actual_ticks <= 8); // Hot path guarantee
/// }
/// ```
///
/// # Errors
///
/// Returns `BeatSchedulerError::InvalidShardCount` if `shard_count == 0`.
/// Returns `BeatSchedulerError::RingBufferFull` if ring capacity exceeded.
///
/// # See Also
///
/// - [`Fiber`] - Cooperative execution unit
/// - [`Receipt`] - Execution provenance record
/// - [`RingBuffer`] - Lock-free SPSC ring implementation
pub struct BeatScheduler { /* ... */ }
```

---

## 🎯 Action Plan for v1.0

### Phase 1: Critical APIs (Week 1)

**Day 1-2: Pipeline Core** (10 APIs)
- [ ] `Pipeline` struct and methods
- [ ] `PipelineError` enum
- [ ] Basic pipeline example (Ingest → Emit)

**Day 3-4: 8-Beat System** (10 APIs)
- [ ] `BeatScheduler` struct and methods
- [ ] `BeatSchedulerError` enum
- [ ] 8-beat architecture explanation
- [ ] Fiber submission example

**Day 5: Configuration** (8 APIs)
- [ ] `Config` struct and fields
- [ ] `load_config()` and env overrides
- [ ] Example TOML configuration

### Phase 2: High Priority APIs (Week 2)

**Day 1-2: Hook Registry** (4 APIs)
- [ ] `HookRegistry` struct and methods
- [ ] `HookMetadata` structure
- [ ] Predicate-to-kernel mapping example

**Day 3-4: Lockchain** (7 APIs)
- [ ] `MerkleTree` methods
- [ ] `QuorumManager` consensus
- [ ] Receipt hashing (URDNA2015 + SHA-256)

**Day 5: ETL Stages** (8 APIs)
- [ ] Stage documentation (Ingest, Transform, Load, Reflex)
- [ ] Stage chaining example

### Phase 3: Medium Priority APIs (Week 3)

**Warm Path Query Execution** (10 APIs)
- [ ] `WarmPathExecutor` methods
- [ ] `WarmPathGraph` SPARQL support
- [ ] Query result types

### Measurement Criteria

**Success Metrics for v1.0:**
- ✅ 50 critical APIs fully documented (100% coverage)
- ✅ Every critical API has usage example
- ✅ Zero doc warnings for CRITICAL/HIGH priority crates
- ✅ Developer can build Pipeline from docs alone

**Acceptable Gaps for v1.0:**
- ⚠️ Internal FFI wrappers remain undocumented
- ⚠️ CLI implementation details remain undocumented
- ⚠️ Advanced tuning parameters have minimal docs

---

## 📚 Documentation Deliverables

### v1.0 Deliverables (MUST-HAVE)

1. **API Reference** - 50 fully documented critical APIs
2. **Getting Started Guide** - Basic pipeline setup
3. **Architecture Guide** - 8-beat system explanation
4. **Configuration Guide** - TOML + env var setup
5. **Examples** - 5 working code examples

### v1.1 Deliverables (FUTURE)

1. **Advanced API Reference** - 200+ internal APIs
2. **Performance Tuning Guide** - Ring buffer sizing, shard tuning
3. **Extension Guide** - Custom kernels, connectors
4. **Migration Guide** - Upgrading from v1.0 to v1.1

---

## 📈 ROI Analysis

### Documentation Effort vs. Developer Value

**Critical APIs (20% of items, 80% of value):**
- **Effort**: 50 APIs × 45 min = 37.5 hours
- **Value**: Developers can use KNHK immediately
- **ROI**: **High** - Essential for v1.0 adoption

**Internal APIs (80% of items, 20% of value):**
- **Effort**: 200 APIs × 30 min = 100 hours
- **Value**: Only benefits advanced users/contributors
- **ROI**: **Low** - Defer to v1.1 when usage patterns are known

### Time Savings: 80/20 Approach

| Approach | APIs Documented | Hours Required | v1.0 Readiness |
|----------|-----------------|----------------|----------------|
| **Document All** | 250 | 125 hours | **BLOCKS v1.0** |
| **80/20 Focus** | 50 | 37.5 hours | **v1.0 READY** |
| **Time Saved** | - | **87.5 hours** | **+70% faster** |

---

## 🚀 Next Steps

1. **Review & Approve**: Stakeholder approval of 50-API priority list
2. **Execute Phase 1**: Document Pipeline + 8-Beat System (Week 1)
3. **Execute Phase 2**: Document Config + Lockchain + Hooks (Week 2)
4. **Execute Phase 3**: Document Warm Path (Week 3)
5. **Validation**: Verify docs via external developer (can they build pipeline?)
6. **v1.0 Release**: Ship with 50 critical APIs documented

---

## 📋 Priority API Checklist

### CRITICAL (Must Have for v1.0)

**Pipeline (5/5)**
- [ ] `Pipeline::new()`
- [ ] `Pipeline::execute()`
- [ ] `Pipeline` fields
- [ ] `PipelineError`
- [ ] `PipelineMetrics`

**8-Beat Scheduler (6/6)**
- [ ] `BeatScheduler::new()`
- [ ] `BeatScheduler::advance_beat()`
- [ ] `BeatScheduler::submit_fiber()`
- [ ] `BeatScheduler::collect_receipts()`
- [ ] `BeatSchedulerError`
- [ ] 8-beat architecture docs

**Configuration (8/8)**
- [ ] `Config::default()`
- [ ] `load_config()`
- [ ] `load_env_config()`
- [ ] `apply_env_overrides()`
- [ ] `Config` struct
- [ ] `Config::validate()`
- [ ] `KNHK_CONFIG_PATH`
- [ ] Example TOML

### HIGH (Should Have for v1.0)

**Hook Registry (4/4)**
- [ ] `HookRegistry::new()`
- [ ] `HookRegistry::register()`
- [ ] `HookRegistry::get_kernel()`
- [ ] `HookMetadata`

**ETL Stages (8/8)**
- [ ] `IngestStage::new()` + `parse_rdf_turtle()`
- [ ] `TransformStage::new()` + `transform()`
- [ ] `LoadStage::new()` + `load()`
- [ ] `ReflexStage::new()`
- [ ] `EmitStage::new()`
- [ ] `RawTriple`
- [ ] `Receipt`

**Lockchain (7/7)**
- [ ] `MerkleTree` methods (3)
- [ ] `QuorumManager` methods (2)
- [ ] `Receipt::compute_hash()`
- [ ] `LockchainStorage::save()`

### MEDIUM (Nice to Have for v1.0)

**Warm Path (10/10)**
- [ ] `WarmPathExecutor` methods (3)
- [ ] `WarmPathGraph` methods (3)
- [ ] Query result types (3)
- [ ] `EpochScheduler::new()`

**Total: 50 Critical APIs**

---

## Conclusion

By focusing on **50 critical developer-facing APIs** (20% of total), we achieve **80% developer value** while saving **87.5 hours** vs. documenting all 250 public items. This pragmatic approach unblocks v1.0 release while deferring internal/advanced APIs to v1.1 when usage patterns are better understood.

**Recommendation**: Approve 50-API priority list and execute 3-week documentation sprint.
