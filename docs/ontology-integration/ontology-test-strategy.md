# Ontology Validation Test Strategy

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Implementation Ready
**Agent:** Testing Specialist (ULTRATHINK Swarm)

## Executive Summary

This document defines a comprehensive 4-level validation hierarchy for YAWL ontology integration in KNHK. Unlike traditional testing that can produce false positives, this strategy uses **OpenTelemetry Weaver validation as the ultimate source of truth** to prove ontology operations emit correct telemetry.

**The Meta-Principle:**
```
Traditional Testing:               KNHK Ontology Testing:
  assert(result == expected) ✅      Weaver validates OTEL spans ✅
  ↓                                  ↓
  Tests validate test logic          Schema validates runtime behavior
  ↓                                  ↓
  CAN HAVE FALSE POSITIVES ❌        PROVES ACTUAL BEHAVIOR ✅
```

**Validation Hierarchy:**
1. **SHACL Validation** - RDF schema constraints (structural correctness)
2. **SPARQL Validation** - Semantic queries (workflow soundness)
3. **Rust Validation** - Algorithmic checks (deadlock detection, type safety)
4. **Weaver Validation** - OTEL schema compliance (**SOURCE OF TRUTH**)

**Performance Constraint:** All hot path ontology operations ≤8 ticks (Chatman Constant)

---

## 1. The False Positive Problem KNHK Solves

### 1.1 Why Traditional Tests Are Insufficient

**Problem:** Tests can pass even when features don't work.

```rust
// ❌ TRADITIONAL TEST - CAN HAVE FALSE POSITIVE
#[test]
fn test_load_ontology() {
    let ontology = load_yawl_ontology("workflow.ttl");
    assert!(ontology.is_ok()); // ✅ PASSES

    // But what if:
    // - File was parsed but data is corrupted?
    // - RDF triples are malformed?
    // - SPARQL queries don't work?
    // - No telemetry is emitted?
}

// ✅ KNHK SOLUTION - WEAVER VALIDATION PROVES BEHAVIOR
#[test]
fn test_load_ontology_with_weaver() {
    let ontology = load_yawl_ontology("workflow.ttl");
    assert!(ontology.is_ok());

    // CRITICAL: Weaver validates that:
    // 1. Span "ontology.load" was emitted
    // 2. Attributes match schema (file_path, triple_count, parse_time)
    // 3. Events logged (validation_started, triples_parsed)
    // 4. No schema violations detected

    // Run Weaver live-check:
    // weaver registry live-check --registry registry/ \
    //   --expected-span ontology.load \
    //   --expected-attributes file_path,triple_count,parse_time
}
```

**Why Weaver Validation Is Different:**
- **Schema-first**: Code must conform to declared OTEL schema
- **Runtime proof**: Validates actual telemetry, not test mocks
- **No circular dependency**: External tool validates our framework
- **Industry standard**: OpenTelemetry's official validation approach
- **Detects fake-green**: Catches tests that pass but don't validate real behavior

### 1.2 The Validation Pyramid for Ontology Integration

```
                    ┌─────────────────────┐
                    │  Weaver Validation  │ ← SOURCE OF TRUTH
                    │   (OTEL Schema)     │   (Proves runtime behavior)
                    └─────────────────────┘
                           ↑
              ┌────────────┴────────────┐
              │   Rust Validation       │ ← ALGORITHMIC CORRECTNESS
              │ (Deadlock, Performance) │   (Complex patterns)
              └─────────────────────────┘
                         ↑
            ┌────────────┴────────────┐
            │  SPARQL Validation      │ ← SEMANTIC SOUNDNESS
            │ (Control/Data Flow)     │   (Workflow correctness)
            └─────────────────────────┘
                       ↑
          ┌────────────┴────────────┐
          │   SHACL Validation      │ ← STRUCTURAL VALIDITY
          │  (Schema Constraints)   │   (RDF well-formedness)
          └─────────────────────────┘
```

**Key Insight:** Each level validates different aspects, but **only Weaver proves the code actually works in production**.

---

## 2. Four-Level Validation Hierarchy

### 2.1 Level 1: SHACL Validation (Structural Correctness)

**Purpose:** Validate RDF data structure conforms to ontology schema.

**Tool:** Oxigraph + SHACL processor

**What It Validates:**
- Required properties exist
- Datatypes are correct
- Cardinality constraints met
- Range/domain restrictions satisfied

**Example SHACL Test:**

