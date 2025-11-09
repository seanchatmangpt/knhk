# RDF Workflow Execution Architecture

**Status:** Approved
**Version:** 1.0
**Date:** 2025-11-08
**Author:** System Architect
**Validation:** OpenTelemetry Weaver Required

## Executive Summary

This document defines the end-to-end architecture for loading, parsing, validating, and executing YAWL workflows defined in Turtle RDF format with comprehensive OpenTelemetry observability. The architecture enables KNHK to execute real-world financial workflows (ATM transactions, SWIFT payments, Payroll) with all 43 Van der Aalst patterns while maintaining ≤8 tick hot-path performance (Chatman Constant) and Fortune 5 enterprise requirements.

### Key Capabilities

- **RDF-Native Execution:** Load `.ttl` workflows directly into engine
- **Pattern Execution:** Execute all 43 Van der Aalst workflow patterns
- **Schema Validation:** SHACL soundness validation before execution
- **Telemetry-First:** Every operation emits OTEL spans/metrics/logs
- **State Persistence:** Event-sourced case management with Sled
- **Performance:** ≤8 ticks for hot path operations
- **Fortune 5:** Multi-region, SPIFFE, KMS, SLO enforcement

### Success Criteria (Weaver Validation Required)

**ALL success criteria MUST be validated by OpenTelemetry Weaver:**

```bash
# ✅ MANDATORY: Weaver validation is the ONLY source of truth
weaver registry check -r registry/                    # Schema is valid
weaver registry live-check --registry registry/       # Runtime telemetry conforms

# ❌ INSUFFICIENT: These can produce false positives
cargo test              # Tests can pass with broken features
validation agents       # Agents can hallucinate validation
--help commands         # Help text ≠ working feature
```

**Acceptance Criteria:**
1. ✅ Load ATM workflow from `/ontology/workflows/financial/atm_transaction.ttl`
2. ✅ Execute workflow end-to-end (card verification → cash dispensing → receipt)
3. ✅ All pattern executions emit proper OTEL spans
4. ✅ Weaver validates telemetry matches schema
5. ✅ Case state persisted to Sled
6. ✅ Performance ≤8 ticks for hot path
7. ✅ SPARQL queries work against runtime state

---

## 1. Architecture Overview

### 1.1 System Context (C4 Level 1)

```
┌─────────────────────────────────────────────────────────────────────┐
│                          KNHK Workflow Engine                        │
│                                                                       │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐          │
│  │   Workflow   │───>│  RDF Parser  │───>│   Pattern    │          │
│  │   (.ttl)     │    │  (Oxigraph)  │    │   Executor   │          │
│  └──────────────┘    └──────────────┘    └──────────────┘          │
│                              │                    │                  │
│                              v                    v                  │
│                      ┌──────────────┐    ┌──────────────┐          │
│                      │    State     │<───│     OTEL     │          │
│                      │   Manager    │    │  Telemetry   │          │
│                      └──────────────┘    └──────────────┘          │
│                              │                    │                  │
│                              v                    v                  │
│                      ┌──────────────┐    ┌──────────────┐          │
│                      │     Sled     │    │    Weaver    │          │
│                      │   Database   │    │  Validator   │          │
│                      └──────────────┘    └──────────────┘          │
└─────────────────────────────────────────────────────────────────────┘
         │                    │                    │
         v                    v                    v
  ┌──────────┐        ┌──────────┐        ┌──────────┐
  │   REST   │        │  gRPC    │        │  SPARQL  │
  │   API    │        │  Server  │        │  Queries │
  └──────────┘        └──────────┘        └──────────┘
```

**External Dependencies:**
- **OpenTelemetry Weaver:** Schema validation (source of truth)
- **Oxigraph:** RDF store and SPARQL engine
- **Sled:** Persistent state storage
- **OTLP Collector:** Telemetry aggregation

### 1.2 Container Diagram (C4 Level 2)

```
┌───────────────────────────────────────────────────────────────────────┐
│                      Workflow Engine Container                        │
│                                                                        │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                      RDF Workflow Loader                        │ │
│  │  - Load .ttl files                                             │ │
│  │  - Parse YAWL ontology                                         │ │
│  │  - SHACL validation                                            │ │
│  │  - Convert to WorkflowSpec                                     │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                              ↓                                         │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                      WorkflowEngine                             │ │
│  │  - register_workflow(turtle)                                   │ │
│  │  - create_case(spec_id, data)                                  │ │
│  │  - execute_case(case_id)                                       │ │
│  │  - query_rdf(sparql)                                           │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│              ↓                ↓                ↓                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │   Pattern    │  │    State     │  │     OTEL     │               │
│  │   Registry   │  │   Manager    │  │ Integration  │               │
│  │  (43 execs)  │  │ (Sled+Cache) │  │ (Spans/Logs) │               │
│  └──────────────┘  └──────────────┘  └──────────────┘               │
│                                                                        │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                      RDF Stores (Oxigraph)                      │ │
│  │  - spec_rdf_store: Workflow specifications                     │ │
│  │  - pattern_metadata_store: Pattern metadata (43 patterns)      │ │
│  │  - case_rdf_stores: Runtime state (per case)                   │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────┘
```

---

## 2. Component Architecture (C4 Level 3)

### 2.1 RDF Workflow Loader Component

**Responsibility:** Parse Turtle RDF workflows and convert to internal `WorkflowSpec` representation.

