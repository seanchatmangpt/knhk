# Console Integration - Quick Reference Guide

## Current Implementation Status

### ✓ DONE - No Changes Needed
- `console start` - Initialize context
- `console load` - Parse Turtle workflow
- Basic OTEL instrumentation
- Context management with OnceLock<Mutex<>>
- Help/status/patterns commands

### ✗ TODO - Needs Integration

#### 1. Console `run()` Command - 4 Placeholders

**Lines 293-334**: `"validate"` command

```rust
"validate" => vec!["Workflow validation passed.".to_string()],
```

**Needs to become**:
```rust
"validate" => {
    let ctx = get_context().lock()?;
    let workflow_id = ctx.workflow_id.clone().ok_or(...)?;
    let engine = get_engine(ctx.state_store_path.as_deref())?;
    let spec = engine.get_workflow(workflow_id).await?;
    // Validate spec...
    vec!["Workflow validation passed.".to_string()]
}
```

---

**Lines 327-328**: `"create-case"` command

```rust
"create-case" => vec!["Case created successfully.".to_string()],
```

**Needs to become**:
```rust
"create-case" => {
    let ctx = get_context().lock()?;
    let spec_id = ctx.workflow_id.clone().ok_or(...)?;
    let engine = get_engine(ctx.state_store_path.as_deref())?;
    let case_id = engine.create_case(spec_id, serde_json::json!({})).await?;
    // Update context with case_id
    vec![format!("Case created: {}", case_id)]
}
```

---

**Lines 328-329**: `"list-cases"` command

```rust
"list-cases" => vec!["No cases found for this workflow.".to_string()],
```

**Needs to become**:
```rust
"list-cases" => {
    let ctx = get_context().lock()?;
    let spec_id = ctx.workflow_id.clone().ok_or(...)?;
    let engine = get_engine(ctx.state_store_path.as_deref())?;
    let service = CaseService::new(engine);
    let cases = service.list_cases(...).await?;
    // Format output
    vec![format!("Found {} cases", cases.len())]
}
```

---

**Missing**: `"quit"` command

Implement exit handling (may need to return specific error or signal)

---

#### 2. Console `query()` Command - Full Placeholder

**Lines 452-471 and 494-507**: SPARQL query execution

**Current code**:
```rust
let results = format!(
    "Query executed on workflow: {}\nQuery: {}\nResults: (RDF store integration pending)",
    ctx.workflow_id.clone().unwrap_or_default(),
    query
);
```

**Needs to become**:

```rust
#[cfg(feature = "rdf")]
{
    let ctx = get_context().lock()?;
    let spec_id = ctx.workflow_id.clone().ok_or(...)?;
    let engine = get_engine(ctx.state_store_path.as_deref())?;
    
    // Get workflow spec to access source turtle
    let spec = engine.get_workflow(spec_id).await?;
    let turtle_str = spec.source_turtle.ok_or(...)?;
    
    // Create RDF store and load turtle
    let mut parser = WorkflowParser::new()?;
    let store = parser.parse_turtle(&turtle_str)?;
    
    // Parse and execute SPARQL query
    let query_stmt = Query::parse(&query, None)
        .map_err(|e| NounVerbError::execution_error(format!("SPARQL parse error: {}", e)))?;
    
    let results = store.query(query_stmt)?;
    
    // Format results as JSON
    let formatted = match results {
        QueryResults::Solutions(solutions) => {
            // Format solution bindings
            serde_json::to_string_pretty(&solutions)
        }
        QueryResults::Boolean(b) => Ok(format!("{{ \"result\": {} }}", b)),
        QueryResults::Graph(graph) => {
            // Format graph results as RDF
            Ok(format!("{} triples", graph.len()))
        }
    };
}
```

---

## Key Classes to Use

### WorkflowEngine
```rust
engine.create_case(spec_id: WorkflowSpecId, data: serde_json::Value) -> Result<CaseId>
engine.start_case(case_id: CaseId) -> Result<()>
engine.execute_case(case_id: CaseId) -> Result<()>
engine.get_case(case_id: CaseId) -> Result<Case>
engine.get_workflow(spec_id: WorkflowSpecId) -> Result<WorkflowSpec>
engine.list_cases(spec_id: WorkflowSpecId) -> Result<Vec<CaseId>>
```