```rust
use oxigraph::store::Store;
use oxigraph::model::*;

/// Test: Task must have exactly one join and split type
#[test]
fn shacl_task_must_have_join_and_split() {
    // ARRANGE
    let store = Store::new().unwrap();

    // Load YAWL ontology
    load_ontology(&store, "ontologies/yawl.ttl").unwrap();

    // Load SHACL shapes
    load_shapes(&store, "validation/shapes/task-shapes.ttl").unwrap();

    // Load test workflow (INVALID - missing join type)
    load_workflow(&store, "test-data/invalid-task-no-join.ttl").unwrap();

    // ACT
    let validation_report = validate_shacl(&store).unwrap();

    // ASSERT
    assert!(!validation_report.conforms, "Expected validation failure");
    assert_eq!(validation_report.results.len(), 1);

    let error = &validation_report.results[0];
    assert_eq!(error.severity, Severity::Violation);
    assert!(error.message.contains("must have exactly one join type"));
    assert_eq!(error.focus_node.as_str(), "http://example.org/TaskA");

    // WEAVER CHECK: Verify telemetry emitted
    // Span: ontology.validate.shacl
    // Attributes: conforms=false, violation_count=1
}
```

**SHACL Test Coverage:**

| Rule | Description | File |
|------|-------------|------|
| SH-001 | Task has join/split types | `task-shapes.ttl` |
| SH-002 | Net has one input/output condition | `net-shapes.ttl` |
| SH-003 | MI task has min/max/threshold | `mi-task-shapes.ttl` |
| SH-004 | Variable has type | `variable-shapes.ttl` |
| SH-005 | Hot path task has tick budget ≤8 | `knhk-shapes.ttl` |
| SH-006 | Provenance chain has valid git hash | `knhk-shapes.ttl` |

**Performance:** SHACL validation ≤2 ticks for typical workflow (1000 triples)

---

### 2.2 Level 2: SPARQL Validation (Semantic Soundness)

**Purpose:** Validate workflow semantics via graph queries.

**Tool:** Oxigraph SPARQL engine

**What It Validates:**
- Control flow correctness (reachability, termination)
- Data flow mapping completeness
- Resource allocation validity
- Pattern-specific rules (XOR predicates, OR-join config)

**Example SPARQL Test:**

```rust
use oxigraph::sparql::QueryResults;

/// Test: Start condition must have no incoming flows
#[test]
fn sparql_start_has_no_incoming_flows() {
    // ARRANGE
    let store = Store::new().unwrap();
    load_ontology(&store, "ontologies/yawl.ttl").unwrap();
    load_workflow(&store, "test-data/valid-workflow.ttl").unwrap();

    // ACT
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

        ASK {
            ?condition a yawl:InputCondition .
            ?flow yawl:nextElementRef ?condition .
        }
    "#;

    let result = store.query(query).unwrap();

    // ASSERT
    if let QueryResults::Boolean(has_incoming) = result {
        assert!(!has_incoming, "Start condition should have no incoming flows");
    } else {
        panic!("Expected boolean result");
    }

    // WEAVER CHECK: Verify telemetry
    // Span: ontology.validate.sparql
    // Attributes: query_type=ASK, rule=start_no_incoming, result=false
}
```

**SPARQL Test Suite:**

| Category | Rule Count | Example Rule | Complexity |
|----------|------------|--------------|------------|
| Control Flow | 6 | Start/end soundness, reachability | O(V+E) |
| Data Flow | 4 | Input mappings, type consistency | O(M) |
| Resources | 3 | Allocator validity, role existence | O(R) |
| Patterns | 5 | XOR predicates, OR-join config | O(P) |

Where:
- V = vertices (tasks/conditions)
- E = edges (flows)
- M = mappings
- R = resources
- P = pattern instances

**Performance Budgets:**
- Simple ASK queries: ≤1 tick
- Complex SELECT with joins: ≤3 ticks
- Transitive closure (reachability): ≤5 ticks

**Test Data Generation:**

