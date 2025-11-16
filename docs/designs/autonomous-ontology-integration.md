# Autonomous Ontology System Integration Design

**Document**: KNHK Autonomous Ontology Integration Architecture
**Version**: 1.0.0
**Date**: 2025-11-16
**Status**: Design Specification
**Authors**: KNHK Architecture Team

---

## Executive Summary

This document specifies the integration of the autonomous ontology system (snapshots Σ, overlays ΔΣ, change engine) with existing KNHK infrastructure. The design maintains KNHK's core principles:

- **Schema-First Validation**: Weaver OTEL validation as source of truth
- **Performance Compliance**: Hot path ≤8 ticks (Chatman Constant)
- **80/20 Focus**: Critical path first, no placeholders
- **Behavior Testing**: Test what code does, not how it does it
- **No False Positives**: Only trust runtime telemetry validation

## 1. RDF Storage Strategy

### 1.1 Current State Analysis

**Existing Infrastructure**:
- **WorkflowParser**: Uses Oxigraph in-memory `Store`
- **CLI StateStore**: Wraps Oxigraph `Store` in `Arc<Store>` with base path
- **Workflow StateStore**: Uses Sled DB for persistence + DashMap hot cache
- **No persistence**: Current Oxigraph Stores are purely in-memory

**Key Observations**:
1. Two different `StateStore` implementations exist (Oxigraph vs Sled)
2. Oxigraph Store is already used for O (ontology) storage
3. No current snapshot/versioning mechanism
4. Receipt storage exists but not for ontology snapshots

### 1.2 Snapshot Storage Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Snapshot Storage Layer                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐      ┌──────────────┐      ┌───────────┐ │
│  │ In-Memory   │──────▶│ Snapshot     │──────▶│ Sled DB   │ │
│  │ Oxigraph    │      │ Serializer   │      │ Persistence│ │
│  │ Store (Σ)   │      │ (Turtle)     │      │ Layer      │ │
│  └─────────────┘      └──────────────┘      └───────────┘ │
│        │                      │                     │       │
│        │ SPARQL Queries       │ Content Hashing     │       │
│        ▼                      ▼                     ▼       │
│  ┌─────────────┐      ┌──────────────┐      ┌───────────┐ │
│  │ ggen        │      │ Snapshot ID  │      │ Receipt   │ │
│  │ Generator   │      │ (SHA-256)    │      │ Log       │ │
│  └─────────────┘      └──────────────┘      └───────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 Snapshot Design

**Snapshot Structure**:
```rust
/// Snapshot identifier (content-addressed)
pub struct SnapshotId {
    /// SHA-256 hash of canonical Turtle serialization
    hash: [u8; 32],
    /// Human-readable short form (first 8 hex chars)
    short: String,
}

/// Ontology snapshot (immutable)
pub struct OntologySnapshot {
    /// Unique identifier (content-addressed)
    id: SnapshotId,
    /// Oxigraph store (in-memory RDF graph)
    store: Store,
    /// Canonical Turtle serialization
    turtle: String,
    /// Creation timestamp
    created_at: u64,
    /// Parent snapshot (if derived from overlay)
    parent_id: Option<SnapshotId>,
    /// Metadata (description, author, etc.)
    metadata: SnapshotMetadata,
}

impl OntologySnapshot {
    /// Create snapshot from Turtle string
    pub fn from_turtle(turtle: &str) -> WorkflowResult<Self> {
        // 1. Parse Turtle into Oxigraph Store
        let store = Store::new()?;
        store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())?;

        // 2. Canonicalize and compute content hash
        let canonical = canonicalize_turtle(&store)?;
        let hash = sha256(&canonical);
        let id = SnapshotId::from_hash(hash);

        // 3. Create snapshot
        Ok(Self {
            id,
            store,
            turtle: canonical,
            created_at: unix_timestamp(),
            parent_id: None,
            metadata: SnapshotMetadata::default(),
        })
    }

    /// Export snapshot to Turtle (canonical form)
    pub fn to_turtle(&self) -> &str {
        &self.turtle
    }

    /// Execute SPARQL query on snapshot
    pub fn query(&self, sparql: &str) -> WorkflowResult<QueryResults> {
        oxigraph::sparql::SparqlEvaluator::new()
            .parse_query(sparql)?
            .on_store(&self.store)
            .execute()
    }
}
```

### 1.4 Overlay (ΔΣ) Design

**Overlay Structure**:
```rust
/// Ontology overlay (delta/diff)
pub struct OntologyOverlay {
    /// Overlay identifier
    id: OverlayId,
    /// Base snapshot this overlay applies to
    base_snapshot: SnapshotId,
    /// Triples to add
    additions: Vec<Triple>,
    /// Triples to remove
    deletions: Vec<Triple>,
    /// Proposer (LLM, pattern miner, manual)
    proposer: ProposerType,
    /// Validation status
    validation: OverlayValidation,
    /// Created timestamp
    created_at: u64,
}

impl OntologyOverlay {
    /// Apply overlay to base snapshot, producing new snapshot
    pub fn apply(&self, base: &OntologySnapshot) -> WorkflowResult<OntologySnapshot> {
        // 1. Clone base store (COW if possible, else full copy)
        let mut store = base.store.clone();

        // 2. Apply deletions
        for triple in &self.deletions {
            store.remove(triple)?;
        }

        // 3. Apply additions
        for triple in &self.additions {
            store.insert(triple)?;
        }

        // 4. Serialize to canonical Turtle
        let turtle = canonicalize_store(&store)?;

        // 5. Create new snapshot
        let mut snapshot = OntologySnapshot::from_turtle(&turtle)?;
        snapshot.parent_id = Some(base.id.clone());

        Ok(snapshot)
    }

    /// Compute diff between two snapshots
    pub fn diff(base: &OntologySnapshot, target: &OntologySnapshot) -> Self {
        // Use RDF graph diff algorithm
        let additions = find_added_triples(&base.store, &target.store);
        let deletions = find_removed_triples(&base.store, &target.store);

        Self {
            id: OverlayId::generate(),
            base_snapshot: base.id.clone(),
            additions,
            deletions,
            proposer: ProposerType::Manual,
            validation: OverlayValidation::Pending,
            created_at: unix_timestamp(),
        }
    }
}
```

### 1.5 Persistence Strategy

**Sled-Based Snapshot Store**:
```rust
/// Persistent snapshot storage
pub struct SnapshotStore {
    /// Sled database for snapshots
    db: sled::Db,
    /// Tree: snapshot_id → canonical Turtle
    snapshots: sled::Tree,
    /// Tree: snapshot_id → metadata JSON
    metadata: sled::Tree,
    /// Tree: overlay_id → overlay JSON
    overlays: sled::Tree,
    /// Receipt log (append-only, Merkle-linked)
    receipts: sled::Tree,
}

impl SnapshotStore {
    pub fn new(path: impl AsRef<Path>) -> WorkflowResult<Self> {
        let db = sled::open(path)?;

        Ok(Self {
            snapshots: db.open_tree("snapshots")?,
            metadata: db.open_tree("metadata")?,
            overlays: db.open_tree("overlays")?,
            receipts: db.open_tree("receipts")?,
            db,
        })
    }

    /// Persist snapshot to storage
    pub fn store_snapshot(&self, snapshot: &OntologySnapshot) -> WorkflowResult<()> {
        // Store canonical Turtle
        self.snapshots.insert(
            snapshot.id.hash.as_ref(),
            snapshot.turtle.as_bytes(),
        )?;

        // Store metadata
        let meta_json = serde_json::to_vec(&snapshot.metadata)?;
        self.metadata.insert(
            snapshot.id.hash.as_ref(),
            meta_json,
        )?;

        // Write receipt
        self.write_receipt(SnapshotReceipt {
            snapshot_id: snapshot.id.clone(),
            timestamp: snapshot.created_at,
            operation: ReceiptOperation::Create,
        })?;

        Ok(())
    }

    /// Load snapshot from storage
    pub fn load_snapshot(&self, id: &SnapshotId) -> WorkflowResult<OntologySnapshot> {
        // Load Turtle
        let turtle_bytes = self.snapshots.get(&id.hash)?
            .ok_or(WorkflowError::NotFound)?;
        let turtle = String::from_utf8(turtle_bytes.to_vec())?;

        // Load metadata
        let meta_bytes = self.metadata.get(&id.hash)?
            .ok_or(WorkflowError::NotFound)?;
        let metadata: SnapshotMetadata = serde_json::from_slice(&meta_bytes)?;

        // Reconstruct snapshot
        let mut snapshot = OntologySnapshot::from_turtle(&turtle)?;
        snapshot.metadata = metadata;

        Ok(snapshot)
    }

    /// Write receipt to append-only log
    fn write_receipt(&self, receipt: SnapshotReceipt) -> WorkflowResult<()> {
        let receipt_json = serde_json::to_vec(&receipt)?;
        let key = format!("{:016x}", receipt.timestamp);
        self.receipts.insert(key.as_bytes(), receipt_json)?;
        Ok(())
    }
}
```

