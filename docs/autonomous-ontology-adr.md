# Architecture Decision Records - Autonomous Ontology System

**Project**: KNHK Autonomous Ontology System
**Date**: 2025-11-16
**Status**: Proposed

---

## ADR-001: Four-Plane Architecture

**Date**: 2025-11-16
**Status**: Accepted
**Decision Makers**: System Architecture Designer

### Context

KNHK needs a mechanism for safe, versioned ontology evolution that preserves performance guarantees while enabling runtime schema changes. Traditional approaches either lack versioning (direct graph mutation) or require system downtime (schema migrations).

### Decision

We will implement a **four-plane architecture** separating:
1. **Observation (O)**: Raw data and events
2. **Ontology (Σ)**: Versioned schema snapshots
3. **Change (ΔΣ + Q)**: Validation and evolution
4. **Projection (μ, Π, Λ)**: Code generation and execution

### Rationale

**Separation of Concerns**:
- Observation plane focuses on data ingestion and storage
- Ontology plane manages schema versioning
- Change plane validates evolution
- Projection plane generates code deterministically

**Performance Isolation**:
- Ontology changes don't affect hot path until recompilation
- Atomic pointer swap for promotion (~1ns)
- Hot path remains ≤8 ticks

**Safety**:
- All changes validated before promotion
- Rollback is instantaneous (pointer swap)
- Complete audit trail via receipts

**Alternatives Considered**:

1. **Direct Graph Mutation**: Fast but no versioning, no safety
2. **Schema Migration Scripts**: Traditional but requires downtime
3. **Dual-Graph Approach**: Complex, requires synchronization

### Consequences

**Positive**:
- Safe ontology evolution without downtime
- Complete version history
- Deterministic code generation
- Performance preserved

**Negative**:
- Additional complexity (4 planes to manage)
- Memory overhead (multiple snapshots)
- Code regeneration required for hot path changes

**Risks**:
- Snapshot proliferation (mitigate: compression, archival)
- Synchronization between planes (mitigate: atomic operations)

---

## ADR-002: Snapshot-Based Versioning

**Date**: 2025-11-16
**Status**: Accepted

### Context

Ontologies need versioning to support safe evolution, rollback, and reproducible builds. Options include:
1. Git-like delta chains
2. Immutable snapshots
3. Hybrid (snapshots + deltas)

### Decision

We will use **immutable snapshots** with content-addressable IDs (128-bit SHA-512 truncated).

### Rationale

**Atomic Transitions**:
- Snapshot promotion is a single pointer swap (~1ns)
- No partial states during transition
- Instant rollback (swap pointer back)

**Content Addressing**:
- Snapshot ID = hash(ontology graph)
- Deterministic: same content → same ID
- Integrity: tamper detection built-in

**Simplicity**:
- Easier to reason about than delta chains
- No complex merge logic
- Snapshots are independent (no dependencies)

**Trade-offs**:
- **Storage**: Full snapshots use more space than deltas
  - **Mitigation**: Delta compression for persistent storage
- **Computation**: Hashing entire graph on creation
  - **Mitigation**: Acceptable for infrequent operations (~1/hour)

### Implementation

```rust
pub struct SnapshotId([u8; 16]);  // 128-bit content hash

impl SnapshotId {
    pub fn from_graph(graph: &Graph) -> Self {
        // URDNA2015 canonicalization + SHA-512 truncated
        let canonical = canonicalize_graph(graph);
        let hash = sha512(&canonical);
        Self(hash[0..16])
    }
}
```

### Consequences

**Positive**:
- Picosecond-scale promotion
- Tamper-evident
- Reproducible builds

**Negative**:
- Storage overhead
- Hashing cost

---

## ADR-003: Meta-Ontology (Σ²) for Self-Description

**Date**: 2025-11-16
**Status**: Accepted

### Context

Ontologies need rules for valid evolution. Options:
1. Hard-coded validation logic
2. SHACL shapes for ontologies
3. Meta-ontology describing ontology structure

### Decision

We will implement a **meta-ontology (Σ²)** that describes the structure of ontologies themselves, validated by SHACL.

### Rationale

**Self-Describing**:
- Ontologies describe their own structure
- Meta-ontology is itself an ontology
- Bootstrap: Σ² validates Σ, Σ validates O