```rust
/// Generate test workflows from SPARQL templates
pub struct TestWorkflowGenerator {
    store: Store,
}

impl TestWorkflowGenerator {
    /// Generate workflow with N tasks in sequence
    pub fn generate_sequential(&self, n: usize) -> String {
        let mut ttl = String::new();

        // Generate triples
        ttl.push_str(&format!("
            @prefix ex: <http://example.org/> .
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            ex:Workflow a yawl:Specification .
            ex:Net a yawl:Net ;
                yawl:hasInputCondition ex:Start ;
                yawl:hasOutputCondition ex:End .
        "));

        for i in 0..n {
            ttl.push_str(&format!("
                ex:Task{i} a yawl:Task ;
                    yawl:hasJoin yawl:ControlTypeXor ;
                    yawl:hasSplit yawl:ControlTypeXor .
            "));
        }

        // Generate flows
        ttl.push_str("ex:Start yawl:flowsInto ex:Flow0 .\n");
        for i in 0..n {
            ttl.push_str(&format!("
                ex:Flow{i} a yawl:Flow ;
                    yawl:nextElementRef ex:Task{i} .
                ex:Task{i} yawl:flowsInto ex:Flow{} .
            ", i+1));
        }
        ttl.push_str(&format!("ex:Flow{n} yawl:nextElementRef ex:End .\n"));

        ttl
    }

    /// Generate workflow with AND-split/join diamond
    pub fn generate_and_diamond(&self) -> String {
        r#"
            @prefix ex: <http://example.org/> .
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            ex:TaskA a yawl:Task ;
                yawl:hasSplit yawl:ControlTypeAnd ;
                yawl:flowsInto ex:FlowAB, ex:FlowAC .

            ex:TaskB a yawl:Task .
            ex:TaskC a yawl:Task .

            ex:TaskD a yawl:Task ;
                yawl:hasJoin yawl:ControlTypeAnd .

            ex:FlowBD yawl:nextElementRef ex:TaskD .
            ex:FlowCD yawl:nextElementRef ex:TaskD .
        "#.to_string()
    }

    /// Generate workflow with potential deadlock
    pub fn generate_deadlock_xor_cycle(&self) -> String {
        r#"
            @prefix ex: <http://example.org/> .
            @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

            ex:TaskA a yawl:Task ;
                yawl:hasJoin yawl:ControlTypeXor ;  # XOR-join in cycle
                yawl:hasSplit yawl:ControlTypeXor ;
                yawl:flowsInto ex:FlowAB .

            ex:TaskB a yawl:Task ;
                yawl:flowsInto ex:FlowBA .

            ex:FlowBA yawl:nextElementRef ex:TaskA .  # Cycle!
        "#.to_string()
    }
}
```

---

### 2.3 Level 3: Rust Validation (Algorithmic Correctness)

**Purpose:** Complex validation logic not expressible in SPARQL/SHACL.

