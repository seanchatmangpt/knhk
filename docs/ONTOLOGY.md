# Ontology Integration - 80/20 Guide

**Version**: 1.0  
**Status**: Production-Ready  
**Last Updated**: 2025-01-XX

---

## Overview

KNHK integrates the YAWL ontology (RDF/OWL) with knhk-workflow-engine, enabling semantic workflow definitions, SPARQL-based validation, and ontology-driven code generation.

**Key Features**:
- ✅ Semantic workflow definitions (YAWL workflows as RDF/OWL)
- ✅ SPARQL-based validation (35+ semantic validation rules)
- ✅ Ontology-driven code generation (Rust types from OWL classes)
- ✅ Schema-first validation (Weaver OTEL validation)
- ✅ knhk extensions (performance, provenance, observability)

---

## Quick Start (80% Use Case)

### Integration Architecture

```
YAWL Ontology (yawl.ttl)
    ↓
Oxigraph RDF Triplestore
    ↓
WorkflowParser (SPARQL Extraction)
    ↓
WorkflowSpec (Rust Types)
    ↓
SPARQL Validator + Weaver Validator
    ↓
WorkflowEngine (Execution)
    ↓
RDF State Store + Lockchain Provenance
```

### Basic Usage

```rust
use knhk_workflow_engine::{WorkflowParser, WorkflowEngine};

// Parse workflow from Turtle
let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;

// Register workflow (validates with SPARQL)
let engine = WorkflowEngine::new(state_store);
engine.register_workflow(spec).await?;
```

---

## Core Integration (80% Value)

### 1. Ontology Analysis

**YAWL Ontology Classes**:
- `yawl:WorkflowSpec` - Workflow specification
- `yawl:Net` - Workflow net
- `yawl:Task` - Workflow task
- `yawl:Condition` - Workflow condition
- `yawl:Flow` - Workflow flow

**Key Properties**:
- `yawl:name` - Name of workflow element
- `yawl:splitType` - Split type (AND, XOR, OR)
- `yawl:joinType` - Join type (AND, XOR, OR)
- `yawl:inputCondition` - Input condition
- `yawl:outputCondition` - Output condition

### 2. Type Mapping Strategy

**OWL → Rust Type Mapping**:
- `yawl:WorkflowSpec` → `WorkflowSpec`
- `yawl:Task` → `Task`
- `yawl:Condition` → `Condition`
- `yawl:Flow` → `Flow`

**Mapping Implementation**:
- SPARQL queries extract OWL classes
- Rust types generated from OWL properties
- Type-safe workflow execution

### 3. SPARQL Query Patterns

**Common Queries**:
- Extract workflow tasks: `SELECT ?task WHERE { ?task rdf:type yawl:Task }`
- Extract workflow conditions: `SELECT ?condition WHERE { ?condition rdf:type yawl:Condition }`
- Extract workflow flows: `SELECT ?flow WHERE { ?flow rdf:type yawl:Flow }`

**Query Examples**:
```sparql
# Extract all tasks
SELECT ?task ?name WHERE {
    ?task rdf:type yawl:Task .
    ?task yawl:name ?name .
}

# Extract task flows
SELECT ?task ?condition WHERE {
    ?task yawl:outputCondition ?condition .
}
```

### 4. Validation Framework

**SPARQL Validation Rules**:
- Soundness validation (35+ rules)
- Data flow validation
- Deadlock detection
- Pattern validation

**Weaver Validation**:
- Schema-first validation
- OTEL telemetry validation
- Runtime validation

---

## knhk Extensions

### Performance Extensions

**Hot Path Optimization**:
- ≤8 ticks for workflow operations
- SIMD-optimized pattern execution
- Zero-copy operations

### Provenance Extensions

**Lockchain Integration**:
- Immutable audit trail
- Cryptographic verification
- Git-based provenance

### Observability Extensions

**OTEL Integration**:
- Spans for workflow execution
- Metrics for workflow performance
- Logs for workflow events

---

## Integration Points

### WorkflowParser Integration

**SPARQL Extraction**:
- Extract tasks, conditions, flows from RDF
- Map OWL classes to Rust types
- Validate workflow structure

**File**: `rust/knhk-workflow-engine/src/parser/extractor.rs`

### WorkflowEngine Integration

**RDF State Store**:
- Store workflow specifications as RDF
- Store case state as RDF
- Query workflow state with SPARQL

**File**: `rust/knhk-workflow-engine/src/state/manager.rs`

### Validation Integration

**SPARQL Validator**:
- Validate workflow soundness
- Check data flow correctness
- Detect deadlocks

**File**: `rust/knhk-workflow-engine/src/validation/`

---

## Production Readiness

### ✅ Ready for Production

- **Ontology Parsing**: Complete (Turtle/RDF)
- **Type Mapping**: Complete (OWL → Rust)
- **SPARQL Queries**: Complete (workflow extraction)
- **Validation**: Complete (35+ rules)
- **State Persistence**: Complete (RDF store)

### ⚠️ Partial Production Readiness

- **Advanced SPARQL Queries**: Some complex queries not optimized
- **OWL Reasoning**: Basic reasoning only (no full OWL inference)

---

## Troubleshooting

### SPARQL Query Fails

**Problem**: SPARQL query returns no results.

**Solution**:
- Verify RDF parsing (Turtle format)
- Check ontology namespace (yawl: prefix)
- Verify query syntax (SPARQL 1.1)

### Type Mapping Fails

**Problem**: OWL classes not mapped to Rust types.

**Solution**:
- Check OWL class definitions
- Verify SPARQL extraction queries
- Check Rust type definitions

### Validation Fails

**Problem**: Workflow validation fails.

**Solution**:
- Check SPARQL validation rules
- Verify Weaver schema validation
- Review workflow structure

---

## Additional Resources

### Detailed Documentation
- **Master Guide**: [Ontology Integration Master Guide](ontology-integration/ONTOLOGY_INTEGRATION_MASTER_GUIDE.md)
- **Architecture**: [YAWL Ontology Architecture](ontology-integration/yawl-ontology-architecture.md)
- **Implementation**: [Ontology Implementation Checklist](ontology-integration/implementation-checklist.md)
- **SPARQL**: [SPARQL Query Patterns](ontology-integration/sparql-query-patterns.md)
- **Validation**: [Semantic Validation Rules](ontology-integration/semantic-validation-rules.md)

### Code Examples
- **Parser**: `rust/knhk-workflow-engine/src/parser/`
- **Validator**: `rust/knhk-workflow-engine/src/validation/`
- **Workflows**: `ontology/workflows/`

### Archived Documentation
- **Detailed Implementation**: `docs/archived/ontology-integration/` (detailed implementation guides)

---

## License

MIT License

---

**Last Updated**: 2025-01-XX  
**Version**: 1.0  
**Status**: Production-Ready

