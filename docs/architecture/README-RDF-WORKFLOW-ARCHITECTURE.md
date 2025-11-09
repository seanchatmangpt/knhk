# RDF Workflow Execution Architecture - Documentation Index

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Approved

## Overview

This directory contains the complete architectural specification for RDF workflow execution in KNHK. The architecture enables loading, parsing, validating, and executing YAWL workflows defined in Turtle RDF format with comprehensive OpenTelemetry observability.

---

## Document Index

### 1. Core Architecture Documents

| Document | Purpose | Audience |
|----------|---------|----------|
| [**rdf-workflow-execution.md**](./rdf-workflow-execution.md) | Complete architectural specification | System Architects, Tech Leads |
| [**rdf-workflow-implementation-guide.md**](./rdf-workflow-implementation-guide.md) | Step-by-step implementation guide | Developers, Engineers |

### 2. Visual Architecture Diagrams

| Diagram | Type | Purpose |
|---------|------|---------|
| [**c4-rdf-workflow-execution.puml**](./c4-rdf-workflow-execution.puml) | C4 Component | System components and relationships |
| [**sequence-rdf-workflow-execution.puml**](./sequence-rdf-workflow-execution.puml) | Sequence | End-to-end execution flow |
| [**dataflow-rdf-workflow.puml**](./dataflow-rdf-workflow.puml) | Data Flow | Data transformation pipeline |

### 3. Related Architecture Documents

| Document | Relevance |
|----------|-----------|
| [ADR-002: Turtle vs YAWL XML](./ADR/ADR-002-turtle-vs-yawl-xml.md) | Why we use Turtle RDF as primary format |
| [ADR-001: Interface B Work Item Lifecycle](./ADR-001-interface-b-work-item-lifecycle.md) | Work item integration |
| [c4-component-engine.puml](./c4-component-engine.puml) | Engine component diagram |

---

## Quick Start

### For System Architects

**Start here:** [rdf-workflow-execution.md](./rdf-workflow-execution.md)

Key sections:
- Section 1: Architecture Overview
- Section 2: Component Architecture (C4 Level 3)
- Section 5: RDF Store Architecture
- Section 6: Performance Architecture (Chatman Constant)

**Then review:** PlantUML diagrams for visual understanding

### For Developers

**Start here:** [rdf-workflow-implementation-guide.md](./rdf-workflow-implementation-guide.md)

Key sections:
- Section 1: Quick Start (Execute ATM Workflow in 4 Steps)
- Section 2: Implementation Checklist
- Section 3: Key Integration Points
- Section 4: Testing Strategy

**Then review:** Code examples in the implementation guide

### For QA/Testers

**Start here:** [rdf-workflow-implementation-guide.md](./rdf-workflow-implementation-guide.md)

Key sections:
- Section 4: Testing Strategy
- Section 5: Performance Targets
- Section 6: Common Pitfalls & Solutions

**Critical:** All testing MUST include Weaver validation (source of truth)

---

## Architecture Highlights

### Key Capabilities

✅ **RDF-Native Execution:** Load `.ttl` workflows directly into engine
✅ **Pattern Execution:** Execute all 43 Van der Aalst workflow patterns
✅ **Schema Validation:** SHACL soundness validation before execution
✅ **Telemetry-First:** Every operation emits OTEL spans/metrics/logs
✅ **State Persistence:** Event-sourced case management with Sled
✅ **Performance:** ≤8 ticks for hot path operations (Chatman Constant)
✅ **Fortune 5:** Multi-region, SPIFFE, KMS, SLO enforcement

### Critical Design Decisions

**1. Turtle RDF as Primary Format**
- 77% smaller than YAWL XML
- Semantic richness (RDF triples)
- SPARQL query capabilities
- See: [ADR-002](./ADR/ADR-002-turtle-vs-yawl-xml.md)

**2. Three-Tier RDF Store Architecture**
- `spec_rdf_store`: Workflow specifications (immutable, shared)
- `pattern_metadata_store`: 43 pattern metadata (immutable, shared)
- `case_rdf_stores`: Runtime state (mutable, per-case)