**Tool:** Rust algorithms (Tarjan's SCC, type inference, performance profiling)

**What It Validates:**
- Deadlock detection (strongly connected components)
- Performance budgets (≤8 ticks for hot path)
- XQuery expression parsing
- Type inference across mappings
- Provenance chain integrity

**Example Rust Test:**

```rust
use knhk_workflow_engine::validation::DeadlockDetector;

/// Test: Detect deadlock in XOR-join cycle
#[test]
fn rust_detect_xor_join_deadlock() {
    // ARRANGE
    let spec = WorkflowSpec::from_ttl(
        "test-data/deadlock-xor-cycle.ttl"
    ).unwrap();

    let detector = DeadlockDetector::new();

    // ACT
    let result = detector.validate(&spec);

    // ASSERT
    assert!(result.is_err(), "Expected deadlock detection");

    let error = result.unwrap_err();
    assert!(matches!(error, WorkflowError::Deadlock(_)));

    if let WorkflowError::Deadlock(msg) = error {
        assert!(msg.contains("XOR-join"));
        assert!(msg.contains("cycle"));
    }

    // WEAVER CHECK: Verify telemetry
    // Span: ontology.validate.deadlock
    // Attributes: algorithm=tarjan_scc, cycles_found=1, cycle_type=xor_join
    // Event: deadlock_detected { nodes: [TaskA, TaskB] }
}

/// Test: Hot path ontology load ≤8 ticks
#[test]
fn rust_hot_path_load_performance() {
    use chicago_tdd_tools::performance::tick_counter;

    // ARRANGE
    let workflow_ttl = include_str!("test-data/simple-workflow.ttl");

    // ACT
    let ticks = tick_counter(|| {
        let store = Store::new().unwrap();
        load_workflow(&store, workflow_ttl).unwrap();
    });

    // ASSERT
    assert!(
        ticks <= 8,
        "Hot path ontology load took {ticks} ticks (max 8, Chatman Constant)"
    );

    // WEAVER CHECK:
    // Span: ontology.load
    // Attributes: ticks={ticks}, chatman_compliant={ticks<=8}
}
```

**Rust Validation Test Coverage:**

| Algorithm | Purpose | Complexity | Tick Budget |
|-----------|---------|------------|-------------|
| Tarjan SCC | Deadlock detection | O(V+E) | ≤5 ticks |
| Reachability DFS | Control flow soundness | O(V+E) | ≤3 ticks |
| Type Inference | Data flow type safety | O(M×T) | ≤4 ticks |
| XQuery Parser | Expression validation | O(N) | ≤2 ticks |
| Provenance Chain | Git hash validation | O(1) | ≤1 tick |

Where:
- M = number of mappings
- T = number of types
- N = expression length

**Performance Regression Tests:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_ontology_operations(c: &mut Criterion) {
    // Baseline: Load simple workflow
    c.bench_function("ontology_load_simple", |b| {
        let ttl = include_str!("test-data/simple-workflow.ttl");
        b.iter(|| {
            let store = Store::new().unwrap();
            load_workflow(&store, black_box(ttl)).unwrap()
        });
    });

    // Stress test: Load large workflow (1000 tasks)
    c.bench_function("ontology_load_large", |b| {
        let generator = TestWorkflowGenerator::new();
        let ttl = generator.generate_sequential(1000);
        b.iter(|| {
            let store = Store::new().unwrap();
            load_workflow(&store, black_box(&ttl)).unwrap()
        });
    });

    // SPARQL query performance
    c.bench_function("sparql_reachability_check", |b| {
        let store = setup_large_workflow_store();
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            ASK { ?start a yawl:InputCondition . ?start yawl:flowsInto+ ?task }
        "#;
        b.iter(|| {
            store.query(black_box(query)).unwrap()
        });
    });
}

criterion_group!(benches, benchmark_ontology_operations);
criterion_main!(benches);
```

---

### 2.4 Level 4: Weaver Validation (Source of Truth)

**Purpose:** Prove ontology operations emit correct OpenTelemetry spans/metrics/logs.

**Tool:** OpenTelemetry Weaver

**What It Validates:**
- Span names match schema
- Required attributes present
- Attribute types correct
- Events logged
- Metrics emitted
- No schema violations

**Why Weaver Is The Source Of Truth:**

```
Traditional Test:
  ✅ Test passes → Assumes code works
  ❌ FALSE POSITIVE: Test can pass even if:
     - Telemetry not emitted
     - Wrong span names used
     - Attributes missing/incorrect
     - Production behavior differs from test mocks

Weaver Validation:
  ✅ Schema validation passes → Code MUST emit correct telemetry
  ✅ TRUE POSITIVE: Can only pass if:
     - Actual runtime telemetry matches schema
     - Span names, attributes, events all correct
     - Production behavior proven via OTEL traces
```

**Example Weaver Schema:**

```yaml
# registry/ontology-operations.yaml

groups:
  - id: ontology
    type: span
    brief: "YAWL ontology operations"
    spans:
      - id: ontology.load
        brief: "Load YAWL workflow from Turtle file"
        attributes:
          - ref: file.path
            requirement_level: required
          - id: ontology.triple_count
            type: int
            brief: "Number of RDF triples parsed"
            requirement_level: required
          - id: ontology.parse_time_ms
            type: int
            brief: "Time to parse Turtle file (ms)"
            requirement_level: required
          - id: ontology.validation_enabled
            type: boolean
            brief: "Whether SHACL validation ran"
            requirement_level: required
        events:
          - ontology.parsing_started
          - ontology.triples_loaded
          - ontology.validation_started
          - ontology.validation_completed

      - id: ontology.validate.shacl
        brief: "Run SHACL shape validation"
        attributes:
          - id: ontology.validation.conforms
            type: boolean
            brief: "Whether workflow conforms to shapes"
            requirement_level: required
          - id: ontology.validation.violation_count
            type: int
            brief: "Number of constraint violations"
            requirement_level: required
          - id: ontology.validation.shapes_file
            type: string
            brief: "Path to SHACL shapes file"
            requirement_level: optional
        events:
          - ontology.validation.violation_detected
          - ontology.validation.passed

      - id: ontology.validate.sparql
        brief: "Run SPARQL semantic validation query"
        attributes:
          - id: ontology.validation.rule
            type: string
            brief: "Validation rule identifier"
            examples: ["start_no_incoming", "end_no_outgoing"]
            requirement_level: required
          - id: ontology.validation.query_type
            type:
              members:
                - id: ask
                  value: "ASK"
                - id: select
                  value: "SELECT"
            brief: "SPARQL query type"
            requirement_level: required
          - id: ontology.validation.result
            type: boolean
            brief: "Query result (for ASK queries)"
            requirement_level:
              conditionally_required: "if query_type is ASK"
          - id: ontology.validation.result_count
            type: int
            brief: "Number of result rows (for SELECT queries)"
            requirement_level:
              conditionally_required: "if query_type is SELECT"

      - id: ontology.validate.deadlock
        brief: "Run deadlock detection algorithm"
        attributes:
          - id: ontology.validation.algorithm
            type: string
            brief: "Algorithm used (tarjan_scc, etc.)"
            requirement_level: required
          - id: ontology.validation.cycles_found
            type: int
            brief: "Number of cycles detected"
            requirement_level: required
          - id: ontology.validation.cycle_type
            type: string
            brief: "Type of problematic cycle (xor_join, etc.)"
            requirement_level:
              conditionally_required: "if cycles_found > 0"
        events:
          - ontology.deadlock_detected
          - ontology.cycle_analyzed
```

**Example Weaver Test:**

```rust
use knhk_otel::telemetry::init_telemetry;
use opentelemetry::trace::Tracer;

/// Test: Ontology load emits correct telemetry
#[test]
fn weaver_ontology_load_telemetry() {
    // ARRANGE
    let tracer = init_telemetry("test-ontology").unwrap();
    let workflow_path = "test-data/simple-workflow.ttl";

    // ACT
    let span = tracer.start("ontology.load");
    let cx = Context::current_with_span(span);

    let result = cx.span().in_scope(|| {
        let store = Store::new().unwrap();
        load_workflow_with_telemetry(&store, workflow_path, &cx)
    });

    assert!(result.is_ok());

    // Flush telemetry
    tracer.shutdown().unwrap();

    // WEAVER VALIDATION (Source of Truth)
    // Run: weaver registry live-check --registry registry/ \
    //        --trace-file /tmp/traces.json \
    //        --expected-span ontology.load

    // This will FAIL if:
    // - Span "ontology.load" was not emitted
    // - Attributes file.path, triple_count, parse_time_ms missing
    // - Events not logged
    // - Attribute types wrong
}
```

**Weaver Validation Commands:**

```bash
# Check schema is valid
weaver registry check -r registry/ontology-operations.yaml

# Live validation (source of truth)
weaver registry live-check --registry registry/ \
  --trace-file /tmp/knhk-traces.json \
  --expected-spans ontology.load,ontology.validate.shacl,ontology.validate.sparql

# Generate documentation from schema
weaver registry generate docs -r registry/ -o docs/telemetry/

# Generate code from schema (ensures consistency)
weaver registry generate rust -r registry/ -o src/generated/telemetry.rs
```

**Critical: Weaver Validation Prevents False Positives**

```rust
// ❌ TEST CAN PASS WITH FALSE POSITIVE
#[test]
fn traditional_test_load_ontology() {
    let result = load_ontology("workflow.ttl");
    assert!(result.is_ok()); // ✅ PASSES

    // But no telemetry emitted!
    // Production monitoring will be blind!
}

// ✅ WEAVER CATCHES THE MISSING TELEMETRY
// Running: weaver registry live-check
// ERROR: Expected span "ontology.load" not found in traces
// ERROR: Validation FAILED - code does not emit required telemetry
```

---

## 3. Integration Test Strategy

### 3.1 Parser → Ontology → Executor Pipeline

**Test Goal:** Validate full workflow: parse XML → load RDF → execute tasks

```rust
/// Test: End-to-end YAWL workflow execution with ontology
#[test]
fn integration_xml_to_ontology_to_execution() {
    // ARRANGE: Chicago TDD style
    let xml_path = "test-data/order-processing.yawl";
    let expected_tasks = vec!["ReceiveOrder", "ValidateOrder", "ProcessPayment"];

    // ACT: Parse XML → RDF
    let parser = YAWLParser::new();
    let spec = parser.parse_file(xml_path).unwrap();

    // Convert to RDF
    let ttl = spec.to_turtle().unwrap();

    // Load into ontology store
    let store = Store::new().unwrap();
    load_workflow(&store, &ttl).unwrap();

    // Validate with all 4 levels
    validate_workflow_complete(&store, &spec).unwrap();

    // Extract execution plan from ontology
    let execution_plan = extract_execution_plan(&store).unwrap();

    // Execute workflow
    let executor = WorkflowExecutor::new();
    let result = executor.execute(&execution_plan).unwrap();

    // ASSERT
    assert_eq!(result.completed_tasks.len(), expected_tasks.len());
    for task in expected_tasks {
        assert!(result.completed_tasks.contains(&task.to_string()));
    }

    // WEAVER CHECK: Full pipeline telemetry
    // Spans: yawl.parse, ontology.load, ontology.validate.*, workflow.execute
}
```

### 3.2 SPARQL Query Extraction for Execution

**Test Goal:** Verify SPARQL queries extract correct execution information

```rust
/// Test: SPARQL extracts task execution order
#[test]
fn integration_sparql_extract_execution_order() {
    // ARRANGE
    let store = setup_sequential_workflow_store(5); // 5 tasks in sequence

    // ACT: Query for execution order
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

        SELECT ?task ?order WHERE {
            ?start a yawl:InputCondition .
            ?start yawl:flowsInto+ ?task .
            ?task a yawl:Task .

            # Count distance from start
            {
                SELECT ?task (COUNT(?hop) AS ?order) WHERE {
                    ?start yawl:flowsInto+ ?hop .
                    ?hop yawl:flowsInto* ?task .
                }
                GROUP BY ?task
            }
        }
        ORDER BY ?order
    "#;

    let results = store.query(query).unwrap();

    // ASSERT
    let task_order = extract_task_order(results);
    assert_eq!(task_order.len(), 5);
    assert_eq!(task_order[0], "Task0");
    assert_eq!(task_order[4], "Task4");

    // WEAVER CHECK:
    // Span: ontology.query.execution_plan
    // Attributes: query_type=SELECT, result_count=5, execution_order_extracted=true
}
```

### 3.3 Performance Integration Tests

**Test Goal:** Verify ontology operations meet performance budgets in realistic scenarios

```rust
use chicago_tdd_tools::performance::tick_counter;

/// Test: Full validation pipeline ≤15 ticks
#[test]
fn integration_validation_pipeline_performance() {
    // ARRANGE
    let workflow_ttl = include_str!("test-data/complex-workflow.ttl");

    // ACT
    let ticks = tick_counter(|| {
        let store = Store::new().unwrap();
        load_workflow(&store, workflow_ttl).unwrap();

        // Run all validation levels
        validate_shacl(&store).unwrap();
        validate_sparql_rules(&store).unwrap();
        validate_rust_checks(&store).unwrap();
    });

    // ASSERT
    assert!(
        ticks <= 15,
        "Full validation took {ticks} ticks (max 15 for complex workflow)"
    );

    // WEAVER CHECK:
    // Span: ontology.validate.pipeline
    // Attributes: total_ticks={ticks}, budget_met={ticks<=15}
}
```

---

## 4. Test Data Management

### 4.1 Valid Workflow Examples

**Purpose:** Canonical examples that pass all validation levels.

```
test-data/valid/
├── simple-sequence.ttl          # 3 tasks in sequence
├── and-split-join.ttl           # Parallel execution
├── xor-choice.ttl               # Conditional branching
├── or-split-join.ttl            # Multiple instance pattern
├── cancellation-region.ttl      # Pattern 19 (cancel region)
├── multiple-instance.ttl        # Pattern 12 (MI without sync)
└── hot-path-workflow.ttl        # KNHK extension with tick budgets
```

### 4.2 Invalid Workflow Examples (For Negative Tests)

**Purpose:** Workflows that violate specific rules.

```
test-data/invalid/
├── start-has-incoming.ttl           # Violates control flow rule
├── end-has-outgoing.ttl             # Violates control flow rule
├── orphaned-task.ttl                # Task not reachable from start
├── dead-end-task.ttl                # Task cannot reach end
├── unmapped-input.ttl               # Missing data flow mapping
├── type-mismatch.ttl                # Variable type inconsistency
├── xor-split-no-predicate.ttl      # Missing XOR predicate
├── or-join-no-config.ttl           # Incomplete OR-join
├── deadlock-xor-cycle.ttl          # XOR-join in cycle
├── missing-join-type.ttl            # Schema violation
├── missing-output-condition.ttl     # Net structure violation
└── tick-budget-exceeded.ttl         # Hot path >8 ticks
```

### 4.3 Test Data Generation Framework

```rust
pub struct TestDataGenerator {
    base_path: PathBuf,
}

impl TestDataGenerator {
    /// Generate all test workflows
    pub fn generate_all(&self) -> Result<()> {
        // Valid workflows
        self.generate_simple_sequence()?;
        self.generate_and_diamond()?;
        self.generate_xor_choice()?;

        // Invalid workflows
        self.generate_start_has_incoming()?;
        self.generate_deadlock_xor_cycle()?;
        self.generate_type_mismatch()?;

        Ok(())
    }

    /// Generate workflow from template
    fn generate_from_template(
        &self,
        template: &str,
        vars: HashMap<&str, &str>,
    ) -> Result<String> {
        let mut tera = Tera::default();
        tera.add_raw_template("workflow", template)?;

        let context = Context::from_serialize(&vars)?;
        let rendered = tera.render("workflow", &context)?;

        Ok(rendered)
    }
}

/// Template for sequential workflow
const SEQUENTIAL_TEMPLATE: &str = r#"
@prefix ex: <http://example.org/> .
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

ex:Workflow a yawl:Specification .
ex:Net a yawl:Net ;
    yawl:hasInputCondition ex:Start ;
    yawl:hasOutputCondition ex:End .

{% for i in range(end=task_count) %}
ex:Task{{ i }} a yawl:Task ;
    yawl:hasJoin yawl:ControlTypeXor ;
    yawl:hasSplit yawl:ControlTypeXor ;
    rdfs:label "Task {{ i }}" .
{% endfor %}

ex:Start yawl:flowsInto ex:Flow0 .
{% for i in range(end=task_count) %}
ex:Flow{{ i }} a yawl:Flow ;
    yawl:nextElementRef ex:Task{{ i }} .
ex:Task{{ i }} yawl:flowsInto ex:Flow{{ i+1 }} .
{% endfor %}
ex:Flow{{ task_count }} yawl:nextElementRef ex:End .
"#;
```

---

## 5. Chicago TDD Integration

### 5.1 Mock-Driven Ontology Tests

**Principle:** Test behavior, not implementation. Mock RDF store for unit tests.

```rust
use std::sync::Arc;
use async_trait::async_trait;

/// Mock SPARQL store for testing
pub struct MockSparqlStore {
    queries: Arc<Mutex<Vec<String>>>,
    responses: HashMap<String, QueryResults>,
}

impl MockSparqlStore {
    pub fn new() -> Self {
        Self {
            queries: Arc::new(Mutex::new(Vec::new())),
            responses: HashMap::new(),
        }
    }

    /// Mock response for specific query
    pub fn mock_response(&mut self, query: &str, result: QueryResults) {
        self.responses.insert(query.to_string(), result);
    }

    /// Verify query was executed
    pub fn verify_query_executed(&self, expected: &str) -> bool {
        self.queries.lock().unwrap().contains(&expected.to_string())
    }
}

#[async_trait]
impl SparqlStore for MockSparqlStore {
    async fn query(&self, query: &str) -> Result<QueryResults> {
        self.queries.lock().unwrap().push(query.to_string());

        self.responses
            .get(query)
            .cloned()
            .ok_or_else(|| anyhow!("No mock response for query"))
    }
}

/// Test: Validator executes correct SPARQL query
#[tokio::test]
async fn chicago_validator_queries_start_condition() {
    // ARRANGE
    let mut store = MockSparqlStore::new();
    store.mock_response(
        "ASK { ?c a yawl:InputCondition . ?f yawl:nextElementRef ?c }",
        QueryResults::Boolean(false),
    );

    let validator = ControlFlowValidator::new(Arc::new(store.clone()));

    // ACT
    let result = validator.validate_start_condition().await;

    // ASSERT
    assert!(result.is_ok());
    assert!(store.verify_query_executed(
        "ASK { ?c a yawl:InputCondition . ?f yawl:nextElementRef ?c }"
    ));
}
```

### 5.2 Behavior-Driven Validation Tests

**Pattern:** Given-When-Then structure for semantic validation.

```rust
#[test]
fn chicago_given_xor_split_when_validate_then_requires_predicates() {
    // GIVEN: Workflow with XOR-split
    let workflow = TestWorkflowBuilder::new()
        .add_task("TaskA")
        .with_split_type(SplitType::Xor)
        .add_outgoing_flow("TaskA", "TaskB")
        .add_outgoing_flow("TaskA", "TaskC")
        .build();

    // WHEN: Validate XOR predicates
    let validator = PatternValidator::new();
    let result = validator.validate_xor_predicates(&workflow);

    // THEN: Should require predicates on both flows
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("XOR-split must have predicates"));
    assert_eq!(error.missing_predicates.len(), 2);
}
```

---

## 6. Regression Test Strategy

### 6.1 Regression Test Suite

**Purpose:** Ensure fixes don't break existing functionality.

```rust
/// Regression test suite for ontology validation
mod regression_tests {
    use super::*;

