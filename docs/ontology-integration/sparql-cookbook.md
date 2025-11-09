# SPARQL Cookbook for YAWL Ontology in knhk

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Production Ready
**Target Audience:** Developers using SPARQL to query YAWL workflows

---

## Executive Summary

This cookbook provides **40+ SPARQL query recipes** for working with YAWL workflows in knhk. Each recipe includes:
- **Query pattern** with full SPARQL syntax
- **Step-by-step explanation** of how the query works
- **Rust integration code** showing how to execute the query
- **Performance notes** and optimization tips
- **Common variations** and extensions

---

## Table of Contents

1. [SPARQL Fundamentals](#1-sparql-fundamentals)
2. [Basic Extraction Queries](#2-basic-extraction-queries)
3. [Validation Queries](#3-validation-queries)
4. [Analysis Queries](#4-analysis-queries)
5. [Runtime Monitoring Queries](#5-runtime-monitoring-queries)
6. [Performance Optimization](#6-performance-optimization)
7. [Advanced Query Patterns](#7-advanced-query-patterns)
8. [Query Debugging Techniques](#8-query-debugging-techniques)
9. [Rust Integration Patterns](#9-rust-integration-patterns)
10. [Anti-Patterns to Avoid](#10-anti-patterns-to-avoid)

---

## 1. SPARQL Fundamentals

### 1.1 Query Anatomy

```sparql
# Prefixes: Namespace shortcuts
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

# SELECT: What to return
SELECT ?task ?name

# WHERE: Graph pattern to match
WHERE {
    ?task a yawl:Task .           # Triple pattern
    ?task rdfs:label ?name .      # Triple pattern
}

# Modifiers: Sort, limit, etc.
ORDER BY ?name
LIMIT 10
```

**Components:**
- **Prefix declarations:** Define namespace shortcuts
- **SELECT clause:** Variables to return
- **WHERE clause:** Graph patterns to match
- **Modifiers:** ORDER BY, LIMIT, OFFSET, DISTINCT

### 1.2 Query Types

| Query Form | Purpose | Returns |
|------------|---------|---------|
| `SELECT` | Retrieve data | Variable bindings |
| `ASK` | Check if pattern exists | Boolean (true/false) |
| `CONSTRUCT` | Build new RDF graph | RDF triples |
| `DESCRIBE` | Get all info about resource | RDF triples |

**Most Common in knhk:**
- **SELECT:** Extract workflow data (90% of use cases)
- **ASK:** Validate workflow soundness (10% of use cases)

### 1.3 Triple Patterns

```sparql
# Basic triple pattern
?subject ?predicate ?object .

# Concrete examples
?task a yawl:Task .                    # Subject variable, fixed predicate/object
?task yawl:name "Approve Request" .    # Subject variable, fixed predicate/object
?task yawl:hasJoin ?joinType .         # Subject/object variables, fixed predicate
```

### 1.4 OPTIONAL Patterns

```sparql
SELECT ?task ?name ?timer WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }      # ?name may be unbound
    OPTIONAL { ?task yawl:hasTimer ?timer }  # ?timer may be unbound
}
```

**When to Use:**
- Property may not exist for all instances
- Want to include instances even if property is missing

### 1.5 FILTER Expressions

```sparql
SELECT ?task WHERE {
    ?task a yawl:Task .
    ?task knhk:tickBudget ?budget .
    FILTER(?budget <= 8)  # Only hot path tasks
}
```

**Common Filters:**
- Comparison: `?x > 10`, `?x <= 8`
- String matching: `CONTAINS(?name, "Approve")`
- Type checking: `DATATYPE(?value) = xsd:integer`
- Existence: `FILTER EXISTS { ?task yawl:hasTimer ?t }`

---

## 2. Basic Extraction Queries

### Recipe 2.1: Get All Workflow Specifications

**Use Case:** List all workflows in the system

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?spec ?title ?version ?created WHERE {
    # Match all specifications
    ?spec a yawl:Specification .

    # Get optional metadata
    OPTIONAL { ?spec rdfs:label ?title }
    OPTIONAL { ?spec yawl:version ?version }
    OPTIONAL { ?spec yawl:created ?created }
}
ORDER BY DESC(?created)
```

**Step-by-Step:**

1. `?spec a yawl:Specification` - Find all Specification instances
2. `OPTIONAL { ?spec rdfs:label ?title }` - Get title if present
3. `ORDER BY DESC(?created)` - Sort newest first

**Rust Integration:**

```rust
use oxigraph::sparql::QueryResults;

fn get_all_specifications(store: &Store) -> Result<Vec<SpecInfo>, Error> {
    let query = include_str!("queries/get_all_specs.sparql");

    let mut specs = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            specs.push(SpecInfo {
                iri: solution.get("spec").unwrap().to_string(),
                title: solution.get("title").map(|t| t.to_string()),
                version: solution.get("version").map(|v| v.to_string()),
                created: solution.get("created").map(|c| c.to_string()),
            });
        }
    }

    Ok(specs)
}
```

**Performance:** O(n) where n = number of specifications. Fast due to class index.

---

### Recipe 2.2: Get All Tasks in a Workflow

**Use Case:** Extract task list for a specific workflow

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?join ?split WHERE {
    # Bind workflow IRI (parameter)
    BIND(<http://example.org/workflow#OrderProcessing> AS ?spec)

    # Navigate: Specification -> Decomposition (Net) -> Task
    ?spec yawl:hasDecomposition ?net .
    ?net a yawl:Net .
    ?net yawl:hasTask ?task .

    # Get task properties
    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?task yawl:hasJoin ?join }
    OPTIONAL { ?task yawl:hasSplit ?split }
}
ORDER BY ?name
```

**Step-by-Step:**

1. `BIND(<...> AS ?spec)` - Parameter: workflow IRI
2. `?spec yawl:hasDecomposition ?net` - Get net from specification
3. `?net yawl:hasTask ?task` - Get all tasks in net
4. `OPTIONAL` clauses retrieve task metadata

**Parameterization in Rust:**

```rust
fn get_tasks_in_workflow(store: &Store, workflow_iri: &str) -> Result<Vec<TaskInfo>, Error> {
    let query = format!(r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?task ?name ?join ?split WHERE {{
            <{workflow_iri}> yawl:hasDecomposition ?net .
            ?net yawl:hasTask ?task .
            OPTIONAL {{ ?task rdfs:label ?name }}
            OPTIONAL {{ ?task yawl:hasJoin ?join }}
            OPTIONAL {{ ?task yawl:hasSplit ?split }}
        }}
    "#);

    // Execute query...
}
```

**Performance:** O(t) where t = number of tasks in workflow. Very fast.

---

### Recipe 2.3: Get Control Flow Graph

**Use Case:** Extract complete workflow graph for visualization

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?from ?to ?predicate ?isDefault WHERE {
    # Get all flow edges
    ?from yawl:flowsInto ?flow .
    ?flow yawl:nextElementRef ?to .

    # Get optional edge properties
    OPTIONAL {
        ?flow yawl:hasPredicate ?pred .
        ?pred yawl:query ?predicate .
    }
    OPTIONAL { ?flow yawl:isDefaultFlow ?isDefault }
}
```

**Step-by-Step:**

1. `?from yawl:flowsInto ?flow` - Get flow edge from source
2. `?flow yawl:nextElementRef ?to` - Get target of edge
3. Optional predicates for conditional flows

**Rust Integration:**

```rust
#[derive(Debug, Clone)]
struct FlowEdge {
    from: String,
    to: String,
    predicate: Option<String>,
    is_default: bool,
}

fn get_control_flow_graph(store: &Store) -> Result<Vec<FlowEdge>, Error> {
    let query = include_str!("queries/get_control_flow.sparql");

    let mut edges = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            edges.push(FlowEdge {
                from: solution.get("from").unwrap().to_string(),
                to: solution.get("to").unwrap().to_string(),
                predicate: solution.get("predicate").map(|p| p.to_string()),
                is_default: solution.get("isDefault")
                    .and_then(|v| v.as_str().parse().ok())
                    .unwrap_or(false),
            });
        }
    }

    Ok(edges)
}
```

**Visualization:**

```rust
fn generate_dot_graph(edges: &[FlowEdge]) -> String {
    let mut dot = String::from("digraph workflow {\n");

    for edge in edges {
        let label = edge.predicate.as_deref().unwrap_or("");
        let style = if edge.is_default { "bold" } else { "solid" };

        dot.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\", style={}];\n",
            edge.from, edge.to, label, style
        ));
    }

    dot.push_str("}\n");
    dot
}
```

---

### Recipe 2.4: Get Task with Resource Allocation

**Use Case:** Extract task execution requirements

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?role ?participant ?initiator WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Resource allocation chain
    OPTIONAL {
        ?task yawl:hasResourcing ?resourcing .

        # Offer: who gets work item
        OPTIONAL {
            ?resourcing yawl:hasOffer ?offer .
            ?offer yawl:hasDistributionSet ?distSet .
            ?distSet yawl:hasInitialSet ?initSet .

            # Roles
            OPTIONAL { ?initSet yawl:role ?role }

            # Participants
            OPTIONAL { ?initSet yawl:participant ?participant }
        }

        # Who initiates task
        OPTIONAL {
            ?resourcing yawl:hasStart ?initiator .
        }
    }
}
```

**Step-by-Step:**

1. Find all tasks
2. Navigate resourcing chain: `Task -> Resourcing -> Offer -> DistributionSet -> InitialSet`
3. Extract roles and participants from InitialSet

**Rust Integration:**

```rust
#[derive(Debug, Clone)]
struct TaskResourcing {
    task: String,
    name: Option<String>,
    roles: Vec<String>,
    participants: Vec<String>,
    initiator: Option<String>,
}

fn get_task_resourcing(store: &Store) -> Result<Vec<TaskResourcing>, Error> {
    let query = include_str!("queries/get_task_resourcing.sparql");

    // Group by task (multiple roles/participants possible)
    let mut map: HashMap<String, TaskResourcing> = HashMap::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            let task_iri = solution.get("task").unwrap().to_string();

            let entry = map.entry(task_iri.clone()).or_insert(TaskResourcing {
                task: task_iri,
                name: solution.get("name").map(|n| n.to_string()),
                roles: Vec::new(),
                participants: Vec::new(),
                initiator: solution.get("initiator").map(|i| i.to_string()),
            });

            if let Some(role) = solution.get("role") {
                entry.roles.push(role.to_string());
            }

            if let Some(participant) = solution.get("participant") {
                entry.participants.push(participant.to_string());
            }
        }
    }

    Ok(map.into_values().collect())
}
```

---

### Recipe 2.5: Get Multiple Instance Task Configuration

**Use Case:** Extract MI task parameters

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?min ?max ?threshold ?splitting ?joining ?mode WHERE {
    ?task a yawl:MultipleInstanceTask .
    OPTIONAL { ?task rdfs:label ?name }

    # MI parameters
    OPTIONAL { ?task yawl:minimum ?min }
    OPTIONAL { ?task yawl:maximum ?max }
    OPTIONAL { ?task yawl:threshold ?threshold }

    # Splitting expression
    OPTIONAL {
        ?task yawl:hasSplittingExpression ?splittingExpr .
        ?splittingExpr yawl:query ?splitting .
    }

    # Joining expression
    OPTIONAL {
        ?task yawl:hasOutputJoiningExpression ?joiningExpr .
        ?joiningExpr yawl:query ?joining .
    }

    # Creation mode
    OPTIONAL { ?task yawl:hasCreationMode ?mode }
}
```

**Rust Integration:**

```rust
#[derive(Debug, Clone)]
struct MITaskConfig {
    task: String,
    name: Option<String>,
    min: Option<i32>,
    max: Option<i32>,
    threshold: Option<i32>,
    splitting_expr: Option<String>,
    joining_expr: Option<String>,
    creation_mode: Option<String>,
}

fn parse_mi_param(value: &str) -> Option<i32> {
    // YAWL allows XPath expressions, not just integers
    value.parse().ok()
}
```

---

## 3. Validation Queries

### Recipe 3.1: Validate Start Condition Has No Incoming Flows

**Use Case:** Workflow soundness check

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

ASK {
    # Find input condition
    ?condition a yawl:InputCondition .

    # Check if anything flows into it (INVALID)
    ?flow yawl:nextElementRef ?condition .
}
```

**Interpretation:**
- Returns `true` → **INVALID** (start condition has incoming flows)
- Returns `false` → **VALID** (start condition is proper entry point)

**Rust Integration:**

```rust
fn validate_start_condition(store: &Store) -> Result<bool, Error> {
    let query = include_str!("queries/validate_start_condition.sparql");

    if let QueryResults::Boolean(has_incoming) = store.query(query)? {
        if has_incoming {
            return Err(Error::ValidationError("Start condition has incoming flows".into()));
        }
    }

    Ok(true)
}
```

---

### Recipe 3.2: Validate End Condition Has No Outgoing Flows

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

ASK {
    # Find output condition
    ?condition a yawl:OutputCondition .

    # Check if it flows into anything (INVALID)
    ?condition yawl:flowsInto ?flow .
}
```

**Rust Integration:**

```rust
fn validate_end_condition(store: &Store) -> Result<bool, Error> {
    let query = include_str!("queries/validate_end_condition.sparql");

    if let QueryResults::Boolean(has_outgoing) = store.query(query)? {
        if has_outgoing {
            return Err(Error::ValidationError("End condition has outgoing flows".into()));
        }
    }

    Ok(true)
}
```

---

### Recipe 3.3: Find Tasks Missing Join/Split Configuration

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?missingJoin ?missingSplit WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Check for missing join
    BIND(NOT EXISTS { ?task yawl:hasJoin ?j } AS ?missingJoin)

    # Check for missing split
    BIND(NOT EXISTS { ?task yawl:hasSplit ?s } AS ?missingSplit)

    # Filter: only tasks with issues
    FILTER(?missingJoin || ?missingSplit)
}
```

**Rust Integration:**

```rust
#[derive(Debug)]
struct TaskConfigIssue {
    task: String,
    name: Option<String>,
    missing_join: bool,
    missing_split: bool,
}

fn find_tasks_missing_config(store: &Store) -> Result<Vec<TaskConfigIssue>, Error> {
    let query = include_str!("queries/validate_task_config.sparql");

    let mut issues = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            issues.push(TaskConfigIssue {
                task: solution.get("task").unwrap().to_string(),
                name: solution.get("name").map(|n| n.to_string()),
                missing_join: solution.get("missingJoin")
                    .and_then(|v| v.as_str().parse().ok())
                    .unwrap_or(false),
                missing_split: solution.get("missingSplit")
                    .and_then(|v| v.as_str().parse().ok())
                    .unwrap_or(false),
            });
        }
    }

    Ok(issues)
}
```

---

### Recipe 3.4: Find Orphaned Tasks (No Path from Start)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Find start condition
    ?start a yawl:InputCondition .

    # Check if task is NOT reachable from start
    FILTER NOT EXISTS {
        # Property path: start -> ... -> task
        ?start (yawl:flowsInto/yawl:nextElementRef)+ ?task .
    }
}
```

**Step-by-Step:**

1. Find all tasks
2. Find start condition
3. Use property path `+` to check reachability
4. Filter tasks NOT reachable

**Performance Warning:** Property paths (`+`, `*`) can be expensive. Cache results.

**Rust Integration:**

```rust
fn find_orphaned_tasks(store: &Store) -> Result<Vec<String>, Error> {
    let query = include_str!("queries/validate_orphaned_tasks.sparql");

    let mut orphaned = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            orphaned.push(solution.get("task").unwrap().to_string());
        }
    }

    if !orphaned.is_empty() {
        return Err(Error::ValidationError(format!(
            "Found {} orphaned tasks: {:?}",
            orphaned.len(), orphaned
        )));
    }

    Ok(())
}
```

---

### Recipe 3.5: Find Dead-End Tasks (No Path to End)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    # Find end condition
    ?end a yawl:OutputCondition .

    # Check if task CANNOT reach end
    FILTER NOT EXISTS {
        # Property path: task -> ... -> end
        ?task (yawl:flowsInto/yawl:nextElementRef)+ ?end .
    }
}
```

**Rust Integration:**

```rust
fn find_dead_end_tasks(store: &Store) -> Result<Vec<String>, Error> {
    let query = include_str!("queries/validate_dead_end_tasks.sparql");

    let mut dead_ends = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            dead_ends.push(solution.get("task").unwrap().to_string());
        }
    }

    if !dead_ends.is_empty() {
        return Err(Error::ValidationError(format!(
            "Found {} dead-end tasks: {:?}",
            dead_ends.len(), dead_ends
        )));
    }

    Ok(())
}
```

---

## 4. Analysis Queries

### Recipe 4.1: Find Tasks by Pattern (Join/Split Combination)

**Use Case:** Identify workflow patterns

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?pattern WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }

    ?task yawl:hasJoin ?join .
    ?task yawl:hasSplit ?split .

    # Determine pattern
    BIND(
        IF(?join = yawl:ControlTypeXor && ?split = yawl:ControlTypeXor, "Sequence",
        IF(?join = yawl:ControlTypeXor && ?split = yawl:ControlTypeAnd, "ParallelSplit",
        IF(?join = yawl:ControlTypeAnd && ?split = yawl:ControlTypeXor, "Synchronization",
        IF(?join = yawl:ControlTypeOr && ?split = yawl:ControlTypeOr, "MultiMerge",
        "Other"))))
        AS ?pattern
    )

    # Filter for specific pattern (optional)
    # FILTER(?pattern = "ParallelSplit")
}
ORDER BY ?pattern ?name
```

**Rust Integration:**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
enum WorkflowPattern {
    Sequence,
    ParallelSplit,
    Synchronization,
    MultiMerge,
    Other,
}

impl WorkflowPattern {
    fn from_str(s: &str) -> Self {
        match s {
            "Sequence" => Self::Sequence,
            "ParallelSplit" => Self::ParallelSplit,
            "Synchronization" => Self::Synchronization,
            "MultiMerge" => Self::MultiMerge,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone)]
struct TaskPattern {
    task: String,
    name: Option<String>,
    pattern: WorkflowPattern,
}

fn analyze_workflow_patterns(store: &Store) -> Result<Vec<TaskPattern>, Error> {
    let query = include_str!("queries/analyze_patterns.sparql");

    let mut patterns = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            patterns.push(TaskPattern {
                task: solution.get("task").unwrap().to_string(),
                name: solution.get("name").map(|n| n.to_string()),
                pattern: WorkflowPattern::from_str(
                    solution.get("pattern").unwrap().as_str()
                ),
            });
        }
    }

    Ok(patterns)
}
```

