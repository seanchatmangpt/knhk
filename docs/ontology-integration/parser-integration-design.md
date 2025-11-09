# Parser Integration Design: YAWL Ontology → WorkflowParser

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Integration Design
**Target:** knhk-workflow-engine v2.0
**Component:** `src/parser/mod.rs`, `src/parser/extractor.rs`

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current Parser Architecture](#current-parser-architecture)
3. [Ontology Loading Sequence](#ontology-loading-sequence)
4. [SPARQL Extraction Pipeline](#sparql-extraction-pipeline)
5. [Rust Type Construction](#rust-type-construction)
6. [Validation Integration](#validation-integration)
7. [Caching Strategy](#caching-strategy)
8. [Error Handling and Recovery](#error-handling-and-recovery)
9. [Backward Compatibility](#backward-compatibility)
10. [Performance Optimization](#performance-optimization)
11. [Implementation Roadmap](#implementation-roadmap)

---

## 1. Executive Summary

This document defines the **detailed integration architecture** between the YAWL ontology (RDF/Turtle) and the existing `WorkflowParser`. The integration uses **Oxigraph** as the RDF triplestore, **SPARQL** for semantic queries, and **schema-first validation** via OpenTelemetry Weaver.

**Key Integration Points:**
1. **Bootstrap:** Load YAWL ontology + knhk extensions into Oxigraph
2. **Parse:** Load workflow TTL into same Oxigraph store
3. **Extract:** SPARQL queries → Rust structs (WorkflowSpec, Task, Condition)
4. **Validate:** SHACL + SPARQL + Deadlock + Weaver validation
5. **Cache:** In-memory cache of WorkflowSpec for hot path execution
6. **Persist:** Optional RocksDB backend for production

**Critical Performance Constraint:** Parsing is cold-path (no ≤8 tick requirement), but caching ensures hot-path execution never queries RDF.

---

## 2. Current Parser Architecture

### 2.1 Existing Components

**File:** `src/parser/mod.rs`

```rust
pub struct WorkflowParser {
    store: Store,  // Oxigraph RDF store (in-memory or RocksDB)
    deadlock_detector: DeadlockDetector,
}

impl WorkflowParser {
    pub fn new() -> WorkflowResult<Self> {
        let store = Store::new()?;
        Ok(Self { store, deadlock_detector: DeadlockDetector })
    }

    pub fn parse_turtle(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // 1. Load TTL into Oxigraph
        self.store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())?;

        // 2. Extract workflow spec via SPARQL
        let spec = extractor::extract_workflow_spec(&self.store)?;

        // 3. Validate for deadlocks
        self.deadlock_detector.validate(&spec)?;

        Ok(spec)
    }

    pub fn load_yawl_ontology(&mut self, ontology_path: &Path) -> WorkflowResult<()> {
        let contents = std::fs::read_to_string(ontology_path)?;
        self.store.load_from_reader(RdfFormat::Turtle, contents.as_bytes())?;
        Ok(())
    }
}
```

**File:** `src/parser/extractor.rs`

Key functions:
- `extract_workflow_spec()` - Main extraction entry point
- `extract_tasks()` - SPARQL query for yawl:Task instances
- `extract_conditions()` - SPARQL query for yawl:Condition instances
- `extract_flows()` - SPARQL query for yawl:hasOutgoingFlow relationships
- `find_start_condition()` - SPARQL query for yawl:StartCondition
- `find_end_condition()` - SPARQL query for yawl:EndCondition

**File:** `src/parser/types.rs`

Core Rust types:
- `WorkflowSpec` - Top-level workflow specification
- `Task` - Task with join/split types, max_ticks, priority, SIMD, resource requirements
- `Condition` - Workflow condition (input/output)
- `WorkflowSpecId` - UUID identifier

### 2.2 Current Data Flow

```
┌─────────────────────────────────────────────────────┐
│  parse_turtle("workflow.ttl")                       │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 1. Load TTL → Oxigraph Store
                  ▼
┌─────────────────────────────────────────────────────┐
│  Oxigraph Store (in-memory)                         │
│  RDF triples: (subject, predicate, object)          │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 2. SPARQL Extraction
                  ▼
┌─────────────────────────────────────────────────────┐
│  extract_workflow_spec(&store)                      │
│  ├─ extract_tasks()                                 │
│  ├─ extract_conditions()                            │
│  ├─ extract_flows()                                 │
│  ├─ find_start_condition()                          │
│  └─ find_end_condition()                            │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 3. Map to Rust Types
                  ▼
┌─────────────────────────────────────────────────────┐
│  WorkflowSpec {                                     │
│    id: WorkflowSpecId,                              │
│    name: String,                                    │
│    tasks: HashMap<String, Task>,                    │
│    conditions: HashMap<String, Condition>,          │
│    start_condition: Option<String>,                 │
│    end_condition: Option<String>,                   │
│  }                                                  │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ 4. Deadlock Validation
                  ▼
┌─────────────────────────────────────────────────────┐
│  deadlock_detector.validate(&spec)                  │
└─────────────────────────────────────────────────────┘
```

---

## 3. Ontology Loading Sequence

### 3.1 Bootstrap Sequence

The parser must load ontologies in a specific order to ensure proper RDF reasoning and validation.

**Phase 1: Core Ontology Bootstrap**

```rust
pub struct WorkflowParser {
    store: Store,
    deadlock_detector: DeadlockDetector,
    // NEW: Track loaded ontologies
    loaded_ontologies: HashSet<String>,
    // NEW: Validation schemas
    shacl_shapes: Option<Vec<ShapeDefinition>>,
}

impl WorkflowParser {
    /// Create parser with ontology preloading
    pub fn new_with_ontology(ontology_dir: &Path) -> WorkflowResult<Self> {
        let store = Store::new()?;
        let mut parser = Self {
            store,
            deadlock_detector: DeadlockDetector,
            loaded_ontologies: HashSet::new(),
            shacl_shapes: None,
        };

        // Bootstrap sequence
        parser.bootstrap_ontologies(ontology_dir)?;

        Ok(parser)
    }

    /// Bootstrap ontology loading in dependency order
    fn bootstrap_ontologies(&mut self, ontology_dir: &Path) -> WorkflowResult<()> {
        // Step 1: Load YAWL core ontology (foundation)
        self.load_ontology_file(ontology_dir.join("yawl-core.ttl"), "yawl-core")?;

        // Step 2: Load YAWL patterns ontology (extends core)
        self.load_ontology_file(ontology_dir.join("yawl-patterns.ttl"), "yawl-patterns")?;

        // Step 3: Load knhk extensions (performance, provenance, OTEL)
        self.load_ontology_file(ontology_dir.join("knhk-extensions.ttl"), "knhk-extensions")?;

        // Step 4: Load SHACL validation shapes
        self.load_shacl_shapes(ontology_dir.join("yawl-shacl.ttl"))?;

        // Step 5: Verify ontology integrity
        self.verify_ontology_integrity()?;

        Ok(())
    }

    /// Load individual ontology file with tracking
    fn load_ontology_file(&mut self, path: PathBuf, name: &str) -> WorkflowResult<()> {
        if self.loaded_ontologies.contains(name) {
            return Ok(()); // Already loaded
        }

        let contents = std::fs::read_to_string(&path)
            .map_err(|e| WorkflowError::Parse(format!("Failed to read {}: {}", name, e)))?;

        self.store
            .load_from_reader(RdfFormat::Turtle, contents.as_bytes())
            .map_err(|e| WorkflowError::Parse(format!("Failed to parse {}: {:?}", name, e)))?;

        self.loaded_ontologies.insert(name.to_string());

        Ok(())
    }

    /// Load SHACL shapes for validation
    fn load_shacl_shapes(&mut self, path: PathBuf) -> WorkflowResult<()> {
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| WorkflowError::Parse(format!("Failed to read SHACL shapes: {}", e)))?;

        // Parse SHACL shapes (will be used for validation)
        self.store
            .load_from_reader(RdfFormat::Turtle, contents.as_bytes())
            .map_err(|e| WorkflowError::Parse(format!("Failed to parse SHACL: {:?}", e)))?;

        // Extract SHACL shapes into Rust structs for validation
        self.shacl_shapes = Some(self.extract_shacl_shapes()?);

        Ok(())
    }

    /// Verify ontology integrity (basic sanity checks)
    fn verify_ontology_integrity(&self) -> WorkflowResult<()> {
        // Check 1: Verify YAWL core classes exist
        let query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            ASK {
                yawl:WorkflowSpecification a owl:Class .
                yawl:Task a owl:Class .
                yawl:Condition a owl:Class .
            }
        "#;

        if !self.ask_query(query)? {
            return Err(WorkflowError::Parse(
                "YAWL core classes not found in ontology".into()
            ));
        }

        // Check 2: Verify knhk extensions exist
        let query = r#"
            PREFIX knhk: <http://knhk.org/ontology#>
            ASK {
                knhk:HotPathTask a owl:Class .
                knhk:tickBudget a owl:DatatypeProperty .
            }
        "#;

        if !self.ask_query(query)? {
            return Err(WorkflowError::Parse(
                "knhk extensions not found in ontology".into()
            ));
        }

        Ok(())
    }

    /// Execute SPARQL ASK query
    fn ask_query(&self, query: &str) -> WorkflowResult<bool> {
        #[allow(deprecated)]
        let result = self.store
            .query(query)
            .map_err(|e| WorkflowError::Parse(format!("ASK query failed: {:?}", e)))?;

        match result {
            oxigraph::sparql::QueryResults::Boolean(b) => Ok(b),
            _ => Err(WorkflowError::Parse("Expected boolean result".into())),
        }
    }
}
```

### 3.2 Incremental Ontology Loading

Support loading multiple ontology files (e.g., domain-specific extensions).

```rust
impl WorkflowParser {
    /// Load additional ontology (e.g., domain-specific extensions)
    pub fn extend_ontology(&mut self, ontology_path: &Path, namespace: &str) -> WorkflowResult<()> {
        // Check for circular imports
        if self.loaded_ontologies.contains(namespace) {
            return Ok(()); // Already loaded
        }

        // Load TTL into store
        self.load_ontology_file(ontology_path.to_path_buf(), namespace)?;

        // Re-verify integrity after extension
        self.verify_ontology_integrity()?;

        Ok(())
    }

    /// List loaded ontologies
    pub fn loaded_ontologies(&self) -> Vec<String> {
        self.loaded_ontologies.iter().cloned().collect()
    }
}
```

---

## 4. SPARQL Extraction Pipeline

### 4.1 Enhanced Extraction Architecture

**Current limitation:** Extraction happens in separate steps (tasks, conditions, flows). This requires multiple SPARQL queries and can be inefficient.

**Enhanced approach:** Use federated SPARQL queries to extract related data in fewer queries.

**File:** `src/parser/extractor.rs`

#### 4.1.1 Optimized Task Extraction

```rust
/// Extract tasks with all properties in a single query
pub fn extract_tasks_optimized(
    store: &Store,
    yawl_ns: &str,
    knhk_ns: &str,
    spec_iri: Option<&str>,
) -> WorkflowResult<HashMap<String, Task>> {
    let mut tasks = HashMap::new();

    // Comprehensive SPARQL query extracting ALL task properties
    let query = if let Some(spec) = spec_iri {
        format!(
            r#"
            PREFIX yawl: <{yawl}>
            PREFIX knhk: <{knhk}>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

            SELECT ?task ?name ?type ?split ?join
                   ?maxTicks ?priority ?simd
                   ?spanTemplate ?provenanceRequired
                   ?allocPolicy ?role ?capability
            WHERE {{
                <{spec}> yawl:hasTask ?task .
                ?task rdf:type ?type .

                # Standard YAWL properties
                OPTIONAL {{ ?task rdfs:label ?name }}
                OPTIONAL {{ ?task yawl:splitType ?split }}
                OPTIONAL {{ ?task yawl:joinType ?join }}

                # knhk performance extensions
                OPTIONAL {{ ?task knhk:tickBudget ?maxTicks }}
                OPTIONAL {{ ?task knhk:priority ?priority }}
                OPTIONAL {{ ?task knhk:useSimd ?simd }}

                # knhk OTEL extensions
                OPTIONAL {{ ?task knhk:hasSpanTemplate ?spanTemplate }}

                # knhk provenance extensions
                OPTIONAL {{ ?task knhk:requiresProvenance ?provenanceRequired }}

                # Resource allocation
                OPTIONAL {{ ?task yawl:hasAllocationPolicy ?allocPolicy }}
                OPTIONAL {{ ?task yawl:requiresRole ?role }}
                OPTIONAL {{ ?task yawl:requiresCapability ?capability }}
            }}
            "#,
            yawl = yawl_ns,
            knhk = knhk_ns,
            spec = spec
        )
    } else {
        // Fallback: Find all tasks in store
        format!(
            r#"
            PREFIX yawl: <{yawl}>
            PREFIX knhk: <{knhk}>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

            SELECT ?task ?name ?type ?split ?join
                   ?maxTicks ?priority ?simd
                   ?spanTemplate ?provenanceRequired
                   ?allocPolicy ?role ?capability
            WHERE {{
                ?task rdf:type yawl:Task .

                # Standard YAWL properties
                OPTIONAL {{ ?task rdfs:label ?name }}
                OPTIONAL {{ ?task yawl:splitType ?split }}
                OPTIONAL {{ ?task yawl:joinType ?join }}

                # knhk extensions
                OPTIONAL {{ ?task knhk:tickBudget ?maxTicks }}
                OPTIONAL {{ ?task knhk:priority ?priority }}
                OPTIONAL {{ ?task knhk:useSimd ?simd }}
                OPTIONAL {{ ?task knhk:hasSpanTemplate ?spanTemplate }}
                OPTIONAL {{ ?task knhk:requiresProvenance ?provenanceRequired }}

                # Resource allocation
                OPTIONAL {{ ?task yawl:hasAllocationPolicy ?allocPolicy }}
                OPTIONAL {{ ?task yawl:requiresRole ?role }}
                OPTIONAL {{ ?task yawl:requiresCapability ?capability }}
            }}
            "#,
            yawl = yawl_ns,
            knhk = knhk_ns
        )
    };

    // Execute query
    #[allow(deprecated)]
    let results = store.query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Task query failed: {:?}", e)))?;

    // Process results
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
        // Group by task IRI (since roles/capabilities may produce multiple rows)
        let mut task_data: HashMap<String, TaskBuilder> = HashMap::new();

        for solution in solutions {
            let solution = solution?;

            let task_id = solution.get("task")
                .ok_or(WorkflowError::Parse("Missing task IRI".into()))?
                .to_string();

            // Get or create task builder
            let builder = task_data.entry(task_id.clone())
                .or_insert_with(|| TaskBuilder::new(task_id.clone()));

            // Extract properties
            if let Some(name) = solution.get("name") {
                builder.name = extract_literal_string(name);
            }
            if let Some(type_term) = solution.get("type") {
                builder.task_type = parse_task_type(type_term);
            }
            if let Some(split) = solution.get("split") {
                builder.split_type = parse_split_type(split);
            }
            if let Some(join) = solution.get("join") {
                builder.join_type = parse_join_type(join);
            }
            if let Some(max_ticks) = solution.get("maxTicks") {
                builder.max_ticks = extract_literal_u32(max_ticks);
            }
            if let Some(priority) = solution.get("priority") {
                builder.priority = extract_literal_u32(priority);
            }
            if let Some(simd) = solution.get("simd") {
                builder.use_simd = extract_literal_bool(simd);
            }
            if let Some(span_template) = solution.get("spanTemplate") {
                builder.span_template = Some(extract_literal_string(span_template));
            }
            if let Some(prov_req) = solution.get("provenanceRequired") {
                builder.provenance_required = extract_literal_bool(prov_req);
            }
            if let Some(policy) = solution.get("allocPolicy") {
                builder.allocation_policy = Some(parse_allocation_policy(policy)?);
            }
            if let Some(role) = solution.get("role") {
                builder.required_roles.push(extract_literal_string(role));
            }
            if let Some(capability) = solution.get("capability") {
                builder.required_capabilities.push(extract_literal_string(capability));
            }
        }

        // Build final tasks
        for (task_id, builder) in task_data {
            tasks.insert(task_id, builder.build());
        }
    }

    Ok(tasks)
}

/// Helper struct to build tasks from SPARQL results
struct TaskBuilder {
    id: String,
    name: String,
    task_type: TaskType,
    split_type: SplitType,
    join_type: JoinType,
    max_ticks: Option<u32>,
    priority: Option<u32>,
    use_simd: bool,
    span_template: Option<String>,
    provenance_required: bool,
    allocation_policy: Option<AllocationPolicy>,
    required_roles: Vec<String>,
    required_capabilities: Vec<String>,
}

impl TaskBuilder {
    fn new(id: String) -> Self {
        Self {
            id,
            name: "Unnamed Task".to_string(),
            task_type: TaskType::Atomic,
            split_type: SplitType::And,
            join_type: JoinType::And,
            max_ticks: None,
            priority: None,
            use_simd: false,
            span_template: None,
            provenance_required: false,
            allocation_policy: None,
            required_roles: Vec::new(),
            required_capabilities: Vec::new(),
        }
    }

    fn build(self) -> Task {
        Task {
            id: self.id,
            name: self.name,
            task_type: self.task_type,
            split_type: self.split_type,
            join_type: self.join_type,
            max_ticks: self.max_ticks,
            priority: self.priority,
            use_simd: self.use_simd,
            input_conditions: Vec::new(),  // Populated by extract_flows
            output_conditions: Vec::new(),
            outgoing_flows: Vec::new(),
            incoming_flows: Vec::new(),
            allocation_policy: self.allocation_policy,
            required_roles: self.required_roles,
            required_capabilities: self.required_capabilities,
            exception_worklet: None,  // Populated separately
        }
    }
}

/// Extract string from RDF literal
fn extract_literal_string(term: &oxigraph::model::Term) -> String {
    if let oxigraph::model::Term::Literal(lit) = term {
        lit.value().to_string()
    } else {
        "".to_string()
    }
}

/// Extract u32 from RDF literal
fn extract_literal_u32(term: &oxigraph::model::Term) -> Option<u32> {
    if let oxigraph::model::Term::Literal(lit) = term {
        lit.value().parse::<u32>().ok()
    } else {
        None
    }
}

/// Extract bool from RDF literal
fn extract_literal_bool(term: &oxigraph::model::Term) -> bool {
    if let oxigraph::model::Term::Literal(lit) = term {
        lit.value().parse::<bool>().unwrap_or(false)
    } else {
        false
    }
}
```

#### 4.1.2 Flow Extraction Optimization

```rust
/// Extract flows with single query instead of multiple passes
pub fn extract_flows_optimized(
    store: &Store,
    yawl_ns: &str,
    tasks: &mut HashMap<String, Task>,
    conditions: &mut HashMap<String, Condition>,
) -> WorkflowResult<()> {
    let query = format!(
        r#"
        PREFIX yawl: <{yawl}>

        SELECT ?from ?to ?predicate
        WHERE {{
            {{ ?from yawl:hasOutgoingFlow ?to . BIND(yawl:hasOutgoingFlow AS ?predicate) }}
            UNION
            {{ ?from yawl:flowsTo ?to . BIND(yawl:flowsTo AS ?predicate) }}
        }}
        "#,
        yawl = yawl_ns
    );

    #[allow(deprecated)]
    let results = store.query(&query)
        .map_err(|e| WorkflowError::Parse(format!("Flow query failed: {:?}", e)))?;

    if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
        for solution in solutions {
            let solution = solution?;

            let from_id = solution.get("from")
                .ok_or(WorkflowError::Parse("Missing from in flow".into()))?
                .to_string();
            let to_id = solution.get("to")
                .ok_or(WorkflowError::Parse("Missing to in flow".into()))?
                .to_string();

            // Update outgoing flows
            if let Some(task) = tasks.get_mut(&from_id) {
                task.outgoing_flows.push(to_id.clone());
            }
            if let Some(condition) = conditions.get_mut(&from_id) {
                condition.outgoing_flows.push(to_id.clone());
            }

            // Update incoming flows
            if let Some(task) = tasks.get_mut(&to_id) {
                task.incoming_flows.push(from_id.clone());
            }
            if let Some(condition) = conditions.get_mut(&to_id) {
                condition.incoming_flows.push(from_id.clone());
            }
        }
    }

    Ok(())
}
```

### 4.2 Prepared Query Optimization

For production performance, compile SPARQL queries once and reuse.

```rust
pub struct PreparedQueries {
    extract_tasks: String,
    extract_conditions: String,
    extract_flows: String,
    find_start: String,
    find_end: String,
}

impl PreparedQueries {
    pub fn new(yawl_ns: &str, knhk_ns: &str) -> Self {
        Self {
            extract_tasks: Self::build_task_query(yawl_ns, knhk_ns),
            extract_conditions: Self::build_condition_query(yawl_ns),
            extract_flows: Self::build_flow_query(yawl_ns),
            find_start: Self::build_start_query(yawl_ns),
            find_end: Self::build_end_query(yawl_ns),
        }
    }

    fn build_task_query(yawl_ns: &str, knhk_ns: &str) -> String {
        format!(
            r#"
            PREFIX yawl: <{yawl}>
            PREFIX knhk: <{knhk}>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

            SELECT ?task ?name ?type ?split ?join ?maxTicks ?priority ?simd
            WHERE {{
                ?task rdf:type yawl:Task .
                OPTIONAL {{ ?task rdfs:label ?name }}
                OPTIONAL {{ ?task yawl:splitType ?split }}
                OPTIONAL {{ ?task yawl:joinType ?join }}
                OPTIONAL {{ ?task knhk:tickBudget ?maxTicks }}
                OPTIONAL {{ ?task knhk:priority ?priority }}
                OPTIONAL {{ ?task knhk:useSimd ?simd }}
            }}
            "#,
            yawl = yawl_ns,
            knhk = knhk_ns
        )
    }

    // ... similar for other queries
}

impl WorkflowParser {
    /// Parse with prepared queries (faster)
    pub fn parse_turtle_optimized(&mut self, turtle: &str, prepared: &PreparedQueries) -> WorkflowResult<WorkflowSpec> {
        // Load TTL
        self.store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())?;

        // Extract using prepared queries (no query string formatting overhead)
        let spec = self.extract_with_prepared(&prepared)?;

        // Validate
        self.deadlock_detector.validate(&spec)?;

        Ok(spec)
    }
}
```

---

## 5. Rust Type Construction

### 5.1 Enhanced Task Type

**File:** `src/parser/types.rs`

Add knhk-specific fields to `Task`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    // Standard YAWL fields (existing)
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub split_type: SplitType,
    pub join_type: JoinType,
    pub max_ticks: Option<u32>,
    pub priority: Option<u32>,
    pub use_simd: bool,
    pub input_conditions: Vec<String>,
    pub output_conditions: Vec<String>,
    pub outgoing_flows: Vec<String>,
    pub incoming_flows: Vec<String>,
    pub allocation_policy: Option<AllocationPolicy>,
    pub required_roles: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub exception_worklet: Option<WorkletId>,

    // NEW: knhk extensions from ontology
    /// OTEL span template (from knhk:hasSpanTemplate)
    pub span_template: Option<String>,

    /// Provenance requirement (from knhk:requiresProvenance)
    pub provenance_required: bool,

    /// Git commit hash constraint (from knhk:requiresCommitHash)
    pub required_commit_hash: Option<String>,

    /// Performance tier (from knhk:performanceTier: "hot", "warm", "cold")
    pub performance_tier: PerformanceTier,

    /// Validation schemas (from knhk:hasValidationSchema)
    pub validation_schemas: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PerformanceTier {
    /// Hot path: ≤8 ticks
    Hot,
    /// Warm path: ≤100 ticks
    Warm,
    /// Cold path: no constraint
    Cold,
}

impl Default for PerformanceTier {
    fn default() -> Self {
        Self::Cold
    }
}
```

### 5.2 Type Mapping Strategy

**RDF → Rust mapping rules:**

| RDF Property | Rust Field | Type | Notes |
|--------------|------------|------|-------|
| `rdfs:label` | `name` | `String` | Task display name |
| `yawl:splitType` | `split_type` | `SplitType` | Enum: And, Xor, Or |
| `yawl:joinType` | `join_type` | `JoinType` | Enum: And, Xor, Or |
| `knhk:tickBudget` | `max_ticks` | `Option<u32>` | Performance constraint |
| `knhk:priority` | `priority` | `Option<u32>` | Execution priority |
| `knhk:useSimd` | `use_simd` | `bool` | SIMD optimization flag |
| `knhk:hasSpanTemplate` | `span_template` | `Option<String>` | OTEL span template ID |
| `knhk:requiresProvenance` | `provenance_required` | `bool` | Lockchain tracking |
| `knhk:performanceTier` | `performance_tier` | `PerformanceTier` | Hot/Warm/Cold |
| `knhk:hasValidationSchema` | `validation_schemas` | `Vec<String>` | Weaver schema refs |

---

## 6. Validation Integration

### 6.1 Multi-Level Validation Pipeline

**Validation Order:**
1. **SHACL Schema Validation** (structural integrity)
2. **SPARQL Semantic Validation** (business rules)
3. **Deadlock Detection** (workflow soundness)
4. **Weaver Schema Validation** (OTEL compliance)

#### 6.1.1 SHACL Validation

```rust
impl WorkflowParser {
    /// Validate workflow against SHACL shapes
    pub fn validate_shacl(&self, spec_iri: &str) -> WorkflowResult<ValidationReport> {
        if self.shacl_shapes.is_none() {
            return Err(WorkflowError::Validation(
                "SHACL shapes not loaded".into()
            ));
        }

        let mut violations = Vec::new();

        // Execute SHACL validation queries
        for shape in self.shacl_shapes.as_ref().unwrap() {
            let validation_query = format!(
                r#"
                PREFIX sh: <http://www.w3.org/ns/shacl#>
                PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>

                SELECT ?focusNode ?message
                WHERE {{
                    <{spec}> ?p ?focusNode .
                    # Shape-specific validation logic here
                    # This would be generated from SHACL shape definitions
                }}
                "#,
                spec = spec_iri
            );

            // Execute validation query
            #[allow(deprecated)]
            let results = self.store.query(&validation_query)
                .map_err(|e| WorkflowError::Validation(format!("SHACL query failed: {:?}", e)))?;

            if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
                for solution in solutions {
                    let solution = solution?;
                    violations.push(ShaclViolation {
                        shape_id: shape.id.clone(),
                        focus_node: solution.get("focusNode").map(|t| t.to_string()),
                        message: solution.get("message").map(|t| extract_literal_string(t)),
                    });
                }
            }
        }

        Ok(ValidationReport {
            valid: violations.is_empty(),
            violations,
        })
    }
}

#[derive(Debug)]
pub struct ValidationReport {
    pub valid: bool,
    pub violations: Vec<ShaclViolation>,
}

#[derive(Debug)]
pub struct ShaclViolation {
    pub shape_id: String,
    pub focus_node: Option<String>,
    pub message: Option<String>,
}
```

#### 6.1.2 SPARQL Semantic Validation

```rust
impl WorkflowParser {
    /// Validate workflow semantics using SPARQL queries
    pub fn validate_semantics(&self, spec_iri: &str) -> WorkflowResult<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Rule 1: Start condition has no incoming flows
        if !self.check_start_condition_validity(spec_iri)? {
            errors.push(ValidationError::StartConditionHasIncomingFlows);
        }

        // Rule 2: End condition has no outgoing flows
        if !self.check_end_condition_validity(spec_iri)? {
            errors.push(ValidationError::EndConditionHasOutgoingFlows);
        }

        // Rule 3: All tasks have at least one incoming flow (except after start)
        errors.extend(self.check_task_connectivity(spec_iri)?);

        // Rule 4: Split/Join type compatibility
        errors.extend(self.check_split_join_compatibility(spec_iri)?);

        // Rule 5: Resource allocation validity
        errors.extend(self.check_resource_allocation(spec_iri)?);

        Ok(errors)
    }

    fn check_start_condition_validity(&self, spec_iri: &str) -> WorkflowResult<bool> {
        let query = format!(
            r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>

            ASK {{
                <{spec}> yawl:hasStartCondition ?start .
                ?from yawl:hasOutgoingFlow ?start .
            }}
            "#,
            spec = spec_iri
        );

        // Returns true if start has incoming flows (invalid)
        // We want this to return false (valid: no incoming flows)
        self.ask_query(&query).map(|result| !result)
    }

    fn check_end_condition_validity(&self, spec_iri: &str) -> WorkflowResult<bool> {
        let query = format!(
            r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>

            ASK {{
                <{spec}> yawl:hasEndCondition ?end .
                ?end yawl:hasOutgoingFlow ?to .
            }}
            "#,
            spec = spec_iri
        );

        self.ask_query(&query).map(|result| !result)
    }
}

#[derive(Debug)]
pub enum ValidationError {
    StartConditionHasIncomingFlows,
    EndConditionHasOutgoingFlows,
    TaskNotConnected(String),
    IncompatibleSplitJoin(String),
    InvalidResourceAllocation(String),
}
```

### 6.2 Integrated Validation Flow

```rust
impl WorkflowParser {
    /// Full validation pipeline
    pub fn validate_workflow(&self, spec: &WorkflowSpec, spec_iri: &str) -> WorkflowResult<()> {
        // Phase 1: SHACL schema validation
        let shacl_report = self.validate_shacl(spec_iri)?;
        if !shacl_report.valid {
            return Err(WorkflowError::Validation(
                format!("SHACL validation failed: {:?}", shacl_report.violations)
            ));
        }

        // Phase 2: SPARQL semantic validation
        let semantic_errors = self.validate_semantics(spec_iri)?;
        if !semantic_errors.is_empty() {
            return Err(WorkflowError::Validation(
                format!("Semantic validation failed: {:?}", semantic_errors)
            ));
        }

        // Phase 3: Deadlock detection
        self.deadlock_detector.validate(spec)?;

        // Phase 4: Weaver OTEL schema validation (if schemas defined)
        self.validate_otel_schemas(spec)?;

        Ok(())
    }

    fn validate_otel_schemas(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        // For each task with span_template, verify it exists in Weaver registry
        for task in spec.tasks.values() {
            if let Some(ref template) = task.span_template {
                // This would call weaver CLI or use weaver library
                // For now, placeholder
                if !self.check_weaver_span_exists(template)? {
                    return Err(WorkflowError::Validation(
                        format!("OTEL span template '{}' not found in Weaver registry", template)
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_weaver_span_exists(&self, template: &str) -> WorkflowResult<bool> {
        // Placeholder: Would call weaver or check loaded schema
        // weaver registry check -r registry/ --span-name <template>
        Ok(true)
    }
}
```

---

## 7. Caching Strategy

### 7.1 WorkflowSpec Cache

**Requirement:** Hot path execution MUST NOT query RDF. All workflow metadata must be in Rust structs.

```rust
pub struct WorkflowParser {
    store: Store,
    deadlock_detector: DeadlockDetector,
    loaded_ontologies: HashSet<String>,
    shacl_shapes: Option<Vec<ShapeDefinition>>,

    // NEW: Workflow spec cache
    spec_cache: Arc<RwLock<HashMap<String, WorkflowSpec>>>,

    // NEW: IRI → WorkflowSpecId mapping
    iri_to_spec_id: Arc<RwLock<HashMap<String, WorkflowSpecId>>>,
}

impl WorkflowParser {
    /// Parse and cache workflow
    pub fn parse_and_cache(&mut self, turtle: &str, cache_key: &str) -> WorkflowResult<WorkflowSpec> {
        // Check cache first
        {
            let cache = self.spec_cache.read().unwrap();
            if let Some(spec) = cache.get(cache_key) {
                return Ok(spec.clone());
            }
        }

        // Parse from TTL
        let spec = self.parse_turtle(turtle)?;

        // Cache the result
        {
            let mut cache = self.spec_cache.write().unwrap();
            cache.insert(cache_key.to_string(), spec.clone());
        }

        Ok(spec)
    }

    /// Get cached spec by key
    pub fn get_cached_spec(&self, cache_key: &str) -> Option<WorkflowSpec> {
        let cache = self.spec_cache.read().unwrap();
        cache.get(cache_key).cloned()
    }

    /// Clear cache (for testing or memory management)
    pub fn clear_cache(&mut self) {
        let mut cache = self.spec_cache.write().unwrap();
        cache.clear();
    }
}
```

### 7.2 RocksDB Backend for Production

```rust
impl WorkflowParser {
    /// Create parser with RocksDB backend (persistent)
    pub fn new_with_rocksdb(db_path: &Path) -> WorkflowResult<Self> {
        let store = Store::open(db_path)
            .map_err(|e| WorkflowError::Parse(format!("Failed to open RocksDB: {:?}", e)))?;

        Ok(Self {
            store,
            deadlock_detector: DeadlockDetector,
            loaded_ontologies: HashSet::new(),
            shacl_shapes: None,
            spec_cache: Arc::new(RwLock::new(HashMap::new())),
            iri_to_spec_id: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}
```

---

## 8. Error Handling and Recovery

### 8.1 Granular Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Ontology not found: {0}")]
    OntologyNotFound(String),

    #[error("Ontology parse error: {0}")]
    OntologyParseError(String),

    #[error("SPARQL query error: {0}")]
    SparqlError(String),

    #[error("SHACL validation failed: {violations:?}")]
    ShaclValidationFailed {
        violations: Vec<ShaclViolation>,
    },

    #[error("Semantic validation failed: {errors:?}")]
    SemanticValidationFailed {
        errors: Vec<ValidationError>,
    },

    #[error("Weaver schema not found: {0}")]
    WeaverSchemaNotFound(String),

    #[error("Missing required property: {property} on {element}")]
    MissingRequiredProperty {
        property: String,
        element: String,
    },
}
```

### 8.2 Recovery Strategies

```rust
impl WorkflowParser {
    /// Parse with fallback to default values
    pub fn parse_with_defaults(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        match self.parse_turtle(turtle) {
            Ok(spec) => Ok(spec),
            Err(e) => {
                // Log error
                tracing::warn!("Parse error, applying defaults: {:?}", e);

                // Return minimal valid spec
                Ok(WorkflowSpec {
                    id: WorkflowSpecId::new(),
                    name: "Fallback Workflow".into(),
                    tasks: HashMap::new(),
                    conditions: HashMap::new(),
                    start_condition: None,
                    end_condition: None,
                })
            }
        }
    }

    /// Incremental parsing: continue on errors, collect all errors
    pub fn parse_lenient(&mut self, turtle: &str) -> (WorkflowSpec, Vec<ParserError>) {
        let mut errors = Vec::new();

        // Load TTL
        if let Err(e) = self.store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes()) {
            errors.push(ParserError::OntologyParseError(format!("{:?}", e)));
        }

        // Extract tasks (continue on error)
        let tasks = match extract_tasks(&self.store, "http://bitflow.ai/ontology/yawl/v2#", None) {
            Ok(tasks) => tasks,
            Err(e) => {
                errors.push(ParserError::SparqlError(format!("Task extraction: {:?}", e)));
                HashMap::new()
            }
        };

        // Extract conditions (continue on error)
        let conditions = match extract_conditions(&self.store, "http://bitflow.ai/ontology/yawl/v2#", None) {
            Ok(conds) => conds,
            Err(e) => {
                errors.push(ParserError::SparqlError(format!("Condition extraction: {:?}", e)));
                HashMap::new()
            }
        };

        // Build partial spec
        let spec = WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: "Partially Parsed Workflow".into(),
            tasks,
            conditions,
            start_condition: None,
            end_condition: None,
        };

        (spec, errors)
    }
}
```

---

## 9. Backward Compatibility

### 9.1 Support for Non-Ontology Workflows

**Requirement:** Existing workflows without ontology annotations must continue to work.

```rust
impl WorkflowParser {
    /// Parse workflow with or without ontology
    pub fn parse_auto(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // Detect if ontology is referenced
        if self.has_ontology_reference(turtle) {
            // Full ontology-based parsing
            self.parse_turtle(turtle)
        } else {
            // Legacy parsing (minimal RDF, direct struct construction)
            self.parse_legacy(turtle)
        }
    }

    fn has_ontology_reference(&self, turtle: &str) -> bool {
        turtle.contains("yawl:") || turtle.contains("knhk:")
    }

    fn parse_legacy(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // Legacy parsing logic (existing implementation)
        self.parse_turtle(turtle)
    }
}
```

### 9.2 Migration Path

Provide tool to migrate legacy workflows to ontology-based format:

```rust
pub fn migrate_legacy_to_ontology(
    legacy_ttl: &str,
    ontology_parser: &WorkflowParser,
) -> WorkflowResult<String> {
    // Parse legacy format
    let mut parser = WorkflowParser::new()?;
    let spec = parser.parse_turtle(legacy_ttl)?;

    // Convert to ontology-compliant TTL
    let mut ttl_output = String::new();

    // Add prefixes
    ttl_output.push_str("@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .\n");
    ttl_output.push_str("@prefix knhk: <http://knhk.org/ontology#> .\n");
    ttl_output.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\n");

    // Add workflow spec
    ttl_output.push_str(&format!(
        "ex:Workflow a yawl:WorkflowSpecification ;\n    rdfs:label \"{}\" .\n\n",
        spec.name
    ));

    // Add tasks
    for (task_id, task) in spec.tasks.iter() {
        ttl_output.push_str(&format!(
            "<{}> a yawl:Task ;\n    rdfs:label \"{}\" ;\n    yawl:splitType \"{}\" ;\n    yawl:joinType \"{}\" .\n\n",
            task_id,
            task.name,
            format!("{:?}", task.split_type).to_uppercase(),
            format!("{:?}", task.join_type).to_uppercase()
        ));
    }

    Ok(ttl_output)
}
```

---

## 10. Performance Optimization

### 10.1 Benchmark Target

**Parsing is cold-path** - no ≤8 tick constraint, but should be reasonably fast:
- **Target:** Parse 100-task workflow in <100ms
- **Target:** Parse 1000-task workflow in <1s

### 10.2 Optimization Techniques

1. **Prepared Queries:** Pre-compile SPARQL queries (see Section 4.2)
2. **Batch Extraction:** Extract all properties in single query (see Section 4.1.1)
3. **RocksDB Backend:** Use persistent store for large ontologies
4. **Lazy Loading:** Don't load full ontology if not needed
5. **Parallel Extraction:** Extract tasks, conditions, flows in parallel

```rust
impl WorkflowParser {
    /// Parallel extraction for large workflows
    pub async fn parse_turtle_parallel(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // Load TTL
        self.store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())?;

        // Extract in parallel
        let (tasks, conditions, start, end) = tokio::join!(
            async { extract_tasks(&self.store, "yawl", None) },
            async { extract_conditions(&self.store, "yawl", None) },
            async { find_start_condition(&self.store, "yawl", None) },
            async { find_end_condition(&self.store, "yawl", None) },
        );

        let mut tasks = tasks?;
        let mut conditions = conditions?;
        let start_condition = start?;
        let end_condition = end?;

        // Extract flows (depends on tasks/conditions)
        extract_flows(&self.store, "yawl", None, &mut tasks, &mut conditions)?;

        Ok(WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: "Parsed Workflow".into(),
            tasks,
            conditions,
            start_condition,
            end_condition,
        })
    }
}
```

---

## 11. Implementation Roadmap

### Phase 1: Enhanced Extraction (Weeks 1-2)
- ✅ Implement `extract_tasks_optimized()` with knhk extensions
- ✅ Implement `extract_flows_optimized()` for batch flow extraction
- ✅ Add `PerformanceTier`, `span_template` fields to `Task`
- ✅ Add prepared query optimization

### Phase 2: Validation Pipeline (Weeks 3-4)
- ⬜ Implement SHACL shape loading and validation
- ⬜ Implement SPARQL semantic validation rules
- ⬜ Integrate Weaver schema validation
- ⬜ Create comprehensive validation reporting

### Phase 3: Caching & Performance (Weeks 5-6)
- ⬜ Implement WorkflowSpec caching with RwLock
- ⬜ Add RocksDB backend support
- ⬜ Implement parallel extraction
- ⬜ Benchmark and optimize SPARQL queries

### Phase 4: Migration & Compatibility (Week 7)
- ⬜ Implement `parse_auto()` for backward compatibility
- ⬜ Create migration tool: legacy TTL → ontology TTL
- ⬜ Add comprehensive error handling and recovery
- ⬜ Create parser integration tests

---

## Summary

This integration design provides a **hyper-detailed, implementation-ready blueprint** for wiring the YAWL ontology into the existing `WorkflowParser`. Key features:

1. **Bootstrap Sequence:** Load ontologies in dependency order (yawl-core → yawl-patterns → knhk-extensions → SHACL shapes)
2. **Optimized Extraction:** Single comprehensive SPARQL queries with TaskBuilder pattern
3. **Multi-Level Validation:** SHACL → SPARQL → Deadlock → Weaver
4. **Caching Strategy:** In-memory cache + optional RocksDB backend
5. **Backward Compatibility:** Support legacy workflows without ontology
6. **Error Handling:** Granular error types with recovery strategies
7. **Performance:** Prepared queries, parallel extraction, hot-path optimization

**Next:** See `executor-integration-design.md` for runtime integration.
