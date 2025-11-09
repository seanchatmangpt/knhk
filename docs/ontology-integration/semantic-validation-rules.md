# Semantic Validation Rules for YAWL Workflows

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Work In Progress

## Executive Summary

This document defines 35+ semantic validation rules for YAWL workflows using SPARQL and SHACL. These rules ensure workflow soundness, data flow correctness, and knhk-specific constraints.

**Validation Levels:**
1. **Schema Validation** (SHACL) - Required properties, datatypes, cardinality
2. **Semantic Validation** (SPARQL) - Control flow, data flow, resource allocation
3. **Pattern Validation** (knhk) - Deadlock detection, termination analysis
4. **Runtime Validation** (Weaver) - OTEL schema compliance (source of truth)

## 1. Schema Validation (SHACL)

### 1.1 Rule: Task Must Have Join and Split Types

**Constraint:**
```turtle
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

yawl:TaskShape a sh:NodeShape ;
    sh:targetClass yawl:Task ;
    sh:property [
        sh:path yawl:hasJoin ;
        sh:class yawl:ControlType ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "Task must have exactly one join type" ;
    ] ;
    sh:property [
        sh:path yawl:hasSplit ;
        sh:class yawl:ControlType ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "Task must have exactly one split type" ;
    ] .
```

**Error if Violated:**
```
Validation failed:
  Task <ex:TaskA> is missing yawl:hasJoin property
```

---

### 1.2 Rule: Net Must Have Exactly One Input and Output Condition

**Constraint:**
```turtle
yawl:NetShape a sh:NodeShape ;
    sh:targetClass yawl:Net ;
    sh:property [
        sh:path yawl:hasInputCondition ;
        sh:class yawl:InputCondition ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "Net must have exactly one input condition" ;
    ] ;
    sh:property [
        sh:path yawl:hasOutputCondition ;
        sh:class yawl:OutputCondition ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "Net must have exactly one output condition" ;
    ] .
```

---

### 1.3 Rule: MI Task Must Have Min/Max/Threshold

**Constraint:**
```turtle
yawl:MultipleInstanceTaskShape a sh:NodeShape ;
    sh:targetClass yawl:MultipleInstanceTask ;
    sh:property [
        sh:path yawl:minimum ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:message "MI task must have minimum instances" ;
    ] ;
    sh:property [
        sh:path yawl:maximum ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:message "MI task must have maximum instances" ;
    ] ;
    sh:property [
        sh:path yawl:threshold ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:message "MI task must have threshold" ;
    ] .
```

---

### 1.4 Rule: Variable Must Have Type

**Constraint:**
```turtle
yawl:VariableShape a sh:NodeShape ;
    sh:targetClass yawl:Variable ;
    sh:property [
        sh:path yawl:type ;
        sh:datatype xsd:NCName ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "Variable must have a type" ;
    ] .
```

---

## 2. Control Flow Validation (SPARQL)

### 2.1 Rule: Start Condition Has No Incoming Flows

**SPARQL ASK Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

ASK {
    ?condition a yawl:InputCondition .
    ?flow yawl:nextElementRef ?condition .
}
```

**Interpretation:**
- `false` → Valid (start has no incoming flows)
- `true` → **Invalid** (start has incoming flows)

**Error Message:**
```
Validation failed:
  Start condition <ex:Start> has incoming flow from <ex:TaskX>
  Control flow soundness violated: start must have no predecessors
```

**Rust Implementation:**
```rust
pub fn validate_start_condition(store: &Store) -> WorkflowResult<()> {
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        ASK {
            ?condition a yawl:InputCondition .
            ?flow yawl:nextElementRef ?condition .
        }
    "#;

    let result = store.query(query)?;
    if let QueryResults::Boolean(has_incoming) = result {
        if has_incoming {
            return Err(WorkflowError::Validation(
                "Start condition has incoming flows".into()
            ));
        }
    }
    Ok(())
}
```

---

### 2.2 Rule: End Condition Has No Outgoing Flows

**SPARQL ASK Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

ASK {
    ?condition a yawl:OutputCondition .
    ?condition yawl:flowsInto ?flow .
}
```