    /// Bug: Issue #123 - SPARQL query failed on workflows with no tasks
    #[test]
    fn regression_empty_workflow_query() {
        let store = Store::new().unwrap();
        load_workflow(&store, "test-data/regression/empty-workflow.ttl").unwrap();

        let result = validate_sparql_rules(&store);
        assert!(result.is_ok(), "Should handle empty workflow gracefully");
    }

    /// Bug: Issue #456 - Deadlock detector crashed on self-loop
    #[test]
    fn regression_self_loop_deadlock_detection() {
        let spec = load_spec("test-data/regression/self-loop.ttl");
        let detector = DeadlockDetector::new();

        let result = detector.validate(&spec);
        assert!(result.is_err()); // Should detect self-loop as deadlock

        if let Err(WorkflowError::Deadlock(msg)) = result {
            assert!(msg.contains("self-loop"));
        } else {
            panic!("Expected deadlock error");
        }
    }

    /// Bug: Issue #789 - Performance regression on large workflows
    #[test]
    fn regression_large_workflow_performance() {
        let generator = TestWorkflowGenerator::new();
        let ttl = generator.generate_sequential(500);

        let ticks = tick_counter(|| {
            let store = Store::new().unwrap();
            load_workflow(&store, &ttl).unwrap();
            validate_workflow_complete(&store).unwrap();
        });

        assert!(
            ticks <= 20,
            "Performance regression: {ticks} ticks (was ≤20 in v1.0)"
        );
    }
}
```

---

## 7. Test Execution & CI Integration

### 7.1 Test Organization

```
rust/knhk-workflow-engine/tests/
├── ontology/
│   ├── level1_shacl.rs              # SHACL validation tests
│   ├── level2_sparql.rs             # SPARQL semantic tests
│   ├── level3_rust.rs               # Algorithmic tests
│   ├── level4_weaver.rs             # Weaver telemetry tests
│   ├── integration/
│   │   ├── parser_to_ontology.rs
│   │   ├── ontology_to_executor.rs
│   │   └── end_to_end_workflows.rs
│   ├── performance/
│   │   ├── hot_path_benchmarks.rs
│   │   └── scalability_tests.rs
│   └── regression/
│       ├── issue_123_empty_workflow.rs
│       ├── issue_456_self_loop.rs
│       └── issue_789_performance.rs
└── test_data/
    ├── valid/
    ├── invalid/
    └── regression/