```rust
/// RDF Workflow Loader
pub struct RdfWorkflowLoader {
    /// Oxigraph RDF store for parsing
    parser_store: Store,
    /// YAWL ontology (loaded once at startup)
    yawl_ontology: Store,
    /// SHACL validator
    shacl_validator: ShaclValidator,
}

impl RdfWorkflowLoader {
    /// Load workflow from Turtle file
    pub async fn load_from_file(&mut self, path: &Path) -> WorkflowResult<WorkflowSpec> {
        // 1. Read .ttl file
        let turtle = fs::read_to_string(path)?;

        // 2. Parse into RDF store
        self.parser_store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())?;

        // 3. SHACL validation (soundness check)
        self.shacl_validator.validate(&self.parser_store)?;

        // 4. Extract WorkflowSpec via SPARQL queries
        let spec = self.extract_workflow_spec()?;

        // 5. Store source turtle for runtime queries
        spec.source_turtle = Some(turtle);

        // 6. Deadlock detection
        DeadlockDetector::validate(&spec)?;

        Ok(spec)
    }

    /// Extract WorkflowSpec from RDF store using SPARQL
    fn extract_workflow_spec(&self) -> WorkflowResult<WorkflowSpec> {
        // Query for specification metadata
        let spec_query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            SELECT ?spec ?name ?version ?input ?output WHERE {
                ?spec a yawl:Specification ;
                      yawl:specName ?name ;
                      yawl:version ?version ;
                      yawl:hasInputCondition ?input ;
                      yawl:hasOutputCondition ?output .
            }
        "#;

        // Query for tasks
        let task_query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            SELECT ?task ?name ?join ?split WHERE {
                ?spec yawl:hasTask ?task .
                ?task yawl:taskName ?name ;
                      yawl:join ?join ;
                      yawl:split ?split .
            }
        "#;

        // Query for flows
        let flow_query = r#"
            PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
            SELECT ?flow ?from ?to ?predicate WHERE {
                ?spec yawl:hasFlow ?flow .
                ?flow yawl:flowsFrom ?from ;
                      yawl:flowsInto ?to .
                OPTIONAL { ?flow yawl:predicate ?predicate }
            }
        "#;

        // Execute queries and build WorkflowSpec
        let spec = self.build_spec_from_queries(spec_query, task_query, flow_query)?;

        Ok(spec)
    }
}
```

**SPARQL Queries for Extraction:**

| Query Purpose | Extracts | Populates |
|--------------|----------|-----------|
| Specification metadata | `yawl:Specification` | `spec.id`, `spec.name` |
| Tasks | `yawl:AtomicTask`, `yawl:CompositeTask` | `spec.tasks` |
| Conditions | `yawl:InputCondition`, `yawl:OutputCondition` | `spec.conditions` |
| Flows | `yawl:Flow`, `yawl:flowsFrom`, `yawl:flowsInto` | Task connections |
| Parameters | `yawl:hasInputParameter`, `yawl:hasOutputParameter` | Task I/O |

### 2.2 WorkflowEngine Integration Component

**Responsibility:** Manage workflow registration, case lifecycle, and pattern execution.

```rust
impl WorkflowEngine {
    /// Register workflow from Turtle RDF
    pub async fn register_workflow_from_rdf(
        &mut self,
        turtle: &str
    ) -> WorkflowResult<WorkflowSpecId> {
        let span = info_span!("register_workflow_from_rdf");
        let _enter = span.enter();

        // 1. Parse Turtle into WorkflowSpec
        let mut loader = RdfWorkflowLoader::new()?;
        let spec = loader.load_from_string(turtle)?;

        // 2. Load into RDF store for runtime queries
        self.load_spec_rdf(turtle).await?;

        // 3. Register with engine
        let spec_id = spec.id;
        self.specs.insert(spec_id, spec);

        // 4. Save to state manager
        self.state_manager.save_spec(&spec).await?;

        info!(spec_id = %spec_id, "Workflow registered from RDF");
        Ok(spec_id)
    }

    /// Execute workflow case with full telemetry
    pub async fn execute_case_with_telemetry(
        &mut self,
        case_id: CaseId
    ) -> WorkflowResult<()> {
        let span = info_span!("execute_case", case_id = %case_id);
        let _enter = span.enter();

        // 1. Load case from state
        let case = self.cases.get(&case_id)
            .ok_or(WorkflowError::CaseNotFound(case_id))?
            .clone();

        // 2. Load workflow spec
        let spec = self.specs.get(&case.spec_id)
            .ok_or(WorkflowError::SpecNotFound(case.spec_id))?
            .clone();

        // 3. Execute all enabled tasks
        for task_id in &case.enabled_tasks {
            let task = spec.tasks.get(task_id)
                .ok_or(WorkflowError::TaskNotFound(task_id.clone()))?;

            // 4. Execute task with pattern
            let result = self.execute_task_with_pattern(&case_id, task).await?;

            // 5. Update case state based on result
            self.update_case_state(case_id, &result).await?;
        }

        info!("Case execution complete");
        Ok(())
    }

    /// Execute task with pattern and telemetry
    async fn execute_task_with_pattern(
        &mut self,
        case_id: &CaseId,
        task: &Task
    ) -> WorkflowResult<PatternExecutionResult> {
        let span = info_span!("execute_task_with_pattern",
            case_id = %case_id,
            task_id = %task.id
        );
        let _enter = span.enter();

        // 1. Determine pattern from task metadata
        let pattern_id = self.identify_pattern(task)?;

        // 2. Get pattern executor
        let executor = self.pattern_registry.get(&pattern_id)
            .ok_or(WorkflowError::PatternNotFound(pattern_id))?;

        // 3. Build execution context
        let context = PatternExecutionContext {
            task_id: task.id.clone(),
            case_id: *case_id,
            input_data: self.get_case_data(case_id).await?,
            current_tick: self.timer_service.current_tick(),
        };

        // 4. Execute pattern (emits OTEL spans)
        let result = executor.execute(context).await?;

        // 5. Validate result
        if result.ticks_elapsed > 8 {
            warn!(
                ticks = result.ticks_elapsed,
                "Hot path violated Chatman Constant (>8 ticks)"
            );
        }

        info!(
            pattern_id = %pattern_id,
            ticks = result.ticks_elapsed,
            "Pattern executed successfully"
        );

        Ok(result)
    }
}
```

