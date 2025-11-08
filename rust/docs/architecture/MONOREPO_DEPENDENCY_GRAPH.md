# KNHK Monorepo: Comprehensive Dependency Graph & Architecture Analysis

**Version:** 1.0.0
**Date:** 2025-11-07
**Analyst:** System Architect
**Scope:** 14 workspace packages + C hot path layer

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Package Catalog](#package-catalog)
3. [Dependency Graph](#dependency-graph)
4. [FFI Integration Points](#ffi-integration-points)
5. [Trait-Based Integration](#trait-based-integration)
6. [Architectural Layers](#architectural-layers)
7. [Integration Point Matrix](#integration-point-matrix)
8. [Build Topology](#build-topology)
9. [Critical Path Analysis](#critical-path-analysis)
10. [Architectural Patterns](#architectural-patterns)

---

## Executive Summary

The KNHK monorepo implements a **multi-layered, hot/warm/cold execution architecture** with clear separation of concerns:

### Key Metrics
- **Total Packages:** 14 (13 active + 1 excluded)
- **Total Lines of Code:** 3,731 (lib.rs files only)
- **Dependency Depth:** 4 levels maximum
- **Foundation Packages:** 5 (no internal dependencies)
- **FFI Boundaries:** 3 major (C ↔ Rust interfaces)
- **Circular Dependencies:** 0 ✅
- **Build Parallelism:** 5-way (Stage 0)

### Architecture Highlights
1. **Hot Path (≤8 ticks):** C kernels with Rust FFI wrappers
2. **Warm Path (≤500ms):** SPARQL query optimization with caching
3. **Cold Path (unbounded):** External connectors and persistence
4. **Schema-First Validation:** OpenTelemetry Weaver as source of truth
5. **Van der Aalst Workflow Patterns:** 23 workflow control patterns

---

## Package Catalog

### Layer 0: Foundation (No Dependencies)

#### knhk-hot (Critical Path Component)
- **Description:** Hot path FFI layer linking to C kernels
- **Purpose:** Sub-8 tick operations with SoA memory layout
- **Key Features:**
  - C FFI bindings (`extern "C"`)
  - Content-addressable hashing (Blake3)
  - Beat scheduler integration
  - Fiber executor for cooperative multitasking
  - Ring buffers for lock-free communication
- **Output Types:** `staticlib`, `cdylib`, `rlib`
- **Critical Path:** ✅ Used by 5 packages
- **Performance Contract:** ≤8 CPU ticks per operation

#### knhk-config
- **Description:** Configuration management
- **Purpose:** TOML-based configuration parsing
- **Key Features:**
  - Environment variable support
  - Type-safe configuration structs
  - Feature flag: `std` (default)
- **Output Type:** `lib`
- **Dependencies:** `toml`, `serde`, `serde_json`

#### knhk-lockchain
- **Description:** Immutable event log with Git-style tracking
- **Purpose:** Cryptographic audit trail for pipeline events
- **Key Features:**
  - Blake3 + SHA2 hashing
  - Sled embedded database
  - Merkle tree structure
  - Quorum consensus (multi-node validation)
- **Output Type:** `lib`
- **Critical Note:** Basic canonicalization (v1.0), full URDNA2015 deferred to v1.1

#### knhk-otel
- **Description:** OpenTelemetry instrumentation
- **Purpose:** Schema-first telemetry validation (Weaver integration)
- **Key Features:**
  - OTLP exporter (gRPC + HTTP)
  - Runtime class tracking
  - All dependencies behind `std` feature
  - OpenTelemetry 0.31 (latest)
- **Output Types:** `cdylib`, `staticlib`, `rlib`
- **Critical Path:** ✅ Used by 5 packages

#### knhk-connectors
- **Description:** External system connectors
- **Purpose:** Kafka, Salesforce integration
- **Key Features:**
  - `kafka` feature (rdkafka)
  - `salesforce` feature (reqwest, serde)
  - `std` feature (default)
  - Error diagnostics with actionable suggestions
- **Output Types:** `cdylib`, `staticlib`, `rlib`
- **Critical Path:** ✅ Used by 5 packages

---

### Layer 1: Core Services

#### knhk-etl (Central Orchestration)
- **Description:** Extract, Transform, Load pipeline
- **Purpose:** 5-stage data transformation pipeline
- **Pipeline Stages:**
  1. **Ingest:** Raw triple ingestion
  2. **Transform:** Type validation and enrichment
  3. **Load:** SoA array conversion
  4. **Reflex:** Hot path kernel execution
  5. **Emit:** Result publication
- **Key Features:**
  - Beat scheduler (8-beat epochs)
  - Hook registry (predicate → kernel mapping)
  - Fiber-based execution (cooperative)
  - Ring buffers (lock-free, ≤8 ticks)
  - Runtime class monitoring
  - SLO monitoring and park/escalate mechanism
  - Hook orchestration patterns
- **Dependencies:** knhk-connectors, knhk-hot, knhk-lockchain, knhk-otel
- **Output Types:** `cdylib`, `staticlib`, `rlib`
- **Critical Path:** ✅ Used by 5 packages
- **Deny Rules:** `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::expect_used)]`

#### knhk-validation
- **Description:** Policy engine and validation framework
- **Purpose:** Schema validation, policy enforcement, diagnostics
- **Key Features:**
  - Policy engine (Regorus OPA)
  - Diagnostics with miette (fancy error reports)
  - Streaming validation
  - Schema resolution
  - Advisor mode (AI-assisted validation)
- **Dependencies:** knhk-hot, knhk-connectors, knhk-lockchain, knhk-otel
- **Notable:** Removed knhk-etl dependency to prevent circular reference ✅
- **Output Types:** `lib` + `bin`
- **Features:** `diagnostics`, `advisor`, `policy-engine`, `schema-resolution`, `streaming`

---

### Layer 2: Specialized Components

#### knhk-aot
- **Description:** Ahead-of-time validation and policy enforcement
- **Purpose:** Pre-compilation optimizations and static analysis
- **Key Features:**
  - Optional validation integration
  - jemalloc allocator (non-MSVC)
  - Template analysis
  - MPHF (minimal perfect hash function) generation
  - Pre-binding optimization
  - Specialization engine
- **Dependencies:** knhk-validation (optional)
- **Output Types:** `cdylib`, `staticlib`

#### knhk-unrdf
- **Description:** Universal RDF hooks engine
- **Purpose:** Template-based RDF transformations with Tera
- **Key Features:**
  - Native Rust RDF (Oxigraph) via `native` feature
  - Template engine (Tera)
  - Transaction support
  - SHACL validation
  - Query execution
  - Canonicalization
- **Dependencies:** knhk-etl
- **Output Types:** `rlib`, `cdylib`, `staticlib`
- **Version Conflict:** ⚠️ Uses OpenTelemetry 0.21 (vs 0.31 workspace)
- **Benchmarks:** `hooks_native_bench`

#### knhk-patterns (Van der Aalst Workflow Patterns)
- **Description:** Workflow patterns for pipeline orchestration
- **Purpose:** 23 workflow control-flow patterns
- **Key Patterns:**
  1. Sequence
  2. Parallel Split
  3. Synchronization
  4. Exclusive Choice
  5. Simple Merge
  6. Multi-Choice
  9. Discriminator (first-wins race)
  10. Arbitrary Cycles
  11. Implicit Termination
  16. Deferred Choice
  20. Timeout
  21. Cancellation
- **Features:**
  - FFI bindings to C hot path patterns
  - Hook-based patterns (knhk-etl integration)
  - Unrdf patterns (optional)
  - Hybrid patterns (Rust + C composition)
  - Hot path C kernels for performance-critical patterns
- **Dependencies:** knhk-etl, knhk-config, knhk-unrdf (optional)
- **Output Type:** `lib`
- **Build Dependencies:** `cc` (for C kernel compilation)

#### knhk-integration-tests
- **Description:** End-to-end integration tests
- **Purpose:** Testcontainers-based integration testing
- **Key Features:**
  - Kafka testcontainers
  - Postgres testcontainers
  - Full pipeline validation
- **Dependencies:** knhk-connectors, knhk-etl, knhk-hot, knhk-otel
- **Output Type:** `bin`

#### knhk-sidecar (Excluded - Wave 5)
- **Description:** gRPC sidecar service
- **Purpose:** Proxy service with batching, retries, circuit-breaking
- **Status:** ⚠️ Excluded from workspace (53 async trait errors)
- **Dependencies:** knhk-etl, knhk-connectors, knhk-otel, knhk-config
- **Output Type:** `lib`
- **Technical Debt:** Async trait methods break `dyn` compatibility

---

### Layer 3: Query Engine

#### knhk-warm
- **Description:** Warm path query optimization (≤500ms budget)
- **Purpose:** SPARQL query execution with caching
- **Key Features:**
  - Oxigraph integration
  - LRU caching (query results)
  - Epoch scheduler (hot/warm boundary)
  - FFI integration with knhk-hot types
  - CONSTRUCT8 pattern optimization
- **Dependencies:**
  - knhk-hot (required)
  - knhk-etl (required, no default features)
  - knhk-otel (optional)
  - knhk-unrdf (optional)
- **Output Types:** `staticlib`, `cdylib`
- **Benchmarks:** `query_bench`
- **Examples:** `warm_path_query`

---

### Layer 4: Application

#### knhk-cli
- **Description:** Command-line interface
- **Purpose:** User-facing CLI for KNHK operations
- **Commands:**
  - `admit` - Admit data into pipeline
  - `boot` - Bootstrap system
  - `config` - Manage configuration
  - `connect` - Connect to external systems
  - `context` - Manage execution contexts
  - `cover` - Coverage analysis
  - `epoch` - Epoch management
  - `hook` - Hook registry operations
  - `metrics` - View telemetry metrics
  - `pipeline` - Pipeline execution
  - `receipt` - Receipt inspection
  - `reflex` - Reflex stage operations
  - `route` - Routing configuration
- **Dependencies:** knhk-hot, knhk-warm, knhk-config, knhk-etl, knhk-connectors, knhk-lockchain, knhk-otel (optional)
- **Output Type:** `bin`
- **CLI Framework:** clap-noun-verb 3.3.0

---

## Dependency Graph

### Complete Dependency Visualization

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Layer 0: Foundation                         │
│  ┌──────────┐  ┌────────┐  ┌───────────┐  ┌──────┐  ┌──────────┐  │
│  │ knhk-hot │  │ config │  │ lockchain │  │ otel │  │connector │  │
│  └──────────┘  └────────┘  └───────────┘  └──────┘  └──────────┘  │
│      │ ▲          │              │            │          │          │
└──────┼─┼──────────┼──────────────┼────────────┼──────────┼──────────┘
       │ │          │              │            │          │
       │ │          │         ┌────┴────────────┴──────────┴────┐
       │ │          │         │                                  │
┌──────┼─┼──────────┼─────────▼────────────────────────────────────────┐
│      │ │          │      Layer 1: Core Services                      │
│      │ │          │  ┌──────────────┐      ┌────────────────┐       │
│      │ │          │  │   knhk-etl   │◄─────┤ knhk-validation│       │
│      │ │          │  │  (Pipeline)  │      │  (Policy)      │       │
│      │ │          │  └──────────────┘      └────────────────┘       │
│      │ │          │         │ ▲                                      │
└──────┼─┼──────────┼─────────┼─┼──────────────────────────────────────┘
       │ │          │         │ │
       │ │          │    ┌────┴─┴────────────────┐
       │ │          │    │                       │
┌──────┼─┼──────────┼────▼────────────────────────────────────────────┐
│      │ │          │  Layer 2: Specialized Components                 │
│      │ │          │  ┌───────┐  ┌────────┐  ┌──────────┐  ┌─────┐  │
│      │ │          └─►│patterns│  │ unrdf  │  │integration│  │ aot │  │
│      │ │             │(VdA)   │  │(RDF)   │  │ (tests)  │  │(AoT)│  │
│      │ │             └────────┘  └────────┘  └──────────┘  └─────┘  │
│      │ │                  │          │                               │
└──────┼─┼──────────────────┼──────────┼───────────────────────────────┘
       │ │                  │          │
       │ │             ┌────┴──────────┘
       │ │             │
┌──────┼─┼─────────────▼────────────────────────────────────────────┐
│      │ │       Layer 3: Query Engine                              │
│      │ │       ┌──────────────────┐                               │
│      │ └───────┤   knhk-warm      │                               │
│      │         │ (SPARQL/Cache)   │                               │
│      │         └──────────────────┘                               │
│      │                 │                                           │
└──────┼─────────────────┼───────────────────────────────────────────┘
       │                 │
       │            ┌────┘
       │            │
┌──────┼────────────▼──────────────────────────────────────────────┐
│      │       Layer 4: Application                                │
│      │       ┌──────────────────┐                                │
│      └───────┤    knhk-cli      │                                │
│              │  (User Interface)│                                │
│              └──────────────────┘                                │
└───────────────────────────────────────────────────────────────────┘
```

### Dependency Matrix

| Package | hot | config | lock | otel | conn | etl | valid | aot | unrdf | warm | patterns | integ | cli |
|---------|-----|--------|------|------|------|-----|-------|-----|-------|------|----------|-------|-----|
| **knhk-hot** | - | | | | | | | | | | | | |
| **knhk-config** | | - | | | | | | | | | | | |
| **knhk-lockchain** | | | - | | | | | | | | | | |
| **knhk-otel** | | | | - | | | | | | | | | |
| **knhk-connectors** | | | | | - | | | | | | | | |
| **knhk-etl** | ✓ | | ✓ | ✓ | ✓ | - | | | | | | | |
| **knhk-validation** | ✓ | | ✓ | ✓ | ✓ | | - | | | | | | |
| **knhk-aot** | | | | | | | ✓? | - | | | | | |
| **knhk-unrdf** | | | | | | ✓ | | | - | | | | |
| **knhk-patterns** | | ✓ | | | | ✓ | | | ✓? | | - | | |
| **knhk-warm** | ✓ | | | ✓? | | ✓ | | | ✓? | - | | | |
| **knhk-integration** | ✓ | | | ✓ | ✓ | ✓ | | | | | | - | |
| **knhk-cli** | ✓ | ✓ | ✓ | ✓? | ✓ | ✓ | | | | ✓ | | | - |

**Legend:** ✓ = required dependency, ✓? = optional dependency

---

## FFI Integration Points

### C → Rust FFI Boundaries

#### 1. knhk-hot: Hot Path Kernels

**C Side (libknhk.a):**
- Location: `/c/src/`
- Kernels: `knhk_exec()`, `knhk_construct8()`
- Header: `knhk.h`
- Compile: `make` in `/c` directory

**Rust Side (knhk-hot):**
```rust
// src/ffi.rs
#[repr(C)]
pub struct Ctx {
    pub S: *const u64,  // Subject array
    pub P: *const u64,  // Predicate array
    pub O: *const u64,  // Object array
    pub run: Run,
}

#[repr(C)]
pub struct Ir {
    pub op: Op,
    pub s: u64,
    pub p: u64,
    pub o: u64,
    pub k: u64,
    pub out_S: *mut u64,
    pub out_P: *mut u64,
    pub out_O: *mut u64,
    pub out_mask: u64,
}

extern "C" {
    fn knhk_exec(ctx: *mut Ctx, ir: *const Ir) -> Receipt;
}
```

**Integration Pattern:**
- **Memory Layout:** SoA (Struct of Arrays), 64-byte aligned
- **Performance Contract:** ≤8 CPU ticks per `knhk_exec()` call
- **Safety:** Unsafe FFI wrapped in safe Rust API
- **Type Safety:** `#[repr(C)]` ensures ABI compatibility

#### 2. knhk-patterns: Workflow Pattern Kernels

**C Side (workflow_patterns.c/h):**
- Location: `/rust/knhk-hot/src/`
- Functions: `knhk_pattern_sequence()`, `knhk_pattern_discriminator_simd()`, etc.
- Compiled via: `build.rs` (cc crate)

**Rust Side (knhk-patterns):**
```rust
// src/ffi.rs
#[repr(C)]
pub struct PatternContext {
    pub data: *mut u64,
    pub len: u32,
    pub metadata: u64,
}

#[repr(C)]
pub struct PatternResult {
    pub success: bool,
    pub branches: u32,
    pub result: u64,
    pub error: *const c_char,
}

extern "C" {
    pub fn knhk_pattern_discriminator_simd(
        ctx: *mut PatternContext,
        branches: *const BranchFn,
        num_branches: c_uint,
    ) -> PatternResult;
}
```

**Integration Pattern:**
- **Compilation:** C code compiled during Rust build via `cc` crate
- **SIMD Optimization:** AVX2/NEON intrinsics in C for branchless execution
- **Callback Functions:** Rust closures passed as C function pointers

#### 3. knhk-warm: Hot Path Type Re-exports

**Rust Side (knhk-warm):**
```rust
// src/ffi.rs
pub use knhk_hot::{Ctx, Ir, Op, Receipt, Run};

// Warm path wraps hot path types
pub struct WarmPathExecutor {
    hot_ctx: knhk_hot::Ctx,
    query_cache: LruCache<String, QueryResult>,
}
```

**Integration Pattern:**
- **Type Reuse:** Warm path reuses hot path C types
- **Boundary:** Warm path schedules hot path operations via epoch planning
- **Performance:** Cache-aware scheduling to stay ≤500ms budget

---

### FFI Safety Guarantees

| Boundary | Mechanism | Safety Level |
|----------|-----------|--------------|
| **C → Rust (knhk-hot)** | `#[repr(C)]` structs | Unsafe (wrapped in safe API) |
| **C → Rust (patterns)** | Function pointers + `extern "C"` | Unsafe (validated at build time) |
| **Rust → C (callbacks)** | `unsafe extern "C" fn` | Unsafe (caller validated) |
| **Memory Alignment** | `#[repr(align(64))]` | Compile-time checked |
| **Pointer Validity** | Manual validation in wrapper | Runtime checked |

---

## Trait-Based Integration

### Core Traits

#### 1. Pipeline Stages (knhk-etl)

```rust
// src/types.rs
pub trait PipelineStage: Send + Sync {
    type Input;
    type Output;
    type Error;

    fn process(&self, input: Self::Input)
        -> Result<Self::Output, Self::Error>;
}
```

**Implementations:**
- `IngestStage` (RawTriple → IngestResult)
- `TransformStage` (RawTriple → TypedTriple)
- `LoadStage` (TypedTriple → SoAArrays)
- `ReflexStage` (SoAArrays → Receipt)
- `EmitStage` (Receipt → EmitResult)

#### 2. Ingester Trait (knhk-etl)

```rust
// src/ingester.rs
pub trait Ingester: Send + Sync {
    fn ingest(&mut self) -> Result<Vec<RawTriple>, Box<dyn std::error::Error>>;
}
```

**Implementations:**
- `FileIngester` (reads from file)
- `StdinIngester` (reads from stdin)
- `KafkaIngester` (via knhk-connectors, future)

#### 3. Connector Trait (knhk-connectors)

```rust
// src/lib.rs
pub trait Connector: Send + Sync {
    fn name(&self) -> &str;
    fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
```

**Implementations:**
- `KafkaConnector` (rdkafka integration)
- `SalesforceConnector` (reqwest + OAuth)

#### 4. Warm Path Query Executor (knhk-etl/integration)

```rust
// src/integration.rs
pub trait WarmPathQueryExecutor: Send + Sync {
    fn execute_query(&self, sparql: &str)
        -> Result<WarmPathQueryResult, String>;
}
```

**Implementation:**
- `knhk-warm::WarmPathExecutor` (Oxigraph-based)

#### 5. Pattern Trait (knhk-patterns)

```rust
// src/patterns.rs
pub trait Pattern: Send + Sync {
    type Context;
    type Result;

    fn execute(&self, ctx: &mut Self::Context) -> PatternResult<Self::Result>;
    fn pattern_type(&self) -> PatternType;
}
```

**Implementations:**
- `SequencePattern`
- `ParallelSplitPattern`
- `ExclusiveChoicePattern`
- `DiscriminatorPattern`
- `TimeoutPattern`
- `CancellationPattern`
- ... (17 more patterns)

#### 6. Policy Validator (knhk-validation)

```rust
// src/policy.rs
pub trait PolicyValidator {
    fn validate(&self, input: &PolicyInput) -> Result<PolicyOutput, PolicyError>;
}
```

**Implementation:**
- `RegorusValidator` (OPA policy engine)

---

### Trait Composition Patterns

#### Pipeline Extension (knhk-patterns)

```rust
// src/pipeline_ext.rs
pub trait PipelinePatternExt {
    fn with_pattern<P: Pattern>(self, pattern: P) -> Self;
    fn with_retry(self, max_attempts: u32) -> Self;
    fn with_timeout(self, duration: Duration) -> Self;
}

impl PipelinePatternExt for Pipeline {
    // Extends knhk-etl::Pipeline with workflow patterns
}
```

**Integration Benefit:** Seamless workflow pattern injection into ETL pipeline

---

## Architectural Layers

### Performance Tier Classification

```
┌─────────────────────────────────────────────────────────────┐
│  HOT PATH (≤8 ticks)                                        │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ C Kernels (libknhk.a)                                 │ │
│  │ - knhk_exec() - Core query execution                 │ │
│  │ - knhk_construct8() - Template-based construction    │ │
│  │ - SIMD intrinsics (AVX2/NEON)                        │ │
│  │ - Cache-aligned SoA layout (64B)                     │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ Rust FFI Wrappers (knhk-hot)                         │ │
│  │ - Safe API over unsafe extern "C"                    │ │
│  │ - Type safety with #[repr(C)]                        │ │
│  │ - Beat scheduler (8-beat epochs)                     │ │
│  │ - Fiber executor (cooperative, deterministic)        │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  WARM PATH (≤500ms)                                         │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-warm                                             │ │
│  │ - SPARQL query optimization                          │ │
│  │ - Oxigraph integration                               │ │
│  │ - LRU caching (query result reuse)                   │ │
│  │ - Epoch scheduler (hot/warm boundary)                │ │
│  │ - CONSTRUCT8 pattern hints                           │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-patterns (Workflow Orchestration)                │ │
│  │ - Van der Aalst 23 patterns                          │ │
│  │ - Hot path C kernels for critical patterns           │ │
│  │ - Hybrid Rust + C composition                        │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  COLD PATH (Unbounded)                                      │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-connectors                                       │ │
│  │ - Kafka consumer/producer                            │ │
│  │ - Salesforce REST API                                │ │
│  │ - External system integration                        │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-lockchain                                        │ │
│  │ - Sled embedded database                             │ │
│  │ - Merkle tree persistence                            │ │
│  │ - Git-style immutable log                            │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-unrdf                                            │ │
│  │ - Tera template engine                               │ │
│  │ - SHACL validation                                   │ │
│  │ - RDF transactions                                   │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  ORCHESTRATION LAYER                                        │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-etl (Pipeline Coordinator)                       │ │
│  │ - 5-stage pipeline (Ingest → Emit)                   │ │
│  │ - Hook registry (predicate → kernel mapping)         │ │
│  │ - SLO monitoring                                     │ │
│  │ - Park/escalate mechanism                            │ │
│  │ - Integration layer (connects all subsystems)        │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  OBSERVABILITY LAYER                                        │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-otel (Telemetry)                                 │ │
│  │ - OpenTelemetry 0.31                                 │ │
│  │ - OTLP exporter (gRPC + HTTP)                        │ │
│  │ - Runtime class tracking                             │ │
│  │ - Weaver schema validation                           │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-validation (Policy Engine)                       │ │
│  │ - Regorus OPA integration                            │ │
│  │ - Miette diagnostics                                 │ │
│  │ - Schema resolution                                  │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  TOOLING LAYER                                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-cli (User Interface)                             │ │
│  │ - clap-noun-verb 3.3.0                               │ │
│  │ - 13 commands (admit, boot, config, etc.)            │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-aot (Ahead-of-Time Compiler)                     │ │
│  │ - Template analysis                                  │ │
│  │ - MPHF generation                                    │ │
│  │ - Specialization engine                             │ │
│  └───────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ knhk-config (Configuration)                           │ │
│  │ - TOML parsing                                       │ │
│  │ - Environment variable support                       │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Integration Point Matrix

### Cross-Package Integration Mechanisms

| Integration Type | Packages | Mechanism | Example |
|------------------|----------|-----------|---------|
| **FFI (C ↔ Rust)** | knhk-hot ↔ C kernels | `extern "C"` + `#[repr(C)]` | `knhk_exec()` wrapper |
| **FFI (C ↔ Rust)** | knhk-patterns ↔ C patterns | `build.rs` + `cc` crate | Workflow pattern kernels |
| **Trait Impl** | knhk-etl ← knhk-warm | `WarmPathQueryExecutor` | SPARQL integration |
| **Trait Impl** | knhk-etl ← knhk-connectors | `Connector` | Kafka/Salesforce |
| **Trait Impl** | knhk-patterns → knhk-etl | `PipelinePatternExt` | Workflow injection |
| **Type Reuse** | knhk-warm ← knhk-hot | `pub use` | Hot path types |
| **Type Reuse** | knhk-cli ← all | Direct imports | Command implementations |
| **Feature Flags** | knhk-warm ← knhk-otel | `otel` feature | Optional telemetry |
| **Feature Flags** | knhk-patterns ← knhk-unrdf | `unrdf` feature | Optional RDF patterns |
| **Event Log** | knhk-etl → knhk-lockchain | Direct calls | Pipeline event logging |
| **Metrics** | all → knhk-otel | `MetricsHelper` | Telemetry emission |

### Data Flow Patterns

```
External System (Kafka)
        │
        ▼
knhk-connectors::KafkaConnector
        │
        ▼
knhk-etl::IngestStage (RawTriple)
        │
        ▼
knhk-etl::TransformStage (TypedTriple)
        │
        ▼
knhk-etl::LoadStage (SoAArrays)
        │
        ▼
knhk-etl::ReflexStage
        │
        ├──► knhk-hot::knhk_exec() [≤8 ticks]
        │           │
        │           ▼
        │    C kernel execution
        │           │
        │           ▼
        │    Receipt (success/ticks)
        │
        └──► knhk-warm::execute_query() [≤500ms]
                    │
                    ▼
             Oxigraph SPARQL
                    │
                    ▼
             QueryResult (cached)
        │
        ▼
knhk-etl::EmitStage
        │
        ├──► knhk-lockchain (audit log)
        │
        └──► knhk-connectors (downstream)
```

---

## Build Topology

### Topological Sort (Dependency Order)

**Stage 0 (Parallel - 5 packages):**
```bash
cargo build -p knhk-hot        &
cargo build -p knhk-config     &
cargo build -p knhk-lockchain  &
cargo build -p knhk-otel       &
cargo build -p knhk-connectors &
wait
```

**Stage 1 (Parallel - 2 packages):**
```bash
cargo build -p knhk-validation &
cargo build -p knhk-etl        &
wait
```

**Stage 2 (Parallel - 4 packages):**
```bash
cargo build -p knhk-aot                &
cargo build -p knhk-unrdf              &
cargo build -p knhk-patterns           &
cargo build -p knhk-integration-tests  &
wait
```

**Stage 3 (Sequential - 1 package):**
```bash
cargo build -p knhk-warm
```

**Stage 4 (Sequential - 1 package):**
```bash
cargo build -p knhk-cli
```

### Build Time Estimates

| Stage | Packages | Est. Time (Debug) | Est. Time (Release) |
|-------|----------|-------------------|---------------------|
| **0** | 5 | ~30s (parallel) | ~90s (parallel) |
| **1** | 2 | ~25s (parallel) | ~70s (parallel) |
| **2** | 4 | ~35s (parallel) | ~100s (parallel) |
| **3** | 1 | ~15s | ~45s |
| **4** | 1 | ~20s | ~60s |
| **Total** | 13 | ~2m 5s | ~6m 5s |

**Optimization:**
- **Incremental:** Use `sccache` for distributed caching
- **Parallel:** `cargo build -j8` (8 cores)
- **Release:** Enable LTO in `[profile.release]` (already configured)

---

## Critical Path Analysis

### Package Criticality Rankings

| Rank | Package | Dependents | Impact | Risk |
|------|---------|------------|--------|------|
| **1** | knhk-hot | 5 | **CRITICAL** | 🔴 Breaking changes affect entire system |
| **2** | knhk-etl | 5 | **CRITICAL** | 🔴 Central orchestration; all pipelines break |
| **3** | knhk-otel | 5 | **CRITICAL** | 🔴 Telemetry loss; blind system operation |
| **4** | knhk-connectors | 5 | **CRITICAL** | 🔴 All external integrations fail |
| **5** | knhk-lockchain | 3 | **HIGH** | 🟡 Audit trail loss; compliance risk |
| **6** | knhk-warm | 1 | **MEDIUM** | 🟡 Query performance degrades |
| **7** | knhk-validation | 1 | **MEDIUM** | 🟡 Policy enforcement breaks |
| **8** | knhk-config | 2 | **MEDIUM** | 🟡 Configuration parsing fails |
| **9** | knhk-unrdf | 2 | **LOW** | 🟢 Optional RDF features |
| **10** | knhk-patterns | 0 | **LOW** | 🟢 Optional workflow patterns |
| **11** | knhk-aot | 0 | **LOW** | 🟢 AoT optimizations only |
| **12** | knhk-cli | 0 | **LOW** | 🟢 User interface only |
| **13** | knhk-integration-tests | 0 | **LOW** | 🟢 Testing infrastructure |

### Critical Path Chains

**Longest Dependency Chain (Depth 4):**
```
knhk-cli → knhk-warm → knhk-unrdf → knhk-etl → knhk-hot
```

**Critical Execution Path (Runtime):**
```
User Input (knhk-cli)
    → Pipeline Setup (knhk-etl)
    → Connector Ingestion (knhk-connectors)
    → Hot Path Execution (knhk-hot)
    → Warm Path Query (knhk-warm)
    → Lockchain Logging (knhk-lockchain)
    → Telemetry Emission (knhk-otel)
```

---

## Architectural Patterns

### 1. Layered Architecture
- **Foundation → Core → Specialized → Application**
- Clear separation of concerns
- Unidirectional dependency flow (mostly)

### 2. Plugin Architecture
- **Connectors:** Trait-based plugin system
- **Patterns:** Composable workflow patterns
- **Validation:** Pluggable policy engines

### 3. FFI Boundary Pattern
- **C for Performance:** Hot path kernels in C (≤8 ticks)
- **Rust for Safety:** Safe wrappers with type safety
- **Zero-Copy:** SoA memory layout, 64-byte aligned

### 4. Observable Architecture
- **Schema-First:** OpenTelemetry Weaver validation
- **Distributed Tracing:** OTLP exporter
- **Metrics:** Runtime class tracking

### 5. Immutable Event Log
- **Lockchain:** Git-style content-addressable storage
- **Merkle Trees:** Cryptographic audit trail
- **Quorum Consensus:** Multi-node validation

### 6. Epoch-Based Scheduling
- **8-Beat Epochs:** Deterministic tick budgeting
- **Park/Escalate:** SLO-aware workload management
- **Fiber Execution:** Cooperative multitasking

### 7. Template-Based Optimization
- **CONSTRUCT8:** Fixed templates for common patterns
- **MPHF:** Minimal perfect hash functions (AoT)
- **Specialization:** Type-specific code generation

### 8. Workflow Orchestration (Van der Aalst)
- **23 Control Patterns:** Sequence, parallel, choice, etc.
- **Hybrid Execution:** Rust logic + C hot path
- **Composable:** Pattern nesting and chaining

---

## Recommendations

### Immediate Actions

1. **✅ Add knhk-patterns to dependency graph** (completed in this analysis)
2. **⚠️ Resolve OpenTelemetry version conflict:**
   - Upgrade `knhk-unrdf` from 0.21 → 0.31
   - Update `Cargo.toml`: `opentelemetry = "0.31"`
3. **🔧 Re-enable knhk-sidecar:**
   - Fix 53 async trait errors
   - Use `#[async_trait]` or refactor to concrete types

### Long-Term Improvements

4. **📊 Dependency Policies:**
   - Max depth: 4 (current ✅)
   - Max dependents: 5 (current ✅)
   - Zero circular deps (current ✅)
5. **🚀 Build Optimization:**
   - Implement `sccache` for distributed caching
   - Add `build-stages.sh` script for parallel builds
6. **🧪 Testing Strategy:**
   - Dependency-aware test selection (via `cargo-nextest`)
   - Full workspace tests on main branch only
7. **📐 Architecture Evolution:**
   - Consider splitting `knhk-etl` if it exceeds 3,000 LOC
   - Extract shared types into `knhk-types` crate
   - Formalize FFI safety contracts with property tests

---

## Appendix A: External Dependencies

### Critical External Dependencies

| Dependency | Version | Used By | Purpose |
|------------|---------|---------|---------|
| **opentelemetry** | 0.31 | 4 packages | Observability |
| **oxigraph** | 0.5 | 3 packages | RDF/SPARQL |
| **tokio** | 1.35 | 3 packages | Async runtime |
| **blake3** | 1.5 | 3 packages | Hashing |
| **serde** | 1.0 | 8+ packages | Serialization |
| **clap** | 4.5 | 1 package | CLI framework |
| **rdkafka** | 0.36 | 2 packages | Kafka client |
| **regorus** | 0.4 | 1 package | OPA engine |

### Version Conflicts

**⚠️ OpenTelemetry Mismatch:**
- Workspace: `0.31`
- knhk-unrdf: `0.21`
- **Impact:** Type incompatibilities, binary bloat
- **Resolution:** Upgrade knhk-unrdf to 0.31 in v1.1.0

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| **SoA** | Struct of Arrays (memory layout for SIMD) |
| **FFI** | Foreign Function Interface (C ↔ Rust boundary) |
| **OTLP** | OpenTelemetry Protocol (telemetry export) |
| **MPHF** | Minimal Perfect Hash Function (O(1) lookup) |
| **Van der Aalst** | Workflow pattern taxonomy (23 patterns) |
| **Weaver** | OpenTelemetry schema validator |
| **Chatman Constant** | 8-tick performance budget |
| **Epoch** | 8-beat scheduling unit (deterministic) |
| **Fiber** | Cooperative multitasking primitive |
| **Lockchain** | Immutable event log (Git-style) |
| **AoT** | Ahead-of-Time (compilation/optimization) |

---

**End of Analysis**
