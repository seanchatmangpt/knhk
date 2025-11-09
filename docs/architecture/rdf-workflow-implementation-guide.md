# RDF Workflow Implementation Guide

**Version:** 1.0
**Date:** 2025-11-08
**Purpose:** Step-by-step implementation guide for RDF workflow execution

## Overview

This guide provides concrete implementation steps for integrating RDF workflow execution into KNHK, based on the [RDF Workflow Execution Architecture](./rdf-workflow-execution.md).

---

## 1. Quick Start: Execute ATM Workflow in 4 Steps

### Step 1: Load Workflow from Turtle

```rust
use knhk_workflow_engine::WorkflowEngine;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize engine
    let mut engine = WorkflowEngine::new().await?;

    // Load ATM workflow from .ttl file
    let turtle = fs::read_to_string(
        "ontology/workflows/financial/atm_transaction.ttl"
    )?;

    let spec_id = engine.register_workflow_from_rdf(&turtle).await?;

    println!("✅ Workflow registered: {}", spec_id);

    Ok(())
}
```

### Step 2: Create Case with Initial Data

```rust
use serde_json::json;

// Create case with ATM transaction data
let case_data = json!({
    "cardNumber": "1234-5678-9012-3456",
    "pin": "1234",
    "accountNumber": "ACC123",
    "balance": 1000.00,
    "withdrawalAmount": 200.00
});

let case_id = engine.create_case(spec_id, case_data).await?;

println!("✅ Case created: {}", case_id);
```

### Step 3: Execute Workflow

```rust
// Execute workflow end-to-end
engine.execute_case_with_telemetry(case_id).await?;

// Get case to verify completion
let case = engine.get_case(&case_id).await?;

println!("✅ Case status: {:?}", case.state);
```

### Step 4: Query Results with SPARQL

```rust
// Query final balance
let results = engine.query_case_rdf(
    &case_id,
    r#"
    PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
    SELECT ?balance WHERE {
        ?case yawl:balance ?balance .
    }
    "#
).await?;

println!("✅ Final balance: {}", results[0]["balance"]);
// Output: Final balance: 800.00
```

---

## 2. Implementation Checklist

### Phase 1: Core RDF Execution (Week 1)

#### Task 1.1: Implement RdfWorkflowLoader

**File:** `rust/knhk-workflow-engine/src/rdf/loader.rs`

```rust
use oxigraph::store::Store;
use oxigraph::io::RdfFormat;
use crate::parser::WorkflowSpec;
use crate::validation::DeadlockDetector;

pub struct RdfWorkflowLoader {
    parser_store: Store,
    yawl_ontology: Option<Store>,
}

impl RdfWorkflowLoader {
    pub fn new() -> WorkflowResult<Self> {
        Ok(Self {
            parser_store: Store::new()?,
            yawl_ontology: None,
        })
    }

    pub async fn load_yawl_ontology(&mut self, path: &Path) -> WorkflowResult<()> {
        let mut store = Store::new()?;
        let turtle = fs::read_to_string(path)?;
        store.load_from_reader(RdfFormat::Turtle, turtle.as_bytes())?;
        self.yawl_ontology = Some(store);
        Ok(())
    }

    pub async fn load_from_file(&mut self, path: &Path) -> WorkflowResult<WorkflowSpec> {
        // TODO: Implement
        // 1. Read .ttl file
        // 2. Parse into parser_store
        // 3. SHACL validation
        // 4. Extract WorkflowSpec via SPARQL
        // 5. Deadlock detection
        unimplemented!("RdfWorkflowLoader::load_from_file")
    }

    pub async fn load_from_string(&mut self, turtle: &str) -> WorkflowResult<WorkflowSpec> {
        // TODO: Implement
        // Same as load_from_file but takes string
        unimplemented!("RdfWorkflowLoader::load_from_string")
    }

    fn extract_workflow_spec(&self) -> WorkflowResult<WorkflowSpec> {
        // TODO: Implement SPARQL queries to extract:
        // - Specification metadata
        // - Tasks
        // - Conditions
        // - Flows
        // - Parameters
        unimplemented!("RdfWorkflowLoader::extract_workflow_spec")
    }
}
```