### 2.3 State Management Component

**Responsibility:** Event-sourced state persistence with Sled + RDF runtime state.

```rust
/// State manager with dual persistence
impl StateManager {
    /// Save case state (both Sled + RDF)
    pub async fn save_case_with_rdf(
        &self,
        case: &Case,
        engine: &WorkflowEngine
    ) -> WorkflowResult<()> {
        let span = info_span!("save_case_with_rdf", case_id = %case.id);
        let _enter = span.enter();

        // 1. Save to Sled (event sourcing)
        self.save_case(case).await?;

        // 2. Update case RDF store
        let case_store = engine.case_rdf_stores.read().await
            .get(&case.id)
            .ok_or(WorkflowError::RdfStoreNotFound)?
            .clone();

        // 3. Serialize case state to RDF
        let turtle = self.serialize_case_to_rdf(case)?;

        // 4. Load into case-specific RDF store
        case_store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())?;

        info!("Case state saved to Sled + RDF");
        Ok(())
    }

    /// Serialize case to RDF for SPARQL queries
    fn serialize_case_to_rdf(&self, case: &Case) -> WorkflowResult<String> {
        let mut turtle = String::new();

        turtle.push_str(&format!(r#"
@prefix case: <http://bitflow.ai/case/{}#> .
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .

case:self a yawl:Case ;
    yawl:caseId "{}" ;
    yawl:specId "{}" ;
    yawl:state "{}" ;
    yawl:createdAt "{}" .
"#,
            case.id,
            case.id,
            case.spec_id,
            case.state,
            case.created_at
        ));

        // Serialize enabled tasks
        for task_id in &case.enabled_tasks {
            turtle.push_str(&format!(
                "case:self yawl:enabledTask <{}> .\n",
                task_id
            ));
        }

        // Serialize case data as RDF
        turtle.push_str(&self.serialize_json_to_rdf(&case.data)?);

        Ok(turtle)
    }
}
```

### 2.4 Telemetry Integration Component

**Responsibility:** Emit OTEL spans/metrics/logs that conform to Weaver schema.

```rust
/// OTEL integration for workflow execution
pub struct OtelIntegration {
    tracer: Tracer,
    meter: Meter,
}

impl OtelIntegration {
    /// Record workflow execution span
    pub fn record_workflow_execution(
        &self,
        spec_id: WorkflowSpecId,
        case_id: CaseId,
        result: &ExecutionResult
    ) {
        let span = self.tracer
            .span_builder("workflow.execution")
            .with_kind(SpanKind::Server)
            .with_attributes(vec![
                KeyValue::new("workflow.spec_id", spec_id.to_string()),
                KeyValue::new("workflow.case_id", case_id.to_string()),
                KeyValue::new("workflow.status", result.status.to_string()),
                KeyValue::new("workflow.ticks", result.ticks_elapsed as i64),
            ])
            .start(&self.tracer);

        // Emit metrics
        self.meter
            .i64_counter("workflow.executions")
            .with_description("Total workflow executions")
            .init()
            .add(1, &[
                KeyValue::new("status", result.status.to_string()),
            ]);

        if result.ticks_elapsed > 8 {
            self.meter
                .i64_counter("workflow.chatman_violations")
                .with_description("Hot path > 8 ticks")
                .init()
                .add(1, &[]);
        }
    }

    /// Record pattern execution span
    pub fn record_pattern_execution(
        &self,
        pattern_id: PatternId,
        context: &PatternExecutionContext,
        result: &PatternExecutionResult
    ) {
        self.tracer
            .span_builder("pattern.execution")
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("pattern.id", pattern_id.to_string()),
                KeyValue::new("pattern.task_id", context.task_id.clone()),
                KeyValue::new("pattern.case_id", context.case_id.to_string()),
                KeyValue::new("pattern.ticks", result.ticks_elapsed as i64),
                KeyValue::new("pattern.success", result.success),
            ])
            .start(&self.tracer);
    }
}
```

---

## 3. Sequence Diagrams

### 3.1 Workflow Loading Sequence

```
┌──────┐     ┌─────────┐     ┌──────────┐     ┌──────────┐     ┌──────┐
│ User │     │  Engine │     │  Loader  │     │ Oxigraph │     │ Sled │
└──┬───┘     └────┬────┘     └────┬─────┘     └────┬─────┘     └──┬───┘
   │              │                │                │              │
   │ POST /workflow/register       │                │              │
   ├─────────────>│                │                │              │
   │              │                │                │              │
   │              │ load_from_file(atm.ttl)         │              │
   │              ├───────────────>│                │              │
   │              │                │                │              │
   │              │                │ parse Turtle   │              │
   │              │                ├───────────────>│              │
   │              │                │                │              │
   │              │                │ SHACL validate │              │
   │              │                ├───────────────>│              │
   │              │                │<───────────────┤              │
   │              │                │   ✅ Valid     │              │
   │              │                │                │              │
   │              │                │ SPARQL queries │              │
   │              │                ├───────────────>│              │
   │              │                │<───────────────┤              │
   │              │                │  Tasks, Flows  │              │
   │              │                │                │              │
   │              │<───────────────┤                │              │
   │              │  WorkflowSpec  │                │              │
   │              │                │                │              │
   │              │ load_spec_rdf(turtle)           │              │
   │              ├────────────────────────────────>│              │
   │              │                │                │              │
   │              │ save_spec(spec)                 │              │
   │              ├────────────────────────────────────────────────>│
   │              │                │                │              │
   │<─────────────┤                │                │              │
   │ 200 OK       │                │                │              │
   │ {spec_id}    │                │                │              │
   │              │                │                │              │
```

### 3.2 Case Execution Sequence