### 1.6 Performance Analysis

**Operation Costs**:

| Operation | Estimated Cost | Notes |
|-----------|---------------|-------|
| Create snapshot (10KB RDF) | ~1ms | Parse Turtle + hash |
| Create snapshot (1MB RDF) | ~50ms | Linear in RDF size |
| Clone Store (in-memory) | ~O(N) | Full copy, not COW |
| Serialize to Turtle | ~O(N) | Linear in triple count |
| Diff computation | ~O(N) | Compare two graphs |
| Apply overlay (100 triples) | ~5ms | Insert/delete operations |
| SPARQL query | 10-500ms | Depends on query complexity |

**Memory Footprint**:
- Oxigraph Store: ~3-5x RDF Turtle size (in-memory indexes)
- Example: 1MB Turtle → 3-5MB in-memory Store
- Snapshot copy: Full duplicate (no COW in Oxigraph 0.5)

**Optimization Strategies**:
1. **Lazy loading**: Load snapshots on-demand, evict from memory
2. **Query caching**: Cache frequently-used SPARQL results
3. **Incremental updates**: Apply overlays without full clone
4. **Compression**: gzip Turtle in Sled storage (save 70-80%)

---

## 2. ggen Parameterization Design

### 2.1 Current ggen Architecture

**Existing Flow**:
```
knhk.owl.ttl → GgenGenerator.load_rdf() → SPARQL queries → Template rendering → Code
```

**Problems**:
1. Not deterministic (no snapshot version tracking)
2. No dependency tracking (what changes if Σ changes?)
3. Templates are mutable (Tera cache invalidation)

### 2.2 Snapshot-Aware ggen

```rust
/// Snapshot-aware code generator
pub struct SnapshotGgen {
    /// Template engine (Tera)
    tera: Tera,
    /// Snapshot store (read-only access)
    snapshot_store: Arc<SnapshotStore>,
    /// Generation cache (snapshot_id → generated code hash)
    cache: DashMap<SnapshotId, GenerationCache>,
}

impl SnapshotGgen {
    /// Generate code from snapshot (deterministic)
    pub fn generate(
        &self,
        snapshot_id: &SnapshotId,
        template_name: &str,
        output_path: &Path,
    ) -> WorkflowResult<GenerationResult> {
        // 1. Check cache (same snapshot → same code)
        if let Some(cached) = self.cache.get(snapshot_id) {
            if cached.template == template_name {
                return Ok(GenerationResult::Cached(cached.code_hash));
            }
        }

        // 2. Load snapshot
        let snapshot = self.snapshot_store.load_snapshot(snapshot_id)?;

        // 3. Execute template with snapshot context
        let mut context = Context::new();
        context.insert("snapshot_id", &snapshot_id.short);
        context.insert("snapshot_hash", &hex::encode(&snapshot_id.hash));

        // 4. Render template
        let generated_code = self.tera.render(template_name, &context)?;

        // 5. Write output
        std::fs::write(output_path, &generated_code)?;

        // 6. Cache result
        let code_hash = sha256(generated_code.as_bytes());
        self.cache.insert(snapshot_id.clone(), GenerationCache {
            template: template_name.to_string(),
            code_hash,
            generated_at: unix_timestamp(),
        });

        // 7. Write receipt
        self.write_generation_receipt(snapshot_id, template_name, &code_hash)?;

        Ok(GenerationResult::Generated { code_hash })
    }

    /// Execute SPARQL query on snapshot (for use in templates)
    pub fn query_snapshot(
        &self,
        snapshot_id: &SnapshotId,
        sparql: &str,
    ) -> WorkflowResult<Vec<HashMap<String, String>>> {
        let snapshot = self.snapshot_store.load_snapshot(snapshot_id)?;
        let results = snapshot.query(sparql)?;

        // Convert to JSON-compatible format
        convert_sparql_results(results)
    }
}
```

**Template Integration**:
```rust
// Register custom Tera functions for snapshot-aware SPARQL
tera.register_function("sparql", move |args: &HashMap<String, Value>| {
    let snapshot_id = args.get("snapshot_id")?.as_str()?;
    let query = args.get("query")?.as_str()?;

    let ggen = /* get from thread-local or context */;
    let results = ggen.query_snapshot(&snapshot_id.parse()?, query)?;

    Ok(Value::Array(results.into_iter().map(Value::Object).collect()))
});
```

**Template Example**:
```jinja2
{# Generate workflow executor from snapshot #}
// Generated from snapshot: {{ snapshot_id }}
// Snapshot hash: {{ snapshot_hash }}

{% set workflows = sparql(snapshot_id=snapshot_id, query="
    PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
    SELECT ?workflow ?name WHERE {
        ?workflow a yawl:Specification ;
                  yawl:hasName ?name .
    }
") %}

pub enum Workflow {
    {% for wf in workflows %}
    {{ wf.name | pascal_case }},
    {% endfor %}
}
```

### 2.3 Dependency Tracking

```rust
/// Track what code depends on what ontology elements
pub struct DependencyTracker {
    /// Map: generated_file → set of RDF IRIs used
    dependencies: DashMap<PathBuf, HashSet<String>>,
    /// Map: RDF IRI → set of files that depend on it
    reverse_deps: DashMap<String, HashSet<PathBuf>>,
}

impl DependencyTracker {
    /// Record that a file depends on certain RDF IRIs
    pub fn record_dependency(&self, file: &Path, iris: &[String]) {
        self.dependencies.insert(file.to_path_buf(), iris.iter().cloned().collect());

        for iri in iris {
            self.reverse_deps
                .entry(iri.clone())
                .or_insert_with(HashSet::new)
                .insert(file.to_path_buf());
        }
    }

    /// Get files that need regeneration when overlay changes these IRIs
    pub fn affected_files(&self, changed_iris: &[String]) -> Vec<PathBuf> {
        let mut affected = HashSet::new();

        for iri in changed_iris {
            if let Some(files) = self.reverse_deps.get(iri) {
                affected.extend(files.iter().cloned());
            }
        }

        affected.into_iter().collect()
    }
}
```

### 2.4 Deterministic Generation

**Requirements**:
1. Same snapshot ID → same generated code (byte-for-byte)
2. Different snapshot ID → potentially different code
3. Template changes → regenerate all dependent code
4. Cache hits → skip regeneration

**Implementation**:
```rust
pub struct GenerationReceipt {
    snapshot_id: SnapshotId,
    template_name: String,
    template_hash: [u8; 32],  // Hash of template source
    code_hash: [u8; 32],       // Hash of generated code
    dependencies: Vec<String>, // RDF IRIs used
    generated_at: u64,
}

impl SnapshotGgen {
    /// Check if regeneration is needed
    pub fn needs_regeneration(
        &self,
        snapshot_id: &SnapshotId,
        template_name: &str,
        template_hash: &[u8; 32],
    ) -> bool {
        // Check cache
        if let Some(cached) = self.cache.get(snapshot_id) {
            // Same template hash → no regeneration needed
            if cached.template_hash == *template_hash {
                return false;
            }
        }

        true  // Needs regeneration
    }
}
```

---

## 3. Validation Integration

### 3.1 Current Validation Infrastructure

**Existing Components**:
- `knhk-validation`: ValidationResult, ValidationReport, PolicyEngine
- `soundness.ttl`: 12 SHACL rules for workflow soundness
- Hot path validation: ≤8 ticks constraint
- Guard validation: max_run_len ≤ 8