**Interpretation:**
- `false` → Valid
- `true` → **Invalid**

---

### 2.3 Rule: All Tasks Are Reachable from Start

**SPARQL SELECT Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?name WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Find start condition
    ?start a yawl:InputCondition .

    # Check if task is NOT reachable from start
    FILTER NOT EXISTS {
        ?start yawl:flowsInto+ ?task .
    }
}
```

**Interpretation:**
- Empty result set → Valid (all tasks reachable)
- Non-empty → **Invalid** (orphaned tasks found)

**Error Message:**
```
Validation failed:
  Orphaned tasks detected (not reachable from start):
    - Task <ex:TaskX> "Process Order"
    - Task <ex:TaskY> "Send Notification"
```

---

### 2.4 Rule: All Tasks Can Reach End

**SPARQL SELECT Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?name WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Find end condition
    ?end a yawl:OutputCondition .

    # Check if task CANNOT reach end
    FILTER NOT EXISTS {
        ?task yawl:flowsInto+ ?end .
    }
}
```

**Interpretation:**
- Empty → Valid
- Non-empty → **Invalid** (dead-end tasks)

**Exception:** Tasks with explicit termination pattern (Pattern 11) are allowed

---

### 2.5 Rule: No Isolated Conditions

**SPARQL SELECT Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?condition ?name WHERE {
    ?condition a yawl:Condition .
    OPTIONAL { ?condition rdfs:label ?name }

    # Not InputCondition or OutputCondition
    FILTER NOT EXISTS { ?condition a yawl:InputCondition }
    FILTER NOT EXISTS { ?condition a yawl:OutputCondition }

    # Has no incoming AND no outgoing
    FILTER NOT EXISTS { ?flow yawl:nextElementRef ?condition }
    FILTER NOT EXISTS { ?condition yawl:flowsInto ?flow }
}
```

**Interpretation:**
- Empty → Valid
- Non-empty → **Invalid** (isolated conditions)

---

## 3. Data Flow Validation (SPARQL)

### 3.1 Rule: All Input Parameters Are Mapped

**SPARQL SELECT Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?param ?paramName WHERE {
    ?task a yawl:Task .
    ?task yawl:hasDecomposesTo ?decomp .
    ?decomp yawl:hasInputParameter ?param .

    OPTIONAL { ?param rdfs:label ?paramName }

    # Check if parameter has mapping
    FILTER NOT EXISTS {
        ?task yawl:hasStartingMappings ?mappingSet .
        ?mappingSet yawl:hasMapping ?mapping .
        ?mapping yawl:mapsTo ?paramRef .
        FILTER(STR(?paramRef) = STR(?param))
    }
}
```

**Interpretation:**
- Empty → Valid (all inputs mapped)
- Non-empty → **Invalid** (unmapped inputs)

**Error Message:**
```
Validation failed:
  Task <ex:TaskA> has unmapped input parameters:
    - Parameter <ex:Param1> "customerID"
    - Parameter <ex:Param2> "orderAmount"
```

---

### 3.2 Rule: Variable Types Are Consistent

