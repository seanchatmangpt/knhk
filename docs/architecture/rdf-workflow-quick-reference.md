# RDF Workflow Execution - Quick Reference Card

**Version:** 1.0 | **Date:** 2025-11-08

## 🚀 Quick Start (5 Minutes)

### Load and Execute ATM Workflow

```rust
use knhk_workflow_engine::WorkflowEngine;
use serde_json::json;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize engine
    let mut engine = WorkflowEngine::new().await?;

    // 2. Load workflow
    let turtle = fs::read_to_string("ontology/workflows/financial/atm_transaction.ttl")?;
    let spec_id = engine.register_workflow_from_rdf(&turtle).await?;

    // 3. Create case
    let case_data = json!({"cardNumber": "1234", "balance": 1000.0, "amount": 200.0});
    let case_id = engine.create_case(spec_id, case_data).await?;

    // 4. Execute
    engine.execute_case_with_telemetry(case_id).await?;

    // 5. Query results
    let results = engine.query_case_rdf(&case_id, "SELECT ?balance WHERE { ?case yawl:balance ?balance }").await?;
    println!("Balance: {}", results[0]["balance"]); // 800.0

    Ok(())
}
```

---

## 📁 File Structure

```
docs/architecture/
├── README-RDF-WORKFLOW-ARCHITECTURE.md       ← START HERE (Index)
├── rdf-workflow-execution.md                 ← Full Architecture (System Architects)
├── rdf-workflow-implementation-guide.md      ← Step-by-step (Developers)
├── rdf-workflow-quick-reference.md           ← This file (Quick lookup)
├── c4-rdf-workflow-execution.puml            ← Component Diagram
├── sequence-rdf-workflow-execution.puml      ← Sequence Diagram
└── dataflow-rdf-workflow.puml                ← Data Flow Diagram
```

---

## 🏗️ Architecture Components

| Component | Purpose | File |
|-----------|---------|------|
| **RdfWorkflowLoader** | Parse .ttl → WorkflowSpec | `src/rdf/loader.rs` |
| **WorkflowEngine** | Orchestrate execution | `src/executor/engine.rs` |
| **PatternExecutor** | Execute patterns 1-43 | `src/patterns/mod.rs` |
| **StateManager** | Persist state (Sled + RDF) | `src/state/manager.rs` |
| **OtelIntegration** | Emit telemetry | `src/integration/otel.rs` |

---

## 🗄️ RDF Stores (3 Tiers)

| Store | Scope | Mutability | Purpose |
|-------|-------|-----------|---------|
| **spec_rdf_store** | All workflows | Immutable | Query workflow structure |
| **pattern_metadata_store** | 43 patterns | Immutable | Query pattern metadata |
| **case_rdf_stores** | Per-case | Mutable | Runtime variables |

---

## 🔍 SPARQL Query Examples

### Query Workflow Structure

```sparql
PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
SELECT ?task ?name WHERE {
    ?spec yawl:hasTask ?task .
    ?task yawl:taskName ?name .
}
```

### Query Case Runtime State

```sparql
PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
SELECT ?balance ?amount WHERE {
    ?case yawl:balance ?balance ;
          yawl:withdrawalAmount ?amount .
}
```

### Query Pattern Metadata

```sparql
PREFIX pattern: <http://bitflow.ai/ontology/workflow-pattern/v1#>
SELECT ?pattern ?name ?category WHERE {
    ?pattern a pattern:Pattern ;
             pattern:name ?name ;
             pattern:category ?category .
}
```

---

## ⚡ Performance Constraints

| Metric | Target | Enforcement |
|--------|--------|-------------|
| **Hot path** | ≤8 ticks | Chatman Constant (CRITICAL) |
| Workflow registration | <500ms P95 | SLO |
| Case creation | <100ms P95 | SLO |
| SPARQL query | <50ms P95 | SLO |

**Monitor:**
```rust
// Emit metric
self.otel_integration.record_hot_path_ticks(elapsed_ticks);

// Alert if violated
if elapsed_ticks > 8 {
    self.otel_integration.record_chatman_violation();
}
```

---

## ✅ Validation Hierarchy (CRITICAL)

**THE ONLY SOURCE OF TRUTH: OpenTelemetry Weaver**

```bash
# Level 1: Weaver Validation (MANDATORY - Source of Truth)
weaver registry check -r registry/                    # Schema is valid
weaver registry live-check --registry registry/       # Runtime telemetry conforms

# Level 2: Compilation & Code Quality (Baseline)
cargo build --release                                 # Must compile
cargo clippy --workspace -- -D warnings               # Zero warnings

# Level 3: Traditional Tests (Supporting Evidence - Can Have False Positives)
cargo test --workspace                                # Rust unit tests
```

**⚠️ CRITICAL:** If Weaver fails, the feature DOES NOT WORK, regardless of test results.

---

## 🧪 Testing Commands

### Unit Tests

```bash
# Test RDF loader
cargo test --test rdf_workflow_loader_test -- --nocapture

# Test ATM workflow registration
cargo test --test chicago_tdd_atm_workflow -- --nocapture

# Test end-to-end execution
cargo test --test chicago_tdd_atm_e2e -- --nocapture
```

### Weaver Validation (MANDATORY)

```bash
# Validate schema
weaver registry check -r registry/

# Validate runtime telemetry
weaver registry live-check --registry registry/
```

