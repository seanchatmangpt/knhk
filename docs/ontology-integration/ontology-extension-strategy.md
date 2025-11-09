# YAWL Ontology Extension Strategy for knhk

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Work In Progress

## Executive Summary

This document defines how to extend the YAWL ontology with knhk-specific features while maintaining compatibility with standard YAWL tools. Extensions are defined in a separate namespace (`knhk:`) and use subclassing to preserve YAWL semantics.

**Key Extensions:**
1. **Hot Path Annotations** - Performance constraints (≤8 ticks)
2. **Lockchain Provenance** - Git commit provenance tracking
3. **OTEL Instrumentation** - OpenTelemetry span templates
4. **Security Policies** - Fine-grained access control
5. **Performance Metrics** - Runtime performance tracking

## 1. Extension Principles

### 1.1 Design Principles

1. **Separate Namespace:** All extensions use `knhk:` namespace
2. **Subclassing:** Extend YAWL classes, don't modify
3. **Backward Compatibility:** Standard YAWL tools ignore knhk extensions
4. **Forward Compatibility:** knhk tools understand standard YAWL
5. **Schema Validation:** Extensions validated via Weaver (OTEL)

### 1.2 Namespace Declaration

```turtle
@prefix knhk: <http://knhk.org/ontology#> .
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix otel: <http://opentelemetry.io/schema/1.0#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

knhk: a owl:Ontology ;
    owl:versionInfo "1.0.0" ;
    rdfs:label "KNHK Workflow Engine Extensions" ;
    rdfs:comment "Extensions to YAWL ontology for knhk-specific features" ;
    owl:imports <http://www.yawlfoundation.org/yawlschema#> .
```

## 2. Performance Extensions

### 2.1 Hot Path Task

**Motivation:** Mark tasks that must complete in ≤8 ticks (Chatman Constant)

**Class Definition:**
```turtle
knhk:HotPathTask a owl:Class ;
    rdfs:subClassOf yawl:Task ;
    rdfs:label "Hot Path Task" ;
    rdfs:comment "Task guaranteed to execute in ≤8 ticks (Chatman Constant)" ;
    rdfs:seeAlso <https://github.com/chatman/chatman-constant> .
```

**Properties:**
```turtle
knhk:tickBudget a owl:DatatypeProperty ;
    rdfs:domain knhk:HotPathTask ;
    rdfs:range xsd:positiveInteger ;
    rdfs:label "tick budget" ;
    rdfs:comment "Maximum ticks allowed for task execution (default: 8)" ;
    owl:maxCardinality 1 .

knhk:useSimd a owl:DatatypeProperty ;
    rdfs:domain knhk:HotPathTask ;
    rdfs:range xsd:boolean ;
    rdfs:label "use SIMD" ;
    rdfs:comment "Whether to use SIMD acceleration for this task" ;
    owl:maxCardinality 1 .

knhk:priority a owl:DatatypeProperty ;
    rdfs:domain knhk:HotPathTask ;
    rdfs:range xsd:nonNegativeInteger ;
    rdfs:label "priority" ;
    rdfs:comment "Task priority (0-255, higher = more important)" ;
    owl:maxCardinality 1 .
```

**Usage Example:**
```turtle
ex:CriticalTask a yawl:Task, knhk:HotPathTask ;
    rdfs:label "Process Payment" ;
    yawl:hasJoin yawl:ControlTypeXor ;
    yawl:hasSplit yawl:ControlTypeAnd ;
    knhk:tickBudget 8 ;
    knhk:useSimd true ;
    knhk:priority 255 .
```

**Rust Extraction:**
```rust
// Extracted into Task struct
pub struct Task {
    // Standard YAWL fields
    pub id: String,
    pub name: String,
    // ...

    // knhk hot path extensions
    pub max_ticks: Option<u32>,    // From knhk:tickBudget (default: 8)
    pub use_simd: bool,             // From knhk:useSimd (default: false)
    pub priority: Option<u32>,      // From knhk:priority (default: none)
}
```