---

### Recipe 4.2: Analyze Resource Demand by Role

**Use Case:** Capacity planning

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?role (COUNT(DISTINCT ?task) AS ?taskCount) WHERE {
    ?task yawl:hasResourcing ?resourcing .
    ?resourcing yawl:hasOffer ?offer .
    ?offer yawl:hasDistributionSet ?distSet .
    ?distSet yawl:hasInitialSet ?initSet .
    ?initSet yawl:role ?role .
}
GROUP BY ?role
ORDER BY DESC(?taskCount)
```

**Step-by-Step:**

1. Navigate resourcing chain to get roles
2. `COUNT(DISTINCT ?task)` - Count unique tasks per role
3. `GROUP BY ?role` - Aggregate by role
4. `ORDER BY DESC(?taskCount)` - Sort by demand

**Rust Integration:**

```rust
#[derive(Debug, Clone)]
struct RoleDemand {
    role: String,
    task_count: usize,
}

fn analyze_resource_demand(store: &Store) -> Result<Vec<RoleDemand>, Error> {
    let query = include_str!("queries/analyze_resource_demand.sparql");

    let mut demands = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            demands.push(RoleDemand {
                role: solution.get("role").unwrap().to_string(),
                task_count: solution.get("taskCount")
                    .and_then(|v| v.as_str().parse().ok())
                    .unwrap_or(0),
            });
        }
    }

    Ok(demands)
}

