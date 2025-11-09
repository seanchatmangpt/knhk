# YAWL Ontology Integration Architecture

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Work In Progress
**Target:** knhk-workflow-engine v2.0

## Executive Summary

This document defines the integration architecture between the YAWL ontology (RDF/OWL) and the knhk-workflow-engine (Rust). The integration uses Oxigraph as the RDF triplestore, SPARQL for semantic queries, and schema-first validation via OpenTelemetry Weaver.

**Key Integration Points:**
1. **Parser:** TTL → Oxigraph → SPARQL → WorkflowSpec (Rust)
2. **Validator:** SPARQL validation + Weaver schema validation
3. **Runtime:** Pattern execution with RDF state tracking
4. **Provenance:** Git commit hashes stored in RDF

## 1. Current knhk Architecture

### 1.1 Component Overview

```
┌─────────────────────────────────────────────────────┐
│              knhk-workflow-engine                   │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────┐    ┌──────────────┐              │
│  │   Parser     │───▶│  WorkflowSpec│              │
│  │ (TTL→Rust)   │    │   (Rust)     │              │
│  └──────────────┘    └──────────────┘              │
│         │                    │                      │
│         ▼                    ▼                      │
│  ┌──────────────┐    ┌──────────────┐              │
│  │  Validator   │    │   Executor   │              │
│  │ (Deadlock)   │    │  (43 Patterns)│              │
│  └──────────────┘    └──────────────┘              │
│                             │                       │
│                             ▼                       │
│                      ┌──────────────┐              │
│                      │ StateManager │              │
│                      │ (Event Source)│              │
│                      └──────────────┘              │
└─────────────────────────────────────────────────────┘
```

### 1.2 Current File Structure

**Parser (`src/parser/mod.rs`):**
- `WorkflowParser` - Main parser using Oxigraph
- `extract_workflow_spec()` - Extract from RDF store
- `parse_turtle()` - Parse TTL string
- `load_yawl_ontology()` - Load ontology into store

**Types (`src/parser/types.rs`):**
- `WorkflowSpec` - Rust workflow specification
- `Task` - Rust task representation
- `Condition` - Rust condition representation
- `WorkflowSpecId` - UUID identifier

**Extractor (`src/parser/extractor.rs`):**
- `extract_tasks()` - SPARQL query for tasks
- `extract_conditions()` - SPARQL query for conditions
- `extract_flows()` - SPARQL query for flows
- `find_start_condition()` - SPARQL query for start
- `find_end_condition()` - SPARQL query for end

**Executor (`src/executor/mod.rs`):**
- `WorkflowEngine` - Main execution engine
- Pattern execution handlers
- Case management
- Task execution
- Resource allocation

**State (`src/state/mod.rs`):**
- `StateManager` - Event-sourced state
- `StateStore` - Persistent state storage

**Patterns (`src/patterns/mod.rs`):**
- 43 Van der Aalst patterns
- `PatternRegistry` - Pattern executor registry
- Pattern execution context

## 2. Integration Architecture

### 2.1 Overall Integration Flow

```
┌─────────────────────────────────────────────────────┐
│              YAWL Ontology (yawl.ttl)               │
│  Classes: Specification, Net, Task, Condition, etc. │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 1. Parse TTL
                  ▼
┌─────────────────────────────────────────────────────┐
│         Oxigraph RDF Triplestore                    │
│  Store: In-memory or persistent RocksDB             │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 2. SPARQL Extraction
                  ▼
┌─────────────────────────────────────────────────────┐
│      WorkflowParser (src/parser/extractor.rs)       │
│  SPARQL → Extract tasks, conditions, flows          │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 3. Map to Rust Types
                  ▼
┌─────────────────────────────────────────────────────┐
│      WorkflowSpec (src/parser/types.rs)             │
│  Rust: WorkflowSpec, Task, Condition                │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 4. Semantic Validation
                  ▼
┌─────────────────────────────────────────────────────┐
│      SPARQL Validator + Weaver Validator            │
│  Check: soundness, data flow, deadlocks             │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 5. Runtime Execution
                  ▼
┌─────────────────────────────────────────────────────┐
│      WorkflowEngine (src/executor/mod.rs)           │
│  Execute: 43 patterns, resource allocation          │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 6. State Persistence
                  ▼
┌─────────────────────────────────────────────────────┐
│      RDF State Store + Lockchain Provenance         │
│  Store: Case state as RDF, Git provenance           │
└─────────────────────────────────────────────────────┘
```