**3. Weaver Validation as Source of Truth**
- Traditional tests can produce false positives
- Only Weaver validation proves features work
- Schema-first approach: code must conform to declared telemetry

**4. Performance Constraints**
- Hot path: ≤8 ticks (Chatman Constant)
- Workflow registration: <500ms (P95)
- Case creation: <100ms (P95)
- SPARQL query: <50ms (P95)

---

## Implementation Roadmap

### Phase 1: Core RDF Execution (Week 1) - HIGH PRIORITY

**Deliverables:**
- `RdfWorkflowLoader` implementation
- `WorkflowEngine::register_workflow_from_rdf()`
- `WorkflowEngine::execute_case_with_telemetry()`
- Chicago TDD tests (ATM workflow end-to-end)

**Success Criteria:**
```bash
# Execute ATM workflow end-to-end
cargo test --test chicago_tdd_atm_workflow -- --nocapture

# Weaver validation (MANDATORY)
weaver registry check -r registry/
weaver registry live-check --registry registry/
```

### Phase 2: Pattern Metadata & SPARQL (Week 2) - MEDIUM PRIORITY

**Deliverables:**
- Load 43 patterns into `pattern_metadata_store`
- SPARQL query API (`query_rdf()`, `query_case_rdf()`)
- REST endpoint `/sparql`

### Phase 3: Performance Optimization (Week 3) - HIGH PRIORITY

**Deliverables:**
- Hot path optimization (≤8 ticks)
- Performance monitoring (tick histogram, SLO violations)
- Benchmarks (ATM workflow execution)

### Phase 4: Fortune 5 Features (Week 4) - ENTERPRISE PRIORITY

**Deliverables:**
- Multi-region deployment
- SPIFFE authentication
- KMS integration
- SLO enforcement

---

## Testing Requirements

### Critical: Weaver Validation

**MANDATORY:** All features MUST pass Weaver validation.

```bash
# 1. Run tests
cargo test --test chicago_tdd_atm_e2e -- --nocapture

# 2. Validate with Weaver (SOURCE OF TRUTH)
weaver registry check -r registry/
weaver registry live-check --registry registry/

# ✅ Only if Weaver passes = Feature works
# ❌ If Weaver fails = Feature DOES NOT WORK (regardless of test results)
```

### Test Hierarchy

**Level 1: Weaver Schema Validation (MANDATORY)**
```bash
weaver registry check -r registry/          # Schema is valid
weaver registry live-check --registry registry/  # Runtime telemetry conforms
```

**Level 2: Compilation & Code Quality (Baseline)**
```bash
cargo build --release                       # Must compile
cargo clippy --workspace -- -D warnings     # Zero warnings
```

**Level 3: Traditional Tests (Supporting Evidence)**
```bash
cargo test --workspace                      # Rust unit tests
make test-chicago-v04                       # C Chicago TDD tests
```

**⚠️ WARNING:** Tests at Level 3 can pass even when features are broken (false positives). Only Weaver validation (Level 1) is the source of truth.

---

## Performance Requirements

### Chatman Constant (≤8 Ticks)

**Critical constraint:** Hot path operations MUST complete in ≤8 ticks.

```rust
const CHATMAN_CONSTANT: u32 = 8;

// Performance targets for hot path
// - Resource allocation: 1 tick
// - Pattern execution: 3-5 ticks
// - State update: 2 ticks
// TOTAL: ≤8 ticks
```

**Monitoring:**
- Emit `hot_path_ticks` histogram metric
- Alert on violations: `chatman_violations_total`
- Track P95/P99 latencies

### SLO Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Workflow registration | <500ms | P95 latency |
| Case creation | <100ms | P95 latency |
| Case execution (ATM) | <1s | P95 latency |
| SPARQL query | <50ms | P95 latency |
| Hot path task execution | ≤8 ticks | P99 ticks |

---

## Example Workflows

### Available Financial Workflows

| Workflow | File | Patterns Used |
|----------|------|---------------|
| **ATM Cash Withdrawal** | `ontology/workflows/financial/atm_transaction.ttl` | Sequence (1), XOR Choice (4), Deferred Choice (16), Cancellation (19) |
| **SWIFT Payment** | `ontology/workflows/financial/swift_payment.ttl` | Multi-instance (12-15), Cancellation (19), Milestone (18) |
| **Payroll Processing** | `ontology/workflows/financial/payroll.ttl` | Parallel Split (2), Synchronization (3), Multi-instance (12-15) |