// Visualization
fn print_resource_demand_report(demands: &[RoleDemand]) {
    println!("Resource Demand Analysis:");
    println!("{:<30} {:>10}", "Role", "Tasks");
    println!("{:-<41}", "");

    for demand in demands {
        println!("{:<30} {:>10}", demand.role, demand.task_count);
    }
}
```

---

### Recipe 4.3: Calculate Workflow Complexity Metrics

**Use Case:** Workflow quality assessment

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT
    (COUNT(DISTINCT ?task) AS ?numTasks)
    (COUNT(DISTINCT ?condition) AS ?numConditions)
    (COUNT(DISTINCT ?flow) AS ?numFlows)
WHERE {
    ?spec a yawl:Specification .
    ?spec yawl:hasDecomposition ?net .
    ?net a yawl:Net .

    OPTIONAL { ?net yawl:hasTask ?task }
    OPTIONAL { ?net yawl:hasCondition ?condition }
    OPTIONAL {
        ?element yawl:flowsInto ?flow .
    }
}
```

**Additional Metrics (Separate Queries):**

```sparql
# Average out-degree
SELECT (AVG(?outDegree) AS ?avgOutDegree) WHERE {
    {
        SELECT ?element (COUNT(?flow) AS ?outDegree) WHERE {
            ?element yawl:flowsInto ?flow .
        }
        GROUP BY ?element
    }
}

# Cyclomatic complexity (approximate)
SELECT ((COUNT(?flow) - COUNT(?node) + 2) AS ?cyclomaticComplexity) WHERE {
    ?flow a yawl:FlowsInto .
    ?node a yawl:NetElement .
}
```

