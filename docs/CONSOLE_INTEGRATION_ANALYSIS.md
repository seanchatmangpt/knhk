# Console Implementation & Workflow Engine Integration Analysis

## Current State Summary

### Console Implementation (console.rs)

**Location**: `/home/user/knhk/rust/knhk-cli/src/console.rs` (17,303 bytes, 509 lines)

**Current Status**: 
- Four verbs implemented with basic skeleton code
- Placeholder implementations for `run` and `query` commands
- Global context management with OnceLock<Mutex<>>
- OTEL tracing integrated for all commands
- Async runtime handling for workflow parsing

**Implemented Functions**:
1. `start(state_store: Option<String>)` - Initialize console session
2. `load(file: PathBuf, state_store: Option<String>)` - Parse Turtle workflow
3. `run(command: String)` - Execute commands (PLACEHOLDER)
4. `query(query: String)` - Execute SPARQL queries (PLACEHOLDER)

**What's Already Wired**:
- ✓ WorkflowParser integration (lines 17, 158, 224)
- ✓ Global context with workflow_path, workflow_id, state_store_path
- ✓ OTEL instrumentation with #[instrument] macros
- ✓ Tokio runtime for async operations
- ✓ Feature-gated compilation (#[cfg(feature = "workflow")] and #[cfg(feature = "otel")])
- ✓ Proper error handling with clap_noun_verb::NounVerbError
- ✓ Basic help/status/patterns output in run() (lines 294-333)

**What's NOT Wired (Placeholders)**:
- ✗ Actual workflow execution in `run()` - only returns static help text
- ✗ Workflow validation in `run()` - "validate" command returns placeholder
- ✗ Case creation in `run()` - "create-case" command returns placeholder  
- ✗ Case listing in `run()` - "list-cases" command returns placeholder
- ✗ SPARQL query execution in `query()` - comment says "RDF store integration pending" (lines 452, 494)

---

## Workflow Engine Structure

### Architecture Overview

**Engine Location**: `/home/user/knhk/rust/knhk-workflow-engine/`

**Core Engine**: `/home/user/knhk/rust/knhk-workflow-engine/src/executor/`

**Engine Structure** (executor/mod.rs):
```
executor/
├── engine.rs           (3,851 bytes) - Core WorkflowEngine struct definition
├── construction.rs    (11,098 bytes) - Engine creation/initialization
├── case.rs            (11,463 bytes) - Case management (create, start, execute, cancel, get, list)
├── workflow_registration.rs (4,069 bytes) - Workflow registration
├── workflow_query.rs   (1,179 bytes) - Workflow query operations
├── workflow_execution.rs (18,818 bytes) - Pattern execution and task execution
├── task.rs            (26,416 bytes) - Task execution with resource allocation
├── pattern.rs         (9,484 bytes) - Pattern execution with reflex bridge
├── accessors.rs       (2,415 bytes) - Getter methods
├── events.rs          (3,355 bytes) - Event loop handlers
├── rdf_query.rs       (5,993 bytes) - Runtime RDF query API
├── xes_export.rs      (9,307 bytes) - XES export functionality
├── provenance.rs      (4,969 bytes) - Lockchain provenance
└── fortune5.rs        (1,041 bytes) - Fortune 5 integration
```

### Core Operations Available

**WorkflowEngine Public Methods** (from executor/engine.rs and files):

1. **Workflow Management**:
   - `register_workflow(spec: WorkflowSpec)` - Register new workflow
   - `get_workflow(spec_id: WorkflowSpecId)` - Retrieve workflow spec
   - `list_workflows()` - List all registered workflows
   - `delete_workflow(spec_id: WorkflowSpecId)` - Delete workflow

2. **Case Management**:
   - `create_case(spec_id, data)` - Create new workflow case
   - `start_case(case_id)` - Start case execution
   - `execute_case(case_id)` - Execute case
   - `cancel_case(case_id)` - Cancel case
   - `get_case(case_id)` - Get case status
   - `list_cases(spec_id)` - List cases for workflow

3. **Pattern Execution**:
   - `identify_task_pattern(task)` - Map task to Van der Aalst pattern (1-43)
   - Pattern execution via `PatternExecutionContext`

4. **Validation**:
   - Via `ValidationFramework` trait
   - Deadlock detection in parser
   - Guard constraint validation (MAX_RUN_LEN)

5. **Integration**:
   - OTEL telemetry (`otel_integration`)
   - Lockchain provenance (`lockchain_integration`)
   - Connectors support (`connector_integration`)
   - Fortune5 integration (`fortune5_integration`)
   - RDF query support (via `spec_rdf_store` and `case_rdf_stores`)

---

## WorkflowParser Structure

**Location**: `/home/user/knhk/rust/knhk-workflow-engine/src/parser/`

**Key Files**:
- `mod.rs` - Main parser interface
- `extractor.rs` - RDF extraction logic
- `types.rs` - Data types (WorkflowSpec, Task, Flow, etc.)

**Parser Capabilities**:

```rust
pub struct WorkflowParser {
    store: Store,  // Oxigraph RDF store
    deadlock_detector: DeadlockDetector,
}
```

**Parsing Methods**:
- `parse_turtle(turtle: &str)` - Parse Turtle RDF string
- `parse_jsonld(jsonld: &str)` - Parse JSON-LD string
- `parse_jsonld_file(path)` - Parse JSON-LD file
- `parse_file(path)` - Parse Turtle file (detects format)
- `load_yawl_ontology(ontology_path)` - Load YAWL ontology
- `export_turtle()` - Export RDF as Turtle

**Output**:
```rust
pub struct WorkflowSpec {
    pub id: WorkflowSpecId,
    pub name: String,
    pub description: Option<String>,
    pub tasks: Vec<Task>,
    pub flows: Vec<Flow>,
    pub input_data: Option<serde_json::Value>,
    pub source_turtle: Option<String>,  // Stored for runtime RDF queries
}
```

---

## Service Layer Architecture

**Location**: `/home/user/knhk/rust/knhk-workflow-engine/src/api/`

**Service Layer** (api/service/):
```
service/
├── mod.rs          - Re-exports
├── workflow.rs     - WorkflowService (register, get, list, delete)
├── case.rs         - CaseService (create, start, execute, cancel, get, list)
└── pattern.rs      - PatternService (list patterns, get pattern info)
```

**Example - WorkflowService.register_workflow()**:
```rust
pub async fn register_workflow(
    &self,
    request: RegisterWorkflowRequest,
) -> ApiResult<RegisterWorkflowResponse>
```

Enforces guard constraints:
- MAX_RUN_LEN validation for task count
- MAX_RUN_LEN validation for flow count

**Example - CaseService.create_case()**:
```rust
pub async fn create_case(
    &self,
    request: CreateCaseRequest,
) -> ApiResult<CreateCaseResponse>
```

---

## OTEL Telemetry Integration

**Location**: `/home/user/knhk/rust/knhk-otel/src/`

**Key Files**:
- `lib.rs` - Core types and integration (52,702 bytes)
- `types/` - Type definitions
- `metrics/` - Metrics helpers
- `exporter/` - OTLP export
- `hot_path.rs` - Hot path validation (11,897 bytes)
- `simd.rs` - SIMD optimizations (7,487 bytes)

**Integration Points**:

1. **Span Creation**:
   ```rust
   #[cfg_attr(feature = "otel", instrument(
       skip_all, 
       fields(operation = "knhk.console.start", state_store = ?state_store)
   ))]
   ```

2. **Custom Spans** (in workflow execution):
   ```rust
   otel_span!(otel, "knhk.workflow_engine.create_case", spec_id: Some(&spec_id))
   ```

3. **Span Attributes**:
   ```rust
   otel_attr!(otel, "workflow_id", &workflow_id);
   otel_attr!(otel, "duration_ms", duration.as_millis());
   ```

4. **Bottleneck Detection**:
   ```rust
   otel_bottleneck!(otel, "workflow_execution", tick_count);
   ```

---

## How Workflow.rs Integrates (Pattern Reference)

**Location**: `/home/user/knhk/rust/knhk-cli/src/workflow.rs` (1,064 lines)

**Verb Functions** (15 total):
1. `parse()` - Parse Turtle file (lines 76-116)
2. `register()` - Register workflow (lines 119-166)
3. `create()` - Create case (lines 169-200)
4. `start()` - Start case (lines 203-227)
5. `execute()` - Execute case (lines 230-253)
6. `cancel()` - Cancel case (lines 256-279)
7. `get()` - Get case status (lines 282-312)
8. `list()` - List cases/workflows (lines 315-354)
9. `patterns()` - List all 43 patterns (lines 357-377)
10. `serve()` - REST API server (lines 379-425)
11. `import_xes()` - Import XES (lines 428-467)
12. `validate_xes()` - Validate XES (lines 587-807)
13. `export_xes()` - Export to XES (lines 809-870)
14. `weaver_live_check()` - Weaver validation (lines 872-996)
15. `discover()` - Process discovery (lines 999+)

**Key Pattern: Service Layer Usage**
```rust
// 1. Create engine
let engine = get_engine(state_store.as_deref())?;

// 2. Wrap in service
let service = WorkflowService::new(engine);

// 3. Create request
let request = RegisterWorkflowRequest { spec };

// 4. Call service
let response = service.register_workflow(request).await?;

// 5. Format output
println!("{}", CliAdapter::format_error(&response));
```

---

## Boot.rs Pattern (Simpler Example)

**Location**: `/home/user/knhk/rust/knhk-cli/src/boot.rs` (71 lines)

**Shows minimal pattern**:
```rust
#[cfg_attr(feature = "otel", instrument(skip_all, fields(operation = "knhk.boot.init", ...)))]
#[verb]
fn init(sigma: String, q: String) -> Result<InitResult> {
    #[cfg(feature = "otel")]
    {
        use std::time::Instant;
        use tracing::{error, info};
        
        let start = Instant::now();
        // Implementation
    }
    #[cfg(not(feature = "otel"))]
    {
        // Implementation without OTEL
    }
}
```

---

## Integration Points for Console

### 1. Case Service Integration

**Current**: Console placeholder in `run()` for "create-case" and "list-cases"

**What's needed**:
```rust
use knhk_workflow_engine::api::service::CaseService;
use knhk_workflow_engine::api::models::requests::CreateCaseRequest;

// In run() command handler:
if command.trim() == "create-case" {
    let case_id = engine.create_case(spec_id, serde_json::json!({})).await?;
    return Ok(vec![format!("Case created: {}", case_id)]);
}
```

**Engine Method**: `WorkflowEngine::create_case(spec_id, data)` ✓ Available
**Service Method**: `CaseService::create_case(request)` ✓ Available

---

### 2. Workflow Validation Integration

**Current**: Console placeholder in `run()` for "validate"

**What's needed**:
```rust
use knhk_workflow_engine::validation::ValidationFramework;

// In run() command handler:
if command.trim() == "validate" {
    let spec = engine.get_workflow(workflow_id).await?;
    // Use ValidationFramework to validate
    return Ok(vec!["Workflow validation passed".to_string()]);
}
```

**Available**: `ValidationFramework` trait ✓
**Available**: `engine.get_workflow()` ✓

---

### 3. SPARQL Query Integration

**Current**: Console placeholder in `query()`

**What's needed**:
```rust
#[cfg(feature = "rdf")]
use oxigraph::query::QueryResults;

// In query() command handler:
let spec = get_context().get_workflow_id()?;
let rdf_store = engine.get_rdf_store_for_case(case_id)?;
// Execute SPARQL against rdf_store
```

**Available**: 
- `spec_rdf_store: Arc<RwLock<Store>>` in WorkflowEngine ✓
- `parse_file()` stores `source_turtle` in spec ✓
- Engine has `rdf_query_api()` or similar ✓

---

### 4. OTEL Integration

**Current**: Partially integrated - has #[instrument] macros

**What's needed**:
- Add `otel_span!()` calls inside run/query functions when creating cases
- Add `otel_attr!()` for case_id, spec_id when executing
- Add `otel_span_end!()` on success/failure

**Pattern from case.rs**:
```rust
let span_ctx: Option<SpanContext> = if let Some(ref otel) = self.otel_integration {
    otel_span!(
        otel,
        "knhk.workflow_engine.create_case",
        spec_id: Some(&spec_id)
    )
    .await
    .map_err(|e| e)?
} else {
    None
};
```

---

## State Management

### Current Console Context

```rust
#[derive(Clone, Debug)]
struct ConsoleContext {
    workflow_path: Option<String>,
    workflow_id: Option<String>,
    state_store_path: Option<String>,
}

static CONSOLE_CONTEXT: std::sync::OnceLock<Mutex<ConsoleContext>> = std::sync::OnceLock::new();
```

**What's Missing**:
- Need to store loaded `WorkflowSpec` for validation
- Need to track active `CaseId` for case-specific operations
- May want to cache parsed workflow to avoid re-parsing

### Proposed Enhancement

```rust
#[derive(Clone, Debug)]
struct ConsoleContext {
    workflow_path: Option<String>,
    workflow_id: Option<String>,
    state_store_path: Option<String>,
    // NEW:
    current_workflow_spec: Option<WorkflowSpec>,  // Cache for validation
    current_case_id: Option<CaseId>,              // Track active case
    engine: Option<Arc<WorkflowEngine>>,          // Reuse engine instance
}
```

---

## Architecture Diagram

```
┌─────────────────────────────────────┐
│ Console Commands (console.rs)       │
│ ┌─────────────────────────────────┐ │
│ │ start() - Initialize context    │ │
│ │ load() - Parse Turtle file      │ │
│ │ run() - Execute commands        │ │ ← Needs wiring
│ │ query() - SPARQL queries        │ │ ← Needs wiring
│ └─────────────────────────────────┘ │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ Service Layer (api/service/)        │
│ ┌──────────────────────────────────┐│
│ │ WorkflowService                  ││
│ │ ├─ register_workflow()           ││
│ │ ├─ get_workflow()                ││
│ │ ├─ list_workflows()              ││
│ │ └─ delete_workflow()             ││
│ ├──────────────────────────────────┤│
│ │ CaseService                      ││
│ │ ├─ create_case()       ← NEEDED  ││
│ │ ├─ start_case()        ← NEEDED  ││
│ │ ├─ execute_case()      ← NEEDED  ││
│ │ ├─ get_case()          ← NEEDED  ││
│ │ ├─ list_cases()        ← NEEDED  ││
│ │ └─ cancel_case()       ← NEEDED  ││
│ ├──────────────────────────────────┤│
│ │ PatternService                   ││
│ │ └─ list_patterns()     ← NEEDED  ││
│ └──────────────────────────────────┘│
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ Executor Layer (executor/)          │
│ ├─ WorkflowEngine                   │
│ │  ├─ register_workflow()           │
│ │  ├─ create_case()                 │
│ │  ├─ start_case()                  │
│ │  ├─ execute_case()                │
│ │  ├─ get_workflow()                │
│ │  ├─ rdf_query_api()               │
│ │  └─ pattern_registry              │
│ ├─ Pattern Execution                │
│ └─ Task Execution                   │
└─────────────────────────────────────┘
```

---

## Testing & Examples

**Example Workflow**: `/home/user/knhk/rust/docs/yawl/code/example.rs`

**Integration Tests**: `/home/user/knhk/rust/knhk-workflow-engine/tests/integration/workflow_execution.rs`
- Currently placeholders

**Workflow Tests**: 
- `/home/user/knhk/rust/knhk-workflow-engine/tests/chicago_tdd_*.rs` (multiple files)

---

## Key Files Reference

| File | Location | Purpose | Lines |
|------|----------|---------|-------|
| console.rs | knhk-cli/src/ | Console verb commands | 509 |
| workflow.rs | knhk-cli/src/ | Workflow verb commands (reference) | 1,064 |
| engine.rs | engine/src/executor/ | Core engine struct | 90 |
| case.rs | engine/src/executor/ | Case management | 80+ |
| workflow.rs | engine/src/executor/ | Service for workflows | 120 |
| parser/mod.rs | engine/src/parser/ | Turtle parser interface | 146 |
| extractor.rs | engine/src/parser/ | RDF extraction | 23,557 |
| WorkflowEngine | executor/mod.rs | Main engine export | - |
| lib.rs | knhk-otel/src/ | OTEL integration | 52,702 |
| console-commands.md | docs/ | Documentation | - |

---

## Dependency Chain

```
knhk-cli
├── knhk-workflow-engine (feature: "workflow")
│   ├── knhk-otel
│   ├── knhk-lockchain
│   ├── knhk-patterns
│   ├── oxigraph (RDF/Turtle)
│   └── tokio (async)
├── knhk-otel (feature: "otel")
├── process_mining (XES handling)
└── clap-noun-verb (command routing)
```

---

## Summary: What Needs Integration

### Console `run()` Command - Needs Implementation

| Command | Status | What's Needed | Complexity |
|---------|--------|---------------|-----------|
| help | ✓ Done | Static text | - |
| status | ✓ Done | Display context | - |
| patterns | ✓ Done | Static list | - |
| validate | ✗ TODO | Call ValidationFramework | Medium |
| create-case | ✗ TODO | Call CaseService::create_case() | Medium |
| list-cases | ✗ TODO | Call CaseService::list_cases() | Low |
| quit | ✗ TODO | Exit cleanly | Low |

### Console `query()` Command - Needs Implementation

| Component | Status | What's Needed | Complexity |
|-----------|--------|---------------|-----------|
| Load RDF Store | ✗ TODO | Access spec_rdf_store or rebuild from source_turtle | Medium |
| SPARQL Parse | ✗ TODO | Use oxigraph query parser | Low |
| Execute Query | ✗ TODO | Execute against RDF store | Medium |
| Format Results | ✗ TODO | Convert QueryResults to JSON/text | Low |

### Engine Context - Enhancement

| Item | Status | Purpose |
|------|--------|---------|
| Cached WorkflowSpec | ✗ TODO | Avoid re-parsing for validation |
| Current CaseId | ✗ TODO | Track active case for queries |
| Engine Instance | ✗ TODO | Reuse across commands |