### Services (Higher-level API)
```rust
CaseService::new(engine)
    .create_case(CreateCaseRequest { spec_id, data }) -> Result<CreateCaseResponse>
    .list_cases(ListCasesRequest { spec_id }) -> Result<ListCasesResponse>

WorkflowService::new(engine)
    .get_workflow(GetWorkflowRequest { spec_id }) -> Result<GetWorkflowResponse>

PatternService::new(engine)
    .list_patterns() -> Result<PatternListResponse>
```

### RDF/SPARQL
```rust
// For SPARQL queries
#[cfg(feature = "rdf")]
use oxigraph::query::QueryResults;
use oxigraph::io::RdfFormat;

WorkflowParser::new()?
    .parse_turtle(turtle_str)?
```

---

## Integration Patterns

### Pattern 1: Get Workflow from Context
```rust
let ctx = get_context().lock()?;
let workflow_id = ctx.workflow_id.clone()
    .ok_or(clap_noun_verb::NounVerbError::execution_error("No workflow loaded"))?;
```

### Pattern 2: Get/Create Engine
```rust
let engine = get_engine(ctx.state_store_path.as_deref())?;
// or if you have it from load():
let engine = get_engine(state_store.as_deref())?;
```

### Pattern 3: Call Service Layer
```rust
let service = CaseService::new(engine);
let request = CreateCaseRequest { spec_id, data };
let response = service.create_case(request).await?;
```

### Pattern 4: Add OTEL Tracing (in run/query)
```rust
#[cfg(feature = "otel")]
{
    let span_ctx = if let Some(ref otel) = engine.otel_integration {
        otel_span!(otel, "knhk.console.run.command", command: %command).await?
    } else {
        None
    };
    
    // Do work...
    
    if let Some(span) = span_ctx {
        otel_span_end!(otel, span, success: true, duration_ms: elapsed)?;
    }
}
```

---

## Required Imports for Completeness

```rust
// Already present:
use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use std::path::PathBuf;
use std::sync::Mutex;
#[cfg(feature = "workflow")]
use knhk_workflow_engine::parser::WorkflowParser;

// Need to add:
#[cfg(feature = "workflow")]
use knhk_workflow_engine::{
    api::service::{CaseService, PatternService, WorkflowService},
    api::models::requests::{
        CreateCaseRequest, ListCasesRequest, GetWorkflowRequest,
    },
    case::CaseId,
};

#[cfg(feature = "rdf")]
use oxigraph::io::RdfFormat;
#[cfg(feature = "rdf")]
use oxigraph::query::{Query, QueryResults};
```

---

## File Locations for Reference

| Component | File | Lines | Purpose |
|-----------|------|-------|---------|
| Console | `rust/knhk-cli/src/console.rs` | 509 | Main implementation |
| Workflow (reference) | `rust/knhk-cli/src/workflow.rs` | 1,064 | Pattern to follow |
| Engine | `rust/knhk-workflow-engine/src/executor/` | 2,920 | Core logic |
| Services | `rust/knhk-workflow-engine/src/api/service/` | 394 | High-level API |
| Parser | `rust/knhk-workflow-engine/src/parser/mod.rs` | 146 | Turtle parsing |
| Documentation | `docs/console-commands.md` | - | User docs |

---

## Testing Checklist

- [ ] `console start` initializes context
- [ ] `console load examples/workflow.ttl` parses and stores workflow
- [ ] `console run "help"` shows help text
- [ ] `console run "status"` displays workflow ID and path
- [ ] `console run "validate"` calls ValidationFramework
- [ ] `console run "create-case"` creates new case via engine
- [ ] `console run "list-cases"` lists cases for workflow
- [ ] `console query "SELECT ..."` executes SPARQL (if RDF feature enabled)
- [ ] OTEL spans appear in telemetry output (if otel feature enabled)

---

## Common Errors & Fixes

### Error: "No workflow loaded"
- Verify console context is initialized: `console load <file>` first
- Check workflow_id is set after load

### Error: "Failed to create parser"
- Verify `workflow` feature is enabled in Cargo.toml
- Check Turtle file syntax is valid

### Error: "Invalid SPARQL"
- Ensure query is valid SPARQL SELECT/ASK/CONSTRUCT
- Wrap in `#[cfg(feature = "rdf")]` for oxigraph features

### Missing OTEL spans
- Verify `otel` feature is enabled
- Check OTLP endpoint is configured
- Ensure `otel_integration` is Some in engine