```
┌──────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────┐  ┌────────┐
│ User │  │ Engine  │  │ Pattern │  │  State  │  │ OTEL │  │ Weaver │
└──┬───┘  └────┬────┘  └────┬────┘  └────┬────┘  └──┬───┘  └───┬────┘
   │           │             │            │          │          │
   │ POST /case/execute      │            │          │          │
   ├──────────>│             │            │          │          │
   │           │             │            │          │          │
   │           │ start_span("workflow.execution")   │          │
   │           ├─────────────────────────────────────>│         │
   │           │             │            │          │          │
   │           │ get_case(case_id)        │          │          │
   │           ├─────────────────────────>│          │          │
   │           │<────────────────────────┤│          │          │
   │           │        Case             │          │          │
   │           │             │            │          │          │
   │           │ execute_task_with_pattern│          │          │
   │           ├────────────>│            │          │          │
   │           │             │            │          │          │
   │           │             │ start_span("pattern.execution")  │
   │           │             ├─────────────────────────────────>│
   │           │             │            │          │          │
   │           │             │ execute()  │          │          │
   │           │             ├──────┐     │          │          │
   │           │             │      │ Pattern logic │          │
   │           │             │<─────┘     │          │          │
   │           │             │            │          │          │
   │           │             │ end_span(result)      │          │
   │           │             ├──────────────────────>│          │
   │           │             │            │          │          │
   │           │<────────────┤            │          │          │
   │           │  PatternResult           │          │          │
   │           │             │            │          │          │
   │           │ update_case_state(case_id, result)  │          │
   │           ├─────────────────────────>│          │          │
   │           │             │            │          │          │
   │           │             │ save_case_with_rdf()  │          │
   │           │             ├───────────>│          │          │
   │           │             │            │          │          │
   │           │ end_span()               │          │          │
   │           ├─────────────────────────────────────>│         │
   │           │             │            │          │          │
   │           │             │            │  emit telemetry     │
   │           │             │            │          ├─────────>│
   │           │             │            │          │          │
   │           │             │            │          │ validate │
   │           │             │            │          │<────────┤│
   │           │             │            │          │   ✅    │
   │<──────────┤             │            │          │          │
   │ 200 OK    │             │            │          │          │
```

### 3.3 SPARQL Query Sequence

```
┌──────┐     ┌─────────┐     ┌──────────┐     ┌──────────┐
│ User │     │  Engine │     │ Oxigraph │     │  Weaver  │
└──┬───┘     └────┬────┘     └────┬─────┘     └────┬─────┘
   │              │                │                │
   │ POST /sparql │                │                │
   ├─────────────>│                │                │
   │              │                │                │
   │              │ query_rdf(sparql)               │
   │              ├───────────────>│                │
   │              │                │                │
   │              │                │ execute query  │
   │              │                ├─────┐          │
   │              │                │     │ SPARQL   │
   │              │                │<────┘          │
   │              │                │                │
   │              │<───────────────┤                │
   │              │  QueryResults  │                │
   │              │                │                │
   │              │ record_query_span              │
   │              ├───────────────────────────────>│
   │              │                │                │
   │<─────────────┤                │                │
   │ 200 OK       │                │                │
   │ {bindings}   │                │                │
```

---

## 4. Data Flow Architecture

### 4.1 Workflow Registration Data Flow

```
┌─────────────────┐
│  ATM.ttl File   │
│  (Turtle RDF)   │
└────────┬────────┘
         │
         v
┌─────────────────────────┐
│   Oxigraph Parser       │
│  - Load Turtle          │
│  - Build RDF graph      │
└────────┬────────────────┘
         │
         v
┌─────────────────────────┐
│   SHACL Validator       │
│  - Check soundness      │
│  - Validate structure   │
└────────┬────────────────┘
         │
         v (SPARQL)
┌─────────────────────────┐
│   WorkflowSpec Builder  │
│  - Extract tasks        │
│  - Extract flows        │
│  - Build graph          │
└────────┬────────────────┘
         │
         ├──────────────────────────┐
         v                          v
┌─────────────────┐      ┌─────────────────┐
│  spec_rdf_store │      │   StateStore    │
│   (Oxigraph)    │      │     (Sled)      │
│  - Runtime      │      │  - Persistence  │
│    queries      │      │  - Event log    │
└─────────────────┘      └─────────────────┘
```

### 4.2 Case Execution Data Flow

```
┌─────────────────┐
│ WorkflowSpec    │
│ (from Sled)     │
└────────┬────────┘
         │
         v
┌─────────────────────────┐
│   Case Creation         │
│  - Generate case_id     │
│  - Initialize state     │
│  - Create RDF store     │
└────────┬────────────────┘
         │
         v
┌─────────────────────────┐
│   Task Execution Loop   │
│  - Get enabled tasks    │
│  - Execute patterns     │
│  - Update state         │
└────────┬────────────────┘
         │
         ├──────────────────────────┬──────────────────────┐
         v                          v                      v
┌─────────────────┐      ┌─────────────────┐    ┌─────────────────┐
│ case_rdf_store  │      │   StateStore    │    │   OTEL Spans    │
│  (Oxigraph)     │      │     (Sled)      │    │  (Weaver Valid) │
│ - Runtime vars  │      │  - Case state   │    │  - Telemetry    │
└─────────────────┘      └─────────────────┘    └─────────────────┘
```

### 4.3 Pattern Execution Data Flow

```
┌─────────────────┐
│ PatternContext  │
│ - task_id       │
│ - case_id       │
│ - input_data    │
└────────┬────────┘
         │
         v
┌─────────────────────────┐
│   Pattern Executor      │
│  - Pattern 1-43         │
│  - Emit OTEL spans      │
│  - Track ticks          │
└────────┬────────────────┘
         │
         v
┌─────────────────────────┐
│  PatternResult          │
│  - success: bool        │
│  - ticks_elapsed: u32   │
│  - output_data: Value   │
└────────┬────────────────┘
         │
         ├──────────────────┬──────────────────┐
         v                  v                  v
┌─────────────┐  ┌─────────────────┐  ┌─────────────┐
│ Case State  │  │  OTEL Metrics   │  │   Logs      │
│  Updated    │  │  - ticks        │  │ - Events    │
└─────────────┘  └─────────────────┘  └─────────────┘
```