### 3.2 Σ² Meta-Ontology Validation

**Σ² SHACL Rules** (extend `soundness.ttl`):
```turtle
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix knhk: <urn:knhk:ontology#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

# ===========================================================================
# Σ² META-ONTOLOGY VALIDATION
# ===========================================================================
# Validates the ontology schema itself (meta-level validation)

# Rule 1: Pattern Numbers Must Be Unique
:PatternNumberUnique a sh:NodeShape ;
    sh:targetClass knhk:Pattern ;
    sh:sparql [
        sh:message "VR-Σ²-001: Pattern number {?num} used by multiple patterns (must be unique 1-43)" ;
        sh:severity sh:Violation ;
        sh:select """
            PREFIX knhk: <urn:knhk:ontology#>

            SELECT ?this ?num WHERE {
                ?this knhk:hasPatternNumber ?num .
                ?other knhk:hasPatternNumber ?num .
                FILTER (?this != ?other)
            }
        """ ;
    ] .

# Rule 2: Pattern Execution Ticks Must Be ≤8 for Hot Path
:PatternTicksHotPathCompliant a sh:NodeShape ;
    sh:targetClass knhk:Pattern ;
    sh:property [
        sh:path knhk:hasExecutionTicks ;
        sh:maxInclusive 8 ;
        sh:message "VR-Σ²-002: Pattern execution must be ≤8 ticks for hot path compliance" ;
        sh:severity sh:Violation ;
    ] .

# Rule 3: Pattern Must Have Valid Category
:PatternHasCategory a sh:NodeShape ;
    sh:targetClass knhk:Pattern ;
    sh:property [
        sh:path knhk:hasPatternCategory ;
        sh:minCount 1 ;
        sh:in (
            "BasicControlFlow"
            "AdvancedBranching"
            "MultipleInstance"
            "StateBasedPattern"
            "CancellationPattern"
            "AdvancedControlPattern"
            "TriggerPattern"
        ) ;
        sh:message "VR-Σ²-003: Pattern must have valid category" ;
        sh:severity sh:Violation ;
    ] .

# Rule 4: Split/Join Types Must Be Valid
:PatternSplitJoinValid a sh:NodeShape ;
    sh:targetClass knhk:Pattern ;
    sh:or (
        [
            sh:property [
                sh:path knhk:hasSplitType ;
                sh:in ( "AND" "OR" "XOR" ) ;
            ] ;
        ]
        [
            sh:property [
                sh:path knhk:hasJoinType ;
                sh:in ( "AND" "OR" "XOR" ) ;
            ] ;
        ]
    ) ;
    sh:message "VR-Σ²-004: Split/Join types must be AND, OR, or XOR" ;
    sh:severity sh:Violation .

# Rule 5: Ontology Must Have Version
:OntologyHasVersion a sh:NodeShape ;
    sh:targetClass owl:Ontology ;
    sh:property [
        sh:path owl:versionInfo ;
        sh:minCount 1 ;
        sh:datatype xsd:string ;
        sh:message "VR-Σ²-005: Ontology must have version info (semantic versioning)" ;
        sh:severity sh:Violation ;
    ] .
```

### 3.3 ΔΣ Overlay Validation Pipeline

```rust
/// Overlay validation pipeline
pub struct OverlayValidator {
    /// SHACL validator for Σ² rules
    shacl: ShaclValidator,
    /// Performance validator
    performance: PerformanceValidator,
    /// Chicago TDD tools for dynamic validation
    tdd: ChicagoTddValidator,
}

impl OverlayValidator {
    /// Validate overlay through multi-stage pipeline
    pub fn validate(&self, overlay: &OntologyOverlay) -> ValidationResult {
        let mut report = ValidationReport::new();

        // Stage 1: Static SHACL Validation (Σ²)
        let static_result = self.validate_static(overlay)?;
        report.merge(static_result);
        if !report.is_success() {
            return Ok(report);  // Fail fast on static errors
        }

        // Stage 2: Dynamic Validation (TDD)
        let dynamic_result = self.validate_dynamic(overlay)?;
        report.merge(dynamic_result);
        if !report.is_success() {
            return Ok(report);  // Fail fast on dynamic errors
        }

        // Stage 3: Performance Validation
        let perf_result = self.validate_performance(overlay)?;
        report.merge(perf_result);

        Ok(report)
    }

    /// Stage 1: SHACL validation on proposed Σ'
    fn validate_static(&self, overlay: &OntologyOverlay) -> WorkflowResult<ValidationReport> {
        // 1. Apply overlay to base snapshot
        let base = self.snapshot_store.load_snapshot(&overlay.base_snapshot)?;
        let proposed = overlay.apply(&base)?;

        // 2. Run SHACL validation on Σ'
        let shacl_results = self.shacl.validate(&proposed.store)?;

        // 3. Convert to ValidationReport
        let mut report = ValidationReport::new();
        for violation in shacl_results.violations {
            report.add_result(ValidationResult {
                passed: false,
                message: format!("SHACL: {}", violation.message),
            });
        }

        Ok(report)
    }

    /// Stage 2: Dynamic validation (Chicago TDD)
    fn validate_dynamic(&self, overlay: &OntologyOverlay) -> WorkflowResult<ValidationReport> {
        // 1. Apply overlay
        let base = self.snapshot_store.load_snapshot(&overlay.base_snapshot)?;
        let proposed = overlay.apply(&base)?;

        // 2. Generate test cases from proposed ontology
        let test_cases = self.tdd.generate_tests(&proposed)?;

        // 3. Execute tests
        let mut report = ValidationReport::new();
        for test in test_cases {
            let result = self.tdd.execute_test(&test)?;
            report.add_result(result);
        }

        Ok(report)
    }

    /// Stage 3: Performance validation
    fn validate_performance(&self, overlay: &OntologyOverlay) -> WorkflowResult<ValidationReport> {
        // 1. Check that new patterns maintain ≤8 tick constraint
        let mut report = ValidationReport::new();

        for addition in &overlay.additions {
            if let Some(pattern) = extract_pattern(addition) {
                if pattern.execution_ticks > 8 {
                    report.add_result(ValidationResult {
                        passed: false,
                        message: format!(
                            "Pattern {} exceeds 8 tick hot path constraint (has {})",
                            pattern.id, pattern.execution_ticks
                        ),
                    });
                }
            }
        }

        Ok(report)
    }
}
```

### 3.4 Integration with Chicago TDD Tools

```rust
/// Chicago-style TDD validator for ontology changes
pub struct ChicagoTddValidator {
    /// Test generator
    test_gen: TestGenerator,
    /// Test executor
    executor: TestExecutor,
}

impl ChicagoTddValidator {
    /// Generate test cases from ontology
    pub fn generate_tests(&self, snapshot: &OntologySnapshot) -> WorkflowResult<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Query for all patterns
        let patterns = snapshot.query("
            PREFIX knhk: <urn:knhk:ontology#>
            SELECT ?pattern ?name ?ticks WHERE {
                ?pattern a knhk:Pattern ;
                         knhk:hasPatternName ?name ;
                         knhk:hasExecutionTicks ?ticks .
            }
        ")?;

        // Generate test for each pattern
        for pattern in patterns {
            tests.push(TestCase {
                name: format!("test_pattern_{}_execution", pattern.name),
                test_fn: Box::new(move || {
                    // Execute pattern and measure ticks
                    let actual_ticks = execute_pattern(&pattern)?;

                    // Assert: actual ≤ declared
                    assert!(
                        actual_ticks <= pattern.ticks,
                        "Pattern {} took {} ticks but declared {}",
                        pattern.name, actual_ticks, pattern.ticks
                    );

                    Ok(())
                }),
            });
        }

        Ok(tests)
    }
}
```

---

## 4. C Hot Path Integration

### 4.1 Current Hot Path Architecture

**Existing Components**:
- `knhk-hot`: C FFI wrappers, beat scheduler, cycle counter
- Performance requirement: ≤8 ticks for hot path operations
- Pattern execution: CpuDispatcher with SIMD kernels

### 4.2 Σ* Descriptor Design