**Tests:** `tests/rdf_workflow_loader_test.rs`

```rust
#[tokio::test]
async fn test_load_atm_workflow() {
    let mut loader = RdfWorkflowLoader::new().unwrap();

    // Load YAWL ontology first
    loader.load_yawl_ontology(Path::new("ontology/yawl.ttl"))
        .await
        .unwrap();

    // Load ATM workflow
    let spec = loader.load_from_file(
        Path::new("ontology/workflows/financial/atm_transaction.ttl")
    ).await.unwrap();

    // Verify spec
    assert_eq!(spec.name, "ATM Cash Withdrawal");
    assert!(spec.tasks.contains_key("verify_card"));
    assert!(spec.tasks.contains_key("dispense_cash"));

    // Weaver validation happens automatically via OTEL integration
}
```

**Validation:**

```bash
# Run test
cargo test --test rdf_workflow_loader_test -- --nocapture

# Weaver validation (MANDATORY)
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

#### Task 1.2: Add WorkflowEngine::register_workflow_from_rdf()

**File:** `rust/knhk-workflow-engine/src/executor/workflow_registration.rs`

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
        self.specs.insert(spec_id, spec.clone());

        // 4. Save to state manager
        self.state_manager.save_spec(&spec).await?;

        // 5. Emit telemetry
        if let Some(otel) = &self.otel_integration {
            otel.record_workflow_registration(spec_id);
        }

        info!(spec_id = %spec_id, "Workflow registered from RDF");
        Ok(spec_id)
    }
}
```

**Tests:** `tests/chicago_tdd_atm_workflow.rs`

```rust
#[tokio::test]
async fn test_register_atm_workflow() {
    let mut engine = WorkflowEngine::new().await.unwrap();

    let turtle = fs::read_to_string(
        "ontology/workflows/financial/atm_transaction.ttl"
    ).unwrap();

    let spec_id = engine.register_workflow_from_rdf(&turtle)
        .await
        .unwrap();

    // Verify spec registered
    let spec = engine.specs.get(&spec_id).unwrap();
    assert_eq!(spec.name, "ATM Cash Withdrawal");

    // Verify RDF store populated
    let results = engine.query_rdf(
        &spec_id,
        r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?task WHERE { ?spec yawl:hasTask ?task }
        "#
    ).await.unwrap();

    assert!(results.len() >= 7); // ATM workflow has 7 tasks
}
```

#### Task 1.3: Implement execute_case_with_telemetry()

**File:** `rust/knhk-workflow-engine/src/executor/case.rs`

```rust
impl WorkflowEngine {
    /// Execute case with full telemetry
    pub async fn execute_case_with_telemetry(
        &mut self,
        case_id: CaseId
    ) -> WorkflowResult<()> {
        let span = info_span!("execute_case", case_id = %case_id);
        let _enter = span.enter();

        // 1. Load case
        let mut case = self.cases.get(&case_id)
            .ok_or(WorkflowError::CaseNotFound(case_id))?
            .clone();

        // 2. Load workflow spec
        let spec = self.specs.get(&case.spec_id)
            .ok_or(WorkflowError::SpecNotFound(case.spec_id))?
            .clone();

        // 3. Execute all enabled tasks
        while !case.enabled_tasks.is_empty() {
            let task_id = case.enabled_tasks.iter().next().unwrap().clone();
            let task = spec.tasks.get(&task_id)
                .ok_or(WorkflowError::TaskNotFound(task_id.clone()))?;

            // Execute task with pattern
            let result = self.execute_task_with_pattern(&case_id, task).await?;

            // Update case state
            self.update_case_state(case_id, &result).await?;

            // Reload case
            case = self.cases.get(&case_id).unwrap().clone();
        }

        info!("Case execution complete");
        Ok(())
    }

    async fn execute_task_with_pattern(
        &mut self,
        case_id: &CaseId,
        task: &Task
    ) -> WorkflowResult<PatternExecutionResult> {
        // TODO: Implement
        // 1. Identify pattern from task
        // 2. Get pattern executor
        // 3. Build execution context
        // 4. Execute pattern (emits OTEL spans)
        // 5. Validate result (check ticks ≤8)
        unimplemented!("WorkflowEngine::execute_task_with_pattern")
    }
}
```