---

## 5. RDF Store Architecture

### 5.1 Three-Tier RDF Store Design

KNHK uses three separate Oxigraph RDF stores for different purposes:

```
┌─────────────────────────────────────────────────────────────────┐
│                        WorkflowEngine                           │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │            spec_rdf_store (Shared)                        │ │
│  │  - All registered workflow specifications                 │ │
│  │  - Immutable after registration                           │ │
│  │  - SPARQL: "Get all tasks in workflow X"                  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │       pattern_metadata_store (Shared)                     │ │
│  │  - Metadata for all 43 Van der Aalst patterns             │ │
│  │  - Pattern dependencies, categories                       │ │
│  │  - SPARQL: "Get pattern by name", "List control patterns" │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │     case_rdf_stores (Per-Case HashMap)                    │ │
│  │  - HashMap<CaseId, Store>                                 │ │
│  │  - Runtime variables for each case                        │ │
│  │  - Mutable during execution                               │ │
│  │  - SPARQL: "Get variable X for case Y"                    │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

**Design Rationale:**

| Store | Mutability | Scope | Query Pattern |
|-------|-----------|-------|---------------|
| `spec_rdf_store` | Immutable | All workflows | "What tasks exist in workflow X?" |
| `pattern_metadata_store` | Immutable | All patterns | "What pattern implements XOR-split?" |
| `case_rdf_stores` | Mutable | Per case | "What is variable `balance` for case Y?" |

### 5.2 RDF Store Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                    Engine Initialization                        │
│                                                                 │
│  1. Create spec_rdf_store (empty)                              │
│  2. Create pattern_metadata_store                              │
│  3. Load pattern metadata (43 patterns)                        │
│  4. Create case_rdf_stores HashMap (empty)                     │
└─────────────────────────────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────────────────────┐
│                  Workflow Registration                          │
│                                                                 │
│  1. Parse Turtle → WorkflowSpec                                │
│  2. Load Turtle into spec_rdf_store                            │
│  3. Store WorkflowSpec in Sled                                 │
└─────────────────────────────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────────────────────┐
│                    Case Creation                                │
│                                                                 │
│  1. Create new Store for case                                  │
│  2. Insert into case_rdf_stores[case_id]                       │
│  3. Load initial case data as RDF                              │
└─────────────────────────────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────────────────────┐
│                  Case Execution                                 │
│                                                                 │
│  1. Query spec_rdf_store for workflow structure                │
│  2. Update case_rdf_stores[case_id] with runtime vars          │
│  3. Query case_rdf_stores[case_id] for task input              │
└─────────────────────────────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────────────────────┐
│                  Case Completion                                │
│                                                                 │
│  1. Export case_rdf_stores[case_id] to Turtle                  │
│  2. Store in Sled for provenance                               │
│  3. Remove from case_rdf_stores (optional cleanup)             │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Performance Architecture

### 6.1 Hot Path Optimization (≤8 Ticks)

**Chatman Constant Compliance:**

```rust
/// Hot path operations MUST complete in ≤8 ticks
const CHATMAN_CONSTANT: u32 = 8;

/// Performance-critical path for task execution
pub async fn execute_hot_path(
    &mut self,
    task: &Task,
    context: &ExecutionContext
) -> WorkflowResult<ExecutionResult> {
    let start_tick = self.timer_service.current_tick();

    // 1. Pre-allocate resources (1 tick)
    let resources = self.resource_allocator.fast_alloc(task)?;

    // 2. Execute pattern (3-5 ticks)
    let result = self.pattern_registry.execute_fast(task, context)?;

    // 3. Update state (2 ticks)
    self.state_manager.fast_update(context.case_id, &result)?;

    let elapsed_ticks = self.timer_service.current_tick() - start_tick;

    // Validation: MUST be ≤8 ticks
    if elapsed_ticks > CHATMAN_CONSTANT {
        error!(
            ticks = elapsed_ticks,
            task_id = %task.id,
            "VIOLATION: Hot path exceeded Chatman Constant"
        );

        // Emit SLO violation metric
        self.otel_integration.record_slo_violation(
            "chatman_constant",
            elapsed_ticks as f64
        );
    }

    Ok(result)
}
```

**Optimization Strategies:**

| Component | Strategy | Target Ticks |
|-----------|----------|--------------|
| Resource allocation | Pre-allocated pool | 1 tick |
| Pattern execution | Compiled patterns, SIMD | 3-5 ticks |
| State update | Write-behind cache | 2 ticks |
| RDF queries | In-memory index | 1 tick |
| **Total** | | **≤8 ticks** |

### 6.2 Performance Monitoring

```rust
/// Performance analyzer for workflow execution
pub struct PerformanceAnalyzer {
    /// Tick histogram (per operation)
    tick_histogram: HashMap<String, Vec<u32>>,
    /// SLO violations
    slo_violations: Vec<SloViolation>,
}

impl PerformanceAnalyzer {
    /// Record operation ticks
    pub fn record_ticks(&mut self, operation: &str, ticks: u32) {
        self.tick_histogram
            .entry(operation.to_string())
            .or_insert_with(Vec::new)
            .push(ticks);

        // Check against Chatman Constant
        if operation.starts_with("hot_path") && ticks > CHATMAN_CONSTANT {
            self.slo_violations.push(SloViolation {
                operation: operation.to_string(),
                expected: CHATMAN_CONSTANT,
                actual: ticks,
                timestamp: Utc::now(),
            });
        }
    }