**SPARQL Query:**
```sparql
PREFIX knhk: <http://knhk.org/ontology#>
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?name ?tickBudget ?simd ?priority WHERE {
    ?task a knhk:HotPathTask .
    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?task knhk:tickBudget ?tickBudget }
    OPTIONAL { ?task knhk:useSimd ?simd }
    OPTIONAL { ?task knhk:priority ?priority }
}
```

---

### 2.2 Performance Constraints

**Motivation:** Define performance SLAs beyond tick budgets

**Properties:**
```turtle
knhk:maxLatency a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:duration ;
    rdfs:label "max latency" ;
    rdfs:comment "Maximum end-to-end latency (e.g., PT5S for 5 seconds)" .

knhk:maxThroughput a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:decimal ;
    rdfs:label "max throughput" ;
    rdfs:comment "Maximum instances per second" .

knhk:maxMemory a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:long ;
    rdfs:label "max memory" ;
    rdfs:comment "Maximum memory usage in bytes" .
```

**Usage Example:**
```turtle
ex:DataProcessingTask a yawl:Task ;
    rdfs:label "Process Large Dataset" ;
    knhk:maxLatency "PT30S"^^xsd:duration ;  # 30 seconds
    knhk:maxMemory 536870912 ;                # 512 MB
    knhk:maxThroughput 100.0 .                # 100 instances/sec
```

---

## 3. Provenance Extensions

### 3.1 Lockchain Provenance

**Motivation:** Track Git commit provenance for workflow definitions and instances

**Properties:**
```turtle
knhk:hasProvenanceChain a owl:ObjectProperty ;
    rdfs:domain yawl:WorkflowInstance ;
    rdfs:range knhk:LockchainReference ;
    rdfs:label "has provenance chain" ;
    rdfs:comment "Git commit hash in lockchain" .

knhk:LockchainReference a owl:Class ;
    rdfs:label "Lockchain Reference" ;
    rdfs:comment "Reference to Git commit in lockchain" .

knhk:commitHash a owl:DatatypeProperty ;
    rdfs:domain knhk:LockchainReference ;
    rdfs:range xsd:string ;
    rdfs:label "commit hash" ;
    rdfs:comment "SHA-256 commit hash" .

knhk:repositoryUri a owl:DatatypeProperty ;
    rdfs:domain knhk:LockchainReference ;
    rdfs:range xsd:anyURI ;
    rdfs:label "repository URI" ;
    rdfs:comment "Git repository URI" .

knhk:branchName a owl:DatatypeProperty ;
    rdfs:domain knhk:LockchainReference ;
    rdfs:range xsd:string ;
    rdfs:label "branch name" ;
    rdfs:comment "Git branch name" .
```

**Usage Example:**
```turtle
ex:WorkflowInstance123 a knhk:WorkflowInstance ;
    knhk:hasSpecification ex:MyWorkflow ;
    knhk:hasState "running" ;
    knhk:hasProvenanceChain ex:Provenance456 .

ex:Provenance456 a knhk:LockchainReference ;
    knhk:commitHash "abc123def456789..." ;
    knhk:repositoryUri <https://github.com/example/workflows.git> ;
    knhk:branchName "main" .
```

**Integration with W3C PROV:**
```turtle
@prefix prov: <http://www.w3.org/ns/prov#> .

ex:WorkflowInstance123 a prov:Entity, knhk:WorkflowInstance ;
    prov:wasGeneratedBy ex:Execution789 ;
    prov:wasDerivedFrom ex:MyWorkflow ;
    knhk:hasProvenanceChain ex:Provenance456 .

ex:Execution789 a prov:Activity ;
    prov:used ex:MyWorkflow ;
    prov:wasAssociatedWith ex:WorkflowEngine .

ex:Provenance456 a prov:Entity, knhk:LockchainReference ;
    prov:wasAttributedTo ex:Developer ;
    prov:generatedAtTime "2025-11-08T10:00:00Z"^^xsd:dateTime .
```

---

## 4. Observability Extensions

### 4.1 OpenTelemetry Instrumentation

**Motivation:** Define OTEL span templates for automatic instrumentation