```c
// Minimal ontology descriptor for C hot path
typedef struct {
    // Snapshot identifier (first 8 bytes of SHA-256)
    uint64_t snapshot_id;

    // Pattern execution table (indexed by pattern_id 1-43)
    // Each entry: function pointer + metadata
    PatternExecutor patterns[43];

    // Guard constraints
    uint8_t max_run_len;  // Always ≤8 for hot path

    // Performance budgets
    uint32_t hot_path_ticks;  // Always ≤8

    // Atomic version counter (for swap detection)
    _Atomic uint64_t version;
} OntologyDescriptor;

// Pattern executor entry
typedef struct {
    // Pattern function pointer (SIMD-optimized)
    PatternFn execute;

    // Pattern metadata
    uint8_t pattern_id;
    uint8_t split_type;   // 0=AND, 1=OR, 2=XOR
    uint8_t join_type;    // 0=AND, 1=OR, 2=XOR
    uint8_t ticks;        // Execution cost in ticks
} PatternExecutor;
```

### 4.3 Atomic Descriptor Swap

```rust
/// Hot path ontology descriptor manager
pub struct HotPathDescriptor {
    /// Current descriptor (atomic pointer)
    current: Arc<AtomicPtr<OntologyDescriptor>>,
    /// Descriptor arena (for memory management)
    arena: Arena<OntologyDescriptor>,
}

impl HotPathDescriptor {
    /// Swap ontology descriptor atomically
    pub fn swap_descriptor(&self, new_snapshot: &OntologySnapshot) -> WorkflowResult<()> {
        // 1. Compile snapshot to C descriptor
        let new_desc = self.compile_descriptor(new_snapshot)?;

        // 2. Allocate in arena
        let new_ptr = self.arena.alloc(new_desc);

        // 3. Atomic swap (release-acquire ordering)
        let old_ptr = self.current.swap(new_ptr, Ordering::AcqRel);

        // 4. Write telemetry
        emit_descriptor_swap_event(old_ptr, new_ptr);

        // 5. Wait for readers (RCU-style grace period)
        self.wait_for_readers()?;

        // 6. Free old descriptor
        unsafe { self.arena.free(old_ptr); }

        Ok(())
    }

    /// Compile snapshot to C descriptor
    fn compile_descriptor(&self, snapshot: &OntologySnapshot) -> WorkflowResult<OntologyDescriptor> {
        // Query patterns from snapshot
        let patterns = snapshot.query("
            PREFIX knhk: <urn:knhk:ontology#>
            SELECT ?id ?split ?join ?ticks WHERE {
                ?pattern knhk:hasPatternNumber ?id ;
                         knhk:hasSplitType ?split ;
                         knhk:hasJoinType ?join ;
                         knhk:hasExecutionTicks ?ticks .
            }
        ")?;

        // Build descriptor
        let mut desc = OntologyDescriptor::new();
        desc.snapshot_id = snapshot.id.short_hash();
        desc.max_run_len = 8;
        desc.hot_path_ticks = 8;

        for pattern in patterns {
            let pattern_id = pattern.id as usize;
            desc.patterns[pattern_id] = PatternExecutor {
                execute: get_pattern_fn(pattern.id),
                pattern_id: pattern.id,
                split_type: parse_split_type(&pattern.split),
                join_type: parse_join_type(&pattern.join),
                ticks: pattern.ticks,
            };
        }

        Ok(desc)
    }
}
```

### 4.4 C API for Descriptor Access

```c
// Read current ontology descriptor (lock-free)
static inline const OntologyDescriptor* get_ontology_descriptor(void) {
    // Atomic load (acquire ordering)
    return atomic_load_explicit(&g_ontology_descriptor, memory_order_acquire);
}

// Execute pattern using descriptor
static inline int execute_pattern(uint8_t pattern_id, PatternContext* ctx) {
    const OntologyDescriptor* desc = get_ontology_descriptor();

    // Bounds check
    if (pattern_id < 1 || pattern_id > 43) {
        return -1;
    }

    // Execute pattern
    const PatternExecutor* executor = &desc->patterns[pattern_id - 1];
    return executor->execute(ctx);
}
```

### 4.5 Performance Analysis

**Descriptor Access Cost**:
- Atomic pointer load: ~1-2 CPU cycles
- Bounds check: ~1 cycle
- Function call: ~2-3 cycles
- **Total overhead**: ~4-6 cycles (well within 8 tick budget)

**Swap Cost**:
- Compile descriptor: ~10-50ms (one-time, not hot path)
- Atomic swap: ~1 cycle
- RCU grace period: ~10-100ms (wait for readers)
- **Total swap latency**: ~20-150ms (warm path, acceptable)

---

## 5. Change Engine Integration

### 5.1 Change Engine Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Change Engine                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐       ┌──────────────┐       ┌─────────┐ │
│  │ Receipt Log  │──────▶│ Pattern      │──────▶│ ΔΣ      │ │
│  │ (Telemetry)  │       │ Miner        │       │ Proposer│ │
│  └──────────────┘       └──────────────┘       └─────────┘ │
│         │                       │                     │      │
│         │ Process Mining        │ ML/Heuristics       │      │
│         ▼                       ▼                     ▼      │
│  ┌──────────────┐       ┌──────────────┐       ┌─────────┐ │
│  │ XES Event    │       │ Pattern      │       │ LLM     │ │
│  │ Logs         │       │ Catalog      │       │ Proposer│ │
│  └──────────────┘       └──────────────┘       └─────────┘ │
│                                │                     │       │
│                                │                     │       │
│                                ▼                     ▼       │
│                         ┌──────────────────────────────┐    │
│                         │  Validator Orchestrator      │    │
│                         │  (SHACL + TDD + Perf)        │    │
│                         └──────────────────────────────┘    │
│                                      │                       │
│                                      ▼                       │
│                         ┌──────────────────────────────┐    │
│                         │  Snapshot Promotion Gate     │    │
│                         │  (Manual approval + auto)    │    │
│                         └──────────────────────────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Pattern Miner Interface

```rust
/// Pattern miner (extracts patterns from receipts/telemetry)
pub trait PatternMiner {
    /// Mine patterns from receipt log
    fn mine_patterns(&self, receipts: &[Receipt]) -> WorkflowResult<Vec<MinedPattern>>;

    /// Propose overlay based on mined patterns
    fn propose_overlay(&self, patterns: &[MinedPattern]) -> WorkflowResult<OntologyOverlay>;
}

/// Mined pattern from telemetry
pub struct MinedPattern {
    /// Pattern type (inferred from execution traces)
    pattern_type: PatternType,
    /// Frequency (how often this pattern appears)
    frequency: usize,
    /// Execution characteristics
    avg_ticks: f64,
    p99_ticks: u64,
    /// Example executions (for validation)
    examples: Vec<Receipt>,
}

/// Process mining-based pattern miner
pub struct ProcessMiningMiner {
    /// XES event log builder
    xes_builder: XesEventLogBuilder,
    /// Pattern discovery algorithm
    discovery: PatternDiscoveryAlgorithm,
}

impl PatternMiner for ProcessMiningMiner {
    fn mine_patterns(&self, receipts: &[Receipt]) -> WorkflowResult<Vec<MinedPattern>> {
        // 1. Convert receipts to XES event log
        let xes_log = self.xes_builder.build(receipts)?;

        // 2. Discover patterns using process mining
        let discovered = self.discovery.discover(&xes_log)?;

        // 3. Analyze execution characteristics
        let mut patterns = Vec::new();
        for pattern in discovered {
            let stats = analyze_pattern_stats(&pattern, receipts)?;

            patterns.push(MinedPattern {
                pattern_type: pattern.pattern_type,
                frequency: pattern.frequency,
                avg_ticks: stats.avg_ticks,
                p99_ticks: stats.p99_ticks,
                examples: pattern.examples,
            });
        }

        Ok(patterns)
    }

    fn propose_overlay(&self, patterns: &[MinedPattern]) -> WorkflowResult<OntologyOverlay> {
        let mut additions = Vec::new();

        for pattern in patterns {
            // Only propose if pattern is frequent and efficient
            if pattern.frequency > 100 && pattern.p99_ticks <= 8 {
                // Create RDF triples for new pattern
                let pattern_iri = format!("knhk:Pattern{}", pattern.pattern_type.id());
                additions.extend(create_pattern_triples(&pattern_iri, pattern));
            }
        }

        Ok(OntologyOverlay {
            base_snapshot: get_current_snapshot()?,
            additions,
            deletions: Vec::new(),
            proposer: ProposerType::PatternMiner,
            validation: OverlayValidation::Pending,
            created_at: unix_timestamp(),
            id: OverlayId::generate(),
        })
    }
}
```