**Declarative Validation**:
- SHACL shapes define valid ontologies
- No hard-coded logic
- Extensible (add new shapes)

**Versioning Semantics**:
- Compatibility flags in meta-ontology
- Migration rules as ontology elements
- Semantic versioning enforced

**Example**:

```turtle
meta:Class a rdfs:Class ;
    rdfs:label "Meta-Class" ;
    rdfs:comment "A class in an ontology" .

meta:ValidClassShape a sh:NodeShape ;
    sh:targetClass meta:Class ;
    sh:property [
        sh:path rdfs:label ;
        sh:minCount 1 ;
    ] .
```

### Consequences

**Positive**:
- Flexible validation rules
- Self-documenting
- Extensible

**Negative**:
- Bootstrap complexity
- Meta-ontology must be stable

---

## ADR-004: Atomic Pointer Swap for Promotion

**Date**: 2025-11-16
**Status**: Accepted

### Context

Snapshot promotion must be:
1. Atomic (no partial states)
2. Fast (≤1ns preferred)
3. Thread-safe

Options:
1. Mutex-protected swap
2. RwLock with write lock
3. Atomic pointer (AtomicU128)

### Decision

Use **AtomicU128** for current snapshot pointer.

### Rationale

**Performance**:
- Single atomic instruction (~1ns)
- No lock contention
- Lock-free reads

**Atomicity**:
- Hardware-guaranteed atomic swap
- No partial states visible

**Simplicity**:
- No deadlocks (no locks)
- Straightforward implementation

**Implementation**:

```rust
pub struct OntologyState {
    current: AtomicU128,  // snapshot_id as u128
    snapshots: Arc<RwLock<HashMap<SnapshotId, Arc<SigmaSnapshot>>>>,
}

impl OntologyState {
    pub fn promote_snapshot(&self, new_id: SnapshotId) -> Result<()> {
        // Atomic pointer swap (picosecond-scale)
        self.current.swap(new_id.as_u128(), Ordering::SeqCst);
        Ok(())
    }
}
```

### Consequences

**Positive**:
- ~1ns promotion time
- Lock-free
- Simple implementation

**Negative**:
- Snapshot ID limited to 128 bits (acceptable for SHA-512 truncated)

---

## ADR-005: Overlays for Experimental Changes

**Date**: 2025-11-16
**Status**: Accepted

### Context

Ontology changes need a staging area for:
1. Experimentation (LLMs proposing changes)
2. Validation (before promotion)
3. Review (human-in-the-loop)

Options:
1. Branch snapshots (like git branches)
2. Overlay diffs (base + delta)
3. Temporary graphs

### Decision

Use **SigmaOverlay** with diff operations (add/remove triples).

### Rationale

**Lightweight**:
- Only store diffs, not full graphs
- Multiple overlays on same base
- Cheap to create/delete

**Validation**:
- Apply overlay → validate result
- If invalid, discard overlay
- If valid, promote to snapshot

**Collaboration**:
- Multiple overlays in parallel
- No conflicts (each is independent)
- Merge by promotion

**Implementation**:

```rust
pub struct SigmaOverlay {
    overlay_id: OverlayId,
    base: SnapshotId,
    diff: SigmaDiff {
        add: Vec<Triple>,
        remove: Vec<Triple>,
    },
}

pub fn apply_overlay(base: &SigmaSnapshot, overlay: &SigmaOverlay) -> Graph {
    let mut result = base.triples.clone();
    for triple in &overlay.diff.remove { result.remove(triple); }
    for triple in &overlay.diff.add { result.insert(triple.clone()); }
    result
}
```

### Consequences

**Positive**:
- Efficient staging area
- Parallel experimentation
- Safe validation workflow

**Negative**:
- Overlay proliferation (mitigate: garbage collection)

---

## ADR-006: Hard Invariants (Q) as Trait Objects

**Date**: 2025-11-16
**Status**: Accepted

### Context

Hard invariants (Q) must be:
1. Extensible (add new invariants)
2. Composable (check multiple invariants)
3. Testable (validate each independently)

Options:
1. Hard-coded checks
2. SHACL shapes only
3. Trait objects (`dyn InvariantChecker`)

### Decision

Use **trait objects** for invariant checkers.

### Rationale

**Extensibility**:
- Add new invariants without changing core
- Domain-specific invariants
- Custom performance checks