### Quick Example: Load and Execute ATM Workflow

```rust
use knhk_workflow_engine::WorkflowEngine;
use serde_json::json;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize engine
    let mut engine = WorkflowEngine::new().await?;

    // 2. Load ATM workflow
    let turtle = fs::read_to_string(
        "ontology/workflows/financial/atm_transaction.ttl"
    )?;
    let spec_id = engine.register_workflow_from_rdf(&turtle).await?;

    // 3. Create case
    let case_data = json!({
        "cardNumber": "1234-5678-9012-3456",
        "pin": "1234",
        "accountNumber": "ACC123",
        "balance": 1000.00,
        "withdrawalAmount": 200.00
    });
    let case_id = engine.create_case(spec_id, case_data).await?;

    // 4. Execute workflow
    engine.execute_case_with_telemetry(case_id).await?;

    // 5. Query results
    let results = engine.query_case_rdf(
        &case_id,
        r#"SELECT ?balance WHERE { ?case yawl:balance ?balance }"#
    ).await?;

    println!("Final balance: {}", results[0]["balance"]);
    // Output: Final balance: 800.00

    Ok(())
}
```

---

## Architecture Diagrams

### How to View Diagrams

**PlantUML diagrams (.puml files):**

1. **VS Code:** Install "PlantUML" extension
2. **IntelliJ:** Built-in PlantUML support
3. **CLI:** `plantuml diagram.puml` → generates PNG
4. **Online:** https://www.plantuml.com/plantuml/uml/

### Diagram Overview

**Component Diagram (C4 Level 3):**
```
┌───────────────────────────────────────────────────────────┐
│                  Workflow Engine Container                 │
│                                                            │
│  RdfWorkflowLoader → WorkflowEngine → PatternExecutor     │
│         ↓                  ↓                  ↓            │
│  spec_rdf_store    StateManager      OTEL Integration     │
│  pattern_metadata  case_rdf_stores   Weaver Validation    │
│         ↓                  ↓                               │
│      Sled Database    (persistence)                       │
└───────────────────────────────────────────────────────────┘
```

**Sequence Flow:**
```
User → API → Engine → Loader → Oxigraph → SHACL → SPARQL
                ↓
        Pattern Execution → State Update → OTEL → Weaver
```

**Data Flow:**
```
TTL File → Parse → Validate → Extract → WorkflowSpec
                                           ↓
                                  spec_rdf_store + Sled
                                           ↓
Case Data → Execute → Pattern → Update → case_rdf_store
                                           ↓
                               Persist → Sled + OTEL
```

---

## Key Integration Points

### Files to Modify

| File | Changes |
|------|---------|
| `src/executor/workflow_registration.rs` | Add `register_workflow_from_rdf()` |
| `src/executor/case.rs` | Add `execute_case_with_telemetry()` |
| `src/parser/mod.rs` | Add `RdfWorkflowLoader` |
| `src/state/manager.rs` | Add `save_case_with_rdf()` |

### Files to Create

| File | Purpose |
|------|---------|
| `src/rdf/loader.rs` | RDF workflow loader |
| `src/rdf/extractor.rs` | SPARQL query extraction |
| `src/rdf/validator.rs` | SHACL validation |
| `tests/chicago_tdd_atm_workflow.rs` | ATM workflow tests |

### Dependencies

**Already present in Cargo.toml:**
```toml
oxigraph = "0.4"      # RDF store + SPARQL
serde_json = "1.0"    # JSON handling
tokio = { version = "1.35", features = ["full"] }
tracing = "0.1"       # OTEL instrumentation
```

**May need to add:**
```toml
shacl = "0.1"         # SHACL validation (if available)
```

---

## Monitoring & Observability

### Key Metrics

**Workflow Metrics:**
- `workflow_registrations_total` (counter)
- `workflow_registration_latency_ms` (histogram)

**Case Metrics:**
- `case_executions_total` (counter)
- `case_execution_latency_ms` (histogram)
- `case_state_transitions_total` (counter)