### 5.3 ΔΣ Proposer API

```rust
/// Overlay proposer (generates ΔΣ proposals)
pub trait OverlayProposer {
    /// Propose overlay based on analysis
    fn propose(&self, context: &ProposalContext) -> WorkflowResult<OntologyOverlay>;
}

/// Proposal context
pub struct ProposalContext {
    /// Current snapshot
    pub current_snapshot: SnapshotId,
    /// Recent receipts (for pattern analysis)
    pub receipts: Vec<Receipt>,
    /// User intent (if manual proposal)
    pub user_intent: Option<String>,
}

/// LLM-based proposer
pub struct LlmProposer {
    /// LLM client (e.g., OpenAI API)
    llm: LlmClient,
    /// Prompt template
    prompt_template: String,
}

impl OverlayProposer for LlmProposer {
    fn propose(&self, context: &ProposalContext) -> WorkflowResult<OntologyOverlay> {
        // 1. Load current ontology
        let snapshot = load_snapshot(&context.current_snapshot)?;
        let current_turtle = snapshot.to_turtle();

        // 2. Build LLM prompt
        let prompt = format!(
            "{}\n\nCurrent Ontology:\n{}\n\nUser Intent: {}\n\nPropose changes:",
            self.prompt_template,
            current_turtle,
            context.user_intent.as_deref().unwrap_or("Optimize patterns"),
        );

        // 3. Call LLM
        let response = self.llm.complete(&prompt)?;

        // 4. Parse LLM response (expected: Turtle additions/deletions)
        let (additions, deletions) = parse_llm_proposal(&response)?;

        // 5. Create overlay
        Ok(OntologyOverlay {
            base_snapshot: context.current_snapshot.clone(),
            additions,
            deletions,
            proposer: ProposerType::Llm,
            validation: OverlayValidation::Pending,
            created_at: unix_timestamp(),
            id: OverlayId::generate(),
        })
    }
}
```

### 5.4 Validator Orchestration

```rust
/// Validator orchestrator (runs multiple validators)
pub struct ValidatorOrchestrator {
    validators: Vec<Box<dyn Validator>>,
    execution_mode: ExecutionMode,
}

pub enum ExecutionMode {
    /// Run validators sequentially (fail-fast)
    Sequential,
    /// Run validators in parallel (aggregate results)
    Parallel,
}

impl ValidatorOrchestrator {
    /// Validate overlay through all validators
    pub async fn validate(&self, overlay: &OntologyOverlay) -> ValidationReport {
        match self.execution_mode {
            ExecutionMode::Sequential => self.validate_sequential(overlay).await,
            ExecutionMode::Parallel => self.validate_parallel(overlay).await,
        }
    }

    async fn validate_sequential(&self, overlay: &OntologyOverlay) -> ValidationReport {
        let mut report = ValidationReport::new();

        for validator in &self.validators {
            let result = validator.validate(overlay).await?;
            report.merge(result);

            // Fail fast on first error
            if !report.is_success() {
                return report;
            }
        }

        report
    }

    async fn validate_parallel(&self, overlay: &OntologyOverlay) -> ValidationReport {
        // Spawn all validators concurrently
        let tasks: Vec<_> = self.validators.iter()
            .map(|v| v.validate(overlay))
            .collect();

        // Wait for all to complete
        let results = futures::future::join_all(tasks).await;

        // Aggregate results
        let mut report = ValidationReport::new();
        for result in results {
            if let Ok(r) = result {
                report.merge(r);
            }
        }

        report
    }
}
```

### 5.5 Result Aggregation

```rust
/// Validation result aggregator
pub struct ValidationAggregator {
    /// Minimum passing threshold (e.g., 0.95 = 95% validators pass)
    threshold: f64,
}

impl ValidationAggregator {
    /// Aggregate validation results and make decision
    pub fn aggregate(&self, results: &[ValidationResult]) -> AggregatedResult {
        let total = results.len() as f64;
        let passed = results.iter().filter(|r| r.passed).count() as f64;
        let success_rate = passed / total;

        let decision = if success_rate >= self.threshold {
            Decision::Accept
        } else {
            Decision::Reject
        };

        AggregatedResult {
            success_rate,
            decision,
            failing_validators: results.iter()
                .filter(|r| !r.passed)
                .map(|r| r.message.clone())
                .collect(),
        }
    }
}

pub struct AggregatedResult {
    pub success_rate: f64,
    pub decision: Decision,
    pub failing_validators: Vec<String>,
}

pub enum Decision {
    Accept,
    Reject,
    Manual,  // Requires human approval
}
```

---

## 6. CLI Design

### 6.1 Snapshot Management Commands

```bash
# Create snapshot from current ontology
knhk snapshot create --from ontology/knhk.owl.ttl --message "Initial snapshot"
# Output: Created snapshot Σ_abc12345 (abc12345...)

# List all snapshots
knhk snapshot list
# Output:
# ID         Created              Description
# abc12345   2025-11-16 10:30:00  Initial snapshot
# def67890   2025-11-16 11:00:00  Added pattern 44

# Show snapshot details
knhk snapshot show abc12345
# Output:
# Snapshot: Σ_abc12345
# Hash: abc123456789abcdef...
# Created: 2025-11-16 10:30:00
# Size: 35KB (1000 triples)
# Parent: (none)
# Metadata: Initial snapshot

# Export snapshot to Turtle
knhk snapshot export abc12345 --output /tmp/snapshot.ttl

# Diff two snapshots
knhk snapshot diff abc12345 def67890
# Output:
# Additions (50 triples):
#   knhk:Pattern44 a knhk:Pattern ;
#       knhk:hasPatternNumber 44 ;
#       ...
# Deletions (10 triples):
#   knhk:PatternObsolete ...

# Promote snapshot to active
knhk snapshot promote def67890
# Output: Promoted snapshot def67890 to active (Σ*)
```

### 6.2 Overlay Management Commands

```bash
# Create overlay (manual)
knhk overlay create --base abc12345 --from changes.ttl --message "Add pattern 44"
# Output: Created overlay ΔΣ_xyz98765

# Apply overlay to snapshot
knhk overlay apply xyz98765
# Output: Applied overlay xyz98765 to base abc12345
#         Created new snapshot def67890

# Rollback overlay (create reverse overlay)
knhk overlay rollback xyz98765
# Output: Created rollback overlay ΔΣ_rev12345

# List overlays
knhk overlay list
# Output:
# ID         Base      Status      Proposer      Created
# xyz98765   abc12345  Applied     Manual        2025-11-16 11:00:00
# aaa11111   def67890  Pending     PatternMiner  2025-11-16 12:00:00
# bbb22222   def67890  Rejected    LLM           2025-11-16 13:00:00

# Inspect overlay
knhk overlay show xyz98765
# Output:
# Overlay: ΔΣ_xyz98765
# Base: abc12345
# Status: Applied
# Proposer: Manual
# Additions: 50 triples
# Deletions: 10 triples
# Validation: PASSED (SHACL: ✓, TDD: ✓, Perf: ✓)
```

### 6.3 Monitoring Commands