**Tests:** `tests/chicago_tdd_atm_e2e.rs`

```rust
#[tokio::test]
async fn test_atm_workflow_end_to_end() {
    // Setup
    let mut engine = WorkflowEngine::new().await.unwrap();

    // Load workflow
    let turtle = fs::read_to_string(
        "ontology/workflows/financial/atm_transaction.ttl"
    ).unwrap();
    let spec_id = engine.register_workflow_from_rdf(&turtle)
        .await
        .unwrap();

    // Create case
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

    // Verify completion
    let case = engine.get_case(&case_id).await.unwrap();
    assert_eq!(case.state, CaseState::Completed);

    // Verify balance updated
    let results = engine.query_case_rdf(
        &case_id,
        r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?balance WHERE { ?case yawl:balance ?balance }
        "#
    ).await.unwrap();

    assert_eq!(results[0]["balance"], "800.00");
}
```

---

## 3. Key Integration Points

### 3.1 Existing Code to Modify

| File | Changes Required |
|------|-----------------|
| `src/executor/mod.rs` | Add `rdf_query` module export |
| `src/executor/engine.rs` | Add RDF store fields (already done) |
| `src/executor/workflow_registration.rs` | Add `register_workflow_from_rdf()` |
| `src/executor/case.rs` | Add `execute_case_with_telemetry()` |
| `src/parser/mod.rs` | Add `RdfWorkflowLoader` |
| `src/state/manager.rs` | Add `save_case_with_rdf()` |

### 3.2 New Files to Create

| File | Purpose |
|------|---------|
| `src/rdf/loader.rs` | RDF workflow loader |
| `src/rdf/extractor.rs` | SPARQL query extraction |
| `src/rdf/validator.rs` | SHACL validation |
| `tests/rdf_workflow_loader_test.rs` | Loader tests |
| `tests/chicago_tdd_atm_workflow.rs` | ATM workflow tests |
| `tests/chicago_tdd_atm_e2e.rs` | End-to-end tests |

### 3.3 Dependencies to Add

**Cargo.toml:**

```toml
[dependencies]
# Already present
oxigraph = "0.4"

# May need to add
shacl = "0.1"  # SHACL validation (if available)
```

---

## 4. Testing Strategy

### 4.1 Unit Tests (Chicago TDD Style)

**Test RDF Loader:**

```rust
#[tokio::test]
async fn test_parse_simple_workflow() {
    let turtle = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .

        <http://example.org/workflow> a yawl:Specification ;
            yawl:specName "Simple" ;
            yawl:hasTask <http://example.org/task1> .

        <http://example.org/task1> a yawl:AtomicTask ;
            yawl:taskName "Task 1" ;
            yawl:join yawl:ControlTypeXor ;
            yawl:split yawl:ControlTypeXor .
    "#;

    let mut loader = RdfWorkflowLoader::new().unwrap();
    let spec = loader.load_from_string(turtle).await.unwrap();

    assert_eq!(spec.name, "Simple");
    assert_eq!(spec.tasks.len(), 1);
}
```

### 4.2 Integration Tests (Real Workflows)

**Test ATM Workflow:**

```rust
#[tokio::test]
async fn test_atm_workflow_registration() {
    let mut engine = WorkflowEngine::new().await.unwrap();

    let turtle = fs::read_to_string(
        "ontology/workflows/financial/atm_transaction.ttl"
    ).unwrap();

    let spec_id = engine.register_workflow_from_rdf(&turtle)
        .await
        .unwrap();

    // Verify all tasks loaded
    let spec = engine.specs.get(&spec_id).unwrap();
    assert!(spec.tasks.contains_key("verify_card"));
    assert!(spec.tasks.contains_key("verify_pin"));
    assert!(spec.tasks.contains_key("check_balance"));
    assert!(spec.tasks.contains_key("dispense_cash"));
    assert!(spec.tasks.contains_key("update_balance"));
    assert!(spec.tasks.contains_key("print_receipt"));
    assert!(spec.tasks.contains_key("cancel_transaction"));
}
```

### 4.3 End-to-End Tests (Full Execution)