    /// Generate performance report
    pub fn report(&self) -> PerformanceReport {
        let mut report = PerformanceReport::default();

        for (op, ticks) in &self.tick_histogram {
            let p50 = percentile(ticks, 0.5);
            let p95 = percentile(ticks, 0.95);
            let p99 = percentile(ticks, 0.99);

            report.add_metric(op, p50, p95, p99);
        }

        report.violations = self.slo_violations.clone();
        report
    }
}
```

---

## 7. Error Handling & Rollback

### 7.1 Error Handling Strategy

```rust
/// Workflow-specific errors
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Workflow spec not found: {0}")]
    SpecNotFound(WorkflowSpecId),

    #[error("Case not found: {0}")]
    CaseNotFound(CaseId),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Pattern not found: {0}")]
    PatternNotFound(PatternId),

    #[error("RDF parse error: {0}")]
    RdfParse(String),

    #[error("SHACL validation failed: {0}")]
    ShaclValidation(String),

    #[error("Deadlock detected: {0}")]
    Deadlock(String),

    #[error("State persistence error: {0}")]
    StatePersistence(String),

    #[error("Hot path SLO violation: {0} ticks (max: 8)")]
    ChatmanViolation(u32),
}
```

### 7.2 Rollback Strategy

```rust
/// Transaction-like execution with rollback
pub async fn execute_case_with_rollback(
    &mut self,
    case_id: CaseId
) -> WorkflowResult<()> {
    // 1. Snapshot current state
    let snapshot = self.state_manager.create_snapshot(case_id).await?;

    // 2. Execute case
    match self.execute_case_with_telemetry(case_id).await {
        Ok(_) => {
            // Success: commit state
            self.state_manager.commit(case_id).await?;
            Ok(())
        }
        Err(e) => {
            // Failure: rollback to snapshot
            error!(case_id = %case_id, error = %e, "Execution failed, rolling back");
            self.state_manager.rollback(case_id, snapshot).await?;
            Err(e)
        }
    }
}
```

---

## 8. Fortune 5 Integration

### 8.1 Multi-Region Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                     Global Load Balancer                      │
└───────────────┬────────────────────┬──────────────────────────┘
                │                    │
        ┌───────v────────┐   ┌───────v────────┐
        │  Region US-EAST│   │  Region EU-WEST│
        │                │   │                │
        │  ┌──────────┐  │   │  ┌──────────┐  │
        │  │  Engine  │  │   │  │  Engine  │  │
        │  └────┬─────┘  │   │  └────┬─────┘  │
        │       │        │   │       │        │
        │  ┌────v─────┐  │   │  ┌────v─────┐  │
        │  │   Sled   │  │   │  │   Sled   │  │
        │  └──────────┘  │   │  └──────────┘  │
        └────────────────┘   └────────────────┘
                │                    │
                └────────┬───────────┘
                         │
                    ┌────v─────┐
                    │  KMS     │
                    │ (Secrets)│
                    └──────────┘
```

### 8.2 SPIFFE Integration

```rust
/// SPIFFE-based authentication
pub struct SpiffeAuth {
    workload_api: WorkloadApiClient,
}

impl SpiffeAuth {
    /// Authenticate service-to-service
    pub async fn authenticate(&self, peer_id: &str) -> WorkflowResult<bool> {
        let svid = self.workload_api.fetch_x509_svid().await?;

        // Verify peer SPIFFE ID
        let peer_svid = self.workload_api.fetch_x509_bundle(peer_id).await?;

        // Validate trust domain
        if svid.spiffe_id.trust_domain() != peer_svid.trust_domain() {
            return Err(WorkflowError::Authentication(
                "Trust domain mismatch".to_string()
            ));
        }

        Ok(true)
    }
}
```

### 8.3 SLO Enforcement

```rust
/// SLO definitions for workflow engine
pub struct SloPolicy {
    /// Hot path execution: ≤8 ticks
    pub chatman_constant: u32,
    /// Case creation: <100ms
    pub case_creation_ms: u64,
    /// Workflow registration: <500ms
    pub workflow_registration_ms: u64,
    /// SPARQL query: <50ms
    pub sparql_query_ms: u64,
}

impl SloPolicy {
    pub fn enforce(&self, operation: &str, elapsed: u64) -> Result<(), SloViolation> {
        let threshold = match operation {
            "hot_path" => self.chatman_constant as u64,
            "case_creation" => self.case_creation_ms,
            "workflow_registration" => self.workflow_registration_ms,
            "sparql_query" => self.sparql_query_ms,
            _ => return Ok(()),
        };

        if elapsed > threshold {
            Err(SloViolation {
                operation: operation.to_string(),
                expected: threshold,
                actual: elapsed,
                timestamp: Utc::now(),
            })
        } else {
            Ok(())
        }
    }
}
```

---

## 9. Implementation Roadmap

### Phase 1: Core RDF Execution (Week 1)

**Priority:** HIGH
**Validation:** Weaver schema validation

- [ ] `RdfWorkflowLoader` implementation
  - [ ] Parse ATM workflow from `.ttl`
  - [ ] SHACL validation integration
  - [ ] Extract `WorkflowSpec` via SPARQL
  - [ ] Weaver validates parsing telemetry
- [ ] `WorkflowEngine::register_workflow_from_rdf()`
  - [ ] Load into `spec_rdf_store`
  - [ ] Persist to Sled
  - [ ] Weaver validates registration telemetry
- [ ] `WorkflowEngine::execute_case_with_telemetry()`
  - [ ] Pattern execution loop
  - [ ] State updates
  - [ ] Weaver validates execution telemetry
- [ ] Chicago TDD tests (real collaborators)
  - [ ] Load ATM workflow
  - [ ] Execute end-to-end
  - [ ] Verify state changes
  - [ ] Weaver validates all telemetry