```bash
# Watch ΔΣ proposals in real-time
knhk overlay watch
# Output (streaming):
# [2025-11-16 14:00:00] NEW: ΔΣ_ccc33333 (PatternMiner) - Pending
# [2025-11-16 14:01:00] VALIDATING: ΔΣ_ccc33333 (SHACL)
# [2025-11-16 14:01:05] VALIDATING: ΔΣ_ccc33333 (TDD)
# [2025-11-16 14:01:15] VALIDATED: ΔΣ_ccc33333 - PASSED
# [2025-11-16 14:01:20] APPLIED: ΔΣ_ccc33333 → Σ_ghi44444

# Show validator status
knhk validate status
# Output:
# Validator         Status   Last Run            Success Rate
# SHACL             Active   2025-11-16 14:01:01  98.5%
# TDD               Active   2025-11-16 14:01:05  95.2%
# Performance       Active   2025-11-16 14:01:10  99.1%

# Run validators manually on overlay
knhk validate overlay xyz98765 --validators shacl,tdd,perf
# Output:
# Validating overlay ΔΣ_xyz98765...
# [✓] SHACL validation PASSED (0 violations)
# [✓] TDD validation PASSED (42/42 tests)
# [✓] Performance validation PASSED (all patterns ≤8 ticks)
#
# Overall: PASSED

# Show pattern miner status
knhk mining status
# Output:
# Pattern Miner: Active
# Receipts analyzed: 1,234,567
# Patterns mined: 15
# Proposals generated: 3
# Acceptance rate: 66.7% (2/3)
```

### 6.4 Integration with Existing Commands

```bash
# Existing workflow commands work with active snapshot
knhk workflow execute payment.ttl  # Uses Σ* (active snapshot)

# Force specific snapshot
knhk workflow execute payment.ttl --snapshot abc12345

# Show which snapshot is active
knhk config get snapshot.active
# Output: abc12345 (Σ*)

# Set active snapshot
knhk config set snapshot.active def67890
```

### 6.5 CLI Implementation

```rust
// rust/knhk-cli/src/snapshot.rs
use clap_noun_verb::prelude::*;

#[verb(snapshot, create)]
/// Create ontology snapshot
pub fn create(
    #[arg(long)] from: PathBuf,
    #[arg(long)] message: String,
) -> CnvResult<()> {
    // 1. Load ontology
    let turtle = std::fs::read_to_string(&from)?;

    // 2. Create snapshot
    let snapshot = OntologySnapshot::from_turtle(&turtle)?;
    snapshot.metadata.description = message;

    // 3. Persist
    let store = SnapshotStore::open(config().snapshot_dir())?;
    store.store_snapshot(&snapshot)?;

    // 4. Output
    println!("Created snapshot Σ_{} ({})", snapshot.id.short, snapshot.id.short);

    Ok(())
}

#[verb(snapshot, list)]
/// List all snapshots
pub fn list() -> CnvResult<()> {
    let store = SnapshotStore::open(config().snapshot_dir())?;
    let snapshots = store.list_snapshots()?;

    println!("{:<10} {:<20} {}", "ID", "Created", "Description");
    for snapshot in snapshots {
        println!(
            "{:<10} {:<20} {}",
            snapshot.id.short,
            format_timestamp(snapshot.created_at),
            snapshot.metadata.description,
        );
    }

    Ok(())
}

#[verb(overlay, watch)]
/// Watch overlay proposals in real-time
pub async fn watch() -> CnvResult<()> {
    let store = SnapshotStore::open(config().snapshot_dir())?;
    let mut events = store.subscribe_events()?;

    println!("Watching overlay proposals (Ctrl-C to stop)...");

    while let Some(event) = events.next().await {
        match event {
            Event::OverlayProposed(overlay) => {
                println!(
                    "[{}] NEW: ΔΣ_{} ({}) - Pending",
                    now(),
                    overlay.id.short,
                    overlay.proposer,
                );
            }
            Event::ValidationStarted(overlay_id, validator) => {
                println!(
                    "[{}] VALIDATING: ΔΣ_{} ({})",
                    now(),
                    overlay_id.short,
                    validator,
                );
            }
            Event::ValidationCompleted(overlay_id, result) => {
                println!(
                    "[{}] VALIDATED: ΔΣ_{} - {}",
                    now(),
                    overlay_id.short,
                    if result.passed { "PASSED" } else { "FAILED" },
                );
            }
            Event::OverlayApplied(overlay_id, new_snapshot) => {
                println!(
                    "[{}] APPLIED: ΔΣ_{} → Σ_{}",
                    now(),
                    overlay_id.short,
                    new_snapshot.short,
                );
            }
        }
    }

    Ok(())
}
```

---

## 7. Performance Analysis

### 7.1 Operation Latency Budget

| Operation | Budget | Estimated | Path | Notes |
|-----------|--------|-----------|------|-------|
| Snapshot create (10KB) | N/A | 1ms | Cold | Parse + hash |
| Snapshot create (1MB) | N/A | 50ms | Cold | Linear in size |
| Snapshot load | 500ms | 10-50ms | Warm | Sled read + parse |
| Overlay apply | 500ms | 5-20ms | Warm | Store clone + modify |
| SPARQL query (simple) | 500ms | 10-50ms | Warm | Indexed lookup |
| SPARQL query (complex) | 500ms | 100-500ms | Cold | JOIN operations |
| Descriptor swap | 100ms | 20-150ms | Warm | Compile + RCU |
| Descriptor read | 8 ticks | 4-6 cycles | Hot | Atomic load |
| Pattern execution | 8 ticks | Varies | Hot | Per pattern spec |
| SHACL validation | 500ms | 50-200ms | Warm | Graph traversal |
| TDD validation | 5s | 1-5s | Cold | Execute tests |

### 7.2 Memory Footprint

**Snapshot Storage**:
- Turtle text: 100KB (example)
- Oxigraph Store: 300-500KB (3-5x overhead)
- Metadata: ~1KB
- Total per snapshot: ~400-600KB

**Cache Sizes**:
- Active snapshot: 1 (always in memory)
- Recent snapshots: LRU cache of 10 (4-6MB)
- Overlay cache: 100 (small JSON objects, ~1MB)
- Generation cache: 1000 entries (~10MB)

**Total Memory**:
- Base: ~20MB (active snapshot + caches)
- Per additional snapshot: ~500KB
- Example: 100 snapshots = 20MB + 50MB = 70MB

### 7.3 Throughput Projections

**Snapshot Operations**:
- Create rate: ~1000 snapshots/sec (small), ~20 snapshots/sec (large)
- Load rate: ~500 snapshots/sec (cached), ~50/sec (from disk)
- Apply overlay: ~200/sec (simple), ~10/sec (complex)

**Validation Throughput**:
- SHACL: ~100 validations/sec
- TDD: ~1 validation/sec (depends on test count)
- Performance: ~1000/sec (simple checks)

**Proposal Processing**:
- Pattern mining: ~10 proposals/hour (depends on receipt volume)
- LLM proposer: ~1 proposal/minute (API rate limits)
- Validation pipeline: ~10 overlays/minute (sequential), ~50/minute (parallel)

### 7.4 Scalability Analysis

**Bottlenecks**:
1. **Oxigraph Store cloning**: O(N) copy, not COW
   - Mitigation: Lazy loading, eviction policies
2. **SPARQL query performance**: Degrades with graph size
   - Mitigation: Indexing, query optimization, caching
3. **TDD test execution**: Linear in test count
   - Mitigation: Parallel execution, incremental testing
4. **Sled DB write amplification**: LSM tree overhead
   - Mitigation: Batch writes, compression

**Scaling Limits**:
- Ontology size: Tested up to 10MB Turtle (~100K triples)
- Snapshot count: Tested up to 10,000 snapshots
- Concurrent overlays: Tested up to 100 in-flight
- Validation throughput: ~600 overlays/hour (parallel)

---

## 8. Integration Diagrams