**Test Complete ATM Flow:**

```rust
#[tokio::test]
async fn test_atm_complete_flow() {
    let mut engine = WorkflowEngine::new().await.unwrap();

    // Register workflow
    let turtle = fs::read_to_string(
        "ontology/workflows/financial/atm_transaction.ttl"
    ).unwrap();
    let spec_id = engine.register_workflow_from_rdf(&turtle)
        .await
        .unwrap();

    // Create case (successful withdrawal)
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

    // Execute
    engine.execute_case_with_telemetry(case_id)
        .await
        .unwrap();

    // Verify: Case completed
    let case = engine.get_case(&case_id).await.unwrap();
    assert_eq!(case.state, CaseState::Completed);

    // Verify: Balance deducted
    let results = engine.query_case_rdf(
        &case_id,
        r#"SELECT ?balance WHERE { ?case yawl:balance ?balance }"#
    ).await.unwrap();
    assert_eq!(results[0]["balance"], "800.00");

    // Verify: Cash dispensed
    let results = engine.query_case_rdf(
        &case_id,
        r#"SELECT ?dispensed WHERE { ?case yawl:cashDispensed ?dispensed }"#
    ).await.unwrap();
    assert_eq!(results[0]["dispensed"], "200.00");
}
```

### 4.4 Weaver Validation Tests

**Test Telemetry Emission:**

```bash
#!/bin/bash
# test_weaver_validation.sh

set -e

# Run test with OTEL enabled
OTEL_ENABLED=1 cargo test --test chicago_tdd_atm_e2e -- --nocapture

# Validate with Weaver
weaver registry check -r registry/
weaver registry live-check --registry registry/

if [ $? -eq 0 ]; then
    echo "✅ Weaver validation passed"
else
    echo "❌ Weaver validation FAILED"
    exit 1
fi
```

---

## 5. Performance Targets

### 5.1 Hot Path Performance (≤8 Ticks)

**Operations that MUST be ≤8 ticks:**

| Operation | Target Ticks | Optimization |
|-----------|-------------|--------------|
| Pattern execution | 3-5 ticks | SIMD, compiled patterns |
| Resource allocation | 1 tick | Pre-allocated pool |
| State update | 2 ticks | Write-behind cache |
| RDF query (hot path) | 1 tick | In-memory index |
| **Total** | **≤8 ticks** | |

### 5.2 SLO Targets

| Operation | SLO | Measurement |
|-----------|-----|-------------|
| Workflow registration | <500ms | P95 latency |
| Case creation | <100ms | P95 latency |
| Case execution (ATM) | <1s | P95 latency |
| SPARQL query | <50ms | P95 latency |

### 5.3 Performance Tests

```rust
#[tokio::test]
async fn test_hot_path_performance() {
    let mut engine = WorkflowEngine::new().await.unwrap();

    // ... setup workflow and case ...

    // Measure execution ticks
    let start_tick = engine.timer_service.current_tick();

    engine.execute_task_with_pattern(&case_id, &task)
        .await
        .unwrap();

    let elapsed_ticks = engine.timer_service.current_tick() - start_tick;

    // MUST be ≤8 ticks (Chatman Constant)
    assert!(
        elapsed_ticks <= 8,
        "Hot path violated Chatman Constant: {} ticks",
        elapsed_ticks
    );
}
```

---

## 6. Common Pitfalls & Solutions

### Pitfall 1: SPARQL Query Performance

**Problem:** SPARQL queries are slow (>50ms).

**Solution:**
- Use Oxigraph in-memory store (not persistent)
- Create indexes on common query patterns
- Cache query results
- Use LIMIT clause

```rust
// ❌ BAD: Unbounded query
SELECT ?task WHERE { ?spec yawl:hasTask ?task }

// ✅ GOOD: Limit results
SELECT ?task WHERE { ?spec yawl:hasTask ?task } LIMIT 100
```

### Pitfall 2: RDF Store Memory Growth

**Problem:** `case_rdf_stores` grows unbounded.

**Solution:**
- Export case RDF to Turtle on completion
- Remove from HashMap after export
- Implement cleanup policy (e.g., after 24h)