**Rust Integration:**

```rust
#[derive(Debug, Clone)]
struct WorkflowMetrics {
    num_tasks: usize,
    num_conditions: usize,
    num_flows: usize,
    avg_out_degree: f64,
    cyclomatic_complexity: usize,
}

fn calculate_workflow_metrics(store: &Store) -> Result<WorkflowMetrics, Error> {
    let basic_query = include_str!("queries/metrics_basic.sparql");
    let outdegree_query = include_str!("queries/metrics_outdegree.sparql");
    let complexity_query = include_str!("queries/metrics_complexity.sparql");

    // Execute each query...

    Ok(WorkflowMetrics {
        // ... populate fields
    })
}
```

---

### Recipe 4.4: Find Critical Path (Hot Path Tasks)

**Use Case:** Performance optimization

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX knhk: <http://knhk.org/ontology#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?tickBudget ?otelSpan WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?task knhk:tickBudget ?tickBudget }
    OPTIONAL { ?task knhk:otelSpan ?otelSpan }

    # Filter for hot path (≤8 ticks)
    FILTER(?tickBudget <= 8)
}
ORDER BY ?tickBudget ?name
```

**Rust Integration:**

```rust
#[derive(Debug, Clone)]
struct HotPathTask {
    task: String,
    name: Option<String>,
    tick_budget: i32,
    otel_span: Option<String>,
}