**Properties:**
```turtle
knhk:hasSpanTemplate a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:string ;
    rdfs:label "has span template" ;
    rdfs:comment "OpenTelemetry span name template" .

knhk:hasMetricTemplate a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:string ;
    rdfs:label "has metric template" ;
    rdfs:comment "OpenTelemetry metric name template" .

knhk:traceIdRequired a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:boolean ;
    rdfs:label "trace ID required" ;
    rdfs:comment "Whether trace ID propagation is required" .
```

**Usage Example:**
```turtle
ex:TaskA a yawl:Task ;
    rdfs:label "Approve Order" ;
    knhk:hasSpanTemplate "workflow.task.execute" ;
    knhk:hasMetricTemplate "workflow.task.duration" ;
    knhk:traceIdRequired true .
```

**Code Generation:**
```rust
// Generated from ontology
#[tracing::instrument(
    name = "workflow.task.execute",
    fields(
        task.id = %task_id,
        task.name = %task_name,
        workflow.id = %workflow_id
    )
)]
async fn execute_task(task_id: &str, task_name: &str, workflow_id: &str) {
    // Automatically emits span: workflow.task.execute
    // Automatically emits metric: workflow.task.duration
    // ...
}
```

**Weaver Validation:**
```yaml
# registry/workflow/task.yaml
groups:
  - id: workflow.task
    type: span
    brief: Workflow task execution
    attributes:
      - ref: task.id
      - ref: task.name
      - ref: workflow.id
```

---

### 4.2 Custom Attributes

**Motivation:** Attach custom OTEL attributes to tasks

**Property:**
```turtle
knhk:hasOtelAttribute a owl:ObjectProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range knhk:OtelAttribute .

knhk:OtelAttribute a owl:Class ;
    rdfs:label "OTEL Attribute" ;
    rdfs:comment "OpenTelemetry attribute key-value pair" .

knhk:attributeKey a owl:DatatypeProperty ;
    rdfs:domain knhk:OtelAttribute ;
    rdfs:range xsd:string .

knhk:attributeValue a owl:DatatypeProperty ;
    rdfs:domain knhk:OtelAttribute ;
    rdfs:range xsd:string .
```

**Usage Example:**
```turtle
ex:TaskA a yawl:Task ;
    rdfs:label "Process Payment" ;
    knhk:hasOtelAttribute ex:Attr1, ex:Attr2 .

ex:Attr1 a knhk:OtelAttribute ;
    knhk:attributeKey "payment.processor" ;
    knhk:attributeValue "stripe" .

ex:Attr2 a knhk:OtelAttribute ;
    knhk:attributeKey "payment.currency" ;
    knhk:attributeValue "USD" .
```

---

## 5. Security Extensions

### 5.1 Access Control

**Motivation:** Define fine-grained access control policies

**Properties:**
```turtle
knhk:requiresCapability a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:string ;
    rdfs:label "requires capability" ;
    rdfs:comment "Required capability for task execution" .

knhk:requiresRole a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:string ;
    rdfs:label "requires role" ;
    rdfs:comment "Required role for task execution" .

knhk:requiresPermission a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:string ;
    rdfs:label "requires permission" ;
    rdfs:comment "Required permission (e.g., 'workflow:execute')" .
```

**Usage Example:**
```turtle
ex:ApproveOrderTask a yawl:Task ;
    rdfs:label "Approve High-Value Order" ;
    knhk:requiresCapability "approve_orders" ;
    knhk:requiresRole "manager" ;
    knhk:requiresPermission "orders:approve:high_value" .
```

**Rust Integration:**
```rust
// Validate before task execution
fn can_execute_task(user: &User, task: &Task) -> bool {
    if let Some(required_cap) = &task.required_capabilities {
        if !user.has_capability(required_cap) {
            return false;
        }
    }
    if let Some(required_role) = &task.required_roles {
        if !user.has_role(required_role) {
            return false;
        }
    }
    true
}
```

---

### 5.2 Data Encryption

**Motivation:** Mark sensitive data for encryption

