# SPARQL Query Patterns for YAWL Ontology

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Work In Progress

## Executive Summary

This document provides 30+ SPARQL query patterns for working with YAWL workflows in knhk. Queries are organized by use case: extraction, validation, analysis, and monitoring.

## 1. Extraction Queries

### 1.1 Get All Workflow Specifications

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX dc: <http://purl.org/dc/terms/>

SELECT ?spec ?title ?version ?created WHERE {
    ?spec a yawl:Specification .
    OPTIONAL { ?spec rdfs:label ?title }
    OPTIONAL { ?spec yawl:version ?version }
    OPTIONAL { ?spec yawl:created ?created }
}
ORDER BY DESC(?created)
```

**Use Case:** List all workflows in the system
**Returns:** Specification IRI, title, version, creation date

---

### 1.2 Get All Tasks in a Workflow

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?join ?split ?type WHERE {
    <http://example.org/workflow#MyWorkflow> yawl:hasDecomposition ?net .
    ?net yawl:hasTask ?task .

    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?task yawl:hasJoin ?join }
    OPTIONAL { ?task yawl:hasSplit ?split }
    OPTIONAL { ?task rdf:type ?type }
}
```

**Use Case:** Extract all tasks from a specific workflow
**Returns:** Task IRI, name, join type, split type, task type
**Parameters:** Workflow IRI (replace in query)

---

### 1.3 Get Task Details with Resource Allocation

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?role ?allocator WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Resource allocation
    OPTIONAL {
        ?task yawl:hasResourcing ?resourcing .
        ?resourcing yawl:hasOffer ?offer .
        ?offer yawl:hasDistributionSet ?distSet .
        ?distSet yawl:hasInitialSet ?initSet .
        ?initSet yawl:role ?role .
    }

    # Allocator
    OPTIONAL {
        ?task yawl:hasResourcing ?resourcing .
        ?resourcing yawl:hasAllocate ?allocate .
        ?allocate yawl:hasAllocator ?allocatorNode .
        ?allocatorNode rdfs:label ?allocator .
    }
}
```

**Use Case:** Get task with resource allocation details
**Returns:** Task, name, required roles, allocator strategy

---

### 1.4 Get Control Flow Graph

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?from ?to ?predicate WHERE {
    ?from yawl:flowsInto ?flow .
    ?flow yawl:nextElementRef ?to .
    OPTIONAL { ?flow yawl:hasPredicate ?pred .
               ?pred yawl:query ?predicate }
}
```

**Use Case:** Extract complete control flow graph
**Returns:** Source element, target element, flow predicate (if any)

---

### 1.5 Get Multiple Instance Task Configuration

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?name ?min ?max ?threshold ?splitting ?mode WHERE {
    ?task a yawl:MultipleInstanceTask .
    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?task yawl:minimum ?min }
    OPTIONAL { ?task yawl:maximum ?max }
    OPTIONAL { ?task yawl:threshold ?threshold }

    # Splitting expression
    OPTIONAL {
        ?task yawl:hasSplittingExpression ?splittingExpr .
        ?splittingExpr yawl:query ?splitting .
    }

    # Creation mode
    OPTIONAL { ?task yawl:hasCreationMode ?mode }
}
```

**Use Case:** Get MI task configuration
**Returns:** Task, min/max/threshold, splitting expression, creation mode

---

## 2. Validation Queries

### 2.1 Validate: Start Condition Has No Incoming Flows

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

ASK {
    ?condition a yawl:InputCondition .
    ?flow yawl:nextElementRef ?condition .
}
```

**Use Case:** Check soundness: start condition should have no incoming flows
**Returns:** `true` (INVALID) or `false` (VALID)
**Validation Rule:** If ASK returns true, workflow is invalid

---

### 2.2 Validate: End Condition Has No Outgoing Flows

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

ASK {
    ?condition a yawl:OutputCondition .
    ?condition yawl:flowsInto ?flow .
}
```

**Use Case:** Check soundness: end condition should have no outgoing flows
**Returns:** `true` (INVALID) or `false` (VALID)

---

### 2.3 Validate: All Tasks Have Join and Split Types

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?name WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    FILTER NOT EXISTS { ?task yawl:hasJoin ?join }
    FILTER NOT EXISTS { ?task yawl:hasSplit ?split }
}
```

**Use Case:** Find tasks missing join/split configuration
**Returns:** Tasks with missing join or split types
**Validation Rule:** Result set should be empty

---