### Performance Benchmarks

```bash
# Hot path performance
cargo test --test performance_hot_path -- --nocapture

# SPARQL query performance
cargo test --test performance_sparql_queries -- --nocapture
```

---

## 📊 Key OTEL Spans

| Span Name | Attributes | Purpose |
|-----------|-----------|---------|
| `workflow.registration` | `spec_id`, `latency_ms` | Workflow loading |
| `case.creation` | `case_id`, `spec_id` | Case initialization |
| `workflow.execution` | `case_id`, `ticks`, `status` | Full execution |
| `pattern.execution` | `pattern_id`, `task_id`, `ticks` | Pattern execution |
| `rdf.query` | `query_type`, `latency_ms` | SPARQL queries |

---

## 🐛 Common Pitfalls

| Problem | Solution |
|---------|----------|
| SPARQL queries slow (>50ms) | Use LIMIT, index common patterns |
| Memory growth in `case_rdf_stores` | Export to Turtle, cleanup completed cases |
| SHACL validation slow (>100ms) | Validate once on registration, not execution |
| Pattern identification ambiguous | Use explicit `yawl:pattern` annotation |
| Hot path >8 ticks | Pre-allocate resources, use SIMD |

---

## 🔐 Security Checklist

- [ ] Sanitize SPARQL queries (block DROP, DELETE, INSERT, UPDATE)
- [ ] Only allow SELECT and ASK queries
- [ ] Implement role-based access control (RBAC)
- [ ] Encrypt Sled database with KMS
- [ ] Use TLS for all API endpoints
- [ ] Use SPIFFE mTLS for service-to-service

---

## 📈 Monitoring Dashboard

**Key Metrics to Watch:**

```
┌─────────────────────────────────────────┐
│ Case Execution Rate: 120/min           │
├─────────────────────────────────────────┤
│ Hot Path P95: 6 ticks ✅                │
├─────────────────────────────────────────┤
│ Chatman Violations: 0 ✅                │
├─────────────────────────────────────────┤
│ SPARQL Query P99: 35ms ✅               │
└─────────────────────────────────────────┘
```

**Alerts:**
- 🔴 CRITICAL: `hot_path_ticks > 8`
- 🔴 CRITICAL: `chatman_violations_total > 0`
- 🟡 WARNING: `sparql_query_latency_ms > 50`

---

## 📝 Implementation Checklist

### Phase 1: Core RDF Execution (Week 1)

- [ ] Implement `RdfWorkflowLoader`
  - [ ] Parse Turtle RDF
  - [ ] SHACL validation
  - [ ] SPARQL extraction
  - [ ] Deadlock detection
- [ ] Add `WorkflowEngine::register_workflow_from_rdf()`
  - [ ] Load into `spec_rdf_store`
  - [ ] Persist to Sled
  - [ ] Emit OTEL span
- [ ] Add `WorkflowEngine::execute_case_with_telemetry()`
  - [ ] Execute patterns
  - [ ] Update state
  - [ ] Emit OTEL spans
- [ ] Chicago TDD tests
  - [ ] Load ATM workflow
  - [ ] Execute end-to-end
  - [ ] Verify results
  - [ ] **Weaver validation (MANDATORY)**

---

## 🎯 Success Criteria

**A feature is COMPLETE when:**

1. ✅ Code compiles (`cargo build --release`)
2. ✅ Zero clippy warnings (`cargo clippy --workspace -- -D warnings`)
3. ✅ Tests pass (`cargo test --workspace`)
4. ✅ **Weaver validation passes** (`weaver registry live-check`)
5. ✅ Performance targets met (hot path ≤8 ticks)
6. ✅ Documentation updated

**⚠️ CRITICAL:** Only criterion #4 (Weaver validation) is the source of truth. All others are baseline requirements.

---

## 📚 Resources

### Documentation

- **Full Architecture:** [rdf-workflow-execution.md](./rdf-workflow-execution.md)
- **Implementation Guide:** [rdf-workflow-implementation-guide.md](./rdf-workflow-implementation-guide.md)
- **README Index:** [README-RDF-WORKFLOW-ARCHITECTURE.md](./README-RDF-WORKFLOW-ARCHITECTURE.md)

### Example Workflows

- **ATM Transaction:** `ontology/workflows/financial/atm_transaction.ttl`
- **SWIFT Payment:** `ontology/workflows/financial/swift_payment.ttl`
- **Payroll:** `ontology/workflows/financial/payroll.ttl`

### External Standards

- [RDF 1.1 Turtle](https://www.w3.org/TR/turtle/)
- [SPARQL 1.1 Query](https://www.w3.org/TR/sparql11-query/)
- [SHACL Validation](https://www.w3.org/TR/shacl/)
- [OpenTelemetry Weaver](https://github.com/open-telemetry/weaver)

---

## 💡 Quick Tips

1. **Always use Weaver validation** - It's the only source of truth
2. **Hot path MUST be ≤8 ticks** - Use profiling to identify bottlenecks
3. **SPARQL queries should be <50ms** - Use LIMIT and indexes
4. **Export case RDF on completion** - Prevent memory growth
5. **Validate once on registration** - Not on every execution
6. **Use real collaborators in tests** - Chicago TDD (no mocks)

---

**Last Updated:** 2025-11-08
**Version:** 1.0

---

**END OF QUICK REFERENCE**