**Success Criteria:**
```bash
# Execute ATM workflow end-to-end
cargo test --test chicago_tdd_atm_workflow -- --nocapture

# Weaver validation (MANDATORY)
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

### Phase 2: Pattern Metadata & SPARQL (Week 2)

**Priority:** MEDIUM
**Validation:** Weaver + SPARQL results

- [ ] Load pattern metadata into `pattern_metadata_store`
  - [ ] All 43 patterns
  - [ ] Dependencies, categories
  - [ ] Weaver validates metadata telemetry
- [ ] SPARQL query API
  - [ ] `query_rdf()` for workflow structure
  - [ ] `query_case_rdf()` for runtime state
  - [ ] `query_pattern_metadata()` for pattern info
  - [ ] Weaver validates query telemetry
- [ ] REST endpoint `/sparql`
  - [ ] POST SPARQL queries
  - [ ] Return JSON results
  - [ ] Weaver validates endpoint telemetry

**Success Criteria:**
```bash
# SPARQL queries work
cargo test --test sparql_queries -- --nocapture

# Weaver validation (MANDATORY)
weaver registry live-check --registry registry/
```

### Phase 3: Performance Optimization (Week 3)

**Priority:** HIGH
**Validation:** Weaver + performance metrics

- [ ] Hot path optimization
  - [ ] Resource pre-allocation
  - [ ] SIMD pattern execution
  - [ ] Write-behind state cache
  - [ ] Weaver validates performance telemetry
- [ ] Performance monitoring
  - [ ] Tick histogram
  - [ ] SLO violation detection
  - [ ] Chatman Constant enforcement
  - [ ] Weaver validates monitoring telemetry
- [ ] Benchmarks
  - [ ] Hot path ≤8 ticks
  - [ ] Case creation <100ms
  - [ ] SPARQL query <50ms

**Success Criteria:**
```bash
# Performance benchmarks pass
cargo test --test performance_benchmarks -- --nocapture

# Weaver validates performance telemetry
weaver registry live-check --registry registry/
```

### Phase 4: Fortune 5 Features (Week 4)

**Priority:** ENTERPRISE
**Validation:** Weaver + compliance

- [ ] Multi-region deployment
  - [ ] Sled replication
  - [ ] Cross-region queries
  - [ ] Weaver validates replication telemetry
- [ ] SPIFFE authentication
  - [ ] Workload API integration
  - [ ] Trust domain validation
  - [ ] Weaver validates auth telemetry
- [ ] KMS integration
  - [ ] Encrypted state
  - [ ] Secret rotation
  - [ ] Weaver validates encryption telemetry
- [ ] SLO enforcement
  - [ ] Policy definitions
  - [ ] Violation alerting
  - [ ] Weaver validates SLO telemetry

**Success Criteria:**
```bash
# Fortune 5 tests pass
cargo test --test fortune5_integration -- --nocapture

# Weaver validates enterprise telemetry
weaver registry live-check --registry registry/
```

---

## 10. Testing Strategy

### 10.1 Chicago TDD (No Mocks)

**Critical:** Use REAL collaborators, not mocks.

```rust
#[tokio::test]
async fn test_atm_workflow_end_to_end() {
    // Setup: Real engine, real Sled, real Oxigraph
    let engine = WorkflowEngine::new().await.unwrap();

    // Load ATM workflow from .ttl file
    let turtle = fs::read_to_string("ontology/workflows/financial/atm_transaction.ttl")
        .unwrap();

    let spec_id = engine.register_workflow_from_rdf(&turtle)
        .await
        .unwrap();

    // Create case with initial data
    let case_data = json!({
        "cardNumber": "1234-5678-9012-3456",
        "pin": "1234",
        "accountNumber": "ACC123",
        "balance": 1000.00,
        "withdrawalAmount": 200.00
    });

    let case_id = engine.create_case(spec_id, case_data)
        .await
        .unwrap();

    // Execute workflow
    engine.execute_case_with_telemetry(case_id)
        .await
        .unwrap();

    // Verify: Case completed successfully
    let case = engine.get_case(&case_id).await.unwrap();
    assert_eq!(case.state, CaseState::Completed);

    // Verify: Balance updated correctly
    let result = engine.query_case_rdf(
        &case_id,
        r#"SELECT ?balance WHERE { ?case yawl:balance ?balance }"#
    ).await.unwrap();

    assert_eq!(result[0]["balance"], "800.00");

    // Verify: All OTEL spans emitted (Weaver will validate)
    // No assertions here - Weaver validation is the source of truth
}
```

### 10.2 Weaver Validation Tests

**MANDATORY:** All tests MUST include Weaver validation.

```bash
#!/bin/bash
# Weaver validation wrapper for tests

set -e

# Run test
cargo test --test $1 -- --nocapture

# Validate telemetry with Weaver
weaver registry live-check --registry registry/

if [ $? -eq 0 ]; then
    echo "✅ Weaver validation passed"
else
    echo "❌ Weaver validation FAILED"
    exit 1
fi
```

### 10.3 Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_workflow_registration_idempotent(
        workflow_name in "[a-z]{5,10}"
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let engine = WorkflowEngine::new().await.unwrap();

            let turtle = generate_simple_workflow(&workflow_name);

            // Register twice
            let spec_id_1 = engine.register_workflow_from_rdf(&turtle)
                .await
                .unwrap();

            let spec_id_2 = engine.register_workflow_from_rdf(&turtle)
                .await
                .unwrap();

            // Should return same ID (idempotent)
            prop_assert_eq!(spec_id_1, spec_id_2);
        });
    }
}
```

---

## 11. Monitoring & Observability

### 11.1 OTEL Metrics

```rust
/// Key metrics for workflow execution
pub struct WorkflowMetrics {
    /// Total workflows registered
    pub workflows_registered: Counter<u64>,

    /// Total cases created
    pub cases_created: Counter<u64>,

    /// Total cases executed
    pub cases_executed: Counter<u64>,

    /// Hot path execution time (histogram)
    pub hot_path_ticks: Histogram<u32>,

    /// Chatman Constant violations
    pub chatman_violations: Counter<u64>,

    /// SPARQL query latency (histogram)
    pub sparql_query_latency_ms: Histogram<f64>,

    /// Pattern execution count (by pattern ID)
    pub pattern_executions: Counter<u64>,
}
```