### 2.2 Data Flow Example

**Input:** YAWL workflow in Turtle format
```turtle
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix ex: <http://example.org/workflow#> .

ex:MyWorkflow a yawl:Specification ;
    rdfs:label "Purchase Order Workflow" ;
    yawl:hasDecomposition ex:MainNet .

ex:MainNet a yawl:Net ;
    yawl:isRootNet true ;
    yawl:hasInputCondition ex:Start ;
    yawl:hasOutputCondition ex:End ;
    yawl:hasTask ex:TaskA, ex:TaskB .

ex:TaskA a yawl:Task ;
    rdfs:label "Approve Order" ;
    yawl:hasJoin yawl:ControlTypeXor ;
    yawl:hasSplit yawl:ControlTypeAnd .
```

**Step 1: Parser loads into Oxigraph**
```rust
let mut parser = WorkflowParser::new()?;
parser.load_yawl_ontology(Path::new("ontology/yawl.ttl"))?;
parser.parse_file(Path::new("workflows/purchase-order.ttl"))?;
```

**Step 2: SPARQL extraction**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
SELECT ?task ?name ?join ?split WHERE {
    ?task a yawl:Task .
    ?task rdfs:label ?name .
    ?task yawl:hasJoin ?join .
    ?task yawl:hasSplit ?split .
}
```

**Step 3: Map to Rust**
```rust
Task {
    id: "http://example.org/workflow#TaskA",
    name: "Approve Order",
    join_type: JoinType::Xor,
    split_type: SplitType::And,
    // ...
}
```

**Step 4: Validation**
```sparql
# Check: Start condition has no incoming flows
ASK {
    ?condition a yawl:InputCondition .
    ?flow yawl:nextElementRef ?condition .
}
# Returns false (valid) or true (invalid)
```

**Step 5: Execution**
```rust
let case_id = engine.create_case(spec_id, data)?;
engine.start_case(case_id)?;
// Pattern execution, resource allocation
```

**Step 6: State persistence**
```turtle
ex:Case123 a yawl:WorkflowInstance ;
    yawl:hasSpecification ex:MyWorkflow ;
    yawl:hasState "running" ;
    yawl:hasCurrentTask ex:TaskA ;
    knhk:hasProvenanceChain "abc123def456..." .
```

## 3. Component Integration Details

### 3.1 Parser Integration

**File:** `src/parser/mod.rs`

**Current Implementation:**
```rust
pub struct WorkflowParser {
    store: Store,  // Oxigraph RDF store
    deadlock_detector: DeadlockDetector,
}

impl WorkflowParser {
    pub fn parse_turtle(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // 1. Load TTL into Oxigraph
        self.store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())?;

        // 2. Extract workflow spec
        let spec = extractor::extract_workflow_spec(&self.store)?;

        // 3. Validate for deadlocks
        self.deadlock_detector.validate(&spec)?;

        Ok(spec)
    }
}
```

**Enhancement Opportunities:**
1. **Schema Validation:** Add SHACL validation before extraction
2. **Incremental Loading:** Support loading multiple TTL files (import)
3. **Caching:** Cache parsed specs for reuse
4. **Streaming:** Support large workflows (pagination)

### 3.2 Extractor Integration

**File:** `src/parser/extractor.rs`

**Current SPARQL Queries:**

**Query 1: Extract Tasks**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
SELECT ?task ?name ?type ?split ?join ?maxTicks ?priority ?simd WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?task yawl:splitType ?split }
    OPTIONAL { ?task yawl:joinType ?join }
    OPTIONAL { ?task yawl:maxTicks ?maxTicks }
    OPTIONAL { ?task yawl:priority ?priority }
    OPTIONAL { ?task yawl:useSimd ?simd }
}
```

