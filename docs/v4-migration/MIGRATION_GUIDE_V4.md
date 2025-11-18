# KNHK v4.0 Migration Guide: TTL-Only Architecture

**Version**: 4.0.0
**Release Date**: 2028-01-15
**Migration Difficulty**: Low (codebase already TTL-based)
**DOCTRINE Alignment**: Covenant 1 (Turtle is source of truth)

---

## Executive Summary

KNHK v4.0 declares **TTL/Turtle as the exclusive workflow format**, formalizing the architecture that has been in place since v1.0. This is not a technical migration but an **architectural declaration** that eliminates ambiguity and potential future XML support requests.

### Key Points

✅ **Codebase already TTL-only** - No code changes needed for existing users
✅ **Legacy XML crate available** - For users migrating from external systems
✅ **Migration tooling provided** - Automated XML→TTL conversion
✅ **DOCTRINE alignment** - Covenant 1: TTL is canonical source of truth

---

## Table of Contents

1. [Who Needs to Migrate?](#who-needs-to-migrate)
2. [Migration Scenarios](#migration-scenarios)
3. [XML to TTL Conversion](#xml-to-ttl-conversion)
4. [Validation & Testing](#validation--testing)
5. [API Changes](#api-changes)
6. [Breaking Changes](#breaking-changes)
7. [Rollback Plan](#rollback-plan)
8. [Support & Resources](#support--resources)

---

## Who Needs to Migrate?

### ✅ No Migration Needed

You **DO NOT** need to migrate if:

- You're already using TTL/Turtle workflows with KNHK
- You're using KNHK v1.0-v3.x with TTL workflows
- You're creating new workflows from scratch

**Action**: Continue using KNHK as normal. v4.0 is a non-breaking upgrade.

### ⚠️ Migration Required

You **DO** need to migrate if:

- You have XML YAWL workflows from external systems (not KNHK)
- You're importing workflows from legacy YAWL Editor
- You have XML workflow definitions you want to use with KNHK

**Action**: Follow the [XML to TTL Conversion](#xml-to-ttl-conversion) guide below.

---

## Migration Scenarios

### Scenario 1: Existing KNHK User (TTL workflows)

**Status**: ✅ No action required

**Steps**:
```bash
# Verify your workflows are TTL
ls workflows/*.ttl

# Upgrade to v4.0
cargo update -p knhk-workflow-engine

# Run existing tests
cargo test --workspace
```

**Expected outcome**: Everything continues working as before.

---

### Scenario 2: New KNHK User (no existing workflows)

**Status**: ✅ No action required

**Steps**:
```bash
# Install KNHK v4.0
cargo install knhk-workflow-engine

# Create workflows in TTL format
cat > workflow.ttl <<'EOF'
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://example.org/my-workflow> a yawl:Specification ;
    rdfs:label "My Workflow" ;
    yawl:hasTask <http://example.org/my-workflow#task1> .
EOF

# Validate and execute
knhk validate workflow.ttl
knhk execute workflow.ttl
```

**Expected outcome**: Clean installation with TTL workflows.

---

### Scenario 3: Migrating from External XML YAWL System

**Status**: ⚠️ Migration required

**Steps**:

#### Step 3.1: Install Migration Tool

```bash
cargo install knhk-workflow-xml-legacy
```

#### Step 3.2: Convert XML to TTL

```bash
# Single file conversion
yawl-xml-to-ttl workflow.yawl > workflow.ttl

# Or with output file
yawl-xml-to-ttl workflow.yawl -o workflow.ttl --validate

# Batch conversion (directory)
yawl-xml-to-ttl dir --dir ./xml-workflows/ --output ./ttl-workflows/

# Recursive conversion
yawl-xml-to-ttl dir --dir ./xml-workflows/ --output ./ttl-workflows/ --recursive
```

#### Step 3.3: Validate Converted Workflows

```bash
# Using KNHK validator
knhk validate workflow.ttl

# Using Weaver (recommended - DOCTRINE-compliant)
weaver registry check -r workflow.ttl

# Run workflow to verify behavior
knhk execute workflow.ttl
```

#### Step 3.4: Review and Adjust

Some XML features may require manual review:

```bash
# Check conversion report
yawl-xml-to-ttl workflow.yawl -o workflow.ttl --validate 2>&1 | tee conversion-report.txt

# Review warnings
grep "WARNING" conversion-report.txt
```

**Common adjustments needed**:

| XML Feature | TTL Equivalent | Action Required |
|-------------|----------------|-----------------|
| Custom namespaces | `@prefix custom:` | Declare in TTL |
| Vendor extensions | Custom predicates | Map to YAWL ontology |
| External schemas | `owl:imports` | Add RDF imports |

**Expected outcome**: Validated TTL workflows ready for KNHK v4.0.

---

## XML to TTL Conversion

### Conversion Tool Architecture

```
┌─────────────┐
│  XML YAWL   │
│  Workflow   │
└──────┬──────┘
       │
       ▼
┌──────────────────┐
│  XML Parser      │  ← roxmltree + quick-xml
│  (Legacy Crate)  │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  Intermediate    │  ← YawlWorkflow struct
│  Representation  │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  TTL Serializer  │  ← rio_turtle + oxigraph
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  TTL/Turtle      │  ← YAWL ontology RDF
│  Workflow        │
└──────────────────┘
```

### Example Conversion

**Input XML** (`workflow.yawl`):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<specification uri="http://example.org/order-processing">
  <name>Order Processing Workflow</name>

  <task id="receive_order">
    <name>Receive Order</name>
    <split code="AND"/>
  </task>

  <task id="validate_order">
    <name>Validate Order</name>
    <join code="AND"/>
  </task>

  <condition id="c_start" isStartCondition="true">
    <name>Start</name>
  </condition>

  <condition id="c_end" isEndCondition="true">
    <name>End</name>
  </condition>

  <flow source="c_start" target="receive_order"/>
  <flow source="receive_order" target="validate_order"/>
  <flow source="validate_order" target="c_end"/>
</specification>
```

**Output TTL** (`workflow.ttl`):

```turtle
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

<http://example.org/order-processing> a yawl:Specification ;
    rdfs:label "Order Processing Workflow" ;
    yawl:hasTask <http://example.org/order-processing#receive_order> ;
    yawl:hasTask <http://example.org/order-processing#validate_order> ;
    yawl:hasStartCondition <http://example.org/order-processing#c_start> ;
    yawl:hasEndCondition <http://example.org/order-processing#c_end> .

<http://example.org/order-processing#receive_order> a yawl:Task ;
    rdfs:label "Receive Order" ;
    yawl:splitType yawl:AND .

<http://example.org/order-processing#validate_order> a yawl:Task ;
    rdfs:label "Validate Order" ;
    yawl:joinType yawl:AND .

<http://example.org/order-processing#c_start> a yawl:Condition ;
    rdfs:label "Start" ;
    yawl:isStartCondition true .

<http://example.org/order-processing#c_end> a yawl:Condition ;
    rdfs:label "End" ;
    yawl:isEndCondition true .

<http://example.org/order-processing#flow_1> a yawl:Flow ;
    yawl:source <http://example.org/order-processing#c_start> ;
    yawl:target <http://example.org/order-processing#receive_order> .

<http://example.org/order-processing#flow_2> a yawl:Flow ;
    yawl:source <http://example.org/order-processing#receive_order> ;
    yawl:target <http://example.org/order-processing#validate_order> .

<http://example.org/order-processing#flow_3> a yawl:Flow ;
    yawl:source <http://example.org/order-processing#validate_order> ;
    yawl:target <http://example.org/order-processing#c_end> .
```

---

## Validation & Testing

### Step 1: Syntax Validation (TTL Parser)

```bash
# Using migration tool
yawl-xml-to-ttl validate workflow.ttl

# Using rio_turtle directly
rio_turtle workflow.ttl
```

### Step 2: Semantic Validation (YAWL Ontology)

```bash
# Using KNHK validator
knhk validate workflow.ttl

# Check for required elements
knhk validate --strict workflow.ttl
```

### Step 3: Weaver Schema Validation (DOCTRINE-Compliant)

```bash
# Validate against OTEL schema
weaver registry check -r /home/user/knhk/registry/

# Live validation (runtime telemetry)
weaver registry live-check --registry /home/user/knhk/registry/
```

### Step 4: Integration Testing

```bash
# Execute workflow in test mode
knhk execute workflow.ttl --dry-run

# Run with sample inputs
knhk execute workflow.ttl --input test-data.json

# Check telemetry output
knhk execute workflow.ttl --telemetry-export stdout
```

### Step 5: Performance Validation

```bash
# Verify Chicago TDD compliance (≤8 ticks hot path)
make test-performance-v04

# Run benchmarks
cargo bench --package knhk-workflow-engine
```

---

## API Changes

### No Breaking Changes for TTL Users

If you're already using TTL workflows, the API remains **100% compatible**:

```rust
// v3.x code (still works in v4.0)
use knhk_workflow_engine::{WorkflowParser, WorkflowEngine};

let mut parser = WorkflowParser::new()?;
let spec = parser.parse_file("workflow.ttl")?;
let engine = WorkflowEngine::new(state_store);
engine.register_workflow(spec).await?;
```

### Feature Flag Changes

#### v3.x Features:
```toml
[features]
default = ["rdf", "storage", "testing", "connectors", "http"]
rdf = ["oxigraph"]
```

#### v4.0 Features:
```toml
[features]
default = ["ttl-only", "storage", "testing", "connectors", "http"]
ttl-only = []  # New: Enforces TTL-only validation
xml-legacy = ["knhk-workflow-xml-legacy"]  # Optional: For migration only
```

### Using Legacy XML Support (Not Recommended)

```toml
# In your Cargo.toml (migration only)
[dependencies]
knhk-workflow-engine = { version = "4.0", features = ["xml-legacy"] }
```

**⚠️ Warning**: `xml-legacy` feature is deprecated and will be removed in v5.0.

---

## Breaking Changes

### Summary: No Breaking Changes for TTL Users

**If you use TTL workflows, v4.0 is a drop-in replacement for v3.x.**

### Breaking Changes for Hypothetical XML Users

Since KNHK never supported XML in core (only TTL), these are hypothetical:

| Change | Impact | Mitigation |
|--------|--------|------------|
| XML parsing removed | N/A (never existed) | Use `xml-legacy` crate |
| TTL-only validation | Enforces best practice | Already compliant |
| Feature flag rename | Clarifies intent | Auto-migrated |

---

## Rollback Plan

### If Migration Issues Occur

```bash
# Rollback to v3.x
cargo update -p knhk-workflow-engine --precise 3.9.0

# Keep both versions during transition
[dependencies]
knhk-workflow-engine-v3 = { package = "knhk-workflow-engine", version = "3.9" }
knhk-workflow-engine-v4 = { package = "knhk-workflow-engine", version = "4.0" }
```

### Parallel Running

Run v3.x and v4.0 side-by-side during migration:

```rust
// Test with v4.0
let result_v4 = test_workflow_v4(workflow_ttl).await;

// Verify with v3.x
let result_v3 = test_workflow_v3(workflow_ttl).await;

// Compare results
assert_eq!(result_v4, result_v3);
```

---

## Support & Resources

### Documentation

- **DOCTRINE Reference**: `/home/user/knhk/DOCTRINE_2027.md`
- **Covenant 1**: `/home/user/knhk/DOCTRINE_COVENANT.md` (Turtle is source of truth)
- **Breaking Changes**: `/home/user/knhk/docs/v4-migration/V4_BREAKING_CHANGES.md`
- **YAWL Ontology**: `/home/user/knhk/ontology/yawl-v2.ttl`

### Tools

- **Migration Tool**: `cargo install knhk-workflow-xml-legacy`
- **KNHK CLI**: `cargo install knhk-cli`
- **Weaver**: `cargo install weaver-forge-cli`

### Getting Help

1. **Check examples**: `/home/user/knhk/examples/workflows/*.ttl`
2. **Run tests**: `cargo test --package knhk-workflow-engine`
3. **Open issue**: https://github.com/knhk/knhk/issues
4. **Read DOCTRINE**: Understanding Covenant 1 explains the "why"

### Migration Checklist

- [ ] Identify workflow format (XML or TTL)
- [ ] If XML: Install migration tool
- [ ] Convert XML → TTL
- [ ] Validate TTL syntax
- [ ] Validate YAWL semantics
- [ ] Run Weaver schema validation
- [ ] Test workflow execution
- [ ] Verify performance (≤8 ticks)
- [ ] Update CI/CD pipelines
- [ ] Document workflow structure
- [ ] Train team on TTL format

---

## DOCTRINE Alignment

### Why TTL-Only? (Covenant 1)

From **DOCTRINE_COVENANT.md**, Covenant 1 states:

> **Turtle is the sole source of truth for all workflow definitions.**
>
> No XML, no JSON (except JSON-LD RDF), no proprietary formats.

**Rationale**:

1. **Σ (Ontology-First)**: RDF/TTL is the canonical semantic representation
2. **Weaver Validation**: OTEL schema validation requires RDF structure
3. **Semantic Completeness**: TTL enables SPARQL queries and reasoning
4. **No Impedance Mismatch**: Direct RDF → execution path
5. **Industry Standard**: W3C recommendation, not proprietary

### v4.0 Enforces Covenant 1

v4.0 **formalizes** what was always true: KNHK is TTL-first, TTL-only.

This isn't a migration; it's an **architectural declaration** that eliminates ambiguity and prevents future drift toward XML or other formats.

---

## Frequently Asked Questions (FAQ)

### Q: Do I need to migrate if I'm already using TTL?

**A**: No. v4.0 is a drop-in replacement for v3.x when using TTL workflows.

### Q: What if I have XML YAWL workflows from another system?

**A**: Use the `knhk-workflow-xml-legacy` migration tool to convert XML → TTL.

### Q: Will XML support be added back in the future?

**A**: No. v4.0 declares TTL-only as permanent architecture per DOCTRINE Covenant 1.

### Q: Can I use JSON for workflows?

**A**: Yes, but only JSON-LD (RDF serialization). Plain JSON is not supported.

### Q: What about BPMN XML?

**A**: BPMN XML is not supported. Convert BPMN → YAWL TTL using external tools.

### Q: How do I create TTL workflows from scratch?

**A**: See `/home/user/knhk/examples/workflows/` for TTL examples and templates.

### Q: What's the performance impact of v4.0?

**A**: Zero. TTL parsing performance is unchanged. v4.0 is a clarification, not a rewrite.

### Q: Can I still use Weaver validation in v4.0?

**A**: Yes. v4.0 enhances Weaver integration (TTL → RDF → OTEL schema).

### Q: What happens to the `xml-legacy` crate in v5.0?

**A**: It will be archived and no longer maintained. Migrate to TTL before v5.0.

---

## Migration Timeline

```
2027 Q4: v3.9 (final v3.x release, TTL-only in practice)
2028 Q1: v4.0 (TTL-only declaration, xml-legacy crate published)
2028 Q2-Q4: Migration period (xml-legacy supported)
2029 Q1: v5.0 (xml-legacy archived, TTL-only permanent)
```

**Recommendation**: Migrate to TTL before 2028 Q4 to avoid rushing before v5.0.

---

## Summary

v4.0 is a **non-breaking architectural declaration**: KNHK is TTL-only.

- ✅ **No migration for TTL users** (99% of users)
- ✅ **Migration tooling provided** for XML users
- ✅ **DOCTRINE-aligned** (Covenant 1)
- ✅ **Future-proof** (no format ambiguity)

**Next steps**: Validate your workflows are TTL, upgrade to v4.0, continue building.

---

**Document Version**: 1.0
**Last Updated**: 2028-01-15
**Author**: KNHK Team
**DOCTRINE Compliance**: Covenant 1 (Σ - Ontology-First)