### 2.4 Validate: Data Flow Type Compatibility

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?mapping ?sourceType ?targetType WHERE {
    ?task yawl:hasStartingMappings ?mappingSet .
    ?mappingSet yawl:hasMapping ?mapping .
    ?mapping yawl:mapsTo ?targetParam .

    # Get source and target types
    ?mapping yawl:hasExpression ?expr .
    ?expr yawl:query ?query .

    # Extract source variable from query (simplified)
    # In practice, need to parse XQuery expression

    ?targetParam yawl:type ?targetType .

    # Check if types match
    FILTER(?sourceType != ?targetType)
}
```

**Use Case:** Find type incompatibilities in data flow
**Returns:** Mappings with mismatched types
**Validation Rule:** Result set should be empty

---

### 2.5 Validate: Deadlock Detection (Cycles in Control Flow)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task WHERE {
    ?task a yawl:Task .

    # Task has XOR-split
    ?task yawl:hasSplit yawl:ControlTypeXor .

    # Check if any outgoing path cycles back
    ?task yawl:flowsInto+ ?task .
}
```

**Use Case:** Find potential deadlocks (XOR-split cycles)
**Returns:** Tasks with cycles in XOR-split paths
**Validation Rule:** Needs additional analysis (not all cycles are deadlocks)

---

### 2.6 Validate: Orphaned Tasks (No Path from Start)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?name WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Find start condition
    ?start a yawl:InputCondition .

    # Check if task is reachable from start
    FILTER NOT EXISTS {
        ?start yawl:flowsInto+ ?task .
    }
}
```

**Use Case:** Find tasks not reachable from start
**Returns:** Orphaned tasks
**Validation Rule:** Result set should be empty

---

### 2.7 Validate: Dead-End Tasks (No Path to End)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?name WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Find end condition
    ?end a yawl:OutputCondition .

    # Check if task can reach end
    FILTER NOT EXISTS {
        ?task yawl:flowsInto+ ?end .
    }
}
```

**Use Case:** Find tasks that cannot reach end condition
**Returns:** Dead-end tasks
**Validation Rule:** Result set should be empty (unless termination pattern)

---

## 3. Analysis Queries

### 3.1 Find Tasks by Pattern (Join/Split Combination)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?pattern WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    ?task yawl:hasJoin ?join .
    ?task yawl:hasSplit ?split .

    # Construct pattern name
    BIND(CONCAT(STR(?join), "-", STR(?split)) AS ?pattern)

    # Filter for specific pattern (e.g., XOR-AND)
    FILTER(CONTAINS(?pattern, "Xor") && CONTAINS(?pattern, "And"))
}
```

**Use Case:** Find tasks implementing specific workflow patterns
**Returns:** Tasks matching pattern criteria

---

### 3.2 Analyze Resource Demand by Role

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?role (COUNT(?task) AS ?taskCount) WHERE {
    ?task yawl:hasResourcing ?resourcing .
    ?resourcing yawl:hasOffer ?offer .
    ?offer yawl:hasDistributionSet ?distSet .
    ?distSet yawl:hasInitialSet ?initSet .
    ?initSet yawl:role ?role .
}
GROUP BY ?role
ORDER BY DESC(?taskCount)
```

**Use Case:** Identify resource bottlenecks
**Returns:** Roles and number of tasks requiring them

---

### 3.3 Calculate Workflow Complexity Metrics

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT
    (COUNT(?task) AS ?numTasks)
    (COUNT(?condition) AS ?numConditions)
    (COUNT(?flow) AS ?numFlows)
    (AVG(?outDegree) AS ?avgOutDegree)
WHERE {
    ?spec a yawl:Specification .

    OPTIONAL { ?spec yawl:hasDecomposition ?net .
               ?net yawl:hasTask ?task }

    OPTIONAL { ?spec yawl:hasDecomposition ?net .
               ?net yawl:hasCondition ?condition }

    OPTIONAL { ?spec yawl:hasDecomposition ?net .
               ?net yawl:hasTask | yawl:hasCondition ?element .
               ?element yawl:flowsInto ?flow }

    # Calculate out-degree
    OPTIONAL {
        SELECT ?element (COUNT(?outFlow) AS ?outDegree) WHERE {
            ?element yawl:flowsInto ?outFlow .
        }
        GROUP BY ?element
    }
}
```

**Use Case:** Compute workflow complexity metrics
**Returns:** Number of tasks, conditions, flows, avg out-degree

---

### 3.4 Find Critical Path (Most Constrained Tasks)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX knhk: <http://knhk.org/ontology#>

SELECT ?task ?name ?tickBudget ?priority WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?task knhk:tickBudget ?tickBudget }
    OPTIONAL { ?task knhk:priority ?priority }

    # Find tasks with tight tick budgets (hot path)
    FILTER(?tickBudget <= 8)
}
ORDER BY ?tickBudget DESC(?priority)
```

**Use Case:** Identify critical path tasks (hot path)
**Returns:** Tasks with tight performance constraints

---