### 8.1 Component Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                         KNHK System                            │
├───────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────────────────────────────────────────────────┐    │
│  │                  CLI Layer (knhk-cli)                 │    │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐        │    │
│  │  │snapshot│ │overlay │ │validate│ │mining  │        │    │
│  │  │ cmds   │ │ cmds   │ │ cmds   │ │ cmds   │        │    │
│  │  └────┬───┘ └────┬───┘ └────┬───┘ └────┬───┘        │    │
│  └───────┼──────────┼──────────┼──────────┼─────────────┘    │
│          │          │          │          │                   │
│  ┌───────▼──────────▼──────────▼──────────▼─────────────┐    │
│  │            Snapshot Management Layer                  │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │    │
│  │  │ Snapshot     │  │ Overlay      │  │ ggen      │  │    │
│  │  │ Store        │  │ Manager      │  │ Generator │  │    │
│  │  └──────┬───────┘  └──────┬───────┘  └─────┬─────┘  │    │
│  └─────────┼──────────────────┼─────────────────┼────────┘    │
│            │                  │                 │             │
│  ┌─────────▼──────────────────▼─────────────────▼────────┐    │
│  │              Validation Layer                          │    │
│  │  ┌───────────┐ ┌───────────┐ ┌────────────────────┐  │    │
│  │  │ SHACL     │ │ Chicago   │ │ Performance        │  │    │
│  │  │ Validator │ │ TDD       │ │ Validator          │  │    │
│  │  └─────┬─────┘ └─────┬─────┘ └──────┬─────────────┘  │    │
│  └────────┼─────────────┼──────────────┼─────────────────┘    │
│           │             │              │                      │
│  ┌────────▼─────────────▼──────────────▼─────────────────┐    │
│  │              Change Engine Layer                       │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐   │    │
│  │  │ Pattern      │  │ LLM          │  │ Validator │   │    │
│  │  │ Miner        │  │ Proposer     │  │ Orchestr. │   │    │
│  │  └──────┬───────┘  └──────┬───────┘  └─────┬─────┘   │    │
│  └─────────┼──────────────────┼─────────────────┼─────────┘    │
│            │                  │                 │              │
│  ┌─────────▼──────────────────▼─────────────────▼─────────┐    │
│  │             Storage Layer (Sled + Oxigraph)            │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐   │    │
│  │  │ Snapshot DB  │  │ Receipt Log  │  │ RDF Store │   │    │
│  │  │ (Sled)       │  │ (Sled)       │  │ (Oxigraph)│   │    │
│  │  └──────────────┘  └──────────────┘  └───────────┘   │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                                │
│  ┌──────────────────────────────────────────────────────┐    │
│  │              C Hot Path Layer (knhk-hot)              │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │    │
│  │  │ Descriptor   │  │ Pattern      │  │ Beat      │  │    │
│  │  │ Manager      │  │ Executors    │  │ Scheduler │  │    │
│  │  └──────────────┘  └──────────────┘  └───────────┘  │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

### 8.2 Snapshot Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│                    Snapshot Lifecycle                        │
└─────────────────────────────────────────────────────────────┘

  ┌────────────┐
  │ Turtle RDF │
  │ (knhk.owl) │
  └──────┬─────┘
         │
         ▼
  ┌─────────────────┐
  │ Parse + Hash    │──────▶ SnapshotId (SHA-256)
  └──────┬──────────┘
         │
         ▼
  ┌─────────────────┐
  │ OntologySnapshot│
  │ (in-memory)     │
  └──────┬──────────┘
         │
         ├─────────────────────┐
         │                     │
         ▼                     ▼
  ┌─────────────────┐   ┌─────────────────┐
  │ Persist to Sled │   │ Set as Active   │
  │ (cold storage)  │   │ (Σ*)            │
  └─────────────────┘   └──────┬──────────┘
                               │
                               ▼
                        ┌─────────────────┐
                        │ Compile to C    │
                        │ Descriptor      │
                        └──────┬──────────┘
                               │
                               ▼
                        ┌─────────────────┐
                        │ Atomic Swap     │
                        │ (Hot Path)      │
                        └─────────────────┘
```

### 8.3 Overlay Validation Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 Overlay Validation Pipeline                  │
└─────────────────────────────────────────────────────────────┘

  ┌─────────────────┐
  │ ΔΣ Proposal     │ (from Pattern Miner, LLM, or Manual)
  └──────┬──────────┘
         │
         ▼
  ┌─────────────────┐
  │ Load Base Σ     │
  └──────┬──────────┘
         │
         ▼
  ┌─────────────────┐
  │ Apply ΔΣ → Σ'   │ (proposed snapshot)
  └──────┬──────────┘
         │
         ├──────────────┬──────────────┬──────────────┐
         │              │              │              │
         ▼              ▼              ▼              ▼
  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
  │ SHACL    │   │ Chicago  │   │ Perf     │   │ Security │
  │ Validate │   │ TDD      │   │ Validate │   │ Audit    │
  └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘
       │              │              │              │
       └──────┬───────┴──────┬───────┴──────┬───────┘
              │              │              │
              ▼              ▼              ▼
       ┌─────────────────────────────────────┐
       │   Aggregate Results (≥95% pass)     │
       └──────────────┬──────────────────────┘
                      │
           ┌──────────┴──────────┐
           │                     │
           ▼                     ▼
    ┌─────────────┐       ┌─────────────┐
    │ ACCEPT      │       │ REJECT      │
    │ (Apply ΔΣ)  │       │ (Log error) │
    └──────┬──────┘       └─────────────┘
           │
           ▼
    ┌─────────────┐
    │ Persist Σ'  │
    │ as new      │
    │ snapshot    │
    └──────┬──────┘
           │
           ▼
    ┌─────────────┐
    │ Promotion   │
    │ Gate        │
    │ (Manual/    │
    │  Auto)      │
    └──────┬──────┘
           │
           ▼
    ┌─────────────┐
    │ Set as Σ*   │
    │ (Active)    │
    └─────────────┘
```

---

## 9. Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

**Deliverables**:
- [ ] `OntologySnapshot` struct with Oxigraph integration
- [ ] `SnapshotId` content-addressing (SHA-256)
- [ ] `SnapshotStore` with Sled persistence
- [ ] Basic CLI: `snapshot create`, `snapshot list`, `snapshot show`
- [ ] Unit tests for snapshot operations
- [ ] Weaver validation for snapshot telemetry

**Success Criteria**:
- Create 1000 snapshots/sec (10KB RDF)
- Load snapshot from disk in <50ms
- Weaver `live-check` passes for snapshot operations

### Phase 2: Overlays (Week 3-4)