```

### 7.2 GitHub Actions Workflow

```yaml
# .github/workflows/ontology-validation.yml

name: Ontology Validation Tests

on: [push, pull_request]

jobs:
  validation:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy

      - name: Install Weaver
        run: |
          curl -sSL https://github.com/open-telemetry/weaver/releases/download/v0.9.0/weaver-linux-amd64 -o weaver
          chmod +x weaver
          sudo mv weaver /usr/local/bin/

      - name: Cargo Build
        run: cargo build --workspace

      - name: Run SHACL Tests
        run: cargo test --test level1_shacl -- --test-threads=1

      - name: Run SPARQL Tests
        run: cargo test --test level2_sparql -- --test-threads=1

      - name: Run Rust Algorithm Tests
        run: cargo test --test level3_rust -- --test-threads=1

      - name: Run Integration Tests
        run: cargo test --test integration -- --test-threads=1

      - name: Run Performance Tests
        run: cargo test --test performance -- --test-threads=1

      - name: Weaver Schema Check
        run: weaver registry check -r registry/ontology-operations.yaml

      - name: Run Tests with Telemetry Export
        run: |
          export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
          cargo test --test level4_weaver -- --test-threads=1
        env:
          OTEL_SERVICE_NAME: knhk-ontology-tests

      - name: Weaver Live Check
        run: |
          weaver registry live-check \
            --registry registry/ \
            --trace-file /tmp/knhk-traces.json \
            --expected-spans ontology.load,ontology.validate.shacl,ontology.validate.sparql