**Testability**:
- Test each invariant independently
- Mock checkers for testing
- Clear pass/fail semantics

**Composability**:
- Combine multiple checkers
- Short-circuit on failure
- Severity levels (Critical, Warning, Info)

**Implementation**:

```rust
pub trait InvariantChecker: Send + Sync {
    fn check(&self, graph: &Graph) -> Result<InvariantReport, InvariantError>;
    fn name(&self) -> &str;
    fn severity(&self) -> InvariantSeverity;
}

pub struct InvariantSet {
    checkers: Vec<Box<dyn InvariantChecker>>,
}
```

### Consequences

**Positive**:
- Flexible invariant system
- Easy to test
- Domain extensibility

**Negative**:
- Dynamic dispatch overhead (acceptable for non-hot-path)

---

## ADR-007: Weaver Schema Generation from Σ

**Date**: 2025-11-16
**Status**: Accepted

### Context

KNHK uses Weaver for OTEL validation. Weaver schemas must evolve with ontology. Options:
1. Manual schema maintenance
2. Generate schemas from Σ
3. Dual-source (ontology + schema)

### Decision

**Generate Weaver schemas from Σ snapshots**.

### Rationale

**Single Source of Truth**:
- Ontology is source of truth
- Weaver schema is derived
- No synchronization issues

**Deterministic**:
- Same snapshot_id → same Weaver schema
- Reproducible
- Version-controlled

**Validation**:
- Weaver validates runtime telemetry
- Telemetry must match Σ
- False positive elimination

**Implementation**:

```rust
pub fn generate_weaver_schema(snapshot: &SigmaSnapshot) -> Result<String> {
    let spans = extract_spans(&snapshot.triples)?;
    let yaml = format_weaver_yaml(spans)?;
    Ok(yaml)
}
```

**Workflow**:
1. Ontology snapshot promoted
2. Generate Weaver YAML
3. Commit to registry/
4. Weaver validates runtime

### Consequences

**Positive**:
- Single source of truth
- Automatic schema evolution
- Weaver validation proves correctness

**Negative**:
- Code generation dependency

---

## ADR-008: Sector-Based Mutability

**Date**: 2025-11-16
**Status**: Accepted

### Context

Not all ontology elements should be mutable. Core standards (RDF, RDFS, OWL) must be stable. Domain ontologies should be evolvable.

### Decision

Partition ontology into **sectors**:
- **Σ_core**: Untouchable kernel (RDF, RDFS, OWL, SHACL, KNHK kernel)
- **Σ_ext**: Mutable extensions (domain ontologies)

### Rationale

**Stability**:
- Core standards don't change
- Breaking changes prevented
- Type system stable

**Flexibility**:
- Domain ontologies evolve
- Custom patterns added
- Application-specific schemas

**Validation**:
- SHACL rules prevent core modifications
- Extension changes validated
- Compatibility enforced

**Implementation**:

```turtle
meta:belongsToSector a rdf:Property ;
    rdfs:range meta:Sector .

knhk:HotPath a rdfs:Class ;
    meta:belongsToSector meta:CoreSector .

myapp:CustomEntity a rdfs:Class ;
    meta:belongsToSector meta:ExtensionSector .
```

### Consequences

**Positive**:
- Core stability
- Extension flexibility
- Clear boundaries

**Negative**:
- Sector management overhead

---

## ADR-009: Performance Regression Testing

**Date**: 2025-11-16
**Status**: Accepted

### Context

Ontology changes must not degrade performance. Hot path operations must remain ≤8 ticks.

### Decision

Include **performance regression testing** in validation pipeline.

### Rationale

**SLO Preservation**:
- Hot path SLO: ≤8 ticks
- Warm path SLO: ≤500ms
- Cold path: Best effort

**Early Detection**:
- Detect regressions before promotion
- Prevent production issues
- Maintain performance guarantees

**Baseline**:
- Current snapshot performance = baseline
- New snapshot must meet or beat baseline
- 10% tolerance for noise

**Implementation**:

```rust
pub struct PerformanceBoundsInvariant {
    baseline: PerformanceBaseline,
}

impl InvariantChecker for PerformanceBoundsInvariant {
    fn check(&self, graph: &Graph) -> Result<InvariantReport> {
        let results = run_performance_tests(graph)?;
        // Compare to baseline...
    }
}
```

