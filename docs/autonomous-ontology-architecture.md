# Autonomous Ontology System Architecture

**Version:** 1.0
**Status:** Design Phase
**Last Updated:** 2025-11-16

## Executive Summary

This document defines the architecture for KNHK's autonomous ontology system (Σ), enabling runtime knowledge graph evolution at hardware speed with zero human intervention. The system implements a four-plane architecture: Observation (O), Ontology (Σ), Change (ΔΣ + Q), and Projection/Execution (μ, Π, Λ).

## Table of Contents

1. [System Overview](#system-overview)
2. [Four-Plane Architecture](#four-plane-architecture)
3. [Rust Crate Structure](#rust-crate-structure)
4. [Component Design](#component-design)
5. [Data Flow Architecture](#data-flow-architecture)
6. [Integration with Existing KNHK](#integration-with-existing-knhk)
7. [Memory Management Strategy](#memory-management-strategy)
8. [Performance Considerations](#performance-considerations)
9. [FFI Boundaries](#ffi-boundaries)
10. [Registry Schema Integration](#registry-schema-integration)

---

## 1. System Overview

### 1.1 Purpose

Enable KNHK to autonomously evolve its understanding of workflow patterns, process structures, and execution semantics based on observed telemetry, without requiring manual ontology updates or human-driven schema migrations.

### 1.2 Key Capabilities

- **Hardware-Speed Evolution**: Ontology changes happen in microseconds (≤8 ticks hot path)
- **Zero Human Intervention**: Fully autonomous pattern detection and ontology refinement
- **Provenance Tracking**: Complete audit trail via receipts and change logs
- **Safe Evolution**: Validation gates prevent breaking changes
- **Continuous Learning**: Closed-loop control with automatic rollback

### 1.3 Design Principles

1. **Schema-First**: All telemetry follows OTel Weaver schemas (source of truth)
2. **Minimal Hot Path Impact**: C hot path remains untouched except for FFI entry points
3. **Fail-Safe Operation**: Invalid changes never reach production
4. **Observable by Design**: Every plane emits structured telemetry
5. **Composable Architecture**: Planes operate independently but coordinate seamlessly

---

## 2. Four-Plane Architecture

### 2.1 Plane O: Observation

**Purpose**: Detect patterns and anomalies from telemetry streams.

**Components**:
- Pattern Miners (workflow patterns, temporal sequences, resource utilization)
- Anomaly Detectors (conformance violations, bottlenecks)
- Telemetry Aggregators (span collectors, metric reducers)

**Inputs**: OTLP telemetry streams (spans, metrics, logs)
**Outputs**: Pattern candidates, anomaly reports → Queue Q

**Key Characteristics**:
- Read-only: Never modifies ontology directly
- High throughput: Processes millions of spans/sec
- Low latency: Real-time pattern detection
- Fault tolerant: Dropped telemetry doesn't break the system

### 2.2 Plane Σ: Ontology (Runtime Knowledge Graph)

**Purpose**: Maintain current understanding of workflows, patterns, and semantics.

**Components**:
- Snapshot Manager (immutable point-in-time views)
- Overlay Engine (trial changes without commit)
- Receipt System (cryptographic proof of changes)
- Query Interface (graph traversal, SPARQL-like queries)

**Storage**: In-memory RDF graph (oxigraph) with persistent snapshots
**Access Pattern**: Read-heavy (99.9%), occasional writes (0.1%)

**Key Characteristics**:
- Immutable snapshots: All historical states preserved
- Copy-on-write overlays: Safe experimentation
- Cryptographic receipts: Blake3 hash chains for provenance
- Fast queries: Sub-millisecond graph traversal

### 2.3 Plane ΔΣ + Q: Change Engine

**Purpose**: Propose, validate, and apply ontology modifications.

**Components**:
- ΔΣ Proposers (suggest additions, modifications, deletions)
- Validators (conformance checkers, conflict detectors)
- Queue Q (ordered change proposals)
- Conflict Resolvers (merge strategies, priority rules)

**Inputs**: Pattern candidates from Observation
**Outputs**: Validated change sets → Applied to Σ

**Key Characteristics**:
- Consensus-based: Multiple validators vote on changes
- Non-blocking: Queue prevents backpressure
- Idempotent: Same pattern detected multiple times = same change
- Rollback-safe: Invalid changes trigger automatic revert

### 2.4 Plane μ, Π, Λ: Projection/Execution

**Purpose**: Compile ontology into executable artifacts (ggen templates, hot path code).

**Components**:
- μ (Model Projection): Σ → ggen templates
- Π (Pipeline Projection): ggen → compiled Rust/C
- Λ (Linker): Hot-reload compiled code into running process

**Inputs**: Ontology snapshots (Σ)
**Outputs**: Executable templates, shared libraries, updated hot path

**Key Characteristics**:
- Just-in-time compilation: Changes live in <100ms
- Hot-reload safe: No process restarts required
- Rollback on failure: Bad compilations don't break production
- Telemetry-validated: Weaver checks ensure correctness

---

## 3. Rust Crate Structure

### 3.1 New Crates (Autonomous Ontology System)

```
rust/
├── knhk-ontology-runtime/       # Core ontology management (Σ)
│   ├── src/
│   │   ├── snapshot.rs          # Immutable snapshots
│   │   ├── overlay.rs           # Copy-on-write overlays
│   │   ├── receipt.rs           # Cryptographic provenance
│   │   ├── query.rs             # Graph query interface
│   │   └── storage.rs           # Oxigraph integration
│   └── Cargo.toml
│
├── knhk-observation/            # Plane O (Pattern detection)
│   ├── src/
│   │   ├── pattern_miner.rs    # Workflow pattern detection
│   │   ├── anomaly_detector.rs # Conformance violations
│   │   ├── aggregator.rs       # Telemetry aggregation
│   │   └── otlp_collector.rs   # OTLP stream processing
│   └── Cargo.toml
│
├── knhk-change-engine/          # Plane ΔΣ + Q (Change management)
│   ├── src/
│   │   ├── proposer.rs         # Change proposal generation
│   │   ├── validator.rs        # Validation logic
│   │   ├── queue.rs            # Change queue (Q)
│   │   ├── conflict.rs         # Conflict resolution
│   │   └── consensus.rs        # Multi-validator consensus
│   └── Cargo.toml
│
├── knhk-projection/             # Plane μ, Π, Λ (Compilation)
│   ├── src/
│   │   ├── model_projection.rs # Σ → ggen templates (μ)
│   │   ├── pipeline.rs         # ggen → compiled artifacts (Π)
│   │   ├── linker.rs           # Hot-reload mechanism (Λ)
│   │   └── codegen.rs          # Code generation utilities
│   └── Cargo.toml
│
├── knhk-ontology-meta/          # Meta-ontology (Σ²)
│   ├── src/
│   │   ├── schema.rs           # Meta-ontology schema definitions
│   │   ├── constraints.rs      # Evolution constraints
│   │   └── rules.rs            # Change validation rules
│   └── Cargo.toml
│
└── knhk-ontology-cli/           # CLI for ontology inspection
    ├── src/
    │   ├── inspect.rs          # View snapshots, overlays
    │   ├── query.rs            # SPARQL-like queries
    │   ├── diff.rs             # Compare snapshots
    │   └── export.rs           # Export to Turtle/RDF
    └── Cargo.toml
```

### 3.2 Updated Workspace Cargo.toml

```toml
[workspace]
members = [
    # ... existing crates ...
    "knhk-ontology-runtime",
    "knhk-observation",
    "knhk-change-engine",
    "knhk-projection",
    "knhk-ontology-meta",
    "knhk-ontology-cli",
]

[workspace.dependencies]
# New ontology system dependencies
knhk-ontology-runtime = { path = "./knhk-ontology-runtime", version = "1.0.0" }
knhk-observation = { path = "./knhk-observation", version = "1.0.0" }
knhk-change-engine = { path = "./knhk-change-engine", version = "1.0.0" }
knhk-projection = { path = "./knhk-projection", version = "1.0.0" }
knhk-ontology-meta = { path = "./knhk-ontology-meta", version = "1.0.0" }

# RDF & semantic web (existing)
oxigraph = "0.5"

# Graph algorithms
petgraph = "0.6"

# Pattern matching
regex = "1.10"
fancy-regex = "0.13"

# Template engine (for projection)
tera = { version = "1.19", features = ["builtins"] }

# Hot-reload support
libloading = "0.8"

# Consensus algorithms
raft = "0.7"
```

### 3.3 Dependency Graph

```
knhk-ontology-cli
    └── knhk-ontology-runtime
            ├── oxigraph
            ├── knhk-ontology-meta
            └── knhk-otel (for telemetry)

knhk-observation
    ├── knhk-otel (OTLP processing)
    ├── knhk-process-mining (pattern detection)
    └── knhk-patterns (Van der Aalst patterns)

knhk-change-engine
    ├── knhk-ontology-runtime (read/write Σ)
    ├── knhk-ontology-meta (validation rules)
    └── knhk-observation (receives pattern candidates)

knhk-projection
    ├── knhk-ontology-runtime (read Σ snapshots)
    ├── tera (template engine)
    └── libloading (hot-reload)

knhk-warm (updated)
    ├── knhk-ontology-runtime (FFI integration)
    └── knhk-projection (receive compiled artifacts)
```

---

## 4. Component Design

### 4.1 Snapshot Manager (Plane Σ)

**Responsibility**: Maintain immutable point-in-time views of ontology.

```rust
// knhk-ontology-runtime/src/snapshot.rs

pub struct Snapshot {
    /// Unique snapshot ID (Blake3 hash of RDF graph)
    pub id: SnapshotId,

    /// Parent snapshot (for diff chains)
    pub parent: Option<SnapshotId>,

    /// RDF graph (oxigraph store)
    pub graph: Arc<Store>,

    /// Metadata (creation time, change summary)
    pub metadata: SnapshotMetadata,

    /// Cryptographic receipt (Blake3 hash chain)
    pub receipt: Receipt,
}

impl Snapshot {
    /// Create snapshot from current ontology state
    pub fn capture(store: &Store) -> Result<Self, OntologyError>;

    /// Load snapshot from persistent storage
    pub fn load(id: &SnapshotId) -> Result<Self, OntologyError>;

    /// Query snapshot using SPARQL-like syntax
    pub fn query(&self, sparql: &str) -> Result<QueryResult, OntologyError>;

    /// Compute diff between two snapshots
    pub fn diff(&self, other: &Snapshot) -> SnapshotDiff;

    /// Export to Turtle/RDF format
    pub fn export(&self, format: RdfFormat) -> Result<String, OntologyError>;
}
```

**Storage Strategy**:
- In-memory: Current snapshot + N recent snapshots (LRU cache)
- On-disk: All historical snapshots (Sled key-value store)
- Cloud: Periodic backups to S3-compatible storage

**Performance Requirements**:
- Snapshot creation: <10ms (including hash computation)
- Snapshot load: <5ms (from disk)
- Query latency: <1ms (for typical workflow queries)

### 4.2 Overlay Engine (Plane Σ)

**Responsibility**: Trial changes without modifying production ontology.

```rust
// knhk-ontology-runtime/src/overlay.rs

pub struct Overlay {
    /// Base snapshot (immutable)
    base: Arc<Snapshot>,

    /// Additions (RDF triples to add)
    additions: Vec<RdfTriple>,

    /// Deletions (RDF triples to remove)
    deletions: Vec<RdfTriple>,

    /// Modifications (updates to existing triples)
    modifications: Vec<RdfModification>,
}

impl Overlay {
    /// Create overlay on top of snapshot
    pub fn new(base: Arc<Snapshot>) -> Self;

    /// Add RDF triple to overlay
    pub fn add_triple(&mut self, triple: RdfTriple);

    /// Remove RDF triple in overlay
    pub fn remove_triple(&mut self, triple: RdfTriple);

    /// Query overlay (applies changes on top of base)
    pub fn query(&self, sparql: &str) -> Result<QueryResult, OntologyError>;

    /// Commit overlay → create new snapshot
    pub fn commit(self) -> Result<Snapshot, OntologyError>;

    /// Discard overlay (no changes applied)
    pub fn discard(self);
}
```

**Use Cases**:
- Validate proposed changes before commit
- Run "what-if" queries for change impact analysis
- Parallel trial of multiple competing change proposals

### 4.3 Pattern Miner (Plane O)

**Responsibility**: Detect workflow patterns from telemetry.

```rust
// knhk-observation/src/pattern_miner.rs

pub struct PatternMiner {
    /// Van der Aalst pattern recognizers (1-43)
    recognizers: Vec<Box<dyn PatternRecognizer>>,

    /// Temporal sequence analyzer
    sequence_analyzer: SequenceAnalyzer,

    /// Resource utilization tracker
    resource_tracker: ResourceTracker,
}

impl PatternMiner {
    /// Process incoming OTLP span
    pub fn process_span(&mut self, span: Span) -> Vec<PatternCandidate>;

    /// Analyze batch of spans for multi-span patterns
    pub fn analyze_batch(&mut self, spans: &[Span]) -> Vec<PatternCandidate>;

    /// Register custom pattern recognizer
    pub fn register_recognizer(&mut self, recognizer: Box<dyn PatternRecognizer>);
}

pub trait PatternRecognizer: Send + Sync {
    /// Pattern ID (e.g., Van der Aalst pattern 1-43)
    fn pattern_id(&self) -> PatternId;

    /// Detect pattern in span(s)
    fn recognize(&self, spans: &[Span]) -> Option<PatternCandidate>;

    /// Confidence score (0.0-1.0)
    fn confidence(&self, candidate: &PatternCandidate) -> f64;
}

pub struct PatternCandidate {
    /// Pattern ID
    pub pattern_id: PatternId,

    /// Evidence spans
    pub spans: Vec<SpanId>,

    /// Confidence score
    pub confidence: f64,

    /// Suggested ontology change (ΔΣ)
    pub change_proposal: ChangeProposal,
}
```

**Pattern Detection Strategies**:
1. **Rule-based**: Match known patterns (Van der Aalst 1-43)
2. **Statistical**: Frequency analysis, correlation detection
3. **ML-based**: Trained classifiers for novel patterns (future)

### 4.4 Change Proposer & Validator (Plane ΔΣ)

**Responsibility**: Generate and validate ontology change proposals.

```rust
// knhk-change-engine/src/proposer.rs

pub struct ChangeProposal {
    /// Unique proposal ID
    pub id: ProposalId,

    /// Source (pattern detection, manual, external system)
    pub source: ChangeSource,

    /// RDF changes (additions, deletions, modifications)
    pub delta: OntologyDelta,

    /// Justification (evidence spans, confidence scores)
    pub justification: Justification,

    /// Priority (0-100, higher = more urgent)
    pub priority: u8,
}

pub struct Validator {
    /// Meta-ontology constraints (Σ²)
    constraints: Vec<Box<dyn Constraint>>,

    /// Conflict detector
    conflict_detector: ConflictDetector,
}

impl Validator {
    /// Validate proposal against meta-ontology
    pub fn validate(&self, proposal: &ChangeProposal) -> ValidationResult;

    /// Check for conflicts with pending proposals
    pub fn check_conflicts(&self, proposal: &ChangeProposal, queue: &Queue) -> Vec<Conflict>;

    /// Suggest resolution for conflicts
    pub fn resolve_conflicts(&self, conflicts: Vec<Conflict>) -> Resolution;
}

pub enum ValidationResult {
    Approved { score: f64 },
    Rejected { reason: RejectionReason },
    NeedsReview { concerns: Vec<String> },
}
```

**Validation Checks**:
1. **Constraint Satisfaction**: Σ² rules must hold
2. **Conflict Detection**: No contradictory changes
3. **Impact Analysis**: Estimate blast radius of change
4. **Rollback Safety**: Ensure changes can be reverted

### 4.5 Projection Pipeline (Plane μ, Π, Λ)

**Responsibility**: Compile ontology into executable code.

```rust
// knhk-projection/src/model_projection.rs

pub struct ModelProjector {
    /// Template engine (Tera)
    engine: tera::Tera,

    /// ggen template registry
    templates: HashMap<String, GgenTemplate>,
}

impl ModelProjector {
    /// Project ontology snapshot → ggen template
    pub fn project(&self, snapshot: &Snapshot) -> Result<GgenTemplate, ProjectionError>;

    /// Incremental projection (only changed parts)
    pub fn project_delta(&self, delta: &SnapshotDiff) -> Result<GgenTemplate, ProjectionError>;
}

// knhk-projection/src/pipeline.rs

pub struct PipelineCompiler {
    /// Rust compiler handle
    rust_compiler: RustCompiler,

    /// C compiler handle
    c_compiler: CCompiler,
}

impl PipelineCompiler {
    /// Compile ggen template → Rust/C code
    pub fn compile(&self, template: &GgenTemplate) -> Result<CompiledArtifact, CompilationError>;

    /// Incremental compilation (only changed parts)
    pub fn compile_incremental(&self, delta: &GgenTemplate) -> Result<CompiledArtifact, CompilationError>;
}

// knhk-projection/src/linker.rs

pub struct HotReloader {
    /// Currently loaded library
    current: Option<Library>,
}

impl HotReloader {
    /// Hot-reload compiled artifact (no restart)
    pub fn reload(&mut self, artifact: &CompiledArtifact) -> Result<(), LinkerError>;

    /// Rollback to previous version
    pub fn rollback(&mut self) -> Result<(), LinkerError>;
}
```

**Compilation Strategy**:
- **Incremental**: Only recompile changed components
- **Parallel**: Compile independent modules concurrently
- **Fallback**: Keep previous version loaded until new one validates

---

## 5. Data Flow Architecture

### 5.1 Telemetry → Pattern → Change → Projection

```
┌─────────────────────────────────────────────────────────────┐
│                    OTLP Telemetry Streams                   │
│              (spans, metrics, logs from runtime)            │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Plane O: Observation                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Pattern      │  │ Anomaly      │  │ Aggregator   │     │
│  │ Miner        │  │ Detector     │  │              │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└───────────────────────────┬─────────────────────────────────┘
                            │ Pattern Candidates
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Plane ΔΣ + Q: Change Engine                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Proposer     │→ │ Queue (Q)    │→ │ Validator    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└───────────────────────────┬─────────────────────────────────┘
                            │ Validated ΔΣ
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Plane Σ: Ontology Runtime                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Snapshot     │  │ Overlay      │  │ Receipt      │     │
│  │ Manager      │  │ Engine       │  │ System       │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└───────────────────────────┬─────────────────────────────────┘
                            │ Snapshot (Σ)
                            ▼
┌─────────────────────────────────────────────────────────────┐
│           Plane μ, Π, Λ: Projection/Execution               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Model        │→ │ Pipeline     │→ │ Hot          │     │
│  │ Projection   │  │ Compiler     │  │ Reloader     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└───────────────────────────┬─────────────────────────────────┘
                            │ Updated Runtime
                            ▼
                   ┌──────────────────┐
                   │  C Hot Path      │
                   │  Rust Warm Path  │
                   └──────────────────┘
```

### 5.2 Closed-Loop Control

```
Runtime Execution → Telemetry → Observation → Change → Ontology → Projection → Runtime Execution
                                                                                        │
                                                                                        └──────┘
                                                                                   (continuous loop)
```

**Feedback Mechanisms**:
1. **Performance Monitoring**: If changes degrade latency → rollback
2. **Conformance Checking**: If execution violates ontology → alert
3. **Anomaly Detection**: If novel patterns emerge → propose ontology update

---

## 6. Integration with Existing KNHK

### 6.1 Minimal C Hot Path Changes

**Current C Hot Path** (`c/src/knhk_hot.c`):
- No changes to core parsing logic
- No changes to performance-critical paths

**New FFI Entry Points**:
```c
// c/include/knhk_ontology_ffi.h

// Initialize ontology runtime (called at startup)
int knhk_ontology_init(const char* snapshot_path);

// Query ontology (called from warm path)
int knhk_ontology_query(const char* sparql, char** result);

// Shutdown ontology runtime (called at exit)
int knhk_ontology_shutdown(void);
```

**Integration Points**:
- **Startup**: Load initial ontology snapshot
- **Query**: Rust warm path queries ontology via FFI
- **Shutdown**: Graceful shutdown with snapshot persistence

### 6.2 Rust Warm Path Extensions

**Updated `knhk-warm` Crate**:

```rust
// knhk-warm/src/ontology_integration.rs

use knhk_ontology_runtime::{Snapshot, OntologyRuntime};

pub struct WarmPathOntology {
    runtime: Arc<OntologyRuntime>,
    current_snapshot: Arc<Snapshot>,
}

impl WarmPathOntology {
    /// Initialize from snapshot
    pub fn new(snapshot_path: &Path) -> Result<Self, OntologyError>;

    /// Query ontology (called from workflow engine)
    pub fn query(&self, sparql: &str) -> Result<QueryResult, OntologyError>;

    /// Refresh snapshot (when ontology changes)
    pub fn refresh(&mut self) -> Result<(), OntologyError>;
}
```

**Affected Crates**:
- `knhk-warm`: Add ontology query interface
- `knhk-workflow-engine`: Use ontology for pattern lookup
- `knhk-process-mining`: Store discovered patterns in ontology
- `knhk-patterns`: Register Van der Aalst patterns in ontology

### 6.3 CLI Integration

**New Commands** (`knhk ontology` subcommands):

```bash
# Inspect current ontology snapshot
knhk ontology inspect

# Query ontology with SPARQL
knhk ontology query "SELECT ?pattern WHERE { ... }"

# List all snapshots
knhk ontology snapshots

# Compare two snapshots
knhk ontology diff <snapshot1> <snapshot2>

# Export ontology to Turtle/RDF
knhk ontology export --format turtle > ontology.ttl

# Visualize ontology graph
knhk ontology visualize --output graph.svg

# Show change history
knhk ontology history

# Apply manual change (admin only)
knhk ontology apply-change --proposal <proposal.json>
```

### 6.4 Registry/Weaver Integration

**New Registry Schemas** (`registry/knhk-ontology.yaml`):

```yaml
groups:
  # Ontology runtime attributes
  - id: knhk.ontology.attributes
    type: attribute_group
    brief: "Autonomous ontology system attributes"
    stability: experimental
    attributes:
      - id: knhk.ontology.snapshot_id
        type: string
        stability: experimental
        brief: "Snapshot identifier (Blake3 hash)"

      - id: knhk.ontology.change_count
        type: int
        stability: experimental
        brief: "Number of changes in proposal"

      - id: knhk.ontology.pattern_id
        type: string
        stability: experimental
        brief: "Detected pattern identifier"

  # Snapshot creation span
  - id: knhk.ontology.snapshot.create
    type: span
    span_kind: internal
    stability: experimental
    brief: "Create ontology snapshot"
    attributes:
      - ref: knhk.ontology.snapshot_id
      - ref: knhk.operation.latency_ms

  # Pattern detection span
  - id: knhk.ontology.pattern.detect
    type: span
    span_kind: internal
    stability: experimental
    brief: "Detect workflow pattern"
    attributes:
      - ref: knhk.ontology.pattern_id
      - ref: knhk.observation.confidence

  # Change validation span
  - id: knhk.ontology.change.validate
    type: span
    span_kind: internal
    stability: experimental
    brief: "Validate ontology change proposal"
    attributes:
      - ref: knhk.ontology.change_count
      - ref: knhk.change.validation_result

  # Projection compilation span
  - id: knhk.ontology.projection.compile
    type: span
    span_kind: internal
    stability: experimental
    brief: "Compile ontology to executable artifacts"
    attributes:
      - ref: knhk.projection.compilation_time_ms
      - ref: knhk.projection.artifact_size_bytes
```

---

## 7. Memory Management Strategy

### 7.1 Snapshot Storage

**In-Memory Cache** (LRU):
- Current snapshot (always cached)
- N most recent snapshots (configurable, default 10)
- Total memory: ~100MB for 10 snapshots (typical workflow ontology)

**Persistent Storage** (Sled):
- All historical snapshots
- Key: Blake3 hash (32 bytes)
- Value: Compressed RDF graph (gzip, ~10KB per snapshot)
- Total disk: ~1MB for 100 snapshots

**Cloud Backup** (Optional):
- Periodic snapshots uploaded to S3
- Retention policy: Keep all snapshots for 30 days, then monthly archives

### 7.2 Overlay Management

**Copy-on-Write**:
- Base snapshot: Shared Arc (zero-copy)
- Delta: Small Vec of RDF triple changes (~1KB)
- Typical overlay: <10KB memory overhead

**Concurrent Overlays**:
- Multiple overlays can exist simultaneously on same base
- Independent validation trials
- No contention (each overlay is isolated)

### 7.3 Change Queue

**Bounded Queue** (crossbeam):
- Capacity: 1000 proposals (configurable)
- Backpressure: Block pattern detection if queue full
- Memory: ~1MB for full queue (1KB per proposal)

---

## 8. Performance Considerations

### 8.1 Hot Path Requirements

**Constraint**: Operations affecting C hot path ≤8 ticks (Chatman Constant).

**Solutions**:
1. **No ontology queries in hot path**: All queries happen in warm path
2. **Precomputed lookups**: Projection precompiles frequent queries
3. **FFI overhead**: Minimal (init/shutdown only, not per-request)

### 8.2 Warm Path Targets

**Ontology Query Latency**:
- Simple lookup: <1ms (e.g., "What is pattern 12?")
- Graph traversal: <10ms (e.g., "Find all related patterns")
- Complex SPARQL: <100ms (acceptable for admin CLI)

**Change Application Latency**:
- Snapshot creation: <10ms
- Change validation: <50ms
- Projection compilation: <1s (incremental), <10s (full)

### 8.3 Scalability

**Telemetry Throughput**:
- Pattern detection: 1M spans/sec (single instance)
- Horizontal scaling: Multiple observation instances (stateless)

**Ontology Size**:
- Initial: ~1000 RDF triples (Van der Aalst patterns + basic workflow concepts)
- Growth rate: ~100 triples/month (new patterns discovered)
- Scalability: Oxigraph handles millions of triples efficiently

---

## 9. FFI Boundaries

### 9.1 C → Rust (Ontology Queries)

**Entry Point**: `knhk_ontology_query`

```c
// c/include/knhk_ontology_ffi.h

// Query ontology (thread-safe)
// Returns 0 on success, error code on failure
// Result is JSON string (caller must free with knhk_free_string)
int knhk_ontology_query(const char* sparql, char** result);

// Free string allocated by Rust
void knhk_free_string(char* s);
```

**Rust Implementation**:
```rust
// knhk-ontology-runtime/src/ffi.rs

#[no_mangle]
pub extern "C" fn knhk_ontology_query(
    sparql: *const c_char,
    result: *mut *mut c_char,
) -> c_int {
    // Safety: sparql must be valid C string
    let sparql_str = unsafe { CStr::from_ptr(sparql).to_str().unwrap() };

    match ONTOLOGY_RUNTIME.query(sparql_str) {
        Ok(query_result) => {
            let json = serde_json::to_string(&query_result).unwrap();
            unsafe { *result = CString::new(json).unwrap().into_raw(); }
            0 // success
        }
        Err(e) => {
            tracing::error!("Ontology query failed: {}", e);
            -1 // error
        }
    }
}

#[no_mangle]
pub extern "C" fn knhk_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}
```

### 9.2 Rust → C (Hot Path Callbacks)

**Not Required**: Ontology system operates independently from hot path.

**Future Extension** (if needed):
- Hot path could emit telemetry via FFI
- Rust observation layer receives telemetry for pattern detection

---

## 10. Registry Schema Integration

### 10.1 Telemetry Requirements

**Every Plane Emits Structured Telemetry**:

| Plane | Span Examples | Metric Examples |
|-------|---------------|-----------------|
| **O** | `knhk.observation.pattern.detect` | `knhk.observation.patterns_detected` |
| **Σ** | `knhk.ontology.snapshot.create` | `knhk.ontology.snapshot_count` |
| **ΔΣ** | `knhk.change.proposal.validate` | `knhk.change.queue_depth` |
| **μ,Π,Λ** | `knhk.projection.compile` | `knhk.projection.compilation_time` |

### 10.2 Weaver Validation Gates

**Phase 1 (Meta-Ontology)**:
- [ ] `weaver registry check -r registry/` passes
- [ ] All ontology operations have schema definitions

**Phase 2 (Runtime)**:
- [ ] `weaver registry live-check` validates snapshot operations
- [ ] Overlay operations emit correct telemetry

**Phase 3 (Change Engine)**:
- [ ] Change validation emits conformance telemetry
- [ ] Queue operations are observable

**Phase 4 (Projection)**:
- [ ] Compilation emits build metrics
- [ ] Hot-reload emits success/failure events

**Phase 5 (Closed-Loop)**:
- [ ] End-to-end telemetry shows complete flow
- [ ] Performance metrics verify ≤8 ticks hot path

### 10.3 Schema Evolution

**Ontology Changes Require Schema Updates**:
1. Propose ontology change (ΔΣ)
2. Validator checks if new telemetry is needed
3. If yes: Update `registry/knhk-ontology.yaml`
4. Run `weaver registry check` to validate
5. Only then apply ontology change

**Prevents**:
- Ontology evolution breaking telemetry contracts
- Unobservable changes sneaking into production

---

## Appendix A: Open Questions

1. **Conflict Resolution Strategy**: When multiple validators disagree, how to reach consensus?
   - Proposed: Majority vote with tie-breaking via priority

2. **Rollback Triggers**: What metrics trigger automatic rollback?
   - Proposed: Latency >10ms increase, error rate >1%, conformance violations >5%

3. **Cloud Storage**: S3-compatible backend for snapshot backups?
   - Proposed: Optional feature, enabled via CLI flag

4. **Horizontal Scaling**: How to coordinate multiple observation instances?
   - Proposed: Shared change queue (Redis/Kafka), consensus via Raft

## Appendix B: Future Extensions

- **Machine Learning Pattern Detection**: Train classifiers on historical telemetry
- **Multi-Tenant Ontologies**: Per-customer ontology isolation
- **Distributed Ontology**: Federated ontology across multiple KNHK instances
- **Visual Ontology Editor**: Web UI for manual ontology curation

---

**Document Status**: Draft v1.0 - Ready for Review
**Next Steps**: Review with engineering team, validate assumptions, begin Phase 1 implementation
**Owners**: KNHK Core Team, Ontology Working Group