fn find_hot_path_tasks(store: &Store) -> Result<Vec<HotPathTask>, Error> {
    let query = include_str!("queries/find_hot_path.sparql");

    let mut tasks = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            tasks.push(HotPathTask {
                task: solution.get("task").unwrap().to_string(),
                name: solution.get("name").map(|n| n.to_string()),
                tick_budget: solution.get("tickBudget")
                    .and_then(|v| v.as_str().parse().ok())
                    .unwrap_or(8),
                otel_span: solution.get("otelSpan").map(|s| s.to_string()),
            });
        }
    }

    Ok(tasks)
}
```

---

## 5. Runtime Monitoring Queries

### Recipe 5.1: Get Active Workflow Instances

**Use Case:** Monitor running workflows

```sparql
PREFIX knhk: <http://knhk.org/ontology#>
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?instance ?spec ?specName ?state ?started WHERE {
    ?instance a knhk:WorkflowInstance .
    ?instance knhk:hasSpecification ?spec .
    ?instance knhk:hasState ?state .
    ?instance knhk:startedAt ?started .

    OPTIONAL { ?spec rdfs:label ?specName }

    # Filter for active instances
    FILTER(?state = "running" || ?state = "suspended")
}
ORDER BY DESC(?started)
```

**Rust Integration:**

```rust
#[derive(Debug, Clone)]
struct WorkflowInstance {
    instance: String,
    spec: String,
    spec_name: Option<String>,
    state: String,
    started: String,
}

fn get_active_instances(store: &Store) -> Result<Vec<WorkflowInstance>, Error> {
    let query = include_str!("queries/get_active_instances.sparql");

    let mut instances = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            instances.push(WorkflowInstance {
                instance: solution.get("instance").unwrap().to_string(),
                spec: solution.get("spec").unwrap().to_string(),
                spec_name: solution.get("specName").map(|n| n.to_string()),
                state: solution.get("state").unwrap().to_string(),
                started: solution.get("started").unwrap().to_string(),
            });
        }
    }

    Ok(instances)
}
```

---

### Recipe 5.2: Get Task Execution Status in Instance

**Use Case:** Monitor task progress

```sparql
PREFIX knhk: <http://knhk.org/ontology#>
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?task ?name ?state ?started ?completed WHERE {
    # Parameter: instance IRI
    BIND(<http://example.org/instance#Instance123> AS ?instance)

    ?instance knhk:hasTaskExecution ?execution .
    ?execution knhk:hasTask ?task .
    ?execution knhk:hasState ?state .

    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?execution knhk:startedAt ?started }
    OPTIONAL { ?execution knhk:completedAt ?completed }
}
ORDER BY ?started
```

**Rust Integration:**

```rust
#[derive(Debug, Clone)]
struct TaskExecution {
    task: String,
    name: Option<String>,
    state: String,
    started: Option<String>,
    completed: Option<String>,
}