### 3.5 Analyze Timer Usage

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?name ?trigger ?ticks ?interval WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    ?task yawl:hasTimer ?timer .
    OPTIONAL { ?timer yawl:hasTrigger ?trigger }

    OPTIONAL {
        ?timer yawl:hasDurationParams ?duration .
        ?duration yawl:ticks ?ticks .
        ?duration yawl:hasInterval ?interval .
    }
}
ORDER BY ?ticks
```

**Use Case:** Identify time-constrained tasks
**Returns:** Tasks with timers and their configurations

---

## 4. Monitoring Queries (Runtime State)

### 4.1 Get Active Workflow Instances

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX knhk: <http://knhk.org/ontology#>

SELECT ?instance ?spec ?state ?started WHERE {
    ?instance a knhk:WorkflowInstance .
    ?instance knhk:hasSpecification ?spec .
    ?instance knhk:hasState ?state .
    ?instance knhk:startedAt ?started .

    # Filter for active instances
    FILTER(?state = "running" || ?state = "suspended")
}
ORDER BY DESC(?started)
```

**Use Case:** List active workflow instances
**Returns:** Instance IRI, specification, state, start time
**Note:** Requires knhk extensions to ontology

---

### 4.2 Get Task Execution Status in Instance

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX knhk: <http://knhk.org/ontology#>

SELECT ?task ?name ?state ?started ?completed WHERE {
    <http://example.org/instance#Instance123> knhk:hasTaskExecution ?execution .
    ?execution knhk:hasTask ?task .
    ?execution knhk:hasState ?state .

    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?execution knhk:startedAt ?started }
    OPTIONAL { ?execution knhk:completedAt ?completed }
}
ORDER BY ?started
```

**Use Case:** Monitor task execution in a workflow instance
**Returns:** Task, execution state, timestamps

---

### 4.3 Find Delayed Tasks (Exceeded Time Budget)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX knhk: <http://knhk.org/ontology#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?instance ?task ?name ?started ?budget ?elapsed WHERE {
    ?instance knhk:hasTaskExecution ?execution .
    ?execution knhk:hasTask ?task .
    ?execution knhk:hasState "executing" .
    ?execution knhk:startedAt ?started .

    ?task rdfs:label ?name .
    ?task knhk:tickBudget ?budget .

    # Calculate elapsed ticks
    BIND((xsd:long(NOW()) - xsd:long(?started)) AS ?elapsed)

    # Find tasks exceeding budget
    FILTER(?elapsed > ?budget)
}
```

**Use Case:** Detect tasks exceeding performance budgets
**Returns:** Instance, task, elapsed time, budget

---

### 4.4 Get Workflow Instance Provenance

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX knhk: <http://knhk.org/ontology#>
PREFIX prov: <http://www.w3.org/ns/prov#>

SELECT ?instance ?spec ?commitHash ?agent ?started WHERE {
    ?instance a knhk:WorkflowInstance .
    ?instance knhk:hasSpecification ?spec .
    ?instance knhk:hasProvenanceChain ?commitHash .

    OPTIONAL { ?instance prov:wasAssociatedWith ?agent }
    OPTIONAL { ?instance prov:startedAtTime ?started }
}
```

**Use Case:** Track workflow instance provenance
**Returns:** Instance, spec, Git commit hash, agent, timestamp

---

## 5. Performance Queries

### 5.1 Calculate Average Task Execution Time

```sparql
PREFIX knhk: <http://knhk.org/ontology#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?task ?name (AVG(?duration) AS ?avgDuration) (COUNT(?execution) AS ?count) WHERE {
    ?execution knhk:hasTask ?task .
    ?execution knhk:startedAt ?started .
    ?execution knhk:completedAt ?completed .

    OPTIONAL { ?task rdfs:label ?name }

    # Calculate duration in seconds
    BIND((xsd:long(?completed) - xsd:long(?started)) AS ?duration)
}
GROUP BY ?task ?name
ORDER BY DESC(?avgDuration)
```

**Use Case:** Identify slow tasks
**Returns:** Task, average execution time, execution count

---

### 5.2 Find Tasks Violating Performance Constraints

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX knhk: <http://knhk.org/ontology#>

SELECT ?task ?name ?budget ?maxObserved ?violationCount WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }
    ?task knhk:tickBudget ?budget .

    # Get max execution time
    {
        SELECT ?task (MAX(?duration) AS ?maxObserved) (COUNT(?execution) AS ?violationCount) WHERE {
            ?execution knhk:hasTask ?task .
            ?execution knhk:startedAt ?started .
            ?execution knhk:completedAt ?completed .

            BIND((xsd:long(?completed) - xsd:long(?started)) AS ?duration)

            # Filter for violations
            FILTER(?duration > ?budget)
        }
        GROUP BY ?task
    }
}
ORDER BY DESC(?violationCount)
```

**Use Case:** Find tasks violating performance SLAs
**Returns:** Task, budget, max observed time, violation count