**Properties:**
```turtle
knhk:encryptionRequired a owl:DatatypeProperty ;
    rdfs:domain yawl:Variable ;
    rdfs:range xsd:boolean ;
    rdfs:label "encryption required" ;
    rdfs:comment "Whether variable data must be encrypted at rest" .

knhk:encryptionAlgorithm a owl:DatatypeProperty ;
    rdfs:domain yawl:Variable ;
    rdfs:range xsd:string ;
    rdfs:label "encryption algorithm" ;
    rdfs:comment "Encryption algorithm (e.g., AES-256-GCM)" .

knhk:piiCategory a owl:DatatypeProperty ;
    rdfs:domain yawl:Variable ;
    rdfs:range xsd:string ;
    rdfs:label "PII category" ;
    rdfs:comment "PII category (e.g., 'email', 'ssn', 'credit_card')" .
```

**Usage Example:**
```turtle
ex:CustomerSSN a yawl:Variable ;
    rdfs:label "Customer SSN" ;
    yawl:type "string" ;
    knhk:encryptionRequired true ;
    knhk:encryptionAlgorithm "AES-256-GCM" ;
    knhk:piiCategory "ssn" .
```

---

## 6. Runtime State Extensions

### 6.1 Workflow Instance

**Motivation:** Track runtime state of workflow instances

**Class:**
```turtle
knhk:WorkflowInstance a owl:Class ;
    rdfs:label "Workflow Instance" ;
    rdfs:comment "Runtime instance of a workflow" .
```

**Properties:**
```turtle
knhk:hasSpecification a owl:ObjectProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range yawl:Specification ;
    rdfs:label "has specification" ;
    rdfs:comment "Workflow specification being executed" .

knhk:hasState a owl:DatatypeProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range xsd:string ;
    rdfs:label "has state" ;
    rdfs:comment "Current state (created, running, completed, failed, cancelled)" .

knhk:startedAt a owl:DatatypeProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range xsd:dateTime ;
    rdfs:label "started at" ;
    rdfs:comment "Workflow instance start timestamp" .

knhk:completedAt a owl:DatatypeProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range xsd:dateTime ;
    rdfs:label "completed at" ;
    rdfs:comment "Workflow instance completion timestamp" .

knhk:hasCurrentTask a owl:ObjectProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range yawl:Task ;
    rdfs:label "has current task" ;
    rdfs:comment "Currently executing task" .
```

**Usage Example:**
```turtle
ex:Instance123 a knhk:WorkflowInstance ;
    knhk:hasSpecification ex:MyWorkflow ;
    knhk:hasState "running" ;
    knhk:startedAt "2025-11-08T10:00:00Z"^^xsd:dateTime ;
    knhk:hasCurrentTask ex:TaskA ;
    knhk:hasProvenanceChain ex:Provenance456 .
```

---

## 7. Extension Versioning

### 7.1 Semantic Versioning

**Strategy:** Use namespace versioning for breaking changes

```turtle
# Version 1.0
@prefix knhk: <http://knhk.org/ontology/1.0#> .

# Version 2.0 (breaking changes)
@prefix knhk2: <http://knhk.org/ontology/2.0#> .

# Workflow can import specific version
ex:MyWorkflow a yawl:Specification ;
    owl:imports <http://knhk.org/ontology/1.0#> .
```

**Migration Strategy:**
1. **Backward Compatible:** Add new properties to existing namespace
2. **Breaking Changes:** Create new versioned namespace
3. **Deprecation:** Mark old properties with `owl:deprecated true`
4. **Migration Scripts:** SPARQL UPDATE to migrate workflows

**Example Migration (1.0 → 2.0):**
```sparql
PREFIX knhk1: <http://knhk.org/ontology/1.0#>
PREFIX knhk2: <http://knhk.org/ontology/2.0#>

DELETE {
    ?task knhk1:maxTicks ?ticks .
}
INSERT {
    ?task knhk2:tickBudget ?ticks .
}
WHERE {
    ?task knhk1:maxTicks ?ticks .
}
```

---

## 8. Validation Rules

### 8.1 SHACL Shapes for Extensions