**Query 2: Extract Flows**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
SELECT ?from ?to WHERE {
    ?from yawl:hasOutgoingFlow ?to .
}
```

**Enhancement Opportunities:**
1. **Federation:** Query across multiple ontologies
2. **Reasoning:** Use RDFS/OWL reasoning for inference
3. **Validation:** Embed validation in queries (ASK queries)
4. **Performance:** Use prepared queries (compile once)

### 3.3 Validation Integration

**Current:** Deadlock detection only

**Enhanced Validation Architecture:**
```
┌─────────────────────────────────────────────────────┐
│              Validation Pipeline                    │
├─────────────────────────────────────────────────────┤
│  1. Schema Validation (SHACL/SPARQL)               │
│     - Check required properties                     │
│     - Check cardinality constraints                 │
│     - Check datatype ranges                         │
├─────────────────────────────────────────────────────┤
│  2. Semantic Validation (SPARQL)                    │
│     - Check control flow soundness                  │
│     - Check data flow completeness                  │
│     - Check resource allocation validity            │
├─────────────────────────────────────────────────────┤
│  3. Pattern Validation (knhk)                       │
│     - Check pattern compatibility                   │
│     - Check deadlock freedom                        │
│     - Check termination properties                  │
├─────────────────────────────────────────────────────┤
│  4. Weaver Validation (OTEL Schema)                 │
│     - Check telemetry schema compliance             │
│     - Validate span definitions                     │
│     - Validate metric definitions                   │
└─────────────────────────────────────────────────────┘
```

See `semantic-validation-rules.md` for complete validation rules.

### 3.4 Runtime Integration

**Pattern Execution with RDF Context:**

```rust
// Current pattern execution
let result = pattern_registry.execute(&pattern_id, &ctx)?;

// Enhanced with RDF context
pub struct PatternExecutionContext {
    pub case_id: CaseId,
    pub workflow_id: WorkflowSpecId,
    pub variables: HashMap<String, String>,
    pub arrived_from: HashSet<String>,
    pub scope_id: String,
    // NEW: RDF context
    pub rdf_store: Option<Arc<Store>>,  // Reference to RDF store
    pub task_iri: Option<String>,        // Task IRI in ontology
}
```

**State Persistence as RDF:**

```rust
pub trait RdfStateStore {
    /// Save case state as RDF
    fn save_case_state(&mut self, case: &Case) -> WorkflowResult<()>;

    /// Load case state from RDF
    fn load_case_state(&self, case_id: &CaseId) -> WorkflowResult<Option<Case>>;

    /// Query case state with SPARQL
    fn query_cases(&self, sparql: &str) -> WorkflowResult<Vec<Case>>;
}
```

### 3.5 Provenance Integration

**Lockchain + RDF Provenance:**

```turtle
@prefix knhk: <http://knhk.org/ontology#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

ex:Case123 a yawl:WorkflowInstance ;
    prov:wasGeneratedBy ex:Execution456 ;
    knhk:hasProvenanceChain "abc123def456..." ;  # Git commit hash
    prov:startedAtTime "2025-11-08T10:00:00Z"^^xsd:dateTime ;
    prov:wasAssociatedWith ex:Agent789 .

ex:Execution456 a prov:Activity ;
    prov:used ex:MyWorkflow ;
    prov:generated ex:Case123 .
```

## 4. Performance Considerations

### 4.1 Hot Path Optimization

**Constraint:** ≤8 ticks for hot path operations

**Strategy:**
1. **Pre-compile SPARQL queries** - Avoid query parsing overhead
2. **Cache workflow specs** - Load from RDF once, cache in Rust
3. **Lazy RDF loading** - Only load RDF when needed (validation, provenance)
4. **SIMD extraction** - Use SIMD for bulk RDF → Rust conversion

**Hot Path:** Pattern execution should NOT query RDF
```rust
// ❌ WRONG: Query RDF in hot path
fn execute_pattern(ctx: &PatternExecutionContext) -> PatternExecutionResult {
    let query = format!("SELECT ?next WHERE {{ <{}> yawl:flowsInto ?next }}", ctx.task_iri);
    let results = ctx.rdf_store.query(&query)?;  // SLOW!
    // ...
}