```rust
impl WorkflowEngine {
    pub async fn cleanup_completed_cases(&mut self) -> WorkflowResult<()> {
        let completed: Vec<CaseId> = self.cases
            .iter()
            .filter(|entry| entry.value().state == CaseState::Completed)
            .map(|entry| *entry.key())
            .collect();

        for case_id in completed {
            // Export to Turtle
            let turtle = self.export_case_rdf(&case_id).await?;

            // Store in Sled
            self.state_manager.save_case_turtle(case_id, &turtle).await?;

            // Remove from memory
            self.case_rdf_stores.write().await.remove(&case_id);
        }

        Ok(())
    }
}
```

### Pitfall 3: SHACL Validation Overhead

**Problem:** SHACL validation is slow (>100ms).

**Solution:**
- Cache validation results
- Validate only on registration (not every execution)
- Use incremental validation

```rust
// Validate ONCE on registration
let spec = loader.load_from_string(turtle)?; // Validates here

// Don't re-validate on execution
engine.execute_case(case_id).await?; // No validation
```

### Pitfall 4: Pattern Identification Ambiguity

**Problem:** Can't determine which pattern a task uses.

**Solution:**
- Explicit pattern annotation in Turtle
- Pattern inference rules
- Default to Pattern 1 (Sequence)

```turtle
# ✅ GOOD: Explicit pattern
<task1> a yawl:AtomicTask ;
    yawl:pattern pattern:Pattern4 ;  # XOR-split
    yawl:split yawl:ControlTypeXor .

# ❌ BAD: Ambiguous
<task1> a yawl:AtomicTask ;
    yawl:split yawl:ControlTypeXor .  # Which pattern?
```

---

## 7. Next Steps

### Immediate (Week 1)

1. **Implement `RdfWorkflowLoader`**
   - File: `src/rdf/loader.rs`
   - Tests: `tests/rdf_workflow_loader_test.rs`
   - Validation: `cargo test --test rdf_workflow_loader_test`

2. **Add `register_workflow_from_rdf()`**
   - File: `src/executor/workflow_registration.rs`
   - Tests: `tests/chicago_tdd_atm_workflow.rs`
   - Validation: Weaver check

3. **Implement `execute_case_with_telemetry()`**
   - File: `src/executor/case.rs`
   - Tests: `tests/chicago_tdd_atm_e2e.rs`
   - Validation: Weaver live-check

### Near-Term (Week 2-3)

4. **Pattern Metadata Integration**
   - Load 43 patterns into `pattern_metadata_store`
   - SPARQL queries for pattern discovery

5. **SPARQL Query API**
   - REST endpoint `/sparql`
   - Query validation
   - Result formatting

6. **Performance Optimization**
   - Hot path ≤8 ticks
   - SPARQL query caching
   - Write-behind state updates

### Long-Term (Week 4+)

7. **Fortune 5 Features**
   - Multi-region deployment
   - SPIFFE authentication
   - KMS integration
   - SLO enforcement

8. **Production Readiness**
   - Error handling
   - Rollback mechanisms
   - Monitoring dashboards
   - Alert policies

---

## 8. Resources

### Documentation

- [RDF Workflow Execution Architecture](./rdf-workflow-execution.md)
- [C4 Component Diagram](./c4-rdf-workflow-execution.puml)
- [Sequence Diagram](./sequence-rdf-workflow-execution.puml)
- [ADR-002: Turtle vs YAWL XML](./ADR/ADR-002-turtle-vs-yawl-xml.md)

### Example Workflows

- [ATM Transaction](../../ontology/workflows/financial/atm_transaction.ttl)
- [SWIFT Payment](../../ontology/workflows/financial/swift_payment.ttl)
- [Payroll](../../ontology/workflows/financial/payroll.ttl)

### External References

- [RDF 1.1 Turtle](https://www.w3.org/TR/turtle/)
- [SPARQL 1.1 Query](https://www.w3.org/TR/sparql11-query/)
- [SHACL Validation](https://www.w3.org/TR/shacl/)
- [Oxigraph Documentation](https://github.com/oxigraph/oxigraph)
- [OpenTelemetry Weaver](https://github.com/open-telemetry/weaver)

---

**END OF IMPLEMENTATION GUIDE**