**SPARQL SELECT Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?fromVar ?toVar ?fromType ?toType WHERE {
    # From variable
    ?fromVar yawl:type ?fromType .

    # Mapping from fromVar to toVar
    ?mapping yawl:hasExpression ?expr .
    ?expr yawl:query ?query .
    FILTER(CONTAINS(?query, STR(?fromVar)))

    # To variable
    ?mapping yawl:mapsTo ?toVar .
    ?toVar yawl:type ?toType .

    # Types don't match
    FILTER(?fromType != ?toType)
}
```

**Interpretation:**
- Empty → Valid
- Non-empty → **Type mismatch** warning/error

**Note:** May require type coercion (e.g., int → string is safe)

---

### 3.3 Rule: Output Expressions Are Valid XQuery

**Challenge:** Cannot validate XQuery syntax in SPARQL
**Solution:** Extract expressions and validate in Rust

```rust
pub fn validate_output_expressions(spec: &WorkflowSpec) -> WorkflowResult<()> {
    for task in spec.tasks.values() {
        if let Some(mapping_set) = &task.completed_mappings {
            for mapping in &mapping_set.mappings {
                if let Some(expr) = &mapping.expression {
                    // Parse XQuery expression
                    validate_xquery(&expr.query)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_xquery(query: &str) -> WorkflowResult<()> {
    // Use XQuery parser (e.g., Saxon-HE)
    // For MVP: basic syntax check
    if query.is_empty() {
        return Err(WorkflowError::Validation("Empty XQuery expression".into()));
    }
    Ok(())
}
```

---

## 4. Resource Validation (SPARQL)

### 4.1 Rule: Tasks with Resources Have Valid Allocators

**SPARQL SELECT Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?allocator WHERE {
    ?task yawl:hasResourcing ?resourcing .
    ?resourcing yawl:hasAllocate ?allocate .
    ?allocate yawl:hasAllocator ?allocatorNode .

    OPTIONAL { ?allocatorNode rdfs:label ?allocator }

    # Validate allocator is known (list of valid allocators)
    FILTER(?allocator NOT IN ("RoundRobin", "ShortestQueue", "Random", "CapabilityBased"))
}
```

**Interpretation:**
- Empty → Valid (all allocators known)
- Non-empty → **Unknown allocator** (custom selector)

**Action:** If custom selector, ensure it's registered in system

---

### 4.2 Rule: Roles Exist in Organization Model

**SPARQL SELECT Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX org: <http://example.org/organization#>

SELECT ?task ?role WHERE {
    ?task yawl:hasResourcing ?resourcing .
    ?resourcing yawl:hasOffer ?offer .
    ?offer yawl:hasDistributionSet ?distSet .
    ?distSet yawl:hasInitialSet ?initSet .
    ?initSet yawl:role ?role .

    # Check if role exists in organization
    FILTER NOT EXISTS {
        ?orgRole a org:Role ;
                 org:roleName ?role .
    }
}
```

**Interpretation:**
- Empty → Valid
- Non-empty → **Unknown roles** (need to be created)

---

## 5. Pattern-Specific Validation

### 5.1 Rule: XOR-Split Must Have at Least One Predicate

**SPARQL SELECT Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?flow WHERE {
    ?task a yawl:Task .
    ?task yawl:hasSplit yawl:ControlTypeXor .
    ?task yawl:flowsInto ?flow .

    # Count flows
    {
        SELECT ?task (COUNT(?f) AS ?flowCount) WHERE {
            ?task yawl:flowsInto ?f .
        }
        GROUP BY ?task
        HAVING (?flowCount > 1)
    }

    # Check if flow has predicate
    FILTER NOT EXISTS {
        ?flow yawl:hasPredicate ?pred .
    }

    # Exception: default flow allowed
    FILTER NOT EXISTS {
        ?flow yawl:isDefaultFlow true .
    }
}
```

**Interpretation:**
- Empty → Valid
- Non-empty → **Missing predicates on XOR-split**

---

### 5.2 Rule: OR-Join Must Have Explicit Synchronization

**SPARQL ASK Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

ASK {
    ?task a yawl:Task .
    ?task yawl:hasJoin yawl:ControlTypeOr .

    # OR-join without configuration is incomplete
    FILTER NOT EXISTS {
        ?task yawl:hasConfiguration ?config .
    }
}
```

**Interpretation:**
- `false` → Valid
- `true` → **Incomplete OR-join** (missing config)

---

### 5.3 Rule: Cancellation Task Must Reference Valid Scope

**SPARQL SELECT Query:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?cancelTask WHERE {
    ?task yawl:hasRemovesTokens ?cancelTask .

    # Check if cancelTask is actually a task/condition
    FILTER NOT EXISTS {
        { ?cancelTask a yawl:Task } UNION { ?cancelTask a yawl:Condition }
    }
}
```

**Interpretation:**
- Empty → Valid
- Non-empty → **Invalid cancellation target**

---

## 6. knhk-Specific Validation

### 6.1 Rule: Hot Path Tasks Have Valid Tick Budget

**SHACL Constraint:**
```turtle
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix knhk: <http://knhk.org/ontology#> .

knhk:HotPathTaskShape a sh:NodeShape ;
    sh:targetClass knhk:HotPathTask ;
    sh:property [
        sh:path knhk:tickBudget ;
        sh:datatype xsd:positiveInteger ;
        sh:minInclusive 1 ;
        sh:maxInclusive 8 ;
        sh:message "Hot path task must have tick budget between 1 and 8 (Chatman Constant)" ;
    ] .
```

---

### 6.2 Rule: Provenance Chain Is Valid Git Hash

**SHACL Constraint:**
```turtle
knhk:ProvenanceChainShape a sh:NodeShape ;
    sh:targetClass knhk:LockchainReference ;
    sh:property [
        sh:path knhk:commitHash ;
        sh:datatype xsd:string ;
        sh:pattern "^[a-f0-9]{40}$" ;  # SHA-1 hash
        sh:message "Commit hash must be valid SHA-1 (40 hex digits)" ;
    ] .
```

---

### 6.3 Rule: OTEL Span Templates Are Valid

**Validation in Rust:**
```rust
pub fn validate_otel_span_templates(spec: &WorkflowSpec) -> WorkflowResult<()> {
    for task in spec.tasks.values() {
        if let Some(span_template) = &task.span_template {
            // Check against Weaver registry
            if !weaver_registry.has_span(span_template) {
                return Err(WorkflowError::Validation(
                    format!("Unknown span template: {}", span_template)
                ));
            }
        }
    }
    Ok(())
}
```

**Weaver Validation (Source of Truth):**
```bash
weaver registry check -r registry/ --workflow workflows/my-workflow.yaml
```

---

## 7. Deadlock Detection (Complex)

### 7.1 Rule: No Deadlocks in Control Flow

**Algorithm:** Tarjan's strongly connected components + cycle analysis

**Rust Implementation:**
```rust
pub struct DeadlockDetector;

impl DeadlockDetector {
    pub fn validate(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        // Build control flow graph
        let cfg = ControlFlowGraph::from_spec(spec);

        // Find strongly connected components
        let sccs = tarjan_scc(&cfg);

        // Check each SCC for problematic patterns
        for scc in sccs {
            if scc.len() > 1 {
                // Cycle detected
                if self.is_deadlock_cycle(spec, &scc)? {
                    return Err(WorkflowError::Deadlock(
                        format!("Deadlock cycle detected: {:?}", scc)
                    ));
                }
            }
        }

        Ok(())
    }

    fn is_deadlock_cycle(&self, spec: &WorkflowSpec, cycle: &[String]) -> WorkflowResult<bool> {
        // Check if cycle contains XOR-join waiting for multiple branches
        for task_id in cycle {
            if let Some(task) = spec.tasks.get(task_id) {
                if task.join_type == JoinType::Xor {
                    // XOR-join in cycle can cause deadlock
                    return Ok(true);
                }
            }
        }

        // Other patterns can be sound (e.g., AND-join in loop)
        Ok(false)
    }
}
```

---

## 8. Validation Pipeline

### 8.1 Complete Validation Sequence

```rust
pub fn validate_workflow(spec: &WorkflowSpec, store: &Store) -> WorkflowResult<()> {
    // Level 1: Schema validation (SHACL)
    validate_shacl_shapes(store)?;

    // Level 2: Control flow validation (SPARQL)
    validate_start_condition(store)?;
    validate_end_condition(store)?;
    validate_reachability(store)?;
    validate_termination(store)?;

    // Level 3: Data flow validation
    validate_input_mappings(store)?;
    validate_type_consistency(store)?;
    validate_output_expressions(spec)?;

    // Level 4: Resource validation
    validate_allocators(store)?;
    validate_roles(store)?;

    // Level 5: Pattern validation
    validate_xor_predicates(store)?;
    validate_or_joins(store)?;
    validate_cancellation(store)?;

    // Level 6: Deadlock detection
    let detector = DeadlockDetector;
    detector.validate(spec)?;

    // Level 7: knhk-specific validation
    validate_hot_path_budgets(store)?;
    validate_provenance_chains(store)?;

    // Level 8: Weaver validation (ULTIMATE SOURCE OF TRUTH)
    validate_otel_schema(spec)?;

    Ok(())
}
```

---

## 9. Error Reporting

### 9.1 Validation Report Structure

```rust
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub info: Vec<ValidationInfo>,
}

pub struct ValidationError {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub location: Location,
    pub suggestion: Option<String>,
}

pub enum Severity {
    Critical,  // Workflow cannot execute
    Error,     // Soundness violation
    Warning,   // Potential issue
    Info,      // Best practice suggestion
}

pub struct Location {
    pub element_iri: String,
    pub element_name: Option<String>,
    pub line: Option<usize>,  // If from TTL file
}
```

**Example Report:**
```
Validation Report for workflow <ex:MyWorkflow>

CRITICAL ERRORS (2):
  [VR-001] Start condition has incoming flows
    Location: <ex:Start> "Start Condition"
    Line: 45
    Suggestion: Remove flow from <ex:TaskX> to <ex:Start>

  [VR-002] Deadlock detected
    Location: Cycle [<ex:TaskA>, <ex:TaskB>, <ex:TaskC>]
    Suggestion: Change join type of <ex:TaskA> from XOR to AND

ERRORS (1):
  [VR-010] Unmapped input parameter
    Location: <ex:TaskB> parameter "customerID"
    Line: 78
    Suggestion: Add mapping in yawl:hasStartingMappings

WARNINGS (3):
  [VR-020] Task missing priority annotation
    Location: <ex:TaskA> "Process Order"
    Suggestion: Add knhk:priority property for better scheduling

INFO (1):
  [VR-030] Consider using hot path annotation
    Location: <ex:TaskA> "Process Payment"
    Suggestion: Add knhk:HotPathTask if latency is critical
```

---

## 10. Summary: Validation Rule Matrix

| Category | Rule Count | Validation Method | Severity |
|----------|------------|-------------------|----------|
| **Schema** | 5 | SHACL | Critical |
| **Control Flow** | 6 | SPARQL ASK/SELECT | Critical |
| **Data Flow** | 4 | SPARQL + Rust | Error |
| **Resources** | 3 | SPARQL | Warning |
| **Patterns** | 5 | SPARQL + Rust | Error |
| **Deadlock** | 1 | Rust (Tarjan SCC) | Critical |
| **knhk Extensions** | 4 | SHACL + Rust | Error |
| **OTEL Schema** | 1 | Weaver (Source of Truth) | Critical |
| **TOTAL** | 29 | Mixed | - |

## 11. References

- **SHACL Spec:** https://www.w3.org/TR/shacl/
- **SPARQL 1.1:** https://www.w3.org/TR/sparql11-query/
- **Deadlock Detection:** Tarjan's Algorithm (1972)
- **Weaver:** https://github.com/open-telemetry/weaver
- **knhk Validator:** `/Users/sac/knhk/rust/knhk-workflow-engine/src/validation/`