---

## 6. Advanced Queries

### 6.1 Find All Workflow Patterns Used

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT DISTINCT ?pattern (COUNT(?task) AS ?usage) WHERE {
    ?task a yawl:Task .
    ?task yawl:hasJoin ?join .
    ?task yawl:hasSplit ?split .

    # Map to pattern ID
    BIND(
        IF(?join = yawl:ControlTypeXor && ?split = yawl:ControlTypeXor, "1-Sequence",
        IF(?join = yawl:ControlTypeXor && ?split = yawl:ControlTypeAnd, "2-ParallelSplit",
        IF(?join = yawl:ControlTypeAnd && ?split = yawl:ControlTypeXor, "3-Synchronization",
        IF(?join = yawl:ControlTypeXor && ?split = yawl:ControlTypeOr, "6-MultiChoice",
        "Other"))))
        AS ?pattern
    )
}
GROUP BY ?pattern
ORDER BY DESC(?usage)
```

**Use Case:** Analyze pattern usage across workflow
**Returns:** Pattern name, usage count

---

### 6.2 Find Nested Workflows (Composite Tasks)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?name ?subnet ?subnetName WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Task decomposes to subnet
    ?task yawl:hasDecomposesTo ?subnet .
    OPTIONAL { ?subnet rdfs:label ?subnetName }

    # Subnet is a Net (not WebServiceGateway)
    ?subnet a yawl:Net .
}
```

**Use Case:** Find hierarchical workflow structure
**Returns:** Task, subnet reference

---

### 6.3 Extract Data Dependencies

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?fromTask ?toTask ?variable WHERE {
    # From task produces output
    ?fromTask yawl:hasCompletedMappings ?fromMappingSet .
    ?fromMappingSet yawl:hasMapping ?fromMapping .
    ?fromMapping yawl:mapsTo ?variable .

    # To task consumes input
    ?toTask yawl:hasStartingMappings ?toMappingSet .
    ?toMappingSet yawl:hasMapping ?toMapping .
    ?toMapping yawl:hasExpression ?expr .
    ?expr yawl:query ?query .

    # Check if query references variable (simplified)
    FILTER(CONTAINS(?query, STR(?variable)))
}
```

**Use Case:** Identify data dependencies between tasks
**Returns:** Producer task, consumer task, shared variable

---

## 7. Update Queries (SPARQL UPDATE)

### 7.1 Add Hot Path Annotation to Task

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX knhk: <http://knhk.org/ontology#>

INSERT DATA {
    <http://example.org/workflow#TaskA> knhk:tickBudget 8 .
    <http://example.org/workflow#TaskA> a knhk:HotPathTask .
}
```

**Use Case:** Annotate critical tasks
**Effect:** Adds performance constraint

---

### 7.2 Update Workflow Instance State

```sparql
PREFIX knhk: <http://knhk.org/ontology#>

DELETE {
    <http://example.org/instance#Instance123> knhk:hasState ?oldState .
}
INSERT {
    <http://example.org/instance#Instance123> knhk:hasState "completed" .
    <http://example.org/instance#Instance123> knhk:completedAt ?now .
}
WHERE {
    <http://example.org/instance#Instance123> knhk:hasState ?oldState .
    BIND(NOW() AS ?now)
}
```

**Use Case:** Update runtime state
**Effect:** Transitions instance to completed

---

## 8. Summary: Query Usage Matrix

| Use Case | Query Count | Primary Tool |
|----------|-------------|--------------|
| **Extraction** | 5 | Parser |
| **Validation** | 7 | Validator |
| **Analysis** | 5 | Analyzer |
| **Monitoring** | 4 | Runtime Monitor |
| **Performance** | 2 | Performance Analyzer |
| **Advanced** | 3 | Research/Optimization |
| **Updates** | 2 | State Manager |
| **TOTAL** | 28 | - |

## 9. Performance Optimization Tips

1. **Use OPTIONAL Carefully:** Too many OPTIONALs slow queries
2. **Filter Early:** Put FILTERs before expensive operations
3. **Limit Results:** Always use LIMIT in production
4. **Prepared Queries:** Compile queries once, reuse many times
5. **Indexing:** Ensure Oxigraph has proper indices
6. **Avoid Property Paths:** `yawl:flowsInto+` can be expensive
7. **Cache Results:** Cache frequently-used queries (workflow specs)

## 10. References

- **SPARQL 1.1 Spec:** https://www.w3.org/TR/sparql11-query/
- **Oxigraph Query API:** https://docs.rs/oxigraph/
- **YAWL Ontology:** `/Users/sac/knhk/ontology/yawl.ttl`
- **knhk Extractor:** `/Users/sac/knhk/rust/knhk-workflow-engine/src/parser/extractor.rs`