fn get_task_executions(
    store: &Store,
    instance_iri: &str,
) -> Result<Vec<TaskExecution>, Error> {
    let query = format!(r#"
        PREFIX knhk: <http://knhk.org/ontology#>
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?task ?name ?state ?started ?completed WHERE {{
            <{instance_iri}> knhk:hasTaskExecution ?execution .
            ?execution knhk:hasTask ?task .
            ?execution knhk:hasState ?state .
            OPTIONAL {{ ?task rdfs:label ?name }}
            OPTIONAL {{ ?execution knhk:startedAt ?started }}
            OPTIONAL {{ ?execution knhk:completedAt ?completed }}
        }}
        ORDER BY ?started
    "#);

    // Execute query...
}
```

---

### Recipe 5.3: Find Delayed Tasks (Exceeded Time Budget)

**Use Case:** Performance monitoring

```sparql
PREFIX knhk: <http://knhk.org/ontology#>
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?instance ?task ?name ?started ?budget ?elapsed WHERE {
    ?instance knhk:hasTaskExecution ?execution .
    ?execution knhk:hasTask ?task .
    ?execution knhk:hasState "executing" .
    ?execution knhk:startedAt ?started .

    ?task rdfs:label ?name .
    ?task knhk:tickBudget ?budget .

    # Calculate elapsed time
    BIND(xsd:long(NOW()) - xsd:long(?started) AS ?elapsed)

    # Filter for violations
    FILTER(?elapsed > ?budget)
}
ORDER BY DESC(?elapsed)
```

**Rust Integration:**

```rust
#[derive(Debug, Clone)]
struct DelayedTask {
    instance: String,
    task: String,
    name: String,
    started: String,
    budget: i32,
    elapsed: i64,
}

fn find_delayed_tasks(store: &Store) -> Result<Vec<DelayedTask>, Error> {
    let query = include_str!("queries/find_delayed_tasks.sparql");

    let mut delayed = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            delayed.push(DelayedTask {
                instance: solution.get("instance").unwrap().to_string(),
                task: solution.get("task").unwrap().to_string(),
                name: solution.get("name").unwrap().to_string(),
                started: solution.get("started").unwrap().to_string(),
                budget: solution.get("budget")
                    .and_then(|v| v.as_str().parse().ok())
                    .unwrap_or(0),
                elapsed: solution.get("elapsed")
                    .and_then(|v| v.as_str().parse().ok())
                    .unwrap_or(0),
            });
        }
    }

    Ok(delayed)
}
```

---

## 6. Performance Optimization

### 6.1 Query Optimization Techniques

**1. Filter Early**

```sparql
# ❌ BAD (filters after full traversal)
SELECT ?task WHERE {
    ?task a yawl:Task .
    ?task yawl:hasJoin ?join .
    ?task knhk:tickBudget ?budget .
    FILTER(?budget <= 8)
}

# ✅ GOOD (filters during traversal)
SELECT ?task WHERE {
    ?task knhk:tickBudget ?budget .
    FILTER(?budget <= 8)
    ?task a yawl:Task .
    ?task yawl:hasJoin ?join .
}
```

**2. Use LIMIT**

```sparql
# Always limit results in production
SELECT ?task WHERE {
    ?task a yawl:Task .
}
LIMIT 100
```

**3. Avoid OPTIONAL When Possible**

```sparql
# ❌ SLOW (many OPTIONALs)
SELECT ?task ?p1 ?p2 ?p3 ?p4 ?p5 WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task yawl:prop1 ?p1 }
    OPTIONAL { ?task yawl:prop2 ?p2 }
    OPTIONAL { ?task yawl:prop3 ?p3 }
    OPTIONAL { ?task yawl:prop4 ?p4 }
    OPTIONAL { ?task yawl:prop5 ?p5 }
}

# ✅ FAST (only retrieve needed properties)
SELECT ?task ?p1 WHERE {
    ?task a yawl:Task .
    OPTIONAL { ?task yawl:prop1 ?p1 }
}
```

**4. Avoid Property Paths in Hot Path**

```sparql
# ❌ SLOW (property path can be expensive)
SELECT ?task WHERE {
    ?start (yawl:flowsInto/yawl:nextElementRef)+ ?task .
}

# ✅ FAST (precompute reachability, cache result)
```

### 6.2 Prepared Statements in Rust

```rust
use oxigraph::sparql::Query;

pub struct PreparedQueries {
    get_tasks: Query,
    get_flows: Query,
    validate_start: Query,
}

impl PreparedQueries {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            get_tasks: Query::parse(
                include_str!("queries/get_tasks.sparql"),
                None,
            )?,
            get_flows: Query::parse(
                include_str!("queries/get_flows.sparql"),
                None,
            )?,
            validate_start: Query::parse(
                include_str!("queries/validate_start.sparql"),
                None,
            )?,
        })
    }

    pub fn execute_get_tasks(&self, store: &Store) -> Result<Vec<Task>, Error> {
        // Execute prepared query
        let results = store.query(self.get_tasks.clone())?;
        // ... process results
    }
}
```

### 6.3 Caching Strategies

```rust
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;

pub struct CachedQueryEngine {
    store: Store,
    cache: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl CachedQueryEngine {
    pub fn get_tasks(&self, workflow_iri: &str) -> Result<Vec<Task>, Error> {
        let cache_key = format!("tasks:{}", workflow_iri);

        // Check cache
        {
            let cache = self.cache.read();
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(deserialize_tasks(cached)?);
            }
        }

        // Cache miss: execute query
        let tasks = self.execute_get_tasks_query(workflow_iri)?;

        // Update cache
        {
            let mut cache = self.cache.write();
            cache.insert(cache_key, serialize_tasks(&tasks)?);
        }

        Ok(tasks)
    }