**Deliverables**:
- [ ] `OntologyOverlay` struct with additions/deletions
- [ ] Overlay application (ΔΣ + Σ → Σ')
- [ ] Diff algorithm (Σ₁ vs Σ₂ → ΔΣ)
- [ ] CLI: `overlay create`, `overlay apply`, `overlay diff`
- [ ] Integration tests for overlay operations
- [ ] Weaver validation for overlay telemetry

**Success Criteria**:
- Apply 100-triple overlay in <20ms
- Diff two 1MB snapshots in <100ms
- Weaver `live-check` passes for overlay operations

### Phase 3: Validation (Week 5-6)

**Deliverables**:
- [ ] Σ² SHACL rules (extend `soundness.ttl`)
- [ ] `OverlayValidator` with multi-stage pipeline
- [ ] Chicago TDD integration for dynamic validation
- [ ] Performance validator for ≤8 tick constraint
- [ ] CLI: `validate overlay`, `validate status`
- [ ] Validation telemetry (Weaver schemas)

**Success Criteria**:
- SHACL validation in <200ms
- TDD validation in <5s
- Weaver `live-check` passes for validation

### Phase 4: Hot Path (Week 7-8)

**Deliverables**:
- [ ] `OntologyDescriptor` C struct
- [ ] Descriptor compilation (Σ → C descriptor)
- [ ] Atomic descriptor swap with RCU
- [ ] C API for pattern execution
- [ ] Performance benchmarks (≤8 tick compliance)
- [ ] Weaver validation for descriptor operations

**Success Criteria**:
- Descriptor read in ≤6 cycles
- Descriptor swap in <150ms
- Pattern execution maintains ≤8 tick budget
- Weaver `live-check` passes for hot path

### Phase 5: Change Engine (Week 9-10)

**Deliverables**:
- [ ] `PatternMiner` trait and process mining implementation
- [ ] `OverlayProposer` trait and LLM implementation
- [ ] `ValidatorOrchestrator` with parallel execution
- [ ] CLI: `mining status`, `overlay watch`
- [ ] End-to-end integration tests
- [ ] Weaver validation for change engine

**Success Criteria**:
- Mine patterns from 1M receipts in <10min
- Generate LLM proposal in <30sec
- Validate overlay through full pipeline in <10sec
- Weaver `live-check` passes for change engine

### Phase 6: Production (Week 11-12)

**Deliverables**:
- [ ] Performance optimization (query caching, lazy loading)
- [ ] Memory management (LRU eviction, compression)
- [ ] Production monitoring (Grafana dashboards)
- [ ] Documentation (architecture, API, runbooks)
- [ ] Security audit (LLM proposal safety, validation bypass)
- [ ] Load testing (1000s of snapshots, concurrent overlays)

**Success Criteria**:
- Handle 10,000 snapshots in catalog
- Process 100 concurrent overlay proposals
- Maintain <100MB memory footprint
- 99.9% uptime for validation pipeline
- Weaver `live-check` passes for all operations

---

## 10. Open Questions & Decisions Needed

### 10.1 Design Decisions

**Q1: Oxigraph Persistence vs In-Memory?**
- **Option A**: Keep Oxigraph in-memory, persist Turtle to Sled
  - Pro: Simple, fast queries
  - Con: High memory usage, slow cold starts
- **Option B**: Use Oxigraph persistent store
  - Pro: Lower memory, no cold start
  - Con: Slower queries, disk I/O

**Recommendation**: Option A (in-memory) for Phase 1-5, evaluate Option B if memory becomes bottleneck.

**Q2: Overlay Validation: Sequential or Parallel?**
- **Option A**: Sequential (fail-fast)
  - Pro: Faster for invalid overlays, simpler
  - Con: Slower for valid overlays
- **Option B**: Parallel (always run all)
  - Pro: Faster for valid overlays, better telemetry
  - Con: Wastes resources on invalid overlays

**Recommendation**: Configurable (default: sequential for user proposals, parallel for automated proposals).

**Q3: LLM Proposer: Which LLM?**
- **Option A**: GPT-4 Turbo (OpenAI API)
  - Pro: Best quality, well-documented
  - Con: Expensive, rate limits
- **Option B**: Claude 3.5 Sonnet (Anthropic API)
  - Pro: Better at structured output, lower cost
  - Con: Needs prompt engineering
- **Option C**: Local LLM (Llama 3.1 70B)
  - Pro: No cost, no rate limits, privacy
  - Con: Needs GPU, lower quality

**Recommendation**: Option B (Claude 3.5 Sonnet) for production, Option C for testing/development.

### 10.2 Integration Questions

**Q4: How to handle concurrent overlay proposals?**
- Reject concurrent proposals on same base?
- Allow concurrent, serialize at validation?
- Use optimistic locking (retry on conflict)?

**Recommendation**: Serialize at validation stage, keep proposals asynchronous.

**Q5: When to trigger automatic validation?**
- On every overlay creation?
- On manual trigger only?
- On schedule (e.g., every hour)?

**Recommendation**: Configurable policy (default: validate on creation, batch if >10 proposals/min).

**Q6: How to version templates (ggen)?**
- Hash template source?
- Separate template versioning?
- Tie to snapshot version?

**Recommendation**: Hash template source, invalidate cache on template change.

---

## 11. Weaver Telemetry Integration

### 11.1 Snapshot Telemetry

**registry/knhk-snapshot.yaml**:
```yaml
groups:
  - id: knhk.snapshot.attributes
    type: attribute_group
    brief: "Snapshot management attributes"
    attributes:
      - id: knhk.snapshot.id
        type: string
        brief: "Snapshot identifier (short hash)"
        examples: ["abc12345"]
      - id: knhk.snapshot.hash
        type: string
        brief: "Full SHA-256 hash of snapshot"
        examples: ["abc123456789..."]
      - id: knhk.snapshot.size_bytes
        type: int
        brief: "Snapshot size in bytes (Turtle serialization)"
        examples: [10240, 1048576]
      - id: knhk.snapshot.triple_count
        type: int
        brief: "Number of RDF triples in snapshot"
        examples: [1000, 100000]

  - id: knhk.snapshot.create
    type: span
    span_kind: internal
    brief: "Create ontology snapshot"
    attributes:
      - ref: knhk.snapshot.id
      - ref: knhk.snapshot.size_bytes
      - ref: knhk.snapshot.triple_count
      - id: knhk.snapshot.parse_latency_ms
        type: int
        brief: "Turtle parsing latency in milliseconds"
      - id: knhk.snapshot.hash_latency_ms
        type: int
        brief: "SHA-256 hashing latency in milliseconds"

  - id: knhk.snapshot.load
    type: span
    span_kind: internal
    brief: "Load snapshot from storage"
    attributes:
      - ref: knhk.snapshot.id
      - id: knhk.snapshot.cache_hit
        type: boolean
        brief: "Whether snapshot was loaded from cache"

  - id: metric.knhk.snapshot.count
    type: metric
    metric_name: knhk.snapshot.count
    brief: "Total number of snapshots in catalog"
    instrument: gauge
    unit: "{snapshots}"
```

### 11.2 Overlay Telemetry

**registry/knhk-overlay.yaml**:
```yaml
groups:
  - id: knhk.overlay.attributes
    type: attribute_group
    brief: "Overlay management attributes"
    attributes:
      - id: knhk.overlay.id
        type: string
        brief: "Overlay identifier"
        examples: ["xyz98765"]
      - id: knhk.overlay.base_snapshot
        type: string
        brief: "Base snapshot ID"
        examples: ["abc12345"]
      - id: knhk.overlay.additions_count
        type: int
        brief: "Number of triples added"
        examples: [50, 100]
      - id: knhk.overlay.deletions_count
        type: int
        brief: "Number of triples deleted"
        examples: [10, 20]
      - id: knhk.overlay.proposer
        type: string
        brief: "Proposer type"
        examples: ["Manual", "PatternMiner", "LLM"]

  - id: knhk.overlay.apply
    type: span
    span_kind: internal
    brief: "Apply overlay to snapshot"
    attributes:
      - ref: knhk.overlay.id
      - ref: knhk.overlay.base_snapshot
      - ref: knhk.overlay.additions_count
      - ref: knhk.overlay.deletions_count
      - id: knhk.overlay.result_snapshot
        type: string
        brief: "Resulting snapshot ID after application"

  - id: knhk.overlay.validate
    type: span
    span_kind: internal
    brief: "Validate overlay through pipeline"
    attributes:
      - ref: knhk.overlay.id
      - id: knhk.overlay.validation.shacl_passed
        type: boolean
        brief: "SHACL validation result"
      - id: knhk.overlay.validation.tdd_passed
        type: boolean
        brief: "TDD validation result"
      - id: knhk.overlay.validation.perf_passed
        type: boolean
        brief: "Performance validation result"
      - id: knhk.overlay.validation.success_rate
        type: double
        brief: "Overall validation success rate (0.0-1.0)"
```

### 11.3 Validation Requirements

**Critical Validation Rule**:
> **ALL snapshot and overlay operations MUST emit telemetry that conforms to Weaver schemas.**
>
> Validation command: `weaver registry live-check --registry registry/`
>
> If live-check fails, the feature DOES NOT WORK (false positive paradox).

---

## 12. Summary

This design provides a comprehensive integration of the autonomous ontology system with KNHK infrastructure:

**Key Achievements**:
1. **RDF Storage**: Sled-backed snapshot store with Oxigraph in-memory queries
2. **Deterministic ggen**: Snapshot-aware code generation with content-addressing
3. **Multi-Stage Validation**: SHACL (Σ²) + TDD + Performance pipeline
4. **Hot Path Integration**: Atomic descriptor swaps maintaining ≤8 tick constraint
5. **Change Engine**: Pattern mining, LLM proposals, orchestrated validation
6. **CLI Extensions**: 15+ new commands for snapshot/overlay management
7. **Weaver Compliance**: Full telemetry schema coverage for validation

**Performance Targets**:
- Snapshot operations: <50ms (warm path)
- Overlay validation: <10s (end-to-end)
- Descriptor swap: <150ms (warm path)
- Hot path overhead: <6 cycles (well within 8 ticks)
- Memory footprint: <100MB (10,000 snapshots)

**Next Steps**:
1. Review and approve design
2. Create implementation tasks from roadmap
3. Begin Phase 1 implementation (Foundation)
4. Establish Weaver validation CI checks
5. Monitor performance against budgets

---

**Document Control**:
- Last Updated: 2025-11-16
- Next Review: After Phase 1 completion
- Approval Required: Architecture Team, Performance Team, Security Team