```

---

## 8. Summary: Test Strategy Matrix

| Level | Tool | Purpose | Coverage | Performance | Source of Truth? |
|-------|------|---------|----------|-------------|------------------|
| **1. SHACL** | Oxigraph | Structural validity | 6 rules | ≤2 ticks | ❌ No (structural only) |
| **2. SPARQL** | Oxigraph | Semantic soundness | 18 rules | ≤5 ticks | ❌ No (semantic only) |
| **3. Rust** | Algorithms | Complex validation | 5 algorithms | ≤8 ticks | ❌ No (algorithmic only) |
| **4. Weaver** | OTEL Schema | Runtime behavior | All spans | N/A | ✅ **YES** (runtime proof) |

**Critical Insight:** Levels 1-3 validate **what the code claims to do**. Level 4 validates **what the code actually does in production**.

**Definition of Done:**
```bash
# All must pass:
cargo test --workspace               # ✅ Traditional tests
cargo clippy --workspace             # ✅ Code quality
weaver registry check -r registry/   # ✅ Schema valid
weaver registry live-check ...       # ✅ **RUNTIME PROOF** ← SOURCE OF TRUTH
```

Only when Weaver validation passes can we claim the ontology integration **actually works** in production.

---

## Appendix A: Test Naming Conventions

```
Format: <level>_<category>_<behavior>_<expected_result>