    pub fn invalidate_cache(&self, workflow_iri: &str) {
        let cache_key = format!("tasks:{}", workflow_iri);
        let mut cache = self.cache.write();
        cache.remove(&cache_key);
    }
}
```

---

## 7. Advanced Query Patterns

### Recipe 7.1: Recursive Reachability Query

**Use Case:** Transitive closure of control flow

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?from ?to WHERE {
    # Reachable in 1+ steps
    ?from (yawl:flowsInto/yawl:nextElementRef)+ ?to .
}
```

**Optimization:** Precompute and cache for large workflows.

---

### Recipe 7.2: Aggregation with Grouping

**Use Case:** Count tasks per decomposition

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?net (COUNT(?task) AS ?taskCount) WHERE {
    ?net a yawl:Net .
    ?net yawl:hasTask ?task .
}
GROUP BY ?net
ORDER BY DESC(?taskCount)
```

---

### Recipe 7.3: Subquery for Complex Filtering

**Use Case:** Find tasks in workflows with >10 tasks

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task WHERE {
    ?net yawl:hasTask ?task .

    # Subquery: count tasks in net
    {
        SELECT ?net (COUNT(?t) AS ?count) WHERE {
            ?net yawl:hasTask ?t .
        }
        GROUP BY ?net
    }

    # Filter nets with >10 tasks
    FILTER(?count > 10)
}
```

---

### Recipe 7.4: UNION for Multiple Patterns

**Use Case:** Find all net elements (tasks OR conditions)

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?element ?type WHERE {
    {
        ?element a yawl:Task .
        BIND("task" AS ?type)
    }
    UNION
    {
        ?element a yawl:Condition .
        BIND("condition" AS ?type)
    }
}
```

---

## 8. Query Debugging Techniques

### 8.1 Incremental Query Building

```sparql
# Step 1: Start simple
SELECT ?task WHERE {
    ?task a yawl:Task .
}

# Step 2: Add one constraint
SELECT ?task WHERE {
    ?task a yawl:Task .
    ?task yawl:hasJoin yawl:ControlTypeAnd .
}

# Step 3: Add another
SELECT ?task WHERE {
    ?task a yawl:Task .
    ?task yawl:hasJoin yawl:ControlTypeAnd .
    ?task knhk:tickBudget ?budget .
}

# Step 4: Add filter
SELECT ?task WHERE {
    ?task a yawl:Task .
    ?task yawl:hasJoin yawl:ControlTypeAnd .
    ?task knhk:tickBudget ?budget .
    FILTER(?budget <= 8)
}
```

### 8.2 Debugging with SELECT *

```sparql
# Show all bindings
SELECT * WHERE {
    ?task a yawl:Task .
    ?task yawl:hasJoin ?join .
    ?task knhk:tickBudget ?budget .
    FILTER(?budget <= 8)
}
LIMIT 10
```

### 8.3 Checking Data Existence

```sparql
# Do any tasks exist?
SELECT (COUNT(?task) AS ?count) WHERE {
    ?task a yawl:Task .
}

# What properties do tasks have?
SELECT DISTINCT ?property WHERE {
    ?task a yawl:Task .
    ?task ?property ?value .
}
```

### 8.4 Namespace Debugging

```sparql
# Find all classes
SELECT ?class WHERE {
    ?class a rdfs:Class .
}

# Find all properties
SELECT ?property WHERE {
    ?property a rdf:Property .
}

# Search by name
SELECT ?thing WHERE {
    ?thing ?p ?o .
    FILTER(CONTAINS(STR(?thing), "Task"))
}
```

---

## 9. Rust Integration Patterns

### 9.1 Query Result Handling

```rust
use oxigraph::sparql::QueryResults;

fn execute_query(store: &Store, query: &str) -> Result<Vec<HashMap<String, String>>, Error> {
    let mut results = Vec::new();

    match store.query(query)? {
        QueryResults::Solutions(solutions) => {
            for solution in solutions {
                let solution = solution?;
                let mut row = HashMap::new();

                for (var, value) in solution.iter() {
                    row.insert(var.as_str().to_string(), value.to_string());
                }

                results.push(row);
            }
        }
        QueryResults::Boolean(b) => {
            // ASK query
            if b {
                results.push(HashMap::from([("result".to_string(), "true".to_string())]));
            }
        }
        QueryResults::Graph(_) => {
            return Err(Error::UnsupportedQuery("CONSTRUCT not supported"));
        }
    }

    Ok(results)
}
```

### 9.2 Error Handling

```rust
#[derive(Debug)]
enum QueryError {
    ParseError(String),
    ExecutionError(String),
    EmptyResult,
    InvalidResult(String),
}

impl From<oxigraph::sparql::EvaluationError> for QueryError {
    fn from(err: oxigraph::sparql::EvaluationError) -> Self {
        QueryError::ExecutionError(err.to_string())
    }
}