**Hot Path Validation:**
```turtle
@prefix sh: <http://www.w3.org/ns/shacl#> .

knhk:HotPathTaskShape a sh:NodeShape ;
    sh:targetClass knhk:HotPathTask ;
    sh:property [
        sh:path knhk:tickBudget ;
        sh:datatype xsd:positiveInteger ;
        sh:minInclusive 1 ;
        sh:maxInclusive 8 ;
        sh:message "Hot path task must have tick budget between 1 and 8" ;
    ] .
```

**Provenance Validation:**
```turtle
knhk:WorkflowInstanceShape a sh:NodeShape ;
    sh:targetClass knhk:WorkflowInstance ;
    sh:property [
        sh:path knhk:hasProvenanceChain ;
        sh:class knhk:LockchainReference ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "Workflow instance must have exactly one provenance chain" ;
    ] .
```

---

## 9. Complete Extension File

**File:** `/Users/sac/knhk/ontology/knhk-extensions.ttl`

```turtle
@prefix knhk: <http://knhk.org/ontology#> .
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# ============================================================================
# Ontology Metadata
# ============================================================================

knhk: a owl:Ontology ;
    owl:versionInfo "1.0.0" ;
    rdfs:label "KNHK Workflow Engine Extensions" ;
    rdfs:comment "Extensions to YAWL ontology for knhk-specific features" ;
    owl:imports <http://www.yawlfoundation.org/yawlschema#> .

# ============================================================================
# Performance Extensions
# ============================================================================

knhk:HotPathTask a owl:Class ;
    rdfs:subClassOf yawl:Task ;
    rdfs:label "Hot Path Task" ;
    rdfs:comment "Task guaranteed to execute in ≤8 ticks" .

knhk:tickBudget a owl:DatatypeProperty ;
    rdfs:domain knhk:HotPathTask ;
    rdfs:range xsd:positiveInteger ;
    rdfs:label "tick budget" .

knhk:useSimd a owl:DatatypeProperty ;
    rdfs:domain knhk:HotPathTask ;
    rdfs:range xsd:boolean .

knhk:priority a owl:DatatypeProperty ;
    rdfs:domain knhk:HotPathTask ;
    rdfs:range xsd:nonNegativeInteger .

# ============================================================================
# Provenance Extensions
# ============================================================================

knhk:LockchainReference a owl:Class ;
    rdfs:label "Lockchain Reference" .

knhk:hasProvenanceChain a owl:ObjectProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range knhk:LockchainReference .

knhk:commitHash a owl:DatatypeProperty ;
    rdfs:domain knhk:LockchainReference ;
    rdfs:range xsd:string .

# ============================================================================
# Observability Extensions
# ============================================================================

knhk:hasSpanTemplate a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:string .

knhk:hasMetricTemplate a owl:DatatypeProperty ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:string .

# ============================================================================
# Runtime State
# ============================================================================

knhk:WorkflowInstance a owl:Class ;
    rdfs:label "Workflow Instance" .

knhk:hasSpecification a owl:ObjectProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range yawl:Specification .

knhk:hasState a owl:DatatypeProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range xsd:string .

knhk:startedAt a owl:DatatypeProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range xsd:dateTime .

knhk:completedAt a owl:DatatypeProperty ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range xsd:dateTime .

# ... (more properties)
```

---

## 10. Summary

**Extension Categories:**
- **Performance:** 4 properties (tickBudget, useSimd, priority, maxLatency)
- **Provenance:** 3 properties (hasProvenanceChain, commitHash, repositoryUri)
- **Observability:** 3 properties (hasSpanTemplate, hasMetricTemplate, traceIdRequired)
- **Security:** 5 properties (requiresCapability, requiresRole, encryptionRequired, etc.)
- **Runtime:** 6 properties (hasState, startedAt, completedAt, hasCurrentTask, etc.)

**Total Extensions:** 21 properties, 4 classes

**Integration Points:**
- Parser extracts knhk properties during TTL parsing
- Validator validates using SHACL shapes
- Executor uses knhk properties for runtime decisions
- Weaver validates OTEL compliance

**Benefits:**
- ✅ Non-invasive (separate namespace)
- ✅ Backward compatible (YAWL tools ignore)
- ✅ Forward compatible (knhk understands YAWL)
- ✅ Versioned (semantic versioning)
- ✅ Validated (SHACL + Weaver)