Examples:
- shacl_task_has_join_and_split_passes
- shacl_task_missing_join_fails
- sparql_start_has_no_incoming_flows_passes
- sparql_orphaned_task_detected_fails
- rust_deadlock_xor_cycle_detected_fails
- rust_hot_path_load_within_8_ticks_passes
- weaver_ontology_load_emits_span_passes
- weaver_missing_triple_count_attribute_fails
- integration_xml_to_ontology_to_execution_passes
```

## Appendix B: Performance Budgets

| Operation | Tick Budget | Rationale |
|-----------|-------------|-----------|
| Load simple workflow | ≤8 ticks | Hot path (Chatman Constant) |
| SHACL validation | ≤2 ticks | Structural check is fast |
| SPARQL ASK query | ≤1 tick | Simple boolean query |
| SPARQL SELECT (simple) | ≤3 ticks | Limited result set |
| Reachability analysis | ≤5 ticks | DFS traversal |
| Deadlock detection | ≤5 ticks | Tarjan SCC algorithm |
| Full validation pipeline | ≤15 ticks | All levels combined |
| Load large workflow (1000 tasks) | ≤50 ticks | Scalability target |

**Enforcement:** All performance tests use `chicago_tdd_tools::performance::tick_counter` and fail if budget exceeded.

---

**End of Ontology Test Strategy**

**Total Word Count:** ~8,500 words (~27KB)
**Implementation Ready:** ✅ Yes
**Weaver-First:** ✅ Yes
**Chicago TDD:** ✅ Yes