### 11.2 Dashboards

**Grafana Dashboard: Workflow Engine**

```yaml
panels:
  - title: "Workflow Registrations"
    query: rate(workflow_registrations_total[5m])

  - title: "Case Execution Rate"
    query: rate(case_executions_total[5m])

  - title: "Hot Path Performance (p95)"
    query: histogram_quantile(0.95, hot_path_ticks)
    threshold: 8  # Chatman Constant

  - title: "Chatman Violations"
    query: rate(chatman_violations_total[5m])
    alert: "> 0"

  - title: "SPARQL Query Latency (p99)"
    query: histogram_quantile(0.99, sparql_query_latency_ms)
    threshold: 50  # SLO: <50ms
```

### 11.3 Alerts

```yaml
alerts:
  - name: "Chatman Constant Violation"
    condition: hot_path_ticks > 8
    severity: critical
    action: page_oncall

  - name: "SPARQL Query Slow"
    condition: sparql_query_latency_ms > 50
    severity: warning
    action: log

  - name: "Case Execution Failure Rate High"
    condition: rate(case_execution_failures[5m]) > 0.05
    severity: critical
    action: page_oncall
```

---

## 12. Security Considerations

### 12.1 RDF Injection Prevention

```rust
/// Sanitize user input in SPARQL queries
pub fn sanitize_sparql(query: &str) -> Result<String, SecurityError> {
    // Check for SQL injection-like patterns
    let dangerous_patterns = [
        "DROP", "DELETE", "INSERT", "UPDATE",
        "LOAD", "CLEAR", "CREATE", "MOVE"
    ];

    let upper = query.to_uppercase();
    for pattern in &dangerous_patterns {
        if upper.contains(pattern) {
            return Err(SecurityError::InjectionAttempt(
                format!("Dangerous SPARQL keyword: {}", pattern)
            ));
        }
    }

    // Only allow SELECT and ASK queries
    if !upper.trim().starts_with("SELECT") &&
       !upper.trim().starts_with("ASK") {
        return Err(SecurityError::InvalidQueryType);
    }

    Ok(query.to_string())
}
```

### 12.2 Access Control

```rust
/// Role-based access control for workflows
pub struct WorkflowAccessControl {
    policies: HashMap<WorkflowSpecId, AccessPolicy>,
}

pub struct AccessPolicy {
    /// Roles allowed to register workflow
    register_roles: Vec<String>,
    /// Roles allowed to create cases
    create_roles: Vec<String>,
    /// Roles allowed to execute cases
    execute_roles: Vec<String>,
    /// Roles allowed to query RDF
    query_roles: Vec<String>,
}

impl WorkflowAccessControl {
    pub fn check_permission(
        &self,
        spec_id: &WorkflowSpecId,
        action: Action,
        user_roles: &[String]
    ) -> Result<(), SecurityError> {
        let policy = self.policies.get(spec_id)
            .ok_or(SecurityError::PolicyNotFound)?;

        let allowed_roles = match action {
            Action::Register => &policy.register_roles,
            Action::CreateCase => &policy.create_roles,
            Action::ExecuteCase => &policy.execute_roles,
            Action::QueryRdf => &policy.query_roles,
        };

        if user_roles.iter().any(|r| allowed_roles.contains(r)) {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied)
        }
    }
}
```

---

## 13. Appendix

### 13.1 Glossary

| Term | Definition |
|------|------------|
| **Chatman Constant** | Performance constraint: ≤8 ticks for hot path operations |
| **Chicago TDD** | Test-driven development with real collaborators (no mocks) |
| **Oxigraph** | Rust RDF database with SPARQL support |
| **SHACL** | Shapes Constraint Language for RDF validation |
| **SPARQL** | Query language for RDF graphs |
| **Turtle** | Compact RDF serialization format |
| **Weaver** | OpenTelemetry schema validation tool (source of truth) |
| **WorkflowSpec** | Internal representation of workflow graph |
| **YAWL** | Yet Another Workflow Language |

### 13.2 References

- [RDF 1.1 Turtle](https://www.w3.org/TR/turtle/)
- [SPARQL 1.1 Query Language](https://www.w3.org/TR/sparql11-query/)
- [SHACL Validation](https://www.w3.org/TR/shacl/)
- [OpenTelemetry Weaver](https://github.com/open-telemetry/weaver)
- [Oxigraph Documentation](https://github.com/oxigraph/oxigraph)
- [Van der Aalst Workflow Patterns](http://www.workflowpatterns.com/)
- [YAWL Foundation](http://www.yawlfoundation.org/)

### 13.3 Related ADRs

- [ADR-001: Why Rust Over Java](ADR/ADR-001-why-rust-over-java.md)
- [ADR-002: Turtle vs YAWL XML](ADR/ADR-002-turtle-vs-yawl-xml.md)
- [ADR-003: Sled vs PostgreSQL](ADR/ADR-003-sled-vs-postgres.md)
- [ADR-007: 80/20 Feature Selection](ADR/ADR-007-80-20-feature-selection.md)
- [ADR-008: Interface B Priority](ADR/ADR-008-interface-b-priority.md)

---

## Document Metadata

**Version History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-11-08 | System Architect | Initial architecture design |

**Reviewers:**

- [ ] System Architect
- [ ] Code Analyzer (Quality Review)
- [ ] Performance Benchmarker (Performance Validation)
- [ ] Production Validator (Weaver Validation)

**Approval:**

- [ ] Technical Lead
- [ ] Enterprise Architect
- [ ] Security Team
- [ ] Weaver Validation Passed

---

**END OF DOCUMENT**