**Performance Metrics:**
- `hot_path_ticks` (histogram) - CRITICAL: Must be ≤8
- `chatman_violations_total` (counter) - CRITICAL: Must be 0

**Query Metrics:**
- `sparql_query_latency_ms` (histogram)
- `sparql_queries_total` (counter)

### Grafana Dashboards

**Dashboard: Workflow Engine Performance**

Panels:
1. Case Execution Rate (rate/5m)
2. Hot Path Performance P95 (histogram, threshold: 8 ticks)
3. Chatman Violations (counter, alert: >0)
4. SPARQL Query Latency P99 (histogram, threshold: 50ms)

### Alerts

**Critical Alerts:**
- Chatman Constant violation (`hot_path_ticks > 8`)
- Case execution failure rate high (`>5%`)
- SPARQL query timeout (`>50ms`)

**Warning Alerts:**
- Workflow registration slow (`>500ms`)
- Case creation slow (`>100ms`)

---

## Security Considerations

### RDF Injection Prevention

**Problem:** Malicious SPARQL queries can access unauthorized data.

**Solution:**
- Sanitize user input
- Only allow SELECT and ASK queries
- Block UPDATE, INSERT, DELETE, DROP
- Use parameterized queries

```rust
pub fn sanitize_sparql(query: &str) -> Result<String, SecurityError> {
    let dangerous = ["DROP", "DELETE", "INSERT", "UPDATE"];
    for keyword in &dangerous {
        if query.to_uppercase().contains(keyword) {
            return Err(SecurityError::InjectionAttempt);
        }
    }
    Ok(query.to_string())
}
```

### Access Control

**Role-Based Access:**
- `register_roles`: Can register workflows
- `create_roles`: Can create cases
- `execute_roles`: Can execute cases
- `query_roles`: Can query RDF stores

### Encryption

**At Rest:**
- Sled database encrypted with KMS keys
- Case RDF snapshots encrypted

**In Transit:**
- TLS for all API endpoints
- mTLS with SPIFFE for service-to-service

---

## References

### Internal Documentation

- [RDF Workflow Execution Architecture](./rdf-workflow-execution.md)
- [RDF Workflow Implementation Guide](./rdf-workflow-implementation-guide.md)
- [ADR-002: Turtle vs YAWL XML](./ADR/ADR-002-turtle-vs-yawl-xml.md)

### External Standards

- [RDF 1.1 Turtle](https://www.w3.org/TR/turtle/)
- [SPARQL 1.1 Query Language](https://www.w3.org/TR/sparql11-query/)
- [SHACL Validation](https://www.w3.org/TR/shacl/)
- [OpenTelemetry Weaver](https://github.com/open-telemetry/weaver)
- [Oxigraph Documentation](https://github.com/oxigraph/oxigraph)
- [Van der Aalst Workflow Patterns](http://www.workflowpatterns.com/)

---

## Glossary

| Term | Definition |
|------|------------|
| **Chatman Constant** | Performance constraint: ≤8 ticks for hot path operations |
| **Chicago TDD** | Test-driven development with real collaborators (no mocks) |
| **Oxigraph** | Rust RDF database with SPARQL support |
| **SHACL** | Shapes Constraint Language for RDF validation |
| **SPARQL** | Query language for RDF graphs |
| **Turtle** | Compact RDF serialization format |
| **Weaver** | OpenTelemetry schema validation tool (source of truth for KNHK) |
| **WorkflowSpec** | Internal representation of workflow graph |
| **YAWL** | Yet Another Workflow Language |

---

## Contact & Support

**Architecture Questions:**
- System Architect (this document author)
- Tech Lead

**Implementation Questions:**
- See: [rdf-workflow-implementation-guide.md](./rdf-workflow-implementation-guide.md)
- Code examples in `tests/chicago_tdd_atm_*.rs`

**Testing Questions:**
- QA Team
- Performance Engineering

**Weaver Validation:**
- OpenTelemetry Team
- Observability Engineering

---

**Last Updated:** 2025-11-08
**Version:** 1.0
**Status:** Approved for Implementation

---

**END OF README**