### Consequences

**Positive**:
- Performance preserved
- Regressions caught early
- SLO compliance

**Negative**:
- Longer validation time (~seconds)

---

## ADR-010: Cryptographic Receipts for Audit Trail

**Date**: 2025-11-16
**Status**: Accepted

### Context

Ontology evolution must be auditable. Who promoted what snapshot when?

### Decision

Generate **cryptographic receipts** for all snapshot promotions.

### Rationale

**Audit Trail**:
- Complete history of changes
- Who, what, when
- Tamper-evident

**Integrity**:
- ed25519 signatures
- Merkle-linked chain
- Verification without re-execution

**Compliance**:
- Enterprise audit requirements
- Regulatory compliance
- Forensic analysis

**Implementation**:

```rust
pub struct SigmaReceipt {
    receipt_id: [u8; 32],
    snapshot_id: SnapshotId,
    parent_receipt: Option<[u8; 32]>,
    timestamp: i64,
    validator: String,
    validation: ValidationReport,
    signature: Vec<u8>,  // ed25519
    merkle_root: [u8; 32],
}
```

### Consequences

**Positive**:
- Complete audit trail
- Tamper-evident
- Compliance-ready

**Negative**:
- Signing overhead (~1ms)
- Storage for signatures

---

## Summary of Key Decisions

| ADR | Decision | Rationale | Trade-off |
|-----|----------|-----------|-----------|
| ADR-001 | Four-plane architecture | Separation of concerns | Complexity |
| ADR-002 | Snapshot-based versioning | Atomic transitions | Storage |
| ADR-003 | Meta-ontology (Σ²) | Self-describing | Bootstrap |
| ADR-004 | Atomic pointer swap | ~1ns promotion | 128-bit ID limit |
| ADR-005 | Overlays for staging | Lightweight experimentation | Overlay GC |
| ADR-006 | Hard invariants as traits | Extensibility | Dynamic dispatch |
| ADR-007 | Weaver schema generation | Single source of truth | Code generation |
| ADR-008 | Sector-based mutability | Core stability | Sector management |
| ADR-009 | Performance regression tests | SLO preservation | Validation time |
| ADR-010 | Cryptographic receipts | Audit trail | Signing overhead |

---

## Decision Review Process

### When to Revisit

1. **Performance Issues**: If snapshot operations exceed budgets
2. **Scale Issues**: If snapshot count exceeds 10K
3. **Security Issues**: If signature scheme compromised
4. **Usability Issues**: If workflow too complex

### Review Triggers

- [ ] After Phase 1 implementation (core infrastructure)
- [ ] After Phase 3 implementation (change management)
- [ ] After 6 months of production use
- [ ] After security audit

### Success Metrics

1. **Performance**: Promotion time ≤1ns, validation ≤100ms
2. **Reliability**: Zero failed promotions in production
3. **Usability**: ≤5 commands for typical workflow
4. **Security**: Zero unauthorized promotions

---

## Open Questions

### Q1: Snapshot Garbage Collection

**Question**: How do we manage snapshot proliferation (10K+ snapshots)?

**Options**:
1. Manual deletion (risky)
2. LRU eviction (automatic)
3. Archival to cold storage (S3)

**Recommendation**: Phase 6 - Implement archival to S3 for snapshots older than 30 days

---

### Q2: Multi-Region Ontology State

**Question**: How do we replicate ontology state across regions?

**Options**:
1. Single region (simple)
2. Active-passive replication
3. Multi-master with consensus

**Recommendation**: Phase 6 - Start with active-passive, evaluate multi-master if needed

---

### Q3: Breaking Change Migration

**Question**: How do we handle breaking ontology changes?

**Options**:
1. Prevent breaking changes (too restrictive)
2. Require manual migration scripts
3. Automated migration via transformation rules

**Recommendation**: Phase 4 - Implement transformation rules in meta-ontology

---

### Q4: LLM-Proposed Changes

**Question**: How do we safely integrate LLM-proposed ontology changes?

**Options**:
1. Human review required (slow)
2. Automatic approval if validated (risky)
3. Staged rollout with monitoring

**Recommendation**: Phase 5 - Implement staged rollout with automatic rollback

---

**Document Status**: ✅ Complete
**Next Review**: After Phase 1 implementation
**Maintained By**: System Architecture Team