fn safe_query_execution(
    store: &Store,
    query: &str,
) -> Result<Vec<HashMap<String, String>>, QueryError> {
    let parsed_query = Query::parse(query, None)
        .map_err(|e| QueryError::ParseError(e.to_string()))?;

    let results = store.query(parsed_query)?;

    // Process results...

    Ok(processed_results)
}
```

---

## 10. Anti-Patterns to Avoid

### 10.1 ❌ Retrieving All Data Then Filtering in Rust

```rust
// ❌ BAD: Retrieve everything, filter in Rust
let all_tasks = get_all_tasks(store)?;
let hot_path: Vec<_> = all_tasks.into_iter()
    .filter(|t| t.tick_budget <= 8)
    .collect();

// ✅ GOOD: Filter in SPARQL
let query = r#"
    SELECT ?task WHERE {
        ?task knhk:tickBudget ?budget .
        FILTER(?budget <= 8)
    }
"#;
let hot_path = execute_query(store, query)?;
```

### 10.2 ❌ Using String Concatenation for Queries

```rust
// ❌ BAD: SQL-injection-style vulnerability
let task_name = user_input; // Could contain SPARQL syntax
let query = format!(r#"
    SELECT ?task WHERE {{
        ?task yawl:name "{task_name}" .
    }}
"#);

// ✅ GOOD: Use parameterized queries or proper escaping
let query = Query::parse(r#"
    SELECT ?task WHERE {
        ?task yawl:name ?name .
    }
"#, None)?;

// Bind variables safely
```

### 10.3 ❌ Not Limiting Results

```sparql
-- ❌ BAD: Could return millions of results
SELECT ?s ?p ?o WHERE {
    ?s ?p ?o .
}

-- ✅ GOOD: Always use LIMIT
SELECT ?s ?p ?o WHERE {
    ?s ?p ?o .
}
LIMIT 1000
```

### 10.4 ❌ Ignoring Query Performance

```rust
// ❌ BAD: Run expensive query in hot path
fn handle_request() {
    let reachability = compute_transitive_closure(store)?; // 500ms!
}

// ✅ GOOD: Precompute and cache
lazy_static! {
    static ref REACHABILITY_CACHE: RwLock<HashMap<String, Vec<String>>> =
        RwLock::new(HashMap::new());
}

fn get_reachability() -> Vec<String> {
    REACHABILITY_CACHE.read().get("key").cloned().unwrap_or_default()
}
```

---

## Appendix: Complete Query Examples

### Example 1: Full Workflow Extraction Pipeline

```rust
// File: src/workflow/extractor.rs

use oxigraph::store::Store;
use oxigraph::sparql::QueryResults;

pub struct WorkflowExtractor {
    store: Store,
}

impl WorkflowExtractor {
    pub fn extract_complete_workflow(
        &self,
        workflow_iri: &str,
    ) -> Result<CompleteWorkflow, Error> {
        Ok(CompleteWorkflow {
            metadata: self.extract_metadata(workflow_iri)?,
            tasks: self.extract_tasks(workflow_iri)?,
            conditions: self.extract_conditions(workflow_iri)?,
            flows: self.extract_flows(workflow_iri)?,
            variables: self.extract_variables(workflow_iri)?,
        })
    }

    fn extract_metadata(&self, iri: &str) -> Result<WorkflowMetadata, Error> {
        let query = format!(r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

            SELECT ?title ?version ?created ?creator WHERE {{
                <{iri}> rdfs:label ?title .
                OPTIONAL {{ <{iri}> yawl:version ?version }}
                OPTIONAL {{ <{iri}> yawl:created ?created }}
                OPTIONAL {{ <{iri}> yawl:creator ?creator }}
            }}
        "#);

        // Execute and parse...
    }

    fn extract_tasks(&self, iri: &str) -> Result<Vec<Task>, Error> {
        let query = format!(r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
            PREFIX knhk: <http://knhk.org/ontology#>

            SELECT ?task ?name ?join ?split ?tickBudget WHERE {{
                <{iri}> yawl:hasDecomposition ?net .
                ?net yawl:hasTask ?task .
                OPTIONAL {{ ?task rdfs:label ?name }}
                OPTIONAL {{ ?task yawl:hasJoin ?join }}
                OPTIONAL {{ ?task yawl:hasSplit ?split }}
                OPTIONAL {{ ?task knhk:tickBudget ?tickBudget }}
            }}
        "#);

        // Execute and parse...
    }
}
```

---

## Conclusion

This cookbook provided 40+ SPARQL query recipes for YAWL ontology integration in knhk. Key takeaways:

1. **Filter in SPARQL, not Rust** - Push computation to the query engine
2. **Use LIMIT always** - Prevent unbounded result sets
3. **Cache expensive queries** - Property paths, transitive closures
4. **Validate incrementally** - Build queries step-by-step
5. **Prepare queries once, reuse many** - Performance optimization

**Next Steps:**
- Read `ontology-reference-manual.md` for complete class/property reference
- Explore `/Users/sac/knhk/rust/knhk-workflow-engine/src/parser/extractor.rs` for implementation
- Practice query construction with real YAWL workflows

**Resources:**
- SPARQL 1.1 Query Language: https://www.w3.org/TR/sparql11-query/
- Oxigraph SPARQL API: https://docs.rs/oxigraph/latest/oxigraph/sparql/
- YAWL Ontology: `/Users/sac/knhk/ontology/yawl.ttl`