// ✅ CORRECT: Pre-load into Rust struct
fn execute_pattern(ctx: &PatternExecutionContext) -> PatternExecutionResult {
    let next_tasks = &ctx.outgoing_flows;  // Already in memory
    // ...
}
```

### 4.2 RDF Storage Options

**Option 1: In-Memory Oxigraph**
- **Pros:** Fast (no disk I/O), simple
- **Cons:** No persistence, limited by RAM
- **Use Case:** Development, testing

**Option 2: RocksDB-backed Oxigraph**
- **Pros:** Persistent, scalable
- **Cons:** Slower than in-memory
- **Use Case:** Production, large workflows

**Option 3: Hybrid**
- **Pros:** Fast reads (cache), persistent writes
- **Cons:** Complex cache invalidation
- **Use Case:** Production with frequent queries

**Recommendation:** Hybrid approach
- Parse-time: Load ontology + spec into RocksDB
- Runtime: Cache WorkflowSpec in Rust
- Provenance: Write case state to RocksDB asynchronously

## 5. Extension Architecture

### 5.1 knhk Ontology Extensions

See `ontology-extension-strategy.md` for:
- `knhk:HotPathTask` - Tasks with ≤8 ticks constraint
- `knhk:hasProvenanceChain` - Git commit provenance
- `knhk:hasSpanTemplate` - OTEL span templates
- `knhk:tickBudget` - Performance constraints

### 5.2 Custom Properties

**Support custom properties in TTL:**
```turtle
@prefix knhk: <http://knhk.org/ontology#> .

ex:TaskA a yawl:Task, knhk:HotPathTask ;
    rdfs:label "Critical Task" ;
    yawl:hasJoin yawl:ControlTypeXor ;
    yawl:hasSplit yawl:ControlTypeAnd ;
    knhk:tickBudget 8 ;
    knhk:hasSpanTemplate "task.execute" .
```

**Extract in Rust:**
```rust
// Add to Task struct
pub struct Task {
    // Standard YAWL fields
    pub id: String,
    pub name: String,
    // ...

    // knhk extensions
    pub max_ticks: Option<u32>,           // From knhk:tickBudget
    pub span_template: Option<String>,     // From knhk:hasSpanTemplate
    pub provenance_required: bool,         // From knhk:requiresProvenance
}
```

## 6. Integration Roadmap

### Phase 1: Foundation (Current)
- ✅ Oxigraph integration
- ✅ SPARQL extraction (tasks, conditions, flows)
- ✅ Deadlock validation
- ✅ Basic pattern execution

### Phase 2: Semantic Validation (Next)
- ⬜ SPARQL validation rules (30+ rules)
- ⬜ SHACL schema validation
- ⬜ Weaver integration (OTEL schema validation)
- ⬜ Validation reporting

### Phase 3: Enhanced Runtime (Future)
- ⬜ RDF state persistence
- ⬜ SPARQL case queries
- ⬜ Provenance tracking (RDF + Lockchain)
- ⬜ Reasoning engine integration

### Phase 4: Optimization (Future)
- ⬜ Query compilation and caching
- ⬜ SIMD RDF extraction
- ⬜ Incremental loading
- ⬜ Federation support

## 7. Open Questions

1. **Ontology Versioning:** How to handle ontology updates without breaking existing workflows?
   - **Option A:** Semantic versioning in namespace URI
   - **Option B:** owl:versionInfo property
   - **Option C:** Separate namespace per version

2. **Schema Evolution:** How to migrate workflows when ontology changes?
   - **Option A:** SPARQL UPDATE migrations
   - **Option B:** RDF-to-RDF transformations
   - **Option C:** Rust-level migration scripts

3. **Performance vs. Semantics:** When to use RDF vs. Rust structs?
   - **Guideline:** RDF for extensibility, Rust for performance
   - **Hot path:** Always use Rust structs
   - **Cold path:** Use RDF for flexibility

4. **Multi-Ontology Support:** How to integrate YAWL with other ontologies?
   - **Option A:** owl:imports for composition
   - **Option B:** Federated SPARQL queries
   - **Option C:** Separate stores with linking

5. **Validation Order:** What order to run validation (SHACL → SPARQL → Weaver)?
   - **Recommendation:** SHACL (schema) → SPARQL (semantics) → Deadlock → Weaver (OTEL)
   - **Rationale:** Fail fast on schema errors, then semantic errors, then runtime errors

## 8. References

- **Oxigraph:** https://github.com/oxigraph/oxigraph
- **SPARQL 1.1:** https://www.w3.org/TR/sparql11-query/
- **SHACL:** https://www.w3.org/TR/shacl/
- **YAWL Foundation:** http://www.yawlfoundation.org/
- **Weaver:** https://github.com/open-telemetry/weaver
- **knhk Parser:** `/Users/sac/knhk/rust/knhk-workflow-engine/src/parser/`
